use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Widget, canvas::Canvas},
};

fn main() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}

struct App {
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self { exit: false }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
        loop {
            handle_key_event(self, terminal)?;
            if self.exit {
                break;
            }

            terminal.draw(|f| self.draw(f))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
        let [left, right] = layout.areas(frame.area());

        frame.render_widget(self.left_canvas(), left);
        frame.render_widget(self.right_canvas(), right);
    }

    fn left_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Menu"))
            .paint(|ctx| {
                ctx.print(0., 0., "Menu here");
            })
    }

    fn right_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Details"))
            .paint(|ctx| {
                ctx.print(0., 0., "Details here");
            })
    }
}

fn handle_key_event(arg: &mut App, _terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
    if event::poll(Duration::from_millis(100))?
        && let Event::Key(key) = event::read()?
    {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => arg.exit = true,
            _ => {}
        }
    }
    Ok(())
}
