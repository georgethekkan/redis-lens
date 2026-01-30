use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::RedisClient;

pub trait HashCommands {
    fn hlen(&self, key: &str) -> Result<i64>;
    fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>>;
    fn hscan(
        &self,
        key: &str,
        cursor: &str,
        count: usize,
    ) -> Result<(String, Vec<(String, String)>)>;
}

impl HashCommands for RedisClient {
    fn hlen(&self, key: &str) -> Result<i64> {
        let mut con = self.get_connection()?;
        let len: i64 = con.hlen(key).context("Failed to get hash length")?;
        Ok(len)
    }

    fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>> {
        let mut con = self.get_connection()?;
        let data: Vec<(String, String)> = redis::cmd("HGETALL")
            .arg(key)
            .query(&mut con)
            .context("Failed to get hash data")?;
        Ok(data)
    }

    fn hscan(
        &self,
        key: &str,
        cursor: &str,
        count: usize,
    ) -> Result<(String, Vec<(String, String)>)> {
        let mut con = self.get_connection()?;
        let res: (String, Vec<(String, String)>) = redis::cmd("HSCAN")
            .arg(key)
            .arg(cursor)
            .arg("COUNT")
            .arg(count)
            .query(&mut con)
            .context("Failed to hscan")?;
        Ok(res)
    }
}
