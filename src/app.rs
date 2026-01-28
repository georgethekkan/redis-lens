use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::widgets::{ListState, Row};

use super::redis::RedisClient;
use super::ui;

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
    pub redis_client: RedisClient,
    pub keys: Vec<String>,
    pub next: String,
    pub list_state: ListState,
    pub message: Option<String>,
    // Collection pagination
    pub collection_page: usize,
    pub collection_page_size: usize,
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
            collection_page: 0,
            collection_page_size: 50,
        };

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, self))?;

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
        let Some(index) = self.list_state.selected() else {
            return Ok(());
        };
        let Some(key) = self.keys.get(index) else {
            return Ok(());
        };

        self.redis_client.del(key)?;
        self.message = Some(format!("Deleted key: {}", key));
        self.keys.remove(index);
        if self.keys.is_empty() {
            self.list_state.select(None);
        } else if index >= self.keys.len() {
            self.list_state.select(Some(self.keys.len() - 1));
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
