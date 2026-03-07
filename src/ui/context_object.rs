use crate::rating::model::{Lineup, RatingPredictionModel, Team};
use crate::rating::position_eval::evaluate_all_positions;
use crate::rating::types::{Attitude, Location, TacticType, Weather};
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

use crate::db::manager::DbManager;
use crate::db::teams::get_players_for_team;
use crate::squad::ui::player_list::create_player_model;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ContextObject {
        pub selected_team: RefCell<Option<TeamObject>>,
        pub selected_player: RefCell<Option<PlayerObject>>,
        pub players: RefCell<Option<gtk::ListStore>>,
        pub opponent_avg_ratings: RefCell<Option<[f32; 7]>>,
        pub best_lineups: RefCell<Option<Vec<crate::rating::optimiser::OptimisedLineup>>>,
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
                    glib::ParamSpecBoolean::builder("has-opponent-ratings")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("has-best-lineups")
                        .explicit_notify()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-team" => self.selected_team.borrow().to_value(),
                "selected-player" => self.selected_player.borrow().to_value(),
                "players" => self.players.borrow().to_value(),
                "has-opponent-ratings" => self.opponent_avg_ratings.borrow().is_some().to_value(),
                "has-best-lineups" => self.best_lineups.borrow().is_some().to_value(),
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

    pub fn set_opponent_avg_ratings(&self, ratings: Option<[f32; 7]>) {
        *self.imp().opponent_avg_ratings.borrow_mut() = ratings;
        self.notify("has-opponent-ratings");
    }

    pub fn get_opponent_avg_ratings(&self) -> Option<[f32; 7]> {
        *self.imp().opponent_avg_ratings.borrow()
    }

    pub fn set_best_lineups(&self, lineups: Option<Vec<crate::rating::optimiser::OptimisedLineup>>) {
        *self.imp().best_lineups.borrow_mut() = lineups;
        self.notify("has-best-lineups");
    }

    pub fn best_lineups(&self) -> Option<Vec<crate::rating::optimiser::OptimisedLineup>> {
        self.imp().best_lineups.borrow().clone()
    }

    fn clear_context(&self) {
        // Clear players list
        let store = create_player_model(&[]);
        self.imp().players.replace(Some(store));
        self.notify("players");

        self.set_opponent_avg_ratings(None);
        self.set_best_lineups(None);

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
                    // Use the local method that includes rating evaluation
                    let list_store = self.create_player_list_store(&players_data);
                    self.imp().players.replace(Some(list_store));
                    self.notify("players");
                }
                Err(e) => error!("ContextObject: Failed to load players: {}", e),
            }
        }

        // Clear selected player when team changes
        self.set_selected_player(None::<PlayerObject>);
    }

    /// Calculate preferred position for a player
    fn calculate_preferred_position(&self, player: &crate::chpp::model::Player) -> String {
        // Use default team settings for evaluation
        let team = Team::default();
        let model = RatingPredictionModel::new(team);

        // Default lineup context
        let lineup = Lineup {
            positions: vec![],
            weather: Weather::Neutral,
            tactic: TacticType::Normal,
            attitude: Attitude::Normal,
            location: Location::Home,
        };

        // Evaluate at minute 45 (mid-game)
        let evaluation = evaluate_all_positions(&model, player, &lineup, 45);

        // Debug logging
        if let Some(skills) = &player.PlayerSkills {
            log::debug!(
                "Player {} {} - Form={} Skills: K={} D={} PM={} W={} P={} S={}",
                player.FirstName,
                player.LastName,
                player.PlayerForm, // <-- ADDED FORM HERE
                skills.KeeperSkill,
                skills.DefenderSkill,
                skills.PlaymakerSkill,
                skills.WingerSkill,
                skills.PassingSkill,
                skills.ScorerSkill
            );
        } else {
            log::warn!(
                "Player {} {} - NO SKILLS DATA!",
                player.FirstName,
                player.LastName
            );
        }
        log::debug!("  Evaluating {} positions", evaluation.positions.len());

        // Format best position
        if let Some(best) = evaluation.best_position {
            log::debug!(
                "  Best position: {:?} ({:?}) with rating {:.2}",
                best.position,
                best.behaviour,
                best.rating
            );
            self.format_position_display(&best.position, &best.behaviour)
        } else {
            log::warn!(
                "  No best position found for player {} {}",
                player.FirstName,
                player.LastName
            );
            "-".to_string()
        }
    }

    /// Format position for display
    fn format_position_display(
        &self,
        position: &crate::rating::types::PositionId,
        behaviour: &crate::rating::types::Behaviour,
    ) -> String {
        use crate::rating::types::{Behaviour, PositionId};
        use gettextrs::gettext;

        let pos_name = match position {
            PositionId::Keeper => gettext("Keeper"),
            PositionId::LeftBack => gettext("Left Back"),
            PositionId::LeftCentralDefender => gettext("Left CD"),
            PositionId::MiddleCentralDefender => gettext("Central Defender"),
            PositionId::RightCentralDefender => gettext("Right CD"),
            PositionId::RightBack => gettext("Right Back"),
            PositionId::LeftWinger => gettext("Left Winger"),
            PositionId::LeftInnerMidfield => gettext("Left IM"),
            PositionId::CentralInnerMidfield => gettext("Central IM"),
            PositionId::RightInnerMidfield => gettext("Right IM"),
            PositionId::RightWinger => gettext("Right Winger"),
            PositionId::LeftForward => gettext("Left Forward"),
            PositionId::CentralForward => gettext("Central Forward"),
            PositionId::RightForward => gettext("Right Forward"),
            PositionId::SetPieces => gettext("Set Pieces"),
        };

        match behaviour {
            Behaviour::Normal => pos_name,
            Behaviour::Offensive => format!("{} ({})", pos_name, gettext("Off")),
            Behaviour::Defensive => format!("{} ({})", pos_name, gettext("Def")),
            Behaviour::TowardsMiddle => format!("{} ({})", pos_name, gettext("TM")),
            Behaviour::TowardsWing => format!("{} ({})", pos_name, gettext("TW")),
        }
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
            glib::Type::STRING, // 13 BG Colour
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

            // Calculate preferred position
            let preferred_pos = self.calculate_preferred_position(p);

            let display = PlayerDisplay::new(p, &locale, Some(&preferred_pos));

            // Get the actual background colour from CSS by creating a styled widget
            let bg = if p.MotherClubBonus {
                // Create a temporary widget with the CSS class to extract the colour
                let temp_widget = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                temp_widget.add_css_class("mother-club");

                // Force style resolution
                #[allow(deprecated)]
                let _style_context = temp_widget.style_context();

                // Try to get background-colour property
                // Since we can't easily query CSS properties in GTK4, use the hardcoded value
                // that matches the CSS definition
                // FIXME: is there really no way to avoid this hardcoded value??!
                Some("rgba(64, 224, 208, 0.3)".to_string())
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

impl Default for ContextObject {
    fn default() -> Self {
        Self::new()
    }
}
