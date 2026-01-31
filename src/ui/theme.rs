use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub base: Style,
    pub block_title: Style,
    pub block_border: Style,
    pub block_border_focused: Style,
    pub header_title: Style,
    pub header_info: Style,

    pub key_folder: Style,
    pub key_item: Style,
    pub key_highlight: Style,
    pub tree_symbol: Style,

    pub metadata_label: Style,
    pub metadata_value_key: Style,
    pub metadata_value_type: Style,
    pub metadata_value_ttl: Style,

    pub table_header: Style,
    pub table_index: Style,
    pub table_field: Style,

    pub help_key: Style,
    pub help_desc: Style,

    pub search_popup: Style,
    pub search_input: Style,

    pub type_string: Color,
    pub type_list: Color,
    pub type_hash: Color,
    pub type_set: Color,
    pub type_zset: Color,
}

pub const THEME: Theme = Theme {
    base: Style::new().fg(Color::White),
    block_title: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    block_border: Style::new().fg(Color::Indexed(240)), // Dark gray
    block_border_focused: Style::new().fg(Color::Cyan),
    header_title: Style::new()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD),
    header_info: Style::new().fg(Color::Cyan).add_modifier(Modifier::ITALIC),

    key_folder: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    key_item: Style::new().fg(Color::White),
    key_highlight: Style::new()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD),
    tree_symbol: Style::new().fg(Color::Yellow),

    metadata_label: Style::new().fg(Color::DarkGray),
    metadata_value_key: Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
    metadata_value_type: Style::new().fg(Color::Yellow),
    metadata_value_ttl: Style::new().fg(Color::Green),

    table_header: Style::new()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED),
    table_index: Style::new().fg(Color::DarkGray),
    table_field: Style::new().fg(Color::Cyan),

    help_key: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    help_desc: Style::new().fg(Color::Gray),

    search_popup: Style::new().fg(Color::Yellow),
    search_input: Style::new().fg(Color::White),

    type_string: Color::White,
    type_list: Color::Green,
    type_hash: Color::Cyan,
    type_set: Color::Magenta,
    type_zset: Color::Yellow,
};
