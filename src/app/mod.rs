pub mod actions;
pub mod events;
pub mod fetch;
pub mod insert;

use crate::{
    redis::{ClientOps, DataType},
    tree::Tree,
    ui,
};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
pub use insert::Insert;
use ratatui::{
    DefaultTerminal,
    widgets::{ListState, TableState},
};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    LeftMenu,
    Details,
}

#[derive(Debug, Default, Clone)]
pub struct AppStats {
    pub total_keys: usize,
    pub used_memory: String,
}

impl AppStats {
    pub fn display(&self, url: &str) -> String {
        format!(
            "{} | Keys: {} | Memory: {}",
            url, self.total_keys, self.used_memory
        )
    }
}

#[derive(Debug, Clone)]
pub struct LoadedKeyData {
    pub key: String,
    pub data_type: DataType,
    pub ttl: String,
    pub length: i64,
    pub content: Data,
}

#[derive(Debug, Clone)]
pub enum Data {
    String(String, usize),
    Hash(Vec<(String, String)>),
    List(Vec<String>),
    Set(Vec<String>),
    ZSet(Vec<(String, f64)>),
    None,
}

#[derive(Debug, Clone)]
pub struct Editing {
    pub buffer: String,
    pub original: String,
}

impl Editing {
    pub fn new(buffer: String, original: String) -> Self {
        Self { buffer, original }
    }
}

pub struct App<R: ClientOps> {
    pub client: R,
    pub keys: Vec<String>,
    pub key_types: HashMap<String, String>,
    pub list_state: ListState,
    pub details_table_state: TableState,
    pub stats: AppStats,
    pub message: Option<String>,
    pub exit: bool,
    pub filter_pattern: String,
    pub next: String,
    pub loaded_key: Option<LoadedKeyData>,
    pub focus: Focus,
    pub show_help: bool,

    pub editing: Option<Editing>,
    pub insert: Option<Insert>,

    // Search
    pub is_searching: bool,
    pub search_query: String,

    // DB Selection
    pub is_selecting_db: bool,
    pub db_cursor: usize,

    // Pagination
    pub collection_page: usize,
    pub collection_page_size: usize,
    pub collection_cursors: Vec<String>, // For Scans

    // Tree View
    pub tree: Tree,
}

impl<R: ClientOps> App<R> {
    pub fn new(redis_client: R) -> Result<Self> {
        let mut app = Self {
            client: redis_client,
            keys: Vec::new(),
            key_types: HashMap::new(),
            list_state: ListState::default(),
            details_table_state: TableState::default(),
            stats: AppStats::default(),
            message: None,
            exit: false,
            filter_pattern: "*".to_string(),
            next: "0".to_string(),
            loaded_key: None,
            focus: Focus::LeftMenu,
            show_help: false,
            editing: None,
            insert: None,
            is_searching: false,
            search_query: String::new(),
            is_selecting_db: false,
            db_cursor: 0,
            collection_page: 0,
            collection_page_size: 50,
            collection_cursors: vec!["0".to_string()],
            tree: Tree::new(),
        };

        app.refresh()?;
        Ok(app)
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

    pub fn refresh(&mut self) -> Result<()> {
        let resp = self.client.scan("0", &self.filter_pattern, 100)?;
        self.next = resp.next;
        self.keys = resp.keys;

        self.rebuild_tree();

        // Refresh current selection if any
        if let Some(loaded) = &self.loaded_key {
            let key_name = loaded.key.clone();
            self.fetch_details_for_key(&key_name)?;
        }

        self.message = Some("Data refreshed.".to_string());
        Ok(())
    }

    pub fn rebuild_tree(&mut self) {
        // For now, Tree::rebuild needs types. We can pass an empty BTreeMap if we don't care about types in the tree view for now,
        // or convert App's key_types.
        let types: BTreeMap<String, String> = self
            .key_types
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        self.tree.rebuild(&self.keys, &types);
    }

    pub fn enable_insert_mode(&mut self) {
        self.insert = Some(Insert::default())
    }

    pub(crate) fn move_focus_to_details(&mut self) {
        if self.loaded_key.is_none() {
            return;
        }
        self.focus = Focus::Details;
        if self.details_table_state.selected().is_none() {
            self.details_table_state.select(Some(0));
        }
    }

    pub fn reset_collection_pagination(&mut self) {
        self.collection_page = 0;
        self.collection_cursors = vec!["0".to_string()];
    }

    pub(crate) fn get_current_cursor(&self) -> &str {
        if self.collection_page < self.collection_cursors.len() {
            &self.collection_cursors[self.collection_page]
        } else {
            "0"
        }
    }

    pub(crate) fn update_next_cursor(&mut self, next_cursor: String) {
        if next_cursor == "0" {
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

    pub fn get_loaded_collection_length(&self) -> usize {
        let Some(data) = &self.loaded_key else {
            return 0;
        };
        match &data.content {
            Data::List(items) => items.len(),
            Data::Hash(fields) => fields.len(),
            Data::Set(members) => members.len(),
            Data::ZSet(items) => items.len(),
            _ => 0,
        }
    }

    pub fn help_message(&self) -> String {
        match &self.message {
            Some(msg) => msg.clone(),
            None => format!(
                "{} | r: Refresh | q: Quit | ↑↓: Navigate | d: Delete | n: Next Keys | ←→: Page",
                self.client.url()
            ),
        }
    }

    pub fn update_stats(&mut self) -> Result<()> {
        let Ok(info) = self.client.info(Some("memory")) else {
            return Ok(());
        };
        for line in info.lines() {
            if line.starts_with("used_memory_human:") {
                self.stats.used_memory = line.trim_start_matches("used_memory_human:").to_string();
                break;
            }
        }

        if let Ok(count) = self.client.dbsize() {
            self.stats.total_keys = count as usize;
        }

        Ok(())
    }
}
