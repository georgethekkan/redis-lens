use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::widgets::{ListState, Row};

use super::redis::commands::*;
use super::redis::{RedisClient, RedisOps};
use super::tree::Tree;
use super::ui;

use std::collections::BTreeMap;

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

pub struct App<R: RedisOps> {
    exit: bool,
    pub redis_client: R,
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
    // Tree view
    pub tree: Tree,
    // Search & Filtering
    pub filter_pattern: String,
    pub search_query: String,
    pub is_searching: bool,
}

impl<R: RedisOps> App<R> {
    pub fn new(redis_client: R) -> Result<Self> {
        let (next, keys) = redis_client.scan("0", "*", 100)?;
        let message = None;

        let mut app = Self {
            exit: false,
            redis_client,
            keys: keys.clone(),
            next,
            list_state: ListState::default(),
            message,
            loaded_key: None,
            collection_page: 0,
            collection_page_size: 50,
            collection_cursors: vec!["0".to_string()],
            tree: Tree::new(),
            filter_pattern: "*".to_string(),
            search_query: String::new(),
            is_searching: false,
        };

        app.rebuild_tree();
        Ok(app)
    }

    pub fn rebuild_tree(&mut self) {
        let mut types = BTreeMap::new();
        for key in &self.keys {
            if let Ok(t) = self.redis_client.key_type(key) {
                types.insert(key.clone(), t);
            }
        }
        self.tree.rebuild(&self.keys, &types);
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        tracing::info!("Starting application loop");
        loop {
            terminal.draw(|f| ui::draw(f, self))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    tracing::debug!("Key event: {:?}", key);
                    self.handle_key_event(key)?;
                }
            }

            if self.exit {
                tracing::info!("Exiting application");
                break;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.is_searching {
            match key.code {
                KeyCode::Enter => {
                    self.confirm_search()?;
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.search_query.clear();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            KeyCode::Down => {
                self.list_state.select_next();
                self.handle_selection_change()?;
            }
            KeyCode::Up => {
                self.list_state.select_previous();
                self.handle_selection_change()?;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_expanded();
            }
            KeyCode::Char('d') => self.delete_selected_key()?,
            // For now disable pagination of key list via 'n' since we are in tree mode
            KeyCode::Char('n') => self.load_next_page()?,

            KeyCode::Right => {
                // If folder and collapsed, expand.
                self.expand_current();
            }
            KeyCode::Left => {
                // If folder and expanded, collapse.
                self.collapse_current();
            }

            _ => {
                // Existing collection pagination logic
                match key.code {
                    KeyCode::Char('l') => {
                        self.next_collection_page();
                        self.fetch_selected_key_details()?;
                    }
                    KeyCode::Char('h') => {
                        self.prev_collection_page();
                        self.fetch_selected_key_details()?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn confirm_search(&mut self) -> Result<()> {
        self.is_searching = false;

        let pattern = if self.search_query.is_empty() {
            "*".to_string()
        } else if self.search_query.contains('*') {
            self.search_query.clone()
        } else {
            format!("{}*", self.search_query)
        };

        self.filter_pattern = pattern;

        // Reset keys and performing scan
        let (next, keys) = self.redis_client.scan("0", &self.filter_pattern, 100)?;
        self.next = next;
        self.keys = keys;

        // Reset selection and rebuild tree
        self.list_state.select(None);
        self.loaded_key = None;
        self.rebuild_tree();

        self.message = Some(format!("Searching for: {}", self.filter_pattern));

        Ok(())
    }

    fn handle_selection_change(&mut self) -> Result<()> {
        // Check if selected item is a key
        let Some(index) = self.list_state.selected() else {
            self.loaded_key = None;
            return Ok(());
        };

        if let Some((_, is_key, _, _, _)) = self.tree.flattened_items.get(index) {
            if *is_key {
                // Fetch details
                // We need the full path
                if let Some(path) = self.tree.flattened_paths.get(index).cloned() {
                    // We need to set up 'loaded_key' based on this path
                    self.fetch_details_for_key(&path)?;
                }
            } else {
                self.loaded_key = None;
            }
        }
        Ok(())
    }

    fn toggle_expanded(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(path) = self.tree.flattened_paths.get(index).cloned() {
                // Find node and toggle
                self.tree.toggle_expansion(&path);
            }
        }
    }

    fn expand_current(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(path) = self.tree.flattened_paths.get(index).cloned() {
                self.tree.set_expansion(&path, true);
            }
        }
    }

    fn collapse_current(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(path) = self.tree.flattened_paths.get(index).cloned() {
                self.tree.set_expansion(&path, false);
            }
        }
    }

    pub fn fetch_details_for_key(&mut self, key: &str) -> Result<()> {
        // Logic previously in fetch_selected_key_details, but taking key arg
        // Reuse code by extracting common logic or just copy-paste with adjustments for now to avoid massive refactor risk
        // Actually, let's keep `fetch_selected_key_details` as the method that uses `list_state`?
        // No, because list_state now points to a tree node, which might not correspond to `self.keys` index.
        // So we need a new method `fetch_details_for_key`.

        let key = key.to_string();

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

    fn delete_selected_key(&mut self) -> Result<()> {
        if let Some(index) = self.list_state.selected() {
            if let Some(path) = self.tree.flattened_paths.get(index).cloned() {
                let (_, is_key, _, _, _) = self.tree.flattened_items[index];
                if is_key {
                    self.redis_client.del(&path)?;
                    self.message = Some(format!("Deleted key: {}", path));

                    // Helper functionality to remove key from tree without full scan?
                    // For now, full rebuild is safer and easier.
                    if let Some(pos) = self.keys.iter().position(|k| *k == path) {
                        self.keys.remove(pos);
                    }
                    self.rebuild_tree();

                    // Reset selection?? Or try to keep index?
                    // If index exists in new tree, fine.
                    if index >= self.tree.flattened_items.len() {
                        self.list_state.select(None);
                    }
                    self.loaded_key = None;
                } else {
                    self.message = Some("Cannot delete folder yet".to_string());
                }
            }
        }
        Ok(())
    }

    fn load_next_page(&mut self) -> Result<()> {
        if self.next == "0" {
            return Ok(());
        }
        let (new_cursor, new_keys) =
            self.redis_client
                .scan(&self.next, &self.filter_pattern, 100)?;
        self.next = new_cursor;
        self.keys.extend(new_keys);
        self.message = Some("Loaded next page of keys.".to_string());

        // Rebuild tree with new keys
        self.rebuild_tree();

        Ok(())
    }

    pub fn fetch_selected_key_details(&mut self) -> Result<()> {
        let Some(index) = self.list_state.selected() else {
            self.loaded_key = None;
            return Ok(());
        };

        let path = if let Some(p) = self.tree.flattened_paths.get(index) {
            p.clone()
        } else {
            self.loaded_key = None;
            return Ok(());
        };

        if let Some((_, is_key, _, _, _)) = self.tree.flattened_items.get(index) {
            if *is_key {
                self.fetch_details_for_key(&path)?;
            }
        }
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
                self.redis_client.url()
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
