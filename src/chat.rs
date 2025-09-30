use crate::vue::Val;
use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::time::Duration;

/// Sender type for messages
#[derive(Clone, Debug, PartialEq)]
pub enum Sender {
    User,
    Agent,
    System,
}

impl Sender {
    pub fn name(&self) -> &str {
        match self {
            Sender::User => "You",
            Sender::Agent => "Agent",
            Sender::System => "System",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Sender::User => Color::Green,
            Sender::Agent => Color::Cyan,
            Sender::System => Color::Yellow,
        }
    }
}

/// Message type
#[derive(Clone, Debug)]
pub enum MessageType {
    Normal(String),
    ThinkingInProgress(std::time::Instant), // Agent is currently thinking (stores start time)
    ThinkingComplete(Duration), // Agent finished thinking (stores duration)
}

/// A single chat message
#[derive(Clone, Debug)]
pub struct Message {
    pub sender: Sender,
    pub message_type: MessageType,
    pub timestamp: DateTime<Local>,
}

impl Message {
    pub fn new(sender: Sender, content: impl Into<String>) -> Self {
        Self {
            sender,
            message_type: MessageType::Normal(content.into()),
            timestamp: Local::now(),
        }
    }
    
    pub fn thinking_in_progress() -> Self {
        Self {
            sender: Sender::Agent,
            message_type: MessageType::ThinkingInProgress(std::time::Instant::now()),
            timestamp: Local::now(),
        }
    }
    
    pub fn thinking_complete(duration: Duration) -> Self {
        Self {
            sender: Sender::Agent,
            message_type: MessageType::ThinkingComplete(duration),
            timestamp: Local::now(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Sender::User, content)
    }
    
    pub fn agent(content: impl Into<String>) -> Self {
        Self::new(Sender::Agent, content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Sender::System, content)
    }
}

/// A reactive chat window component that displays messages with scrolling
#[derive(Clone)]
pub struct ReactiveChat {
    /// Reactive list of messages
    pub messages: Val<Vec<Message>>,
    /// Current scroll position in lines from top (0 = top)
    scroll_position: Val<u16>,
}

impl ReactiveChat {
    pub fn new() -> Self {
        Self {
            messages: Val::new(vec![
                Message::system("Welcome to the chat!"),
                Message::system("Type a message below and press Ctrl+D to send."),
            ]),
            scroll_position: Val::new(0), // Will auto-scroll to bottom on first render
        }
    }

    /// Add a new message to the chat
    pub fn add_message(&self, message: Message) {
        let mut msgs = (*self.messages.value()).clone();
        msgs.push(message);
        self.messages.set(msgs);

        // Auto-scroll to bottom when new message arrives
        self.scroll_to_bottom();
    }

    /// Scroll to the bottom of the chat
    pub fn scroll_to_bottom(&self) {
        // Set to max value - render will clamp it appropriately
        self.scroll_position.set(u16::MAX);
    }

    /// Scroll up in the chat
    pub fn scroll_up(&self) {
        let current = *self.scroll_position.value();
        let new_pos = current.saturating_sub(5);
        self.scroll_position.set(new_pos);
    }

    /// Scroll down in the chat  
    pub fn scroll_down(&self) {
        let current = *self.scroll_position.value();
        let new_pos = current.saturating_add(5);
        self.scroll_position.set(new_pos);
    }

    /// Render the chat window
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let messages = self.messages.value();
        let scroll_pos = *self.scroll_position.value();

        // Create the chat block
        let block = Block::default()
            .title(format!(" Chat ({} messages) ", messages.len()))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        // Calculate how many messages we can display
        let inner_area = block.inner(area);
        let available_height = inner_area.height as usize;

        // Build the message lines
        let mut lines = Vec::new();
        for (idx, msg) in messages.iter().enumerate() {
            // Add separator between messages only if sender changes
            if idx > 0 {
                let prev_sender = &messages[idx - 1].sender;
                if prev_sender != &msg.sender {
                    lines.push(Line::from(""));
                }
            }

            // Format timestamp
            let time_str = msg.timestamp.format("%H:%M:%S").to_string();

            // Get sender info
            let sender_name = msg.sender.name();
            let sender_color = msg.sender.color();

            // Handle different message types
            match &msg.message_type {
                MessageType::Normal(content) => {
                    // Message content - render as markdown
                    let markdown_text = tui_markdown::from_str(content);

                    // Check if this is a single-line message
                    if markdown_text.lines.len() == 1 {
                        // Single line: display inline with header
                        let mut header_spans = vec![
                            Span::styled(
                                format!("[{}] ", time_str),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                sender_name,
                                Style::default()
                                    .fg(sender_color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(": ", Style::default().fg(Color::White)),
                        ];
                        // Add the content to the same line
                        header_spans.extend(markdown_text.lines[0].spans.clone());
                        lines.push(Line::from(header_spans));
                    } else {
                        // Multi-line: display header then indented content
                        let header = Line::from(vec![
                            Span::styled(
                                format!("[{}] ", time_str),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                sender_name,
                                Style::default()
                                    .fg(sender_color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(":", Style::default().fg(Color::White)),
                        ]);
                        lines.push(header);

                        // Add indented content lines
                        for line in markdown_text.lines {
                            let mut indented_spans = vec![Span::raw("  ")]; // Indent content
                            indented_spans.extend(line.spans);
                            lines.push(Line::from(indented_spans));
                        }
                    }
                }
                MessageType::ThinkingInProgress(start_time) => {
                    // Display animated thinking message with elapsed time
                    let elapsed = start_time.elapsed();
                    let thinking_text = format!("💭 Thinking... ({:.1}s)", elapsed.as_secs_f64());
                    
                    let header_spans = vec![
                        Span::styled(
                            format!("[{}] ", time_str),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            sender_name,
                            Style::default()
                                .fg(sender_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(": ", Style::default().fg(Color::White)),
                        Span::styled(
                            thinking_text,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ];
                    lines.push(Line::from(header_spans));
                }
                MessageType::ThinkingComplete(duration) => {
                    // Display completed thinking message
                    let thinking_text = if duration.as_secs() > 0 {
                        format!("💭 Thought for {:.1}s", duration.as_secs_f64())
                    } else {
                        format!("💭 Thought for {}ms", duration.as_millis())
                    };
                    
                    let header_spans = vec![
                        Span::styled(
                            format!("[{}] ", time_str),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            sender_name,
                            Style::default()
                                .fg(sender_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(": ", Style::default().fg(Color::White)),
                        Span::styled(
                            thinking_text,
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ];
                    lines.push(Line::from(header_spans));
                }
            }
        }

        // Calculate scroll offset to show the most recent messages
        let total_lines = lines.len();
        let max_scroll = total_lines.saturating_sub(available_height);
        let scroll_offset = (scroll_pos as usize).min(max_scroll);

        // Update scroll_position to the clamped value so next scroll works correctly
        if scroll_pos as usize > max_scroll {
            self.scroll_position.set(max_scroll as u16);
        }

        // Create paragraph with scrolling
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((scroll_offset as u16, 0));

        f.render_widget(paragraph, area);

        // Render scrollbar if needed
        if total_lines > available_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray));

            // ScrollbarState needs to know: total content size and current position
            // When scroll_offset == max_scroll, we're at the bottom
            let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll_offset);

            f.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }
}
