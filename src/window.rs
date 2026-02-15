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
// use crate::db::teams::get_teams_summary; // Moved to controller
// use crate::service::sync::DataSyncService; // Moved to controller
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate, TemplateChild};
use log::{debug, error, info, warn};

use crate::ui::context_object::ContextObject;
use crate::ui::player_object::PlayerObject;
use crate::ui::team_object::TeamObject;

use crate::squad::player_details::SquadPlayerDetails;
use crate::squad::player_list::SquadPlayerList;
// use crate::ui::oauth_dialog::OAuthDialog; // Not needed anymore


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
    pub fn load_teams(&self) {
        use crate::ui::controllers::teams::TeamController;
        TeamController::load_teams(&self.imp().combo_teams);
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
        }
    }

