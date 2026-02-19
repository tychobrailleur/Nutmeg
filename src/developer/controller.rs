/* controller.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::db::download_entries::{create_download, create_download_entry, NewDownloadEntry};
use crate::db::manager::DbManager;
use crate::developer::page::DeveloperAuditPage;
use chrono::Utc;
use gtk::glib;
use gtk::prelude::*;
use std::sync::Arc;

/// Controller for the Developer Audit page.
///
/// Handles retry logic for failed API calls, creating new insert-only
/// download records to track the retry attempt.
pub struct AuditController;

impl AuditController {
    /// Retry a failed API call by creating new download + entry records.
    ///
    /// This follows the insert-only pattern: a new download record is created,
    /// along with a new download_entry that logs the retry attempt.
    pub fn retry_entry(
        page: &DeveloperAuditPage,
        endpoint: &str,
        version: &str,
        user_id: Option<i32>,
    ) {
        let imp = page.imp();
        let db_ref = imp.db_manager.borrow();
        let db = match db_ref.as_ref() {
            Some(db) => db.clone(),
            None => {
                log::error!("No database manager available for retry");
                return;
            }
        };

        let endpoint = endpoint.to_string();
        let version = version.to_string();
        let page_weak = page.downgrade();

        // Spawn the retry operation on the Tokio runtime
        glib::MainContext::default().spawn_local(async move {
            let result = Self::do_retry(db.clone(), &endpoint, &version, user_id).await;

            match result {
                Ok(entry_id) => {
                    log::info!(
                        "Retry recorded as entry {} for endpoint {}",
                        entry_id,
                        endpoint
                    );
                }
                Err(e) => {
                    log::error!("Retry failed for endpoint {}: {}", endpoint, e);
                }
            }

            // Refresh the audit view
            if let Some(page) = page_weak.upgrade() {
                page.load_entries();
            }
        });
    }

    /// Perform the retry: create download record + entry with status "retrying",
    /// then attempt the actual API call.
    ///
    /// For now, this creates the retry record. The actual API call dispatch
    /// can be extended once per-endpoint retry handlers are implemented.
    async fn do_retry(
        db: Arc<DbManager>,
        endpoint: &str,
        version: &str,
        user_id: Option<i32>,
    ) -> Result<i32, String> {
        let endpoint = endpoint.to_string();
        let version = version.to_string();
        let timestamp = Utc::now().to_rfc3339();

        let db_clone = db.clone();
        let ep = endpoint.clone();
        let ver = version.clone();
        let ts = timestamp.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = db_clone
                .get_connection()
                .map_err(|e| format!("DB connection error: {}", e))?;

            // Create a new download record (insert-only)
            let download_id = create_download(&mut conn, &ts, "retry")
                .map_err(|e| format!("Failed to create download: {}", e))?;

            // Create a new download entry for the retry
            let entry = NewDownloadEntry {
                download_id,
                endpoint: ep,
                version: ver,
                user_id,
                status: "retrying".to_string(),
                fetched_date: ts,
                error_message: None,
                retry_count: 0,
            };

            let entry_id = create_download_entry(&mut conn, entry)
                .map_err(|e| format!("Failed to create entry: {}", e))?;

            Ok(entry_id)
        })
        .await
        .map_err(|e| format!("Join error: {}", e))?
    }
}

use gtk::subclass::prelude::ObjectSubclassIsExt;
