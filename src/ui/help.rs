use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, help_area: Rect) {
    let details = Paragraph::new(app.help_message()).block(Block::bordered());
    frame.render_widget(details, help_area);
}
