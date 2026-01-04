use color_eyre::eyre::{Context, Result};
use r2d2::{Pool, PooledConnection};
use redis::{Client, Commands};

pub trait RedisClient {
    fn url(&self) -> String;
    fn get(&self, key: &str) -> Result<String>;
    fn scan(&self) -> Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct RedisClientImpl {
    pub url: String,
    pub pool: Pool<Client>,
}

impl RedisClientImpl {
    pub fn new(url: String, db: u8) -> Result<RedisClientImpl> {
        println!("Connecting to Redis at {} using DB {}", url, db);

        let url = format!("redis://{}/{}", url, db);

        let client = Client::open(url.clone()).context("Failed to connect to Redis")?;

        let pool = r2d2::Pool::builder().build(client)?;

        Ok(RedisClientImpl { url, pool })

        /*let mut con = client
            .get_connection()
            .context("Failed to get Redis connection")?;

        //let keys = con.scan::<String>("*").context("Failed to get keys from Redis")?;

        Ok(())*/
    }

    fn get_connection(&self) -> Result<PooledConnection<Client>> {
        let conn = self
            .pool
            .get_timeout(std::time::Duration::from_secs(5))
            .context("Failed to get Redis connection")?;
        Ok(conn)
    }
}

impl RedisClient for RedisClientImpl {
    fn url(&self) -> String {
        self.url.clone()
    }

    fn get(&self, key: &str) -> Result<String> {
        let mut con = self.get_connection()?;
        let value: String = con.get(key).context("Failed to get key from Redis")?;
        Ok(value)
    }

    fn scan(&self) -> Result<Vec<String>> {
        let mut con = self.get_connection()?;
        let keys = con
            .scan::<String>()
            .context("Failed to get keys from Redis")?
            .map(|key| key.unwrap())
            .collect::<Vec<String>>();
        Ok(keys)
    }
}

#[derive(Debug, Clone)]
pub struct RedisClientMock {
    url: String,
}

impl RedisClientMock {
    pub fn new(url: String) -> RedisClientMock {
        RedisClientMock { url }
    }
}

impl RedisClient for RedisClientMock {
    fn url(&self) -> String {
        self.url.clone()
    }
    fn get(&self, key: &str) -> Result<String> {
        Ok(format!("value for {}", key))
    }

    fn scan(&self) -> Result<Vec<String>> {
        let keys = (1..20).map(|k| format!("key {}", k)).collect::<Vec<_>>();
        Ok(keys)
    }
}

impl RedisClient for Box<dyn RedisClient> {
    fn url(&self) -> String {
        (**self).url()
    }

    fn get(&self, key: &str) -> Result<String> {
        (**self).get(key)
    }

    fn scan(&self) -> Result<Vec<String>> {
        (**self).scan()
    }
}
