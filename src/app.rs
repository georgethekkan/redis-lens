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

#[derive(Clone, Debug)]
pub enum CollectionData {
    String(String),
    List(Vec<String>),
    Hash(Vec<(String, String)>),
    Set(Vec<String>),
    ZSet(Vec<(String, f64)>),
}

pub struct App {
    exit: bool,
    redis_client: RedisClient,
    keys: Vec<String>,
    next: String,
    list_state: ListState,
    message: Option<String>,
    // Collection pagination
    collection_page: usize,
    collection_page_size: usize,
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
            collection_page: 0,
            collection_page_size: 50,
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
            KeyCode::Right => self.next_collection_page(),
            KeyCode::Left => self.prev_collection_page(),
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

    fn next_collection_page(&mut self) {
        self.collection_page += 1;
    }

    fn prev_collection_page(&mut self) {
        if self.collection_page > 0 {
            self.collection_page -= 1;
        }
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
                "{} | q: Quit | ↑↓: Navigate | d: Delete | n: Next Keys | ←→: Page Collection",
                self.redis_client.url()
            ),
        };

        let details = Paragraph::new(message).block(Block::bordered());
        frame.render_widget(details, help_area);
    }

    fn draw_details(&self, frame: &mut Frame, right: Rect) {
        // Right panel: Details based on data type
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

            let mut rows = vec![
                Row::new(vec!["Key".to_string(), key.clone()]),
                Row::new(vec!["Type".to_string(), key_type.clone()]),
                Row::new(vec!["TTL".to_string(), ttl_info]),
            ];

            // Type-specific data
            match key_type.as_str() {
                "string" => {
                    let value_info = match self.redis_client.get(key) {
                        Ok(value) => {
                            let len = match self.redis_client.strlen(key) {
                                Ok(l) => l,
                                Err(_) => 0,
                            };
                            format!("{} ({} bytes)", value, len)
                        }
                        Err(e) => format!("Error - {}", e),
                    };
                    rows.push(Row::new(vec!["Value".to_string(), value_info]));
                }
                "list" => {
                    match self.redis_client.llen(key) {
                        Ok(len) => {
                            rows.push(Row::new(vec!["Length".to_string(), len.to_string()]));
                            // Get paginated items
                            let start = (self.collection_page * self.collection_page_size) as i64;
                            let stop = start + self.collection_page_size as i64 - 1;
                            if let Ok(items) = self.redis_client.lrange(key, start, stop) {
                                for (i, item) in items.iter().enumerate() {
                                    let idx = start as usize + i;
                                    rows.push(Row::new(vec![
                                        format!("[{}]", idx),
                                        item.clone(),
                                    ]));
                                }
                            }
                        }
                        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
                    }
                }
                "hash" => {
                    match self.redis_client.hlen(key) {
                        Ok(len) => {
                            rows.push(Row::new(vec!["Fields".to_string(), len.to_string()]));
                            // Get all hash data and paginate
                            if let Ok(data) = self.redis_client.hgetall(key) {
                                let start = self.collection_page * self.collection_page_size;
                                let end = std::cmp::min(start + self.collection_page_size, data.len());
                                for (field, value) in &data[start..end] {
                                    rows.push(Row::new(vec![field.clone(), value.clone()]));
                                }
                            }
                        }
                        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
                    }
                }
                "set" => {
                    match self.redis_client.scard(key) {
                        Ok(count) => {
                            rows.push(Row::new(vec!["Members".to_string(), count.to_string()]));
                            // Get all members and paginate
                            if let Ok(members) = self.redis_client.smembers(key) {
                                let start = self.collection_page * self.collection_page_size;
                                let end = std::cmp::min(start + self.collection_page_size, members.len());
                                for member in &members[start..end] {
                                    rows.push(Row::new(vec!["Member".to_string(), member.clone()]));
                                }
                            }
                        }
                        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
                    }
                }
                "zset" => {
                    match self.redis_client.zcard(key) {
                        Ok(count) => {
                            rows.push(Row::new(vec!["Members".to_string(), count.to_string()]));
                            // Get paginated items with scores
                            let start = (self.collection_page * self.collection_page_size) as i64;
                            let stop = start + self.collection_page_size as i64 - 1;
                            if let Ok(items) = self.redis_client.zrange_with_scores(key, start, stop) {
                                for (member, score) in items {
                                    rows.push(Row::new(vec![
                                        member,
                                        format!("{:.2}", score),
                                    ]));
                                }
                            }
                        }
                        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
                    }
                }
                _ => {
                    rows.push(Row::new(vec!["Status".to_string(), "Unknown type".to_string()]));
                }
            }

            rows
        } else {
            vec![]
        };

        let table = Table::new(rows, &[Constraint::Length(12), Constraint::Min(0)])
            .block(Block::bordered().title("Details"))
            .style(Style::default())
            .column_spacing(1);

        frame.render_widget(table, right);
    }
}
