use crate::chpp::oauth::create_oauth_context;
use crate::chpp::request::{matches_archive_request, team_details_request};
use crate::config::{consumer_key, consumer_secret};
use crate::db::manager::DbManager;
use crate::service::secret::{SecretStorageService, SystemSecretService};
use crate::ui::context_object::ContextObject;
use crate::ui::team_object::TeamObject;
use gtk::glib;
use gtk::prelude::*;
use log::{info, warn};

pub struct SeriesTabController {
    context: ContextObject,
}

impl SeriesTabController {
    pub fn new(context: ContextObject) -> Self {
        Self { context }
    }

    pub fn on_team_selected(&self, team: &TeamObject) {
        let team_id = team.team_data().id;
        info!("SeriesTabController: Loading data for team {}", team_id);
        let ctx = self.context.clone();

        glib::MainContext::default().spawn_local(async move {
            let db = DbManager::new();
            if let Ok(mut conn) = db.get_connection() {
                let team_in_db = crate::db::teams::get_team(&mut conn, team_id).unwrap_or(None);
                if let Some(t) = team_in_db {
                    if let Some(unit) = t.LeagueLevelUnit {
                        let league_unit_id = unit.LeagueLevelUnitID;
                        let db_league = crate::db::series::get_latest_league_details(&mut conn, league_unit_id).unwrap_or(None);
                        let db_matches = crate::db::series::get_latest_matches(&mut conn, team_id).unwrap_or(None);

                        if let (Some(league), Some(matches)) = (db_league, db_matches) {
                            let team_ids: Vec<i32> = league.Teams.iter().filter_map(|t| t.TeamID.parse::<i32>().ok()).collect();
                            let all_matches = crate::db::series::get_matches_for_teams(&mut conn, &team_ids).unwrap_or_default();
                            let logos = crate::db::teams::get_logo_urls_for_teams(&mut conn, &team_ids).unwrap_or_default();
                            let filtered = crate::series::ui::controller::filter_matches_for_season(&league, matches);

                            ctx.set_league_details(Some(league));
                            ctx.set_matches(Some(filtered));
                            ctx.set_all_series_matches(Some(all_matches));
                            ctx.set_series_logo_urls(Some(logos));

                            // Start a separate background task to fetch missing opponent details and matches
                            glib::MainContext::default().spawn_local(async move {
                                let secret_service = SystemSecretService::new();
                                let token_res = secret_service.get_secret("access_token").await;
                                let secret_res = secret_service.get_secret("access_secret").await;

                                if let (Ok(Some(token)), Ok(Some(secret))) = (token_res, secret_res) {
                                    let ck = consumer_key();
                                    let cs = consumer_secret();

                                    let mut conn = match DbManager::new().get_connection() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };

                                    // Create a new download record for this background sync
                                    // to ensure the data is picked up as "latest" by queries.
                                    let download_id = match crate::db::download_entries::create_download(
                                        &mut conn,
                                        &chrono::Utc::now().to_rfc3339(),
                                        "completed",
                                    ) {
                                        Ok(id) => id,
                                        Err(e) => {
                                            warn!("SeriesTabController: Failed to create background download record: {}", e);
                                            0
                                        }
                                    };

                                    let mut updated = false;

                                    for tid in &team_ids {
                                        if *tid == team_id as i32 {
                                            continue;
                                        }

                                        let (oauth_data1, signing_key1) = create_oauth_context(&ck, &cs, &token, &secret);
                                        if let Ok(team_data) = team_details_request(oauth_data1, signing_key1, Some(*tid as u32)).await {
                                            if let Some(team) = team_data.Teams.Teams.first() {
                                                if crate::db::teams::save_team(&mut conn, team, &team_data.User, download_id, false).is_ok() {
                                                    updated = true;
                                                }
                                            }
                                        }

                                        let (oauth_data2, signing_key2) = create_oauth_context(&ck, &cs, &token, &secret);
                                        if let Ok(archived) = matches_archive_request(oauth_data2, signing_key2, Some(*tid as u32), None, None).await {
                                            if crate::db::series::save_matches(&mut conn, download_id, &archived).is_ok() {
                                                updated = true;
                                            }
                                        }
                                    }

                                    if updated {
                                        // Re-query and update ContextObject
                                        let all_matches_new = crate::db::series::get_matches_for_teams(&mut conn, &team_ids).unwrap_or_default();
                                        let logos_new = crate::db::teams::get_logo_urls_for_teams(&mut conn, &team_ids).unwrap_or_default();
                                        
                                        // Update the context object to trigger UI re-renders
                                        ctx.set_all_series_matches(Some(all_matches_new));
                                        ctx.set_series_logo_urls(Some(logos_new));
                                        
                                        // Explicitly notify that data has been updated
                                        ctx.notify("data-loaded");
                                        
                                        info!("SeriesTabController: Background match/logo sync completed for team {}", team_id);
                                    }
                                }
                            });
                        } else {
                            warn!("SeriesTabController: Missing league/matches data in DB for team {}", team_id);
                        }
                    }
                }
            }
        });
    }

    pub fn clear(&self) {
        self.context.set_league_details(None);
        self.context.set_matches(None);
        self.context.set_all_series_matches(None);
        self.context.set_series_logo_urls(None);
    }
}
