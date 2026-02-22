use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::LensClient;

pub trait StringCommands {
    fn get(&self, key: &str) -> Result<String>;
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn strlen(&self, key: &str) -> Result<i64>;
}

impl StringCommands for LensClient {
    fn get(&self, key: &str) -> Result<String> {
        let mut con = self.get_connection()?;
        let value: String = con.get(key).context("Failed to get key from Redis")?;
        Ok(value)
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con
            .set(key.to_string(), value.to_string())
            .context("Failed to set key in Redis")?;
        Ok(())
    }

    fn strlen(&self, key: &str) -> Result<i64> {
        let mut con = self.get_connection()?;
        let len: i64 = con.strlen(key).context("Failed to get string length")?;
        Ok(len)
    }
}
