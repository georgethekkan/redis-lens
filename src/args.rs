use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Debug, Clone, Parser)]
pub struct Arg {
    /// Redis/Valkey server URL
    #[clap(long, default_value = "localhost:6379")]
    pub url: String,

    /// Database to use
    #[clap(long, default_value = "0")]
    pub db: u8,

    /// Specific key to fetch
    #[clap(short, long)]
    pub key: Option<String>,

    /// dry run, do not connect to redis
    #[clap(short, long)]
    pub dry_run: bool,

    /// delete all keys matching the pattern
    #[clap(long)]
    pub delete_all: Option<String>,
}

pub fn parse() -> Arg {
    Arg::parse()
}
