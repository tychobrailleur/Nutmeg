/* opponent_analysis.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::client::ChppClient;
use crate::chpp::error::Error;
use crate::chpp::model::Player;
use crate::db::manager::DbManager;
use crate::db::match_ratings::{MatchRating, NewMatchRating};
use crate::rating::types::{Behaviour, PositionId};
use crate::ui::components::pitch_view::PitchPlayer;
use oauth_1a::{OAuthData, SigningKey};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct OpponentAnalysis {
    pub team_id: u32,
    pub matches: Vec<OpponentMatchData>,
    pub injured_or_suspended_players: Vec<Player>,
    pub formation_frequencies: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UpcomingOpponent {
    pub team_id: u32,
    pub team_name: String,
    pub match_date: String,
    pub match_id: u32,
}

#[derive(Debug, Clone)]
pub struct OpponentMatchData {
    pub match_id: u32,
    pub match_date: String,
    pub is_home: bool,
    pub home_team_name: String,
    pub away_team_name: String,
    pub opponent_team_name: String,
    pub match_type: u32,
    pub home_goals: Option<u32>,
    pub away_goals: Option<u32>,
    pub formation: Option<String>,
    pub tactic_type: Option<u32>,
    pub rating_midfield: Option<u32>,
    pub rating_right_def: Option<u32>,
    pub rating_mid_def: Option<u32>,
    pub rating_left_def: Option<u32>,
    pub rating_right_att: Option<u32>,
    pub rating_mid_att: Option<u32>,
    pub rating_left_att: Option<u32>,
}

pub struct OpponentAnalysisService {
    client: Arc<dyn ChppClient>,
}

impl OpponentAnalysisService {
    pub fn new(client: Arc<dyn ChppClient>) -> Self {
        Self { client }
    }

    pub async fn get_upcoming_opponents<F>(
        &self,
        get_auth: &F,
        our_team_id: u32,
    ) -> Result<Vec<UpcomingOpponent>, Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        let (data, key) = get_auth();
        let matches_data = self.client.matches(data, key, Some(our_team_id)).await?;
        let match_list = matches_data.Team.MatchList.Matches;

        let mut opponents = Vec::new();

        for m in match_list {
            if m.Status == "UPCOMING" {
                // Determine which team is the opponent
                let home_id = m.HomeTeam.HomeTeamID.parse::<u32>().unwrap_or(0);

                let (opp_id, opp_name) = if home_id == our_team_id {
                    let away_id = m.AwayTeam.AwayTeamID.parse::<u32>().unwrap_or(0);
                    (away_id, m.AwayTeam.AwayTeamName.clone())
                } else {
                    (home_id, m.HomeTeam.HomeTeamName.clone())
                };

                opponents.push(UpcomingOpponent {
                    team_id: opp_id,
                    team_name: opp_name,
                    match_date: m.MatchDate.clone(),
                    match_id: m.MatchID,
                });
            }
        }

        // Sort ascending by date for upcoming matches
        opponents.sort_by(|a, b| a.match_date.cmp(&b.match_date));

        Ok(opponents)
    }

    pub async fn analyze_opponent<F>(
        &self,
        get_auth: &F,
        team_id: u32,
        limit: usize,
        match_type_filter: Option<Vec<u32>>,
    ) -> Result<OpponentAnalysis, Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        let (data, key) = get_auth();
        let matches_data = self.client.matches(data, key, Some(team_id)).await?;
        let mut match_list = matches_data.Team.MatchList.Matches.clone();

        // Sort descending by date
        match_list.sort_by(|a, b| b.MatchDate.cmp(&a.MatchDate));

        let finished_matches: Vec<_> = match_list
            .into_iter()
            .filter(|m| m.Status == "FINISHED")
            .filter(|m| {
                if let Some(ref filter) = match_type_filter {
                    filter.contains(&m.MatchType)
                } else {
                    true
                }
            })
            .take(limit)
            .collect();

        // Persist the matches to DB so they are available without re-analysis
        if let Ok(mut conn) = DbManager::new().get_connection() {
            if let Ok(download_id) = crate::db::download_entries::get_latest_download_id(&mut conn)
            {
                let _ = crate::db::series::save_matches(&mut conn, download_id, &matches_data);
            }
        }

        let mut opponent_matches = Vec::new();
        let mut formation_frequencies = HashMap::new();

        for m in finished_matches {
            let (data, key) = get_auth();
            let details_res = self
                .client
                .match_details(data, key, m.MatchID, "hattrick")
                .await;

            if let Ok(details) = details_res {
                let is_home = details
                    .Match
                    .HomeTeam
                    .HomeTeamID
                    .parse::<u32>()
                    .unwrap_or(0)
                    == team_id;

                let (formation, tactic_type, opponent_team_name, ratings) = if is_home {
                    (
                        details.Match.HomeTeam.Formation,
                        details.Match.HomeTeam.TacticType,
                        details.Match.AwayTeam.AwayTeamName.clone(),
                        (
                            details.Match.HomeTeam.RatingMidfield,
                            details.Match.HomeTeam.RatingRightDef,
                            details.Match.HomeTeam.RatingMidDef,
                            details.Match.HomeTeam.RatingLeftDef,
                            details.Match.HomeTeam.RatingRightAtt,
                            details.Match.HomeTeam.RatingMidAtt,
                            details.Match.HomeTeam.RatingLeftAtt,
                        ),
                    )
                } else {
                    (
                        details.Match.AwayTeam.Formation,
                        details.Match.AwayTeam.TacticType,
                        details.Match.HomeTeam.HomeTeamName.clone(),
                        (
                            details.Match.AwayTeam.RatingMidfield,
                            details.Match.AwayTeam.RatingRightDef,
                            details.Match.AwayTeam.RatingMidDef,
                            details.Match.AwayTeam.RatingLeftDef,
                            details.Match.AwayTeam.RatingRightAtt,
                            details.Match.AwayTeam.RatingMidAtt,
                            details.Match.AwayTeam.RatingLeftAtt,
                        ),
                    )
                };

                if let Some(ref f) = formation {
                    *formation_frequencies.entry(f.clone()).or_insert(0) += 1;
                }

                opponent_matches.push(OpponentMatchData {
                    match_id: m.MatchID,
                    match_date: m.MatchDate,
                    is_home,
                    home_team_name: details.Match.HomeTeam.HomeTeamName.clone(),
                    away_team_name: details.Match.AwayTeam.AwayTeamName.clone(),
                    opponent_team_name,
                    match_type: details.Match.MatchType,
                    home_goals: details.Match.HomeGoals,
                    away_goals: details.Match.AwayGoals,
                    formation,
                    tactic_type,
                    rating_midfield: ratings.0,
                    rating_right_def: ratings.1,
                    rating_mid_def: ratings.2,
                    rating_left_def: ratings.3,
                    rating_right_att: ratings.4,
                    rating_mid_att: ratings.5,
                    rating_left_att: ratings.6,
                });
            }
        }

        let (data, key) = get_auth();
        let players_resp = self.client.players(data, key, Some(team_id)).await;

        let mut injured_or_suspended = Vec::new();
        if let Ok(players_data) = players_resp {
            if let Some(player_list) = players_data.Team.PlayerList {
                for player in player_list.players {
                    let is_injured = player.InjuryLevel.unwrap_or(-1) > 0;
                    // Usually Cards >= 3 implies red card or suspension limits in Hattrick CHPP
                    let has_red_card = player.Cards.unwrap_or(0) >= 3;

                    if is_injured || has_red_card {
                        injured_or_suspended.push(player);
                    }
                }
            }
        }

        Ok(OpponentAnalysis {
            team_id,
            matches: opponent_matches,
            injured_or_suspended_players: injured_or_suspended,
            formation_frequencies,
        })
    }

    pub async fn get_opponent_match_lineup<F>(
        &self,
        get_auth: &F,
        match_id: u32,
        team_id: u32,
    ) -> Result<Vec<PitchPlayer>, Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        let (data, key) = get_auth();
        let lineup_data = self
            .client
            .match_lineup(data, key, match_id, team_id, "hattrick")
            .await?;

        let mut pitch_players = Vec::new();

        if let Some(starting) = lineup_data.Team.StartingLineup {
            for p in starting.Players {
                let role = PositionId::from(p.RoleID);
                let behaviour = Behaviour::from(p.Behaviour.unwrap_or(0));

                pitch_players.push(PitchPlayer {
                    id: p.PlayerID,
                    first_name: p.FirstName.unwrap_or_else(|| "".to_string()),
                    last_name: p.LastName.unwrap_or_else(|| "".to_string()),
                    role,
                    behaviour,
                    rating: None, // CHPP doesn't send individual match ratings in matchlineup
                });
            }
        }

        Ok(pitch_players)
    }

    pub fn get_stored_match_ratings(
        &self,
        team_id: u32,
    ) -> Result<Vec<MatchRating>, Box<dyn StdError>> {
        let db_manager = DbManager::new();
        let mut conn = db_manager.get_connection()?;
        let ratings = crate::db::match_ratings::get_match_ratings(&mut conn, team_id)?;
        Ok(ratings)
    }

    pub fn save_match_ratings(&self, ratings: &[NewMatchRating]) -> Result<(), Box<dyn StdError>> {
        let db_manager = DbManager::new();
        let mut conn = db_manager.get_connection()?;
        crate::db::match_ratings::save_match_ratings(&mut conn, ratings)?;
        Ok(())
    }

    pub fn get_latest_matches_from_db(
        &self,
        team_id: u32,
    ) -> Result<Option<crate::chpp::model::MatchesData>, Box<dyn StdError>> {
        let db_manager = DbManager::new();
        let mut conn = db_manager.get_connection()?;
        let matches = crate::db::series::get_latest_matches(&mut conn, team_id)?;
        Ok(matches)
    }

    pub fn get_upcoming_opponents_from_db(
        &self,
        our_team_id: u32,
    ) -> Result<Vec<UpcomingOpponent>, Box<dyn StdError>> {
        let db_manager = DbManager::new();
        let mut conn = db_manager.get_connection()?;
        let opponents = crate::db::series::get_upcoming_opponents_from_db(&mut conn, our_team_id)?;
        Ok(opponents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::*;
    use async_trait::async_trait;

    struct MockChppClient;

    #[async_trait]
    impl ChppClient for MockChppClient {
        async fn world_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
        ) -> Result<WorldDetails, Error> {
            unimplemented!()
        }

        async fn team_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            team_id: Option<u32>,
        ) -> Result<HattrickData, Error> {
            let t = Team {
                TeamID: team_id.unwrap_or(12345).to_string(),
                ..Default::default()
            };
            Ok(HattrickData {
                Teams: Teams { Teams: vec![t] },
                User: User {
                    UserID: 1,
                    ..Default::default()
                },
            })
        }

        async fn players(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            team_id: Option<u32>,
        ) -> Result<PlayersData, Error> {
            let mut list = Vec::new();
            if team_id == Some(10) {
                // Return an injured player
                let p = Player {
                    PlayerID: 100,
                    FirstName: "Injured".to_string(),
                    LastName: "Guy".to_string(),
                    InjuryLevel: Some(2), // Injured for 2 weeks
                    ..Player::default()
                };
                list.push(p);

                // Return a suspended player
                let p2 = Player {
                    PlayerID: 101,
                    FirstName: "Suspended".to_string(),
                    LastName: "Guy".to_string(),
                    Cards: Some(3), // Red card
                    ..Player::default()
                };
                list.push(p2);

                // Return a healthy player
                let p3 = Player {
                    PlayerID: 102,
                    FirstName: "Healthy".to_string(),
                    LastName: "Guy".to_string(),
                    InjuryLevel: Some(-1), // Healthy
                    Cards: Some(0),
                    ..Player::default()
                };
                list.push(p3);
            }

            Ok(PlayersData {
                Team: Team {
                    TeamID: team_id.unwrap_or(0).to_string(),
                    PlayerList: Some(PlayerList { players: list }),
                    ..Default::default()
                },
            })
        }

        async fn player_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _player_id: u32,
        ) -> Result<Player, Error> {
            unimplemented!()
        }

        async fn avatars(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<AvatarsData, Error> {
            unimplemented!()
        }

        async fn league_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _league_level_unit_id: u32,
        ) -> Result<LeagueDetailsData, Error> {
            unimplemented!()
        }

        async fn matches(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            team_id: Option<u32>,
        ) -> Result<MatchesData, Error> {
            let mut matches = Vec::new();

            if team_id == Some(10) || team_id == Some(12345) || team_id.is_none() {
                matches.push(MatchDetails {
                    MatchID: 1,
                    MatchDate: "2026-02-15 14:00:00".to_string(),
                    Status: "FINISHED".to_string(),
                    HomeTeam: MatchHomeTeam {
                        HomeTeamID: "10".to_string(),
                        ..Default::default()
                    },
                    AwayTeam: MatchAwayTeam {
                        AwayTeamID: "20".to_string(),
                        ..Default::default()
                    },
                    SourceSystem: None,
                    MatchType: 1,
                    MatchContextId: None,
                    CupLevel: None,
                    CupLevelIndex: None,
                    HomeGoals: None,
                    AwayGoals: None,
                    OrdersGiven: None,
                });

                matches.push(MatchDetails {
                    MatchID: 2,
                    MatchDate: "2026-02-22 14:00:00".to_string(),
                    Status: "UPCOMING".to_string(),
                    HomeTeam: MatchHomeTeam {
                        HomeTeamID: "12345".to_string(), // Our team
                        ..Default::default()
                    },
                    AwayTeam: MatchAwayTeam {
                        AwayTeamID: "30".to_string(),
                        AwayTeamName: "Future Opponent".to_string(),
                        ..Default::default()
                    },
                    SourceSystem: None,
                    MatchType: 1,
                    MatchContextId: None,
                    CupLevel: None,
                    CupLevelIndex: None,
                    HomeGoals: None,
                    AwayGoals: None,
                    OrdersGiven: None,
                });
            }

            Ok(MatchesData {
                Team: MatchesTeamWrapper {
                    TeamID: team_id.unwrap_or(0).to_string(),
                    TeamName: "".to_string(),
                    ShortTeamName: None,
                    League: None,
                    LeagueLevelUnit: None,
                    MatchList: MatchesListWrapper { Matches: matches },
                },
            })
        }

        async fn matches_archive(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
            _first_match_date: Option<String>,
            _last_match_date: Option<String>,
        ) -> Result<MatchesArchiveData, crate::chpp::Error> {
            Ok(MatchesArchiveData {
                Team: MatchesArchiveTeamWrapper {
                    TeamID: "0".to_string(),
                    TeamName: "".to_string(),
                    MatchList: MatchesListWrapper { Matches: vec![] },
                    ..Default::default()
                },
            })
        }

        async fn match_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            match_id: u32,
            _source_system: &str,
        ) -> Result<MatchDetailsData, Error> {
            if match_id == 1 {
                Ok(MatchDetailsData {
                    Match: MatchDetails {
                        MatchID: 1,
                        MatchType: 1,
                        MatchDate: "2023-01-01 20:00:00".to_string(),
                        SourceSystem: None,
                        MatchContextId: None,
                        CupLevel: None,
                        CupLevelIndex: None,
                        OrdersGiven: None,
                        Status: "FINISHED".to_string(),
                        HomeGoals: Some(2),
                        AwayGoals: Some(1),
                        HomeTeam: MatchHomeTeam {
                            HomeTeamID: "10".to_string(),
                            HomeTeamName: "Team A".to_string(),
                            HomeTeamNameShortName: None,
                            Formation: Some("3-5-2".to_string()),
                            TacticType: Some(0),
                            TacticSkill: Some(5),
                            RatingMidDef: Some(40),
                            RatingMidfield: Some(60),
                            ..Default::default()
                        },
                        AwayTeam: MatchAwayTeam {
                            AwayTeamID: "20".to_string(),
                            AwayTeamName: "Team B".to_string(),
                            AwayTeamNameShortName: None,
                            Formation: Some("4-4-2".to_string()),
                            TacticType: Some(1),
                            TacticSkill: Some(6),
                            ..Default::default()
                        },
                    },
                })
            } else {
                Err(Error::Network("Not Found".to_string()))
            }
        }

        async fn match_lineup(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _match_id: u32,
            _team_id: u32,
            _source_system: &str,
        ) -> Result<MatchLineupData, Error> {
            unimplemented!()
        }

        async fn staff_list(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<StaffListData, Error> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_get_upcoming_opponents() {
        let client = Arc::new(MockChppClient);
        let service = OpponentAnalysisService::new(client);

        let get_auth = || crate::chpp::oauth::create_oauth_context("", "", "", "");

        let opponents = service
            .get_upcoming_opponents(&get_auth, 12345)
            .await
            .unwrap();

        // Should find 1 upcoming opponent (match 2)
        assert_eq!(opponents.len(), 1);
        assert_eq!(opponents[0].team_id, 30);
        assert_eq!(opponents[0].team_name, "Future Opponent");
        assert_eq!(opponents[0].match_id, 2);
    }

    #[tokio::test]
    async fn test_analyze_opponent() {
        let client = Arc::new(MockChppClient);
        let service = OpponentAnalysisService::new(client);

        let get_auth = || crate::chpp::oauth::create_oauth_context("", "", "", "");

        let result = service
            .analyze_opponent(&get_auth, 10, 10, None)
            .await
            .unwrap();

        assert_eq!(result.team_id, 10);
        assert_eq!(result.matches.len(), 1);
        // Since we mock `matches` and `match_details`, we should at least get the formation
        let m = &result.matches[0];
        assert_eq!(m.match_id, 1);
        assert!(m.is_home);
        assert_eq!(m.opponent_team_name, "Team B");
        assert_eq!(m.home_goals, Some(2));
        assert_eq!(m.away_goals, Some(1));
        assert_eq!(m.formation.as_deref(), Some("3-5-2"));
        assert_eq!(m.tactic_type, Some(0));
        assert_eq!(m.rating_midfield, Some(60));

        assert_eq!(result.injured_or_suspended_players.len(), 2);
        assert_eq!(result.injured_or_suspended_players[0].PlayerID, 100);
        assert_eq!(result.injured_or_suspended_players[1].PlayerID, 101);

        assert_eq!(result.formation_frequencies.get("3-5-2"), Some(&1));
    }
}
