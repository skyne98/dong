use std::io::{self, Result, stdout};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, poll},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Paragraph},
};

mod spinner;
mod vue;

struct App {
    spinner: spinner::Spinner,
    progress: u8,
    frame_count: u8,
}

impl App {
    fn new() -> Self {
        Self {
            spinner: spinner::Spinner::new(),
            progress: 0,
            frame_count: 0,
        }
    }

    fn update(&mut self) {
        self.spinner.update();
        // Simulate progress increasing over time (slower)
        self.frame_count += 1;
        if self.frame_count >= 3 && self.progress < 100 {
            // Increase every 3 frames
            self.progress += 1;
            self.frame_count = 0;
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run the app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Update app state
        app.update();

        terminal.draw(|f| ui(f, app))?;

        // Poll for events with a short timeout to keep the spinner animating
        if poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create a layout with progress bar, spinner, and main content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20), // Progress bar - increased from 10%
            Constraint::Percentage(20), // Spinner
            Constraint::Percentage(40), // Main content - reduced
            Constraint::Percentage(20), // Bottom spacing
        ])
        .split(size);

    // Progress bar
    let progress_bar = Gauge::default()
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(app.progress as u16)
        .label(format!("{}%", app.progress));

    f.render_widget(progress_bar, chunks[0]);

    // Spinner
    app.spinner.render(f, chunks[1]);

    // Main content
    let block = Block::default()
        .title("Simple Ratatui App")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![Span::styled(
            "Welcome to Ratatui!",
            Style::default().fg(Color::Cyan),
        )]),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from("This is a simple terminal user interface built with Ratatui."),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![Span::styled(
            "Press 'q' to quit",
            Style::default().fg(Color::Yellow),
        )]),
    ]);
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, chunks[2]);
}
