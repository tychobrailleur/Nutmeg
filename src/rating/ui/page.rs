use crate::chpp::model::Player;
use crate::rating::controller::RatingController;
use crate::rating::optimiser::{Formation, OptimisedLineup};
use crate::rating::RatingSector;
use crate::ui::components::pitch_view::{PitchPlayer, PitchView};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use log::{error, info};
use num_format::{Buffer, SystemLocale};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/rating/ui/page.ui")]
    pub struct FormationOptimiserWidget {
        #[template_child]
        pub calculate_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub formations_flowbox: TemplateChild<gtk::FlowBox>,

        pub players: RefCell<Vec<Player>>,
        pub card_size_group: gtk::SizeGroup,
        pub context: RefCell<Option<crate::ui::context_object::ContextObject>>,
    }

    impl Default for FormationOptimiserWidget {
        fn default() -> Self {
            Self {
                calculate_button: Default::default(),
                formations_flowbox: Default::default(),
                players: RefCell::new(Vec::new()),
                card_size_group: gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
                context: RefCell::new(None),
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
            self.obj().populate_empty_cards();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<crate::ui::context_object::ContextObject>(
                        "context",
                    )
                    .explicit_notify()
                    .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "context" => self.context.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "context" => {
                    let ctx = value
                        .get::<Option<crate::ui::context_object::ContextObject>>()
                        .expect("Value must be ContextObject");
                    self.context.replace(ctx);
                    self.obj().notify("context");
                }
                _ => unimplemented!(),
            }
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

    /// Populates the FlowBox with empty placeholder cards for each formation.
    fn populate_empty_cards(&self) {
        let imp = self.imp();
        let flowbox = imp.formations_flowbox.get();
        flowbox.remove_all();

        for formation in Formation::all() {
            let card = self.create_empty_card(&formation);
            imp.card_size_group.add_widget(&card);
            flowbox.append(&card);
        }
    }

    fn on_calculate_clicked(&self, _button: &gtk::Button) {
        let imp = self.imp();
        let players = imp.players.borrow().clone();

        info!("Optimiser started with {} players", players.len());

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
        imp.formations_flowbox.remove_all();

        // Calculate best formation score if opponent ratings are available
        let mut best_index = None;
        if let Some(ctx) = imp.context.borrow().as_ref() {
            if let Some(opp_ratings) = ctx.get_opponent_avg_ratings() {
                let opp_att_avg = (opp_ratings[0] + opp_ratings[1] + opp_ratings[2]) / 3.0;
                let opp_mid = opp_ratings[3];
                let opp_def_avg = (opp_ratings[4] + opp_ratings[5] + opp_ratings[6]) / 3.0;

                let mut max_score = -1.0;

                for (idx, result) in results.iter().enumerate() {
                    let our_mid = *result
                        .sector_ratings
                        .get(&RatingSector::Midfield)
                        .unwrap_or(&0.0);
                    let our_def_avg = (*result
                        .sector_ratings
                        .get(&RatingSector::DefenceLeft)
                        .unwrap_or(&0.0)
                        + *result
                            .sector_ratings
                            .get(&RatingSector::DefenceCentral)
                            .unwrap_or(&0.0)
                        + *result
                            .sector_ratings
                            .get(&RatingSector::DefenceRight)
                            .unwrap_or(&0.0))
                        / 3.0;
                    let our_att_avg = (*result
                        .sector_ratings
                        .get(&RatingSector::AttackLeft)
                        .unwrap_or(&0.0)
                        + *result
                            .sector_ratings
                            .get(&RatingSector::AttackCentral)
                            .unwrap_or(&0.0)
                        + *result
                            .sector_ratings
                            .get(&RatingSector::AttackRight)
                            .unwrap_or(&0.0))
                        / 3.0;

                    // Score calculation: Midfield control + defense/attack ratios
                    let score = (our_mid as f32 / opp_mid.max(0.1))
                        + (our_def_avg as f32 / opp_att_avg.max(0.1))
                        + (our_att_avg as f32 / opp_def_avg.max(0.1));

                    if score > max_score {
                        max_score = score;
                        best_index = Some(idx);
                    }
                }
            }
        }

        for (idx, result) in results.into_iter().enumerate() {
            let card = self.create_formation_card(&result, best_index == Some(idx));
            imp.card_size_group.add_widget(&card);
            imp.formations_flowbox.append(&card);
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
    fn create_pitch_visualisation(players: Option<&Vec<PitchPlayer>>) -> gtk::Grid {
        if let Some(list) = players {
            PitchView::create(list)
        } else {
            PitchView::create(&[])
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Card builders
    // ─────────────────────────────────────────────────────────────────────

    /// Creates an empty placeholder card showing formation name, empty ratings,
    /// empty roles, and the full pitch with empty slots.
    fn create_empty_card(&self, formation: &Formation) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        card.add_css_class("card");
        card.set_margin_top(6);
        card.set_margin_bottom(6);
        card.set_margin_start(6);
        card.set_margin_end(6);

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let title = gtk::Label::builder()
            .label(formation.name())
            .css_classes(["heading"])
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        let hatstats = gtk::Label::builder()
            .label("—")
            .css_classes(["accent"])
            .halign(gtk::Align::End)
            .build();
        header.append(&title);
        header.append(&hatstats);
        card.append(&header);

        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Empty ratings with dotted leaders
        let ratings_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        for label in [
            "Midfield", "Def R", "Def C", "Def L", "Att R", "Att C", "Att L",
        ] {
            ratings_box.append(&Self::create_rating_row(label, "—"));
        }
        card.append(&ratings_box);

        // Empty roles
        let roles_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        roles_box.set_margin_top(8);
        roles_box.set_margin_bottom(8);
        for role_label in ["Captain", "Set Pieces"] {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let lbl = gtk::Label::builder()
                .label(role_label)
                .halign(gtk::Align::Start)
                .css_classes(["dim-label"])
                .build();
            let val = gtk::Label::builder()
                .label("—")
                .halign(gtk::Align::End)
                .hexpand(true)
                .build();
            row.append(&lbl);
            row.append(&val);
            roles_box.append(&row);
        }
        card.append(&roles_box);

        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Full pitch with empty slots (no player data)
        let visual_box = Self::create_pitch_visualisation(None);
        card.append(&visual_box);

        card.upcast()
    }

    /// Creates a populated formation card with all data filled in.
    fn create_formation_card(&self, result: &OptimisedLineup, highlight: bool) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        card.add_css_class("card");
        if highlight {
            card.add_css_class("highlighted-formation");
        }
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
        let mut pitch_players = Vec::new();

        for pos in &result.lineup.positions {
            let sector_rating = match pos.role_id.sector() {
                crate::rating::types::Sector::Goal => result
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
                crate::rating::types::Sector::InnerMidfield => result
                    .sector_ratings
                    .get(&RatingSector::Midfield)
                    .copied()
                    .unwrap_or(0.0),
                crate::rating::types::Sector::Wing => result
                    .sector_ratings
                    .get(&RatingSector::Midfield)
                    .copied()
                    .unwrap_or(0.0),
                crate::rating::types::Sector::Forward => result
                    .sector_ratings
                    .get(&RatingSector::AttackCentral)
                    .copied()
                    .unwrap_or(0.0),
            };

            pitch_players.push(PitchPlayer {
                id: pos.player.PlayerID,
                first_name: pos.player.FirstName.clone(),
                last_name: pos.player.LastName.clone(),
                role: pos.role_id.clone(),
                behaviour: pos.behaviour.clone(),
                rating: Some(sector_rating),
            });
        }

        let visual_box = Self::create_pitch_visualisation(Some(&pitch_players));
        card.append(&visual_box);

        card.upcast()
    }
}
