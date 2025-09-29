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
    widgets::{Block, Borders, Paragraph},
};

mod spinner;

struct App {
    spinner: spinner::Spinner,
}

impl App {
    fn new() -> Self {
        Self {
            spinner: spinner::Spinner::new(),
        }
    }

    fn update(&mut self) {
        self.spinner.update();
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

    // Create a layout with a centered block
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(20), // Increased spinner area
            Constraint::Percentage(50),
            Constraint::Percentage(20),
        ])
        .split(size);

    // Render spinner
    app.spinner.render(f, chunks[1]);

    // Create a block with borders
    let block = Block::default()
        .title("Simple Ratatui App")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    // Create a paragraph with some text
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
