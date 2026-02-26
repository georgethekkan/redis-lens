//#![allow(unused)]
use color_eyre::Result;

use color_eyre::eyre::Ok;
use redis_lens::redis::{LensClient, MockClient};
use redis_lens::{args, run};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    let _guard = setup_logging();

    color_eyre::install()?;

    let args = args::parse();

    if args.config.mock {
        let mock = MockClient::default();
        mock.setup_keys()?;
        run(&args, mock)?;
    } else {
        let client = LensClient::new(&args.config)?;
        run(&args, client)?;
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
