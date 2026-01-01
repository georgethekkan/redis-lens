use color_eyre::eyre::{Context, Result};
use r2d2::{Pool, PooledConnection};
use redis::{Client, Commands};

#[derive(Debug, Clone)]
pub struct RedisClient {
    pub url: String,
    pub pool: Pool<Client>,
}

impl RedisClient {
    pub fn new(url: String, db: u8) -> Result<RedisClient> {
        println!("Connecting to Redis at {} using DB {}", url, db);

        let url = format!("redis://{}/{}", url, db);

        let client = Client::open(url.clone()).context("Failed to connect to Redis")?;

        let pool = r2d2::Pool::builder().build(client)?;

        Ok(RedisClient { url, pool })

        /*let mut con = client
            .get_connection()
            .context("Failed to get Redis connection")?;

        //let keys = con.scan::<String>("*").context("Failed to get keys from Redis")?;

        Ok(())*/
    }

    pub fn get_connection(&self) -> Result<PooledConnection<Client>> {
        let conn = self
            .pool
            .get_timeout(std::time::Duration::from_secs(5))
            .context("Failed to get Redis connection")?;
        Ok(conn)
    }
}
