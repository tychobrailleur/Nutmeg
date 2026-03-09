use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::sync::OnceLock;

use crate::squad::ui::player_list::create_player_model;

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
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "selected-team" => {
                    let team = value
                        .get::<Option<TeamObject>>()
                        .expect("Value must be TeamObject");
                    let mut current_team = self.selected_team.borrow_mut();

                    *current_team = team.clone();
                    drop(current_team); // release borrow before notifying/loading

                    self.obj().notify_selected_team();
                }
                "selected-player" => {
                    let player = value
                        .get::<Option<PlayerObject>>()
                        .expect("Value must be PlayerObject");
                    self.selected_player.replace(player);
                    self.obj().notify_selected_player();
                }
                _ => unimplemented!(),
            }
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
}

impl Default for ContextObject {
    fn default() -> Self {
        Self::new()
    }
}
