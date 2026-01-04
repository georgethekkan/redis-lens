#![allow(unused)]
use std::io::stdout;

use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use crossterm::ExecutableCommand;

use redis::Commands;
use redis_lens::redis::{RedisClient, RedisClientImpl, RedisClientMock};
use redis_lens::{app::App, args};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = args::parse();

    let redis_client: Box<dyn RedisClient> = if args.dry_run {
        Box::new(RedisClientMock::new("mock".to_string()))
    } else {
        Box::new(RedisClientImpl::new(args.url, args.db)?)
    };

    if let Some(key) = args.key {
        handle_key(&key, &redis_client);
        return Ok(());
    }

    stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();
    App::new(redis_client)?.run(&mut terminal)?;
    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;

    Ok(())
}

fn handle_key(key: &str, redis_client: &dyn RedisClient) -> Result<()> {
    println!("Fetching key: {}", key);
    let value: String = redis_client
        .get(key)
        .context("Failed to get key from Redis")?;
    println!("Value: {}", value);
    Ok(())
}
