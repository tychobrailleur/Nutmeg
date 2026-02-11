use crate::chpp::model::Player;
use crate::rating::model::Team;
use crate::rating::optimizer::{Formation, LineupOptimizer};
use crate::rating::{RatingPredictionModel, RatingSector};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use log::{debug, error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::rating::types::PositionId;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="FormationOptimizerWidget" parent="GtkBox">
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
                <property name="label" translatable="yes">Formation Optimizer</property>
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
    pub struct FormationOptimizerWidget {
        #[template_child]
        pub calculate_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub formations_flowbox: TemplateChild<gtk::FlowBox>,
        
        pub players: RefCell<Vec<Player>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FormationOptimizerWidget {
        const NAME: &'static str = "FormationOptimizerWidget";
        type Type = super::FormationOptimizerWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FormationOptimizerWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_callbacks();
        }
    }
    
    impl WidgetImpl for FormationOptimizerWidget {}
    impl BoxImpl for FormationOptimizerWidget {}
}

glib::wrapper! {
    pub struct FormationOptimizerWidget(ObjectSubclass<imp::FormationOptimizerWidget>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl FormationOptimizerWidget {
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

    fn on_calculate_clicked(&self, _button: &gtk::Button) {
        let imp = self.imp();
        let players = imp.players.borrow().clone();
        
        info!("Optimizer started with {} players", players.len());
        if let Some(p) = players.first() {
            debug!("Sample player: {} {} (Form: {})", p.FirstName, p.LastName, p.PlayerForm);
            if let Some(s) = &p.PlayerSkills {
                debug!("  Skills: Keeper={}, Def={}, PM={}, Winger={}, Passing={}, Scoring={}, SP={}", 
                    s.KeeperSkill, s.DefenderSkill, s.PlaymakerSkill, s.WingerSkill, s.PassingSkill, s.ScorerSkill, s.SetPiecesSkill);
            } else {
                error!("  Sample player has NO SKILLS data!");
            }
        }

        imp.calculate_button.set_sensitive(false);
        imp.formations_flowbox.remove_all();

        let weak_self = self.downgrade();

        glib::MainContext::default().spawn_local(async move {
            let result = tokio::task::spawn_blocking(move || {
                let team = Team::default(); 
                let model = RatingPredictionModel::new(team);
                let optimizer = LineupOptimizer::new(&model, &players);

                let mut results = Vec::new();
                for formation in Formation::all() {
                    let opt_lineup = optimizer.optimize(formation);
                    info!("Formation {:?} -> HatStats {:.1}", formation, opt_lineup.hatstats);
                    results.push(opt_lineup);
                }
                results
            }).await;

            if let Some(obj) = weak_self.upgrade() {
                obj.imp().calculate_button.set_sensitive(true);
                match result {
                    Ok(results) => obj.display_results(results),
                    Err(e) => eprintln!("Optimization task failed: {}", e),
                }
            }
        });
    }

    fn display_results(&self, results: Vec<crate::rating::optimizer::OptimizedLineup>) {
        let flowbox = self.imp().formations_flowbox.get();
        
        for result in results {
            let card = self.create_formation_card(&result);
            flowbox.append(&card);
        }
    }

    fn create_formation_card(&self, result: &crate::rating::optimizer::OptimizedLineup) -> gtk::Widget {
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
            
        let hatstats = gtk::Label::builder()
            .label(&format!("{:.1}", result.hatstats))
            .css_classes(["accent"])
            .halign(gtk::Align::End)
            .tooltip_text("HatStats")
            .build();
            
        header.append(&title);
        header.append(&hatstats);
        card.append(&header);
        
        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Ratings Grid
        let grid = gtk::Grid::builder()
            .row_spacing(4)
            .column_spacing(8)
            .build();

        // Helper to add row
        let add_row = |label: &str, value: f64, row: i32| {
            let lbl = gtk::Label::builder().label(label).halign(gtk::Align::Start).build();
            let val = gtk::Label::builder().label(&format!("{:.2}", value)).halign(gtk::Align::End).hexpand(true).build();
            grid.attach(&lbl, 0, row, 1, 1);
            grid.attach(&val, 1, row, 1, 1);
        };
        
        add_row("Midfield", *result.sector_ratings.get(&RatingSector::Midfield).unwrap_or(&0.0), 0);
        add_row("Def R", *result.sector_ratings.get(&RatingSector::DefenceRight).unwrap_or(&0.0), 1);
        add_row("Def C", *result.sector_ratings.get(&RatingSector::DefenceCentral).unwrap_or(&0.0), 2);
        add_row("Def L", *result.sector_ratings.get(&RatingSector::DefenceLeft).unwrap_or(&0.0), 3);
        add_row("Att R", *result.sector_ratings.get(&RatingSector::AttackRight).unwrap_or(&0.0), 4);
        add_row("Att C", *result.sector_ratings.get(&RatingSector::AttackCentral).unwrap_or(&0.0), 5);
        add_row("Att L", *result.sector_ratings.get(&RatingSector::AttackLeft).unwrap_or(&0.0), 6);

        card.append(&grid);

        card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Visual Lineup
        let visual_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        visual_box.set_vexpand(true);
        visual_box.add_css_class("pitch-view");

        let mut player_map = HashMap::new();
        for pos in &result.lineup.positions {
            player_map.insert(pos.role_id, &pos.player);
        }

        // Helper to create a slot widget
        let create_slot = |id: PositionId, label_text: &str| -> gtk::Widget {
            let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
            container.set_hexpand(true);
            container.set_valign(gtk::Align::Center);
            // container.set_halign(gtk::Align::Fill); // Default is Fill, allowing it to stretch
            
            // Slot Label (e.g. LF)
            let slot_label = gtk::Label::builder()
                .label(label_text)
                .css_classes(["caption", "dim-label"])
                .halign(gtk::Align::Center)
                .build();
            
            container.append(&slot_label);

            if let Some(player) = player_map.get(&id) {
                // Player Name (Last Name / Initial)
                let sort_name = format!("{}. {}", 
                    player.FirstName.chars().next().unwrap_or('?'), 
                    player.LastName);
                
                let name = gtk::Label::builder()
                    .label(&sort_name)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .max_width_chars(10) // Prevent overflow
                    .tooltip_text(&format!("{} {}", player.FirstName, player.LastName))
                    .css_classes(["body", "strong"])
                    .halign(gtk::Align::Center)
                    .build();
                container.append(&name);
                container.add_css_class("slot-filled");
            } else {
                 let empty = gtk::Label::builder()
                    .label("-")
                    .css_classes(["dim-label"])
                    .halign(gtk::Align::Center)
                    .build();
                 container.append(&empty);
                 container.add_css_class("slot-empty");
            }
            container.upcast()
        };

        // Row 1: Attack (3 slots)
        let row_att = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_att.set_homogeneous(true);
        row_att.append(&create_slot(PositionId::LeftForward, "LF"));
        row_att.append(&create_slot(PositionId::CentralForward, "CF"));
        row_att.append(&create_slot(PositionId::RightForward, "RF"));
        visual_box.append(&row_att);

        // Row 2: Midfield (5 slots)
        let row_mid = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_mid.set_homogeneous(true);
        row_mid.append(&create_slot(PositionId::LeftWinger, "LW"));
        row_mid.append(&create_slot(PositionId::LeftInnerMidfield, "LIM"));
        row_mid.append(&create_slot(PositionId::CentralInnerMidfield, "MIM"));
        row_mid.append(&create_slot(PositionId::RightInnerMidfield, "RIM"));
        row_mid.append(&create_slot(PositionId::RightWinger, "RW"));
        visual_box.append(&row_mid);

        // Row 3: Defense (5 slots)
        let row_def = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row_def.set_homogeneous(true);
        row_def.append(&create_slot(PositionId::LeftBack, "LB"));
        row_def.append(&create_slot(PositionId::LeftCentralDefender, "LCD"));
        row_def.append(&create_slot(PositionId::MiddleCentralDefender, "MCD"));
        row_def.append(&create_slot(PositionId::RightCentralDefender, "RCD"));
        row_def.append(&create_slot(PositionId::RightBack, "RB"));
        visual_box.append(&row_def);

        // Row 4: Keeper (1 slot)
        let row_gk = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        // Center the GK by using a 3-column layout principle or just spacers
        let spacer_l = gtk::Box::new(gtk::Orientation::Horizontal, 0); spacer_l.set_hexpand(true);
        let spacer_r = gtk::Box::new(gtk::Orientation::Horizontal, 0); spacer_r.set_hexpand(true);
        
        row_gk.append(&spacer_l);
        let gk_slot = create_slot(PositionId::Keeper, "GK");
        gk_slot.set_hexpand(false);
        gk_slot.set_width_request(80);
        row_gk.append(&gk_slot);
        row_gk.append(&spacer_r);
        visual_box.append(&row_gk);

        card.append(&visual_box);

        // Lineup Details
        let players_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        
        // Sort positions for display consistency: Keeper, Def, Mid, Att
        let mut positions = result.lineup.positions.clone();
        positions.sort_by_key(|p| p.role_id as u8);

        for pos in positions {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            
            // Position Label (e.g. "GK", "IM")
            let pos_label = gtk::Label::builder()
                .label(&format!("{:?}", pos.role_id)) // TODO: Friendly names
                .halign(gtk::Align::Start)
                .width_chars(8)
                .build();
                
            // Player Name
            let name_label = gtk::Label::builder()
                .label(&format!("{} {}", pos.player.FirstName, pos.player.LastName))
                .halign(gtk::Align::Start)
                .hexpand(true)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            
            row.append(&pos_label);
            row.append(&name_label);
            players_box.append(&row);
        }
        card.append(&players_box);
        
        card.upcast()
    }
}
