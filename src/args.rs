use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Debug, Clone, Parser)]
pub struct Arg {
    /// Redis/Valkey server URL
    #[clap(long, default_value = "localhost:6379")]
    pub url: String,

    /// Database to use
    #[clap(long, default_value = "0")]
    pub db: u8,

    /// dry run, do not connect to redis
    #[clap(short, long)]
    pub dry_run: bool,

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