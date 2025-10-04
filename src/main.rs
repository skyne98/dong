use std::io::{self, Result};
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::crossterm::{execute};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text,
    widgets::{Block, Borders, Paragraph},
};
use std::cell::RefCell;
use std::rc::Rc;

mod services;
mod textbox;
mod vue;

use crate::textbox::ReactiveTextbox;

struct DemoApp {
    textbox: Rc<RefCell<ReactiveTextbox>>,
}

impl DemoApp {
    fn new() -> Self {
        let textbox = Rc::new(RefCell::new(
            ReactiveTextbox::new("Type a command... (Ctrl+D to submit)")
                .with_validator(|text: &Vec<String>| {
                    let cmd = text.join(" ");
                    match cmd.as_str() {
                        "toast" => (true, String::new()),
                        "session" => (true, String::new()),
                        "status" => (true, String::new()),
                        "theme" => (true, String::new()),
                        "" => (true, String::new()),
                        _ => (false, "Unknown command. Try: toast, session, status, theme".to_string()),
                    }
                })
                .on_submit(|text: &Vec<String>| {
                    let cmd = text.join(" ");
                    match cmd.as_str() {
                        "toast" => {
                            crate::services::toast_mut(|toast| {
                                toast.success("Toast notification!");
                                toast.info("This is an info message");
                                toast.warning("Warning message");
                                toast.error("Error message");
                            });
                        }
                        "session" => {
                            crate::services::session_mut(|session| {
                                let id = session.create_session("New Session");
                                crate::services::toast_mut(|toast| {
                                    toast.success(&format!("Created session: {}", &id[..8]));
                                });
                            });
                        }
                        "status" => {
                            crate::services::status_mut(|status| {
                                status.set_model("gpt-4o");
                                status.set_working(true);
                            });
                        }
                        "theme" => {
                            // Toggle between light and dark modes
                            let new_mode = crate::services::theme(|theme| {
                                use crate::services::theme::ThemeMode;
                                match &*theme.mode.value() {
                                    ThemeMode::Light => ThemeMode::Dark,
                                    ThemeMode::Dark => ThemeMode::Light,
                                    ThemeMode::Auto => ThemeMode::Light,
                                }
                            });
                            
                            crate::services::theme_mut(|theme| {
                                theme.set_mode(new_mode.clone());
                            });
                            
                            crate::services::toast_mut(|toast| {
                                toast.info(&format!("Theme: {:?}", new_mode));
                            });
                        }
                        _ => {}
                    }
                }),
        ));

        // Focus the textbox so it can accept input
        textbox.borrow_mut().focus();

        Self { textbox }
    }

    fn draw(&self, f: &mut Frame) {
        let size = f.area();

        // Get theme colors
        let (accent_color, border_color, text_muted) = crate::services::theme(|theme| {
            (theme.accent_color(), theme.border_color(), theme.secondary_color())
        });

        // Check if we have an active session
        let has_session = crate::services::session(|session| {
            session.current_session_title().is_some()
        });

        // Split screen based on whether we have a session
        let main_chunks = if has_session {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Session header
                    Constraint::Min(3),     // Content
                    Constraint::Length(3),  // Input
                    Constraint::Length(1),  // Status bar
                ])
                .split(size)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),     // Content (home screen)
                    Constraint::Length(3),  // Input
                    Constraint::Length(1),  // Status bar
                ])
                .split(size)
        };

        let (content_idx, input_idx, status_idx) = if has_session {
            (1, 2, 3)
        } else {
            (0, 1, 2)
        };

        // Render session header if active
        if has_session {
            let session_title = crate::services::session(|session| {
                session.current_session_title().unwrap_or_default()
            });
            
            let header_lines = vec![
                text::Line::from(format!("# {}", session_title)).style(Style::default().fg(accent_color)),
                text::Line::from("  0K/0% ($0.00)            /share to create link").style(Style::default().fg(text_muted)),
            ];
            
            let header = Paragraph::new(header_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)));
            f.render_widget(header, main_chunks[0]);
        }

        // Main content area
        if has_session {
            // Chat view - messages will go here
            let content = vec![
                text::Line::from(""),
                text::Line::from("Chat messages will appear here..."),
                text::Line::from(""),
            ];
            f.render_widget(Paragraph::new(content), main_chunks[content_idx]);
        } else {
            // Home screen with ASCII logo
            let logo_lines = vec![
                "█▀▀▄  █▀▀█  █▀▀▄  █▀▀▀",
                "█  █  █  █  █  █  █ ▀█",
                "▀▀▀   ▀▀▀▀  ▀  ▀  ▀▀▀▀",
            ];
            
            let mut home_content = vec![
                text::Line::from(""),
                text::Line::from(""),
            ];
            
            // Center the logo
            for line in logo_lines {
                let padding = (size.width.saturating_sub(line.len() as u16)) / 2;
                home_content.push(
                    text::Line::from(format!("{}{}", " ".repeat(padding as usize), line))
                        .style(Style::default().fg(accent_color))
                );
            }
            
            let version_text = format!("v{}", env!("CARGO_PKG_VERSION"));
            let version_padding = (size.width.saturating_sub(version_text.len() as u16)) / 2;
            
            home_content.extend(vec![
                text::Line::from(""),
                text::Line::from(format!("{}{}", " ".repeat(version_padding as usize), version_text))
                    .style(Style::default().fg(text_muted)),
                text::Line::from(""),
                text::Line::from(""),
                text::Line::from("  Available commands:"),
                text::Line::from("    toast    - Show toast notifications"),
                text::Line::from("    session  - Create a new session"),
                text::Line::from("    status   - Update status bar"),
                text::Line::from("    theme    - Toggle light/dark mode"),
                text::Line::from(""),
            ]);
            
            f.render_widget(Paragraph::new(home_content), main_chunks[content_idx]);
        }

        // Render toast notifications
        let secondary_color = crate::services::theme(|theme| theme.secondary_color());
        crate::services::toast_mut(|toast_service| {
            let toasts = toast_service.current_toasts();
            if !toasts.is_empty() {
                // Show up to 5 toasts stacked vertically from top-right
                let max_toasts = 5;
                let toast_height = 4;
                
                // Start toasts below the session header if it exists (header is 3 lines)
                let toast_start_y = if has_session { 4 } else { 1 };
                
                for (i, toast) in toasts.iter().rev().take(max_toasts).enumerate() {
                    let elapsed = toast.created_at.elapsed().as_secs();
                    let total = toast.duration.as_secs();
                    let remaining = total.saturating_sub(elapsed);
                    
                    let toast_text = vec![
                        text::Line::from(format!("{} {}", toast.icon(), toast.message)),
                        text::Line::from(format!("{}s", remaining)).style(Style::default().fg(secondary_color)),
                    ];
                    
                    let toast_widget = Paragraph::new(toast_text)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(toast.color())));
                    
                    let toast_area = Rect {
                        x: size.width.saturating_sub(50),
                        y: toast_start_y + (i as u16 * toast_height),
                        width: 48,
                        height: toast_height,
                    };
                    f.render_widget(toast_widget, toast_area);
                }
            }
        });

        // Input area (with OpenCode-style prompt)
        self.textbox.borrow().render(f, main_chunks[input_idx], "");
        
        // Status bar (OpenCode-style)
        let status_text = crate::services::status(|status| {
            format!("dong v{}    {}", env!("CARGO_PKG_VERSION"), status.status_text())
        });
        
        let status_bar = Paragraph::new(text::Line::from(status_text))
            .style(Style::default().fg(text_muted));
        f.render_widget(status_bar, main_chunks[status_idx]);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => return false, // Exit (only way to quit)
                KeyCode::Tab => {
                    // Focus next
                }
                _ => {
                    self.textbox.borrow_mut().handle_key(key);
                }
            },
            _ => {}
        }
        true
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = DemoApp::new();

    // Initialize services
    crate::services::session_mut(|session| {
        session.create_session("Demo Session");
    });

    crate::services::status_mut(|status| {
        status.set_model("demo");
    });

    // Main loop
    loop {
        // Draw
        terminal.draw(|f| {
            app.draw(f);
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if !app.handle_event(Event::Key(key)) {
                    break;
                }
            }
        }

        // Clean up expired toasts
        crate::services::toast_mut(|toast| {
            toast.cleanup();
        });

        // Small delay to prevent busy loop
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
