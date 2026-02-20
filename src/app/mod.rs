use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::widgets::{ListState, Row};
use tracing::warn;

use crate::app::insert::{Insert, InsertDataType};

use super::redis::commands::*;
use super::redis::{RedisClient, RedisOps};
use super::tree::Tree;
use super::ui;

mod insert;

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    LeftMenu,
    Details,
}

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
    // Navigation
    pub focus: Focus,
    pub details_table_state: ratatui::widgets::TableState,
    pub stats: Stats,
    // Caches
    pub key_types: BTreeMap<String, String>,
    // Editing
    pub is_editing: bool,
    pub edit_buffer: String,
    pub original_value: String,
    // Insertion
    pub insert: Option<Insert>,
    // Database selection
    pub is_selecting_db: bool,
    pub db_cursor: u8,
    // Help
    pub show_help: bool,
}

#[derive(Debug, Default)]
pub struct Stats {
    pub used_memory: String,
    pub used_cpu: String,
    pub server_name: String,
    pub server_version: String,
    pub total_keys: u64,
}

impl Stats {
    pub fn display(&self, clean_url: &str) -> String {
        format!(
            " {} {}  |  {}  |  Keys: {}  |  Mem: {}  |  CPU: {} ",
            self.server_name.to_uppercase(),
            self.server_version,
            clean_url,
            self.total_keys,
            self.used_memory,
            self.used_cpu
        )
    }
}

impl<R: RedisOps> App<R> {
    pub fn new(redis_client: R) -> Result<Self> {
        let (next, keys) = redis_client.scan("0", "*", 100)?;

        let mut app = Self {
            exit: false,
            redis_client,
            keys: keys.clone(),
            next,
            list_state: ListState::default(),
            message: None,
            loaded_key: None,
            collection_page: 0,
            collection_page_size: 50,
            collection_cursors: vec!["0".to_string()],
            tree: Tree::new(),
            filter_pattern: "*".to_string(),
            search_query: String::new(),
            is_searching: false,
            focus: Focus::LeftMenu,
            details_table_state: ratatui::widgets::TableState::default(),
            stats: Stats::default(),
            key_types: BTreeMap::new(),
            is_editing: false,
            edit_buffer: String::new(),
            original_value: String::new(),
            insert: None,
            is_selecting_db: false,
            db_cursor: 0,
            show_help: false,
        };

        app.update_stats()?;
        app.rebuild_tree();
        Ok(app)
    }

    pub fn update_stats(&mut self) -> Result<()> {
        let info = match self.redis_client.info() {
            Ok(info) => info,
            Err(err) => {
                self.message = Some(format!("Error fetching info: {}", err));
                return Ok(());
            }
        };

        let mut stats = &mut self.stats;
        stats.total_keys = self.redis_client.dbsize().unwrap_or(0);

        for line in info.lines() {
            if line.starts_with("used_memory_human:") {
                stats.used_memory = line.split(':').nth(1).unwrap_or("N/A").to_string();
            } else if line.starts_with("used_cpu_user:") {
                stats.used_cpu = line.split(':').nth(1).unwrap_or("N/A").to_string();
            } else if line.starts_with("redis_version:") {
                stats.server_version = line.split(':').nth(1).unwrap_or("N/A").to_string();
            } else if line.starts_with("valkey_version:") {
                stats.server_name = "Valkey".to_string();
                stats.server_version = line.split(':').nth(1).unwrap_or("N/A").to_string();
            }
        }

        Ok(())
    }

    pub fn rebuild_tree(&mut self) {
        for key in &self.keys {
            if !self.key_types.contains_key(key)
                && let Ok(t) = self.redis_client.key_type(key)
            {
                self.key_types.insert(key.clone(), t);
            }
        }
        self.tree.rebuild(&self.keys, &self.key_types);
    }

    pub fn refresh(&mut self) -> Result<()> {
        // 1. Refresh server stats
        self.update_stats()?;

        // 2. Refresh key list (keep current filter). Clear cache to catch type changes if any.
        self.key_types.clear();
        let (next, keys) = self.redis_client.scan("0", &self.filter_pattern, 100)?;
        self.next = next;
        self.keys = keys;
        self.rebuild_tree();

        // 3. Refresh current selection if any
        if let Some(loaded) = &self.loaded_key {
            let key_name = loaded.key.clone();
            self.fetch_details_for_key(&key_name)?;
        }

        self.message = Some("Data refreshed.".to_string());
        Ok(())
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        tracing::info!("Starting application loop");
        loop {
            terminal.draw(|f| ui::draw(f, self))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
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

        if self.is_editing {
            match key.code {
                KeyCode::Enter => {
                    self.save_edit()?;
                }
                KeyCode::Esc => {
                    self.is_editing = false;
                    self.edit_buffer.clear();
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.edit_buffer.pop();
                }
                _ => {}
            }
            return Ok(());
        }

        if let Some(ins) = &mut self.insert {
            let resp = ins.handle_insertion_key_event(key)?;
            match resp {
                insert::InsertKeyEvent::Noop => {}
                insert::InsertKeyEvent::PerformInsert => {
                    let ins = ins.clone();
                    self.perform_insertion(ins.name, ins.value, ins.insert_type)?;
                    self.insert = None;
                }
                insert::InsertKeyEvent::NotInserting => self.insert = None,
            }
            return Ok(());
        }

        if self.is_selecting_db {
            self.handle_db_selection_key_event(key)?;
            return Ok(());
        }

        if self.show_help {
            self.show_help = false;
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('?') | KeyCode::Char('h') => self.show_help = true,
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            KeyCode::Char('e') => self.start_editing(),
            KeyCode::Char('i') => {
                self.enable_insert_mode();
            }
            KeyCode::Char('a') => {
                if self.focus == Focus::Details
                    && let Some(loaded) = &self.loaded_key
                {
                    let key_type = loaded.key_type.clone();
                    if key_type != "string" {
                        let insert = Insert {
                            step: 2, // Skip name/type, go straight to value
                            name: loaded.key.clone(),
                            insert_type: key_type.as_str().into(),
                            value: String::new(),
                        };
                        self.insert = Some(insert);
                    }
                }
            }
            KeyCode::Char('r') => self.refresh()?,
            KeyCode::Char('b') => {
                self.is_selecting_db = true;
                // self.db_cursor = ... we should probably store current_db in App or get it from client
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::LeftMenu => {
                        // Only switch to details if a key is loaded
                        if self.loaded_key.is_some() {
                            Focus::Details
                        } else {
                            Focus::LeftMenu
                        }
                    }
                    Focus::Details => Focus::LeftMenu,
                };
                // Initialize details table state if moving focus to Details
                if self.focus == Focus::Details && self.details_table_state.selected().is_none() {
                    self.details_table_state.select(Some(0));
                }
            }
            _ => match self.focus {
                Focus::LeftMenu => self.handle_left_menu_key_event(key)?,
                Focus::Details => self.handle_details_key_event(key)?,
            },
        }
        Ok(())
    }

    fn enable_insert_mode(&mut self) {
        self.insert = Some(Insert::default())
    }

    fn perform_insertion(
        &mut self,
        key: String,
        value: String,
        insert_type: InsertDataType,
    ) -> Result<()> {
        match insert_type {
            InsertDataType::String => {
                self.redis_client.set(&key, &value)?;
            }
            InsertDataType::Hash => {
                // Expect "field:value"
                if let Some((field, val)) = value.split_once(':') {
                    self.redis_client.hset(&key, field, val)?;
                } else {
                    self.message = Some("Format for Hash: field:value".to_string());
                    return Ok(());
                }
            }
            InsertDataType::List => {
                self.redis_client.rpush(&key, &value)?;
            }
            InsertDataType::Set => {
                self.redis_client.sadd(&key, &value)?;
            }
            InsertDataType::Zset => {
                // Expect "score:member"
                if let Some((score_str, member)) = value.split_once(':') {
                    if let Ok(score) = score_str.parse::<f64>() {
                        self.redis_client.zadd(&key, score, member)?;
                    } else {
                        self.message = Some("Format for ZSet: score:member".to_string());
                        return Ok(());
                    }
                } else {
                    self.message = Some("Format for ZSet: score:member".to_string());
                    return Ok(());
                }
            }
            _ => {}
        }

        self.message = Some(format!("Inserted into {}", key));
        self.refresh()?;
        Ok(())
    }

    fn handle_left_menu_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Down => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        let len = self.tree.flattened_items.len();
                        if i >= len.saturating_sub(1) { i } else { i + 1 }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                self.handle_selection_change()?;
            }
            KeyCode::Up => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            0
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                self.handle_selection_change()?;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_expanded();
            }
            KeyCode::Char('d') => self.delete_selected_key()?,
            // For now disable pagination of key list via 'n' since we are in tree mode
            KeyCode::Char('n') => self.load_next_page()?,

            KeyCode::Right => {
                if let Some(index) = self.list_state.selected()
                    && let Some((_, is_key, _, is_expanded, _)) =
                        self.tree.flattened_items.get(index)
                {
                    if *is_key {
                        // If it's a key, move focus to Details (only if loaded)
                        if self.loaded_key.is_some() {
                            self.focus = Focus::Details;
                            if self.details_table_state.selected().is_none() {
                                self.details_table_state.select(Some(0));
                            }
                        }
                    } else {
                        // If it's a folder, just expand and stay focused here
                        if !*is_expanded {
                            self.expand_current();
                        }
                    }
                }
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

    fn handle_details_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Down => {
                let i = match self.details_table_state.selected() {
                    Some(i) => {
                        let len = self.get_loaded_collection_length();
                        if i >= len.saturating_sub(1) { i } else { i + 1 }
                    }
                    None => 0,
                };
                self.details_table_state.select(Some(i));
            }
            KeyCode::Up => {
                let i = match self.details_table_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            0
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.details_table_state.select(Some(i));
            }
            KeyCode::Left | KeyCode::BackTab => {
                self.focus = Focus::LeftMenu;
            }
            KeyCode::Char('d') => self.delete_collection_item()?,
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
        Ok(())
    }

    fn get_loaded_collection_length(&self) -> usize {
        if let Some(data) = &self.loaded_key {
            match &data.content {
                CollectionData::List(items) => items.len(),
                CollectionData::Hash(fields) => fields.len(),
                CollectionData::Set(members) => members.len(),
                CollectionData::ZSet(items) => items.len(),
                _ => 0,
            }
        } else {
            0
        }
    }

    fn delete_collection_item(&mut self) -> Result<()> {
        let Some(index) = self.details_table_state.selected() else {
            return Ok(());
        };
        let Some(loaded) = &self.loaded_key else {
            return Ok(());
        };

        let key = loaded.key.clone();
        match &loaded.content {
            CollectionData::Hash(fields) => {
                if let Some((field, _)) = fields.get(index) {
                    self.redis_client.hdel(&key, field)?;
                }
            }
            CollectionData::List(items) => {
                if let Some(value) = items.get(index) {
                    self.redis_client.lrem(&key, 1, value)?;
                }
            }
            CollectionData::Set(members) => {
                if let Some(member) = members.get(index) {
                    self.redis_client.srem(&key, member)?;
                }
            }
            CollectionData::ZSet(items) => {
                if let Some((member, _)) = items.get(index) {
                    self.redis_client.zrem(&key, member)?;
                }
            }
            _ => return Ok(()),
        }

        // Reload data
        self.fetch_details_for_key(&key.clone())?;

        // Adjust selection
        let new_len = self.get_loaded_collection_length();
        if new_len == 0 {
            self.details_table_state.select(None);
        } else if index >= new_len {
            self.details_table_state.select(Some(new_len - 1));
        }

        Ok(())
    }

    fn start_editing(&mut self) {
        if let Some(loaded) = &self.loaded_key {
            match &loaded.content {
                CollectionData::String(val, _) => {
                    self.is_editing = true;
                    self.edit_buffer = val.clone();
                    self.original_value = val.clone();
                }
                CollectionData::Hash(fields) => {
                    if let Some(index) = self.details_table_state.selected()
                        && let Some((_, val)) = fields.get(index)
                    {
                        self.is_editing = true;
                        self.edit_buffer = val.clone();
                        self.original_value = val.clone();
                    }
                }
                CollectionData::List(items) => {
                    if let Some(index) = self.details_table_state.selected()
                        && let Some(val) = items.get(index)
                    {
                        self.is_editing = true;
                        self.edit_buffer = val.clone();
                        self.original_value = val.clone();
                    }
                }
                CollectionData::Set(members) => {
                    if let Some(index) = self.details_table_state.selected()
                        && let Some(val) = members.get(index)
                    {
                        self.is_editing = true;
                        self.edit_buffer = val.clone();
                        self.original_value = val.clone();
                    }
                }
                CollectionData::ZSet(items) => {
                    if let Some(index) = self.details_table_state.selected()
                        && let Some((val, _)) = items.get(index)
                    {
                        self.is_editing = true;
                        self.edit_buffer = val.clone();
                        self.original_value = val.clone();
                    }
                }
                _ => {
                    self.message = Some("Editing not supported for this type yet".to_string());
                }
            }
        }
    }

    fn save_edit(&mut self) -> Result<()> {
        let Some(loaded) = &self.loaded_key else {
            self.is_editing = false;
            return Ok(());
        };

        let key = loaded.key.clone();
        let new_value = self.edit_buffer.clone();

        match &loaded.content {
            CollectionData::String(_, _) => {
                self.redis_client.set(&key, &new_value)?;
                self.message = Some(format!("Updated string: {}", key));
            }
            CollectionData::Hash(fields) => {
                if let Some(index) = self.details_table_state.selected()
                    && let Some((field, _)) = fields.get(index)
                {
                    self.redis_client.hset(&key, field, &new_value)?;
                    self.message = Some(format!("Updated hash field: {}", field));
                }
            }
            CollectionData::List(_) => {
                if let Some(index) = self.details_table_state.selected() {
                    let list_index =
                        (self.collection_page * self.collection_page_size + index) as i64;
                    self.redis_client.lset(&key, list_index, &new_value)?;
                    self.message = Some(format!("Updated list item at index {}", list_index));
                }
            }
            CollectionData::Set(_) => {
                self.redis_client.srem(&key, &self.original_value)?;
                self.redis_client.sadd(&key, &new_value)?;
                self.message = Some("Updated set member".to_string());
            }
            CollectionData::ZSet(items) => {
                if let Some(index) = self.details_table_state.selected()
                    && let Some((_, score)) = items.get(index)
                {
                    self.redis_client.zrem(&key, &self.original_value)?;
                    self.redis_client.zadd(&key, *score, &new_value)?;
                    self.message = Some("Updated sorted set member".to_string());
                }
            }
            _ => {}
        }

        self.is_editing = false;
        self.edit_buffer.clear();
        self.fetch_details_for_key(&key)?;
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
        if let Some(index) = self.list_state.selected()
            && let Some(path) = self.tree.flattened_paths.get(index).cloned()
        {
            // Find node and toggle
            self.tree.toggle_expansion(&path);
        }
    }

    fn expand_current(&mut self) {
        if let Some(index) = self.list_state.selected()
            && let Some(path) = self.tree.flattened_paths.get(index).cloned()
        {
            self.tree.set_expansion(&path, true);
        }
    }

    fn collapse_current(&mut self) {
        if let Some(index) = self.list_state.selected()
            && let Some(path) = self.tree.flattened_paths.get(index).cloned()
        {
            self.tree.set_expansion(&path, false);
        }
    }

    pub fn fetch_details_for_key(&mut self, key: &str) -> Result<()> {
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
            "string" => self.fetch_string_content(&key),
            "list" => self.fetch_list_content(&key),
            "hash" => self.fetch_hash_content(&key),
            "set" => self.fetch_set_content(&key),
            "zset" => self.fetch_zset_content(&key),
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

    fn fetch_string_content(&self, key: &str) -> (i64, CollectionData) {
        let val = self.redis_client.get(key).unwrap_or_else(|e| e.to_string());
        let len = self.redis_client.strlen(key).unwrap_or(0);
        (len, CollectionData::String(val, len as usize))
    }

    fn fetch_list_content(&self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.llen(key).unwrap_or(0);
        let (start, stop) = page_range_i64(self.collection_page, self.collection_page_size);
        let items = self
            .redis_client
            .lrange(key, start, stop)
            .unwrap_or_default();
        (len, CollectionData::List(items))
    }

    fn fetch_hash_content(&mut self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.hlen(key).unwrap_or(0);
        let cursor = self.get_current_cursor();
        let (next_cursor, items) = self
            .redis_client
            .hscan(key, cursor, self.collection_page_size)
            .unwrap_or(("0".to_string(), vec![]));

        self.update_next_cursor(next_cursor);
        (len, CollectionData::Hash(items))
    }

    fn fetch_set_content(&mut self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.scard(key).unwrap_or(0);
        let cursor = self.get_current_cursor();
        let (next_cursor, items) = self
            .redis_client
            .sscan(key, cursor, self.collection_page_size)
            .unwrap_or(("0".to_string(), vec![]));

        self.update_next_cursor(next_cursor);
        (len, CollectionData::Set(items))
    }

    fn fetch_zset_content(&self, key: &str) -> (i64, CollectionData) {
        let len = self.redis_client.zcard(key).unwrap_or(0);
        let (start, stop) = page_range_i64(self.collection_page, self.collection_page_size);
        let items = self
            .redis_client
            .zrange_with_scores(key, start, stop)
            .unwrap_or_default();
        (len, CollectionData::ZSet(items))
    }

    fn delete_selected_key(&mut self) -> Result<()> {
        if let Some(index) = self.list_state.selected()
            && let Some(path) = self.tree.flattened_paths.get(index).cloned()
        {
            let (_, is_key, _, _, _) = self.tree.flattened_items[index];
            if is_key {
                self.redis_client.del(&path)?;
                self.message = Some(format!("Deleted key: {}", path));

                // Helper functionality to remove key from tree without full scan?
                // For now, full rebuild is safer and easier.
                if let Some(pos) = self.keys.iter().position(|k| *k == path) {
                    self.keys.remove(pos);
                }
                self.key_types.remove(&path);
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

        if let Some((_, is_key, _, _, _)) = self.tree.flattened_items.get(index)
            && *is_key
        {
            self.fetch_details_for_key(&path)?;
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
                "{} | r: Refresh | q: Quit | ↑↓: Navigate | d: Delete | n: Next Keys | ←→: Page",
                self.redis_client.url()
            ),
        }
    }

    fn handle_db_selection_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                self.confirm_db_selection()?;
            }
            KeyCode::Esc => {
                self.is_selecting_db = false;
            }
            KeyCode::Up => {
                self.db_cursor = self.db_cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.db_cursor < 15 {
                    self.db_cursor += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn confirm_db_selection(&mut self) -> Result<()> {
        let new_db = self.db_cursor;
        self.redis_client.select_db(new_db)?;
        self.is_selecting_db = false;

        // Refresh everything
        self.keys.clear();
        self.key_types.clear();
        self.loaded_key = None;
        let (next, keys) = self.redis_client.scan("0", &self.filter_pattern, 100)?;
        self.next = next;
        self.keys = keys;
        self.rebuild_tree();
        self.update_stats()?;

        self.message = Some(format!("Switched to Database {}", new_db));
        Ok(())
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
