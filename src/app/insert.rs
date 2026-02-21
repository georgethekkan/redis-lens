use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Default, Clone)]
pub struct Insert {
    pub name: String,
    pub value: String,
    pub insert_type: InsertDataType, // "string", "hash", "list", "set", "zset"
    pub step: usize,                 // 0: Name, 1: Type, 2: Value/Field
}

#[derive(Debug, Clone, Default)]
pub enum InsertDataType {
    #[default]
    None,
    String,
    Hash,
    List,
    Set,
    Zset,
}

pub enum InsertKeyEvent {
    Noop,
    PerformInsert, //clear too
    NotInserting,
}

impl InsertDataType {
    pub fn as_str(&self) -> &str {
        match self {
            InsertDataType::String => "string",
            InsertDataType::Hash => "hash",
            InsertDataType::List => "list",
            InsertDataType::Set => "set",
            InsertDataType::Zset => "zset",
            InsertDataType::None => "",
        }
    }
}

impl From<char> for InsertDataType {
    fn from(c: char) -> Self {
        // Type selection - keep it simple: s: string, h: hash, l: list, e: set, z: zset

        match c {
            's' => InsertDataType::String,
            'h' => InsertDataType::Hash,
            'l' => InsertDataType::List,
            'e' => InsertDataType::Set,
            'z' => InsertDataType::Zset,
            _ => InsertDataType::None,
        }
    }
}

impl From<&str> for InsertDataType {
    fn from(c: &str) -> Self {
        // Type selection - keep it simple: s: string, h: hash, l: list, e: set, z: zset

        match c.to_lowercase().as_str() {
            "string" => InsertDataType::String,
            "hash" => InsertDataType::Hash,
            "list" => InsertDataType::List,
            "set" => InsertDataType::Set,
            "zset" => InsertDataType::Zset,
            _ => InsertDataType::None,
        }
    }
}

impl Insert {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            insert_type: InsertDataType::String,
            step: 0,
        }
    }

    pub fn enable_insert_mode(&mut self) {
        self.step = 0;
        self.name.clear();
        self.value.clear();
        self.insert_type = InsertDataType::String;
    }

    pub fn handle_insertion_key_event(&mut self, key: KeyEvent) -> Result<InsertKeyEvent> {
        match key.code {
            KeyCode::Enter => {
                if self.step < 2 {
                    self.step += 1;
                } else {
                    return Ok(InsertKeyEvent::PerformInsert);
                }
            }
            KeyCode::Esc => {
                return Ok(InsertKeyEvent::NotInserting);
            }
            KeyCode::Char(c) => match self.step {
                0 => self.name.push(c),
                1 => self.insert_type = c.into(),
                2 => self.value.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.step {
                0 => {
                    self.name.pop();
                }
                2 => {
                    self.value.pop();
                }
                _ => {}
            },
            _ => {}
        }
        Ok(InsertKeyEvent::Noop)
    }
}
