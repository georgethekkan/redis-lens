use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::LensClient;

pub trait HashCommands {
    fn hlen(&self, key: &str) -> Result<i64>;
    fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>>;
    fn hscan(
        &self,
        key: &str,
        cursor: &str,
        count: usize,
    ) -> Result<(String, Vec<(String, String)>)>;
    fn hset(&self, key: &str, field: &str, value: &str) -> Result<()>;
    fn hdel(&self, key: &str, field: &str) -> Result<()>;
}

impl HashCommands for LensClient {
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

    fn hset(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con.hset(key, field, value).context("Failed to hset")?;
        Ok(())
    }

    fn hdel(&self, key: &str, field: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con.hdel(key, field).context("Failed to hdel")?;
        Ok(())
    }
}
