use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Widget};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, help_area: Rect) {
    let help_text = format!(
        "{} | Filter: {} | /: Search | q: Quit | ↑↓: Navigate | Enter: Expand/Select | d: Delete | n: Load More | ←→: Page Collection",
        app.redis_client.url, app.filter_pattern
    );
    let p = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(p, help_area);
}
