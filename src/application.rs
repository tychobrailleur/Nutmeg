/* application.rs
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

use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::service::sync::DataSyncService;
use crate::window::NutmegWindow;
use log::{error, info};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NutmegApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for NutmegApplication {
        const NAME: &'static str = "NutmegApplication";
        type Type = super::NutmegApplication;
        type ParentType = gtk::Application; // Should we make it adw::Application?
    }

    impl ObjectImpl for NutmegApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for NutmegApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();

            info!("Application activated");

            // Check for first run (empty database)
            let db_manager = crate::db::manager::DbManager::new();
            let is_first_run = match db_manager.has_users() {
                Ok(has) => !has,
                Err(e) => {
                    error!("Failed to check DB state: {}. Defaulting to first run.", e);
                    true
                }
            };

            if is_first_run {
                info!("First run detected (no users in DB)");
                // Show Setup Window immediately to keep app alive
                let setup = crate::setup_window::SetupWindow::new(&*application);
                setup.present();

                // Try to sync with stored credentials
                let app_clone = application.clone();
                let setup_clone = setup.clone();

                glib::MainContext::default().spawn_local(async move {
                    let db = std::sync::Arc::new(crate::db::manager::DbManager::new());
                    let sync = crate::service::sync::SyncService::new(db);
                    let key = crate::config::consumer_key();
                    let secret = crate::config::consumer_secret();

                    match sync
                        .perform_sync_with_stored_secrets(
                            key,
                            secret,
                            Box::new(|p, m| {
                                log::debug!("Background sync: {:.0}% - {}", p * 100.0, m)
                            }),
                        )
                        .await
                    {
                        Ok(true) => {
                            info!("Successfully synced with stored credentials");
                            // Synced successfully, open main window and close setup
                            let window = crate::window::NutmegWindow::new(&app_clone);
                            window.present();
                            setup_clone.close();
                        }
                        Ok(false) => {
                            info!("No stored credentials found, staying on setup screen");
                            // No secrets, setup is already shown
                        }
                        Err(e) => {
                            error!("Failed to sync with stored secrets: {}", e);
                            // Setup is already shown
                        }
                    }
                });
            } else {
                info!("Existing user found, opening main window");
                // Show Main Window
                let window = application.active_window().unwrap_or_else(|| {
                    let window = NutmegWindow::new(&*application);
                    window.upcast()
                });
                window.present();
            }
        }
    }

    impl GtkApplicationImpl for NutmegApplication {}
}

glib::wrapper! {
    pub struct NutmegApplication(ObjectSubclass<imp::NutmegApplication>)
        @extends gio::Application, gtk::Application,
    @implements gio::ActionGroup, gio::ActionMap;
}

impl NutmegApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/org/gnome/Nutmeg")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        if let Some(window) = self.active_window() {
            let about = gtk::AboutDialog::builder()
                .transient_for(&window)
                .modal(true)
                .program_name("nutmeg")
                .logo_icon_name("org.gnome.Nutmeg")
                .version(VERSION)
                .authors(vec!["Sébastien Le Callonnec"])
                //     .license_type(gtk::LicenseType::Gpl3)
                .translator_credits(gettext("translator-credits"))
                .copyright("© 2026 Sébastien Le Callonnec")
                .build();

            about.present();
        }
    }
}
