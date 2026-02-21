use redis_lens::args::Config;
use redis_lens::redis::RedisClient;
use redis_lens::redis::commands::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn get_client() -> RedisClient {
    let config = Config {
        url: "127.0.0.1:6379".to_string(),
        db: 15, // Use DB 15 for tests
        username: None,
        password: None,
    };
    RedisClient::new(&config).expect("Failed to connect to Redis")
}

fn get_random_key(prefix: &str) -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    format!("{}:{}", prefix, since_the_epoch.as_nanos())
}

#[test]
fn test_connection() {
    let _client = get_client();
    // If we got here, connection creation (and pool build) succeeded.
}

#[test]
fn test_strings() {
    let client = get_client();
    let key = get_random_key("test:string");
    let value = "hello world";

    // Set
    client.set(&key, value).expect("Failed to set key");

    // Get
    let fetched = client.get(&key).expect("Failed to get key");
    assert_eq!(fetched, value);

    // Strlen
    let len = client.strlen(&key).expect("Failed to get strlen");
    assert_eq!(len, value.len() as i64);

    // Del
    client.del(&key).expect("Failed to del key");
    let fetched_after = client.get(&key);
    assert!(fetched_after.is_err() || fetched_after.unwrap().is_empty()); // Redis "get" might return empty string or error for key-not-found depending on impl
}

#[test]
fn test_lists() {
    let client = get_client();
    let key = get_random_key("test:list");

    client.rpush(&key, "item1").expect("RPUSH failed");
    client.rpush(&key, "item2").expect("RPUSH failed");

    let len = client.llen(&key).expect("Failed to get llen");
    assert_eq!(len, 2);

    let items = client.lrange(&key, 0, -1).expect("Failed to get lrange");
    assert_eq!(items, vec!["item1", "item2"]);

    client.del(&key).expect("Del failed");
}

#[test]
fn test_hashes() {
    let client = get_client();
    let key = get_random_key("test:hash");

    client.hset(&key, "field1", "val1").expect("HSET failed");
    client.hset(&key, "field2", "val2").expect("HSET failed");

    let len = client.hlen(&key).expect("Failed to messure hlen");
    assert_eq!(len, 2);

    let all = client.hgetall(&key).expect("Failed to hgetall");
    // Order is random in hash
    assert_eq!(all.len(), 2);

    // HSCAN
    let (cursor, items) = client.hscan(&key, "0", 10).expect("Failed to hscan");
    assert_eq!(cursor, "0"); // Should fit in one page
    assert_eq!(items.len(), 2);

    client.del(&key).expect("Del failed");
}

#[test]
fn test_sets() {
    let client = get_client();
    let key = get_random_key("test:set");

    client.sadd(&key, "member1").expect("SADD failed");
    client.sadd(&key, "member2").expect("SADD failed");

    let card = client.scard(&key).expect("Failed to scard");
    assert_eq!(card, 2);

    let members = client.smembers(&key).expect("Failed to smembers");
    assert_eq!(members.len(), 2);
    assert!(members.contains(&"member1".to_string()));

    let (cursor, page) = client.sscan(&key, "0", 10).expect("Failed to sscan");
    assert_eq!(cursor, "0");
    assert_eq!(page.len(), 2);

    client.del(&key).expect("Del failed");
}

#[test]
fn test_sorted_sets() {
    let client = get_client();
    let key = get_random_key("test:zset");

    client.zadd(&key, 10.0, "p1").expect("ZADD failed");
    client.zadd(&key, 20.0, "p2").expect("ZADD failed");

    let card = client.zcard(&key).expect("Failed to zcard");
    assert_eq!(card, 2);

    let range = client
        .zrange_with_scores(&key, 0, -1)
        .expect("Failed to zrange");
    assert_eq!(range.len(), 2);
    assert_eq!(range[0], ("p1".to_string(), 10.0));
    assert_eq!(range[1], ("p2".to_string(), 20.0));

    let (cursor, page) = client.zscan(&key, "0", 10).expect("Failed to zscan");
    assert_eq!(cursor, "0");
    assert_eq!(page.len(), 2);

    client.del(&key).expect("Del failed");
}

#[test]
fn test_pagination() {
    let client = get_client();

    // 1. Key Scan Pagination
    let prefix = get_random_key("scan_test");
    let total_keys = 50;
    for i in 0..total_keys {
        client
            .set(&format!("{}:{}", prefix, i), "val")
            .expect("Set failed");
    }

    let mut cursor = "0".to_string();
    let mut collected_keys = std::collections::HashSet::new();
    let pattern = format!("{}*", prefix);

    loop {
        let (next, keys) = client.scan(&cursor, &pattern, 10).expect("Scan failed");
        for k in keys {
            collected_keys.insert(k);
        }
        cursor = next;
        if cursor == "0" {
            break;
        }
    }

    // Cleanup first to avoid polluting if assertion fails (though panic stops cleanup)
    // Actually better to delete by pattern? But we don't have delete_all implemented in client logic generically yet, only via main.
    // We can rely on randomness of keys to avoid collision.

    assert_eq!(collected_keys.len(), total_keys);

    // 2. Set Pagination
    let set_key = get_random_key("set_pagination");
    for i in 0..total_keys {
        client
            .sadd(&set_key, &format!("member{}", i))
            .expect("Sadd failed");
    }

    let mut cursor = "0".to_string();
    let mut collected_members = std::collections::HashSet::new();
    loop {
        let (next, members) = client.sscan(&set_key, &cursor, 10).expect("Sscan failed");
        for m in members {
            collected_members.insert(m);
        }
        cursor = next;
        if cursor == "0" {
            break;
        }
    }
    assert_eq!(collected_members.len(), total_keys);

    // 3. Hash Pagination
    let hash_key = get_random_key("hash_pagination");
    for i in 0..total_keys {
        client
            .hset(&hash_key, &format!("f{}", i), "v")
            .expect("Hset failed");
    }

    let mut cursor = "0".to_string();
    let mut collected_fields = std::collections::HashSet::new();
    loop {
        let (next, items) = client.hscan(&hash_key, &cursor, 10).expect("Hscan failed");
        for (f, _) in items {
            collected_fields.insert(f);
        }
        cursor = next;
        if cursor == "0" {
            break;
        }
    }
    assert_eq!(collected_fields.len(), total_keys);

    // 4. ZSet Pagination
    let zset_key = get_random_key("zset_pagination");
    for i in 0..total_keys {
        client
            .zadd(&zset_key, i as f64, &format!("m{}", i))
            .expect("Zadd failed");
    }

    let mut cursor = "0".to_string();
    let mut collected_zmembers = std::collections::HashSet::new();
    loop {
        let (next, items) = client.zscan(&zset_key, &cursor, 10).expect("Zscan failed");
        for (m, _) in items {
            collected_zmembers.insert(m);
        }
        cursor = next;
        if cursor == "0" {
            break;
        }
    }
    assert_eq!(collected_zmembers.len(), total_keys);
}

#[test]
fn test_delete_all() {
    let client = get_client();
    let prefix = get_random_key("del_test");
    let total_keys = 20;

    // Setup keys
    for i in 0..total_keys {
        client
            .set(&format!("{}:{}", prefix, i), "val")
            .expect("Set failed");
    }

    // Verify they exist
    let (_, keys) = client
        .scan("0", &format!("{}*", prefix), 100)
        .expect("Scan failed");
    assert_eq!(keys.len(), total_keys);

    // Delete all
    let deleted_count = client
        .delete_all(&format!("{}*", prefix))
        .expect("Delete all failed");
    assert_eq!(deleted_count, total_keys);

    // Verify gone
    let (_, keys_after) = client
        .scan("0", &format!("{}*", prefix), 100)
        .expect("Scan failed");
    assert_eq!(keys_after.len(), 0);
}
