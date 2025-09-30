use std::cell::RefCell;
use std::io::{self, Result, stdout};
use std::rc::Rc;
use std::time::Duration;

use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, poll,
};
use ratatui::crossterm::{
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
mod textbox;
mod vue;

use spinner::ReactiveSpinner;
use textbox::ReactiveTextbox;
use vue::{Computed, Val, computed, val};

struct ReactiveApp {
    // Reactive state
    counter: Val<i32>,
    message: Val<String>,
    is_running: Val<bool>,
    status_message: Val<String>,

    // Computed values
    doubled_counter: Computed<i32>,
    spinner_message: Computed<String>,

    // Reactive textbox
    textbox: Rc<RefCell<ReactiveTextbox>>,

    // Reactive spinner
    spinner: ReactiveSpinner,
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

        let status_message = val("Running - Counter: 0".to_string());

        // Create reactive textbox
        let textbox = Rc::new(RefCell::new(
            ReactiveTextbox::new("Type something here...").on_submit({
                let status_message = status_message.clone();
                let counter = counter.clone();
                move |text: &Vec<String>| {
                    let line_count = text.len();
                    let total_chars: usize = text.iter().map(|s| s.len()).sum();
                    status_message.set(format!(
                        "✓ Submitted: {} lines, {} chars",
                        line_count, total_chars
                    ));

                    // Double the counter as visual feedback
                    let current = *counter.value();
                    counter.set(current * 2);
                }
            }),
        ));

        // Create spinner message that reacts to textbox content
        let spinner_message = computed({
            let textbox_text = textbox.borrow().text.clone();
            move || {
                let text = (*textbox_text.value()).clone();
                if text.is_empty() || (text.len() == 1 && text[0].is_empty()) {
                    "Waiting for input...".to_string()
                } else {
                    format!("Text: \"{}\"", text.join("\n"))
                }
            }
        });

        // Create reactive spinner
        let spinner = ReactiveSpinner::new(spinner_message.clone());

        Self {
            counter,
            message,
            is_running,
            doubled_counter,
            status_message,
            spinner_message,
            textbox,
            spinner,
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

        // Update status message
        let counter_value = *self.counter.value();
        let status = if *self.is_running.value() {
            format!("Running - Counter: {}", counter_value)
        } else {
            format!("Paused - Counter: {}", counter_value)
        };
        self.status_message.set(status);
    }

    fn toggle_running(&mut self) {
        let current_running = *self.is_running.value();
        self.is_running.set(!current_running);

        // Update status message immediately
        let counter_value = *self.counter.value();
        let status = if !current_running {
            format!("Running - Counter: {}", counter_value)
        } else {
            format!("Paused - Counter: {}", counter_value)
        };
        self.status_message.set(status);
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

    fn focus_textbox(&mut self) {
        self.textbox.borrow_mut().focus();
    }

    fn unfocus_textbox(&mut self) {
        self.textbox.borrow_mut().unfocus();
    }

    fn handle_textbox_key(&mut self, key: ratatui::crossterm::event::KeyEvent) {
        self.textbox.borrow_mut().handle_key(key);
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
                // Handle textbox input if focused
                if *app.textbox.borrow().is_focused.value() {
                    match key.code {
                        KeyCode::Esc => app.unfocus_textbox(),
                        _ => app.handle_textbox_key(key),
                    }
                } else {
                    // Handle global controls
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char(' ') => app.toggle_running(),
                        KeyCode::Char('+') | KeyCode::Up => app.increment_counter(),
                        KeyCode::Char('-') | KeyCode::Down => app.decrement_counter(),
                        KeyCode::Char('m') => {
                            let new_msg = format!("Message at {}", *app.counter.value());
                            app.update_message(new_msg);
                        }
                        KeyCode::Char('t') => app.focus_textbox(),
                        _ => {}
                    }
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

    // Create a layout with reactive data display, controls, textbox, and status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15), // Reactive data display
            Constraint::Percentage(25), // Controls
            Constraint::Percentage(25), // Textbox (increased)
            Constraint::Percentage(15), // Spinner
            Constraint::Percentage(20), // Status
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
            "SPACE: Toggle running/paused | +/↑: Inc counter | -/↓: Dec counter | m: Update msg | t: Focus textbox",
            Style::default().fg(Color::White),
        )]),
        ratatui::text::Line::from(vec![Span::styled(
            "Textbox: Enter for newlines | Ctrl+D to submit | ESC to unfocus",
            Style::default().fg(Color::Cyan),
        )]),
        ratatui::text::Line::from(vec![Span::styled(
            "Spinner shows textbox content dynamically!",
            Style::default().fg(Color::Magenta),
        )]),
    ]);

    let controls_paragraph = Paragraph::new(controls_text)
        .block(controls_block)
        .alignment(Alignment::Left);

    f.render_widget(controls_paragraph, chunks[1]);

    // Textbox
    app.textbox.borrow().render(
        f,
        chunks[2],
        "Reactive Textbox (Enter=newline, Ctrl+D=submit, ESC=unfocus)",
    );

    // Spinner
    app.spinner.render(f, chunks[3]);

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

    f.render_widget(status_paragraph, chunks[4]);
}
