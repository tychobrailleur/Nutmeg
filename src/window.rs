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

use crate::service::secret::SecretStorageService;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate, TemplateChild};
use log::info;

use crate::rating::ui::page::FormationOptimiserWidget;
use crate::training::ui::page::TrainingPlannerPage;
use crate::ui::context_object::ContextObject;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;

use crate::opponent_analysis::ui::OpponentAnalysis;
use crate::series::ui::page::SeriesPage;
use crate::squad::ui::player_details::SquadPlayerDetails;
use crate::squad::ui::player_list::SquadPlayerList;
mod imp {
    use super::*;

    // See https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4_macros/derive.CompositeTemplate.html
    // for composite template, it brings template and template_child attributes.
    #[derive(Default, CompositeTemplate)]
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

        #[template_child]
        pub training_planner: TemplateChild<TrainingPlannerPage>,

        #[template_child]
        pub opponent_analysis: TemplateChild<OpponentAnalysis>,

        pub context_object: ContextObject,
        pub main_controller: std::cell::RefCell<
            Option<std::rc::Rc<crate::ui::controllers::main_controller::MainController>>,
        >,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NutmegWindow {
        const NAME: &'static str = "NutmegWindow";
        type Type = super::NutmegWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            FormationOptimiserWidget::ensure_type();
            TrainingPlannerPage::ensure_type();
            OpponentAnalysis::ensure_type();
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

            // Setup Signals
            obj.setup_signals();

            // Setup Bindings
            obj.setup_bindings();

            let factory = gtk::SignalListItemFactory::new();
            obj.setup_team_dropdown_factory(&factory);
            obj.imp().combo_teams.set_factory(Some(&factory));

            // Load Global State (Teams, etc.)
            let controller = crate::ui::controllers::main_controller::MainController::new(
                obj.imp().context_object.clone(),
            );
            obj.imp().main_controller.replace(Some(controller.clone()));
            controller.refresh_all_teams();

            // Inject ContextObject into sub-pages
            obj.imp()
                .training_planner
                .set_context_object(&obj.imp().context_object.clone());

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

    // Team loading is now managed internally by `MainController`,
    // and DB lookups for series data happen immediately upon `selected-team` change.

    // We keep these empty implementations around if they are called recursively from signals
    // but they no longer perform any logic themselves. We will prune them shortly.

    fn setup_bindings(&self) {
        let imp = self.imp();
        let model = &imp.context_object;

        // Listen to players list changes to update optimiser AND bind to player list
        let window = self.clone();
        model.connect_notify_local(Some("players"), move |model, _| {
            window.update_optimiser_players(model.property("players"));
        });

        // Initialise optimiser with current players (if any already loaded)
        if let Some(store) = model.property::<Option<gtk::ListStore>>("players") {
            self.update_optimiser_players(Some(store));
        }

        // Wait for data load notification rather than selected-team
        // This ensures the data is strictly hydrated from the DB before the series page draws
        let window = self.clone();
        model.connect_notify_local(Some("data-loaded"), move |m, _| {
            let win_imp = window.imp();

            // Extract the snapshot
            let league_opt = m.league_details();
            let matches_opt = m.matches();
            let all_series = m.all_series_matches().unwrap_or_default();
            let logos = m.series_logo_urls().unwrap_or_default();

            win_imp.series_page.set_data(
                league_opt.as_ref(),
                matches_opt.as_ref(),
                &all_series,
                &logos,
            );
        });

        // Bind ContextObject selected-team to OpponentAnalysis
        model
            .bind_property("selected-team", &*imp.opponent_analysis, "selected-team")
            .sync_create()
            .build();

        imp.opponent_analysis.set_property("context", model);
        imp.optimiser.set_property("context", model);

        // Bind ContextObject all-teams to the main dropdown.
        model
            .bind_property("all-teams", &*imp.combo_teams, "model")
            .sync_create()
            .build();

        // Bind combo_teams selected item to ContextObject selected-team.
        imp.combo_teams
            .bind_property("selected-item", model, "selected-team")
            .sync_create()
            .build();

        // Bind ContextObject players to TreeView model (inside PlayerList)
        model
            .bind_property("players", &imp.player_list.tree_view(), "model")
            .sync_create()
            .build();
    }

    fn setup_team_dropdown_factory(&self, factory: &gtk::SignalListItemFactory) {
        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            hbox.set_margin_start(4);
            hbox.set_margin_end(4);
            hbox.set_margin_top(4);
            hbox.set_margin_bottom(4);

            let logo = gtk::Image::new();
            logo.set_pixel_size(24);
            hbox.append(&logo);

            let label = gtk::Label::new(None);
            label.set_xalign(0.0);
            hbox.append(&label);

            item.set_child(Some(&hbox));
        });

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

            let markup = format!(
                "{} <span foreground='gray'>({})</span>",
                glib::markup_escape_text(&team_data.name),
                team_data.id
            );
            label.set_markup(&markup);

            if let Some(mut url) = team_data.logo_url {
                if url.starts_with("//") {
                    url = format!("https:{}", url);
                }

                let logo_clone = logo.clone();
                glib::MainContext::default().spawn_local(async move {
                    use crate::utils::image::load_image_from_url;
                    match load_image_from_url(&url).await {
                        Ok(texture) => {
                            logo_clone.set_paintable(Some(&texture));
                        }
                        Err(e) => {
                            log::debug!("Failed to load team logo from {}: {}", url, e);
                        }
                    }
                });
            }
        });
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

                let player_obj = obj_val.get::<PlayerObject>().ok();

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
        let context_object_clone = imp.context_object.clone();
        imp.team_sync.connect_clicked(move |_| {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let imp = window.imp();
            let context = context_object_clone.clone();

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
                SyncController::perform_sync(window_weak_completion.clone(), context, sender).await;

                // UI Cleanup
                if let Some(win) = window_weak_completion.upgrade() {
                    let imp = win.imp();

                    // Delay hiding the status bar slightly so user sees result
                    glib::timeout_future_seconds(2).await;

                    imp.sync_revealer.set_reveal_child(false);
                    imp.team_sync.set_sensitive(true);

                    if let Some(ctrl) = win.imp().main_controller.borrow().as_ref() {
                        ctrl.refresh_all_teams();
                    }
                }
            });
        });

        // Notebook page switch handler (no longer needs to demand data loading)
        imp.notebook.connect_switch_page(move |_, _page, _| {
            // ContextObject is now autonomous; data should already be present
            // when switching to the Series tab.
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
                                    if let Some(ctrl) = win.imp().main_controller.borrow().as_ref() {
                                        ctrl.refresh_all_teams();
                                    }

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
                            let secret_service = crate::service::secret::SystemSecretService::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combo_teams_has_factory() {
        gtk::init().unwrap();
        gio::resources_register_include!("nutmeg.gresource").unwrap();
        let app = gtk::Application::new(
            Some("org.gnome.Nutmeg.TestWindowFactory"),
            gio::ApplicationFlags::FLAGS_NONE,
        );
        let window = NutmegWindow::new(&app);

        let dropdown = window.imp().combo_teams.clone();
        assert!(
            dropdown.factory().is_some(),
            "combo_teams dropdown MUST have a factory configured to render the team name and logo."
        );
    }
}
