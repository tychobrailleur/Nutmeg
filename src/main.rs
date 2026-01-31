/* main.rs
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

mod application;
mod config;
mod window;
mod chpp;

use std::ptr;

use self::application::HoctaneApplication;
use self::window::HoctaneWindow;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::{gio, glib};
use gtk::prelude::*;
use std::env;

fn prompt_browser(url: &str) {
    open::that(url).expect("Failed to open URL in browser");
}

fn main() -> glib::ExitCode {
    // // Set up gettext translations
    // bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    // bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
    //     .expect("Unable to set the text domain encoding");
    // textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // // Load resources
    // let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/hoctane.gresource")
    //     .expect("Could not load resources");
    // gio::resources_register(&resources);

    // // Create a new GtkApplication. The application manages our main loop,
    // // application windows, integration with the window manager/compositor, and
    // // desktop features such as file opening and single-instance applications.
    // let app = HoctaneApplication::new(
    //     "org.gnome.Hoctane",
    //     &gio::ApplicationFlags::empty()
    // );

    // // Run the application. This function will block until the application
    // // exits. Upon return, we have our exit code to return to the shell. (This
    // // is the code you see when you do `echo $?` after running a command in a
    // // terminal.
    // app.run()

    let consumer_key = env::var("HT_CONSUMER_KEY").unwrap().to_string();
    let consumer_secret = env::var("HT_CONSUMER_SECRET").unwrap().to_string();


    let oauth_settings = OAuthSettings.default();
    let oauth_settings_with_keys = request_token(
        &oauth_settings,
        &consumer_key,
        &consumer_secret,
        prompt_browser,
    ).expect("Failed to obtain request token");

    glib::ExitCode::SUCCESS
}
