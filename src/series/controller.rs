/* series.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::{LeagueDetailsData, MatchesData};
use crate::chpp::request::{league_details_request, matches_request, team_details_request};
use crate::service::secret::{SecretStorageService, SystemSecretService};

use std::error::Error;

pub struct SeriesController;

impl SeriesController {
    pub async fn load_series_data(
        team_id: u32,
    ) -> Result<(LeagueDetailsData, MatchesData), Box<dyn Error>> {
        let secret_service = SystemSecretService::new();
        let token = secret_service
            .get_secret("access_token")
            .await?
            .ok_or("No access token found")?;
        let token_secret = secret_service
            .get_secret("access_secret")
            .await?
            .ok_or("No access secret found")?;

        if token.is_empty() || token_secret.is_empty() {
            return Err("Empty access token or secret".into());
        }

        let key = crate::config::consumer_key();
        let secret = crate::config::consumer_secret();

        // 0. Initialize DB Manager
        let db_manager = crate::db::manager::DbManager::new();
        let mut conn = db_manager.get_connection()?;

        // 1. Get LeagueUnitID
        let league_unit_id =
            Self::get_league_unit_id(team_id, &mut conn, &key, &secret, &token, &token_secret)
                .await?;

        log::debug!("Found LeagueUnitID: {}", league_unit_id);

        // 2. Check DB for League and Matches
        let db_league = crate::db::series::get_latest_league_details(&mut conn, league_unit_id)?;
        let db_matches = crate::db::series::get_latest_matches(&mut conn, team_id)?;

        if let (Some(league_details), Some(matches_data)) = (db_league, db_matches) {
            log::info!("Loaded Series and Matches data from Database.");
            let filtered = filter_matches_for_season(&league_details, matches_data);
            return Ok((league_details, filtered));
        }

        log::info!("Data not found in DB. Fetching from CHPP API...");

        // Create a download record for this session
        let timestamp = chrono::Utc::now().to_rfc3339();
        let download_id =
            crate::db::download_entries::create_download(&mut conn, &timestamp, "completed")?;

        // 3. Fetch and Save League Details
        let league_details = Self::fetch_and_save_league_details(
            league_unit_id,
            download_id,
            &mut conn,
            &key,
            &secret,
            &token,
            &token_secret,
        )
        .await?;

        // 4. Fetch and Save Matches
        let matches = Self::fetch_and_save_matches(
            team_id,
            download_id,
            &mut conn,
            &key,
            &secret,
            &token,
            &token_secret,
        )
        .await?;

        let filtered_matches = filter_matches_for_season(&league_details, matches);
        Ok((league_details, filtered_matches))
    }

    async fn get_league_unit_id(
        team_id: u32,
        conn: &mut diesel::SqliteConnection,
        key: &str,
        secret: &str,
        token: &str,
        token_secret: &str,
    ) -> Result<u32, Box<dyn Error>> {
        // Try DB first
        let team_details_opt = crate::db::teams::get_team(conn, team_id)?;
        if let Some(team) = team_details_opt {
            log::debug!("Found team details in DB for team_id: {}", team_id);
            return Ok(team
                .LeagueLevelUnit
                .ok_or("No LeagueLevelUnit found for team")?
                .LeagueLevelUnitID);
        }

        // Fetch from API
        log::debug!("Fetching team details from API for team_id: {}", team_id);
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(key, secret, token, token_secret);
        let team_details_data =
            team_details_request(oauth_data, signing_key, Some(team_id)).await?;

        let team_str = team_id.to_string();
        let team = team_details_data
            .Teams
            .Teams
            .iter()
            .find(|t| t.TeamID == team_str)
            .ok_or_else(|| {
                log::error!("Team {} not found in team details response", team_id);
                "Team not found in response"
            })?;

        Ok(team
            .LeagueLevelUnit
            .as_ref()
            .ok_or_else(|| {
                log::error!("No LeagueLevelUnit found for team {}", team_id);
                "No LeagueLevelUnit found for team"
            })?
            .LeagueLevelUnitID)
    }

    async fn fetch_and_save_league_details(
        league_unit_id: u32,
        download_id: i32,
        conn: &mut diesel::SqliteConnection,
        key: &str,
        secret: &str,
        token: &str,
        token_secret: &str,
    ) -> Result<LeagueDetailsData, Box<dyn Error>> {
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(key, secret, token, token_secret);

        log::debug!("Fetching league details for unit: {}", league_unit_id);
        let league_details =
            league_details_request(oauth_data, signing_key, league_unit_id).await?;

        crate::db::series::save_league_details(conn, download_id, &league_details)?;
        Ok(league_details)
    }

    async fn fetch_and_save_matches(
        team_id: u32,
        download_id: i32,
        conn: &mut diesel::SqliteConnection,
        key: &str,
        secret: &str,
        token: &str,
        token_secret: &str,
    ) -> Result<MatchesData, Box<dyn Error>> {
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(key, secret, token, token_secret);

        log::debug!("Fetching upcoming matches for team: {}", team_id);
        let upcoming_matches = matches_request(oauth_data, signing_key, Some(team_id)).await?;
        log::debug!(
            "Fetched {} upcoming matches",
            upcoming_matches.Team.MatchList.Matches.len()
        );

        // We also need archived matches to show results
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(key, secret, token, token_secret);

        log::debug!("Fetching archived matches for team: {}", team_id);
        let archived_matches_res = crate::chpp::request::matches_archive_request(
            oauth_data,
            signing_key,
            Some(team_id),
            None,
            None,
        )
        .await;

        let mut matches = upcoming_matches;

        match archived_matches_res {
            Ok(archived) => {
                log::debug!(
                    "Fetched {} archived matches",
                    archived.Team.MatchList.Matches.len()
                );
                let mut all_matches = archived.Team.MatchList.Matches;
                all_matches.extend(matches.Team.MatchList.Matches);
                matches.Team.MatchList.Matches = all_matches;
            }
            Err(e) => {
                log::warn!("Failed to fetch archived matches: {}", e);
            }
        }

        crate::db::series::save_matches(conn, download_id, &matches)?;
        Ok(matches)
    }
}

fn filter_matches_for_season(
    league_details: &LeagueDetailsData,
    mut matches: MatchesData,
) -> MatchesData {
    // Filter matches to show only those for this league and current season
    let league_unit_id = league_details.LeagueLevelUnitID;
    let current_round = league_details.CurrentMatchRound.unwrap_or(14); // Default to full season if unknown

    let league_matches: Vec<crate::chpp::model::MatchDetails> = matches
        .Team
        .MatchList
        .Matches
        .into_iter()
        .filter(|m| {
            // Keep only league matches (MatchType 1) for this team's specific division.
            // MatchContextId carries the LeagueLevelUnitId for league matches, which lets
            // us distinguish "our division" from other league matches that may appear in
            // the results (e.g. if archived matches span multiple seasons/divisions).
            // We also accept None to remain compatible with rows that were written before
            // the match_context_id column was added to the database.
            m.MatchType == 1
                && (m.MatchContextId.is_none() || m.MatchContextId == Some(league_unit_id))
        })
        .collect();

    // Separate finished and upcoming
    let (mut finished, upcoming): (
        Vec<crate::chpp::model::MatchDetails>,
        Vec<crate::chpp::model::MatchDetails>,
    ) = league_matches
        .into_iter()
        .partition(|m| m.Status == "FINISHED");

    // Sort finished matches by date descending to get the most recent ones
    finished.sort_by(|a, b| b.MatchDate.cmp(&a.MatchDate));

    // Keep only the matches for the current season (based on round)
    // If we represent a season, we expect a maximum of 'current_round' finished matches
    let matches_to_keep = std::cmp::min(finished.len(), current_round as usize);
    let mut relevant_matches: Vec<crate::chpp::model::MatchDetails> =
        finished.into_iter().take(matches_to_keep).collect();

    // Add all upcoming matches (they belong to this season)
    relevant_matches.extend(upcoming);

    // Sort everything by date ascending for display
    relevant_matches.sort_by(|a, b| a.MatchDate.cmp(&b.MatchDate));

    matches.Team.MatchList.Matches = relevant_matches;

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{
        LeagueDetailsData, MatchAwayTeam, MatchDetails, MatchHomeTeam, MatchesData,
        MatchesListWrapper, MatchesTeamWrapper,
    };

    fn create_dummy_match(
        id: u32,
        date: &str,
        match_type: u32,
        context_id: Option<u32>,
        status: &str,
    ) -> MatchDetails {
        MatchDetails {
            MatchID: id,
            HomeTeam: MatchHomeTeam {
                HomeTeamID: "1".to_string(),
                HomeTeamName: "Home".to_string(),
                HomeTeamNameShortName: None,
            },
            AwayTeam: MatchAwayTeam {
                AwayTeamID: "2".to_string(),
                AwayTeamName: "Away".to_string(),
                AwayTeamNameShortName: None,
            },
            MatchDate: date.to_string(),
            SourceSystem: None,
            MatchType: match_type,
            MatchContextId: context_id,
            CupLevel: None,
            CupLevelIndex: None,
            HomeGoals: None,
            AwayGoals: None,
            OrdersGiven: None,
            Status: status.to_string(),
        }
    }

    #[test]
    fn test_filter_matches_for_season() {
        let league_unit_id = 1000;
        let current_round = 5;

        let league_details = LeagueDetailsData {
            LeagueID: 1,
            LeagueName: "Test League".to_string(),
            LeagueLevel: 1,
            MaxLevel: None,
            LeagueLevelUnitID: league_unit_id,
            LeagueLevelUnitName: "Test Unit".to_string(),
            CurrentMatchRound: Some(current_round),
            Rank: None,
            Teams: vec![],
        };

        let matches_data = MatchesData {
            Team: MatchesTeamWrapper {
                TeamID: "1".to_string(),
                TeamName: "Test Team".to_string(),
                ShortTeamName: None,
                League: None,
                LeagueLevelUnit: None,
                MatchList: MatchesListWrapper {
                    Matches: vec![
                        // Previous season match (should be filtered out by count)
                        create_dummy_match(1, "2023-01-01", 1, Some(league_unit_id), "FINISHED"),
                        // Cup Match (should be filtered out by type)
                        create_dummy_match(
                            2,
                            "2023-02-01",
                            2, // Cup
                            Some(2000),
                            "FINISHED",
                        ),
                        // League Match, Wrong Unit (should be filtered out)
                        create_dummy_match(3, "2023-02-08", 1, Some(999), "FINISHED"),
                        // Current Season Matches (Round 1-5)
                        create_dummy_match(11, "2023-03-01", 1, Some(league_unit_id), "FINISHED"),
                        create_dummy_match(12, "2023-03-08", 1, Some(league_unit_id), "FINISHED"),
                        create_dummy_match(13, "2023-03-15", 1, Some(league_unit_id), "FINISHED"),
                        create_dummy_match(14, "2023-03-22", 1, Some(league_unit_id), "FINISHED"),
                        create_dummy_match(15, "2023-03-29", 1, Some(league_unit_id), "FINISHED"),
                        // Upcoming Match
                        create_dummy_match(16, "2023-04-05", 1, Some(league_unit_id), "UPCOMING"),
                    ],
                },
            },
        };

        let filtered = filter_matches_for_season(&league_details, matches_data);
        let result_matches = filtered.Team.MatchList.Matches;

        // We expect:
        // - No Cup match (ID 2)
        // - No Wrong Unit match (ID 3)
        // - 5 Finished matches (IDs 11-15), ID 1 should be dropped as it's the 6th oldest
        // - 1 Upcoming match (ID 16)
        // Total = 6

        assert_eq!(result_matches.len(), 6);

        // Check if ID 1 is gone (oldest finished)
        assert!(!result_matches.iter().any(|m| m.MatchID == 1));

        // Check if ID 2 is gone (Cup)
        assert!(!result_matches.iter().any(|m| m.MatchID == 2));

        // Check if ID 3 is gone (Wrong Unit)
        assert!(!result_matches.iter().any(|m| m.MatchID == 3));

        // Check if ID 16 is present (Upcoming)
        assert!(result_matches.iter().any(|m| m.MatchID == 16));

        // Check IDs 11-15 are present
        for i in 11..=15 {
            assert!(result_matches.iter().any(|m| m.MatchID == i));
        }
    }
}
