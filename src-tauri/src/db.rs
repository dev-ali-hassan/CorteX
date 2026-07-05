use std::{error::Error, fs, sync::Mutex};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use crate::models::AppSettings;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn Error>> {
        let app_dir = app.path().app_data_dir()?;
        fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("cortex.sqlite3");
        let conn = Connection::open(db_path)?;
        let database = Self {
            conn: Mutex::new(conn),
        };
        database.init()?;
        Ok(database)
    }

    fn init(&self) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().map_err(|_| "database lock poisoned")?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            DROP TABLE IF EXISTS rewrite_history;
            "#,
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        let stored: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'settings'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;

        match stored {
            Some(value) => serde_json::from_str(&value).map_err(|error| error.to_string()),
            None => Ok(AppSettings::default()),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<AppSettings, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        let payload = serde_json::to_string(settings).map_err(|error| error.to_string())?;
        conn.execute(
            r#"
            INSERT INTO settings (key, value, updated_at)
            VALUES ('settings', ?1, ?2)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
            params![payload, Utc::now().to_rfc3339()],
        )
        .map_err(|error| error.to_string())?;
        Ok(settings.clone())
    }

}
