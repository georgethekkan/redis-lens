#![allow(unused)]
use std::io::stdout;

use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use crossterm::ExecutableCommand;

use redis::Commands;
use redis_lens::redis::RedisClient;
use redis_lens::{app::App, args};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = args::parse();

    let redis_client = RedisClient::new(&args.redis_config)?;

    match &args.cmd {
        Some(args::Commands::Get { key }) => get(key, &redis_client),
        Some(args::Commands::Set { key, value }) => set(key, value, &redis_client),
        Some(args::Commands::Delete { key }) => delete_keys(key, &redis_client),
        Some(args::Commands::DeleteAll { pattern }) => delete_keys(pattern, &redis_client),
        None => start_ui(redis_client),
    };

    Ok(())
}

fn start_ui(redis_client: RedisClient) -> Result<()> {
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();
    App::new(redis_client)?.run(&mut terminal)?;
    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;
    Ok(())
}

fn delete_keys(pattern: &str, redis_client: &RedisClient) -> Result<()> {
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

fn get(key: &str, redis_client: &RedisClient) -> Result<()> {
    println!("Fetching key: {}", key);
    let value: String = redis_client
        .get(key)
        .context("Failed to get key from Redis")?;
    println!("Value: {}", value);
    Ok(())
}

fn set(key: &str, value: &str, redis_client: &RedisClient) -> Result<()> {
    println!("Setting key: {} to value: {}", key, value);
    redis_client
        .set(key, value)
        .context("Failed to set key in Redis")?;
    println!("Key set successfully.");
    Ok(())
}
