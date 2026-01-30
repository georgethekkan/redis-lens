use color_eyre::eyre::{Context, Result};
use redis::Commands; // Import redis trait for low-level calls if needed, or just specific methods

use crate::redis::RedisClient;

pub trait KeyCommands {
    fn scan(&self, cursor: &str, pattern: &str, count: usize) -> Result<(String, Vec<String>)>;
    fn del(&self, key: &str) -> Result<()>;
    fn ttl(&self, key: &str) -> Result<Option<i64>>;
    fn key_type(&self, key: &str) -> Result<String>;
}

impl KeyCommands for RedisClient {
    fn scan(&self, cursor: &str, pattern: &str, count: usize) -> Result<(String, Vec<String>)> {
        let mut con = self.get_connection()?;
        let (next_cursor, keys): (String, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(count.to_string())
            .query(&mut con)
            .context("Failed to scan keys from Redis")?;
        Ok((next_cursor, keys))
    }

    fn del(&self, key: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: i64 = con.del(key).context("Failed to delete key from Redis")?;
        Ok(())
    }

    fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let mut con = self.get_connection()?;
        let ttl: i64 = con.ttl(key).context("Failed to get TTL from Redis")?;
        match ttl {
            -2 => Ok(None),     // Key doesn't exist
            -1 => Ok(Some(-1)), // No TTL
            _ => Ok(Some(ttl)),
        }
    }

    fn key_type(&self, key: &str) -> Result<String> {
        let mut con = self.get_connection()?;
        let key_type: String = redis::cmd("TYPE")
            .arg(key)
            .query(&mut *con)
            .context("Failed to get key type from Redis")?;
        Ok(key_type)
    }
}
