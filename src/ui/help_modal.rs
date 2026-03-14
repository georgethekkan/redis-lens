use crate::app::App;
use crate::redis::ClientOps;
use crate::ui::theme::THEME;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::{Block, Borders, Clear, Row, Table};

pub fn draw<R: ClientOps>(frame: &mut Frame, app: &mut App<R>, area: Rect) {
    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(THEME.block_title)
        .borders(Borders::ALL)
        .border_style(THEME.search_popup);

    let mut rows = vec![
        Row::new(vec!["Global", "", ""]),
        Row::new(vec!["  ?", "Show this help modal", ""]),
        Row::new(vec!["  q, Esc", "Quit application", ""]),
        Row::new(vec!["  r", "Refresh all data", ""]),
        Row::new(vec!["  b", "Switch Database (0-15)", ""]),
        Row::new(vec!["  /", "Search/Filter keys", ""]),
    ];

    if app.read_only {
        rows.push(Row::new(vec!["  i", "Insert new key (Disabled)", ""]));
    } else {
        rows.push(Row::new(vec!["  i", "Insert new key", ""]));
    }

    rows.extend(vec![
        Row::new(vec!["", "", ""]),
        Row::new(vec!["Navigation", "", ""]),
        Row::new(vec!["  ↑, ↓", "Navigate lists (no cycling)", ""]),
        Row::new(vec!["  Tab", "Switch focus between panels", ""]),
        Row::new(vec!["  Enter", "Expand folder / Select key", ""]),
        Row::new(vec!["  →", "Expand folder / Focus details", ""]),
        Row::new(vec!["  ←", "Collapse folder / Focus tree", ""]),
        Row::new(vec!["", "", ""]),
        Row::new(vec!["Operations", "", ""]),
    ]);

    if app.read_only {
        rows.push(Row::new(vec!["  e", "Edit current value/item (Disabled)", ""]));
        rows.push(Row::new(vec!["  a", "Add item to collection (Disabled)", ""]));
        rows.push(Row::new(vec!["  d", "Delete key or item (Disabled)", ""]));
    } else {
        rows.push(Row::new(vec!["  e", "Edit current value/item", ""]));
        rows.push(Row::new(vec!["  a", "Add item to collection", ""]));
        rows.push(Row::new(vec!["  d", "Delete key or item", ""]));
    }

    rows.extend(vec![
        Row::new(vec!["  n", "Load next page of keys", ""]),
        Row::new(vec!["  h, l", "Prev/Next collection page", ""]),
    ]);

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
