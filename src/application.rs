/* application.rs
 *
 * Copyright 2026 sebastien
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
use crate::window::HoctaneWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct HoctaneApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for HoctaneApplication {
        const NAME: &'static str = "HoctaneApplication";
        type Type = super::HoctaneApplication;
        type ParentType = gtk::Application; // Should we make it adw::Application?
    }

    impl ObjectImpl for HoctaneApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for HoctaneApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();

            // Check for first run (empty database)
            let db_manager = crate::db::manager::DbManager::new();
            let is_first_run = match db_manager.has_users() {
                Ok(has) => !has,
                Err(e) => {
                    eprintln!("Failed to check DB state: {}. Defaulting to first run.", e);
                    true
                }
            };

            if is_first_run {
                // Show Setup Wizard
                let setup = crate::setup_window::SetupWindow::new(&*application);
                // setup.build_ui() removed as it is now constructed via template
                setup.present();
            } else {
                // Show Main Window
                let window = application.active_window().unwrap_or_else(|| {
                    let window = HoctaneWindow::new(&*application);
                    window.upcast()
                });
                window.present();
            }
        }
    }

    impl GtkApplicationImpl for HoctaneApplication {}
}

glib::wrapper! {
    pub struct HoctaneApplication(ObjectSubclass<imp::HoctaneApplication>)
        @extends gio::Application, gtk::Application,
    @implements gio::ActionGroup, gio::ActionMap;
}

impl HoctaneApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/org/gnome/Hoctane")
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
                .program_name("hoctane")
                .logo_icon_name("org.gnome.Hoctane")
                .version(VERSION)
                .authors(vec!["sebastien"])
                .translator_credits(&gettext("translator-credits"))
                .copyright("© 2026 sebastien")
                .build();

            about.present();
        }
    }
}
