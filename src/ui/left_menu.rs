use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;
use crate::redis::RedisOps;

pub fn draw<R: RedisOps>(frame: &mut Frame, app: &mut App<R>, left: Rect) {
    // Left panel: key tree
    let list_items: Vec<ListItem> = app
        .tree
        .flattened_items
        .iter()
        .map(|(name, is_key, depth, is_expanded, key_type)| {
            let indent = "  ".repeat(*depth);
            let symbol = if *is_key {
                "•"
            } else if *is_expanded {
                "▼"
            } else {
                "▶"
            };

            // Apply different styles depending on if it's a key or folder
            let content = format!("{} {} {}", indent, symbol, name);
            let mut style = Style::default();

            if !*is_key {
                style = style.add_modifier(Modifier::BOLD).fg(Color::Blue);
            }

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Keys"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, left, &mut app.list_state);
}
