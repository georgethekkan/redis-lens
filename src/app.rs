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
    next_cursor: String,
    list_state: ListState,
    message: Option<String>,
}

impl App {
    pub fn new(redis_client: RedisClient) -> Result<Self> {
        let (next_cursor, keys) = redis_client.scan("0", "*", 100)?;
        let app = Self {
            exit: false,
            redis_client,
            keys,
            next_cursor,
            list_state: ListState::default(),
            message: None,
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
                    KeyCode::Char('n') => self.load_next_page()?,
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
                self.message = Some(format!("Deleted key: {}", key));
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

    fn load_next_page(&mut self) -> Result<()> {
        if self.next_cursor == "0" {
            return Ok(());
        }
        let (new_cursor, new_keys) = self.redis_client.scan(&self.next_cursor, "*", 100)?;
        self.next_cursor = new_cursor;
        self.keys.extend(new_keys);
        self.message = Some("Loaded next page of keys.".to_string());

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) -> Result<()> {
        // Overall layout: main area and help area
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)]);
        let [main_area, help_area] = layout.areas(frame.area());

        // Add left and right panels in main_area
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
        let [left, right] = layout.areas(main_area);

        // Left panel: key list
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

        let message = match &self.message {
            Some(msg) => msg.clone(),
            None => format!(
                "{} | q/Esc: Quit | Up/Down: Navigate | d: Delete Key | n: Next Page",
                self.redis_client.url()
            ),
        };

        let details = Paragraph::new(message).block(Block::bordered());
        frame.render_widget(details, help_area);

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
