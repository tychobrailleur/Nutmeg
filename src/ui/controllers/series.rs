/* series.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::{LeagueDetailsData, MatchesData};
use crate::chpp::request::{league_details_request, matches_request, team_details_request};
use crate::service::secret::{GnomeSecretService, SecretStorageService};

use std::error::Error;

pub struct SeriesController;

impl SeriesController {
    pub async fn load_series_data(
        team_id: u32,
    ) -> Result<(LeagueDetailsData, MatchesData), Box<dyn Error>> {
        let secret_service = GnomeSecretService::new();
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

        // 1. Fetch Team Details to get LeagueUnitID
        // Try DB first
        let team_details_opt = crate::db::teams::get_team(&mut conn, team_id)?;
        let league_unit_id = if let Some(team) = team_details_opt {
            log::debug!("Found team details in DB for team_id: {}", team_id);
            team.LeagueLevelUnit
                .ok_or("No LeagueLevelUnit found for team")?
                .LeagueLevelUnitID
        } else {
            // Fetch from API
            log::debug!("Fetching team details from API for team_id: {}", team_id);
            let (oauth_data, signing_key) =
                crate::chpp::oauth::create_oauth_context(&key, &secret, &token, &token_secret);
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

            team.LeagueLevelUnit
                .as_ref()
                .ok_or_else(|| {
                    log::error!("No LeagueLevelUnit found for team {}", team_id);
                    "No LeagueLevelUnit found for team"
                })?
                .LeagueLevelUnitID
        };

        log::debug!("Found LeagueUnitID: {}", league_unit_id);

        // 2. Check DB for League and Matches
        let db_league = crate::db::series::get_latest_league_details(&mut conn, league_unit_id)?;
        let db_matches = crate::db::series::get_latest_matches(&mut conn, team_id)?;

        if let (Some(league_details), Some(matches_data)) = (db_league, db_matches) {
            log::info!("Loaded Series and Matches data from Database.");
            return Ok((league_details, matches_data));
        }

        log::info!("Data not found in DB. Fetching from CHPP API...");

        // Create a download record for this session
        let timestamp = chrono::Utc::now().to_rfc3339();
        let download_id =
            crate::db::download_entries::create_download(&mut conn, &timestamp, "completed")?;

        // 3. Fetch League Details
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(&key, &secret, &token, &token_secret);

        log::debug!("Fetching league details for unit: {}", league_unit_id);
        let league_details =
            league_details_request(oauth_data, signing_key, league_unit_id).await?;

        // Save League Details
        crate::db::series::save_league_details(&mut conn, download_id, &league_details)?;

        // 4. Fetch Matches (Archived and Upcoming)
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(&key, &secret, &token, &token_secret);

        log::debug!("Fetching upcoming matches for team: {}", team_id);
        let upcoming_matches = matches_request(oauth_data, signing_key, Some(team_id)).await?;
        log::debug!(
            "Fetched {} upcoming matches",
            upcoming_matches.Team.MatchList.Matches.len()
        );

        // We also need archived matches to show results
        let (oauth_data, signing_key) =
            crate::chpp::oauth::create_oauth_context(&key, &secret, &token, &token_secret);

        // For now, fetch default archived matches (last 50 or current season)
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

        // Save Matches
        crate::db::series::save_matches(&mut conn, download_id, &matches)?;

        Ok((league_details, matches))
    }
}
