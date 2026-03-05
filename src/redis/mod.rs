use color_eyre::eyre::{Context, Result};
use r2d2::{Pool, PooledConnection};
use redis::{Client, Connection};
use tracing::info;

use crate::args::Config;

pub mod commands;
mod datatype;
pub use datatype::DataType;

pub mod mock;
pub use mock::MockClient;

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

/// Core trait defining operations that can be performed against a Redis-compatible store.
pub trait ClientOps:
    commands::KeysCommands
    + commands::StringCommands
    + commands::HashCommands
    + commands::ListCommands
    + commands::SetCommands
    + commands::SortedSetCommands
    + commands::ServerCommands
    + commands::PubSub
    + Send
    + Sync
{
    /// Returns the connection URL.
    fn url(&self) -> String;
    /// Switches the active database index.
    fn select_db(&mut self, db: u8) -> Result<()>;
}

/// A standard Redis client implementation using `r2d2` for connection pooling.
pub struct LensClient {
    pub url: String,
    pub config: Config,
    pub pool: Pool<RedisConnectionManager>,
}

impl ClientOps for LensClient {
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

impl LensClient {
    /// Creates a new `LensClient` from the provided configuration.
    ///
    /// This will attempt to establish a connection pool to the Redis server.
    pub fn new(cfg: &Config) -> Result<LensClient> {
        info!("Connecting to Redis at {} using DB {}", cfg.url, cfg.db);

        let url = build_redis_url(cfg);
        let client = Client::open(url.clone()).context("Failed to connect to Redis")?;

        let manager = RedisConnectionManager { client };
        let pool = r2d2::Pool::builder().build(manager)?;
        info!("Connected to Redis successfully");

        Ok(LensClient {
            url,
            config: cfg.clone(),
            pool,
        })
    }

    /// Acquires a connection from the pool with a default timeout of 5 seconds.
    pub fn get_connection(&self) -> Result<PooledConnection<RedisConnectionManager>> {
        self.pool
            .get_timeout(std::time::Duration::from_secs(5))
            .context("Failed to get Redis connection")
    }
}

fn build_redis_url(cfg: &Config) -> String {
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

pub struct ScanResponse<T> {
    pub next: String,
    pub keys: T,
}

pub type ScanResult<T> = Result<ScanResponse<T>>;

impl<T> ScanResponse<T> {
    pub fn new(next: String, keys: T) -> Self {
        Self { next, keys }
    }
}

mod test {
    use std::sync::{Arc, Mutex};

    use lazy_static::lazy_static;

    use crate::{
        args::Config,
        redis::{LensClient, commands::PubSub},
    };

    lazy_static! {
        static ref cfg: Config = Config {
            url: "localhost:6379".to_string(),
            db: 0,
            mock: false,
            username: None,
            password: None,
        };
        static ref CLIENT: Arc<Mutex<LensClient>> =
            Arc::new(Mutex::new(LensClient::new(&cfg).unwrap()));
    }

    #[test]
    fn test_pubsub() {
        let client = CLIENT.lock().unwrap();
        let res = client.scan_channels("", "", 10);
        assert_eq!(true, res.is_ok());
    }
}
