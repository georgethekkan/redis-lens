use color_eyre::eyre::{Context, Result};
use r2d2::{Pool, PooledConnection};
use redis::{Client, Commands, Connection};

use crate::args;

pub struct RedisConnectionManager {
    client: Client,
}

impl r2d2::ManageConnection for RedisConnectionManager {
    type Connection = Connection;
    type Error = redis::RedisError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.client.get_connection()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        redis::cmd("PING").query(conn)
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        redis::cmd("PING").query::<String>(conn).is_err()
    }
}

pub struct RedisClient {
    pub url: String,
    pub pool: Pool<RedisConnectionManager>,
}

impl RedisClient {
    pub fn new(cfg: &args::RedisConfig) -> Result<RedisClient> {
        println!("Connecting to Redis at {} using DB {}", cfg.url, cfg.db);

        let url = build_redis_url(cfg);

        let client = Client::open(url.clone()).context("Failed to connect to Redis")?;

        let manager = RedisConnectionManager { client };

        let pool = r2d2::Pool::builder().build(manager)?;

        Ok(RedisClient { url, pool })

        /*let mut con = client
            .get_connection()
            .context("Failed to get Redis connection")?;

        //let keys = con.scan::<String>("*").context("Failed to get keys from Redis")?;

        Ok(())*/
    }

    fn get_connection(&self) -> Result<PooledConnection<RedisConnectionManager>> {
        let conn = self
            .pool
            .get_timeout(std::time::Duration::from_secs(5))
            .context("Failed to get Redis connection")?;
        Ok(conn)
    }
}

fn build_redis_url(cfg: &args::RedisConfig) -> String {
    if let Some(username) = &cfg.username {
        if let Some(password) = &cfg.password {
            format!("redis://{}:{}@{}/{}", username, password, cfg.url, cfg.db)
        } else {
            format!("redis://{}@{}/{}", username, cfg.url, cfg.db)
        }
    } else if let Some(password) = &cfg.password {
        format!("redis://:{}@{}/{}", password, cfg.url, cfg.db)
    } else {
        format!("redis://{}/{}", cfg.url, cfg.db)
    }
}

impl RedisClient {
    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn get(&self, key: &str) -> Result<String> {
        let mut con = self.get_connection()?;
        let value: String = con.get(key).context("Failed to get key from Redis")?;
        Ok(value)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: () = con
            .set(key.to_string(), value.to_string())
            .context("Failed to set key in Redis")?;
        Ok(())
    }

    pub fn scan(&self) -> Result<Vec<String>> {
        let mut con = self.get_connection()?;
        let keys: Vec<String> = con
            .scan::<Vec<u8>>()
            .context("Failed to get keys from Redis")?
            .map(|key| key.unwrap())
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .collect();
        Ok(keys)
    }

    pub fn scan_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        let mut con = self.get_connection()?;
        let keys: Vec<String> = con
            .scan_match::<&str, Vec<u8>>(pattern)
            .context("Failed to get keys from Redis")?
            .map(|key| key.unwrap())
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .collect();
        Ok(keys)
    }

    pub fn del(&self, key: &str) -> Result<()> {
        let mut con = self.get_connection()?;
        let _: i64 = con.del(key).context("Failed to delete key from Redis")?;
        Ok(())
    }

    pub fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let mut con = self.get_connection()?;
        let ttl: i64 = con.ttl(key).context("Failed to get TTL from Redis")?;
        match ttl {
            -2 => Ok(None),     // Key doesn't exist
            -1 => Ok(Some(-1)), // No TTL
            _ => Ok(Some(ttl)),
        }
    }

    pub fn key_type(&self, key: &str) -> Result<String> {
        let mut con = self.get_connection()?;
        let key_type: String = redis::cmd("TYPE")
            .arg(key)
            .query(&mut *con)
            .context("Failed to get key type from Redis")?;
        Ok(key_type)
    }
}
