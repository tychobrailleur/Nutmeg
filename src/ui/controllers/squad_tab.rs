use crate::rating::model::{Lineup, RatingPredictionModel, Team};
use crate::rating::position_eval::evaluate_all_positions;
use crate::rating::types::{Attitude, Location, TacticType, Weather};
use crate::ui::context_object::ContextObject;
use crate::ui::player_display::PlayerDisplay;
use crate::ui::player_object::PlayerObject;
use gtk::glib;
use log::{debug, warn};
use num_format::SystemLocale;

pub struct SquadTabController {
    context: ContextObject,
}

impl SquadTabController {
    pub fn new(context: ContextObject) -> Self {
        Self { context }
    }

    pub fn clear(&self) {
        // No-op: ContextObject clears its own properties.
    }

    /// Format position for display
    fn format_position_display(
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

    /// Calculate preferred position for a player
    fn calculate_preferred_position(player: &crate::chpp::model::Player) -> String {
        let team = Team::default();
        let model = RatingPredictionModel::new(team);

        let lineup = Lineup {
            positions: vec![],
            weather: Weather::Neutral,
            tactic: TacticType::Normal,
            attitude: Attitude::Normal,
            location: Location::Home,
        };

        let evaluation = evaluate_all_positions(&model, player, &lineup, 45);

        if let Some(skills) = &player.PlayerSkills {
            debug!(
                "Player {} {} - Form={} Skills: K={} D={} PM={} W={} P={} S={}",
                player.FirstName,
                player.LastName,
                player.PlayerForm,
                skills.KeeperSkill,
                skills.DefenderSkill,
                skills.PlaymakerSkill,
                skills.WingerSkill,
                skills.PassingSkill,
                skills.ScorerSkill
            );
        } else {
            warn!(
                "Player {} {} - NO SKILLS DATA!",
                player.FirstName, player.LastName
            );
        }

        if let Some(best) = evaluation.best_position {
            debug!(
                "  Best position: {:?} ({:?}) with rating {:.2}",
                best.position, best.behaviour, best.rating
            );
            Self::format_position_display(&best.position, &best.behaviour)
        } else {
            warn!(
                "  No best position found for player {} {}",
                player.FirstName, player.LastName
            );
            "-".to_string()
        }
    }

    pub fn create_player_list_store(players: &[crate::chpp::model::Player]) -> gtk::ListStore {
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
            let preferred_pos = Self::calculate_preferred_position(p);
            let display = PlayerDisplay::new(p, &locale, Some(&preferred_pos));

            let bg = if p.MotherClubBonus {
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
