use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App, left: Rect) {
    // Left panel: key list
    let keys: Vec<String> = app.keys.to_vec();
    let list_items: Vec<ListItem> = keys.iter().map(|k| ListItem::new(k.as_str())).collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Items"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, left, &mut app.list_state);
}
