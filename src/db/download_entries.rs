/*
 * nutmeg - Hattrick Organizer written in Rust
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

use crate::db::schema::download_entries;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = download_entries)]
pub struct DownloadEntry {
    pub id: i32,
    pub download_id: i32,
    pub endpoint: String,
    pub version: String,
    pub user_id: Option<i32>,
    pub status: String,
    pub fetched_date: String,
    pub error_message: Option<String>,
    pub retry_count: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = download_entries)]
pub struct NewDownloadEntry {
    pub download_id: i32,
    pub endpoint: String,
    pub version: String,
    pub user_id: Option<i32>,
    pub status: String,
    pub fetched_date: String,
    pub error_message: Option<String>,
    pub retry_count: i32,
}

/// Create a new download entry
pub fn create_download_entry(
    conn: &mut SqliteConnection,
    entry: NewDownloadEntry,
) -> QueryResult<i32> {
    diesel::insert_into(download_entries::table)
        .values(&entry)
        .execute(conn)?;

    // Get the last inserted ID
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
        "last_insert_rowid()",
    ))
    .get_result(conn)
}

/// Update the status of a download entry (for retry tracking)
pub fn update_entry_status(
    conn: &mut SqliteConnection,
    entry_id: i32,
    new_status: &str,
    error_msg: Option<String>,
    increment_retry: bool,
) -> QueryResult<usize> {
    use crate::db::schema::download_entries::dsl::*;

    if increment_retry {
        diesel::update(download_entries.find(entry_id))
            .set((
                status.eq(new_status),
                error_message.eq(error_msg),
                retry_count.eq(retry_count + 1),
            ))
            .execute(conn)
    } else {
        diesel::update(download_entries.find(entry_id))
            .set((status.eq(new_status), error_message.eq(error_msg)))
            .execute(conn)
    }
}

/// Get all entries for a specific download
pub fn get_entries_for_download(
    conn: &mut SqliteConnection,
    dl_id: i32,
) -> QueryResult<Vec<DownloadEntry>> {
    use crate::db::schema::download_entries::dsl::*;

    download_entries
        .filter(download_id.eq(dl_id))
        .order(id.asc())
        .load::<DownloadEntry>(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::manager::DbManager;
    use crate::db::schema::downloads;

    #[derive(Insertable)]
    #[diesel(table_name = downloads)]
    struct NewDownload {
        timestamp: String,
        status: String,
    }

    #[test]
    fn test_create_and_retrieve_entry() {
        let db = DbManager::new_in_memory().expect("Failed to create in-memory DB");
        let mut conn = db.get_connection().expect("Failed to get connection");

        // Create a download first
        let download: i32 = diesel::insert_into(downloads::table)
            .values(NewDownload {
                timestamp: "2026-02-04T18:45:00Z".to_string(),
                status: "in_progress".to_string(),
            })
            .returning(downloads::id)
            .get_result(&mut conn)
            .expect("Failed to create download");

        // Create an entry
        let entry = NewDownloadEntry {
            download_id: download,
            endpoint: "worlddetails".to_string(),
            version: "2.4".to_string(),
            user_id: Some(123456),
            status: "success".to_string(),
            fetched_date: "2026-02-04T18:45:00Z".to_string(),
            error_message: None,
            retry_count: 0,
        };

        let entry_id = create_download_entry(&mut conn, entry).expect("Failed to create entry");
        assert!(entry_id > 0);

        // Retrieve entries
        let entries = get_entries_for_download(&mut conn, download).expect("Failed to get entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].endpoint, "worlddetails");
        assert_eq!(entries[0].status, "success");
        assert_eq!(entries[0].retry_count, 0);
    }

    #[test]
    fn test_update_entry_status() {
        let db = DbManager::new_in_memory().expect("Failed to create in-memory DB");
        let mut conn = db.get_connection().expect("Failed to get connection");

        // Create download and entry
        let download: i32 = diesel::insert_into(downloads::table)
            .values(NewDownload {
                timestamp: "2026-02-04T18:45:00Z".to_string(),
                status: "in_progress".to_string(),
            })
            .returning(downloads::id)
            .get_result(&mut conn)
            .expect("Failed to create download");

        let entry = NewDownloadEntry {
            download_id: download,
            endpoint: "players".to_string(),
            version: "2.4".to_string(),
            user_id: None,
            status: "retrying".to_string(),
            fetched_date: "2026-02-04T18:45:00Z".to_string(),
            error_message: Some("Connection timeout".to_string()),
            retry_count: 1,
        };

        let entry_id = create_download_entry(&mut conn, entry).expect("Failed to create entry");

        // Update to success
        let rows_updated = update_entry_status(&mut conn, entry_id, "success", None, false)
            .expect("Failed to update entry");

        assert_eq!(rows_updated, 1);

        // Verify the update
        let entries = get_entries_for_download(&mut conn, download).expect("Failed to get entries");
        assert_eq!(entries[0].status, "success");
        assert_eq!(entries[0].retry_count, 1); // Should not increment
    }
}
