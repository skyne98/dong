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
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

mod ai;
mod chat;
mod collapsible;
mod textbox;
mod vue;

use crate::chat::{Message, ReactiveChat};
use crate::textbox::ReactiveTextbox;

enum AIEvent {
    ThinkingComplete(Duration),
    AgentResponse(String),
}

struct ChatApp {
    // Chat window with messages
    chat: ReactiveChat,

    // Input textbox
    textbox: Rc<RefCell<ReactiveTextbox>>,

    // Channel to receive user messages that need to be sent to AI
    user_msg_rx: mpsc::UnboundedReceiver<String>,

    // Channel sender to send AI events back to UI
    ai_event_tx: mpsc::UnboundedSender<AIEvent>,

    // Channel to receive AI responses
    ai_rx: mpsc::UnboundedReceiver<AIEvent>,

    // Handle to the current AI task (for cancellation)
    ai_task_handle: Option<JoinHandle<()>>,

    // Index of the thinking message (if any)
    thinking_message_index: Option<usize>,
}

impl ChatApp {
    fn new() -> Self {
        let chat = ReactiveChat::new();

        // Create channels for user messages
        let (user_msg_tx, user_msg_rx) = mpsc::unbounded_channel::<String>();

        // Create channels for AI events
        let (ai_event_tx, ai_rx) = mpsc::unbounded_channel::<AIEvent>();

        // Create reactive textbox for message input
        let chat_clone = chat.clone();
        let user_msg_tx_clone = user_msg_tx.clone();
        let textbox = Rc::new(RefCell::new(
            ReactiveTextbox::new("Type your message...")
                .with_validator(|_text: &Vec<String>| {
                    // Chat messages always valid - no restrictions
                    (true, String::new())
                })
                .on_submit(move |text: &Vec<String>| {
                    let message_content = text.join("\n");
                    if !message_content.trim().is_empty() {
                        // Add user message to chat
                        chat_clone.add_message(Message::user(&message_content));

                        // Send to AI processing
                        let _ = user_msg_tx_clone.send(message_content);
                    }
                }),
        ));

        Self {
            chat,
            textbox,
            user_msg_rx,
            ai_event_tx,
            ai_rx,
            ai_task_handle: None,
            thinking_message_index: None,
        }
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

    fn is_textbox_focused(&self) -> bool {
        *self.textbox.borrow().is_focused.value()
    }

    fn scroll_chat_up(&self) {
        self.chat.scroll_up();
    }

    fn scroll_chat_down(&self) {
        self.chat.scroll_down();
    }

    fn focus_next_thinking(&self) {
        self.chat.focus_next_thinking();
    }

    fn focus_prev_thinking(&self) {
        self.chat.focus_prev_thinking();
    }

    fn toggle_focused_thinking(&self) {
        self.chat.toggle_focused_thinking();
    }

    fn has_focused_message(&self) -> bool {
        self.chat.focused_message.value().is_some()
    }

    fn process_ai_events(&mut self) {
        // Check for new user messages to send to AI
        while let Ok(user_message) = self.user_msg_rx.try_recv() {
            // Cancel existing task if any
            if let Some(handle) = self.ai_task_handle.take() {
                handle.abort();
            }

            // Remove old thinking message if exists
            if let Some(idx) = self.thinking_message_index.take() {
                let mut msgs = (*self.chat.messages.value()).clone();
                if idx < msgs.len() {
                    msgs.remove(idx);
                    self.chat.messages.set(msgs);
                }
            }

            // Add new thinking message
            self.chat.add_message(Message::thinking_in_progress());
            let msg_count = self.chat.messages.value().len();
            self.thinking_message_index = Some(msg_count - 1);

            // Spawn new AI task
            let event_tx = self.ai_event_tx.clone();
            let handle = tokio::spawn(async move {
                use crate::ai::{AIService, MockAI};
                let ai = MockAI::new();

                // Get AI response with thinking
                let (thinking_duration, response) =
                    ai.send_message_with_thinking(&user_message).await;

                // Send events back to UI
                let _ = event_tx.send(AIEvent::ThinkingComplete(thinking_duration));
                let _ = event_tx.send(AIEvent::AgentResponse(response));
            });

            self.ai_task_handle = Some(handle);
        }

        // Process AI response events
        while let Ok(event) = self.ai_rx.try_recv() {
            match event {
                AIEvent::ThinkingComplete(duration) => {
                    // Replace thinking message with completed version
                    if let Some(idx) = self.thinking_message_index {
                        let mut msgs = (*self.chat.messages.value()).clone();
                        if idx < msgs.len() {
                            msgs[idx] = Message::thinking_complete(duration);
                            self.chat.messages.set(msgs);
                        }
                    }
                }
                AIEvent::AgentResponse(response) => {
                    self.chat.add_message(Message::agent(response));
                    self.thinking_message_index = None;
                    self.ai_task_handle = None;
                }
            }
        }
    }

    fn cancel_ai_task(&mut self) {
        // Cancel current AI task
        if let Some(handle) = self.ai_task_handle.take() {
            handle.abort();
        }

        // Remove thinking message
        if let Some(idx) = self.thinking_message_index.take() {
            let mut msgs = (*self.chat.messages.value()).clone();
            if idx < msgs.len() {
                msgs.remove(idx);
                self.chat.messages.set(msgs);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create chat app
    let mut app = ChatApp::new();

    // Auto-focus the textbox
    app.focus_textbox();

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
    app: &mut ChatApp,
) -> io::Result<()> {
    loop {
        // Process any pending AI events
        app.process_ai_events();

        terminal.draw(|f| ui(f, app))?;

        // Poll for events with a short timeout
        if poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Check for Ctrl+C to cancel AI task (works globally)
                if key.code == KeyCode::Char('c')
                    && key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
                {
                    app.cancel_ai_task();
                    continue;
                }

                // Handle textbox input if focused
                if app.is_textbox_focused() {
                    match key.code {
                        KeyCode::Esc => app.unfocus_textbox(),
                        KeyCode::PageUp => app.scroll_chat_up(),
                        KeyCode::PageDown => app.scroll_chat_down(),
                        _ => app.handle_textbox_key(key),
                    }
                } else {
                    // Handle global controls
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_chat_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_chat_down(),
                        KeyCode::PageUp => app.scroll_chat_up(),
                        KeyCode::PageDown => app.scroll_chat_down(),

                        // W/S: Navigate between thinking messages
                        KeyCode::Char('w') | KeyCode::Char('W') => app.focus_prev_thinking(),
                        KeyCode::Char('s') | KeyCode::Char('S') => app.focus_next_thinking(),

                        // Space: Toggle expand/collapse of focused message
                        KeyCode::Char(' ') => {
                            if app.has_focused_message() {
                                app.toggle_focused_thinking();
                            } else {
                                app.focus_textbox();
                            }
                        }

                        _ => app.focus_textbox(),
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &ChatApp) {
    let size = f.area();

    // Calculate dynamic textbox height based on content (min 3 lines, max 30% of screen)
    let textbox_lines = app.textbox.borrow().text.value().len().max(1);
    let textbox_height = (textbox_lines + 2)
        .min(size.height as usize * 30 / 100)
        .max(3); // +2 for borders

    // Create vertical layout: chat area on top, input on bottom
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                        // Chat area (takes remaining space)
            Constraint::Length(textbox_height as u16), // Input textbox (dynamic)
        ])
        .split(size);

    // Create horizontal layout for the chat area: chat on left, stats on right
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(75), // Chat messages (75%)
            Constraint::Percentage(25), // Stats panel (25%)
        ])
        .split(vertical_chunks[0]);

    // Render chat window
    app.chat.render(f, horizontal_chunks[0]);

    // Render stats panel
    let message_count = app.chat.messages.value().len();
    let stats_block = Block::default()
        .title(" Stats ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let stats_text = vec![
        Line::from(vec![
            Span::styled("Messages: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", message_count),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Controls:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(vec![Span::raw("  ↑/k - Scroll up")]),
        Line::from(vec![Span::raw("  ↓/j - Scroll down")]),
        Line::from(vec![Span::raw("  w/s - Select msg")]),
        Line::from(vec![Span::raw("  Space - Toggle")]),
        Line::from(vec![Span::raw("  PgUp/PgDn - Page")]),
        Line::from(vec![Span::raw("  Ctrl+C - Cancel AI")]),
        Line::from(vec![Span::raw("  ESC - Unfocus")]),
        Line::from(vec![Span::raw("  q - Quit")]),
    ];

    let stats_paragraph = Paragraph::new(stats_text)
        .block(stats_block)
        .alignment(Alignment::Left);

    f.render_widget(stats_paragraph, horizontal_chunks[1]);

    // Render input textbox
    app.textbox.borrow().render(
        f,
        vertical_chunks[1],
        "Message Input (Enter=newline, Ctrl+D=send, ESC=unfocus, q=quit when unfocused)",
    );
}
