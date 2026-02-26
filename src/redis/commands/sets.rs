use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::{LensClient, ScanResponse, ScanResult};

pub trait SetCommands {
    fn scard(&self, key: &str) -> Result<i64>;
    fn smembers(&self, key: &str) -> Result<Vec<String>>;
    fn sscan(&self, key: &str, cursor: &str, count: usize) -> ScanResult<Vec<String>>;
    fn sadd(&self, key: &str, member: &str) -> Result<()>;
    fn srem(&self, key: &str, member: &str) -> Result<()>;
}

impl SetCommands for LensClient {
    fn scard(&self, key: &str) -> Result<i64> {
        let mut con = self.get_connection()?;
        let count: i64 = con.scard(key).context("Failed to get set cardinality")?;
        Ok(count)
    }

    fn smembers(&self, key: &str) -> Result<Vec<String>> {
        let mut con = self.get_connection()?;
        let members: Vec<String> = con.smembers(key).context("Failed to get set members")?;
        Ok(members)
    }

    fn sscan(&self, key: &str, cursor: &str, count: usize) -> ScanResult<Vec<String>> {
        let mut con = self.get_connection()?;
        let res: (String, Vec<String>) = redis::cmd("SSCAN")
            .arg(key)
            .arg(cursor)
            .arg("COUNT")
            .arg(count)
            .query(&mut con)
            .context("Failed to sscan")?;
        Ok(ScanResponse::new(res.0, res.1))
    }

    fn sadd(&self, key: &str, member: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con.sadd(key, member).context("Failed to sadd")?;
        Ok(())
    }

    fn srem(&self, key: &str, member: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con.srem(key, member).context("Failed to srem")?;
        Ok(())
    }
}
