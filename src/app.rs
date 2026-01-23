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

pub struct App {
    exit: bool,
    redis_client: RedisClient,
    keys: Vec<String>,
    list_state: ListState,
}

impl App {
    pub fn new(redis_client: RedisClient) -> Result<Self> {
        let keys = redis_client.scan()?;
        let app = Self {
            exit: false,
            redis_client,
            keys,
            list_state: ListState::default(),
        };
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f).unwrap())?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                    KeyCode::Down => self.list_state.select_next(),
                    KeyCode::Up => self.list_state.select_previous(),
                    KeyCode::Char('d') => self.delete_selected_key()?,
                    _ => {}
                }
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn delete_selected_key(&mut self) -> Result<()> {
        if let Some(index) = self.list_state.selected() {
            if let Some(key) = self.keys.get(index) {
                self.redis_client.del(key)?;
                self.keys.remove(index);
                if self.keys.is_empty() {
                    self.list_state.select(None);
                } else if index >= self.keys.len() {
                    self.list_state.select(Some(self.keys.len() - 1));
                }
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
            // Get type
            let key_type = match self.redis_client.key_type(key) {
                Ok(t) => format!("Type: {}\n", t),
                Err(e) => format!("Type: Error - {}\n", e),
            };

            // Get TTL
            let ttl_info = match self.redis_client.ttl(key) {
                Ok(Some(-1)) => "TTL: No expiration\n".to_string(),
                Ok(Some(ttl)) => format!("TTL: {} seconds\n", ttl),
                Ok(None) => "TTL: Key does not exist\n".to_string(),
                Err(e) => format!("TTL: Error - {}\n", e),
            };

            // Get value
            let value_info = match self.redis_client.get(key) {
                Ok(value) => format!("Value:\n{}", value),
                Err(e) => format!("Value: Error - {}", e),
            };

            details_text = format!("{}{}{}", key_type, ttl_info, value_info);
        }

        let details = Paragraph::new(details_text).block(Block::bordered().title("Details"));
        frame.render_widget(details, right);

        Ok(())
    }
}
