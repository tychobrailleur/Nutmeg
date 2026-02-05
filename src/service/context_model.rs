use crate::db::manager::DbManager;
use crate::db::teams::get_players_for_team;
use crate::ui::player_display::PlayerDisplay;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use log::{error, info};
use num_format::SystemLocale;
use std::cell::RefCell;
use std::sync::OnceLock;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ContextModel {
        pub selected_team: RefCell<Option<TeamObject>>,
        pub selected_player: RefCell<Option<PlayerObject>>,
        pub players: RefCell<Option<gtk::ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContextModel {
        const NAME: &'static str = "ContextModel";
        type Type = super::ContextModel;
    }

    impl ObjectImpl for ContextModel {
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
    pub struct ContextModel(ObjectSubclass<imp::ContextModel>);
}

impl ContextModel {
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
        let store = self.create_player_list_store(&[]);
        self.imp().players.replace(Some(store));
        self.notify("players");

        // Clear selected player
        self.set_selected_player(None::<PlayerObject>);
    }

    fn load_context_for_team(&self, team: TeamObject) {
        let team_data = team.team_data();
        let team_id = team_data.id;
        info!("ContextModel: Loading context for team {}", team_id);

        let db = DbManager::new();
        if let Ok(mut conn) = db.get_connection() {
            match get_players_for_team(&mut conn, team_id) {
                Ok(players_data) => {
                    info!("ContextModel: Loaded {} players", players_data.len());
                    let list_store = self.create_player_list_store(&players_data);
                    self.imp().players.replace(Some(list_store));
                    self.notify("players");
                }
                Err(e) => error!("ContextModel: Failed to load players: {}", e),
            }
        }

        // Clear selected player when team changes
        self.set_selected_player(None::<PlayerObject>);
    }

    // Copied/Refactored from window.rs
    fn create_player_list_store(&self, players: &[crate::chpp::model::Player]) -> gtk::ListStore {
        #[allow(deprecated)]
        let store = gtk::ListStore::new(&[
            glib::Type::STRING, // 0 Name
            glib::Type::STRING, // 1 Flag
            glib::Type::STRING, // 2 Number
            glib::Type::STRING, // 3 Age
            glib::Type::STRING, // 4 Form
            glib::Type::STRING, // 5 TSI
            glib::Type::STRING, // 6 Salary
            glib::Type::STRING, // 7 Specialty
            glib::Type::STRING, // 8 Experience
            glib::Type::STRING, // 9 Leadership
            glib::Type::STRING, // 10 Loyalty
            glib::Type::STRING, // 11 Best Position
            glib::Type::STRING, // 12 Last Position
            glib::Type::STRING, // 13 BG Color
            glib::Type::STRING, // 14 Stamina
            glib::Type::STRING, // 15 Injured
            glib::Type::STRING, // 16 Cards
            glib::Type::STRING, // 17 Mother Club
            glib::Type::OBJECT, // 18 PlayerObject
        ]);

        let locale =
            SystemLocale::default().unwrap_or_else(|_| SystemLocale::from_name("C").unwrap());

        for p in players {
            let obj = PlayerObject::new(p.clone());
            let display = PlayerDisplay::new(&p, &locale);

            let bg = if p.MotherClubBonus {
                Some("mother_club_bg")
            } else {
                None
            };

            store.insert_with_values(
                None,
                &[
                    (0, &display.name),
                    (1, &display.flag),
                    (2, &display.number),
                    (3, &display.age),
                    (4, &display.form),
                    (5, &display.tsi),
                    (6, &display.salary),
                    (7, &display.specialty),
                    (8, &display.xp),
                    (9, &display.leadership),
                    (10, &display.loyalty),
                    (11, &display.best_pos),
                    (12, &display.last_pos),
                    (13, &bg),
                    (14, &display.stamina),
                    (15, &display.injured),
                    (16, &display.cards),
                    (17, &display.mother_club),
                    (18, &obj),
                ],
            );
        }
        store
    }
}

impl Default for ContextModel {
    fn default() -> Self {
        Self::new()
    }
}
