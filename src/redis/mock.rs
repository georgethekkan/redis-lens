use color_eyre::eyre::Result;

use super::RedisClient;

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

    fn set(&self, key: &str, value: &str) -> Result<()> {
        println!("Mock set key: {} to value: {}", key, value);
        Ok(())
    }

    fn scan(&self) -> Result<Vec<String>> {
        let keys = (1..20).map(|k| format!("key {}", k)).collect::<Vec<_>>();
        Ok(keys)
    }

    fn scan_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        let keys = (1..20)
            .map(|k| format!("key {}", k))
            .filter(|k| k.contains(pattern))
            .collect::<Vec<_>>();
        Ok(keys)
    }

    fn del(&self, key: &str) -> Result<()> {
        println!("Mock delete key: {}", key);
        Ok(())
    }

    fn ttl(&self, key: &str) -> Result<Option<i64>> {
        // Mock: return some TTL for even keys, no TTL for odd
        let num: i32 = key.split_whitespace().last().unwrap_or("0").parse().unwrap_or(0);
        if num % 2 == 0 {
            Ok(Some(3600)) // 1 hour
        } else {
            Ok(Some(-1)) // No TTL
        }
    }

    fn key_type(&self, key: &str) -> Result<String> {
        // Mock: alternate between string and hash
        let num: i32 = key.split_whitespace().last().unwrap_or("0").parse().unwrap_or(0);
        if num % 2 == 0 {
            Ok("string".to_string())
        } else {
            Ok("hash".to_string())
        }
    }
}