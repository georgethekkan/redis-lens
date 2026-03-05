use color_eyre::eyre::Context;

use crate::redis::{LensClient, ScanResponse, ScanResult};

pub trait PubSub {
    fn scan_channels(&self, cursor: &str, pattern: &str, count: usize) -> ScanResult<Vec<String>>;
}

impl PubSub for LensClient {
    fn scan_channels(&self, cursor: &str, pattern: &str, count: usize) -> ScanResult<Vec<String>> {
        let mut con = self.get_connection()?;
        let (next, keys): (String, Vec<String>) = redis::cmd("PUBSUB CHANNELS")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(count.to_string())
            .query(&mut con)
            .context("Failed to scan keys from Redis")?;
        Ok(ScanResponse::new(next, keys))
    }
}
