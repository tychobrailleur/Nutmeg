#![allow(deprecated)]
/* window.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::{Player, Team};
use crate::db::manager::DbManager;
use crate::db::teams::{get_players_for_team, get_teams_summary};
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate, TemplateChild};
use log::{debug, error, info};
use num_format::{Buffer, SystemLocale};
use std::cell::RefCell;

use crate::service::context::{AppContext, ContextService};
use std::sync::Arc;

// TODO see if the template cannot be dfined as a .ui file
mod team_object {
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Clone, Debug)]
    pub struct TeamData {
        pub id: u32,
        pub name: String,
        pub logo_url: Option<String>,
    }

    mod imp {
        use super::*;

        #[derive(Default)]
        pub struct TeamObject {
            pub data: RefCell<Option<TeamData>>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for TeamObject {
            const NAME: &'static str = "TeamObject";
            type Type = super::TeamObject;
        }

        impl ObjectImpl for TeamObject {}
    }

    glib::wrapper! {
        pub struct TeamObject(ObjectSubclass<imp::TeamObject>);
    }

    impl TeamObject {
        pub fn new(id: u32, name: String, logo_url: Option<String>) -> Self {
            let obj: Self = glib::Object::new();
            obj.imp()
                .data
                .replace(Some(TeamData { id, name, logo_url }));
            obj
        }

        pub fn team_data(&self) -> TeamData {
            self.imp().data.borrow().as_ref().unwrap().clone()
        }
    }
}

mod player_object {
    use super::*;
    use gtk::glib;

    mod imp {
        use super::*;

        #[derive(Default)]
        pub struct PlayerObject {
            pub data: RefCell<Option<Player>>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for PlayerObject {
            const NAME: &'static str = "PlayerObject";
            type Type = super::PlayerObject;
        }

        impl ObjectImpl for PlayerObject {}
    }

    glib::wrapper! {
        pub struct PlayerObject(ObjectSubclass<imp::PlayerObject>);
    }

    impl PlayerObject {
        pub fn new(player: Player) -> Self {
            let obj: Self = glib::Object::new();
            obj.imp().data.replace(Some(player));
            obj
        }

        pub fn player(&self) -> Player {
            self.imp().data.borrow().as_ref().unwrap().clone()
        }
    }
}

use player_object::PlayerObject;
use team_object::TeamObject;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/window.ui")]
    pub struct NutmegWindow {
        #[template_child]
        pub combo_teams: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub view_players: TemplateChild<gtk::TreeView>,

        pub context: RefCell<AppContext>,

        #[template_child]
        pub details_panel: TemplateChild<gtk::Box>,
        #[template_child]
        pub details_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_id: TemplateChild<gtk::Label>,

        // Category
        #[template_child]
        pub details_category: TemplateChild<gtk::Label>,

        // Level
        #[template_child]
        pub details_form: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_stamina: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_tsi: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_injury: TemplateChild<gtk::Label>,

        // Skills
        #[template_child]
        pub details_skill_keeper: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_defender: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_playmaker: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_winger: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_passing: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_scorer: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_set_pieces: TemplateChild<gtk::Label>,

        // Career / Club
        #[template_child]
        pub details_career_goals: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_league_goals: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_loyalty: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_mother_club: TemplateChild<gtk::Label>,

        // Last Match
        #[template_child]
        pub details_last_match_date: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_played_minutes: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_position_code: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_rating: TemplateChild<gtk::Label>,

        pub current_players: RefCell<Option<gtk::ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NutmegWindow {
        const NAME: &'static str = "NutmegWindow";
        type Type = super::NutmegWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NutmegWindow {
        fn constructed(&self) {
            info!("NutmegWindow constructed");
            self.parent_constructed();
            let obj = self.obj();

            // Setup TreeView Columns
            obj.setup_tree_view();

            // Load Teams
            obj.load_teams();

            // Setup Signals
            obj.setup_signals();

            // Load CSS
            let provider = gtk::CssProvider::new();
            provider.load_from_data(include_str!("style.css"));
            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().unwrap(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
    impl WidgetImpl for NutmegWindow {}
    impl WindowImpl for NutmegWindow {}
    impl ApplicationWindowImpl for NutmegWindow {}
}

glib::wrapper! {
    pub struct NutmegWindow(ObjectSubclass<imp::NutmegWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl NutmegWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_tree_view(&self) {
        let imp = self.imp();
        let view = &imp.view_players;

        // Helper to add a text column
        let add_column = |title: &str, col_id: i32| {
            let renderer = gtk::CellRendererText::new();
            let column = gtk::TreeViewColumn::new();
            column.set_title(title);
            column.set_reorderable(true);
            column.set_resizable(true);
            column.pack_start(&renderer, true);
            column.add_attribute(&renderer, "text", col_id);
            column.add_attribute(&renderer, "cell-background", 13); // BG Color is now at index 13
            view.append_column(&column);
        };

        // Columns:
        // 0: Name, 1: Flag, 2: Number, 3: Age, 4: Form, 5: TSI
        // 6: Salary, 7: Specialty, 8: Experience, 9: Leadership, 10: Loyalty
        // 11: Best Pos, 12: Last Pos, 13: BG Color, 14: Stamina, 15: Injured, 16: Cards, 17: Mother Club
        // 18: PlayerObj

        add_column(&gettext("Name"), 0);
        add_column(&gettext("Flag"), 1);
        add_column(&gettext("No."), 2);
        add_column(&gettext("Age"), 3);
        add_column(&gettext("Form"), 4);
        add_column(&gettext("TSI"), 5);
        add_column(&gettext("Salary"), 6);
        add_column(&gettext("Specialty"), 7);
        add_column(&gettext("XP"), 8);
        add_column(&gettext("Lead"), 9);
        add_column(&gettext("Loyalty"), 10);
        add_column(&gettext("Best Pos"), 11);
        add_column(&gettext("Last Pos"), 12);
        // BG Color is 13, not displayed as column
        add_column(&gettext("Stamina"), 14);
        add_column(&gettext("Injured"), 15);
        add_column(&gettext("Cards"), 16);
        add_column(&gettext("Mother Club"), 17);
    }

    fn load_teams(&self) {
        let imp = self.imp();
        let db = DbManager::new();
        if let Ok(mut conn) = db.get_connection() {
            match get_teams_summary(&mut conn) {
                Ok(teams) => {
                    info!("Loaded {} teams", teams.len());

                    // Create list store for teams
                    let model = gio::ListStore::new::<TeamObject>();
                    for (id, name, logo_url) in teams {
                        model.append(&TeamObject::new(id, name, logo_url));
                    }

                    // Create factory for team items
                    let factory = gtk::SignalListItemFactory::new();

                    // Setup: create the widget structure
                    factory.connect_setup(|_, item| {
                        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                        hbox.set_margin_start(4);
                        hbox.set_margin_end(4);
                        hbox.set_margin_top(4);
                        hbox.set_margin_bottom(4);

                        // Logo placeholder (32x32)
                        let logo = gtk::Image::new();
                        logo.set_pixel_size(32);
                        hbox.append(&logo);

                        // Team name + ID label
                        let label = gtk::Label::new(None);
                        label.set_xalign(0.0);
                        hbox.append(&label);

                        item.set_child(Some(&hbox));
                    });

                    // Bind: populate the widgets with data
                    factory.connect_bind(|_, item| {
                        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                        let team_obj = item.item().and_downcast::<TeamObject>().unwrap();
                        let hbox = item.child().and_downcast::<gtk::Box>().unwrap();

                        let logo = hbox
                            .first_child()
                            .unwrap()
                            .downcast::<gtk::Image>()
                            .unwrap();
                        let label = logo
                            .next_sibling()
                            .unwrap()
                            .downcast::<gtk::Label>()
                            .unwrap();

                        let team_data = team_obj.team_data();

                        // Set label with markup (name + gray ID)
                        let markup = format!(
                            "{} <span foreground='gray'>({})</span>",
                            glib::markup_escape_text(&team_data.name),
                            team_data.id
                        );
                        label.set_markup(&markup);

                        // Load logo if URL is available
                        if let Some(mut url) = team_data.logo_url {
                            // Hattrick URLs are protocol-relative, add https:
                            if url.starts_with("//") {
                                url = format!("https:{}", url);
                            }

                            let logo_clone = logo.clone();
                            glib::MainContext::default().spawn_local(async move {
                                match load_image_from_url(&url).await {
                                    Ok(texture) => {
                                        logo_clone.set_paintable(Some(&texture));
                                    }
                                    Err(e) => {
                                        debug!("Failed to load team logo from {}: {}", url, e);
                                    }
                                }
                            });
                        }
                    });

                    // Set model and factory on dropdown
                    imp.combo_teams.set_model(Some(&model));
                    imp.combo_teams.set_factory(Some(&factory));

                    // Select first team if available and load its players
                    if model.n_items() > 0 {
                        imp.combo_teams.set_selected(0);
                        // Manually load players for first team since signal isn't connected yet
                        if let Some(first_team) = model.item(0) {
                            if let Ok(team_obj) = first_team.downcast::<TeamObject>() {
                                let team_id = team_obj.team_data().id;
                                debug!("Loading players for initial team: {}", team_id);
                                self.load_players(team_id);
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to load teams: {}", e),
            }
        } else {
            error!("Failed to get DB connection");
        }
    }

    fn setup_signals(&self) {
        let imp = self.imp();
        let window = self.clone();

        imp.combo_teams.connect_selected_notify(move |dropdown| {
            if let Some(selected) = dropdown.selected_item() {
                if let Ok(team_obj) = selected.downcast::<TeamObject>() {
                    let team_id = team_obj.team_data().id;
                    debug!("Team selection changed to {}", team_id);
                    window.load_players(team_id);
                }
            }
        });

        // Player selection
        let view = &imp.view_players;
        let selection = view.selection();
        let window = self.clone();

        selection.connect_changed(move |selection| {
            #[allow(deprecated)]
            if let Some((model, iter)) = selection.selected() {
                #[allow(deprecated)]
                let obj_val = model.get_value(&iter, 18);
                if let Ok(player_obj) = obj_val.get::<PlayerObject>() {
                    let p = player_obj.player();
                    let imp = window.imp();

                    // Update context
                    {
                        let mut ctx = imp.context.borrow_mut();
                        ctx.player = Some(p.clone());
                        info!("Context updated: Player={}", p.LastName);
                    }

                    imp.details_panel.set_visible(true);
                    imp.details_name
                        .set_label(&format!("{} {}", p.FirstName, p.LastName));
                    imp.details_id.set_label(&p.PlayerID.to_string());

                    // Category
                    let cat_str = match p.PlayerCategoryId {
                        Some(1) => gettext("Keeper"),
                        Some(2) => gettext("Right Back"),
                        Some(3) => gettext("Central Defender"),
                        Some(4) => gettext("Winger"),
                        Some(5) => gettext("Inner Midfielder"),
                        Some(6) => gettext("Forward"),
                        _ => gettext("Unknown/Unset"),
                    };
                    imp.details_category.set_label(&cat_str);

                    // Level
                    imp.details_form.set_label(&p.PlayerForm.to_string());

                    let stamina = p
                        .PlayerSkills
                        .as_ref()
                        .map(|s| s.StaminaSkill.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    imp.details_stamina.set_label(&stamina);

                    imp.details_tsi.set_label(&p.TSI.to_string());
                    imp.details_injury.set_label(
                        &p.InjuryLevel
                            .map(|v| v.to_string())
                            .unwrap_or("-".to_string()),
                    );

                    // Skills
                    let skills = p.PlayerSkills.as_ref();
                    imp.details_skill_keeper.set_label(
                        &skills
                            .map(|s| s.KeeperSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_skill_defender.set_label(
                        &skills
                            .map(|s| s.DefenderSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_skill_playmaker.set_label(
                        &skills
                            .map(|s| s.PlaymakerSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_skill_winger.set_label(
                        &skills
                            .map(|s| s.WingerSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_skill_passing.set_label(
                        &skills
                            .map(|s| s.PassingSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_skill_scorer.set_label(
                        &skills
                            .map(|s| s.ScorerSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_skill_set_pieces.set_label(
                        &skills
                            .map(|s| s.SetPiecesSkill.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );

                    // Career / Club
                    imp.details_career_goals.set_label(
                        &p.CareerGoals
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_league_goals.set_label(
                        &p.LeagueGoals
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_loyalty.set_label(&p.Loyalty.to_string());

                    let mother_club_text = if p.MotherClubBonus {
                        gettext("Yes")
                    } else {
                        gettext("No")
                    };
                    imp.details_mother_club.set_label(&mother_club_text);

                    // Last Match
                    imp.details_last_match_date
                        .set_label(p.LastMatch.as_ref().map(|m| m.Date.as_str()).unwrap_or("-"));
                    imp.details_played_minutes.set_label(
                        &p.LastMatch
                            .as_ref()
                            .map(|m| m.PlayedMinutes.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    imp.details_position_code.set_label(
                        &p.LastMatch
                            .as_ref()
                            .map(|m| m.PositionCode.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );

                    let rating_str = p
                        .LastMatch
                        .as_ref()
                        .and_then(|m| m.Rating)
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    imp.details_rating.set_label(&rating_str);
                }
            } else {
                let imp = window.imp();
                imp.details_panel.set_visible(false);
                let mut ctx = imp.context.borrow_mut();
                ctx.player = None;
            }
        });
    }

    fn load_players(&self, team_id: u32) {
        let imp = self.imp();
        let db_manager = Arc::new(DbManager::new());
        let context_service = ContextService::new(db_manager.clone());
        let new_ctx = context_service.load_team_context(team_id);

        {
            let mut ctx = self.imp().context.borrow_mut();
            *ctx = new_ctx;
        }

        if let Ok(mut conn) = db_manager.get_connection() {
            match get_players_for_team(&mut conn, team_id) {
                Ok(players) => {
                    info!("Loaded {} players for team {}", players.len(), team_id);

                    // Create ListStore
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
                        glib::Type::STRING, // 13 Background Color
                        glib::Type::STRING, // 14 Stamina
                        glib::Type::STRING, // 15 Injured
                        glib::Type::STRING, // 16 Cards
                        glib::Type::STRING, // 17 Mother Club
                        glib::Type::OBJECT, // 18 PlayerObject
                    ]);

                    // Get color from CSS
                    #[allow(deprecated)]
                    let context = self.imp().view_players.style_context();
                    #[allow(deprecated)]
                    let mother_club_bg_str = context
                        .lookup_color("mother_club_bg")
                        .map(|c| c.to_string())
                        .or_else(|| Some("rgba(64, 224, 208, 0.3)".to_string())); // Fallback

                    // Get locale for formatting
                    let locale = SystemLocale::default()
                        .unwrap_or_else(|_| SystemLocale::from_name("C").unwrap());

                    for p in players {
                        let obj = PlayerObject::new(p.clone());
                        let display = crate::player_display::PlayerDisplay::new(&p, &locale);

                        let bg = if p.MotherClubBonus {
                            mother_club_bg_str.as_deref()
                        } else {
                            None
                        };

                        #[allow(deprecated)]
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

                    let imp = self.imp();
                    #[allow(deprecated)]
                    imp.view_players.set_model(Some(&store));
                    imp.current_players.replace(Some(store));
                }
                Err(e) => error!("Failed to load players: {}", e),
            }
        }
    }
}

// Helper function to load images from URLs
async fn load_image_from_url(url: &str) -> Result<gdk::Texture, Box<dyn std::error::Error>> {
    use gdk_pixbuf::Pixbuf;
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let gbytes = glib::Bytes::from(&bytes[..]);
    let stream = gio::MemoryInputStream::from_bytes(&gbytes);
    let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE)?;
    Ok(gdk::Texture::for_pixbuf(&pixbuf))
}
