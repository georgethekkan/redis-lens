use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::widgets::{ListState, Row};

use super::redis::RedisClient;
use super::redis::commands::*;
use super::ui;

#[derive(Clone, Debug)]
pub enum CollectionData {
    String(String, usize), // Value, length (bytes)
    List(Vec<String>),
    Hash(Vec<(String, String)>),
    Set(Vec<String>),
    ZSet(Vec<(String, f64)>),
    None,
}

#[derive(Clone, Debug)]
pub struct LoadedKeyData {
    pub key: String,
    pub key_type: String,
    pub ttl: String,
    pub length: i64,
    pub content: CollectionData,
}

pub struct App {
    exit: bool,
    pub redis_client: RedisClient,
    pub keys: Vec<String>,
    pub next: String,
    pub list_state: ListState,
    pub message: Option<String>,
    // Key Details
    pub loaded_key: Option<LoadedKeyData>,
    // Collection pagination
    pub collection_page: usize,
    pub collection_page_size: usize,
    pub collection_cursors: Vec<String>,
}

impl App {
    pub fn new(redis_client: RedisClient) -> Result<Self> {
        let (next, keys) = redis_client.scan("0", "*", 100)?;
        let message = None;

        let app = Self {
            exit: false,
            redis_client,
            keys,
            next,
            list_state: ListState::default(),
            message,
            loaded_key: None,
            collection_page: 0,
            collection_page_size: 50,
            collection_cursors: vec!["0".to_string()],
        };

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        tracing::info!("Starting application loop");
        loop {
            terminal.draw(|f| ui::draw(f, self))?;

            if let Event::Key(key) = event::read()? {
                tracing::debug!("Key event: {:?}", key);
                self.handle_key_event(key)?;
            }

            if self.exit {
                tracing::info!("Exiting application");
                break;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Down => {
                self.list_state.select_next();
                self.fetch_selected_key_details()?;
            }
            KeyCode::Up => {
                self.list_state.select_previous();
                self.fetch_selected_key_details()?;
            }
            KeyCode::Char('d') => self.delete_selected_key()?,
            KeyCode::Char('n') => {
                self.load_next_page()?;
                // If selection was out of bounds or None, it might be valid now?
                // But usually 'n' adds keys to end.
            }
            KeyCode::Right => {
                self.next_collection_page();
                self.fetch_selected_key_details()?;
            }
            KeyCode::Left => {
                self.prev_collection_page();
                self.fetch_selected_key_details()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn delete_selected_key(&mut self) -> Result<()> {
        let Some(index) = self.list_state.selected() else {
            return Ok(());
        };
        let Some(key) = self.keys.get(index) else {
            return Ok(());
        };

        self.redis_client.del(key)?;
        self.message = Some(format!("Deleted key: {}", key));
        self.keys.remove(index);

        // Adjust selection
        if self.keys.is_empty() {
            self.list_state.select(None);
        } else if index >= self.keys.len() {
            self.list_state.select(Some(self.keys.len() - 1));
        }

        // Refresh details
        self.fetch_selected_key_details()?;

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

    pub fn fetch_selected_key_details(&mut self) -> Result<()> {
        let Some(index) = self.list_state.selected() else {
            self.loaded_key = None;
            return Ok(());
        };
        let Some(key) = self.keys.get(index) else {
            self.loaded_key = None;
            return Ok(());
        };
        let key = key.clone();

        // 1. Get Type
        let key_type = self
            .redis_client
            .key_type(&key)
            .unwrap_or_else(|e| format!("Error - {}", e));

        // 2. Get TTL
        let ttl_info = match self.redis_client.ttl(&key) {
            Ok(Some(-1)) => "No expiration".to_string(),
            Ok(Some(ttl)) => format!("{} seconds", ttl),
            Ok(None) => "Key does not exist".to_string(),
            Err(e) => format!("Error - {}", e),
        };

        // 3. Get Content & Length based on type
        let (length, content) = match key_type.as_str() {
            "string" => {
                let val = self
                    .redis_client
                    .get(&key)
                    .unwrap_or_else(|e| e.to_string());
                let len = self.redis_client.strlen(&key).unwrap_or(0);
                (len, CollectionData::String(val, len as usize))
            }
            "list" => {
                let len = self.redis_client.llen(&key).unwrap_or(0);
                let (start, stop) = page_range_i64(self.collection_page, self.collection_page_size);
                let items = self
                    .redis_client
                    .lrange(&key, start, stop)
                    .unwrap_or_default();
                (len, CollectionData::List(items))
            }
            "hash" => {
                let len = self.redis_client.hlen(&key).unwrap_or(0);
                let cursor = self.get_current_cursor();
                // hscan returns (next_cursor, items)
                let (next_cursor, items) = self
                    .redis_client
                    .hscan(&key, cursor, self.collection_page_size)
                    .unwrap_or(("0".to_string(), vec![]));

                self.update_next_cursor(next_cursor);
                (len, CollectionData::Hash(items))
            }
            "set" => {
                let len = self.redis_client.scard(&key).unwrap_or(0);
                let cursor = self.get_current_cursor();
                let (next_cursor, items) = self
                    .redis_client
                    .sscan(&key, cursor, self.collection_page_size)
                    .unwrap_or(("0".to_string(), vec![]));

                self.update_next_cursor(next_cursor);
                (len, CollectionData::Set(items))
            }
            "zset" => {
                let len = self.redis_client.zcard(&key).unwrap_or(0);
                let (start, stop) = page_range_i64(self.collection_page, self.collection_page_size);
                let items = self
                    .redis_client
                    .zrange_with_scores(&key, start, stop)
                    .unwrap_or_default();
                (len, CollectionData::ZSet(items))
            }
            _ => (0, CollectionData::None),
        };

        self.loaded_key = Some(LoadedKeyData {
            key,
            key_type,
            ttl: ttl_info,
            length,
            content,
        });

        Ok(())
    }

    fn next_collection_page(&mut self) {
        // Only advance if we have a valid next cursor for Scans, or just index for List/ZSet
        // Logic: If list/zset, just increment.
        // If hash/set, check if we have a next cursor available (populated by update_next_cursor).

        let should_advance = if let Some(data) = &self.loaded_key {
            match data.key_type.as_str() {
                "hash" | "set" => {
                    // We can advance if collection_cursors has an entry for collection_page + 1
                    self.collection_cursors.len() > self.collection_page + 1
                }
                _ => true, // List/ZSet can always try to advance (bounds checked in fetch)
            }
        } else {
            false
        };

        if should_advance {
            self.collection_page += 1;
        }
    }

    fn prev_collection_page(&mut self) {
        if self.collection_page > 0 {
            self.collection_page -= 1;
        }
    }

    fn get_current_cursor(&self) -> &str {
        if self.collection_page < self.collection_cursors.len() {
            &self.collection_cursors[self.collection_page]
        } else {
            "0"
        }
    }

    fn update_next_cursor(&mut self, next_cursor: String) {
        // If we received a cursor (even "0"), we should ensure logic allows exploring it.
        // If next_cursor is "0", it means end of iteration.
        // If we are at page X, and next_cursor is Y. We should set cursors[X+1] = Y.

        if next_cursor == "0" {
            // End of iteration.
            // Ensure we don't have extra cursors if we re-scanned?
            // Truncate?
            if self.collection_cursors.len() > self.collection_page + 1 {
                self.collection_cursors.truncate(self.collection_page + 1);
            }
            return;
        }

        if self.collection_page + 1 < self.collection_cursors.len() {
            self.collection_cursors[self.collection_page + 1] = next_cursor;
        } else {
            self.collection_cursors.push(next_cursor);
        }
    }

    // Reset pagination when selecting a new key
    pub fn reset_collection_pagination(&mut self) {
        self.collection_page = 0;
        self.collection_cursors = vec!["0".to_string()];
    }

    pub fn help_message(&self) -> String {
        match &self.message {
            Some(msg) => msg.clone(),
            None => format!(
                "{} | q: Quit | ↑↓: Navigate | d: Delete | n: Next Keys | ←→: Page Collection",
                self.redis_client.url
            ),
        }
    }
}

// Pagination helpers
fn page_range_i64(page: usize, page_size: usize) -> (i64, i64) {
    let start = (page.saturating_mul(page_size)) as i64;
    let stop = start + page_size as i64 - 1;
    (start, stop)
}

fn slice_bounds(total: usize, page: usize, page_size: usize) -> (usize, usize) {
    let start = page.saturating_mul(page_size);
    let end = std::cmp::min(start + page_size, total);
    (start, end)
}
