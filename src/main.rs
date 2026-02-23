//#![allow(unused)]
use color_eyre::Result;

use redis_lens::args::{self, Commands};
use redis_lens::redis::LensClient;
use redis_lens::redis::commands::*;
use redis_lens::{delete_keys, get, scan, set, start_ui};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    let _guard = setup_logging();

    color_eyre::install()?;

    let args = args::parse();

    if args.config.mock {
        let mock = redis_lens::redis::MockClient::default();
        // Pre-populate with some sample data for demo
        mock.set("demo:string", "Hello Redis Lens!")?;
        mock.hset("demo:hash", "version", "0.1.0")?;
        mock.hset("demo:hash", "author", "George")?;
        mock.rpush("demo:list", "item 1")?;
        mock.rpush("demo:list", "item 2")?;
        mock.sadd("demo:set", "member A")?;
        mock.sadd("demo:set", "member B")?;

        match &args.cmd {
            Some(Commands::Get { key }) => get(key, &mock),
            Some(Commands::Set { key, value }) => set(key, value, &mock),
            Some(Commands::Delete { key }) => delete_keys(key, &mock),
            Some(Commands::DeleteAll { pattern }) => delete_keys(pattern, &mock),
            Some(Commands::Scan { pattern }) => scan(pattern, &mock),
            None => start_ui(mock),
        }?;
    } else {
        let client = LensClient::new(&args.config)?;
        match &args.cmd {
            Some(Commands::Get { key }) => get(key, &client),
            Some(Commands::Set { key, value }) => set(key, value, &client),
            Some(Commands::Delete { key }) => delete_keys(key, &client),
            Some(Commands::DeleteAll { pattern }) => delete_keys(pattern, &client),
            Some(Commands::Scan { pattern }) => scan(pattern, &client),
            None => start_ui(client),
        }?;
    }

    Ok(())
}

fn setup_logging() -> WorkerGuard {
    if let Err(e) = std::fs::create_dir_all("./logs") {
        eprintln!("Warning: Failed to create logs directory: {}", e);
    }
    let file_appender = tracing_appender::rolling::daily("./logs", "redis-lens.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    guard
}
