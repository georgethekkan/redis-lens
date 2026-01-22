use color_eyre::eyre::Result;

pub mod mock;
pub mod real;

pub use mock::RedisClientMock;
pub use real::{RedisClientImpl, RedisConnectionManager};

pub trait RedisClient {
    fn url(&self) -> String;
    fn get(&self, key: &str) -> Result<String>;
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn scan(&self) -> Result<Vec<String>>;
    fn scan_pattern(&self, pattern: &str) -> Result<Vec<String>>;
    fn del(&self, key: &str) -> Result<()>;
    fn ttl(&self, key: &str) -> Result<Option<i64>>;
    fn key_type(&self, key: &str) -> Result<String>;
}

impl RedisClient for Box<dyn RedisClient> {
    fn url(&self) -> String {
        (**self).url()
    }

    fn get(&self, key: &str) -> Result<String> {
        (**self).get(key)
    }
    fn set(&self, key: &str, value: &str) -> Result<()> {
        (**self).set(key, value)
    }

    fn scan(&self) -> Result<Vec<String>> {
        (**self).scan()
    }
    fn scan_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        (**self).scan_pattern(pattern)
    }
    fn del(&self, key: &str) -> Result<()> {
        (**self).del(key)
    }

    fn ttl(&self, key: &str) -> Result<Option<i64>> {
        (**self).ttl(key)
    }

    fn key_type(&self, key: &str) -> Result<String> {
        (**self).key_type(key)
    }
}