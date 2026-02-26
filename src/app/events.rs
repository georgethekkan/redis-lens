use crate::{
    app::{App, Focus, Insert},
    redis::DataType,
};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};

impl<R: crate::redis::ClientOps> App<R> {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
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

        if let Some(e) = &mut self.editing {
            match key.code {
                KeyCode::Enter => {
                    self.save_edit()?;
                }
                KeyCode::Esc => {
                    self.editing = None;
                }
                KeyCode::Char(c) => {
                    e.buffer.push(c);
                }
                KeyCode::Backspace => {
                    e.buffer.pop();
                }
                _ => {}
            }
            return Ok(());
        }

        if let Some(ins) = &mut self.insert {
            let resp = ins.handle_insertion_key_event(key)?;
            match resp {
                crate::app::insert::InsertKeyEvent::Noop => {}
                crate::app::insert::InsertKeyEvent::PerformInsert => {
                    let ins = ins.clone();
                    self.perform_insertion(ins.name, ins.value, ins.data_type)?;
                    self.insert = None;
                }
                crate::app::insert::InsertKeyEvent::NotInserting => self.insert = None,
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
                if self.focus != Focus::Details {
                    return Ok(());
                }
                let Some(loaded) = &self.loaded_key else {
                    return Ok(());
                };
                let data_type = loaded.data_type.clone();
                if data_type != DataType::String {
                    let insert = Insert {
                        step: 2, // Skip name/type, go straight to value
                        name: loaded.key.clone(),
                        data_type: DataType::from_str(data_type.as_str()),
                        value: String::new(),
                    };
                    self.insert = Some(insert);
                }
            }
            KeyCode::Char('r') => self.refresh()?,
            KeyCode::Char('b') => {
                self.is_selecting_db = true;
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::LeftMenu => {
                        if self.loaded_key.is_some() {
                            Focus::Details
                        } else {
                            Focus::LeftMenu
                        }
                    }
                    Focus::Details => Focus::LeftMenu,
                };
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

    pub fn handle_left_menu_key_event(&mut self, key: KeyEvent) -> Result<()> {
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
            KeyCode::Char('n') => self.load_next_page()?,

            KeyCode::Right => {
                let index = self.list_state.selected().unwrap_or(0);
                let Some((_, is_key, _, is_expanded, _)) = self.tree.flattened_items.get(index)
                else {
                    return Ok(());
                };

                if *is_key {
                    self.move_focus_to_details();
                } else if !*is_expanded {
                    self.expand_current();
                }
            }
            KeyCode::Left => {
                self.collapse_current();
            }

            _ => match key.code {
                KeyCode::Char('l') => {
                    self.next_collection_page();
                    self.fetch_selected_key_details()?;
                }
                KeyCode::Char('h') => {
                    self.prev_collection_page();
                    self.fetch_selected_key_details()?;
                }
                _ => {}
            },
        }
        Ok(())
    }

    pub fn handle_details_key_event(&mut self, key: KeyEvent) -> Result<()> {
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

    pub fn handle_db_selection_key_event(&mut self, key: KeyEvent) -> Result<()> {
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

    pub fn handle_selection_change(&mut self) -> Result<()> {
        let Some(index) = self.list_state.selected() else {
            self.loaded_key = None;
            return Ok(());
        };

        if let Some((_, is_key, _, _, _)) = self.tree.flattened_items.get(index) {
            if *is_key {
                if let Some(path) = self.tree.flattened_paths.get(index).cloned() {
                    self.fetch_details_for_key(&path)?;
                }
            } else {
                self.loaded_key = None;
            }
        }
        Ok(())
    }

    pub fn toggle_expanded(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(path) = self.tree.flattened_paths.get(index).cloned() else {
            return;
        };
        self.tree.toggle_expansion(&path);
    }

    pub fn expand_current(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(path) = self.tree.flattened_paths.get(index).cloned() else {
            return;
        };
        self.tree.set_expansion(&path, true);
    }

    pub fn collapse_current(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(path) = self.tree.flattened_paths.get(index).cloned() else {
            return;
        };
        self.tree.set_expansion(&path, false);
    }

    pub fn load_next_page(&mut self) -> Result<()> {
        if self.next == "0" {
            return Ok(());
        }
        let resp = self.client.scan(&self.next, &self.filter_pattern, 100)?;
        self.next = resp.next;
        self.keys.extend(resp.keys);
        self.message = Some("Loaded next page of keys.".to_string());
        self.rebuild_tree();
        Ok(())
    }

    pub fn fetch_selected_key_details(&mut self) -> Result<()> {
        let Some(index) = self.list_state.selected() else {
            self.loaded_key = None;
            return Ok(());
        };

        let Some(path) = self.tree.flattened_paths.get(index).cloned() else {
            self.loaded_key = None;
            return Ok(());
        };

        let Some((_, is_key, _, _, _)) = self.tree.flattened_items.get(index) else {
            return Ok(());
        };

        if *is_key {
            self.fetch_details_for_key(&path)?;
        }
        Ok(())
    }
}
