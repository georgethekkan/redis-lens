use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;
use crate::redis::RedisOps;

mod details;
mod header;
mod help;
mod help_modal;
mod left_menu;
pub mod theme;

use theme::THEME;

pub fn draw<R: RedisOps>(frame: &mut Frame, app: &mut App<R>) {
    // Overall layout: Header, main area, and help area
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Main area
            Constraint::Length(3), // Help area
        ]);
    let [header_area, main_area, help_area] = layout.areas(frame.area());

    header::draw(frame, app, header_area);

    // Add left and right panels in main_area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
    let [left, right] = layout.areas(main_area);

    left_menu::draw(frame, app, left);
    help::draw(frame, app, help_area);
    details::draw(frame, app, right);

    if app.is_searching {
        let area = centered_rect(60, 20, frame.area());
        let block = Block::default()
            .title(" Search Pattern ")
            .title_style(THEME.block_title)
            .borders(Borders::ALL)
            .border_style(THEME.search_popup);
        let p = Paragraph::new(app.search_query.as_str())
            .block(block)
            .style(THEME.search_input);
        frame.render_widget(ratatui::widgets::Clear, area); // Clear background
        frame.render_widget(p, area);
    }

    if app.is_editing {
        let area = centered_rect(60, 20, frame.area());
        let block = Block::default()
            .title(" Edit Value ")
            .title_style(THEME.block_title)
            .borders(Borders::ALL)
            .border_style(THEME.search_popup);
        let p = Paragraph::new(app.edit_buffer.as_str())
            .block(block)
            .style(THEME.search_input);
        frame.render_widget(ratatui::widgets::Clear, area); // Clear background
        frame.render_widget(p, area);
    }

    if app.is_inserting {
        let area = centered_rect(60, 25, frame.area());
        let (title, content, hint) = match app.insert_step {
            0 => (
                " 1/3: Key Name ",
                app.insert_name.as_str(),
                "Enter the key name",
            ),
            1 => (
                " 2/3: Key Type ",
                app.insert_type.as_str(),
                "(s:string, h:hash, l:list, e:set, z:zset)",
            ),
            2 => (
                " 3/3: Value Data ",
                app.insert_value.as_str(),
                "Enter value (H: f:v, Z: s:m)",
            ),
            _ => ("", "", ""),
        };

        let block = Block::default()
            .title(title)
            .title_style(THEME.block_title)
            .borders(Borders::ALL)
            .border_style(THEME.search_popup);

        // Show hint in the footer or similar? Let's just use Paragraph with multiple lines or title_bottom
        let p = Paragraph::new(format!("{}\n\nHint: {}", content, hint))
            .block(block)
            .style(THEME.search_input);

        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(p, area);
    }

    if app.is_selecting_db {
        let area = centered_rect(40, 50, frame.area());
        let block = Block::default()
            .title(" Select Database ")
            .title_style(THEME.block_title)
            .borders(Borders::ALL)
            .border_style(THEME.search_popup);

        let mut items = Vec::new();
        for i in 0..16 {
            let style = if i == app.db_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            items.push(ListItem::new(format!(" Database {} ", i)).style(style));
        }

        let list = List::new(items).block(block);

        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(list, area);
    }

    if app.show_help {
        let area = centered_rect(60, 70, frame.area());
        help_modal::draw(frame, app, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
