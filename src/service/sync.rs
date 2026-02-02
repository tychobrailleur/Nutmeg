/* sync.rs
 *
 * Copyright 2026 sebastien
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::client::HattrickClient;
use crate::chpp::{self, create_oauth_context, retry_with_default_config, ChppClient, Error};
use crate::db::manager::DbManager;
use crate::db::schema::downloads;
use crate::db::teams::{save_players, save_team, save_world_details};
use crate::service::secret::{GnomeSecretService, SecretStorageService};
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, info};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait DataSyncService {
    fn perform_initial_sync(
        &self,
        consumer_key: String,
        consumer_secret: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;

    fn perform_sync_with_stored_secrets(
        &self,
        consumer_key: String,
        consumer_secret: String,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send + '_>>;
}

pub struct SyncService {
    db_manager: Arc<DbManager>,
    client: Arc<dyn ChppClient>,
    secret_service: Arc<dyn SecretStorageService>,
}

impl SyncService {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self {
            db_manager,
            client: Arc::new(HattrickClient::new()),
            secret_service: Arc::new(GnomeSecretService::new()),
        }
    }

    // For testing
    pub fn new_with_client(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        secret_service: Arc<dyn SecretStorageService>,
    ) -> Self {
        Self {
            db_manager,
            client,
            secret_service,
        }
    }
}

impl DataSyncService for SyncService {
    fn perform_initial_sync(
        &self,
        consumer_key: String,
        consumer_secret: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let consumer_key = consumer_key.clone();
        let consumer_secret = consumer_secret.clone();
        let db_manager = self.db_manager.clone();
        let client = self.client.clone();
        let secret_service = self.secret_service.clone();

        Box::pin(async move {
            Self::do_sync(
                db_manager,
                client,
                secret_service,
                consumer_key,
                consumer_secret,
            )
            .await
        })
    }

    fn perform_sync_with_stored_secrets(
        &self,
        consumer_key: String,
        consumer_secret: String,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send + '_>> {
        let consumer_key = consumer_key.clone();
        let consumer_secret = consumer_secret.clone();
        let db_manager = self.db_manager.clone();
        let client = self.client.clone();
        let secret_service = self.secret_service.clone();

        Box::pin(async move {
            // Check if secrets exist first to return boolean
            let token_exists = secret_service.get_secret("access_token").await.is_ok();
            let secret_exists = secret_service.get_secret("access_secret").await.is_ok();

            if token_exists && secret_exists {
                match Self::do_sync(
                    db_manager,
                    client,
                    secret_service,
                    consumer_key,
                    consumer_secret,
                )
                .await
                {
                    Ok(_) => Ok(true),
                    Err(Error::Io(s)) if s.contains("Missing credentials") => Ok(false),
                    Err(e) => Err(e),
                }
            } else {
                Ok(false)
            }
        })
    }
}

impl SyncService {
    async fn do_sync(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        secret_service: Arc<dyn SecretStorageService>,
        consumer_key: String,
        consumer_secret: String,
    ) -> Result<(), Error> {
        let access_token = secret_service
            .get_secret("access_token")
            .await
            .map_err(|e| Error::Io(e.to_string()))?
            .ok_or(Error::Io("Missing credentials (token)".to_string()))?;

        let access_secret = secret_service
            .get_secret("access_secret")
            .await
            .map_err(|e| Error::Io(e.to_string()))?
            .ok_or(Error::Io("Missing credentials (secret)".to_string()))?;

        debug!("consumer_key: {}", consumer_key);
        debug!("consumer_secret: {}", consumer_secret);
        // Tokens are sensitive, maybe don't log them directly in production
        // debug!("access_token: {}", access_token);
        // debug!("access_secret: {}", access_secret);

        // Helper to get fresh auth data
        let get_auth = || {
            create_oauth_context(
                &consumer_key,
                &consumer_secret,
                &access_token,
                &access_secret,
            )
        };

        // Create Download Record
        let db = db_manager.clone();
        let download_id = {
            let conn = &mut db_manager
                .get_connection()
                .map_err(|e| Error::Db(format!("Failed to get database connection: {}", e)))?;

            let timestamp = Utc::now().to_rfc3339();

            diesel::insert_into(downloads::table)
                .values((
                    downloads::timestamp.eq(&timestamp),
                    downloads::status.eq("in_progress"),
                ))
                .execute(conn)
                .map_err(|e| Error::Db(format!("Failed to create download record: {}", e)))?;

            let id: i32 = downloads::table
                .select(downloads::id)
                .order(downloads::id.desc())
                .first(conn)
                .map_err(|e| Error::Db(format!("Failed to get download ID: {}", e)))?;

            id
        };

        // Get user / team details
        let (data, key) = get_auth();
        let hattrick_data = client.team_details(data, key, None).await?;

        log::info!(
            "User: {} ({})",
            hattrick_data.User.Name,
            hattrick_data.User.Loginname
        );

        // Save user and teams
        let user = hattrick_data.User;
        let teams = hattrick_data.Teams.Teams;

        //Extract first team ID for player fetching (simplified - just using first team)
        let team_id: u32 = teams
            .first()
            .and_then(|t| t.TeamID.parse().ok())
            .unwrap_or(0);

        log::info!("Processing teams, found {} team(s)", teams.len());

        let db = db_manager.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            for team in &teams {
                log::info!("Saving team: {} ({})", team.TeamName, team.TeamID);
                save_team(&mut conn, team, &user, download_id)?;
            }
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        // Get World Details
        let (data, key) = get_auth();
        let world_details = client.world_details(data, key).await?;
        log::info!(
            "Fetched world details with {} leagues",
            world_details.LeagueList.Leagues.len()
        );

        // Save world details
        let db = db_manager.clone();
        let wd = world_details;
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            save_world_details(&mut conn, &wd)
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        // Get Players for the team
        let (data, key) = get_auth();
        let players_resp = client.players(data, key, None).await?;

        let player_list = if let Some(pl) = players_resp.Team.PlayerList {
            pl
        } else {
            log::warn!("No player list found for team");
            return Err(Error::Parse("No player list in response".to_string()));
        };

        // Fetch detailed player data for each player
        // Use retry utility from the integration layer
        let detailed_players = {
            info!(
                "Fetching detailed player data for {} players",
                player_list.players.len()
            );

            let mut detailed_players = Vec::new();
            for basic_player in &player_list.players {
                info!("Fetching details for player ID: {}", basic_player.PlayerID);

                let player_id = basic_player.PlayerID;
                let operation_name = format!("player_details({})", player_id);

                // Use retry utility for player details fetching
                let result = retry_with_default_config(&operation_name, &get_auth, |data, key| {
                    client.player_details(data, key, player_id)
                })
                .await;

                match result {
                    Ok(detailed_player) => {
                        detailed_players.push(detailed_player);
                    }
                    Err(e) => {
                        // Retries have already been attempted by retry utility
                        log::warn!(
                            "Failed to fetch details for player {}: {}. Falling back to basic data.",
                            player_id,
                            e
                        );
                        detailed_players.push(basic_player.clone());
                    }
                }
            }
            detailed_players
        };

        // Save players
        let db = db_manager.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            save_players(&mut conn, &detailed_players, team_id, download_id)
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        // Mark download as completed
        let db = db_manager.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            use crate::db::schema::downloads::dsl::*;

            diesel::update(downloads.filter(id.eq(download_id)))
                .set(status.eq("completed"))
                .execute(&mut conn)
                .map_err(|e| Error::Io(format!("Failed to update download status: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        info!("Download {} completed successfully", download_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::*;
    use crate::db::manager::DbManager;
    use async_trait::async_trait;
    use oauth_1a::{OAuthData, SigningKey};

    use crate::service::secret::MockSecretService;

    struct MockChppClient;

    #[async_trait]
    impl ChppClient for MockChppClient {
        async fn world_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
        ) -> Result<WorldDetails, Error> {
            Ok(WorldDetails {
                LeagueList: WorldLeagueList {
                    Leagues: vec![WorldLeague {
                        LeagueID: 100,
                        LeagueName: "TestLeague".to_string(),
                        Country: WorldCountry {
                            CountryID: Some(10),
                            CountryName: Some("TestCountry".to_string()),
                            CurrencyName: Some("TestCurrency".to_string()),
                            CurrencyRate: Some("1,0".to_string()),
                            CountryCode: Some("TC".to_string()),
                        },
                    }],
                },
            })
        }

        async fn team_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<HattrickData, Error> {
            Ok(HattrickData {
                User: User {
                    UserID: 12345,
                    Name: "Test User".to_string(),
                    Loginname: "testuser".to_string(),
                    SignupDate: "2000-01-01 00:00:00".to_string(),
                    ActivationDate: "2000-01-01 00:00:00".to_string(),
                    LastLoginDate: "2020-01-01 00:00:00".to_string(),
                    HasManagerLicense: false,
                    SupporterTier: SupporterTier::None,
                    Language: Language {
                        LanguageID: 2,
                        LanguageName: "English".to_string(),
                    },
                },
                Teams: Teams {
                    Teams: vec![Team {
                        TeamID: "54321".to_string(),
                        TeamName: "Test Team".to_string(),
                        ShortTeamName: Some("TT".to_string()),
                        IsPrimaryClub: Some(true),
                        FoundedDate: None,
                        IsDeactivated: None,
                        Arena: None,
                        League: None,
                        Country: None,
                        Region: None,
                        Trainer: None,
                        HomePage: None,
                        Cup: None,
                        PowerRating: None,
                        FriendlyTeamID: None,
                        LeagueLevelUnit: None,
                        NumberOfVictories: None,
                        NumberOfUndefeated: None,
                        Fanclub: None,
                        LogoURL: None,
                        TeamColors: None,
                        DressURI: None,
                        DressAlternateURI: None,
                        BotStatus: None,
                        TeamRank: None,
                        YouthTeamID: None,
                        YouthTeamName: None,
                        NumberOfVisits: None,
                        //                      TrophyList: None,
                        PlayerList: None,
                        PossibleToChallengeMidweek: None,
                        PossibleToChallengeWeekend: None,
                    }],
                },
            })
        }

        async fn players(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<PlayersData, Error> {
            Ok(PlayersData {
                Team: Team {
                    TeamID: "123".to_string(),
                    TeamName: "Test FC".to_string(),
                    ShortTeamName: Some("TFC".to_string()),
                    IsPrimaryClub: Some(true),
                    FoundedDate: None,
                    IsDeactivated: None,
                    Arena: None,
                    League: None,
                    Country: None,
                    Region: None,
                    Trainer: None,
                    HomePage: None,
                    Cup: None,
                    PowerRating: None,
                    FriendlyTeamID: None,
                    LeagueLevelUnit: None,
                    NumberOfVictories: None,
                    NumberOfUndefeated: None,
                    Fanclub: None,
                    LogoURL: None,
                    TeamColors: None,
                    DressURI: None,
                    DressAlternateURI: None,
                    BotStatus: None,
                    TeamRank: None,
                    YouthTeamID: None,
                    YouthTeamName: None,
                    NumberOfVisits: None,
                    //               TrophyList: None,
                    PlayerList: Some(PlayerList {
                        players: vec![Player {
                            PlayerID: 1000,
                            FirstName: "Test".to_string(),
                            LastName: "Player".to_string(),
                            PlayerNumber: Some(10),
                            Age: 20,
                            AgeDays: Some(100),
                            TSI: 1000,
                            PlayerForm: 5,
                            Statement: None,
                            Experience: 3,
                            Loyalty: 10,
                            ReferencePlayerID: None,
                            MotherClubBonus: false,
                            Leadership: 3,
                            Salary: 500,
                            IsAbroad: false,
                            Agreeability: 3,
                            Aggressiveness: 3,
                            Honesty: 3,
                            LeagueGoals: Some(0),
                            CupGoals: Some(0),
                            FriendliesGoals: Some(0),
                            CareerGoals: Some(0),
                            CareerHattricks: Some(0),
                            Speciality: Some(0),
                            TransferListed: false,
                            NationalTeamID: None,
                            CountryID: Some(10),
                            Caps: Some(0),
                            CapsU20: Some(0),
                            Cards: Some(0),
                            InjuryLevel: Some(-1),
                            Sticker: None,
                            PlayerSkills: None,
                            LastMatch: None,
                        }],
                    }),
                    PossibleToChallengeMidweek: None,
                    PossibleToChallengeWeekend: None,
                },
            })
        }

        async fn player_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _player_id: u32,
        ) -> Result<Player, Error> {
            Ok(Player {
                PlayerID: 1001,
                FirstName: "John".to_string(),
                LastName: "Doe".to_string(),
                PlayerNumber: Some(10),
                Age: 20,
                AgeDays: Some(50),
                TSI: 1000,
                PlayerForm: 5,
                Statement: Some("".to_string()),
                Experience: 3,
                Loyalty: 10,
                ReferencePlayerID: None,
                MotherClubBonus: false,
                Leadership: 3,
                Salary: 500,
                IsAbroad: false,
                Agreeability: 3,
                Aggressiveness: 3,
                Honesty: 3,
                LeagueGoals: Some(0),
                CupGoals: Some(0),
                FriendliesGoals: Some(0),
                CareerGoals: Some(0),
                CareerHattricks: Some(0),
                Speciality: Some(0),
                TransferListed: false,
                NationalTeamID: Some(0),
                CountryID: Some(10),
                Caps: Some(0),
                CapsU20: Some(0),
                Cards: Some(0),
                InjuryLevel: Some(-1),
                Sticker: Some("".to_string()),
                PlayerSkills: None,
                LastMatch: None,
            })
        }
    }

    #[tokio::test]
    async fn test_perform_initial_sync() {
        let db_manager = Arc::new(DbManager::from_url(":memory:"));
        db_manager.run_migrations().expect("Migrations failed");

        let client = Arc::new(MockChppClient);
        let secret_service = Arc::new(MockSecretService::new());

        // Seed some fake secrets
        secret_service
            .store_secret("access_token", "dummy_token")
            .await
            .expect("Failed to store token");
        secret_service
            .store_secret("access_secret", "dummy_secret")
            .await
            .expect("Failed to store secret");

        let service = SyncService::new_with_client(db_manager.clone(), client, secret_service);

        let res = service
            .perform_initial_sync("dummy_key".into(), "dummy_secret".into())
            .await;

        assert!(res.is_ok(), "Sync failed: {:?}", res.err());

        // Verify DB calls
        let has_users = db_manager.has_users().expect("Failed to check users");
        assert!(has_users, "User should satisfy DB check");

        // Could verify more details here if needed, like specific data presence
    }
}
