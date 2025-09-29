use std::io::{self, Result, stdout};
use std::time::{Duration, Instant};

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

struct App {
    spinner_frame: usize,
    last_update: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            spinner_frame: 0,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(80) {
            self.spinner_frame = (self.spinner_frame + 1) % 8; // 8 frames now
            self.last_update = now;
        }
    }

    fn get_spinner_char(&self) -> &'static str {
        match self.spinner_frame {
            0 => "⠋",
            1 => "⠙",
            2 => "⠹",
            3 => "⠸",
            4 => "⠼",
            5 => "⠴",
            6 => "⠦",
            7 => "⠧",
            _ => "⠋",
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

    // Create spinner block
    let spinner_block = Block::default()
        .title("Loading")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let spinner_text = format!(
        "{} Processing... (Frame: {})",
        app.get_spinner_char(),
        app.spinner_frame
    );
    let spinner_paragraph = Paragraph::new(spinner_text)
        .block(spinner_block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    f.render_widget(spinner_paragraph, chunks[1]);

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
