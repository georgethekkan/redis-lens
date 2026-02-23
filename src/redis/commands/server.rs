use crate::redis::LensClient;
use color_eyre::eyre::{Context, Result};

pub trait ServerCommands {
    fn info(&self, section: Option<&str>) -> Result<String>;
    fn dbsize(&self) -> Result<i64>;
}

impl ServerCommands for LensClient {
    fn info(&self, section: Option<&str>) -> Result<String> {
        let mut con = self.get_connection()?;
        let mut cmd = redis::cmd("INFO");
        if let Some(s) = section {
            cmd.arg(s);
        }
        let info: String = cmd
            .query(&mut con)
            .context("Failed to get INFO from Redis")?;
        Ok(info)
    }

    fn dbsize(&self) -> Result<i64> {
        let mut con = self.get_connection()?;
        let count: i64 = redis::cmd("DBSIZE")
            .query(&mut con)
            .context("Failed to get DBSIZE from Redis")?;
        Ok(count)
    }
}
