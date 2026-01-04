use std::io;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::canvas::{Canvas, Map, MapResolution};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};

use super::redis::RedisClient;

#[derive(Debug)]
pub struct App<T: RedisClient> {
    exit: bool,
    redis_client: T,
    keys: Vec<String>,
    list_state: ListState,
}

impl<T: RedisClient> App<T> {
    pub fn new(redis_client: T) -> Result<Self> {
        let keys = redis_client.scan()?;
        Ok(Self {
            exit: false,
            redis_client,
            keys,
            list_state: ListState::default(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f).unwrap())?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                    KeyCode::Down => self.list_state.select_next(),
                    KeyCode::Up => self.list_state.select_previous(),
                    _ => {}
                }
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
        let [left, right] = layout.areas(frame.area());

        // Left panel: List
        let list_items: Vec<ListItem> = self
            .keys
            .iter()
            .map(|k| ListItem::new(k.as_str()))
            .collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("Items"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, left, &mut self.list_state);

        // Right panel: Details
        let mut details_text = String::new();
        if let Some(index) = self.list_state.selected()
            && let Some(key) = self.keys.get(index)
        {
            match self.redis_client.get(key) {
                Ok(value) => details_text = value,
                Err(e) => details_text = format!("Error: {}", e),
            }
        }

        let details = Paragraph::new(details_text).block(Block::bordered().title("Details"));
        frame.render_widget(details, right);

        Ok(())
    }
}
