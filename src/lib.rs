//#![allow(unused)]
use std::io::stdout;

use color_eyre::eyre::{Context, Result};
use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
};

use crate::redis::ClientOps;
use crate::{app::App, redis::ScanResponse};

pub mod app;
pub mod args;
pub mod redis;
pub mod tracing;
pub mod tree;
pub mod ui;

use crate::args::{Arg, Commands};

/// Main entry point for the CLI and TUI application.
///
/// Dispatches to subcommands or starts the interactive TUI based on arguments.
pub fn run<T: ClientOps>(args: &Arg, client: T) -> Result<()> {
    match &args.cmd {
        Some(Commands::Get { key }) => get(key, &client),
        Some(Commands::Set { key, value }) => set(key, value, &client),
        Some(Commands::Delete { key }) => delete_keys(key, &client),
        Some(Commands::DeleteAll { pattern }) => delete_keys(pattern, &client),
        Some(Commands::Scan { pattern }) => scan(pattern, &client),
        None => start_ui(client),
    }
}

/// Starts the interactive Terminal User Interface (TUI).
pub fn start_ui<R: ClientOps>(client: R) -> Result<()> {
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();

    let mut app = App::new(client)?;
    app.run(&mut terminal)?;

    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;
    Ok(())
}

/// Deletes keys matching the specified pattern.
pub fn delete_keys<R: ClientOps>(pattern: &str, client: &R) -> Result<()> {
    println!("Deleting keys matching pattern: {}", pattern);
    let count = client.delete_all(pattern)?;
    if count == 0 {
        println!("No keys found matching pattern: {}", pattern);
    } else {
        println!("Deleted {} keys matching pattern: {}", count, pattern);
    }
    Ok(())
}

/// Fetches the value of a string key.
pub fn get<R: ClientOps>(key: &str, client: &R) -> Result<()> {
    println!("Fetching key: {}", key);
    let value: String = client.get(key).context("Failed to get key from Redis")?;
    println!("Value: {}", value);
    Ok(())
}

/// Sets the value of a string key.
pub fn set<R: ClientOps>(key: &str, value: &str, client: &R) -> Result<()> {
    println!("Setting key: {} to value: {}", key, value);
    client
        .set(key, value)
        .context("Failed to set key in Redis")?;
    println!("Key set successfully.");
    Ok(())
}

/// Scans keys matching the specified pattern and prints the first page of results.
pub fn scan<R: ClientOps>(pattern: &str, client: &R) -> Result<()> {
    let ScanResponse { next, keys } = client.scan("0", pattern, 100)?;

    println!("Found {} keys (first page): {:?}", keys.len(), keys);
    if next != "0" {
        println!("Next cursor: {}", next);
    }

    Ok(())
}
