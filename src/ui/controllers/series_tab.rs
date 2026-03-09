use crate::db::manager::DbManager;
use crate::series::ui::controller::filter_matches_for_season;
use crate::ui::context_object::ContextObject;
use crate::ui::team_object::TeamObject;
use log::{info, warn};

pub struct SeriesTabController {
    context: ContextObject,
}

impl SeriesTabController {
    pub fn new(context: ContextObject) -> Self {
        Self { context }
    }

    /// Loads series data for the selected team eagerly from the local DB.
    ///
    /// All reads are synchronous — SQLite lookups are fast enough to run on the GTK
    /// main thread without perceptible lag. Only the logo image download (URL → pixel
    /// data) remains asynchronous; that happens later in the column-bind callback of
    /// `SeriesPage::add_badge_name_column`.
    ///
    /// All four series context fields are written atomically via `set_series_data` so
    /// the series page redraws exactly once.
    pub fn on_team_selected(&self, team: &TeamObject) {
        let team_id = team.team_data().id;
        info!("SeriesTabController: Loading data for team {}", team_id);

        let db = DbManager::new();
        let Ok(mut conn) = db.get_connection() else {
            warn!("SeriesTabController: Could not open DB connection for team {}", team_id);
            return;
        };

        let team_in_db = crate::db::teams::get_team(&mut conn, team_id).unwrap_or(None);
        let Some(t) = team_in_db else {
            warn!("SeriesTabController: Team {} not found in DB", team_id);
            return;
        };
        let Some(unit) = t.LeagueLevelUnit else {
            warn!("SeriesTabController: No LeagueLevelUnit for team {}", team_id);
            return;
        };

        let league_unit_id = unit.LeagueLevelUnitID;
        let db_league =
            crate::db::series::get_latest_league_details(&mut conn, league_unit_id)
                .unwrap_or(None);
        let db_matches =
            crate::db::series::get_latest_matches(&mut conn, team_id).unwrap_or(None);

        let (Some(league), Some(matches)) = (db_league, db_matches) else {
            warn!(
                "SeriesTabController: Missing league/matches data in DB for team {}",
                team_id
            );
            return;
        };

        let team_ids: Vec<i32> = league
            .Teams
            .iter()
            .filter_map(|t| t.TeamID.parse::<i32>().ok())
            .collect();

        let all_matches =
            crate::db::series::get_matches_for_teams(&mut conn, &team_ids).unwrap_or_default();
        let logos =
            crate::db::teams::get_logo_urls_for_teams(&mut conn, &team_ids).unwrap_or_default();

        let filtered = filter_matches_for_season(&league, matches);

        self.context.set_series_data(
            Some(league),
            Some(filtered),
            Some(all_matches),
            Some(logos),
        );
    }

    pub fn clear(&self) {
        self.context.set_series_data(None, None, None, None);
    }
}
