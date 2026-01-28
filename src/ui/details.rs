use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Widget};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, right: Rect) {
    let rows = build_detail_rows(&app);
    let header = Row::new(vec![
        Cell::from("Field").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Value").style(Style::default().fg(Color::White)),
    ]);

    let table = Table::new(rows, &[Constraint::Length(12), Constraint::Min(0)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .style(Style::default().fg(Color::White))
        .widths(&[Constraint::Length(20), Constraint::Min(0)])
        .column_spacing(1);

    frame.render_widget(table, right);
}

pub fn build_detail_rows(app: &App) -> Vec<Row<'_>> {
    let Some(index) = app.list_state.selected() else {
        return vec![];
    };

    let Some(key) = app.keys.get(index) else {
        return vec![];
    };

    let key_type = app
        .redis_client
        .key_type(key)
        .unwrap_or_else(|e| format!("Error - {}", e));

    let ttl_info = match app.redis_client.ttl(key) {
        Ok(Some(-1)) => "No expiration".to_string(),
        Ok(Some(ttl)) => format!("{} seconds", ttl),
        Ok(None) => "Key does not exist".to_string(),
        Err(e) => format!("Error - {}", e),
    };

    let mut rows = vec![
        Row::new(vec!["Key".to_string(), key.clone()]),
        Row::new(vec!["Type".to_string(), key_type.clone()]),
        Row::new(vec!["TTL".to_string(), ttl_info]),
    ];

    match key_type.as_str() {
        "string" => add_string_rows(&app, &mut rows, key),
        "list" => add_list_rows(&app, &mut rows, key),
        "hash" => add_hash_rows(&app, &mut rows, key),
        "set" => add_set_rows(&app, &mut rows, key),
        "zset" => add_zset_rows(&app, &mut rows, key),
        _ => rows.push(Row::new(vec![
            "Status".to_string(),
            "Unknown type".to_string(),
        ])),
    }

    rows
}

fn add_string_rows(app: &App, rows: &mut Vec<Row>, key: &str) {
    let value_info = match app.redis_client.get(key) {
        Ok(value) => {
            let len = app.redis_client.strlen(key).unwrap_or(0);
            format!("{} ({} bytes)", value, len)
        }
        Err(e) => format!("Error - {}", e),
    };
    rows.push(Row::new(vec!["Value".to_string(), value_info]));
}

// Pagination helpers unified for all collection types
fn page_range_i64(page: usize, page_size: usize) -> (i64, i64) {
    let start = (page.saturating_mul(page_size)) as i64;
    let stop = start + page_size as i64 - 1;
    (start, stop)
}

fn slice_bounds(total: usize, page: usize, page_size: usize) -> (usize, usize) {
    let start = page.saturating_mul(page_size);
    let end = std::cmp::min(start + page_size, total);
    (start, end)
}

fn add_list_rows(app: &App, rows: &mut Vec<Row>, key: &str) {
    let Ok(len) = app.redis_client.llen(key) else {
        rows.push(Row::new(vec![
            "Error".to_string(),
            "Failed to get length".to_string(),
        ]));
        return;
    };

    rows.push(Row::new(vec!["Length".to_string(), len.to_string()]));
    let (start_i64, stop_i64) = page_range_i64(app.collection_page, app.collection_page_size);
    if let Ok(items) = app.redis_client.lrange(key, start_i64, stop_i64) {
        let start_usize = start_i64 as usize;
        for (i, item) in items.iter().enumerate() {
            let idx = start_usize + i;
            rows.push(Row::new(vec![format!("[{}]", idx), item.clone()]));
        }
    }
}

fn add_hash_rows(app: &App, rows: &mut Vec<Row>, key: &str) {
    match app.redis_client.hlen(key) {
        Ok(len) => {
            rows.push(Row::new(vec!["Fields".to_string(), len.to_string()]));
            if let Ok(data) = app.redis_client.hgetall(key) {
                let (start, end) =
                    slice_bounds(data.len(), app.collection_page, app.collection_page_size);
                for (field, value) in &data[start..end] {
                    rows.push(Row::new(vec![field.clone(), value.clone()]));
                }
            }
        }
        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
    }
}

fn add_set_rows(app: &App, rows: &mut Vec<Row>, key: &str) {
    match app.redis_client.scard(key) {
        Ok(count) => {
            rows.push(Row::new(vec!["Members".to_string(), count.to_string()]));
            if let Ok(members) = app.redis_client.smembers(key) {
                let (start, end) =
                    slice_bounds(members.len(), app.collection_page, app.collection_page_size);
                for member in &members[start..end] {
                    rows.push(Row::new(vec!["Member".to_string(), member.clone()]));
                }
            }
        }
        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
    }
}

fn add_zset_rows(app: &App, rows: &mut Vec<Row>, key: &str) {
    match app.redis_client.zcard(key) {
        Ok(count) => {
            rows.push(Row::new(vec!["Members".to_string(), count.to_string()]));
            let (start_i64, stop_i64) =
                page_range_i64(app.collection_page, app.collection_page_size);
            if let Ok(items) = app
                .redis_client
                .zrange_with_scores(key, start_i64, stop_i64)
            {
                for (member, score) in items {
                    rows.push(Row::new(vec![member, format!("{:.2}", score)]));
                }
            }
        }
        Err(e) => rows.push(Row::new(vec!["Error".to_string(), format!("{}", e)])),
    }
}
