use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use tracing::{debug, info};

pub mod cache;
pub mod schema;

pub use cache::Cache;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path).context("Failed to open database")?;

        info!("Database opened at {}", path.display());

        let mut db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&mut self) -> Result<()> {
        debug!("Initializing database schema");
        schema::create_tables(&self.conn)?;
        info!("Database schema initialized");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn cache(&self) -> Cache<'_> {
        Cache::new(&self.conn)
    }

    pub fn save_device(&self, id: &str, name: &str, model: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO devices (id, name, model, last_seen)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![id, name, model],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_devices(&self) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, name, model FROM devices ORDER BY name")?;

        let devices = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(devices)
    }

    #[allow(dead_code)]
    pub fn save_state(&self, device_id: &str, state_json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO device_state (device_id, state_json, updated_at)
             VALUES (?1, ?2, datetime('now'))",
            params![device_id, state_json],
        )?;
        Ok(())
    }
}
