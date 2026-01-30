use color_eyre::eyre::{Context, Result};
use redis::Commands;

use crate::redis::RedisClient;

pub trait SortedSetCommands {
    fn zcard(&self, key: &str) -> Result<i64>;
    fn zrange_with_scores(&self, key: &str, start: i64, stop: i64) -> Result<Vec<(String, f64)>>;
    fn zscan(&self, key: &str, cursor: &str, count: usize) -> Result<(String, Vec<(String, f64)>)>;
    fn zadd(&self, key: &str, score: f64, member: &str) -> Result<()>;
}

impl SortedSetCommands for RedisClient {
    fn zcard(&self, key: &str) -> Result<i64> {
        let mut con = self.get_connection()?;
        let count: i64 = con
            .zcard(key)
            .context("Failed to get sorted set cardinality")?;
        Ok(count)
    }

    fn zrange_with_scores(&self, key: &str, start: i64, stop: i64) -> Result<Vec<(String, f64)>> {
        let mut con = self.get_connection()?;
        let items: Vec<(String, f64)> = redis::cmd("ZRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query(&mut con)
            .context("Failed to get sorted set range")?;
        Ok(items)
    }

    fn zscan(&self, key: &str, cursor: &str, count: usize) -> Result<(String, Vec<(String, f64)>)> {
        let mut con = self.get_connection()?;
        let res: (String, Vec<(String, f64)>) = redis::cmd("ZSCAN")
            .arg(key)
            .arg(cursor)
            .arg("COUNT")
            .arg(count)
            .query(&mut con)
            .context("Failed to zscan")?;
        Ok(res)
    }

    fn zadd(&self, key: &str, score: f64, member: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con.zadd(key, member, score).context("Failed to zadd")?;
        Ok(())
    }
}
