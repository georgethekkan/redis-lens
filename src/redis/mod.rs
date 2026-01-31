use color_eyre::eyre::{Context, Result};
use r2d2::{Pool, PooledConnection};
use redis::{Client, Connection};
use tracing::info;

use crate::args::{self, RedisConfig};

pub mod commands;

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

pub trait RedisOps:
    commands::KeyCommands
    + commands::StringCommands
    + commands::HashCommands
    + commands::ListCommands
    + commands::SetCommands
    + commands::SortedSetCommands
    + commands::ServerCommands
    + Send
    + Sync
{
    fn url(&self) -> String;
    fn select_db(&mut self, db: u8) -> Result<()>;
}

pub struct RedisClient {
    pub url: String,
    pub config: RedisConfig,
    pub pool: Pool<RedisConnectionManager>,
}

impl RedisOps for RedisClient {
    fn url(&self) -> String {
        self.url.clone()
    }

    fn select_db(&mut self, db: u8) -> Result<()> {
        self.config.db = db;
        let url = build_redis_url(&self.config);
        let client = Client::open(url.clone()).context("Failed to connect to Redis")?;
        let manager = RedisConnectionManager { client };
        let pool = r2d2::Pool::builder().build(manager)?;

        self.url = url;
        self.pool = pool;
        Ok(())
    }
}

impl RedisClient {
    #[tracing::instrument(skip(cfg))]
    pub fn new(cfg: &RedisConfig) -> Result<RedisClient> {
        info!("Connecting to Redis at {} using DB {}", cfg.url, cfg.db);

        let url = build_redis_url(cfg);

        let client = Client::open(url.clone()).context("Failed to connect to Redis")?;

        let manager = RedisConnectionManager { client };

        let pool = r2d2::Pool::builder().build(manager)?;

        info!("Connected to Redis successfully");
        Ok(RedisClient {
            url,
            config: cfg.clone(),
            pool,
        })
    }

    pub fn get_connection(&self) -> Result<PooledConnection<RedisConnectionManager>> {
        self.pool
            .get_timeout(std::time::Duration::from_secs(5))
            .context("Failed to get Redis connection")
    }
}

fn build_redis_url(cfg: &RedisConfig) -> String {
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
