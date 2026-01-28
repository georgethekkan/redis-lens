use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;

mod details;
mod help;
mod left_menu;

pub fn draw(frame: &mut Frame, app: &mut App) {
    // Overall layout: main area and help area
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Min(0), Constraint::Length(3)]);
    let [main_area, help_area] = layout.areas(frame.area());

    // Add left and right panels in main_area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
    let [left, right] = layout.areas(main_area);

    left_menu::draw(frame, app, left);
    help::draw(frame, app, help_area);
    details::draw(frame, app, right);
}
