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
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <https://www.gnu.org/licenses/>.

*
* SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::db::manager::DbManager;
use crate::db::teams::get_teams_summary;
use crate::service::sync::DataSyncService;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate, TemplateChild};
use log::{debug, error, info};

use crate::ui::context_object::ContextObject;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;

use crate::squad::player_details::SquadPlayerDetails;
use crate::squad::player_list::SquadPlayerList;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/window.ui")]
    pub struct NutmegWindow {
        #[template_child]
        pub combo_teams: TemplateChild<gtk::DropDown>,

        #[template_child]
        pub player_list: TemplateChild<SquadPlayerList>,

        #[template_child]
        pub player_details: TemplateChild<SquadPlayerDetails>,

        #[template_child]
        pub team_sync: TemplateChild<gtk::Button>,

        // https://docs.gtk.org/gtk4/class.Revealer.html
        #[template_child]
        pub sync_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub sync_progress_bar: TemplateChild<gtk::ProgressBar>,

        #[template_child]
        pub sync_status_label: TemplateChild<gtk::Label>,

        pub context_object: ContextObject,
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

            // Load Teams
            obj.load_teams();

            // Setup Bindings
            obj.setup_bindings();

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

            obj.set_maximized(true);
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

    // Loads the teams associated with the user to populate the main dropdown.
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

                        // Set label with markup (name + grey ID)
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

                    // Select first team if available
                    // Property binding will automatically load players
                    if model.n_items() > 0 {
                        imp.combo_teams.set_selected(0);
                    }
                }
                Err(e) => error!("Failed to load teams: {}", e),
            }
        } else {
            error!("Failed to get DB connection");
        }
    }

    fn setup_bindings(&self) {
        let imp = self.imp();
        let model = &imp.context_object;

        // Bind combo_teams selected item to ContextObject selected-team
        imp.combo_teams
            .bind_property("selected-item", model, "selected-team")
            .sync_create()
            .build();

        // Bind ContextObject players to TreeView model (inside PlayerList)
        model
            .bind_property("players", &imp.player_list.tree_view(), "model")
            .sync_create()
            .build();

        // Listen to selected-player changes in ContextObject to update details panel
        let window = self.clone();
        model.connect_notify_local(Some("selected-player"), move |model, _| {
            let player_obj: Option<PlayerObject> = model.property("selected-player");
            window.imp().player_details.set_player(player_obj);
        });
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        // Player selection handler - updates ContextObject
        let view = imp.player_list.tree_view();
        let selection = view.selection();
        let context_object = imp.context_object.clone();

        selection.connect_changed(move |selection| {
            #[allow(deprecated)]
            if let Some((model, iter)) = selection.selected() {
                #[allow(deprecated)]
                let obj_val = model.get_value(&iter, 18);
                if let Ok(player_obj) = obj_val.get::<PlayerObject>() {
                    context_object.set_selected_player(Some(player_obj));
                }
            } else {
                context_object.set_selected_player(None);
            }
        });

        // Sync Handler
        let window_weak = self.downgrade();
        imp.team_sync.connect_clicked(move |_| {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let imp = window.imp();

            // Disable button
            imp.team_sync.set_sensitive(false);

            // Show status bar
            imp.sync_revealer.set_reveal_child(true);
            imp.sync_progress_bar.set_fraction(0.0);
            imp.sync_status_label.set_label("Starting sync...");

            // Channel for progress updates
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<(f64, String)>();

            // Handle progress updates on main thread
            let progress_bar = imp.sync_progress_bar.clone();
            let status_label = imp.sync_status_label.clone();

            glib::MainContext::default().spawn_local(async move {
                while let Some((p, msg)) = receiver.recv().await {
                    progress_bar.set_fraction(p);
                    status_label.set_label(&msg);
                }
            });

            // Keep weak ref to update UI later
            let window_weak_completion = window.downgrade();

            glib::MainContext::default().spawn_local(async move {
                let db = std::sync::Arc::new(DbManager::new());
                let sync = crate::service::sync::SyncService::new(db);
                let key = crate::config::consumer_key();
                let secret = crate::config::consumer_secret();

                // Progress callback
                let progress_cb = Box::new(move |p: f64, msg: &str| {
                    let _ = sender.send((p, msg.to_string()));
                });

                match sync
                    .perform_sync_with_stored_secrets(key, secret, progress_cb)
                    .await
                {
                    Ok(true) => {
                        info!("Sync completed successfully");
                    }
                    Ok(false) => {
                        error!("Sync failed: No credentials found");
                    }
                    Err(e) => {
                        error!("Sync failed: {}", e);
                    }
                }

                // UI Cleanup
                if let Some(win) = window_weak_completion.upgrade() {
                    let imp = win.imp();

                    // Delay hiding the status bar slightly so user sees 100%
                    glib::timeout_future_seconds(2).await;

                    imp.sync_revealer.set_reveal_child(false);
                    imp.team_sync.set_sensitive(true);

                    // Refresh teams
                    win.load_teams();
                }
            });
        });
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
