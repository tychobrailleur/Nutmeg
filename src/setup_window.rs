/* setup_window.rs
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

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, Entry, Stack, Window};
use log::{debug, error, info};
use open;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::config::{consumer_key, consumer_secret};
use crate::db::manager::DbManager;
use crate::service::sync::SyncService;
use crate::window::HoctaneWindow;

mod imp {
    use super::*;
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, TemplateChild};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Hoctane/setup_window.ui")]
    pub struct SetupWindow {
        #[template_child]
        pub stack: TemplateChild<Stack>,
        #[template_child]
        pub btn_start: TemplateChild<Button>,
        #[template_child]
        pub btn_browser: TemplateChild<Button>,
        #[template_child]
        pub entry_code: TemplateChild<Entry>,
        #[template_child]
        pub btn_verify: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SetupWindow {
        const NAME: &'static str = "SetupWindow";
        type Type = super::SetupWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SetupWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_signals();
        }
    }
    impl WidgetImpl for SetupWindow {}
    impl WindowImpl for SetupWindow {}
}

glib::wrapper! {
    pub struct SetupWindow(ObjectSubclass<imp::SetupWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SetupWindow {
    pub fn new<P: IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn setup_signals(&self) {
        use crate::service::auth::{AuthenticationService, HattrickAuthService};
        use crate::service::secret::{GnomeSecretService, SecretStorageService};
        use crate::service::sync::DataSyncService;

        let imp = self.imp();

        // Btn Start -> Page 2
        let stack = imp.stack.clone();
        imp.btn_start.connect_clicked(move |_| {
            debug!("Start button clicked");
            stack.set_visible_child_name("page2");
        });

        // Auth State (Request Token, Secret)
        let auth_state = Rc::new(RefCell::new((None::<String>, None::<String>)));

        // Btn Browser -> Page 3
        let auth_state_clone = auth_state.clone();
        let stack = imp.stack.clone();

        imp.btn_browser.connect_clicked(move |_| {
            debug!("Browser button clicked");
            let state = auth_state_clone.clone();
            let stack = stack.clone();

            glib::MainContext::default().spawn_local(async move {
                // Call AuthService
                let res = tokio::task::spawn_blocking(move || {
                    let service = HattrickAuthService::default();
                    service.get_authorization_url()
                })
                .await;

                match res {
                    Ok(Ok((url, rt, rs))) => {
                        info!("Got auth URL: {}", url);
                        let mut data = state.borrow_mut();
                        data.0 = Some(rt);
                        data.1 = Some(rs);

                        if let Err(e) = open::that(url) {
                            eprintln!("Failed to open browser: {}", e);
                        }
                        stack.set_visible_child_name("page3");
                    }
                    Ok(Err(e)) => error!("Auth error: {}", e),
                    Err(e) => error!("Task join error: {}", e),
                }
            });
        });

        // Btn Verify -> Sync -> Page 4 -> Finish
        let auth_state_clone2 = auth_state.clone();
        let stack = imp.stack.clone();
        let entry = imp.entry_code.clone();
        let window = self.clone();

        imp.btn_verify.connect_clicked(move |_| {
            debug!("Verify button clicked");
            let code = entry.text().to_string();
            let state = auth_state_clone2.clone();
            let stack = stack.clone();
            let win = window.clone();

            stack.set_visible_child_name("page4");

            glib::MainContext::default().spawn_local(async move {
                let (rt, rs) = {
                    let s = state.borrow();
                    (s.0.clone(), s.1.clone())
                };

                let (rt, rs) = match (rt, rs) {
                    (Some(rt), Some(rs)) => (rt, rs),
                    _ => {
                        error!("Missing request token state");
                        stack.set_visible_child_name("page2");
                        return;
                    }
                };

                let code_clone = code.clone();

                // Exchange Code
                let verify_res = tokio::task::spawn_blocking(move || {
                    let service = HattrickAuthService::default();
                    service.verify_user(&code_clone, &rt, &rs)
                })
                .await;

                match verify_res {
                    Ok(Ok((access_token, access_secret))) => {
                        let secret_service = GnomeSecretService::new();
                        if let Err(e) = secret_service
                            .store_secret("access_token", &access_token)
                            .await
                        {
                            error!("Failed to store access token: {}", e);
                        }
                        if let Err(e) = secret_service
                            .store_secret("access_secret", &access_secret)
                            .await
                        {
                            error!("Failed to store access secret: {}", e);
                        }

                        let db_manager = Arc::new(DbManager::new());
                        let sync_service = SyncService::new(db_manager);

                        match sync_service
                            .perform_initial_sync(consumer_key(), consumer_secret())
                            .await
                        {
                            Ok(_) => {
                                info!("Initial sync successful");
                                if let Some(app) = win.application() {
                                    let main_win = HoctaneWindow::new(&app);
                                    main_win.present();
                                }
                                win.close();
                            }
                            Err(e) => {
                                error!("Sync Error: {}", e);
                                stack.set_visible_child_name("page3");
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Verification Error: {}", e);
                        stack.set_visible_child_name("page3");
                    }
                    Err(e) => {
                        error!("Join Error: {}", e);
                        stack.set_visible_child_name("page3");
                    }
                }
            });
        });
    }
}
