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
use crate::chpp::{create_oauth_context, retry_with_default_config, ChppClient, Error};
use crate::db::manager::DbManager;
use crate::db::schema::downloads;
use crate::db::teams::{save_players, save_team, save_world_details};
use crate::service::secret::{GnomeSecretService, SecretStorageService};
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, info};
use oauth_1a::{OAuthData, SigningKey};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait DataSyncService {
    fn perform_initial_sync(
        &self,
        consumer_key: String,
        consumer_secret: String,
        on_progress: Box<dyn Fn(f64, &str) + Send + Sync>,
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
        on_progress: Box<dyn Fn(f64, &str) + Send + Sync>,
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
                on_progress,
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
                    Box::new(|p, m| debug!("Background sync: {:.0}% - {}", p * 100.0, m)),
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

/// Merges player data from two sources: basic (from teamdetails) and detailed (from playerdetails).
///
/// Strategy:
/// - If detailed data is available, use it as the primary source
/// - Fill in any None fields in detailed data with values from basic data
/// - This ensures we capture all available information from both endpoints
///
/// Note: PlayerSkills are only available in playerdetails for own team,
/// so basic data will never have skills to contribute.
fn merge_player_data(
    basic: &crate::chpp::model::Player,
    detailed: Option<crate::chpp::model::Player>,
) -> crate::chpp::model::Player {
    match detailed {
        Some(mut d) => {
            // Use detailed as base, fill in missing fields from basic
            // Most fields should be present in detailed, but we check anyway

            // Basic identification (should always be in detailed)
            // PlayerID, FirstName, LastName are always present

            // Optional fields that might be missing in detailed but present in basic
            if d.PlayerNumber.is_none() && basic.PlayerNumber.is_some() {
                d.PlayerNumber = basic.PlayerNumber;
            }
            if d.AgeDays.is_none() && basic.AgeDays.is_some() {
                d.AgeDays = basic.AgeDays;
            }
            if d.Statement.is_none() && basic.Statement.is_some() {
                d.Statement = basic.Statement.clone();
            }
            if d.ReferencePlayerID.is_none() && basic.ReferencePlayerID.is_some() {
                d.ReferencePlayerID = basic.ReferencePlayerID;
            }
            if d.LeagueGoals.is_none() && basic.LeagueGoals.is_some() {
                d.LeagueGoals = basic.LeagueGoals;
            }
            if d.CupGoals.is_none() && basic.CupGoals.is_some() {
                d.CupGoals = basic.CupGoals;
            }
            if d.FriendliesGoals.is_none() && basic.FriendliesGoals.is_some() {
                d.FriendliesGoals = basic.FriendliesGoals;
            }
            if d.CareerGoals.is_none() && basic.CareerGoals.is_some() {
                d.CareerGoals = basic.CareerGoals;
            }
            if d.CareerHattricks.is_none() && basic.CareerHattricks.is_some() {
                d.CareerHattricks = basic.CareerHattricks;
            }
            if d.Speciality.is_none() && basic.Speciality.is_some() {
                d.Speciality = basic.Speciality;
            }
            if d.NationalTeamID.is_none() && basic.NationalTeamID.is_some() {
                d.NationalTeamID = basic.NationalTeamID;
            }
            if d.CountryID.is_none() && basic.CountryID.is_some() {
                d.CountryID = basic.CountryID;
            }
            // Set country ID to native country ID if country ID is not present.
            if d.CountryID.is_none() && d.NativeCountryID.is_some() {
                d.CountryID = d.NativeCountryID;
            }
            // National team stats
            if d.Caps.is_none() && basic.Caps.is_some() {
                d.Caps = basic.Caps;
            }
            if d.CapsU20.is_none() && basic.CapsU20.is_some() {
                d.CapsU20 = basic.CapsU20;
            }
            if d.Cards.is_none() && basic.Cards.is_some() {
                d.Cards = basic.Cards;
            }
            if d.InjuryLevel.is_none() && basic.InjuryLevel.is_some() {
                d.InjuryLevel = basic.InjuryLevel;
            }
            if d.Sticker.is_none() && basic.Sticker.is_some() {
                d.Sticker = basic.Sticker.clone();
            }
            if d.LastMatch.is_none() && basic.LastMatch.is_some() {
                d.LastMatch = basic.LastMatch.clone();
            }

            if d.ArrivalDate.is_none() && basic.ArrivalDate.is_some() {
                d.ArrivalDate = basic.ArrivalDate.clone();
            }
            if d.PlayerCategoryId.is_none() && basic.PlayerCategoryId.is_some() {
                d.PlayerCategoryId = basic.PlayerCategoryId;
            }
            if d.MotherClub.is_none() && basic.MotherClub.is_some() {
                d.MotherClub = basic.MotherClub.clone();
            }
            if d.NativeCountryID.is_none() && basic.NativeCountryID.is_some() {
                d.NativeCountryID = basic.NativeCountryID;
            }
            if d.NativeLeagueID.is_none() && basic.NativeLeagueID.is_some() {
                d.NativeLeagueID = basic.NativeLeagueID;
            }
            if d.NativeLeagueName.is_none() && basic.NativeLeagueName.is_some() {
                d.NativeLeagueName = basic.NativeLeagueName.clone();
            }
            if d.MatchesCurrentTeam.is_none() && basic.MatchesCurrentTeam.is_some() {
                d.MatchesCurrentTeam = basic.MatchesCurrentTeam;
            }
            if d.GoalsCurrentTeam.is_none() && basic.GoalsCurrentTeam.is_some() {
                d.GoalsCurrentTeam = basic.GoalsCurrentTeam;
            }
            if d.AssistsCurrentTeam.is_none() && basic.AssistsCurrentTeam.is_some() {
                d.AssistsCurrentTeam = basic.AssistsCurrentTeam;
            }
            if d.CareerAssists.is_none() && basic.CareerAssists.is_some() {
                d.CareerAssists = basic.CareerAssists;
            }

            d
        }
        None => {
            // No detailed data available, use basic data
            basic.clone()
        }
    }
}

impl SyncService {
    async fn create_download_record(db_manager: Arc<DbManager>) -> Result<i32, Error> {
        let db = db_manager.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| Error::Db(format!("Failed to get database connection: {}", e)))?;

            let timestamp = Utc::now().to_rfc3339();

            diesel::insert_into(downloads::table)
                .values((
                    downloads::timestamp.eq(&timestamp),
                    downloads::status.eq("in_progress"),
                ))
                .execute(&mut conn)
                .map_err(|e| Error::Db(format!("Failed to create download record: {}", e)))?;

            let id: i32 = downloads::table
                .select(downloads::id)
                .order(downloads::id.desc())
                .first(&mut conn)
                .map_err(|e| Error::Db(format!("Failed to get download ID: {}", e)))?;

            Ok(id)
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))?
    }

    async fn complete_download_record(
        db_manager: Arc<DbManager>,
        download_id: i32,
    ) -> Result<(), Error> {
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
        Ok(())
    }

    async fn fetch_and_save_user_data<F>(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        get_auth: &F,
        download_id: i32,
    ) -> Result<u32, Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
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

        Ok(team_id)
    }

    async fn fetch_and_save_world_details<F>(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        get_auth: &F,
        download_id: i32,
    ) -> Result<(), Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
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
            save_world_details(&mut conn, &wd, download_id)
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        Ok(())
    }

    async fn fetch_and_save_players<F>(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        get_auth: &F,
        team_id: u32,
        download_id: i32,
    ) -> Result<(), Error>
    where
        // Send is for concurrency, F safe to be sent to another thread, Sync means muliple threads can safely access
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        // Get Players for the team
        let (data, key) = get_auth();
        let players_resp = client.players(data, key, None).await?;

        let player_list = if let Some(pl) = players_resp.Team.PlayerList {
            pl
        } else {
            log::warn!("No player list found for team");
            return Err(Error::Parse("No player list in response".to_string()));
        };

        // Fetch detailed player data for each player and merge with basic data
        let merged_players = {
            info!(
                "Fetching detailed player data for {} players",
                player_list.players.len()
            );

            let mut merged_players = Vec::new();
            for basic_player in &player_list.players {
                info!("Fetching details for player ID: {}", basic_player.PlayerID);

                let player_id = basic_player.PlayerID;
                let operation_name = format!("player_details({})", player_id);

                // Use retry utility for player details fetching
                let result = retry_with_default_config(&operation_name, get_auth, |data, key| {
                    client.player_details(data, key, player_id)
                })
                .await;

                // Merge detailed data with basic data
                let merged = match result {
                    Ok(detailed_player) => {
                        debug!(
                            "Successfully fetched detailed data for player {}",
                            player_id
                        );
                        merge_player_data(basic_player, Some(detailed_player))
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to fetch details for player {}: {}. Using basic data only.",
                            player_id,
                            e
                        );
                        merge_player_data(basic_player, None)
                    }
                };

                merged_players.push(merged);
            }
            merged_players
        };

        // Save players
        let db = db_manager.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            save_players(&mut conn, &merged_players, team_id, download_id)
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        Ok(())
    }

    async fn do_sync(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        secret_service: Arc<dyn SecretStorageService>,
        consumer_key: String,
        consumer_secret: String,
        on_progress: Box<dyn Fn(f64, &str) + Send + Sync>,
    ) -> Result<(), Error> {
        on_progress(0.0, "Checking credentials...");
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
        debug!("access_token: {}", access_token);
        debug!("access_secret: {}", access_secret);

        // Helper to get fresh auth data
        let get_auth = || {
            create_oauth_context(
                &consumer_key,
                &consumer_secret,
                &access_token,
                &access_secret,
            )
        };

        on_progress(0.05, "Creating download record...");
        let download_id = Self::create_download_record(db_manager.clone()).await?;

        on_progress(0.1, "Fetching user data...");
        let team_id = Self::fetch_and_save_user_data(
            db_manager.clone(),
            client.clone(),
            &get_auth,
            download_id,
        )
        .await?;

        on_progress(0.3, "Fetching world details (leagues, currency)...");
        Self::fetch_and_save_world_details(
            db_manager.clone(),
            client.clone(),
            &get_auth,
            download_id,
        )
        .await?;

        on_progress(0.6, "Fetching players...");
        Self::fetch_and_save_players(
            db_manager.clone(),
            client.clone(),
            &get_auth,
            team_id,
            download_id,
        )
        .await?;

        on_progress(0.9, "Finalizing download...");
        Self::complete_download_record(db_manager.clone(), download_id).await?;

        on_progress(1.0, "Done.");
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
                        ShortName: None,
                        Continent: None,
                        Season: None,
                        SeasonOffset: None,
                        MatchRound: None,
                        ZoneName: None,
                        EnglishName: None,
                        LanguageID: None,
                        LanguageName: None,
                        NationalTeamId: None,
                        U20TeamId: None,
                        ActiveTeams: None,
                        ActiveUsers: None,
                        NumberOfLevels: None,
                        Country: WorldCountry {
                            CountryID: Some(10),
                            CountryName: Some("TestCountry".to_string()),
                            CurrencyName: Some("TestCurrency".to_string()),
                            CurrencyRate: Some("1,0".to_string()),
                            CountryCode: Some("TC".to_string()),
                            DateFormat: None,
                            TimeFormat: None,
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
                            Flag: None,
                            PlayerSkills: None,
                            LastMatch: None,
                            ArrivalDate: None,
                            PlayerCategoryId: None,
                            MotherClub: None,
                            NativeCountryID: None,
                            NativeLeagueID: None,
                            NativeLeagueName: None,
                            MatchesCurrentTeam: None,
                            GoalsCurrentTeam: None,
                            AssistsCurrentTeam: None,
                            CareerAssists: None,
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
                Flag: None,
                PlayerSkills: None,
                LastMatch: None,
                ArrivalDate: None,
                PlayerCategoryId: None,
                MotherClub: None,
                NativeCountryID: None,
                NativeLeagueID: None,
                NativeLeagueName: None,
                MatchesCurrentTeam: None,
                GoalsCurrentTeam: None,
                AssistsCurrentTeam: None,
                CareerAssists: None,
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
            .perform_initial_sync(
                "dummy_key".into(),
                "dummy_secret".into(),
                Box::new(|_, _| {}),
            )
            .await;

        assert!(res.is_ok(), "Sync failed: {:?}", res.err());

        // Verify DB calls
        let has_users = db_manager.has_users().expect("Failed to check users");
        assert!(has_users, "User should satisfy DB check");

        // Could verify more details here if needed, like specific data presence
    }

    #[test]
    fn test_merge_player_data_with_detailed() {
        use crate::chpp::model::Player;

        // Create basic player with some fields
        let basic = Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            PlayerNumber: Some(10),
            Age: 25,
            AgeDays: Some(100),
            TSI: 1000,
            PlayerForm: 5,
            Statement: Some("Basic statement".to_string()),
            Experience: 3,
            Loyalty: 10,
            ReferencePlayerID: Some(999),
            MotherClubBonus: false,
            Leadership: 3,
            Salary: 500,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: Some(5),
            CupGoals: Some(2),
            FriendliesGoals: Some(1),
            CareerGoals: Some(50),
            CareerHattricks: Some(2),
            Speciality: Some(1),
            TransferListed: false,
            NationalTeamID: Some(100),
            CountryID: Some(10),
            Caps: Some(5),
            CapsU20: Some(10),
            Cards: Some(1),
            InjuryLevel: Some(-1),
            Sticker: Some("Basic sticker".to_string()),
            Flag: None,
            PlayerSkills: None,
            LastMatch: None,
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            CareerAssists: None,
        };

        // Create detailed player with most fields but some missing
        let detailed = Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            PlayerNumber: None, // Missing in detailed
            Age: 25,
            AgeDays: None, // Missing in detailed
            TSI: 1500,     // Different value
            PlayerForm: 6, // Different value
            Statement: Some("Detailed statement".to_string()),
            Experience: 4,
            Loyalty: 11,
            ReferencePlayerID: None, // Missing in detailed
            MotherClubBonus: false,
            Leadership: 4,
            Salary: 600,
            IsAbroad: false,
            Agreeability: 4,
            Aggressiveness: 4,
            Honesty: 4,
            LeagueGoals: Some(6),
            CupGoals: None, // Missing in detailed
            FriendliesGoals: Some(2),
            CareerGoals: Some(55),
            CareerHattricks: None, // Missing in detailed
            Speciality: Some(1),
            TransferListed: false,
            NationalTeamID: Some(100),
            CountryID: Some(10),
            Caps: Some(6),
            CapsU20: None, // Missing in detailed
            Cards: Some(1),
            InjuryLevel: Some(0),
            Sticker: None, // Missing in detailed
            Flag: None,
            PlayerSkills: Some(crate::chpp::model::PlayerSkills {
                StaminaSkill: 7,
                KeeperSkill: 1,
                PlaymakerSkill: 5,
                ScorerSkill: 6,
                PassingSkill: 5,
                WingerSkill: 4,
                DefenderSkill: 3,
                SetPiecesSkill: 4,
            }),
            LastMatch: None,
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            CareerAssists: None,
        };

        let merged = super::merge_player_data(&basic, Some(detailed));

        // Verify detailed data is primary
        assert_eq!(merged.TSI, 1500);
        assert_eq!(merged.PlayerForm, 6);
        assert_eq!(merged.Statement, Some("Detailed statement".to_string()));
        assert!(merged.PlayerSkills.is_some());

        // Verify missing fields filled from basic
        assert_eq!(merged.PlayerNumber, Some(10)); // From basic
        assert_eq!(merged.AgeDays, Some(100)); // From basic
        assert_eq!(merged.ReferencePlayerID, Some(999)); // From basic
        assert_eq!(merged.CupGoals, Some(2)); // From basic
        assert_eq!(merged.CareerHattricks, Some(2)); // From basic
        assert_eq!(merged.CapsU20, Some(10)); // From basic
        assert_eq!(merged.Sticker, Some("Basic sticker".to_string())); // From basic
    }

    #[test]
    fn test_merge_player_data_without_detailed() {
        use crate::chpp::model::Player;

        let basic = Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            PlayerNumber: Some(10),
            Age: 25,
            AgeDays: Some(100),
            TSI: 1000,
            PlayerForm: 5,
            Statement: Some("Basic statement".to_string()),
            Experience: 3,
            Loyalty: 10,
            ReferencePlayerID: Some(999),
            MotherClubBonus: false,
            Leadership: 3,
            Salary: 500,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: Some(5),
            CupGoals: Some(2),
            FriendliesGoals: Some(1),
            CareerGoals: Some(50),
            CareerHattricks: Some(2),
            Speciality: Some(1),
            TransferListed: false,
            NationalTeamID: Some(100),
            CountryID: Some(10),
            Caps: Some(5),
            CapsU20: Some(10),
            Cards: Some(1),
            InjuryLevel: Some(-1),
            Sticker: Some("Basic sticker".to_string()),
            Flag: None,
            PlayerSkills: None,
            LastMatch: None,
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            CareerAssists: None,
        };

        let merged = super::merge_player_data(&basic, None);

        // Should be identical to basic
        assert_eq!(merged.PlayerID, basic.PlayerID);
        assert_eq!(merged.TSI, basic.TSI);
        assert_eq!(merged.PlayerNumber, basic.PlayerNumber);
        assert_eq!(merged.Statement, basic.Statement);
        assert!(merged.PlayerSkills.is_none());
    }
}
