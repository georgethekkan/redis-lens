use redis_lens::tree::Tree;
use std::collections::BTreeMap;

#[test]
fn test_tree_rebuild_simple() {
    let mut tree = Tree::new();
    let keys = vec!["a:b:c".to_string(), "a:b:d".to_string(), "x:y".to_string()];
    tree.rebuild(&keys, &BTreeMap::new());

    assert_eq!(tree.root.children.len(), 2);
    assert!(tree.root.children.contains_key("a"));
    assert!(tree.root.children.contains_key("x"));

    let a = tree.root.children.get("a").unwrap();
    assert_eq!(a.children.len(), 1);
    assert!(a.children.contains_key("b"));

    let b = a.children.get("b").unwrap();
    assert_eq!(b.children.len(), 2);
    assert!(b.children.contains_key("c"));
    assert!(b.children.contains_key("d"));

    let c = b.children.get("c").unwrap();
    assert!(c.is_key);
    assert_eq!(c.full_path, "a:b:c");
}

#[test]
fn test_tree_flattening() {
    let mut tree = Tree::new();
    let keys = vec!["a:b".to_string(), "a:c".to_string()];
    tree.rebuild(&keys, &BTreeMap::new());

    // Default collapsed, so only top level "a" should be there
    assert_eq!(tree.flattened_items.len(), 1);
    assert_eq!(tree.flattened_items[0].0, "a");
    assert!(!tree.flattened_items[0].1); // Not a key

    // Expand "a"
    tree.set_expansion("a", true);
    assert_eq!(tree.flattened_items.len(), 3); // a, b, c
    assert_eq!(tree.flattened_items[1].0, "b");
    assert!(tree.flattened_items[1].1); // Is a key
    assert_eq!(tree.flattened_items[1].2, 1); // Depth 1
}

#[test]
fn test_tree_toggle_expansion() {
    let mut tree = Tree::new();
    let keys = vec!["a:b".to_string()];
    tree.rebuild(&keys, &BTreeMap::new());

    assert!(!tree.root.children.get("a").unwrap().is_expanded);
    tree.toggle_expansion("a");
    assert!(tree.root.children.get("a").unwrap().is_expanded);
    tree.toggle_expansion("a");
    assert!(!tree.root.children.get("a").unwrap().is_expanded);
}

#[test]
fn test_tree_with_types() {
    let mut tree = Tree::new();
    let keys = vec!["a:b".to_string()];
    let mut types = BTreeMap::new();
    types.insert("a:b".to_string(), "string".to_string());

    tree.rebuild(&keys, &types);

    tree.set_expansion("a", true);
    let item = &tree.flattened_items[1];
    assert_eq!(item.0, "b");
    assert_eq!(item.4, Some("string".to_string()));
}
