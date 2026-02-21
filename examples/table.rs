use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // Sample table data
    let header_cells = ["ID", "Name", "Status", "Progress"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = [
        vec!["1", "Build UI", "Done", "100%"],
        vec!["2", "Write tests", "In Progress", "60%"],
        vec!["3", "Fix bugs", "Todo", "0%"],
        vec!["4", "Release", "Blocked", "0%"],
    ];

    loop {
        terminal
            .draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(size);

                let table_rows = rows.iter().map(|r| {
                    let cells = r.iter().map(|c| Cell::from(*c));
                    Row::new(cells).height(1)
                });

                let widths = &[
                    Constraint::Length(4),
                    Constraint::Length(20),
                    Constraint::Length(12),
                    Constraint::Length(10),
                ];

                let table = Table::new(table_rows, widths)
                    .header(header.clone())
                    .block(Block::default().borders(Borders::ALL).title("Tasks"))
                    .column_spacing(1)
                    .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

                f.render_widget(table, chunks[0]);
            })
            .unwrap();

        // Exit on 'q' or Esc
        if event::poll(std::time::Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    Ok(())
}
