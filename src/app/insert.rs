use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};

use crate::redis::DataType;

#[derive(Debug, Default, Clone)]
pub struct Insert {
    pub name: String,
    pub value: String,
    pub data_type: DataType, // "string", "hash", "list", "set", "zset"
    pub step: usize,         // 0: Name, 1: Type, 2: Value/Field
}

pub enum InsertKeyEvent {
    Noop,
    PerformInsert, //clear too
    NotInserting,
}

impl Insert {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            data_type: DataType::String,
            step: 0,
        }
    }

    pub fn enable_insert_mode(&mut self) {
        self.step = 0;
        self.name.clear();
        self.value.clear();
        self.data_type = DataType::String;
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
            KeyCode::Esc => return Ok(InsertKeyEvent::NotInserting),
            KeyCode::Char(c) => match self.step {
                0 => self.name.push(c),
                1 => self.data_type = DataType::from_char(c),
                2 => self.value.push(c),
                _ => {}
            },
            KeyCode::Backspace => self.handle_backspace(),
            _ => {}
        }
        Ok(InsertKeyEvent::Noop)
    }

    fn handle_backspace(&mut self) {
        if self.step == 0 {
            self.name.pop();
        } else if self.step == 2 {
            self.value.pop();
        }
    }
}
