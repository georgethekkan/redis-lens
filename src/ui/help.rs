use crate::app::App;
use crate::redis::ClientOps;
use crate::ui::theme::THEME;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw<R: ClientOps>(frame: &mut Frame, app: &mut App<R>, area: Rect) {
    let help_line = Line::from(vec![
        Span::styled("?", THEME.help_key),
        Span::styled(" Help ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("Filter:", THEME.help_desc),
        Span::styled(format!(" {} ", app.filter_pattern), THEME.help_key),
        Span::raw("| "),
        Span::styled("/", THEME.help_key),
        Span::styled(" Search ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("q", THEME.help_key),
        Span::styled(" Quit ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("↑↓", THEME.help_key),
        Span::styled(" Nav ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("Enter", THEME.help_key),
        Span::styled(" Expand ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("e", THEME.help_key),
        Span::styled(" Edit ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("i", THEME.help_key),
        Span::styled(" Ins ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("a", THEME.help_key),
        Span::styled(" Add ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("b", THEME.help_key),
        Span::styled(" DB ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("d", THEME.help_key),
        Span::styled(" Del ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("n", THEME.help_key),
        Span::styled(" More ", THEME.help_desc),
        Span::raw("| "),
        Span::styled("←→", THEME.help_key),
        Span::styled(" Page ", THEME.help_desc),
    ]);

    let p = Paragraph::new(help_line).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_style(THEME.block_title)
            .border_style(THEME.block_border),
    );
    frame.render_widget(p, area);
}
