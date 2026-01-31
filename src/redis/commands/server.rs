use crate::redis::RedisClient;
use color_eyre::eyre::{Context, Result};

pub trait ServerCommands {
    fn info(&self) -> Result<String>;
}

impl ServerCommands for RedisClient {
    fn info(&self) -> Result<String> {
        let mut con = self.get_connection()?;
        let info: String = redis::cmd("INFO")
            .query(&mut con)
            .context("Failed to get INFO from Redis")?;
        Ok(info)
    }
}
