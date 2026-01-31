use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::redis::RedisOps;
use crate::ui::theme::THEME;

pub fn draw<R: RedisOps>(frame: &mut Frame, app: &mut App<R>, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(15), Constraint::Min(0)])
        .split(area);

    // App Name / Title
    let title = Paragraph::new(" REDIS LENS ").style(THEME.header_title);
    frame.render_widget(title, layout[0]);

    // Info area (Total keys, etc.)
    let raw_url = app.redis_client.url();
    let clean_url = raw_url
        .trim_start_matches("redis://")
        .trim_start_matches("rediss://");

    let info_text = Line::from(vec![Span::styled(
        format!(
            " {} {}  |  {}  |  Keys: {}  |  Mem: {}  |  CPU: {} ",
            app.server_name.to_uppercase(),
            app.server_version,
            clean_url,
            app.total_keys,
            app.used_memory,
            app.used_cpu
        ),
        THEME.header_info,
    )]);
    let info = Paragraph::new(info_text);
    frame.render_widget(info, layout[1]);
}
