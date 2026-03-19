use crate::{
    app::{App, Data, Editing},
    redis::DataType,
};
use color_eyre::eyre::Result;

impl<R: crate::redis::ClientOps> App<R> {
    pub fn perform_insertion(
        &mut self,
        key: String,
        value: String,
        data_type: DataType,
    ) -> Result<()> {
        match data_type {
            DataType::String => {
                self.client.set(&key, &value)?;
            }
            DataType::Hash => {
                // Expect "field:value"
                let Some((field, val)) = value.split_once(':') else {
                    self.message = Some("Format for Hash: field:value".to_string());
                    return Ok(());
                };
                self.client.hset(&key, field, val)?;
            }
            DataType::List => {
                self.client.rpush(&key, &value)?;
            }
            DataType::Set => {
                self.client.sadd(&key, &value)?;
            }
            DataType::Zset => {
                // Expect "score:member"
                let Some((score_str, member)) = value.split_once(':') else {
                    self.message = Some("Format for ZSet: score:member".to_string());
                    return Ok(());
                };

                let Ok(score) = score_str.parse::<f64>() else {
                    self.message = Some("Format for ZSet: score:member".to_string());
                    return Ok(());
                };

                self.client.zadd(&key, score, member)?;
            }
            _ => {}
        }

        self.message = Some(format!("Inserted into {}", key));
        self.refresh()?;
        Ok(())
    }

    pub fn delete_selected_key(&mut self) -> Result<()> {
        let Some(index) = self.list_state.selected() else {
            return Ok(());
        };
        let Some(path) = self.tree.flattened_paths.get(index).cloned() else {
            return Ok(());
        };
        let (_, is_key, _, _, _) = self.tree.flattened_items[index];

        self.confirm_delete = Some(crate::app::DeleteConfirmation {
            path,
            is_folder: !is_key,
        });

        Ok(())
    }

    pub fn confirm_delete_action(&mut self) -> Result<()> {
        let Some(conf) = self.confirm_delete.take() else {
            return Ok(());
        };

        if conf.is_folder {
            let pattern = format!("{}:*", conf.path);
            let count = self.client.delete_all(&pattern)?;
            // Also check if the path itself is a key
            let deleted_self = self.client.del(&conf.path)?;
            self.message = Some(format!(
                "Deleted tree: {} ({} keys removed)",
                conf.path,
                count + deleted_self as usize
            ));
        } else {
            self.client.del(&conf.path)?;
            self.message = Some(format!("Deleted key: {}", conf.path));
        }

        self.refresh()?;
        self.loaded_key = None;
        Ok(())
    }

    pub fn delete_collection_item(&mut self) -> Result<()> {
        let Some(index) = self.details_table_state.selected() else {
            return Ok(());
        };
        let Some(loaded) = &self.loaded_key else {
            return Ok(());
        };

        let key = loaded.key.clone();
        match &loaded.content {
            Data::Hash(fields) => {
                if let Some((field, _)) = fields.get(index) {
                    self.client.hdel(&key, field)?;
                }
            }
            Data::List(items) => {
                if let Some(value) = items.get(index) {
                    self.client.lrem(&key, 1, value)?;
                }
            }
            Data::Set(members) => {
                if let Some(member) = members.get(index) {
                    self.client.srem(&key, member)?;
                }
            }
            Data::ZSet(items) => {
                if let Some((member, _)) = items.get(index) {
                    self.client.zrem(&key, member)?;
                }
            }
            _ => return Ok(()),
        }

        // Reload data
        self.fetch_details_for_key(&key)?;

        // Adjust selection
        let new_len = self.get_loaded_collection_length();
        if new_len == 0 {
            self.details_table_state.select(None);
        } else if index >= new_len {
            self.details_table_state.select(Some(new_len - 1));
        }

        Ok(())
    }

    pub fn start_editing(&mut self) {
        let Some(loaded) = &self.loaded_key else {
            return;
        };
        match &loaded.content {
            Data::String(val, _) => {
                self.editing = Some(Editing::new(val.clone(), val.clone()));
            }
            Data::Hash(fields) => {
                let Some(index) = self.details_table_state.selected() else {
                    return;
                };
                let Some((_, val)) = fields.get(index) else {
                    return;
                };
                self.editing = Some(Editing::new(val.clone(), val.clone()));
            }
            Data::List(items) => {
                let Some(index) = self.details_table_state.selected() else {
                    return;
                };
                let Some(val) = items.get(index) else { return };
                self.editing = Some(Editing::new(val.clone(), val.clone()));
            }
            Data::Set(members) => {
                let Some(index) = self.details_table_state.selected() else {
                    return;
                };
                let Some(val) = members.get(index) else {
                    return;
                };
                self.editing = Some(Editing::new(val.clone(), val.clone()));
            }
            Data::ZSet(items) => {
                let Some(index) = self.details_table_state.selected() else {
                    return;
                };
                let Some((val, _)) = items.get(index) else {
                    return;
                };
                self.editing = Some(Editing::new(val.clone(), val.clone()));
            }
            _ => {
                self.message = Some("Editing not supported for this type yet".to_string());
            }
        }
    }

    pub fn save_edit(&mut self) -> Result<()> {
        let Some(loaded) = &self.loaded_key else {
            self.editing = None;
            return Ok(());
        };

        let Some(e) = &self.editing else {
            return Ok(());
        };

        let key = loaded.key.clone();
        let new_value = e.buffer.clone();

        match &loaded.content {
            Data::String(_, _) => {
                self.client.set(&key, &new_value)?;
                self.message = Some(format!("Updated string: {}", key));
            }
            Data::Hash(fields) => {
                let Some(index) = self.details_table_state.selected() else {
                    return Ok(());
                };
                let Some((field, _)) = fields.get(index) else {
                    return Ok(());
                };
                self.client.hset(&key, field, &new_value)?;
                self.message = Some(format!("Updated hash field: {}", field));
            }
            Data::List(_) => {
                let Some(index) = self.details_table_state.selected() else {
                    return Ok(());
                };
                let list_index = (self.collection_page * self.collection_page_size + index) as i64;
                self.client.lset(&key, list_index, &new_value)?;
                self.message = Some(format!("Updated list item at index {}", list_index));
            }
            Data::Set(_) => {
                self.client.srem(&key, &e.original)?;
                self.client.sadd(&key, &new_value)?;
                self.message = Some("Updated set member".to_string());
            }
            Data::ZSet(items) => {
                let Some(index) = self.details_table_state.selected() else {
                    return Ok(());
                };
                let Some((_, score)) = items.get(index) else {
                    return Ok(());
                };
                self.client.zrem(&key, &e.original)?;
                self.client.zadd(&key, *score, &new_value)?;
                self.message = Some("Updated sorted set member".to_string());
            }
            _ => {}
        }

        self.editing = None;
        self.fetch_details_for_key(&key)?;
        Ok(())
    }

    pub fn confirm_search(&mut self) -> Result<()> {
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
        let resp = self.client.scan("0", &self.filter_pattern, 100)?;
        self.next = resp.next;
        self.keys = resp.keys;

        // Reset selection and rebuild tree
        self.list_state.select(None);
        self.loaded_key = None;
        self.rebuild_tree();

        self.message = Some(format!("Searching for: {}", self.filter_pattern));

        Ok(())
    }

    pub fn confirm_db_selection(&mut self) -> Result<()> {
        let new_db = self.db_cursor as u8;
        self.client.select_db(new_db)?;
        self.is_selecting_db = false;

        // Refresh everything
        self.keys.clear();
        self.key_types.clear();
        self.loaded_key = None;
        let resp = self.client.scan("0", &self.filter_pattern, 100)?;
        self.next = resp.next;
        self.keys = resp.keys;
        self.rebuild_tree();
        self.update_stats()?;

        self.message = Some(format!("Switched to Database {}", new_db));
        Ok(())
    }
}
