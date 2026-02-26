use color_eyre::eyre::{Context, Result};
use redis::Commands; // Import redis trait for low-level calls if needed, or just specific methods

use crate::redis::{LensClient, ScanResponse, ScanResult, datatype::DataType};

pub trait KeysCommands {
    fn scan(&self, cursor: &str, pattern: &str, count: usize) -> ScanResult<Vec<String>>;
    fn del(&self, key: &str) -> Result<i32>;
    fn ttl(&self, key: &str) -> Result<Option<i64>>;
    fn data_type(&self, key: &str) -> Result<DataType>;
    fn delete_all(&self, pattern: &str) -> Result<usize>;
}

impl KeysCommands for LensClient {
    fn scan(&self, cursor: &str, pattern: &str, count: usize) -> ScanResult<Vec<String>> {
        let mut con = self.get_connection()?;
        let (next, keys): (String, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(count.to_string())
            .query(&mut con)
            .context("Failed to scan keys from Redis")?;
        Ok(ScanResponse::new(next, keys))
    }

    fn del(&self, key: &str) -> Result<i32> {
        let mut con = self.get_connection()?;
        let deleted_count: i32 = con.del(key).context("Failed to delete key from Redis")?;
        Ok(deleted_count)
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

    fn data_type(&self, key: &str) -> Result<DataType> {
        let mut con = self.get_connection()?;
        let data_type: String = redis::cmd("TYPE")
            .arg(key)
            .query(&mut *con)
            .context("Failed to get key type from Redis")?;
        Ok(DataType::new(&data_type))
    }

    fn delete_all(&self, pattern: &str) -> Result<usize> {
        let mut cursor = "0".to_string();
        let mut total_deleted = 0;
        loop {
            let resp = self.scan(&cursor, pattern, 100)?;
            if !resp.keys.is_empty() {
                let mut con = self.get_connection()?;
                let _: () = con
                    .del(&resp.keys)
                    .context("Failed to delete batch of keys")?;
                total_deleted += resp.keys.len();
            }
            cursor = resp.next;
            if cursor == "0" {
                break;
            }
        }
        Ok(total_deleted)
    }
}

impl LensClient {
    pub fn dbsize(&self) -> Result<u64> {
        let mut con = self.get_connection()?;
        let size: u64 = redis::cmd("DBSIZE")
            .query(&mut con)
            .context("Failed to get DBSIZE from Redis")?;
        Ok(size)
    }
}
