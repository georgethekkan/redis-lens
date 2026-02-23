use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::LensClient;

pub trait ListCommands {
    fn llen(&self, key: &str) -> Result<i64>;
    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>>;
    fn rpush(&self, key: &str, value: &str) -> Result<()>;
    fn lrem(&self, key: &str, count: i64, value: &str) -> Result<()>;
    fn lset(&self, key: &str, index: i64, value: &str) -> Result<()>;
}

impl ListCommands for LensClient {
    fn llen(&self, key: &str) -> Result<i64> {
        let mut con = self.get_connection()?;
        let len: i64 = con.llen(key).context("Failed to get list length")?;
        Ok(len)
    }

    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>> {
        let mut con = self.get_connection()?;
        let items: Vec<String> = con
            .lrange(key, start as isize, stop as isize)
            .context("Failed to get list range")?;
        Ok(items)
    }

    fn rpush(&self, key: &str, value: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con.rpush(key, value).context("Failed to rpush")?;
        Ok(())
    }

    fn lrem(&self, key: &str, count: i64, value: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con
            .lrem(key, count as isize, value)
            .context("Failed to lrem")?;
        Ok(())
    }

    fn lset(&self, key: &str, index: i64, value: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con
            .lset(key, index as isize, value)
            .context("Failed to lset")?;
        Ok(())
    }
}
