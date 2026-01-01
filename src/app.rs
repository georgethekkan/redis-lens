use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use ratatui::widgets::canvas::{Canvas, Map, MapResolution};
use ratatui::widgets::{Block, Widget};
use ratatui::{DefaultTerminal, Frame};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    x: f64,
    y: f64,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(16);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if !event::poll(timeout)? {
                last_tick = Instant::now();
                continue;
            }
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key)
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if !key.is_press() {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.y += 1.0,
            KeyCode::Char('k') | KeyCode::Up => self.y -= 1.0,
            KeyCode::Char('l') | KeyCode::Right => self.x += 1.0,
            KeyCode::Char('h') | KeyCode::Left => self.x -= 1.0,
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let header = Text::from_iter([
            "Canvas Example".bold(),
            "<q> Quit | <enter> Change Marker | <hjkl> Move".into(),
        ]);

        let vertical = Layout::vertical([
            Constraint::Length(header.height() as u16),
            Constraint::Fill(1),
        ]);
        let [text_area, up] = frame.area().layout(&vertical);
        frame.render_widget(header.centered(), text_area);

        let horizontal = Layout::horizontal(Constraint::from_percentages([40, 65]));
        let [left, right] = up.layout(&horizontal);

        frame.render_widget(self.map_canvas(), left);
        frame.render_widget(self.pong_canvas(), right);
    }

    fn map_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Keys"))
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                });
                ctx.print(self.x, -self.y, "Keys here".yellow());
            })
    }

    fn pong_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Value"))
            .paint(|ctx| {
                ctx.print(self.x, -self.y, "Value here".yellow());
            })
    }
}
