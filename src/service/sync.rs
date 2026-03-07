/* sync.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
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
use crate::chpp::metadata::ChppEndpoints;
use crate::chpp::{create_oauth_context, retry_with_default_config, ChppClient, Error};
use crate::db::download_entries::{create_download_entry, update_entry_status, NewDownloadEntry};
use crate::db::manager::DbManager;
use crate::db::schema::downloads;
use crate::db::series::{save_league_details, save_matches};
use crate::db::staff::save_staff;
use crate::db::teams::{save_avatars, save_players, save_team, save_world_details};
use crate::service::avatar::AvatarService;
use crate::service::secret::{SecretStorageService, SystemSecretService};
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, info, warn};
use oauth_1a::{OAuthData, SigningKey};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type ProgressCallback = Box<dyn Fn(f64, &str) + Send + Sync>;

pub trait DataSyncService {
    fn perform_initial_sync(
        &self,
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_secret: String,
        on_progress: ProgressCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;

    fn perform_sync_with_stored_secrets(
        &self,
        consumer_key: String,
        consumer_secret: String,
        on_progress: ProgressCallback,
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
            secret_service: Arc::new(SystemSecretService::new()),
        }
    }

    #[cfg(test)]
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
        access_token: String,
        access_secret: String,
        on_progress: ProgressCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let db_manager = self.db_manager.clone();
        let client = self.client.clone();

        Box::pin(async move {
            Self::do_full_sync(
                db_manager,
                client,
                consumer_key,
                consumer_secret,
                access_token,
                access_secret,
                on_progress,
            )
            .await
        })
    }

    fn perform_sync_with_stored_secrets(
        &self,
        consumer_key: String,
        consumer_secret: String,
        on_progress: ProgressCallback,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send + '_>> {
        let db_manager = self.db_manager.clone();
        let client = self.client.clone();
        let secret_service = self.secret_service.clone();

        Box::pin(async move {
            let access_token = match secret_service.get_secret("access_token").await {
                Ok(Some(token)) => token,
                Ok(None) => return Ok(false),
                Err(e) => return Err(Error::Io(e.to_string())),
            };

            let access_secret = match secret_service.get_secret("access_secret").await {
                Ok(Some(secret)) => secret,
                Ok(None) => return Ok(false),
                Err(e) => return Err(Error::Io(e.to_string())),
            };

            Self::do_full_sync(
                db_manager,
                client,
                consumer_key,
                consumer_secret,
                access_token,
                access_secret,
                on_progress,
            )
            .await
            .map(|_| true)
        })
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

    /// Log a download entry for an API call
    async fn log_download_entry(
        db_manager: Arc<DbManager>,
        download_id: i32,
        endpoint: &str,
        version: &str,
        user_id: Option<i32>,
    ) -> Result<i32, Error> {
        let db = db_manager.clone();
        let endpoint = endpoint.to_string();
        let version = version.to_string();
        let fetched_date = Utc::now().to_rfc3339();

        tokio::task::spawn_blocking(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| Error::Db(format!("Failed to get database connection: {}", e)))?;

            let entry = NewDownloadEntry {
                download_id,
                endpoint,
                version,
                user_id,
                status: "in_progress".to_string(),
                fetched_date,
                error_message: None,
                retry_count: 0,
            };

            create_download_entry(&mut conn, entry)
                .map_err(|e| Error::Db(format!("Failed to create download entry: {}", e)))
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))?
    }

    /// Update download entry status (success or error).
    ///
    /// This is a documented exception to the insert-only principle: `download_entries`
    /// rows are used for operational retry tracking and audit logging, not for domain
    /// history. Mutating the status in-place avoids creating phantom "in_progress"
    /// entries that would never be resolved. See also `update_entry_status` in
    /// `download_entries.rs`.
    async fn update_download_entry(
        db_manager: Arc<DbManager>,
        entry_id: i32,
        status: &str,
        error_msg: Option<String>,
    ) -> Result<(), Error> {
        let db = db_manager.clone();
        let status = status.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = db
                .get_connection()
                .map_err(|e| Error::Db(format!("Failed to get database connection: {}", e)))?;

            update_entry_status(&mut conn, entry_id, &status, error_msg, false)
                .map_err(|e| Error::Db(format!("Failed to update download entry: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))?
    }

    /// Downloads user data, including Teams details.
    async fn fetch_and_save_user_data<F>(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        get_auth: &F,
        download_id: i32,
    ) -> Result<(u32, Option<u32>), Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        // Log download entry for team_details
        let entry_id = Self::log_download_entry(
            db_manager.clone(),
            download_id,
            ChppEndpoints::TEAM_DETAILS.name,
            ChppEndpoints::TEAM_DETAILS.version,
            None,
        )
        .await?;

        let (data, key) = get_auth();
        let hattrick_data = match client.team_details(data, key, None).await {
            Ok(data) => {
                Self::update_download_entry(db_manager.clone(), entry_id, "success", None).await?;
                data
            }
            Err(e) => {
                Self::update_download_entry(
                    db_manager.clone(),
                    entry_id,
                    "error",
                    Some(e.to_string()),
                )
                .await?;
                return Err(e);
            }
        };

        log::info!(
            "User: {} ({})",
            hattrick_data.User.UserID,
            hattrick_data.User.Loginname
        );

        let user = hattrick_data.User;
        let teams = hattrick_data.Teams.Teams;

        // Uses first team only; multi-team accounts are not currently supported.
        let team_id: u32 = teams
            .first()
            .and_then(|t| t.TeamID.parse().ok())
            .unwrap_or(0);

        let db = db_manager.clone();
        let teams_clone = teams.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            conn.transaction::<_, Error, _>(|conn| {
                for team in &teams_clone {
                    log::info!("Saving team: {} ({})", team.TeamName, team.TeamID);
                    save_team(conn, team, &user, download_id, true)?;
                }
                Ok(())
            })
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        let league_unit_id_opt = teams
            .first()
            .and_then(|t| t.LeagueLevelUnit.as_ref())
            .map(|unit| unit.LeagueLevelUnitID);

        match league_unit_id_opt {
            Some(series) => log::info!("Team {} belongs to series {}", team_id, series),
            None => log::warn!("No series found for team {}", team_id),
        }

        Ok((team_id, league_unit_id_opt))
    }

    async fn fetch_and_save_match_data<F>(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        get_auth: &F,
        team_id: u32,
        league_unit_id_opt: Option<u32>,
        download_id: i32,
    ) -> Result<(), Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        if let Some(unit_id) = league_unit_id_opt {
            let entry_id = Self::log_download_entry(
                db_manager.clone(),
                download_id,
                ChppEndpoints::LEAGUE_DETAILS.name,
                ChppEndpoints::LEAGUE_DETAILS.version,
                Some(team_id as i32),
            )
            .await?;

            let (data, key) = get_auth();
            match client.league_details(data, key, unit_id).await {
                Ok(league_details) => {
                    Self::update_download_entry(db_manager.clone(), entry_id, "success", None)
                        .await?;
                    let db = db_manager.clone();
                    tokio::task::spawn_blocking(move || {
                        let mut conn = db.get_connection()?;
                        conn.transaction::<_, Error, _>(|conn| {
                            save_league_details(conn, download_id, &league_details)
                        })
                    })
                    .await
                    .map_err(|e| Error::Io(format!("Join error: {}", e)))??;
                }
                Err(e) => {
                    Self::update_download_entry(
                        db_manager.clone(),
                        entry_id,
                        "error",
                        Some(e.to_string()),
                    )
                    .await?;
                    log::warn!("Failed to fetch league details: {}", e);
                }
            }
        }

        let entry_id = Self::log_download_entry(
            db_manager.clone(),
            download_id,
            ChppEndpoints::MATCHES.name,
            ChppEndpoints::MATCHES.version,
            Some(team_id as i32),
        )
        .await?;

        let (data, key) = get_auth();
        let upcoming_matches_res = client.matches(data, key, Some(team_id)).await;

        match upcoming_matches_res {
            Ok(matches_data) => {
                Self::update_download_entry(db_manager.clone(), entry_id, "success", None).await?;

                let db = db_manager.clone();
                tokio::task::spawn_blocking(move || {
                    let mut conn = db.get_connection()?;
                    conn.transaction::<_, Error, _>(|conn| {
                        save_matches(conn, download_id, &matches_data)
                    })
                })
                .await
                .map_err(|e| Error::Io(format!("Join error: {}", e)))??;
            }
            Err(e) => {
                Self::update_download_entry(
                    db_manager.clone(),
                    entry_id,
                    "error",
                    Some(e.to_string()),
                )
                .await?;
                log::warn!("Failed to fetch matches: {}", e);
            }
        }

        // Also fetch archived (already-played) matches so the full season is available in the DB
        let (data, key) = get_auth();
        match client
            .matches_archive(data, key, Some(team_id), None, None)
            .await
        {
            Ok(archived_data) => {
                log::debug!(
                    "Fetched {} archived matches during sync",
                    archived_data.Team.MatchList.Matches.len()
                );
                let db = db_manager.clone();
                tokio::task::spawn_blocking(move || {
                    let mut conn = db.get_connection()?;
                    save_matches(&mut conn, download_id, &archived_data)
                })
                .await
                .map_err(|e| Error::Io(format!("Join error: {}", e)))??;
            }
            Err(e) => {
                log::warn!("Failed to fetch archived matches during sync: {}", e);
                // Non-fatal: upcoming matches are already saved
            }
        }

        Ok(())
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
        // Log download entry for world_details
        let entry_id = Self::log_download_entry(
            db_manager.clone(),
            download_id,
            ChppEndpoints::WORLD_DETAILS.name,
            ChppEndpoints::WORLD_DETAILS.version,
            None,
        )
        .await?;

        let (data, key) = get_auth();
        let world_details = match client.world_details(data, key).await {
            Ok(data) => {
                Self::update_download_entry(db_manager.clone(), entry_id, "success", None).await?;
                data
            }
            Err(e) => {
                Self::update_download_entry(
                    db_manager.clone(),
                    entry_id,
                    "error",
                    Some(e.to_string()),
                )
                .await?;
                return Err(e);
            }
        };

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
        // Log download entry for players
        let entry_id = Self::log_download_entry(
            db_manager.clone(),
            download_id,
            ChppEndpoints::PLAYERS.name,
            ChppEndpoints::PLAYERS.version,
            None,
        )
        .await?;

        // Get Players for the team
        let (data, key) = get_auth();
        let players_resp = match client.players(data, key, None).await {
            Ok(data) => {
                Self::update_download_entry(db_manager.clone(), entry_id, "success", None).await?;
                data
            }
            Err(e) => {
                Self::update_download_entry(
                    db_manager.clone(),
                    entry_id,
                    "error",
                    Some(e.to_string()),
                )
                .await?;
                return Err(e);
            }
        };

        let player_list = if let Some(pl) = players_resp.Team.PlayerList {
            pl
        } else {
            log::warn!("No player list found for team");
            return Err(Error::Parse("No player list in response".to_string()));
        };

        // Fetch detailed player data for each player and merge with basic data
        let (players_list, avatars_list) = {
            info!(
                "Fetching detailed player data for {} players",
                player_list.players.len()
            );

            // Fetch avatars for the team
            let (data, key) = get_auth();
            let avatar_map = match client.avatars(data, key, Some(team_id)).await {
                Ok(avatars) => {
                    info!(
                        "Fetched {} player avatars for team {}",
                        avatars.team.players.players.len(),
                        team_id
                    );
                    let mut map = HashMap::new();
                    for avatar_player in avatars.team.players.players {
                        map.insert(avatar_player.player_id, avatar_player.avatar.layers);
                    }
                    map
                }
                Err(e) => {
                    warn!("Failed to fetch avatars for team {}: {}", team_id, e);
                    HashMap::new()
                }
            };

            use futures::stream::{self, StreamExt};

            let mut merged_players = Vec::new();
            let mut avatars_to_save = Vec::new();

            let futures = player_list.players.into_iter().map(|basic_player| {
                let db_manager = db_manager.clone();
                let client = client.clone();
                let avatar_map = &avatar_map;

                async move {
                    let player_id = basic_player.PlayerID;
                    info!("Fetching details for player ID: {}", player_id);

                    let operation_name = format!("player_details({})", player_id);

                    // Log download entry for this player_details call
                    let entry_id = match Self::log_download_entry(
                        db_manager.clone(),
                        download_id,
                        ChppEndpoints::PLAYER_DETAILS.name,
                        ChppEndpoints::PLAYER_DETAILS.version,
                        None,
                    )
                    .await
                    {
                        Ok(id) => id,
                        Err(e) => {
                            warn!("Failed to log download entry: {}", e);
                            0
                        }
                    };

                    // Use retry utility for player details fetching
                    let result =
                        retry_with_default_config(&operation_name, get_auth, |data, key| {
                            client.player_details(data, key, player_id)
                        })
                        .await;

                    // Update entry status based on result
                    if entry_id != 0 {
                        match &result {
                            Ok(_) => {
                                let _ = Self::update_download_entry(
                                    db_manager.clone(),
                                    entry_id,
                                    "success",
                                    None,
                                )
                                .await;
                            }
                            Err(e) => {
                                let _ = Self::update_download_entry(
                                    db_manager.clone(),
                                    entry_id,
                                    "error",
                                    Some(e.to_string()),
                                )
                                .await;
                            }
                        }
                    }

                    // Merge detailed data with basic data
                    let mut merged = match result {
                        Ok(detailed_player) => {
                            debug!(
                                "Successfully fetched detailed data for player {}",
                                player_id
                            );
                            basic_player
                                .clone()
                                .merge_player_data(Some(detailed_player))
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to fetch details for player {}: {}. Using basic data only.",
                                player_id,
                                e
                            );
                            basic_player.clone().merge_player_data(None)
                        }
                    };

                    let mut avatar_tuple = None;
                    if let Some(layers) = avatar_map.get(&player_id) {
                        if let Some(avatar_blob) =
                            AvatarService::fetch_and_composite_avatar(player_id, layers).await
                        {
                            merged.AvatarBlob = Some(avatar_blob.clone());
                            avatar_tuple = Some((player_id, avatar_blob));
                        }
                    }

                    (merged, avatar_tuple)
                }
            });

            // Execute concurrently with a limit of 4
            let mut stream = stream::iter(futures).buffer_unordered(4);
            while let Some((merged, avatar_tuple)) = stream.next().await {
                merged_players.push(merged);
                if let Some(avatar) = avatar_tuple {
                    avatars_to_save.push(avatar);
                }
            }

            (merged_players, avatars_to_save)
        };

        // Save players
        let db = db_manager.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            conn.transaction::<_, Error, _>(|conn| {
                save_players(conn, &players_list, team_id, download_id)?;
                save_avatars(conn, &avatars_list, download_id)
            })
        })
        .await
        .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

        Ok(())
    }

    async fn fetch_and_save_staff<F>(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        get_auth: &F,
        team_id: u32,
        download_id: i32,
    ) -> Result<(), Error>
    where
        F: Fn() -> (OAuthData, SigningKey) + Send + Sync,
    {
        // Log download entry
        let entry_id = Self::log_download_entry(
            db_manager.clone(),
            download_id,
            ChppEndpoints::STAFF_LIST.name,
            ChppEndpoints::STAFF_LIST.version,
            Some(team_id as i32),
        )
        .await?;

        let (data, key) = get_auth();
        let staff_res = client.staff_list(data, key, Some(team_id)).await;

        match staff_res {
            Ok(staff_data) => {
                Self::update_download_entry(db_manager.clone(), entry_id, "success", None).await?;

                let db = db_manager.clone();
                let sl = staff_data.staff_list;
                let staff_count = sl.StaffMembers.as_ref().map_or(0, |m| m.staff.len());

                tokio::task::spawn_blocking(move || {
                    let mut conn = db.get_connection()?;
                    save_staff(&mut conn, &sl, team_id, download_id)
                        .map_err(|e| Error::Db(format!("Failed to save staff: {}", e)))
                })
                .await
                .map_err(|e| Error::Io(format!("Join error: {}", e)))??;

                log::info!("Saved {} staff members", staff_count);
            }
            Err(e) => {
                Self::update_download_entry(
                    db_manager.clone(),
                    entry_id,
                    "error",
                    Some(e.to_string()),
                )
                .await?;
                log::warn!("Failed to fetch staff list: {}", e);
            }
        }

        Ok(())
    }

    async fn do_full_sync(
        db_manager: Arc<DbManager>,
        client: Arc<dyn ChppClient>,
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_secret: String,
        on_progress: ProgressCallback,
    ) -> Result<(), Error> {
        on_progress(0.0, "Checking credentials...");

        debug!("consumer_key: {}", consumer_key);
        debug!("access_token length: {}", access_token.len());
        debug!("access_secret length: {}", access_secret.len());

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

        on_progress(
            0.1,
            "Fetching world details (countries, leagues, currencies)...",
        );
        Self::fetch_and_save_world_details(
            db_manager.clone(),
            client.clone(),
            &get_auth,
            download_id,
        )
        .await?;

        on_progress(0.5, "Fetching user data...");
        let (team_id, league_unit_id_opt) = Self::fetch_and_save_user_data(
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

        on_progress(0.7, "Fetching staff...");
        Self::fetch_and_save_staff(
            db_manager.clone(),
            client.clone(),
            &get_auth,
            team_id,
            download_id,
        )
        .await?;

        on_progress(0.8, "Fetching series and matches...");
        Self::fetch_and_save_match_data(
            db_manager.clone(),
            client.clone(),
            &get_auth,
            team_id,
            league_unit_id_opt,
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
                        LanguageId: None,
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
                    Language: Some(Language {
                        LanguageID: 2,
                        LanguageName: "English".to_string(),
                    }),
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
                        GenderID: Some(1),
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
                    GenderID: Some(1),
                    PlayerList: Some(PlayerList {
                        players: vec![Player {
                            PlayerID: 1000,
                            FirstName: "Test".to_string(),
                            LastMatch: None,
                            AvatarBlob: None,
                            GenderID: Some(1),
                            Flag: None,
                            NativeCountryFlag: None,
                            LastName: "Player".to_string(),
                            NickName: None,
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
                            Specialty: Some(0),
                            TransferListed: false,
                            NationalTeamID: None,
                            CountryID: Some(10),
                            Caps: Some(0),
                            CapsU20: Some(0),
                            Cards: Some(0),
                            InjuryLevel: Some(-1),
                            PlayerCategoryId: None,
                            MotherClub: None,
                            NativeCountryID: None,
                            NativeLeagueID: None,
                            NativeLeagueName: None,
                            MatchesCurrentTeam: None,
                            GoalsCurrentTeam: None,
                            AssistsCurrentTeam: None,
                            CareerAssists: None,
                            ArrivalDate: None,
                            PlayerSkills: None,
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
                LastMatch: None,
                AvatarBlob: None,
                GenderID: Some(1),
                Flag: None,
                NativeCountryFlag: None,
                NickName: None,
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
                Specialty: Some(0),
                TransferListed: false,
                NationalTeamID: Some(0),
                CountryID: Some(10),
                Caps: Some(0),
                CapsU20: Some(0),
                Cards: Some(0),
                InjuryLevel: Some(-1),
                PlayerCategoryId: None,
                MotherClub: None,
                NativeCountryID: None,
                NativeLeagueID: None,
                NativeLeagueName: None,
                MatchesCurrentTeam: None,
                GoalsCurrentTeam: None,
                AssistsCurrentTeam: None,
                CareerAssists: None,
                PlayerSkills: None,
                ArrivalDate: None,
            })
        }

        async fn avatars(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<AvatarsData, Error> {
            Ok(AvatarsData {
                file_name: "avatars.xml".to_string(),
                version: "1.0".to_string(),
                user_id: 12345,
                team: AvatarsTeam {
                    team_id: 54321,
                    players: AvatarsPlayers { players: vec![] },
                },
            })
        }

        async fn league_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _league_level_unit_id: u32,
        ) -> Result<LeagueDetailsData, Error> {
            // Return dummy league details
            Ok(LeagueDetailsData {
                LeagueID: 0,
                LeagueName: "Test League".to_string(),
                LeagueLevel: 5,
                MaxLevel: Some(8),
                LeagueLevelUnitID: 100,
                LeagueLevelUnitName: "Test League Unit".to_string(),
                CurrentMatchRound: Some(1),
                Rank: None,
                Teams: vec![],
            })
        }

        async fn matches(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<MatchesData, Error> {
            // Return dummy matches
            Ok(MatchesData {
                Team: MatchesTeamWrapper {
                    TeamID: "54321".to_string(),
                    TeamName: "Test Team".to_string(),
                    ShortTeamName: None,
                    League: None,
                    LeagueLevelUnit: None,
                    MatchList: MatchesListWrapper { Matches: vec![] },
                },
            })
        }

        async fn staff_list(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _team_id: Option<u32>,
        ) -> Result<StaffListData, Error> {
            Ok(StaffListData {
                file_name: "stafflist.xml".to_string(),
                version: "1.2".to_string(),
                user_id: 12345,
                fetched_date: None,
                staff_list: StaffList {
                    Trainer: None,
                    StaffMembers: None,
                    TotalStaffMembers: Some(0),
                    TotalCost: Some(0),
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
        ) -> Result<MatchesData, Error> {
            // Return empty archive in tests
            Ok(MatchesData {
                Team: MatchesTeamWrapper {
                    TeamID: "54321".to_string(),
                    TeamName: "Test Team".to_string(),
                    ShortTeamName: None,
                    League: None,
                    LeagueLevelUnit: None,
                    MatchList: MatchesListWrapper { Matches: vec![] },
                },
            })
        }

        async fn match_details(
            &self,
            _data: OAuthData,
            _key: SigningKey,
            _match_id: u32,
            _source_system: &str,
        ) -> Result<MatchDetailsData, Error> {
            unimplemented!()
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
    }

    #[tokio::test]
    async fn test_perform_initial_sync() {
        let db_manager = Arc::new(DbManager::from_url(":memory:"));
        db_manager.run_migrations().expect("Migrations failed");

        let client = Arc::new(MockChppClient);
        let service = SyncService::new_with_client(
            db_manager.clone(),
            client,
            Arc::new(MockSecretService::new()),
        );

        let res = service
            .perform_initial_sync(
                "dummy_key".into(),
                "dummy_secret".into(),
                "dummy_token".into(),
                "dummy_secret_val".into(),
                Box::new(|_, _| {}),
            )
            .await;

        assert!(res.is_ok(), "Sync failed: {:?}", res.err());

        // Verify DB calls
        let has_users = db_manager.has_users().expect("Failed to check users");
        assert!(has_users, "User should satisfy DB check");

        // Could verify more details here if needed, like specific data presence
    }
}
