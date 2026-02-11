use crate::db::manager::DbManager;
use crate::db::teams::get_players_for_team;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use log::{error, info};
use std::cell::RefCell;
use std::sync::OnceLock;

use crate::squad::player_list::create_player_model;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ContextObject {
        pub selected_team: RefCell<Option<TeamObject>>,
        pub selected_player: RefCell<Option<PlayerObject>>,
        pub players: RefCell<Option<gtk::ListStore>>,
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
                    glib::ParamSpecObject::builder::<TeamObject>("selected-team")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<PlayerObject>("selected-player")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<gtk::ListStore>("players")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-team" => self.selected_team.borrow().to_value(),
                "selected-player" => self.selected_player.borrow().to_value(),
                "players" => self.players.borrow().to_value(),
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

                    // Trigger loading of players
                    if let Some(t) = team {
                        self.obj().load_context_for_team(t);
                    } else {
                        self.obj().clear_context();
                    }
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

    fn notify_selected_team(&self) {
        self.notify("selected-team");
    }

    fn notify_selected_player(&self) {
        self.notify("selected-player");
    }

    pub fn set_selected_player(&self, player: Option<PlayerObject>) {
        self.set_property("selected-player", player);
    }

    fn clear_context(&self) {
        // Clear players list
        let store = create_player_model(&[]);
        self.imp().players.replace(Some(store));
        self.notify("players");

        // Clear selected player
        self.set_selected_player(None::<PlayerObject>);
    }

    fn load_context_for_team(&self, team: TeamObject) {
        let team_data = team.team_data();
        let team_id = team_data.id;
        info!("ContextObject: Loading context for team {}", team_id);

        let db = DbManager::new();
        if let Ok(mut conn) = db.get_connection() {
            match get_players_for_team(&mut conn, team_id) {
                Ok(players_data) => {
                    info!("ContextObject: Loaded {} players", players_data.len());
                    let list_store = create_player_model(&players_data);
                    self.imp().players.replace(Some(list_store));
                    self.notify("players");
                }
                Err(e) => error!("ContextObject: Failed to load players: {}", e),
            }
        }

        // Clear selected player when team changes
        self.set_selected_player(None::<PlayerObject>);
    }
}

impl Default for ContextObject {
    fn default() -> Self {
        Self::new()
    }
}
