use crate::app::App;
use crate::redis::ClientOps;
use crate::ui::theme::THEME;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn draw<R: ClientOps>(frame: &mut Frame, app: &mut App<R>, area: Rect) {
    let Some(conf) = &app.confirm_delete else {
        return;
    };

    let title = if conf.is_folder {
        " Delete Tree "
    } else {
        " Delete Key "
    };

    let message = if conf.is_folder {
        format!(
            "Are you sure you want to delete folder '{}' and all keys below it?",
            conf.path
        )
    } else {
        format!("Are you sure you want to delete key '{}'?", conf.path)
    };

    let block = Block::default()
        .title(title)
        .title_style(THEME.block_title)
        .borders(Borders::ALL)
        .border_style(THEME.search_popup);

    let content = format!(
        "\n{}\n\n(y) Yes, Delete  (n) No, Cancel",
        message
    );

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center)
        .style(THEME.search_input);

    let area = centered_rect(60, 20, area);
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .flex(Flex::Center)
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .flex(Flex::Center)
    .split(popup_layout[1])[1]
}
