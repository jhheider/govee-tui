use anyhow::Result;
use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<()> {
    // Devices table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            model TEXT NOT NULL,
            capabilities TEXT,
            last_seen DATETIME NOT NULL
        )",
        [],
    )?;

    // Device state table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS device_state (
            device_id TEXT PRIMARY KEY,
            state_json TEXT NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY(device_id) REFERENCES devices(id)
        )",
        [],
    )?;

    // Command history table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS command_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            command TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            success BOOLEAN NOT NULL,
            FOREIGN KEY(device_id) REFERENCES devices(id)
        )",
        [],
    )?;

    // Cache table for API responses
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cache (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            expires_at DATETIME NOT NULL
        )",
        [],
    )?;

    // Preferences table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS preferences (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Create indices for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_command_history_device
         ON command_history(device_id, timestamp DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cache_expires
         ON cache(expires_at)",
        [],
    )?;

    Ok(())
}
