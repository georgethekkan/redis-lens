use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::RedisClient;

pub trait ListCommands {
    fn llen(&self, key: &str) -> Result<i64>;
    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>>;
    fn rpush(&self, key: &str, value: &str) -> Result<()>;
}

impl ListCommands for RedisClient {
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
}
