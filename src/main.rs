#![allow(unused)]
use std::io::stdout;

use color_eyre::Result;
use color_eyre::eyre::Context;
use crossterm::ExecutableCommand;

use redis::Commands;
use redis_lens::redis::RedisClient;
use redis_lens::{app::App, args};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = args::parse();

    let redis_client = RedisClient::new(args.url, args.db)?;

    if let Some(key) = args.key {
        handle_key(&key, &redis_client);
        return Ok(());
    }

    stdout().execute(EnableMouseCapture)?;
    let terminal = ratatui::init();
    App::new(redis_client).run(terminal)?;
    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;

    Ok(())
}

fn handle_key(key: &str, redis_client: &RedisClient) -> Result<()> {
    println!("Fetching key: {}", key);
    let mut conn = redis_client.get_connection()?;

    let value: String = conn.get(key).context("Failed to get key from Redis")?;
    println!("Value: {}", value);
    Ok(())
}
