use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

fn main() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}

struct App {
    exit: bool,
    list_state: ListState,
    list_items: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            exit: false,
            list_state: ListState::default(),
            list_items: vec![
                "Item 1", "Item 2", "Item 3", "Item 4", "Item 5", "Item 6", "Item 7",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                    KeyCode::Down => self.list_state.select_next(),
                    KeyCode::Up => self.list_state.select_previous(),
                    _ => {}
                }
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]);
        let [left, right] = layout.areas(frame.area());

        // Left panel: List
        let list_items: Vec<ListItem> = self
            .list_items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("Items"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("⇾ ");

        frame.render_stateful_widget(list, left, &mut self.list_state);

        // Right panel: Placeholder for details or other content
        let details = Block::bordered().title("Details");
        frame.render_widget(details, right);
    }
}
