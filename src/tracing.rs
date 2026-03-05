use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup() -> WorkerGuard {
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
