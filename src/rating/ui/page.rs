use crate::chpp::model::Player;
use crate::rating::controller::RatingController;
use crate::rating::model::Lineup;
use crate::rating::optimiser::{Formation, LineupOptimiser, OptimisedLineup};
use crate::rating::types::{
    Attitude, Behaviour, Location, PositionId, RatingSector, TacticType, Weather,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use log::{debug, error, info};
use num_format::{SystemLocale, ToFormattedString};
use std::cell::RefCell;
use std::collections::HashMap;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="FormationOptimiserWidget" parent="GtkBox">
        <property name="orientation">vertical</property>
        <property name="spacing">12</property>
        <property name="margin-top">12</property>
        <property name="margin-bottom">12</property>
        <property name="margin-start">12</property>
        <property name="margin-end">12</property>

        <child>
          <object class="GtkBox">
            <property name="orientation">horizontal</property>
            <property name="spacing">12</property>

            <child>
              <object class="GtkLabel">
                <property name="label" translatable="yes">Formation Optimiser</property>
                <property name="css-classes">title-1</property>
                <property name="halign">start</property>
                <property name="hexpand">true</property>
              </object>
            </child>

            <child>
              <object class="GtkButton" id="calculate_button">
                <property name="label" translatable="yes">Calculate Best Lineups</property>
                <property name="css-classes">suggested-action</property>
              </object>
            </child>
          </object>
        </child>

        <child>
            <object class="GtkScrolledWindow">
                <property name="vexpand">true</property>
                <property name="hexpand">true</property>
                <property name="hscrollbar-policy">never</property>
                <child>
                    <object class="GtkFlowBox" id="formations_flowbox">
                        <property name="valign">start</property>
                        <property name="max-children-per-line">5</property>
                        <property name="min-children-per-line">1</property>
                        <property name="selection-mode">none</property>
                        <property name="row-spacing">12</property>
                        <property name="column-spacing">12</property>
                    </object>
                </child>
            </object>
        </child>

      </template>
    </interface>
    "#)]
    pub struct FormationOptimiserWidget {
        #[template_child]
        pub calculate_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub formations_flowbox: TemplateChild<gtk::FlowBox>,

        pub players: RefCell<Vec<Player>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FormationOptimiserWidget {
        const NAME: &'static str = "FormationOptimiserWidget";
        type Type = super::FormationOptimiserWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FormationOptimiserWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_callbacks();
            self.obj().populate_initial_cards();
        }
    }

    impl WidgetImpl for FormationOptimiserWidget {}
    impl BoxImpl for FormationOptimiserWidget {}
}

glib::wrapper! {
    pub struct FormationOptimiserWidget(ObjectSubclass<imp::FormationOptimiserWidget>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl FormationOptimiserWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_players(&self, players: Vec<Player>) {
        let imp = self.imp();
        *imp.players.borrow_mut() = players;
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();
        let obj = self.downgrade();
        imp.calculate_button.connect_clicked(move |btn| {
            if let Some(obj) = obj.upgrade() {
                obj.on_calculate_clicked(btn);
            }
        });
    }

    fn populate_initial_cards(&self) {
        let flowbox = self.imp().formations_flowbox.get();
        // Remove existing items if any (though constructed is called once)
        flowbox.remove_all();

        for formation in Formation::all() {
            let empty_lineup = OptimisedLineup {
                formation,
                lineup: Lineup {
                    positions: vec![],
                    weather: Weather::default(),
                    tactic: TacticType::Normal,
                    attitude: Attitude::Normal,
                    location: Location::default(),
                },
                sector_ratings: HashMap::new(),
                player_ratings: HashMap::new(),
                hatstats: 0.0,
                captain: None,
                set_pieces_taker: None,
            };
            let card = self.create_formation_card(&empty_lineup);
            flowbox.append(&card);
        }
    }

    fn on_calculate_clicked(&self, _button: &gtk::Button) {
        let imp = self.imp();
        let players = imp.players.borrow().clone();

        info!("Optimiser started with {} players", players.len());
        if let Some(p) = players.first() {
            debug!(
                "Sample player: {} {} (Form: {})",
                p.FirstName, p.LastName, p.PlayerForm
            );
            // Skills check omitted for brevity in controller call but good for debug
        }

        imp.calculate_button.set_sensitive(false);
        // Do NOT remove items here, so user sees them while calculating
        // imp.formations_flowbox.remove_all();

        let weak_self = self.downgrade();

        glib::MainContext::default().spawn_local(async move {
            let result = tokio::task::spawn_blocking(move || {
                // Use Controller
                RatingController::calculate_best_lineups(&players)
            })
            .await;

            if let Some(obj) = weak_self.upgrade() {
                obj.imp().calculate_button.set_sensitive(true);
                match result {
                    Ok(results) => obj.display_results(results),
                    Err(e) => error!("Optimisation task failed: {}", e),
                }
            }
        });
    }

    fn display_results(&self, results: Vec<OptimisedLineup>) {
        let flowbox = self.imp().formations_flowbox.get();
        flowbox.remove_all();

        for result in results {
            let card = self.create_formation_card(&result);
            flowbox.append(&card);
        }
    }

    fn create_formation_card(&self, result: &OptimisedLineup) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        card.add_css_class("card");
        card.set_margin_top(6);
        card.set_margin_bottom(6);
        card.set_margin_start(6);
        card.set_margin_end(6);
        card.set_width_request(400);

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let title = gtk::Label::builder()
            .label(result.formation.name())
            .css_classes(["heading"])
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();

        // Format Hatstats with locale
        let locale =
            SystemLocale::default().unwrap_or_else(|_| SystemLocale::from_name("C").unwrap());
        // Round to 1 decimal place properly
        let val_rounded = (result.hatstats * 10.0).round() / 10.0;
        let int_part = val_rounded.trunc() as i64;
        let frac_part = (val_rounded.fract() * 10.0).round().abs() as i64;
        let formatted_int = int_part.to_formatted_string(&locale);
        // Note: num-format is for integers. We assume '.' for decimal or hardcode it for now.
        // Usually locales with space/dot grouping come with comma decimal, but sticking to dot for consistency with other parts if any.
        // User asked for "thousand separators", didn't explicitly demand decimal separator change, but "locale" implies it.
        // For simplicity and safety (not breaking if locale fails), we use dot.
        // Improvement: could check locale name but SystemLocale hides details.
        let hatstats_text = format!("{}.{}", formatted_int, frac_part);

        let hatstats = gtk::Label::builder()
            .label(&hatstats_text)
            .css_classes(["accent"])
            .halign(gtk::Align::End)
            .tooltip_text("HatStats")
            .build();

        header.append(&title);
        header.append(&hatstats);
        card.append(&header);

        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Ratings Grid
        let ratings_box = gtk::Box::new(gtk::Orientation::Vertical, 4);

        // Helper to add row
        let add_row = |label: &str, value: f64| {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);

            let lbl = gtk::Label::builder()
                .label(label)
                .halign(gtk::Align::Start)
                .build();

            // Leader
            let leader = gtk::Label::builder()
                .label(" . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .")
                .css_classes(["dim-label"])
                .hexpand(true)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .halign(gtk::Align::Fill)
                .build();

            let val = gtk::Label::builder()
                .label(&format!("{:.2}", value))
                .halign(gtk::Align::End)
                .build();

            row.append(&lbl);
            row.append(&leader);
            row.append(&val);

            ratings_box.append(&row);
        };

        add_row(
            "Midfield",
            *result
                .sector_ratings
                .get(&RatingSector::Midfield)
                .unwrap_or(&0.0),
        );
        add_row(
            "Def R",
            *result
                .sector_ratings
                .get(&RatingSector::DefenceRight)
                .unwrap_or(&0.0),
        );
        add_row(
            "Def C",
            *result
                .sector_ratings
                .get(&RatingSector::DefenceCentral)
                .unwrap_or(&0.0),
        );
        add_row(
            "Def L",
            *result
                .sector_ratings
                .get(&RatingSector::DefenceLeft)
                .unwrap_or(&0.0),
        );
        add_row(
            "Att R",
            *result
                .sector_ratings
                .get(&RatingSector::AttackRight)
                .unwrap_or(&0.0),
        );
        add_row(
            "Att C",
            *result
                .sector_ratings
                .get(&RatingSector::AttackCentral)
                .unwrap_or(&0.0),
        );
        add_row(
            "Att L",
            *result
                .sector_ratings
                .get(&RatingSector::AttackLeft)
                .unwrap_or(&0.0),
        );

        card.append(&ratings_box);

        // Roles Section
        let roles_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        roles_box.set_margin_top(8);
        roles_box.set_margin_bottom(8);

        let add_role = |label: &str, player: Option<&Player>| {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let lbl = gtk::Label::builder()
                .label(label)
                .halign(gtk::Align::Start)
                .css_classes(["dim-label"])
                .build();
            let name_text = player
                .map(|p| format!("{} {}", p.FirstName, p.LastName))
                .unwrap_or_else(|| "-".to_string());
            let val = gtk::Label::builder()
                .label(&name_text)
                .halign(gtk::Align::End)
                .hexpand(true)
                .build();
            row.append(&lbl);
            row.append(&val);
            roles_box.append(&row);
        };

        add_role("Captain", result.captain.as_ref());
        add_role("Set Pieces", result.set_pieces_taker.as_ref());

        card.append(&roles_box);

        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Visual Lineup
        let visual_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        visual_box.set_vexpand(true);
        visual_box.add_css_class("pitch-view");

        // Use SizeGroupMode::Both for uniform width AND height
        let size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

        // Row 1: Attack (3 slots)
        let row_att = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_att.set_halign(gtk::Align::Center);
        row_att.append(&self.create_player_slot(
            result,
            PositionId::LeftForward,
            "LF",
            &size_group,
        ));
        row_att.append(&self.create_player_slot(
            result,
            PositionId::CentralForward,
            "CF",
            &size_group,
        ));
        row_att.append(&self.create_player_slot(
            result,
            PositionId::RightForward,
            "RF",
            &size_group,
        ));
        visual_box.append(&row_att);

        // Row 2: Midfield (5 slots)
        let row_mid = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_mid.set_halign(gtk::Align::Center);
        row_mid.append(&self.create_player_slot(result, PositionId::LeftWinger, "LW", &size_group));
        row_mid.append(&self.create_player_slot(
            result,
            PositionId::LeftInnerMidfield,
            "LIM",
            &size_group,
        ));
        row_mid.append(&self.create_player_slot(
            result,
            PositionId::CentralInnerMidfield,
            "MIM",
            &size_group,
        ));
        row_mid.append(&self.create_player_slot(
            result,
            PositionId::RightInnerMidfield,
            "RIM",
            &size_group,
        ));
        row_mid.append(&self.create_player_slot(
            result,
            PositionId::RightWinger,
            "RW",
            &size_group,
        ));
        visual_box.append(&row_mid);

        // Row 3: Defense (5 slots)
        let row_def = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_def.set_halign(gtk::Align::Center);
        row_def.append(&self.create_player_slot(result, PositionId::LeftBack, "LB", &size_group));
        row_def.append(&self.create_player_slot(
            result,
            PositionId::LeftCentralDefender,
            "LCD",
            &size_group,
        ));
        row_def.append(&self.create_player_slot(
            result,
            PositionId::MiddleCentralDefender,
            "MCD",
            &size_group,
        ));
        row_def.append(&self.create_player_slot(
            result,
            PositionId::RightCentralDefender,
            "RCD",
            &size_group,
        ));
        row_def.append(&self.create_player_slot(result, PositionId::RightBack, "RB", &size_group));
        visual_box.append(&row_def);

        // Row 4: Keeper (1 slot)
        let row_gk = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_gk.set_halign(gtk::Align::Center);

        let gk_slot = self.create_player_slot(result, PositionId::Keeper, "GK", &size_group);
        row_gk.append(&gk_slot);
        visual_box.append(&row_gk);

        card.append(&visual_box);

        card.upcast()
    }

    fn create_player_slot(
        &self,
        result: &OptimisedLineup,
        id: PositionId,
        label_text: &str,
        size_group: &gtk::SizeGroup,
    ) -> gtk::Widget {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_valign(gtk::Align::Center);
        size_group.add_widget(&container);

        // Header Box (Position + Triangle)
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        header_box.set_halign(gtk::Align::Center);

        // Slot Label (e.g. LF)
        let slot_label = gtk::Label::builder()
            .label(label_text)
            .css_classes(["caption", "dim-label"])
            .build();
        header_box.append(&slot_label);

        if let Some(pos) = result.lineup.positions.iter().find(|p| p.role_id == id) {
            let player = &pos.player;
            // Player Name (Last Name / Initial)
            let sort_name = format!(
                "{}. {}",
                player.FirstName.chars().next().unwrap_or('?'),
                player.LastName
            );

            // Behaviour Indicator
            let behaviour_icon = match pos.behaviour {
                Behaviour::Normal => None,
                Behaviour::Offensive => Some("▲"),
                Behaviour::Defensive => Some("▼"),
                Behaviour::TowardsMiddle => {
                    if id.is_left_side() {
                        Some("▶")
                    } else if id.is_right_side() {
                        Some("◀")
                    } else {
                        None
                    }
                }
                Behaviour::TowardsWing => {
                    if id.is_left_side() {
                        Some("◀")
                    } else if id.is_right_side() {
                        Some("▶")
                    } else {
                        Some("↔")
                    }
                }
            };

            if let Some(icon) = behaviour_icon {
                let icon_label = gtk::Label::builder()
                    .label(icon)
                    .css_classes(["caption", "orange-label"]) // Applied orange-label class
                    // Fallback to inline markings if CSS isn't loaded, but class is better
                    .attributes(&gtk::pango::AttrList::new()) // placeholder, maybe set color via markup if class fails?
                    .build();
                // Manual orange color using attributes or markup since we might not have CSS class 'orange-label' defined yet
                // But user asked for orange.
                icon_label.set_markup(&format!("<span color='orange'>{}</span>", icon));
                header_box.append(&icon_label);
            }

            container.append(&header_box);

            let name = gtk::Label::builder()
                .label(&sort_name)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .max_width_chars(10) // Prevent overflow
                .css_classes(["body", "strong"])
                .halign(gtk::Align::Center)
                .build();
            container.append(&name);
            container.add_css_class("slot-filled");

            // Tooltip
            let rating = result
                .player_ratings
                .get(&id)
                .map(|r| r.rating)
                .unwrap_or(0.0);
            let behaviour_name = format!("{:?}", pos.behaviour);
            let tooltip = format!(
                "{} {}\n{}\nRating: {:.1}",
                player.FirstName, player.LastName, behaviour_name, rating
            );
            container.set_tooltip_text(Some(&tooltip));
        } else {
            container.append(&header_box); // Show label even if empty
            let empty = gtk::Label::builder()
                .label("-")
                .css_classes(["dim-label"])
                .halign(gtk::Align::Center)
                .build();
            container.append(&empty);
            container.add_css_class("slot-empty");
        }
        container.upcast()
    }
}
