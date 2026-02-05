/* main.rs
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

mod application;
mod chpp;
mod config;
mod db;
mod player_display;
mod service;
mod setup_window;
mod window;

use self::application::NutmegApplication;

use config::{GETTEXT_PACKAGE, LOCALEDIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};

use tokio::runtime::Runtime;

fn main() -> glib::ExitCode {
    // Load env vars from .env file if present
    match dotenvy::dotenv() {
        Ok(path) => println!("INFO: Loaded .env from {:?}", path),
        Err(e) => println!("INFO: Could not load .env: {}", e),
    }

    match std::env::var("HT_CONSUMER_KEY") {
        Ok(val) => println!("INFO: HT_CONSUMER_KEY found (length: {})", val.len()),
        Err(e) => println!("ERROR: HT_CONSUMER_KEY not found in env: {}", e),
    }
    match std::env::var("HT_CONSUMER_SECRET") {
        Ok(val) => println!("INFO: HT_CONSUMER_SECRET found (length: {})", val.len()),
        Err(e) => println!("ERROR: HT_CONSUMER_SECRET not found in env: {}", e),
    }

    // Initialize logger
    env_logger::init();

    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    gio::resources_register_include!("nutmeg.gresource").expect("Failed to register resources");

    // Create a new GtkApplication. The application manages our main loop,
    // application windows, integration with the window manager/compositor, and
    // desktop features such as file opening and single-instance applications.
    let app = NutmegApplication::new("org.gnome.Nutmeg", &gio::ApplicationFlags::NON_UNIQUE);

    // Initialize Tokio Runtime to support async features in the GTK loop
    let runtime = Runtime::new().expect("Unable to create Tokio runtime");
    let _guard = runtime.enter();

    // Run the application. This function will block until the application
    // exits. Upon return, we have our exit code to return to the shell. (This
    // is the code you see when you do `echo $?` after running a command in a
    // terminal.
    app.run()

    /*
    // Load env vars
    if let Err(_) = dotenvy::dotenv() {
        // Fallback to .zshrc if .env fails (as requested originally)
        let _ = dotenvy::from_filename(".zshrc");
    }

    crate::chpp::authenticator::perform_cli_auth()
    */
}
