// src/main.rs
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::{io, time::Duration};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items = (1..=4)
        .map(|i| format!("item {}", i))
        .collect::<Vec<String>>();

    let mut state = ListState::default();
    state.select(Some(0)); // start selected at first item

    // Main loop
    let res = run_app(&mut terminal, items, &mut state);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    items: Vec<String>,
    state: &mut ListState,
) -> Result<(), io::Error> {
    loop {
        draw_list(terminal, &items, state)?;

        if !handle_key_event(state, &items)? {
            return Ok(());
        }
    }
}

fn handle_key_event(state: &mut ListState, items: &[String]) -> Result<bool, io::Error> {
    // Input
    if event::poll(Duration::from_millis(100))?
        && let CEvent::Key(key) = event::read()?
    {
        match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Up => {
                let i = match state.selected() {
                    Some(i) => {
                        if i == 0 {
                            items.len().saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.select(Some(i));
            }
            KeyCode::Down => {
                let i = match state.selected() {
                    Some(i) => {
                        if i >= items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.select(Some(i));
            }
            /*KeyCode::Enter => {
                if let Some(i) = state.selected() {
                    // Do something with selection; here we just print and exit
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    println!("Selected: {}", items[i]);
                    //return Ok(());
                }
            }*/
            _ => {}
        }
    }
    Ok(true)
}

fn draw_list(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    items: &[String],
    state: &mut ListState,
) -> Result<(), io::Error> {
    terminal.draw(|f| {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(35), Constraint::Min(0)].as_ref())
            .split(size);

        let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.clone())).collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("Items"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[0], state);
    })?;
    Ok(())
}
