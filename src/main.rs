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
mod vue;

use vue::{Computed, Val, computed, val};

struct ReactiveApp {
    // Reactive state
    counter: Val<i32>,
    message: Val<String>,
    is_running: Val<bool>,

    // Computed values
    doubled_counter: Computed<i32>,
    status_message: Computed<String>,

    // Non-reactive state for UI
    spinner: spinner::Spinner,
    frame_count: u8,
}

impl ReactiveApp {
    fn new() -> Self {
        let counter = val(0);
        let message = val("Hello Reactive World!".to_string());
        let is_running = val(true);

        // Create computed values
        let doubled_counter = computed({
            let counter = counter.clone();
            move || *counter.value() * 2
        });

        let status_message = computed({
            let counter = counter.clone();
            let is_running = is_running.clone();
            move || {
                if *is_running.value() {
                    format!("Running - Counter: {}", *counter.value())
                } else {
                    format!("Paused - Counter: {}", *counter.value())
                }
            }
        });

        // Set up reactive effects (none needed for this demo)

        Self {
            counter,
            message,
            is_running,
            doubled_counter,
            status_message,
            spinner: spinner::Spinner::new(),
            frame_count: 0,
        }
    }

    fn update(&mut self) {
        self.spinner.update();

        // Update reactive state
        self.frame_count += 1;
        if self.frame_count >= 5 && *self.is_running.value() {
            // Increase counter every 5 frames when running
            let current_count = *self.counter.value();
            self.counter.set(current_count + 1);
            self.frame_count = 0;
        }
    }

    fn toggle_running(&mut self) {
        let current_running = *self.is_running.value();
        self.is_running.set(!current_running);
    }

    fn increment_counter(&mut self) {
        let current_count = *self.counter.value();
        self.counter.set(current_count + 1);
    }

    fn decrement_counter(&mut self) {
        let current_count = *self.counter.value();
        self.counter.set(current_count - 1);
    }

    fn update_message(&mut self, new_message: String) {
        self.message.set(new_message);
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create reactive app state
    let mut app = ReactiveApp::new();

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
    app: &mut ReactiveApp,
) -> io::Result<()> {
    loop {
        // Update reactive app state
        app.update();

        terminal.draw(|f| ui(f, app))?;

        // Poll for events with a short timeout to keep the spinner animating
        if poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => app.toggle_running(),
                    KeyCode::Char('+') | KeyCode::Up => app.increment_counter(),
                    KeyCode::Char('-') | KeyCode::Down => app.decrement_counter(),
                    KeyCode::Char('m') => {
                        let new_msg = format!("Message at {}", *app.counter.value());
                        app.update_message(new_msg);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &ReactiveApp) {
    let size = f.area();

    // Cache reactive values to avoid borrowing conflicts during rendering
    let counter_value = *app.counter.value();
    let doubled_value = *app.doubled_counter.value();
    let message_value = app.message.value().clone();
    let status_message_value = app.status_message.value().clone();
    let is_running_value = *app.is_running.value();

    // Create a layout with reactive data display, controls, and spinner
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25), // Reactive data display
            Constraint::Percentage(25), // Controls
            Constraint::Percentage(25), // Spinner
            Constraint::Percentage(25), // Status
        ])
        .split(size);

    // Reactive data display
    let reactive_block = Block::default()
        .title("Reactive Data")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let reactive_text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![
            Span::styled("Counter: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", counter_value),
                Style::default().fg(Color::Green),
            ),
        ]),
        ratatui::text::Line::from(vec![
            Span::styled("Doubled: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", doubled_value),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        ratatui::text::Line::from(vec![
            Span::styled("Message: ", Style::default().fg(Color::White)),
            Span::styled(message_value, Style::default().fg(Color::Magenta)),
        ]),
    ]);

    let reactive_paragraph = Paragraph::new(reactive_text)
        .block(reactive_block)
        .alignment(Alignment::Left);

    f.render_widget(reactive_paragraph, chunks[0]);

    // Controls
    let controls_block = Block::default()
        .title("Controls")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let controls_text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![Span::styled(
            "SPACE: Toggle running/paused",
            Style::default().fg(Color::White),
        )]),
        ratatui::text::Line::from(vec![Span::styled(
            "+/↑: Increment counter",
            Style::default().fg(Color::White),
        )]),
        ratatui::text::Line::from(vec![Span::styled(
            "-/↓: Decrement counter",
            Style::default().fg(Color::White),
        )]),
        ratatui::text::Line::from(vec![Span::styled(
            "m: Update message",
            Style::default().fg(Color::White),
        )]),
        ratatui::text::Line::from(vec![Span::styled(
            "q: Quit",
            Style::default().fg(Color::White),
        )]),
    ]);

    let controls_paragraph = Paragraph::new(controls_text)
        .block(controls_block)
        .alignment(Alignment::Left);

    f.render_widget(controls_paragraph, chunks[1]);

    // Spinner
    app.spinner.render(f, chunks[2]);

    // Status
    let status_block = Block::default()
        .title("Status")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let status_color = if is_running_value {
        Color::Green
    } else {
        Color::Red
    };
    let status_text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![Span::styled(
            status_message_value,
            Style::default().fg(status_color),
        )]),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![Span::styled(
            "Reactive Ratatui Demo - Vue 3-like reactivity in Rust!",
            Style::default().fg(Color::Blue),
        )]),
    ]);

    let status_paragraph = Paragraph::new(status_text)
        .block(status_block)
        .alignment(Alignment::Center);

    f.render_widget(status_paragraph, chunks[3]);
}
