use crate::chpp::model::Player;
use crate::rating::controller::RatingController;
use crate::rating::model::Lineup;
use crate::rating::optimiser::{Formation, OptimisedLineup};
use crate::rating::types::{
    Attitude, Behaviour, Location, PositionId, RatingSector, TacticType, Weather,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use log::{debug, error, info};
use num_format::{Buffer, SystemLocale};
use std::cell::RefCell;
use std::collections::HashMap;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
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
        pub card_size_group: gtk::SizeGroup,
    }

    impl Default for FormationOptimiserWidget {
        fn default() -> Self {
            Self {
                calculate_button: Default::default(),
                formations_flowbox: Default::default(),
                players: RefCell::new(Vec::new()),
                card_size_group: gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
            }
        }
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

impl Default for FormationOptimiserWidget {
    fn default() -> Self {
        Self::new()
    }
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
        let imp = self.imp();
        let flowbox = imp.formations_flowbox.get();
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
            imp.card_size_group.add_widget(&card);
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
        }

        imp.calculate_button.set_sensitive(false);

        let weak_self = self.downgrade();

        glib::MainContext::default().spawn_local(async move {
            let result = tokio::task::spawn_blocking(move || {
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
        let imp = self.imp();
        let flowbox = imp.formations_flowbox.get();
        flowbox.remove_all();

        for result in results {
            let card = self.create_formation_card(&result);
            imp.card_size_group.add_widget(&card);
            flowbox.append(&card);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Formatting helpers
    // ─────────────────────────────────────────────────────────────────────

    /// Returns the system locale for number formatting, falling back to "C" locale.
    fn get_locale() -> SystemLocale {
        SystemLocale::default().unwrap_or_else(|_| {
            SystemLocale::from_name("C").expect("The \"C\" locale should always be available")
        })
    }

    /// Formats a HatStats value with locale-aware thousand separators.
    fn format_hatstats(value: f64) -> String {
        let locale = Self::get_locale();
        let whole = value as i64;
        let mut buf = Buffer::default();
        buf.write_formatted(&whole, &locale);

        let decimal_part = value - whole as f64;
        if decimal_part.abs() > 0.001 {
            format!(
                "{}.{}",
                buf.as_str(),
                format!("{:.1}", decimal_part).trim_start_matches("0.")
            )
        } else {
            buf.as_str().to_string()
        }
    }

    /// Creates a dotted leader that stretches to fill all available space
    /// between a label and its value, clipping any excess dots.
    /// e.g.  "Midfield . . . . . . . . . . . . 4.52"
    fn create_dotted_leader() -> gtk::Widget {
        let spacer = gtk::Box::builder().hexpand(true).build();

        let dots = gtk::Label::builder()
            .label(" . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . ")
            .hexpand(true)
            .halign(gtk::Align::Fill)
            .xalign(0.0)
            .margin_start(4)
            .margin_end(4)
            .css_classes(["dotted-leader"])
            .build();
        dots.set_overflow(gtk::Overflow::Hidden);

        let overlay = gtk::Overlay::builder().child(&spacer).hexpand(true).build();

        overlay.add_overlay(&dots);
        overlay.set_clip_overlay(&dots, true);

        overlay.upcast()
    }

    /// Creates a single rating row: "Label . . . . . Value"
    fn create_rating_row(label: &str, value_text: &str) -> gtk::Grid {
        let grid = gtk::Grid::builder().column_spacing(0).build();

        let lbl = gtk::Label::builder()
            .label(label)
            .halign(gtk::Align::Start)
            .build();

        let leader = Self::create_dotted_leader();

        let val = gtk::Label::builder()
            .label(value_text)
            .halign(gtk::Align::End)
            .build();

        grid.attach(&lbl, 0, 0, 1, 1);
        grid.attach(&leader, 1, 0, 1, 1);
        grid.attach(&val, 2, 0, 1, 1);
        grid
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pitch visualisation (shared between empty and populated cards)
    // ─────────────────────────────────────────────────────────────────────

    /// Creates the pitch visualisation with all position slots.
    /// If `player_map` is `None`, all slots are shown as empty.
    fn create_pitch_visualisation(
        player_map: Option<&HashMap<PositionId, (&Player, Behaviour)>>,
        rating_map: Option<&HashMap<u32, f64>>,
    ) -> gtk::Grid {
        // Use a 5-column grid so all slots share exactly the same
        // column width, regardless of how many slots each row has.
        let grid = gtk::Grid::builder()
            .row_spacing(4)
            .column_spacing(2)
            .column_homogeneous(true)
            .row_homogeneous(true)
            .vexpand(true)
            .build();
        grid.add_css_class("pitch-view");

        // SizeGroup for uniform slot dimensions
        let slot_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

        // Helper to create a single slot
        let create_slot =
            |id: PositionId, label_text: &str, size_group: &gtk::SizeGroup| -> gtk::Widget {
                let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
                container.set_halign(gtk::Align::Fill);
                container.set_valign(gtk::Align::Center);

                // Position label row (with behaviour triangle if populated)
                let pos_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
                pos_row.set_halign(gtk::Align::Center);

                let slot_label = gtk::Label::builder()
                    .label(label_text)
                    .css_classes(["caption", "dim-label"])
                    .halign(gtk::Align::Center)
                    .build();
                pos_row.append(&slot_label);

                if let Some(pmap) = player_map {
                    if let Some((_player, behaviour)) = pmap.get(&id) {
                        let symbol = behaviour.symbol(&id);
                        if !symbol.is_empty() {
                            let triangle = gtk::Label::builder()
                                .label(symbol)
                                .css_classes(["caption", "behaviour-indicator"])
                                .halign(gtk::Align::Center)
                                .build();
                            pos_row.append(&triangle);
                        }
                    }
                }

                container.append(&pos_row);

                // Player name or empty dash
                if let Some(pmap) = player_map {
                    if let Some((player, behaviour)) = pmap.get(&id) {
                        let short_name = format!(
                            "{}. {}",
                            player.FirstName.chars().next().unwrap_or('?'),
                            player.LastName
                        );

                        let player_id = player.PlayerID;
                        let rating = rating_map
                            .and_then(|rm| rm.get(&player_id).copied())
                            .unwrap_or(0.0);
                        let tooltip = format!(
                            "{} {} ({} {}) - Rating: {:.1}",
                            player.FirstName,
                            player.LastName,
                            label_text,
                            behaviour.name(),
                            rating
                        );

                        let name = gtk::Label::builder()
                            .label(&short_name)
                            .ellipsize(gtk::pango::EllipsizeMode::End)
                            .max_width_chars(10)
                            .css_classes(["body", "strong"])
                            .halign(gtk::Align::Center)
                            .build();
                        container.append(&name);
                        container.add_css_class("slot-filled");
                        container.set_tooltip_text(Some(&tooltip));
                    } else {
                        Self::add_empty_slot_label(&container);
                    }
                } else {
                    Self::add_empty_slot_label(&container);
                }

                size_group.add_widget(&container);
                container.upcast()
            };

        // Row 0: Attack — columns 1, 2, 3 (centred in the 5-column grid)
        grid.attach(
            &create_slot(PositionId::LeftForward, "LF", &slot_size_group),
            1,
            0,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::CentralForward, "CF", &slot_size_group),
            2,
            0,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightForward, "RF", &slot_size_group),
            3,
            0,
            1,
            1,
        );

        // Row 1: Midfield — all 5 columns
        grid.attach(
            &create_slot(PositionId::LeftWinger, "LW", &slot_size_group),
            0,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::LeftInnerMidfield, "LIM", &slot_size_group),
            1,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::CentralInnerMidfield, "MIM", &slot_size_group),
            2,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightInnerMidfield, "RIM", &slot_size_group),
            3,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightWinger, "RW", &slot_size_group),
            4,
            1,
            1,
            1,
        );

        // Row 2: Defence — all 5 columns
        grid.attach(
            &create_slot(PositionId::LeftBack, "LB", &slot_size_group),
            0,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::LeftCentralDefender, "LCD", &slot_size_group),
            1,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::MiddleCentralDefender, "MCD", &slot_size_group),
            2,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightCentralDefender, "RCD", &slot_size_group),
            3,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightBack, "RB", &slot_size_group),
            4,
            2,
            1,
            1,
        );

        // Row 3: Keeper — column 2 (centred)
        grid.attach(
            &create_slot(PositionId::Keeper, "GK", &slot_size_group),
            2,
            3,
            1,
            1,
        );

        grid
    }

    fn add_empty_slot_label(container: &gtk::Box) {
        let empty = gtk::Label::builder()
            .label("-")
            .css_classes(["dim-label"])
            .halign(gtk::Align::Center)
            .build();
        container.append(&empty);
        container.add_css_class("slot-empty");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Card builders
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a formation card. When `result` has an empty lineup (no positions),
    /// all slots are shown as dashes, producing a placeholder card.
    /// When called with a populated `OptimisedLineup` it shows the full lineup.
    fn create_formation_card(&self, result: &OptimisedLineup) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        card.add_css_class("card");
        card.set_margin_top(6);
        card.set_margin_bottom(6);
        card.set_margin_start(6);
        card.set_margin_end(6);

        // ── Header: Formation Name + HatStats ──
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let title = gtk::Label::builder()
            .label(result.formation.name())
            .css_classes(["heading"])
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();

        let hatstats_text = Self::format_hatstats(result.hatstats);
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

        // ── Ratings with dotted leaders ──
        let ratings_box = gtk::Box::new(gtk::Orientation::Vertical, 2);

        let sectors = [
            ("Midfield", RatingSector::Midfield),
            ("Def R", RatingSector::DefenceRight),
            ("Def C", RatingSector::DefenceCentral),
            ("Def L", RatingSector::DefenceLeft),
            ("Att R", RatingSector::AttackRight),
            ("Att C", RatingSector::AttackCentral),
            ("Att L", RatingSector::AttackLeft),
        ];

        for (label, sector) in &sectors {
            let value = *result.sector_ratings.get(sector).unwrap_or(&0.0);
            ratings_box.append(&Self::create_rating_row(label, &format!("{:.2}", value)));
        }

        card.append(&ratings_box);

        // ── Roles Section: Captain & Set Pieces ──
        let roles_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        roles_box.set_margin_top(8);
        roles_box.set_margin_bottom(8);

        let add_role = |label: &str, player: Option<&Player>| -> gtk::Box {
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
            row
        };

        roles_box.append(&add_role("Captain", result.captain.as_ref()));
        roles_box.append(&add_role("Set Pieces", result.set_pieces_taker.as_ref()));

        card.append(&roles_box);

        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // ── Visual Lineup on Pitch ──
        let mut player_map: HashMap<PositionId, (&Player, Behaviour)> = HashMap::new();
        for pos in &result.lineup.positions {
            player_map.insert(pos.role_id, (&pos.player, pos.behaviour));
        }

        // Pre-calculate ratings for tooltip display
        let mut rating_map: HashMap<u32, f64> = HashMap::new();
        for pos in &result.lineup.positions {
            let player_id = pos.player.PlayerID;
            rating_map
                .entry(player_id)
                .or_insert_with(|| match pos.role_id.sector() {
                    crate::rating::types::Sector::Goal
                    | crate::rating::types::Sector::InnerMidfield
                    | crate::rating::types::Sector::Wing => result
                        .sector_ratings
                        .get(&RatingSector::Midfield)
                        .copied()
                        .unwrap_or(0.0),
                    crate::rating::types::Sector::CentralDefence
                    | crate::rating::types::Sector::Back => result
                        .sector_ratings
                        .get(&RatingSector::DefenceCentral)
                        .copied()
                        .unwrap_or(0.0),
                    crate::rating::types::Sector::Forward => result
                        .sector_ratings
                        .get(&RatingSector::AttackCentral)
                        .copied()
                        .unwrap_or(0.0),
                });
        }

        let visual_box = Self::create_pitch_visualisation(Some(&player_map), Some(&rating_map));
        card.append(&visual_box);

        card.upcast()
    }
}
