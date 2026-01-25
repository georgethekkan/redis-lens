#![allow(unused)]
use std::io::stdout;

use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use crossterm::ExecutableCommand;

use redis::Commands;
use redis_lens::redis::RedisClient;
use redis_lens::{app::App, args};
use redis_lens::{delete_keys, get, scan, set, start_ui};

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
        Some(args::Commands::Scan { pattern }) => scan(pattern, &redis_client),
        None => start_ui(redis_client),
    };

    Ok(())
}
