#![allow(unused)]
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use color_eyre::eyre::{Ok, Result, bail};

use crate::redis::{
    ClientOps, ScanResponse, ScanResult,
    commands::{
        HashCommands, KeysCommands, ListCommands, ServerCommands, SetCommands, SortedSetCommands,
        StringCommands,
    },
    datatype::DataType,
};

#[derive(Debug, Clone)]
pub enum RedisValue {
    String(String),
    Hash(HashMap<String, String>),
    List(Vec<String>),
    Set(HashSet<String>),
    SortedSet(Vec<(String, f64)>),
}

#[derive(Default, Debug)]
pub struct MockClient {
    items: RwLock<HashMap<String, RedisValue>>,
}
impl MockClient {
    pub fn setup_keys(&self) -> Result<()> {
        // Pre-populate with some sample data for demo
        self.set("demo:string", "Hello Redis Lens!")?;
        self.hset("demo:hash", "version", "0.1.0")?;
        self.hset("demo:hash", "author", "George")?;
        self.rpush("demo:list", "item 1")?;
        self.rpush("demo:list", "item 2")?;
        self.sadd("demo:set", "member A")?;
        self.sadd("demo:set", "member B")?;
        for i in 1..=500 {
            self.set(&format!("demo:strings:{}", i), &i.to_string())?;
        }
        Ok(())
    }
}

impl ClientOps for MockClient {
    fn url(&self) -> String {
        "mock".to_string()
    }

    fn select_db(&mut self, db: u8) -> Result<()> {
        Ok(())
    }
}

impl ServerCommands for MockClient {
    fn info(&self, section: Option<&str>) -> Result<String> {
        Ok("test".to_string())
    }

    fn dbsize(&self) -> Result<i64> {
        Ok(self.items.read().unwrap().len() as i64)
    }
}

impl StringCommands for MockClient {
    fn get(&self, key: &str) -> Result<String> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::String(v)) => Ok(v.clone()),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => bail!("(nil)"),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        items.insert(key.to_string(), RedisValue::String(value.to_string()));
        Ok(())
    }

    fn strlen(&self, key: &str) -> Result<i64> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::String(v)) => Ok(v.len() as i64),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(0),
        }
    }
}

impl KeysCommands for MockClient {
    fn scan(&self, _cursor: &str, pattern: &str, _count: usize) -> ScanResult<Vec<String>> {
        let items = self.items.read().unwrap();
        let res = if pattern == "*" || pattern.is_empty() {
            items.iter().map(|it| it.0.to_owned()).collect::<Vec<_>>()
        } else {
            items
                .iter()
                .filter(|it| it.0.contains(pattern))
                .map(|it| it.0.to_owned())
                .collect::<Vec<_>>()
        };

        Ok(ScanResponse::new("0".to_string(), res))
    }

    fn del(&self, key: &str) -> Result<i32> {
        let mut items = self.items.write().unwrap();
        let res = if items.remove(key).is_none() { 0 } else { 1 };

        Ok(res)
    }

    fn ttl(&self, key: &str) -> Result<Option<i64>> {
        Ok(None)
    }

    fn data_type(&self, key: &str) -> Result<DataType> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::String(_)) => Ok(DataType::String),
            Some(RedisValue::Hash(_)) => Ok(DataType::Hash),
            Some(RedisValue::List(_)) => Ok(DataType::List),
            Some(RedisValue::Set(_)) => Ok(DataType::Set),
            Some(RedisValue::SortedSet(_)) => Ok(DataType::Zset),
            None => Ok(DataType::None),
        }
    }

    fn delete_all(&self, pattern: &str) -> Result<usize> {
        let mut items = self.items.write().unwrap();
        let keys_to_delete: Vec<String> = if pattern == "*" || pattern.is_empty() {
            items.keys().cloned().collect()
        } else if let Some(prefix) = pattern.strip_suffix('*') {
            items
                .keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect()
        } else {
            items
                .keys()
                .filter(|k| k.contains(pattern)) // Simplified fallback
                .cloned()
                .collect()
        };

        let count = keys_to_delete.len();
        for key in keys_to_delete {
            items.remove(&key);
        }
        Ok(count)
    }
}

impl HashCommands for MockClient {
    fn hlen(&self, key: &str) -> Result<i64> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::Hash(h)) => Ok(h.len() as i64),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(0),
        }
    }

    fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::Hash(h)) => {
                Ok(h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            }
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(vec![]),
        }
    }

    fn hscan(
        &self,
        key: &str,
        cursor: &str,
        count: usize,
    ) -> Result<(String, Vec<(String, String)>)> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::Hash(h)) => {
                let start = cursor.parse::<usize>().unwrap_or(0);
                let fields: Vec<_> = h.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let end = std::cmp::min(start + count, fields.len());
                let next = if end >= fields.len() {
                    "0".to_string()
                } else {
                    end.to_string()
                };
                Ok((next, fields[start..end].to_vec()))
            }
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(("0".to_string(), vec![])),
        }
    }

    fn hset(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        let hash = items
            .entry(key.to_string())
            .or_insert_with(|| RedisValue::Hash(HashMap::new()));
        if let RedisValue::Hash(h) = hash {
            h.insert(field.to_string(), value.to_string());
            Ok(())
        } else {
            bail!("WRONGTYPE Operation against a key holding the wrong kind of value")
        }
    }

    fn hdel(&self, key: &str, field: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        if let Some(RedisValue::Hash(h)) = items.get_mut(key) {
            h.remove(field);
        }
        Ok(())
    }
}

impl ListCommands for MockClient {
    fn llen(&self, key: &str) -> Result<i64> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::List(l)) => Ok(l.len() as i64),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(0),
        }
    }

    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::List(l)) => {
                let len = l.len() as i64;
                let start = if start < 0 { len + start } else { start };
                let stop = if stop < 0 { len + stop } else { stop };
                let start = std::cmp::max(0, start) as usize;
                let stop = std::cmp::min(len - 1, stop) as usize;
                if start > stop || start >= l.len() {
                    Ok(vec![])
                } else {
                    Ok(l[start..=stop].to_vec())
                }
            }
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(vec![]),
        }
    }

    fn rpush(&self, key: &str, value: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        let list = items
            .entry(key.to_string())
            .or_insert_with(|| RedisValue::List(vec![]));
        if let RedisValue::List(l) = list {
            l.push(value.to_string());
            Ok(())
        } else {
            bail!("WRONGTYPE Operation against a key holding the wrong kind of value")
        }
    }

    fn lrem(&self, key: &str, _count: i64, value: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        if let Some(RedisValue::List(l)) = items.get_mut(key) {
            l.retain(|val| val != value);
        }
        Ok(())
    }

    fn lset(&self, key: &str, index: i64, value: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        if let Some(RedisValue::List(l)) = items.get_mut(key) {
            if index >= 0 && (index as usize) < l.len() {
                l[index as usize] = value.to_string();
                Ok(())
            } else {
                bail!("ERR index out of range")
            }
        } else {
            bail!("ERR no such key")
        }
    }
}

impl SetCommands for MockClient {
    fn scard(&self, key: &str) -> Result<i64> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::Set(s)) => Ok(s.len() as i64),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(0),
        }
    }

    fn smembers(&self, key: &str) -> Result<Vec<String>> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::Set(s)) => Ok(s.iter().cloned().collect()),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(vec![]),
        }
    }

    fn sscan(&self, key: &str, cursor: &str, count: usize) -> ScanResult<Vec<String>> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::Set(s)) => {
                let start = cursor.parse::<usize>().unwrap_or(0);
                let members: Vec<String> = s.iter().cloned().collect();
                let end = std::cmp::min(start + count, members.len());
                let next = if end >= members.len() {
                    "0".to_string()
                } else {
                    end.to_string()
                };
                Ok(ScanResponse::new(next, members[start..end].to_vec()))
            }
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(ScanResponse::new("0".to_string(), vec![])),
        }
    }

    fn sadd(&self, key: &str, member: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        let set = items
            .entry(key.to_string())
            .or_insert_with(|| RedisValue::Set(HashSet::new()));
        if let RedisValue::Set(s) = set {
            s.insert(member.to_string());
            Ok(())
        } else {
            bail!("WRONGTYPE Operation against a key holding the wrong kind of value")
        }
    }

    fn srem(&self, key: &str, member: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        if let Some(RedisValue::Set(s)) = items.get_mut(key) {
            s.remove(member);
        }
        Ok(())
    }
}

impl SortedSetCommands for MockClient {
    fn zcard(&self, key: &str) -> Result<i64> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::SortedSet(s)) => Ok(s.len() as i64),
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(0),
        }
    }

    fn zrange_with_scores(&self, key: &str, start: i64, stop: i64) -> Result<Vec<(String, f64)>> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::SortedSet(s)) => {
                let len = s.len() as i64;
                let start = if start < 0 { len + start } else { start };
                let stop = if stop < 0 { len + stop } else { stop };
                let start = std::cmp::max(0, start) as usize;
                let stop = std::cmp::min(len - 1, stop) as usize;
                if start > stop || start >= s.len() {
                    Ok(vec![])
                } else {
                    Ok(s[start..=stop].to_vec())
                }
            }
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(vec![]),
        }
    }

    fn zscan(&self, key: &str, cursor: &str, count: usize) -> ScanResult<Vec<(String, f64)>> {
        let items = self.items.read().unwrap();
        match items.get(key) {
            Some(RedisValue::SortedSet(s)) => {
                let start = cursor.parse::<usize>().unwrap_or(0);
                let end = std::cmp::min(start + count, s.len());
                let next = if end >= s.len() {
                    "0".to_string()
                } else {
                    end.to_string()
                };
                Ok(ScanResponse::new(next, s[start..end].to_vec()))
            }
            Some(_) => bail!("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => Ok(ScanResponse::new("0".to_string(), vec![])),
        }
    }

    fn zadd(&self, key: &str, score: f64, member: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        let zset = items
            .entry(key.to_string())
            .or_insert_with(|| RedisValue::SortedSet(vec![]));
        if let RedisValue::SortedSet(s) = zset {
            if let Some(pos) = s.iter().position(|(m, _)| m == member) {
                s.remove(pos);
            }
            s.push((member.to_string(), score));
            s.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(())
        } else {
            bail!("WRONGTYPE Operation against a key holding the wrong kind of value")
        }
    }

    fn zrem(&self, key: &str, member: &str) -> Result<()> {
        let mut items = self.items.write().unwrap();
        if let Some(RedisValue::SortedSet(s)) = items.get_mut(key) {
            s.retain(|(m, _)| m != member);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::commands::*;

    #[test]
    fn test_mock_strings() -> Result<()> {
        let mock = MockClient::default();
        mock.set("key1", "val1")?;
        assert_eq!(mock.get("key1")?, "val1");
        assert_eq!(mock.strlen("key1")?, 4);
        assert_eq!(mock.data_type("key1")?, DataType::String);
        Ok(())
    }

    #[test]
    fn test_mock_hashes() -> Result<()> {
        let mock = MockClient::default();
        mock.hset("h1", "f1", "v1")?;
        mock.hset("h1", "f2", "v2")?;
        assert_eq!(mock.hlen("h1")?, 2);
        let all = mock.hgetall("h1")?;
        assert_eq!(all.len(), 2);
        assert!(all.contains(&("f1".to_string(), "v1".to_string())));
        Ok(())
    }

    #[test]
    fn test_mock_lists() -> Result<()> {
        let mock = MockClient::default();
        mock.rpush("l1", "i1")?;
        mock.rpush("l1", "i2")?;
        assert_eq!(mock.llen("l1")?, 2);
        let range = mock.lrange("l1", 0, -1)?;
        assert_eq!(range, vec!["i1", "i2"]);
        Ok(())
    }

    #[test]
    fn test_mock_sets() -> Result<()> {
        let mock = MockClient::default();
        mock.sadd("s1", "m1")?;
        mock.sadd("s1", "m2")?;
        assert_eq!(mock.scard("s1")?, 2);
        let members = mock.smembers("s1")?;
        assert_eq!(members.len(), 2);
        Ok(())
    }

    #[test]
    fn test_mock_zsets() -> Result<()> {
        let mock = MockClient::default();
        mock.zadd("z1", 10.0, "m1")?;
        mock.zadd("z1", 20.0, "m2")?;
        assert_eq!(mock.zcard("z1")?, 2);
        let range = mock.zrange_with_scores("z1", 0, -1)?;
        assert_eq!(range[0], ("m1".to_string(), 10.0));
        assert_eq!(range[1], ("m2".to_string(), 20.0));
        Ok(())
    }

    #[test]
    fn test_mock_delete_all() -> Result<()> {
        let mock = MockClient::default();
        mock.set("test:1", "v")?;
        mock.set("test:2", "v")?;
        mock.set("other", "v")?;
        let deleted = mock.delete_all("test:*")?;
        assert_eq!(deleted, 2);
        assert_eq!(mock.dbsize()?, 1);
        Ok(())
    }
}
