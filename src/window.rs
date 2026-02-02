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
use gtk::{gio, glib, CompositeTemplate, TemplateChild};
use log::{debug, error, info};
use std::cell::RefCell;
use std::rc::Rc;

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

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Hoctane/window.ui")]
    pub struct HoctaneWindow {
        #[template_child]
        pub combo_teams: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub view_players: TemplateChild<gtk::ColumnView>,

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
                label.set_halign(gtk::Align::Start);
                item.set_child(Some(&label));
            });
        };

        // Number
        setup_label(&imp.factory_number);
        imp.factory_number.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
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
            let p = player_obj.player();
            label.set_label(&format!("{} {}", p.FirstName, p.LastName));
        });

        // Age
        setup_label(&imp.factory_age);
        imp.factory_age.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            let p = player_obj.player();
            label.set_label(&format!("{}.{}", p.Age, p.AgeDays.unwrap_or(0)));
        });

        // Form
        setup_label(&imp.factory_form);
        imp.factory_form.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_label(&player_obj.player().PlayerForm.to_string());
        });

        // TSI
        setup_label(&imp.factory_tsi);
        imp.factory_tsi.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let player_obj = item.item().and_downcast::<PlayerObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
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
                    for (id, name) in teams {
                        imp.combo_teams.append(Some(&id.to_string()), &name);
                    }
                    if imp.combo_teams.button_sensitivity() == gtk::SensitivityType::On {
                        imp.combo_teams.set_active(Some(0)); // Select first
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

        imp.combo_teams.connect_changed(move |combo| {
            if let Some(id_str) = combo.active_id() {
                if let Ok(team_id) = id_str.parse::<u32>() {
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
