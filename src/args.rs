use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Debug, Clone, Parser)]
pub struct Arg {
    /// Redis/Valkey server URL
    #[clap(short, long, default_value = "localhost:6379")]
    pub url: String,

    /// Database to use
    #[clap(short, long, default_value = "0")]
    pub db: u8,

    /// Specific key to fetch
    #[clap(short, long)]
    pub key: Option<String>,
}

pub fn parse() -> Arg {
    Arg::parse()
}
