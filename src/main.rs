//#![allow(unused)]
use color_eyre::Result;

use redis_lens::args::{self, Commands};
use redis_lens::redis::RedisClient;
use redis_lens::{delete_keys, get, scan, set, start_ui};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    let _guard = setup_logging();

    color_eyre::install()?;

    let args = args::parse();

    let redis_client = RedisClient::new(&args.config)?;

    match &args.cmd {
        Some(Commands::Get { key }) => get(key, &redis_client),
        Some(Commands::Set { key, value }) => set(key, value, &redis_client),
        Some(Commands::Delete { key }) => delete_keys(key, &redis_client),
        Some(Commands::DeleteAll { pattern }) => delete_keys(pattern, &redis_client),
        Some(Commands::Scan { pattern }) => scan(pattern, &redis_client),
        None => start_ui(redis_client),
    }?;

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
