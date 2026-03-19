use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::app::App;
use crate::redis::ClientOps;
use crate::ui::theme::THEME;

pub fn draw<R: ClientOps>(frame: &mut Frame, app: &mut App<R>, left: Rect) {
    // Left panel: key tree
    let list_items: Vec<ListItem> = app
        .tree
        .flattened_items
        .iter()
        .map(|(name, is_key, depth, is_expanded, _key_type)| {
            let indent = "  ".repeat(*depth);
            let symbol = if *is_key {
                "•"
            } else if *is_expanded {
                "▼"
            } else {
                "▶"
            };

            // Apply different styles depending on if it's a key or folder
            let style = if !*is_key {
                THEME.key_folder
            } else {
                THEME.key_item
            };

            use ratatui::text::{Line, Span};
            let content = Line::from(vec![
                Span::raw(indent),
                Span::styled(symbol, THEME.tree_symbol),
                Span::raw("  "),
                Span::styled(name, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let border_style = if app.focus == crate::app::Focus::LeftMenu {
        THEME.block_border_focused
    } else {
        THEME.block_border
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keys ")
                .title_style(THEME.block_title)
                .border_style(border_style),
        )
        .highlight_style(THEME.key_highlight)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, left, &mut app.list_state);
}
