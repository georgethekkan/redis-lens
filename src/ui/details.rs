use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, Widget};

use crate::app::{App, CollectionData, LoadedKeyData};

pub fn draw(frame: &mut Frame, app: &App, right: Rect) {
    let rows = build_detail_rows(app);
    let header = Row::new(vec![
        Cell::from("Field").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Value").style(Style::default().fg(Color::White)),
    ]);

    let table = Table::new(rows, &[Constraint::Length(20), Constraint::Min(0)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .style(Style::default().fg(Color::White))
        // .widths(&[Constraint::Length(20), Constraint::Min(0)]) // widths is deprecated in favor of new(), but let's check ratatui version. 0.30.0 uses new(rows, widths).
        // Wait, the previous code used .widths(). ratatui 0.30.0 might have deprecated it.
        // The previous code: Table::new(rows, constraints).widths(...)
        // In recent ratatui versions, `Table::new` takes `rows` and `widths`.
        // Let's stick to the previous code structure if possible, but the previous code was:
        // Table::new(rows, &[Constraint::Length(12), Constraint::Min(0)]).widths(...)
        // Actually, Table::new(rows, widths) is the modern way. The previous code might have been using an older pattern or mismatched.
        // Let's look at the previous code in step 27.
        // Table::new(rows, &[Constraint::Length(12), Constraint::Min(0)])
        // .widths(&[Constraint::Length(20), Constraint::Min(0)])
        // This looks redundant. I will just use Table::new with constraints.
        .column_spacing(1);

    frame.render_widget(table, right);
}

pub fn build_detail_rows(app: &App) -> Vec<Row<'_>> {
    let Some(data) = &app.loaded_key else {
        return vec![Row::new(vec!["Status", "No key selected or loaded"])];
    };

    let mut rows = vec![
        Row::new(vec!["Key".to_string(), data.key.clone()]),
        Row::new(vec!["Type".to_string(), data.key_type.clone()]),
        Row::new(vec!["TTL".to_string(), data.ttl.clone()]),
    ];

    match &data.content {
        CollectionData::String(val, len) => {
            rows.push(Row::new(vec![
                "Value".to_string(),
                format!("{} ({} bytes)", val, len),
            ]));
        }
        CollectionData::List(items) => {
            rows.push(Row::new(vec![
                "Length".to_string(),
                data.length.to_string(),
            ]));
            // We need to calculate indices.
            // The items in `data.content` are the SLICE for the current page.
            // We need `app.collection_page` and `app.collection_page_size` to calculate absolute index.
            let start_index = app.collection_page * app.collection_page_size;
            for (i, item) in items.iter().enumerate() {
                let idx = start_index + i;
                rows.push(Row::new(vec![format!("[{}]", idx), item.clone()]));
            }
        }
        CollectionData::Hash(fields) => {
            rows.push(Row::new(vec![
                "Fields".to_string(),
                data.length.to_string(),
            ]));
            for (field, value) in fields {
                rows.push(Row::new(vec![field.clone(), value.clone()]));
            }
        }
        CollectionData::Set(members) => {
            rows.push(Row::new(vec![
                "Members".to_string(),
                data.length.to_string(),
            ]));
            for member in members {
                rows.push(Row::new(vec!["Member".to_string(), member.clone()]));
            }
        }
        CollectionData::ZSet(items) => {
            rows.push(Row::new(vec![
                "Members".to_string(),
                data.length.to_string(),
            ]));
            for (member, score) in items {
                rows.push(Row::new(vec![member.clone(), format!("{:.2}", score)]));
            }
        }
        CollectionData::None => {
            // Do nothing specific
        }
    }

    rows
}
