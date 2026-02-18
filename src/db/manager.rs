/* manager.rs
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

use crate::chpp::error::Error;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// Inspired by Shortwave
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct DbManager {
    pool: SqlitePool,
}

#[derive(Debug)]
struct ConnectionOptions;

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;
        conn.batch_execute(
            "
            PRAGMA busy_timeout = 5000;
            PRAGMA foreign_keys = ON;
            ",
        )
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

impl DbManager {
    pub fn new() -> Self {
        let pool = DB_POOL.get_or_init(|| {
            let db_path = Self::get_db_path();
            let database_url = db_path.to_string_lossy().to_string();
            let manager = ConnectionManager::<SqliteConnection>::new(database_url);
            let pool = r2d2::Pool::builder()
                .connection_customizer(Box::new(ConnectionOptions))
                .build(manager)
                .expect("Failed to create pool.");

            // Enable WAL mode (persistent setting, so only needed once)
            if let Ok(mut conn) = pool.get() {
                use diesel::connection::SimpleConnection;
                if let Err(e) = conn.batch_execute("PRAGMA journal_mode = WAL;") {
                    eprintln!("Failed to enable WAL mode: {}", e);
                }
            }

            // Run migrations on first initialization
            let db_manager = Self { pool: pool.clone() };
            db_manager
                .run_migrations()
                .expect("Failed to run migrations on startup");

            pool
        });

        Self { pool: pool.clone() }
    }

    // Constructor for testing with in-memory DB or custom path
    #[allow(dead_code)]
    pub fn from_url(database_url: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .connection_customizer(Box::new(ConnectionOptions))
            .build(manager)
            .expect("Failed to create pool.");
        Self { pool }
    }

    fn get_db_path() -> PathBuf {
        let home_dir = env::var("HOME").expect("HOME environment variable not set");
        let config_dir = Path::new(&home_dir).join(".nutmeg");

        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("Failed to create config directory: {}", e);
        }

        config_dir.join("nutmeg.db")
    }

    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, Error> {
        self.pool
            .get()
            .map_err(|e| Error::Io(format!("Failed to get connection from pool: {}", e)))
    }

    pub fn run_migrations(&self) -> Result<(), Error> {
        let mut conn = self.get_connection()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| Error::Io(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    pub fn has_users(&self) -> Result<bool, Error> {
        use crate::db::schema::users::dsl::*;
        let mut conn = self.get_connection()?;
        let count = users
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| Error::Io(format!("Failed to count users: {}", e)))?;
        Ok(count > 0)
    }

    /// Clear all data from the database (useful for debugging/reset)
    /// This deletes all rows from all tables but preserves the schema
    pub fn clear_all_data(&self) -> Result<(), Error> {
        use crate::db::schema::*;
        use diesel::prelude::*;

        let mut conn = self.get_connection()?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // Delete from tables with foreign key constraints first (children before parents)
            diesel::delete(avatars::table).execute(conn)?;
            diesel::delete(players::table).execute(conn)?;
            diesel::delete(league_unit_teams::table).execute(conn)?;
            diesel::delete(league_units::table).execute(conn)?;
            diesel::delete(matches::table).execute(conn)?;
            diesel::delete(teams::table).execute(conn)?;
            diesel::delete(users::table).execute(conn)?;
            diesel::delete(cups::table).execute(conn)?;
            diesel::delete(regions::table).execute(conn)?;
            diesel::delete(countries::table).execute(conn)?;
            diesel::delete(currencies::table).execute(conn)?;
            diesel::delete(downloads::table).execute(conn)?;

            Ok(())
        })
        .map_err(|e| Error::Io(format!("Failed to clear database: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_manager_pool() {
        // Use in-memory DB for testing
        let manager = DbManager::from_url(":memory:");

        // Run migrations
        manager.run_migrations().expect("Migrations failed");

        // Get a connection
        let _conn = manager.get_connection().expect("Failed to get connection");

        // Get another connection (should work as it's a pool)
        let _conn2 = manager
            .get_connection()
            .expect("Failed to get second connection");
    }
}
