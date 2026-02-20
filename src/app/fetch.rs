use crate::app::{App, CollectionData, LoadedKeyData};
use color_eyre::eyre::Result;

impl<R: crate::redis::RedisOps> App<R> {
    pub fn fetch_details_for_key(&mut self, key: &str) -> Result<()> {
        let key = key.to_string();

        // 1. Get Type
        let key_type = self
            .redis_client
            .key_type(&key)
            .unwrap_or_else(|e| format!("Error - {}", e));

        // 2. Get TTL
        let ttl_info = match self.redis_client.ttl(&key) {
            Ok(Some(-1)) => "No expiration".to_string(),
            Ok(Some(ttl)) => format!("{} seconds", ttl),
            Ok(None) => "Key does not exist".to_string(),
            Err(e) => format!("Error - {}", e),
        };

        // 3. Get Content & Length based on type
        let (length, content) = match key_type.as_str() {
            "string" => self.fetch_string_content(&key),
            "list" => self.fetch_list_content(&key),
            "hash" => self.fetch_hash_content(&key),
            "set" => self.fetch_set_content(&key),
            "zset" => self.fetch_zset_content(&key),
            _ => (0, CollectionData::None),
        };

        self.loaded_key = Some(LoadedKeyData {
            key,
            key_type,
            ttl: ttl_info,
            length,
            content,
        });

        Ok(())
    }

    fn fetch_string_content(&self, key: &str) -> (i64, CollectionData) {
        let val = self.redis_client.get(key).unwrap_or_else(|e| e.to_string());
        let len = self.redis_client.strlen(key).unwrap_or(0);
        (len, CollectionData::String(val, len as usize))
    }

    fn fetch_list_content(&self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.llen(key).unwrap_or(0);
        let (start, stop) = page_range_i64(self.collection_page, self.collection_page_size);
        let items = self
            .redis_client
            .lrange(key, start, stop)
            .unwrap_or_default();
        (len, CollectionData::List(items))
    }

    fn fetch_hash_content(&mut self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.hlen(key).unwrap_or(0);
        let cursor = self.get_current_cursor();
        let (next_cursor, items) = self
            .redis_client
            .hscan(key, cursor, self.collection_page_size)
            .unwrap_or(("0".to_string(), vec![]));

        self.update_next_cursor(next_cursor);
        (len, CollectionData::Hash(items))
    }

    fn fetch_set_content(&mut self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.scard(key).unwrap_or(0);
        let cursor = self.get_current_cursor();
        let (next_cursor, items) = self
            .redis_client
            .sscan(key, cursor, self.collection_page_size)
            .unwrap_or(("0".to_string(), vec![]));

        self.update_next_cursor(next_cursor);
        (len, CollectionData::Set(items))
    }

    fn fetch_zset_content(&self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.zcard(key).unwrap_or(0);
        let (start, stop) = page_range_i64(self.collection_page, self.collection_page_size);
        let items = self
            .redis_client
            .zrange_with_scores(key, start, stop)
            .unwrap_or_default();
        (len, CollectionData::ZSet(items))
    }

    pub fn next_collection_page(&mut self) {
        let should_advance = if let Some(data) = &self.loaded_key {
            match data.key_type.as_str() {
                "hash" | "set" => self.collection_cursors.len() > self.collection_page + 1,
                _ => true,
            }
        } else {
            false
        };

        if should_advance {
            self.collection_page += 1;
        }
    }

    pub fn prev_collection_page(&mut self) {
        if self.collection_page > 0 {
            self.collection_page -= 1;
        }
    }
}

pub fn page_range_i64(page: usize, page_size: usize) -> (i64, i64) {
    let start = (page.saturating_mul(page_size)) as i64;
    let stop = start + page_size as i64 - 1;
    (start, stop)
}

#[allow(dead_code)]
pub fn slice_bounds(total: usize, page: usize, page_size: usize) -> (usize, usize) {
    let start = page.saturating_mul(page_size);
    let end = std::cmp::min(start + page_size, total);
    (start, end)
}
