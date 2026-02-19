/* page.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::metadata::ChppEndpoints;
use crate::db::download_entries::DownloadEntry;
use crate::db::manager::DbManager;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    glib, ColumnView, ColumnViewColumn, CompositeTemplate, Label, ListItem, SignalListItemFactory,
    StringList,
};
use std::cell::RefCell;
use std::sync::Arc;

// GObject wrapper for DownloadEntry
mod imp_model {
    use super::*;

    #[derive(Default)]
    pub struct AuditEntryObject {
        pub data: RefCell<Option<DownloadEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AuditEntryObject {
        const NAME: &'static str = "AuditEntryObject";
        type Type = super::AuditEntryObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AuditEntryObject {}
}

glib::wrapper! {
    pub struct AuditEntryObject(ObjectSubclass<imp_model::AuditEntryObject>);
}

impl AuditEntryObject {
    pub fn new(entry: DownloadEntry) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().data.borrow_mut() = Some(entry);
        obj
    }
}

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/developer/page.ui")]
    pub struct DeveloperAuditPage {
        #[template_child]
        pub filter_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub audit_view: TemplateChild<ColumnView>,

        pub db_manager: RefCell<Option<Arc<DbManager>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DeveloperAuditPage {
        const NAME: &'static str = "DeveloperAuditPage";
        type Type = super::DeveloperAuditPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DeveloperAuditPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_filter_dropdown();
            obj.setup_columns();
            obj.setup_signals();
        }
    }
    impl WidgetImpl for DeveloperAuditPage {}
    impl BoxImpl for DeveloperAuditPage {}
}

glib::wrapper! {
    pub struct DeveloperAuditPage(ObjectSubclass<imp::DeveloperAuditPage>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl DeveloperAuditPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    /// Set the database manager for querying entries
    pub fn set_db_manager(&self, db_manager: Arc<DbManager>) {
        *self.imp().db_manager.borrow_mut() = Some(db_manager);
    }

    /// Populate the filter dropdown with endpoint names from metadata
    fn setup_filter_dropdown(&self) {
        let imp = self.imp();
        let mut names: Vec<&str> = vec!["All"];
        let endpoints = ChppEndpoints::all();
        let endpoint_names: Vec<&str> = endpoints.iter().map(|e| e.name).collect();
        names.extend(endpoint_names);

        let string_list = StringList::new(&names);
        imp.filter_dropdown.set_model(Some(&string_list));
    }

    /// Set up ColumnView columns for the audit table
    fn setup_columns(&self) {
        let imp = self.imp();
        let view = &imp.audit_view;

        self.add_text_column(view, "ID", 60, false, |entry: &DownloadEntry| {
            entry.id.to_string()
        });

        self.add_text_column(view, "Download", 80, false, |entry: &DownloadEntry| {
            entry.download_id.to_string()
        });

        self.add_text_column(view, "Endpoint", 140, true, |entry: &DownloadEntry| {
            entry.endpoint.clone()
        });

        self.add_text_column(view, "Version", 60, false, |entry: &DownloadEntry| {
            entry.version.clone()
        });

        self.add_text_column(view, "Status", 80, false, |entry: &DownloadEntry| {
            entry.status.clone()
        });

        self.add_text_column(view, "Date", 160, false, |entry: &DownloadEntry| {
            entry.fetched_date.clone()
        });

        self.add_text_column(view, "Error", 200, true, |entry: &DownloadEntry| {
            entry.error_message.as_deref().unwrap_or("").to_string()
        });

        self.add_text_column(view, "Retries", 60, false, |entry: &DownloadEntry| {
            entry.retry_count.to_string()
        });

        // Action column with retry button
        self.setup_action_column(view);
    }

    /// Connect filter dropdown and refresh button signals
    fn setup_signals(&self) {
        let imp = self.imp();

        // Filter dropdown change
        let page_weak = self.downgrade();
        imp.filter_dropdown.connect_selected_notify(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.load_entries();
            }
        });

        // Refresh button click
        let page_weak = self.downgrade();
        imp.refresh_button.connect_clicked(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.load_entries();
            }
        });
    }

    /// Load entries from the database, applying any active filter
    pub fn load_entries(&self) {
        let imp = self.imp();
        let db_ref = imp.db_manager.borrow();
        let db = match db_ref.as_ref() {
            Some(db) => db.clone(),
            None => {
                log::warn!("No database manager set for audit page");
                return;
            }
        };

        let selected = imp.filter_dropdown.selected();
        let filter_endpoint = if selected == 0 {
            None
        } else {
            // Get endpoint name from the dropdown model
            let model = imp.filter_dropdown.model();
            model.and_then(|m| {
                m.item(selected)
                    .and_then(|item| item.downcast::<gtk::StringObject>().ok())
                    .map(|so| so.string().to_string())
            })
        };

        let mut conn = match db.get_connection() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to get DB connection: {}", e);
                return;
            }
        };

        let entries = if let Some(ref ep) = filter_endpoint {
            crate::db::download_entries::get_entries_by_endpoint(&mut conn, ep)
        } else {
            crate::db::download_entries::get_all_entries(&mut conn)
        };

        match entries {
            Ok(entries) => {
                let store = gtk::gio::ListStore::new::<AuditEntryObject>();
                for entry in entries {
                    store.append(&AuditEntryObject::new(entry));
                }
                let selection_model = gtk::NoSelection::new(Some(store));
                imp.audit_view.set_model(Some(&selection_model));
                log::info!(
                    "Loaded {} audit entries (filter: {})",
                    selection_model.n_items(),
                    filter_endpoint.as_deref().unwrap_or("all")
                );
            }
            Err(e) => {
                log::error!("Failed to load audit entries: {}", e);
            }
        }
    }

    /// Helper to add a text column to the ColumnView
    fn add_text_column<F>(
        &self,
        view: &ColumnView,
        title: &str,
        fixed_width: i32,
        expand: bool,
        extractor: F,
    ) where
        F: Fn(&DownloadEntry) -> String + 'static + Clone,
    {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<ListItem>().expect("Expected ListItem");
            let label = Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            item.set_child(Some(&label));
        });

        let extractor_clone = extractor.clone();
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().expect("Expected ListItem");
            if let Some(obj) = item.item().and_downcast::<AuditEntryObject>() {
                let label = item
                    .child()
                    .and_downcast::<Label>()
                    .expect("Expected Label");
                let data = obj.imp().data.borrow();
                if let Some(ref entry) = *data {
                    let text = extractor_clone(entry);
                    label.set_text(&text);

                    // Colour the status column
                    if entry.status == "error" {
                        label.add_css_class("error");
                    } else if entry.status == "success" {
                        label.add_css_class("success");
                    }
                }
            }
        });

        let column = ColumnViewColumn::new(Some(title), Some(factory));
        column.set_fixed_width(fixed_width);
        if expand {
            column.set_expand(true);
        }
        view.append_column(&column);
    }

    /// Set up the action column with retry buttons for failed entries
    fn setup_action_column(&self, view: &ColumnView) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<ListItem>().expect("Expected ListItem");
            let button = gtk::Button::with_label("Retry");
            button.add_css_class("suggested-action");
            button.set_visible(false);
            item.set_child(Some(&button));
        });

        let page_weak = self.downgrade();
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().expect("Expected ListItem");
            if let Some(obj) = item.item().and_downcast::<AuditEntryObject>() {
                let button = item
                    .child()
                    .and_downcast::<gtk::Button>()
                    .expect("Expected Button");
                let data = obj.imp().data.borrow();
                if let Some(ref entry) = *data {
                    let is_error = entry.status == "error";
                    button.set_visible(is_error);

                    if is_error {
                        let endpoint = entry.endpoint.clone();
                        let version = entry.version.clone();
                        let user_id = entry.user_id;
                        let page_weak_inner = page_weak.clone();

                        // Disconnect any previous handler by replacing with a new one
                        button.connect_clicked(move |_btn| {
                            if let Some(page) = page_weak_inner.upgrade() {
                                log::info!(
                                    "Retry requested for endpoint: {} v{}",
                                    endpoint,
                                    version
                                );
                                crate::developer::controller::AuditController::retry_entry(
                                    &page, &endpoint, &version, user_id,
                                );
                            }
                        });
                    }
                }
            }
        });

        let column = ColumnViewColumn::new(Some("Action"), Some(factory));
        column.set_fixed_width(80);
        view.append_column(&column);
    }
}
