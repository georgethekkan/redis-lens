use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;
use crate::redis::RedisOps;

mod details;
mod header;
mod help;
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
