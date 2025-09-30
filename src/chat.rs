use crate::vue::Val;
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// A single chat message
#[derive(Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

impl Message {
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            timestamp: Local::now(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new("You", content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new("System", content)
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
        eprintln!("scroll_up called, current: {}", current);
        let new_pos = current.saturating_sub(5);
        self.scroll_position.set(new_pos);
        eprintln!("scroll_up set to: {}", new_pos);
    }

    /// Scroll down in the chat  
    pub fn scroll_down(&self) {
        let current = *self.scroll_position.value();
        eprintln!("scroll_down called, current: {}", current);
        let new_pos = current.saturating_add(5);
        self.scroll_position.set(new_pos);
        eprintln!("scroll_down set to: {}", new_pos);
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
            // Add separator between messages
            if idx > 0 {
                lines.push(Line::from(""));
            }

            // Format timestamp
            let time_str = msg.timestamp.format("%H:%M:%S").to_string();
            
            // Message header with sender and timestamp
            let sender_color = if msg.sender == "You" {
                Color::Green
            } else if msg.sender == "System" {
                Color::Yellow
            } else {
                Color::Blue
            };

            let header = Line::from(vec![
                Span::styled(
                    format!("[{}] ", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    &msg.sender,
                    Style::default()
                        .fg(sender_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(":", Style::default().fg(Color::White)),
            ]);
            lines.push(header);

            // Message content - render as markdown
            let markdown_text = tui_markdown::from_str(&msg.content);
            
            // Add indentation to each markdown line
            for line in markdown_text.lines {
                let mut indented_spans = vec![Span::raw("  ")]; // Indent content
                indented_spans.extend(line.spans);
                lines.push(Line::from(indented_spans));
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
        
        // Debug: you can uncomment this to see scroll values
        eprintln!("total_lines: {}, available_height: {}, scroll_pos: {}, max_scroll: {}, scroll_offset: {}", 
                  total_lines, available_height, scroll_pos, max_scroll, scroll_offset);

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
            let mut scrollbar_state = ScrollbarState::new(max_scroll)
                .position(scroll_offset);
            
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
