use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};

use crate::app::{App, CollectionData, LoadedKeyData};
use crate::redis::RedisOps;
use crate::ui::theme::THEME;

pub fn draw<R: RedisOps>(frame: &mut Frame, app: &mut App<R>, area: Rect) {
    // Check if we have a loaded key
    let Some(data) = &app.loaded_key else {
        let block = Block::default()
            .title(" Details ")
            .title_style(THEME.block_title)
            .borders(Borders::ALL)
            .border_style(THEME.block_border)
            .style(Style::default().fg(Color::DarkGray));
        let p = Paragraph::new("No key selected").block(block);
        frame.render_widget(p, area);
        return;
    };

    // Split area into Metadata (top) and Content (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Metadata height
            Constraint::Min(0),    // Content height
        ])
        .split(area);

    draw_metadata(frame, data, chunks[0]);
    draw_content(frame, app, data, chunks[1]);
}

fn draw_metadata(frame: &mut Frame, data: &LoadedKeyData, area: Rect) {
    let block = Block::default()
        .title(" Metadata ")
        .title_style(THEME.block_title)
        .borders(Borders::ALL)
        .border_style(THEME.block_border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner_area);

    // Row 1: Key Name
    let key_text = format!(" Key:  {}", data.key);
    frame.render_widget(
        Paragraph::new(key_text).style(THEME.metadata_value_key),
        layout[0],
    );

    // Row 2: Type & Length
    let type_text = format!(
        " Type: {} | Length: {}",
        data.key_type.to_uppercase(),
        data.length
    );
    frame.render_widget(
        Paragraph::new(type_text).style(THEME.metadata_value_type),
        layout[1],
    );

    // Row 3: TTL
    let ttl_text = format!(" TTL:  {}", data.ttl);
    frame.render_widget(
        Paragraph::new(ttl_text).style(THEME.metadata_value_ttl),
        layout[2],
    );
}

fn draw_content<R: RedisOps>(frame: &mut Frame, app: &App<R>, data: &LoadedKeyData, area: Rect) {
    let type_color = match data.key_type.as_str() {
        "string" => THEME.type_string,
        "list" => THEME.type_list,
        "hash" => THEME.type_hash,
        "set" => THEME.type_set,
        "zset" => THEME.type_zset,
        _ => Color::White,
    };

    let block = Block::default()
        .title(" Content ")
        .title_style(THEME.block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(type_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    match &data.content {
        CollectionData::String(val, _len) => {
            let p = Paragraph::new(val.clone())
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::White));
            frame.render_widget(p, inner_area);
        }
        CollectionData::List(items) => {
            let start_index = app.collection_page * app.collection_page_size;
            let rows: Vec<Row> = items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    Row::new(vec![
                        Cell::from((start_index + i).to_string()).style(THEME.table_index),
                        Cell::from(item.clone()),
                    ])
                })
                .collect();

            let table = Table::new(rows, [Constraint::Length(6), Constraint::Min(0)])
                .header(Row::new(vec!["Index", "Value"]).style(THEME.table_header))
                .column_spacing(1);
            frame.render_widget(table, inner_area);
        }
        CollectionData::Hash(fields) => {
            let rows: Vec<Row> = fields
                .iter()
                .map(|(field, value)| {
                    Row::new(vec![
                        Cell::from(field.clone()).style(THEME.table_field),
                        Cell::from(value.clone()),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [Constraint::Percentage(30), Constraint::Percentage(70)],
            )
            .header(Row::new(vec!["Field", "Value"]).style(THEME.table_header))
            .column_spacing(1);
            frame.render_widget(table, inner_area);
        }
        CollectionData::Set(members) => {
            let rows: Vec<Row> = members
                .iter()
                .map(|member| Row::new(vec![Cell::from(member.clone())]))
                .collect();

            let table = Table::new(rows, [Constraint::Min(0)])
                .header(Row::new(vec!["Member"]).style(THEME.table_header))
                .column_spacing(1);
            frame.render_widget(table, inner_area);
        }
        CollectionData::ZSet(items) => {
            let rows: Vec<Row> = items
                .iter()
                .map(|(member, score)| {
                    Row::new(vec![
                        Cell::from(format!("{:.4}", score)).fg(THEME.type_zset),
                        Cell::from(member.clone()),
                    ])
                })
                .collect();

            let table = Table::new(rows, [Constraint::Length(15), Constraint::Min(0)])
                .header(Row::new(vec!["Score", "Member"]).style(THEME.table_header))
                .column_spacing(1);
            frame.render_widget(table, inner_area);
        }
        CollectionData::None => {
            frame.render_widget(Paragraph::new("No content loaded").italic(), inner_area);
        }
    }
}
