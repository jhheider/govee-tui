use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};

#[allow(dead_code)]
pub struct Cache<'a> {
    conn: &'a Connection,
}

#[allow(dead_code)]
impl<'a> Cache<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let now = Utc::now().to_rfc3339();

        match self.conn.query_row(
            "SELECT value FROM cache
             WHERE key = ?1 AND expires_at > ?2",
            params![key, now],
            |row| row.get(0),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set(&self, key: &str, value: &str, ttl_seconds: i64) -> Result<()> {
        let expires_at = Utc::now() + Duration::seconds(ttl_seconds);

        self.conn.execute(
            "INSERT OR REPLACE INTO cache (key, value, expires_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, expires_at.to_rfc3339()],
        )?;

        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM cache WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn cleanup_expired(&self) -> Result<usize> {
        let now = Utc::now().to_rfc3339();
        let deleted = self
            .conn
            .execute("DELETE FROM cache WHERE expires_at <= ?1", params![now])?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_cache_set_get() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();

        let cache = Cache::new(&conn);
        cache.set("test_key", "test_value", 300).unwrap();

        let value = cache.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[test]
    fn test_cache_expiry() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();

        let cache = Cache::new(&conn);
        cache.set("expired", "value", -1).unwrap(); // Already expired

        let value = cache.get("expired").unwrap();
        assert_eq!(value, None);
    }
}
