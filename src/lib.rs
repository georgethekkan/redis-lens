#![allow(unused)]
use std::io::stdout;

use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use crossterm::ExecutableCommand;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

use crate::app::App;
use crate::redis::RedisClient;

pub mod app;
pub mod args;
pub mod redis;

pub fn start_ui(redis_client: RedisClient) -> Result<()> {
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();
    App::new(redis_client)?.run(&mut terminal)?;
    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;
    Ok(())
}

pub fn delete_keys(pattern: &str, redis_client: &RedisClient) -> Result<()> {
    let keys = redis_client.scan_pattern(pattern)?;
    if keys.is_empty() {
        println!("No keys found matching pattern: {}", pattern);
        return Ok(());
    }
    println!("Deleting {} keys matching pattern: {}", keys.len(), pattern);
    for key in keys {
        redis_client.del(&key)?;
        println!("Deleted key: {}", key);
    }
    Ok(())
}

pub fn get(key: &str, redis_client: &RedisClient) -> Result<()> {
    println!("Fetching key: {}", key);
    let value: String = redis_client
        .get(key)
        .context("Failed to get key from Redis")?;
    println!("Value: {}", value);
    Ok(())
}

pub fn set(key: &str, value: &str, redis_client: &RedisClient) -> Result<()> {
    println!("Setting key: {} to value: {}", key, value);
    redis_client
        .set(key, value)
        .context("Failed to set key in Redis")?;
    println!("Key set successfully.");
    Ok(())
}

pub fn scan(pattern: &str, redis_client: &RedisClient) -> Result<()> {
    let (next_cursor, keys) = redis_client.scan("0", 100)?;

    /*let keys = redis_client.scan_pattern(pattern)?;
    if keys.is_empty() {
        println!("No keys found matching pattern: {}", pattern);
        return Ok(());
    }*/

    println!("Found {} keys (first page): {:?}", keys.len(), keys);
    if next_cursor != "0" {
        println!("Next cursor: {}", next_cursor);
    }

    Ok(())
}
