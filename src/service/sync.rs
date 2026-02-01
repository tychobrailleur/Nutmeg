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

use crate::chpp::error::Error;
use crate::chpp::oauth::{OAuthData, SigningKey};

use crate::db::manager::DbManager;
use crate::db::teams::{save_players, save_team, save_world_details};
use log::{debug, info};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait DataSyncService {
    fn perform_initial_sync(
        &self,
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_secret: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;
}

use crate::chpp::client::{ChppClient, HattrickClient};

pub struct SyncService {
    db_manager: Arc<DbManager>,
    client: Arc<dyn ChppClient>,
}

impl SyncService {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self {
            db_manager,
            client: Arc::new(HattrickClient::new()),
        }
    }

    // For testing
    pub fn new_with_client(db_manager: Arc<DbManager>, client: Arc<dyn ChppClient>) -> Self {
        Self { db_manager, client }
    }
}

impl DataSyncService for SyncService {
    fn perform_initial_sync(
        &self,
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_secret: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let consumer_key = consumer_key.clone();
        let consumer_secret = consumer_secret.clone();
        let access_token = access_token.clone();
        let access_secret = access_secret.clone();
        let db_manager = self.db_manager.clone();
        let client = self.client.clone();

        Box::pin(async move {
            Self::do_sync(
                db_manager,
                client,
                consumer_key,
                consumer_secret,
                access_token,
                access_secret,
            )
            .await
        })
    }
}

impl SyncService {
    async fn do_sync(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_secret: String,
    ) -> Result<(), Error> {
        debug!("consumer_key: {}", consumer_key);
        debug!("consumer_secret: {}", consumer_secret);
        debug!("access_token: {}", access_token);
        debug!("access_secret: {}", access_secret);

        // Helper to get fresh auth data
        let get_auth = || {
            crate::chpp::oauth::create_oauth_context(
                &consumer_key,
                &consumer_secret,
                &access_token,
                &access_secret,
            )
        };

        // 1. World Details
        let (data, key) = get_auth();
        debug!("auth_data: {:#?}", get_auth());

        let world_details = client.world_details(data, key).await?;
        info!("world_details: {:#?}", world_details);

        let db = db_manager.clone();
        let wd = world_details; // move
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            save_world_details(&mut conn, &wd)
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        // 2. Team Details
        let (data, key) = get_auth();
        let hattrick_data = client.team_details(data, key, None).await?;

        // We might get multiple teams. The user object is at top level.
        let user = hattrick_data.User;
        let teams = hattrick_data.Teams.Teams;

        // Save User and Teams
        let db = db_manager.clone();
        let u = user; // move
        let ts = teams; // move

        // We need team IDs for players request, so we must capture them or reload them.
        // Let's clone what we need before moving
        let team_ids: Vec<u32> = ts.iter().map(|t| t.TeamID.parse().unwrap_or(0)).collect();

        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            for team in &ts {
                save_team(&mut conn, team, &u)?;
            }
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        // 3. Players for each team
        for team_id in team_ids {
            if team_id == 0 {
                continue;
            }

            let (data, key) = get_auth();
            let players_data = client.players(data, key, Some(team_id)).await?;
            let player_list = players_data
                .Team
                .PlayerList
                .ok_or(Error::Xml("No player list found in response".to_string()))?;

            let db = db_manager.clone();
            let pl = player_list.players;

            tokio::task::spawn_blocking(move || {
                let mut conn = db.get_connection()?;
                save_players(&mut conn, &pl, team_id)
            })
            .await
            .map_err(|e| Error::Io(format!("Join error: {}", e)))??;
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::*;
    use crate::db::manager::DbManager;
    use async_trait::async_trait;

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
                    UserID: 1,
                    Loginname: "TestUser".to_string(),
                    Name: "Test User".to_string(),
                    SupporterTier: SupporterTier::Platinum,
                    SignupDate: "2000-01-01 00:00:00".to_string(),
                    ActivationDate: "2000-01-01 00:00:00".to_string(),
                    LastLoginDate: "2020-01-01 00:00:00".to_string(),
                    HasManagerLicense: true,
                    Language: Language {
                        LanguageID: 2,
                        LanguageName: "English".to_string(),
                    },
                },
                Teams: Teams {
                    Teams: vec![Team {
                        TeamID: "123".to_string(),
                        TeamName: "Test FC".to_string(),
                        ShortTeamName: Some("TFC".to_string()),
                        IsPrimaryClub: Some(true),
                        FoundedDate: Some("2010-01-01".to_string()),
                        Arena: Some(Arena {
                            ArenaID: 500,
                            ArenaName: "Test Arena".to_string(),
                        }),
                        League: Some(League {
                            LeagueID: 100,
                            LeagueName: "TestLeague".to_string(),
                        }),
                        Country: Some(Country {
                            CountryID: 10,
                            CountryName: "TestCountry".to_string(),
                            Currency: None,
                        }),
                        Region: Some(Region {
                            RegionID: 50,
                            RegionName: "Test Region".to_string(),
                        }),
                        HomePage: Some("".to_string()),
                        DressURI: Some("".to_string()),
                        DressAlternateURI: Some("".to_string()),
                        LogoURL: Some("".to_string()),
                        Trainer: Some(Trainer { PlayerID: 999 }),
                        Cup: Some(Cup {
                            StillInCup: false,
                            CupID: Some(55),
                            CupName: Some("Test Cup".to_string()),
                            CupLeagueLevel: Some(1),
                            CupLevel: Some(1),
                            CupLevelIndex: Some(1),
                            MatchRound: Some(1),
                            MatchRoundsLeft: Some(0),
                        }),
                        PowerRating: Some(PowerRating {
                            GlobalRanking: 1000,
                            LeagueRanking: 500,
                            RegionRanking: 200,
                            PowerRating: 100,
                        }),
                        FriendlyTeamID: Some(0),
                        LeagueLevelUnit: Some(LeagueLevelUnit {
                            LeagueLevelUnitID: 400,
                            LeagueLevelUnitName: "IV.1".to_string(),
                            LeagueLevel: 4,
                        }),
                        NumberOfVictories: Some(10),
                        NumberOfUndefeated: Some(5),
                        NumberOfVisits: Some(0),
                        TeamRank: Some(1),
                        Fanclub: Some(Fanclub {
                            FanclubID: 600,
                            FanclubName: "Fans".to_string(),
                            FanclubSize: 100,
                        }),
                        IsDeactivated: Some(false),
                        TeamColors: None,
                        BotStatus: None,
                        PlayerList: None,
                        YouthTeamID: Some(0),
                        YouthTeamName: Some("".to_string()),
                        PossibleToChallengeMidweek: Some(false),
                        PossibleToChallengeWeekend: Some(false),
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
                    FoundedDate: Some("2010-01-01".to_string()),
                    Arena: Some(Arena {
                        ArenaID: 500,
                        ArenaName: "Test Arena".to_string(),
                    }),
                    League: Some(League {
                        LeagueID: 100,
                        LeagueName: "TestLeague".to_string(),
                    }),
                    Country: Some(Country {
                        CountryID: 10,
                        CountryName: "TestCountry".to_string(),
                        Currency: None,
                    }),
                    Region: Some(Region {
                        RegionID: 50,
                        RegionName: "Test Region".to_string(),
                    }),
                    HomePage: Some("".to_string()),
                    DressURI: Some("".to_string()),
                    DressAlternateURI: Some("".to_string()),
                    LogoURL: Some("".to_string()),
                    Trainer: Some(Trainer { PlayerID: 999 }),
                    Cup: Some(Cup {
                        StillInCup: false,
                        CupID: Some(55),
                        CupName: Some("Test Cup".to_string()),
                        CupLeagueLevel: Some(1),
                        CupLevel: Some(1),
                        CupLevelIndex: Some(1),
                        MatchRound: Some(1),
                        MatchRoundsLeft: Some(0),
                    }),
                    PowerRating: Some(PowerRating {
                        GlobalRanking: 1000,
                        LeagueRanking: 500,
                        RegionRanking: 200,
                        PowerRating: 100,
                    }),
                    FriendlyTeamID: Some(0),
                    LeagueLevelUnit: Some(LeagueLevelUnit {
                        LeagueLevelUnitID: 400,
                        LeagueLevelUnitName: "IV.1".to_string(),
                        LeagueLevel: 4,
                    }),
                    NumberOfVictories: Some(10),
                    NumberOfUndefeated: Some(5),
                    NumberOfVisits: Some(0),
                    TeamRank: Some(1),
                    Fanclub: Some(Fanclub {
                        FanclubID: 600,
                        FanclubName: "Fans".to_string(),
                        FanclubSize: 100,
                    }),
                    IsDeactivated: Some(false),
                    TeamColors: None,
                    BotStatus: None,
                    YouthTeamID: Some(0),
                    YouthTeamName: Some("".to_string()),
                    PossibleToChallengeMidweek: Some(false),
                    PossibleToChallengeWeekend: Some(false),
                    PlayerList: Some(PlayerList {
                        players: vec![Player {
                            PlayerID: 1001,
                            FirstName: "John".to_string(),
                            LastName: "Doe".to_string(),
                            PlayerNumber: 10,
                            Age: 20,
                            AgeDays: Some(50),
                            TSI: 1000,
                            PlayerForm: 5,
                            Statement: Some("".to_string()),
                            Experience: 3,
                            Loyalty: 10,
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
                            CountryID: 10,
                            Caps: Some(0),
                            CapsU20: Some(0),
                            Cards: Some(0),
                            InjuryLevel: Some(-1),
                            Sticker: Some("".to_string()),
                            ReferencePlayerID: None,
                            PlayerSkills: None,
                        }],
                    }),
                },
            })
        }
    }

    #[tokio::test]
    async fn test_perform_initial_sync() {
        let db_manager = Arc::new(DbManager::from_url(":memory:"));
        db_manager.run_migrations().expect("Migrations failed");

        let client = Arc::new(MockChppClient);
        let service = SyncService::new_with_client(db_manager.clone(), client);

        let res = service
            .perform_initial_sync(
                "dummy_key".into(),
                "dummy_secret".into(),
                "dummy_token".into(),
                "dummy_secret".into(),
            )
            .await;

        assert!(res.is_ok(), "Sync failed: {:?}", res.err());

        // Verify DB calls
        let has_users = db_manager.has_users().expect("Failed to check users");
        assert!(has_users, "User should satisfy DB check");

        // Could verify more details here if needed, like specific data presence
    }
}
