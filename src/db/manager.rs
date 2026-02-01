use crate::chpp::error::Error;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct DbManager {
    database_url: String,
}

impl DbManager {
    pub fn new() -> Self {
        let db_path = Self::get_db_path();
        let database_url = db_path.to_string_lossy().to_string();
        Self { database_url }
    }

    fn get_db_path() -> PathBuf {
        let home_dir = env::var("HOME").expect("HOME environment variable not set");
        let config_dir = Path::new(&home_dir).join(".hoctane");

        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("Failed to create config directory: {}", e);
        }

        config_dir.join("hoctane.db")
    }

    pub fn establish_connection(&self) -> Result<SqliteConnection, Error> {
        let mut conn = SqliteConnection::establish(&self.database_url)
            .map_err(|e| Error::Io(format!("Error connecting to {}: {}", self.database_url, e)))?;

        self.run_migrations(&mut conn)?;

        Ok(conn)
    }

    fn run_migrations(&self, conn: &mut SqliteConnection) -> Result<(), Error> {
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| Error::Io(format!("Migration failed: {}", e)))?;
        Ok(())
    }
}
