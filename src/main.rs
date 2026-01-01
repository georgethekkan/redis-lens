#![allow(unused)]
use std::io::stdout;

use color_eyre::Result;
use crossterm::ExecutableCommand;

use redis_lens::app::App;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

fn main() -> Result<()> {
    color_eyre::install()?;
    stdout().execute(EnableMouseCapture)?;
    let terminal = ratatui::init();
    App::new().run(terminal)?;
    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;

    Ok(())
}
