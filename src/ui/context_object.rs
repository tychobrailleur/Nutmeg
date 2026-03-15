use crate::squad::ui::player_list::create_player_model;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::sync::OnceLock;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ContextObject {
        // Source of truth for all available items
        pub all_teams: RefCell<Option<gtk::gio::ListStore>>,

        // Active State
        pub selected_team: RefCell<Option<TeamObject>>,
        pub selected_player: RefCell<Option<PlayerObject>>,

        // Data populated per-team
        pub upcoming_opponents: RefCell<Option<gtk::gio::ListStore>>,
        pub players: RefCell<Option<gtk::ListStore>>,
        pub opponent_avg_ratings: RefCell<Option<[f32; 7]>>,
        pub best_lineups: RefCell<Option<Vec<crate::rating::optimiser::OptimisedLineup>>>,
        pub league_details: RefCell<Option<crate::chpp::model::LeagueDetailsData>>,
        pub matches: RefCell<Option<crate::chpp::model::MatchesData>>,
        pub all_series_matches: RefCell<Option<Vec<crate::chpp::model::MatchDetails>>>,
        pub series_logo_urls: RefCell<Option<std::collections::HashMap<i32, String>>>,
        pub prediction_result: RefCell<Option<crate::rating::match_predictor::PredictionResult>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContextObject {
        const NAME: &'static str = "ContextObject";
        type Type = super::ContextObject;
    }

    impl ObjectImpl for ContextObject {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<gtk::gio::ListStore>("all-teams")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<TeamObject>("selected-team")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<PlayerObject>("selected-player")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<gtk::ListStore>("players")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<gtk::gio::ListStore>("upcoming-opponents")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("has-opponent-ratings")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("has-best-lineups")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("data-loaded")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("has-prediction")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("prediction-text")
                        .read_only()
                        .build(),
                    glib::ParamSpecDouble::builder("win-probability")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "all-teams" => self.all_teams.borrow().to_value(),
                "selected-team" => self.selected_team.borrow().to_value(),
                "selected-player" => self.selected_player.borrow().to_value(),
                "players" => self.players.borrow().to_value(),
                "upcoming-opponents" => self.upcoming_opponents.borrow().to_value(),
                "has-opponent-ratings" => self.opponent_avg_ratings.borrow().is_some().to_value(),
                "has-best-lineups" => self.best_lineups.borrow().is_some().to_value(),
                "data-loaded" => (self.league_details.borrow().is_some()
                    && self.matches.borrow().is_some())
                .to_value(),
                "has-prediction" => self.prediction_result.borrow().is_some().to_value(),
                "prediction-text" => {
                    let res = self.prediction_result.borrow();
                    let lineups = self.best_lineups.borrow();
                    if let (Some(r), Some(l)) = (res.as_ref(), lineups.as_ref()) {
                        if !l.is_empty() {
                            let best = &l[0];
                            format!(
                                "<b>{} ({})</b>\nWin: {:.1}% | Draw: {:.1}% | Loss: {:.1}%",
                                best.formation.name(),
                                best.tactic.name(),
                                r.win_prob * 100.0,
                                r.draw_prob * 100.0,
                                r.loss_prob * 100.0
                            )
                            .to_value()
                        } else {
                            "No data yet.".to_value()
                        }
                    } else {
                        "No data yet.".to_value()
                    }
                }
                "win-probability" => {
                    let res = self.prediction_result.borrow();
                    if let Some(r) = res.as_ref() {
                        r.win_prob.to_value()
                    } else {
                        0.0f64.to_value()
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "selected-team" => {
                    let team = value.get::<Option<TeamObject>>().ok().flatten();
                    let mut current_team = self.selected_team.borrow_mut();

                    *current_team = team.clone();
                    drop(current_team); // release borrow before notifying/loading

                    self.obj().notify_selected_team();
                }
                "selected-player" => {
                    let player = value.get::<Option<PlayerObject>>().ok().flatten();
                    self.selected_player.replace(player);
                    self.obj().notify_selected_player();
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            // Setup automatic refresh when team changes
            let obj = self.obj();
            obj.connect_notify_local(Some("selected-team"), |ctx, _| {
                ctx.refresh_from_db();
            });
        }
    }
}

glib::wrapper! {
    pub struct ContextObject(ObjectSubclass<imp::ContextObject>);
}

impl ContextObject {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn selected_team(&self) -> Option<TeamObject> {
        self.property::<Option<TeamObject>>("selected-team")
    }

    fn notify_selected_team(&self) {
        self.notify("selected-team");
    }

    fn notify_selected_player(&self) {
        self.notify("selected-player");
    }

    pub fn set_selected_player(&self, player: Option<PlayerObject>) {
        self.set_property("selected-player", player);
    }

    pub fn set_upcoming_opponents(&self, store: Option<gtk::gio::ListStore>) {
        self.imp().upcoming_opponents.replace(store);
        self.notify("upcoming-opponents");
    }

    pub fn set_opponent_avg_ratings(&self, ratings: Option<[f32; 7]>) {
        *self.imp().opponent_avg_ratings.borrow_mut() = ratings;
        self.notify("has-opponent-ratings");
    }

    pub fn get_opponent_avg_ratings(&self) -> Option<[f32; 7]> {
        *self.imp().opponent_avg_ratings.borrow()
    }

    pub fn set_best_lineups(
        &self,
        lineups: Option<Vec<crate::rating::optimiser::OptimisedLineup>>,
    ) {
        *self.imp().best_lineups.borrow_mut() = lineups;
        self.notify("has-best-lineups");
    }

    pub fn best_lineups(&self) -> Option<Vec<crate::rating::optimiser::OptimisedLineup>> {
        self.imp().best_lineups.borrow().clone()
    }

    pub fn league_details(&self) -> Option<crate::chpp::model::LeagueDetailsData> {
        self.imp().league_details.borrow().clone()
    }

    pub fn matches(&self) -> Option<crate::chpp::model::MatchesData> {
        self.imp().matches.borrow().clone()
    }

    pub fn all_series_matches(&self) -> Option<Vec<crate::chpp::model::MatchDetails>> {
        self.imp().all_series_matches.borrow().clone()
    }

    pub fn series_logo_urls(&self) -> Option<std::collections::HashMap<i32, String>> {
        self.imp().series_logo_urls.borrow().clone()
    }

    pub fn set_prediction_result(
        &self,
        result: Option<crate::rating::match_predictor::PredictionResult>,
    ) {
        *self.imp().prediction_result.borrow_mut() = result;
        self.notify("has-prediction");
        self.notify("prediction-text");
        self.notify("win-probability");
    }

    pub fn prediction_result(&self) -> Option<crate::rating::match_predictor::PredictionResult> {
        self.imp().prediction_result.borrow().clone()
    }

    fn clear_context(&self) {
        // Clear players list
        let store = create_player_model(&[]);
        self.imp().players.replace(Some(store));
        self.notify("players");

        self.set_opponent_avg_ratings(None);
        self.set_best_lineups(None);

        self.imp().league_details.replace(None);
        self.imp().matches.replace(None);
        self.imp().all_series_matches.replace(None);
        self.imp().series_logo_urls.replace(None);
        self.set_prediction_result(None);
        self.notify("data-loaded");

        // Clear selected player
        self.set_selected_player(None::<PlayerObject>);
    }

    pub fn set_all_teams(&self, store: Option<gtk::gio::ListStore>) {
        self.imp().all_teams.replace(store);
        self.notify("all-teams");
    }

    pub fn set_players(&self, store: Option<gtk::ListStore>) {
        self.imp().players.replace(store);
        self.notify("players");
    }

    /// Sets all four series fields atomically and fires `data-loaded` exactly once.
    ///
    /// Prefer this over calling the individual setters to avoid triggering multiple
    /// partial redraws of the series page.
    pub fn set_series_data(
        &self,
        league: Option<crate::chpp::model::LeagueDetailsData>,
        matches: Option<crate::chpp::model::MatchesData>,
        all_matches: Option<Vec<crate::chpp::model::MatchDetails>>,
        logo_urls: Option<std::collections::HashMap<i32, String>>,
    ) {
        self.imp().league_details.replace(league);
        self.imp().matches.replace(matches);
        self.imp().all_series_matches.replace(all_matches);
        self.imp().series_logo_urls.replace(logo_urls);
        self.notify("data-loaded");
    }

    /// Centralised method to refresh all in-memory snapshot data from the database
    /// for the currently selected team.
    pub fn refresh_from_db(&self) {
        let Some(team) = self.selected_team() else {
            self.clear_context();
            return;
        };

        let team_id = team.team_data().id;
        log::info!("ContextObject: Refreshing snapshot for team {}", team_id);

        let db = crate::db::manager::DbManager::new();
        let Ok(mut conn) = db.get_connection() else {
            log::warn!("ContextObject: Failed to connect to DB for refresh");
            return;
        };

        // 1. Players
        match crate::db::teams::get_players_for_team(&mut conn, team_id) {
            Ok(players) => {
                let store =
                    crate::ui::controllers::squad_tab::SquadTabController::create_player_list_store(
                        &players,
                    );
                self.set_players(Some(store));

                let weak_self = self.downgrade();
                glib::MainContext::default().spawn_local(async move {
                    let results = tokio::task::spawn_blocking(move || {
                        crate::rating::controller::RatingController::calculate_best_lineups(
                            &players,
                        )
                    })
                    .await
                    .unwrap_or_default();

                    if let Some(obj) = weak_self.upgrade() {
                        obj.set_best_lineups(Some(results));
                    }
                });
            }
            Err(e) => log::error!("ContextObject: Failed to load players: {}", e),
        }

        // 2. Series Data
        let mut league_unit_id = None;
        if let Ok(Some(t)) = crate::db::teams::get_team(&mut conn, team_id) {
            if let Some(unit) = t.LeagueLevelUnit {
                league_unit_id = Some(unit.LeagueLevelUnitID);
            }
        }

        if let Some(unit_id) = league_unit_id {
            let db_league = crate::db::series::get_latest_league_details(&mut conn, unit_id)
                .ok()
                .flatten();
            let db_matches = crate::db::series::get_latest_matches(&mut conn, team_id)
                .ok()
                .flatten();

            if let (Some(league), Some(matches)) = (db_league, db_matches) {
                let team_ids: Vec<i32> = league
                    .Teams
                    .iter()
                    .filter_map(|t| t.TeamID.parse::<i32>().ok())
                    .collect();
                let all_matches = crate::db::series::get_matches_for_teams(&mut conn, &team_ids)
                    .unwrap_or_default();
                let logos = crate::db::teams::get_logo_urls_for_teams(&mut conn, &team_ids)
                    .unwrap_or_default();
                let filtered =
                    crate::series::ui::controller::filter_matches_for_season(&league, matches);

                self.set_series_data(Some(league), Some(filtered), Some(all_matches), Some(logos));
            }
        }

        // 3. Upcoming Opponents
        let list_store =
            gtk::gio::ListStore::new::<crate::opponent_analysis::ui::model::OpponentItem>();
        if let Ok(opponents) = crate::db::series::get_upcoming_opponents_from_db(&mut conn, team_id)
        {
            for opp in opponents {
                let logo_url = format!(
                    "https://res.hattrick.org/teamlogo/{}/{}/{}/{}/{}.png",
                    opp.team_id % 10,
                    opp.team_id % 100,
                    opp.team_id % 1000,
                    opp.team_id,
                    opp.team_id
                );
                let item = crate::opponent_analysis::ui::model::OpponentItem::new(
                    opp.team_id,
                    &opp.team_name,
                    &opp.match_date,
                    &logo_url,
                );
                list_store.append(&item);
            }
        }
        self.set_upcoming_opponents(Some(list_store));
    }
}

impl Default for ContextObject {
    fn default() -> Self {
        Self::new()
    }
}
