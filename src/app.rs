use std::io;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::canvas::{Canvas, Map, MapResolution};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Table, Row, Cell, Widget};
use ratatui::{DefaultTerminal, Frame};

use super::redis::RedisClient;

pub struct App {
    exit: bool,
    redis_client: RedisClient,
    keys: Vec<String>,
    next: String,
    list_state: ListState,
    message: Option<String>,
}

impl App {
    pub fn new(redis_client: RedisClient) -> Result<Self> {
        let res =  redis_client.scan("0", "*", 100);

        let (next, keys, message) = res
            .map(|(cursor, ks)| (cursor, ks, None))
            .unwrap_or_else(|e| ("0".to_owned(), Vec::new(), Some(format!("Failed to load keys: {}", e))));

        let app = Self {
            exit: false,
            redis_client,
            keys,
            next,
            list_state: ListState::default(),
            message,
        };

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f).unwrap())?;

            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key)?;
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Down => self.list_state.select_next(),
            KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Char('d') => self.delete_selected_key()?,
            KeyCode::Char('n') => self.load_next_page()?,
            _ => {}
        }
        Ok(())
    }

    fn delete_selected_key(&mut self) -> Result<()> {
        if let Some(index) = self.list_state.selected()
            && let Some(key) = self.keys.get(index)
        {
            self.redis_client.del(key)?;
            self.message = Some(format!("Deleted key: {}", key));
            self.keys.remove(index);
            if self.keys.is_empty() {
                self.list_state.select(None);
            } else if index >= self.keys.len() {
                self.list_state.select(Some(self.keys.len() - 1));
            }
        }
        Ok(())
    }

    fn load_next_page(&mut self) -> Result<()> {
        if self.next == "0" {
            return Ok(());
        }
        let (new_cursor, new_keys) = self.redis_client.scan(&self.next, "*", 100)?;
        self.next = new_cursor;
        self.keys.extend(new_keys);
        self.message = Some("Loaded next page of keys.".to_string());

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) -> Result<()> {
        // Overall layout: main area and help area
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Min(0), Constraint::Length(3)]);
        let [main_area, help_area] = layout.areas(frame.area());

        // Add left and right panels in main_area
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
        let [left, right] = layout.areas(main_area);

        self.draw_left_menu(frame, left);

        self.draw_help_area(frame, help_area);
        self.draw_details(frame, right);
        Ok(())
    }

    fn draw_left_menu(&mut self, frame: &mut Frame, left: Rect) {
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
    }

    fn draw_help_area(&self, frame: &mut Frame, help_area: Rect) {
        let message = match &self.message {
            Some(msg) => msg.clone(),
            None => format!(
                "{} | q/Esc: Quit | Up/Down: Navigate | d: Delete Key | n: Next Page",
                self.redis_client.url()
            ),
        };

        let details = Paragraph::new(message).block(Block::bordered());
        frame.render_widget(details, help_area);
    }

    fn draw_details(&self, frame: &mut Frame, right: Rect) {
        // Right panel: Details
        let rows = if let Some(index) = self.list_state.selected()
            && let Some(key) = self.keys.get(index)
        {
            let key_type = match self.redis_client.key_type(key) {
                Ok(t) => t,
                Err(e) => format!("Error - {}", e),
            };

            let ttl_info = match self.redis_client.ttl(key) {
                Ok(Some(-1)) => "No expiration".to_string(),
                Ok(Some(ttl)) => format!("{} seconds", ttl),
                Ok(None) => "Key does not exist".to_string(),
                Err(e) => format!("Error - {}", e),
            };

            let value_info = match self.redis_client.get(key) {
                Ok(value) => value,
                Err(e) => format!("Error - {}", e),
            };

            vec![
                Row::new(vec!["Type".to_string(), key_type]),
                Row::new(vec!["TTL".to_string(), ttl_info]),
                Row::new(vec!["Value".to_string(), value_info]),
            ]
        } else {
            vec![]
        };

        let table = Table::new(rows, &[Constraint::Length(10), Constraint::Min(0)])
            .block(Block::bordered().title("Details"))
            .style(Style::default())
            .column_spacing(1);

        frame.render_widget(table, right);
    }
}
