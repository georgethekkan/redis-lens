use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct TreeNode {
    pub name: String,
    pub full_path: String,
    pub children: BTreeMap<String, TreeNode>,
    pub is_expanded: bool,
    pub is_key: bool,
    pub key_type: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub root: TreeNode,
    pub flattened_items: Vec<(String, bool, usize, bool, Option<String>)>, // (name, is_key, depth, is_expanded, type)
    pub flattened_paths: Vec<String>, // parallel to items, stores full path
}

impl Tree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild(&mut self, keys: &[String], types: &BTreeMap<String, String>) {
        self.root = TreeNode::default();
        for key in keys {
            let parts: Vec<&str> = key.split(':').collect();
            let mut current = &mut self.root;
            let mut path_acc = String::new();

            for (i, part) in parts.iter().enumerate() {
                if !path_acc.is_empty() {
                    path_acc.push(':');
                }
                path_acc.push_str(part);

                let is_last = i == parts.len() - 1;

                if !current.children.contains_key(*part) {
                    let node = TreeNode {
                        name: part.to_string(),
                        full_path: path_acc.clone(),
                        children: BTreeMap::new(),
                        is_expanded: false,
                        is_key: false,
                        key_type: None,
                    };
                    current.children.insert(part.to_string(), node);
                }

                current = current.children.get_mut(*part).unwrap();
                if is_last {
                    current.is_key = true;
                    current.key_type = types.get(key).cloned();
                }
            }
        }
        self.flatten();
    }

    pub fn flatten(&mut self) {
        let mut items = vec![];
        let mut paths = vec![];

        Self::flatten_recursive(&self.root, 0, &mut items, &mut paths);

        self.flattened_items = items;
        self.flattened_paths = paths;
    }

    fn flatten_recursive(
        node: &TreeNode,
        depth: usize,
        items: &mut Vec<(String, bool, usize, bool, Option<String>)>,
        paths: &mut Vec<String>,
    ) {
        for child in node.children.values() {
            items.push((
                child.name.clone(),
                child.is_key,
                depth,
                child.is_expanded,
                child.key_type.clone(),
            ));
            paths.push(child.full_path.clone());

            if child.is_expanded {
                Self::flatten_recursive(child, depth + 1, items, paths);
            }
        }
    }

    pub fn toggle_expansion(&mut self, path: &str) {
        if let Some(node) = self.find_node_mut(path) {
            node.is_expanded = !node.is_expanded;
            self.flatten();
        }
    }

    pub fn set_expansion(&mut self, path: &str, expanded: bool) {
        if let Some(node) = self.find_node_mut(path) {
            node.is_expanded = expanded;
            self.flatten();
        }
    }

    fn find_node_mut(&mut self, path: &str) -> Option<&mut TreeNode> {
        let parts: Vec<&str> = path.split(':').collect();
        let mut current = &mut self.root;
        for part in parts {
            if let Some(node) = current.children.get_mut(part) {
                current = node;
            } else {
                return None;
            }
        }
        Some(current)
    }
}
