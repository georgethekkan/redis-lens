use crate::app::App;
use crate::redis::RedisOps;
use crate::ui::theme::THEME;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Row, Table};

pub fn draw<R: RedisOps>(frame: &mut Frame, _app: &mut App<R>, area: Rect) {
    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(THEME.block_title)
        .borders(Borders::ALL)
        .border_style(THEME.search_popup);

    let rows = vec![
        Row::new(vec!["Global", "", ""]),
        Row::new(vec!["  ?", "Show this help modal", ""]),
        Row::new(vec!["  q, Esc", "Quit application", ""]),
        Row::new(vec!["  r", "Refresh all data", ""]),
        Row::new(vec!["  b", "Switch Database (0-15)", ""]),
        Row::new(vec!["  /", "Search/Filter keys", ""]),
        Row::new(vec!["  i", "Insert new key", ""]),
        Row::new(vec!["", "", ""]),
        Row::new(vec!["Navigation", "", ""]),
        Row::new(vec!["  ↑, ↓", "Navigate lists (no cycling)", ""]),
        Row::new(vec!["  Tab", "Switch focus between panels", ""]),
        Row::new(vec!["  Enter", "Expand folder / Select key", ""]),
        Row::new(vec!["  →", "Expand folder / Focus details", ""]),
        Row::new(vec!["  ←", "Collapse folder / Focus tree", ""]),
        Row::new(vec!["", "", ""]),
        Row::new(vec!["Operations", "", ""]),
        Row::new(vec!["  e", "Edit current value/item", ""]),
        Row::new(vec!["  a", "Add item to collection", ""]),
        Row::new(vec!["  d", "Delete key or item", ""]),
        Row::new(vec!["  n", "Load next page of keys", ""]),
        Row::new(vec!["  h, l", "Prev/Next collection page", ""]),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Min(30),
            Constraint::Length(0),
        ],
    )
    .block(block)
    .style(THEME.search_input);

    frame.render_widget(Clear, area);
    frame.render_widget(table, area);
}
