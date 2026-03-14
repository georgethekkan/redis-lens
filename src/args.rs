use clap::{Args, Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
pub struct Arg {
    #[command(flatten)]
    pub config: Config,

    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Get {
        /// Key to fetch
        key: String,
    },
    Set {
        /// Key to set
        key: String,
        /// Value to set
        value: String,
    },
    Delete {
        /// Key to delete
        key: String,
    },
    DeleteAll {
        /// Pattern to match keys to delete
        pattern: String,
    },
    Scan {
        /// Pattern to match
        pattern: String,
    },
}

#[derive(Debug, Clone, Args)]
pub struct Config {
    /// Redis server URL
    #[clap(long, default_value = "localhost:6379")]
    pub url: String,

    /// Username for Redis authentication
    #[clap(long)]
    pub username: Option<String>,

    /// Password for Redis authentication
    #[clap(long)]
    pub password: Option<String>,

    /// Database to use
    #[clap(long, default_value = "0")]
    pub db: u8,

    /// Use a mock Redis for testing/dry run
    #[clap(long)]
    pub mock: bool,

    /// Start in read-only mode (destructive actions disabled)
    #[clap(long)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Args)]
pub struct KeyValue {
    /// key to set
    #[clap(long)]
    pub key: String,
    /// value to set
    #[clap(long)]
    pub value: String,
}

pub fn parse() -> Arg {
    Arg::parse()
}

impl Config {
    pub fn new(url: String, db: u8) -> Self {
        Self {
            url,
            username: None,
            password: None,
            db,
            mock: false,
            read_only: false,
        }
    }
}
