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

// use crate::db::manager::DbManager;
// use crate::db::teams::get_teams_summary; // Moved to controller
// use crate::service::sync::DataSyncService; // Moved to controller
use crate::service::secret::SecretStorageService;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate, TemplateChild};
use log::info;

use crate::ui::context_object::ContextObject;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;
use crate::rating::ui::page::FormationOptimiserWidget;

use crate::squad::player_details::SquadPlayerDetails;
use crate::squad::player_list::SquadPlayerList;
use crate::series::page::SeriesPage;
// use crate::ui::oauth_dialog::OAuthDialog; // Not needed anymore

mod imp {
    use super::*;

    // See https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4_macros/derive.CompositeTemplate.html
    // for composite template, it brings template and template_child attributes.
    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/window.ui")]
    pub struct NutmegWindow {
        #[template_child]
        pub combo_teams: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub optimiser: TemplateChild<FormationOptimiserWidget>,

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

        #[template_child]
        pub notebook: TemplateChild<gtk::Notebook>,

        #[template_child]
        pub series_page: TemplateChild<SeriesPage>,

        pub context_object: ContextObject,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NutmegWindow {
        const NAME: &'static str = "NutmegWindow";
        type Type = super::NutmegWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            FormationOptimiserWidget::ensure_type();
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

            // Setup window actions
            obj.setup_actions();

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
    pub fn load_teams(&self) {
        use crate::ui::controllers::teams::TeamController;
        TeamController::load_teams(&self.imp().combo_teams);
    }

    pub fn load_current_team_series_data(&self) {
        let imp = self.imp();
        let model = &imp.context_object;
        let team_obj: Option<TeamObject> = model.property("selected-team");

        if let Some(team_object) = team_obj {
            let team_data = team_object.team_data();
            let team_id = team_data.id;
            let window_weak = self.downgrade();

            glib::MainContext::default().spawn_local(async move {
                use crate::series::controller::SeriesController;

                match SeriesController::load_series_data(team_id).await {
                    Ok((league_data, matches_data)) => {
                        if let Some(window) = window_weak.upgrade() {
                            window
                                .imp()
                                .series_page
                                .set_data(Some(&league_data), Some(&matches_data));
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load series data: {}", e);
                        // TODO: Show error in UI
                    }
                }
            });
        } else {
            imp.series_page.set_data(None, None);
        }
    }

    fn setup_bindings(&self) {
        let imp = self.imp();
        let model = &imp.context_object;

        // Listen to selected-player changes in ContextObject to update details panel
        // REMOVED: Handled in selection handler to include preferred position which is not in ContextObject
        // let window = self.clone();
        // model.connect_notify_local(Some("selected-player"), move |model, _| {
        //     let player_obj: Option<PlayerObject> = model.property("selected-player");
        //     window.imp().player_details.set_player(player_obj, None);
        // });

        // Listen to players list changes to update optimiser AND bind to player list
        let window = self.clone();
        model.connect_notify_local(Some("players"), move |model, _| {
             window.update_optimiser_players(model.property("players"));
        });


        // Initialize optimiser with current players (if any already loaded)
        if let Some(store) = model.property::<Option<gtk::ListStore>>("players") {
            self.update_optimiser_players(Some(store));
        }

        // Bind combo_teams selected item to ContextObject selected-team
        imp.combo_teams
            .bind_property("selected-item", model, "selected-team")
            .sync_create()
            .build();

        // Listen to selected-team changes to load series data
        let window = self.clone();
        model.connect_notify_local(Some("selected-team"), move |_, _| {
            window.load_current_team_series_data();
        });

        // Bind ContextObject players to TreeView model (inside PlayerList)
        model
            .bind_property("players", &imp.player_list.tree_view(), "model")
            .sync_create()
            .build();
    }

    fn update_optimiser_players(&self, list_store: Option<gtk::ListStore>) {
        if let Some(store) = list_store {
            let mut players = Vec::new();
            if let Some(iter) = store.iter_first() {
                loop {
                    #[allow(deprecated)]
                    let obj_val = store.get_value(&iter, 18);
                    if let Ok(player_obj) = obj_val.get::<PlayerObject>() {
                        players.push(player_obj.player().clone());
                    }
                    if !store.iter_next(&iter) {
                        break;
                    }
                }
            }
            info!("Updating optimiser with {} players", players.len());
            self.imp().optimiser.set_players(players);
        } else {
            self.imp().optimiser.set_players(Vec::new());
        }
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        // Player selection handler - updates ContextObject
        let view = imp.player_list.tree_view();
        let selection = view.selection();
        let context_object = imp.context_object.clone();
        
        // Needed for manual update of details
        let player_details = imp.player_details.clone();

        selection.connect_changed(move |selection| {
            #[allow(deprecated)]
            if let Some((model, iter)) = selection.selected() {
                #[allow(deprecated)]
                let obj_val = model.get_value(&iter, 18);
                
                let player_obj = if let Ok(obj) = obj_val.get::<PlayerObject>() {
                    Some(obj)
                } else {
                    None
                };

                // Get preferred position from column 11
                #[allow(deprecated)]
                let best_pos_val = model.get_value(&iter, 11);
                let best_pos = best_pos_val.get::<String>().ok();

                // Update ContextObject (for other consumers)
                context_object.set_selected_player(player_obj.clone());
                
                // Update UI directly to include best_pos
                player_details.set_player(player_obj, best_pos);
            } else {
                context_object.set_selected_player(None);
                player_details.set_player(None, None);
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
                // Delegate to SyncController
                use crate::ui::controllers::sync::SyncController;
                SyncController::perform_sync(window_weak_completion.clone(), sender).await;

                // UI Cleanup
                if let Some(win) = window_weak_completion.upgrade() {
                    let imp = win.imp();

                    // Delay hiding the status bar slightly so user sees result
                    glib::timeout_future_seconds(2).await;

                    imp.sync_revealer.set_reveal_child(false);
                    imp.team_sync.set_sensitive(true);

                    // Refresh teams if successful
                    win.load_teams();
                }
            });
        });

        // Notebook page switch handler
        let window_weak = self.downgrade();
        imp.notebook.connect_switch_page(move |_, page, _| {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let series_page_widget = window.imp().series_page.upcast_ref::<gtk::Widget>();
            if page == series_page_widget {
                window.load_current_team_series_data();
            }
        });
    }

    fn setup_actions(&self) {
        use gio::prelude::*;
        use gtk::prelude::*;

        let window_weak = self.downgrade();

        // Action: clear-database
        let clear_db_action = gio::ActionEntry::builder("clear-database")
            .activate(move |window: &Self, _, _| {
                let dialog = gtk::MessageDialog::builder()
                    .transient_for(window)
                    .modal(true)
                    .message_type(gtk::MessageType::Warning)
                    .buttons(gtk::ButtonsType::OkCancel)
                    .text("Clear Database?")
                    .secondary_text("This will permanently delete ALL synced data including teams, players, matches, and league information. This cannot be undone.")
                    .build();

                let window_weak = window.downgrade();
                dialog.connect_response(move |dialog, response| {
                    if response == gtk::ResponseType::Ok {
                        if let Some(win) = window_weak.upgrade() {
                            let db = crate::db::manager::DbManager::new();
                            match db.clear_all_data() {
                                Ok(()) => {
                                    log::info!("Database cleared successfully");
                                    // Reload teams to show empty state
                                    win.load_teams();

                                    let success_dialog = gtk::MessageDialog::builder()
                                        .transient_for(&win)
                                        .modal(true)
                                        .message_type(gtk::MessageType::Info)
                                        .buttons(gtk::ButtonsType::Ok)
                                        .text("Database Cleared")
                                        .secondary_text("All data has been removed from the database.")
                                        .build();
                                    success_dialog.connect_response(|dialog, _| {
                                        dialog.close();
                                    });
                                    success_dialog.present();
                                }
                                Err(e) => {
                                    log::error!("Failed to clear database: {}", e);
                                    let error_dialog = gtk::MessageDialog::builder()
                                        .transient_for(&win)
                                        .modal(true)
                                        .message_type(gtk::MessageType::Error)
                                        .buttons(gtk::ButtonsType::Ok)
                                        .text("Failed to Clear Database")
                                        .secondary_text(format!("Error: {}", e))
                                        .build();
                                    error_dialog.connect_response(|dialog, _| {
                                        dialog.close();
                                    });
                                    error_dialog.present();
                                }
                            }
                        }
                    }
                    dialog.close();
                });

                dialog.present();
            })
            .build();

        // Action: delete-secrets
        let delete_secrets_action = gio::ActionEntry::builder("delete-secrets")
            .activate(move |window: &Self, _, _| {
                let dialog = gtk::MessageDialog::builder()
                    .transient_for(window)
                    .modal(true)
                    .message_type(gtk::MessageType::Warning)
                    .buttons(gtk::ButtonsType::OkCancel)
                    .text("Delete OAuth Secrets?")
                    .secondary_text("This will delete all stored OAuth tokens. You will need to re-authenticate the next time you sync.")
                    .build();

                let window_weak = window_weak.clone();
                dialog.connect_response(move |dialog, response| {
                    if response == gtk::ResponseType::Ok {
                        if let Some(_win) = window_weak.upgrade() {
                            let secret_service = crate::service::secret::GnomeSecretService::new();
                            glib::MainContext::default().spawn_local(async move {
                                match secret_service.clear_all_oauth_secrets().await {
                                    Ok(()) => {
                                        log::info!("OAuth secrets deleted successfully");
                                        let success_dialog = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Info)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text("Secrets Deleted")
                                            .secondary_text("All OAuth tokens have been removed. You will need to re-authenticate on next sync.")
                                            .build();
                                        success_dialog.connect_response(|dialog, _| {
                                            dialog.close();
                                        });
                                        success_dialog.present();
                                    }
                                    Err(e) => {
                                        log::error!("Failed to delete secrets: {}", e);
                                        let error_dialog = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Error)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text("Failed to Delete Secrets")
                                            .secondary_text(format!("Error: {}", e))
                                            .build();
                                        error_dialog.connect_response(|dialog, _| {
                                            dialog.close();
                                        });
                                        error_dialog.present();
                                    }
                                }
                            });
                        }
                    }
                    dialog.close();
                });

                dialog.present();
            })
            .build();

        self.add_action_entries([clear_db_action, delete_secrets_action]);
    }
}
