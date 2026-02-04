/* window.rs
 *
 * Copyright 2026 sebastien
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::Player;
use crate::db::manager::DbManager;
use crate::db::teams::{get_players_for_team, get_teams_summary};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate, TemplateChild};
use log::{debug, error, info};
use std::cell::RefCell;
use std::rc::Rc;

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
    #[template(resource = "/org/gnome/Hoctane/window.ui")]
    pub struct HoctaneWindow {
        #[template_child]
        pub combo_teams: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub view_players: TemplateChild<gtk::ColumnView>,

        #[template_child]
        pub factory_flag: TemplateChild<gtk::SignalListItemFactory>,
        #[template_child]
        pub factory_number: TemplateChild<gtk::SignalListItemFactory>,
        #[template_child]
        pub factory_name: TemplateChild<gtk::SignalListItemFactory>,
        #[template_child]
        pub factory_age: TemplateChild<gtk::SignalListItemFactory>,
        #[template_child]
        pub factory_form: TemplateChild<gtk::SignalListItemFactory>,
        #[template_child]
        pub factory_tsi: TemplateChild<gtk::SignalListItemFactory>,

        pub current_players: RefCell<Option<gtk::SingleSelection>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HoctaneWindow {
        const NAME: &'static str = "HoctaneWindow";
        type Type = super::HoctaneWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HoctaneWindow {
        fn constructed(&self) {
            info!("HoctaneWindow constructed");
            self.parent_constructed();
            let obj = self.obj();

            // Setup Factories
            obj.setup_factories();

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
    impl WidgetImpl for HoctaneWindow {}
    impl WindowImpl for HoctaneWindow {}
    impl ApplicationWindowImpl for HoctaneWindow {}
}

glib::wrapper! {
    pub struct HoctaneWindow(ObjectSubclass<imp::HoctaneWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl HoctaneWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_factories(&self) {
        let imp = self.imp();

        // Helper to setup a simple label cell
        let setup_label = |factory: &gtk::SignalListItemFactory| {
            factory.connect_setup(move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let label = gtk::Label::new(None);
                label.set_halign(gtk::Align::Fill);
                label.set_hexpand(true);
                label.set_xalign(0.0);
                item.set_child(Some(&label));
            });
        };

        // Flag
        setup_label(&imp.factory_flag);
        imp.factory_flag.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            
            if player_obj.player().MotherClubBonus {
                label.add_css_class("mother-club");
            } else {
                label.remove_css_class("mother-club");
            }
            
            let flag_str = player_obj.player().Flag.unwrap_or_else(|| "🏳️".to_string());
            label.set_label(&flag_str);
        });

        // Number
        setup_label(&imp.factory_number);
        imp.factory_number.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();

            if player_obj.player().MotherClubBonus {
                label.add_css_class("mother-club");
            } else {
                label.remove_css_class("mother-club");
            }

            let num_str = player_obj
                .player()
                .PlayerNumber
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_string());
            label.set_label(&num_str);
        });

        // Name
        setup_label(&imp.factory_name);
        imp.factory_name.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();

            if player_obj.player().MotherClubBonus {
                label.add_css_class("mother-club");
            } else {
                label.remove_css_class("mother-club");
            }
            let p = player_obj.player();
            label.set_label(&format!("{} {}", p.FirstName, p.LastName));
        });

        // Age
        setup_label(&imp.factory_age);
        imp.factory_age.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();

            if player_obj.player().MotherClubBonus {
                label.add_css_class("mother-club");
            } else {
                label.remove_css_class("mother-club");
            }

            let p = player_obj.player();
            label.set_label(&format!("{}.{}", p.Age, p.AgeDays.unwrap_or(0)));
        });

        // Form
        setup_label(&imp.factory_form);
        imp.factory_form.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();

            if player_obj.player().MotherClubBonus {
                label.add_css_class("mother-club");
            } else {
                label.remove_css_class("mother-club");
            }

            label.set_label(&player_obj.player().PlayerForm.to_string());
        });

        // TSI
        setup_label(&imp.factory_tsi);
        imp.factory_tsi.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            
            if player_obj.player().MotherClubBonus {
                label.add_css_class("mother-club");
            } else {
                label.remove_css_class("mother-club");
            }

            label.set_label(&player_obj.player().TSI.to_string());
        });
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
    }

    fn load_players(&self, team_id: u32) {
        let imp = self.imp();
        let db = DbManager::new();

        if let Ok(mut conn) = db.get_connection() {
            match get_players_for_team(&mut conn, team_id) {
                Ok(players) => {
                    info!("Loaded {} players for team {}", players.len(), team_id);
                    let model = gio::ListStore::new::<PlayerObject>();
                    for p in players {
                        model.append(&PlayerObject::new(p));
                    }

                    let selection = gtk::SingleSelection::new(Some(model));
                    imp.view_players.set_model(Some(&selection));
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
