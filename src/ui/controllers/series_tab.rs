use crate::db::manager::DbManager;
use crate::ui::context_object::ContextObject;
use crate::ui::team_object::TeamObject;
use gtk::glib;
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
