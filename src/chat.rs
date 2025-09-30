use crate::vue::Val;
use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::Alignment,
    layout::Rect,
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
    ThinkingComplete(Duration),             // Agent finished thinking (stores duration)
    ToolUseInProgress(String, String),      // Tool being called (tool_name, arguments_json)
    ToolUseComplete(String, String, String), // Tool finished (tool_name, arguments_json, result)
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

    pub fn tool_use_in_progress(
        tool_name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            sender: Sender::Agent,
            message_type: MessageType::ToolUseInProgress(tool_name.into(), arguments.into()),
            timestamp: Local::now(),
        }
    }

    pub fn tool_use_complete(
        tool_name: impl Into<String>,
        arguments: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            sender: Sender::Agent,
            message_type: MessageType::ToolUseComplete(
                tool_name.into(),
                arguments.into(),
                result.into(),
            ),
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
    /// Indices of expanded thinking messages
    pub expanded_thinking: Val<Vec<usize>>,
    /// Index of focused message (for navigation)
    pub focused_message: Val<Option<usize>>,
}

impl ReactiveChat {
    pub fn new() -> Self {
        Self {
            messages: Val::new(vec![
                Message::system("Welcome to the chat!"),
                Message::system("Type a message below and press Ctrl+D to send."),
                // Sample interaction showing thinking and tool use flow
                Message::user("Hi"),
                Message::thinking_complete(std::time::Duration::from_secs_f64(1.2)),
                Message::thinking_complete(std::time::Duration::from_secs_f64(0.8)),
                Message::tool_use_complete(
                    "search_docs",
                    r#"{"query": "greetings"}"#,
                    "Found 3 relevant greeting examples",
                ),
                Message::agent(
                    "After careful consideration of \"Hi\", here's my thoughtful response!",
                ),
            ]),
            scroll_position: Val::new(0), // Will auto-scroll to bottom on first render
            expanded_thinking: Val::new(Vec::new()),
            focused_message: Val::new(None),
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

    /// Focus the next thinking message
    pub fn focus_next_thinking(&self) {
        let messages = self.messages.value();
        let current_focus = *self.focused_message.value();

        // Find all intermediate message indices (thinking and tool use)
        let thinking_indices: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| {
                matches!(
                    msg.message_type,
                    MessageType::ThinkingInProgress(_)
                        | MessageType::ThinkingComplete(_)
                        | MessageType::ToolUseInProgress(_, _)
                        | MessageType::ToolUseComplete(_, _, _)
                )
            })
            .map(|(idx, _)| idx)
            .collect();

        if thinking_indices.is_empty() {
            return; // No intermediate messages to focus
        }

        // Find the next index to focus
        let next_idx = match current_focus {
            Some(current) => {
                // Find current in thinking_indices and get next
                if let Some(pos) = thinking_indices.iter().position(|&i| i == current) {
                    // Move to next, wrap around if at end
                    thinking_indices[(pos + 1) % thinking_indices.len()]
                } else {
                    // Current focus is not a thinking message, focus first
                    thinking_indices[0]
                }
            }
            None => thinking_indices[0], // No focus, select first
        };

        self.focused_message.set(Some(next_idx));
        self.scroll_to_message(next_idx);
    }

    /// Focus the previous thinking message
    pub fn focus_prev_thinking(&self) {
        let messages = self.messages.value();
        let current_focus = *self.focused_message.value();

        // Find all intermediate message indices (thinking and tool use)
        let thinking_indices: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| {
                matches!(
                    msg.message_type,
                    MessageType::ThinkingInProgress(_)
                        | MessageType::ThinkingComplete(_)
                        | MessageType::ToolUseInProgress(_, _)
                        | MessageType::ToolUseComplete(_, _, _)
                )
            })
            .map(|(idx, _)| idx)
            .collect();

        if thinking_indices.is_empty() {
            return; // No intermediate messages to focus
        }

        // Find the previous index to focus
        let prev_idx = match current_focus {
            Some(current) => {
                // Find current in thinking_indices and get previous
                if let Some(pos) = thinking_indices.iter().position(|&i| i == current) {
                    // Move to previous, wrap around if at beginning
                    if pos == 0 {
                        thinking_indices[thinking_indices.len() - 1]
                    } else {
                        thinking_indices[pos - 1]
                    }
                } else {
                    // Current focus is not a thinking message, focus last
                    thinking_indices[thinking_indices.len() - 1]
                }
            }
            None => thinking_indices[thinking_indices.len() - 1], // No focus, select last
        };

        self.focused_message.set(Some(prev_idx));
        self.scroll_to_message(prev_idx);
    }

    /// Scroll to make a specific message visible
    fn scroll_to_message(&self, msg_idx: usize) {
        let messages = self.messages.value();
        if msg_idx >= messages.len() {
            return;
        }

        let expanded = self.expanded_thinking.value();

        // Calculate the actual line number for this message
        let mut line_count = 0;

        for (idx, msg) in messages.iter().enumerate() {
            if idx == msg_idx {
                // Found our message, scroll to this position
                // Scroll to show it near the top (with some offset)
                let target_scroll = (line_count as i32 - 2).max(0) as u16;
                self.scroll_position.set(target_scroll);
                return;
            }

            // Add separator line if message changed
            if idx > 0 {
                line_count += 1; // Blank line separator (now always present)
            }

            // Count lines for this message
            match &msg.message_type {
                MessageType::Normal(content) => {
                    let markdown_text = tui_markdown::from_str(content);
                    // Header (1) + content lines + bottom border (1)
                    line_count += 1 + markdown_text.lines.len() + 1;
                }
                MessageType::ThinkingInProgress(_)
                | MessageType::ThinkingComplete(_)
                | MessageType::ToolUseInProgress(_, _)
                | MessageType::ToolUseComplete(_, _, _) => {
                    // Header line always present
                    line_count += 1;

                    // If expanded, count actual content lines
                    if expanded.contains(&idx) {
                        let content_text = match &msg.message_type {
                            MessageType::ThinkingInProgress(_) => {
                                "Analyzing request...\n\nProcessing query and determining response strategy."
                            }
                            MessageType::ThinkingComplete(_) => {
                                "**Analysis complete**\n\nProcessed user query and generated comprehensive response.\n\nKey considerations:\n- Context understanding\n- Response accuracy\n- Helpful formatting"
                            }
                            MessageType::ToolUseInProgress(_, args) => args.as_str(),
                            MessageType::ToolUseComplete(_, args, result) => &format!(
                                "**Arguments:**\n```json\n{}\n```\n\n**Response:**\n{}",
                                args, result
                            ),
                            _ => "",
                        };
                        let markdown = tui_markdown::from_str(content_text);
                        line_count += markdown.lines.len() + 2; // content + spacing lines
                    }
                }
            }
        }
    }

    /// Toggle the focused thinking message expanded/collapsed
    pub fn toggle_focused_thinking(&self) {
        if let Some(focused_idx) = *self.focused_message.value() {
            let mut expanded = (*self.expanded_thinking.value()).clone();

            if let Some(pos) = expanded.iter().position(|&i| i == focused_idx) {
                // Already expanded, collapse it
                expanded.remove(pos);
            } else {
                // Not expanded, expand it
                expanded.push(focused_idx);
            }

            self.expanded_thinking.set(expanded);
        }
    }

    /// Toggle a specific thinking message by index
    pub fn toggle_thinking_at_index(&self, msg_idx: usize) {
        let messages = self.messages.value();

        // Verify it's a thinking message
        if msg_idx >= messages.len() {
            return;
        }

        if !matches!(
            messages[msg_idx].message_type,
            MessageType::ThinkingInProgress(_) | MessageType::ThinkingComplete(_)
        ) {
            return;
        }

        let mut expanded = (*self.expanded_thinking.value()).clone();

        if let Some(pos) = expanded.iter().position(|&i| i == msg_idx) {
            // Already expanded, collapse it
            expanded.remove(pos);
        } else {
            // Not expanded, expand it
            expanded.push(msg_idx);
        }

        self.expanded_thinking.set(expanded);
    }

    /// Toggle all thinking messages expanded/collapsed
    pub fn toggle_all_thinking(&self) {
        let messages = self.messages.value();
        let expanded = self.expanded_thinking.value();

        // If any are collapsed, expand all; otherwise collapse all
        let thinking_indices: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| {
                matches!(
                    msg.message_type,
                    MessageType::ThinkingInProgress(_) | MessageType::ThinkingComplete(_)
                )
            })
            .map(|(idx, _)| idx)
            .collect();

        if expanded.len() < thinking_indices.len() {
            // Some are collapsed, expand all
            self.expanded_thinking.set(thinking_indices);
        } else {
            // All are expanded, collapse all
            self.expanded_thinking.set(Vec::new());
        }
    }

    /// Clear focus
    pub fn clear_focus(&self) {
        self.focused_message.set(None);
    }

    /// Render the chat window
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let messages = self.messages.value();
        let scroll_pos = *self.scroll_position.value();
        let expanded = self.expanded_thinking.value();
        let focused = *self.focused_message.value();

        // Create the chat block
        let block = Block::default()
            .title(format!(" Chat ({} messages) ", messages.len()))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        // Calculate how many lines we need
        let inner_area = block.inner(area);
        let available_height = inner_area.height as usize;

        // Build the message content
        let mut lines = Vec::new();

        for (msg_idx, msg) in messages.iter().enumerate() {
            // Determine if we should add a separator before this message
            // Don't add separator before thinking/tool messages - they're intermediate steps
            let is_intermediate = matches!(
                msg.message_type,
                MessageType::ThinkingInProgress(_)
                    | MessageType::ThinkingComplete(_)
                    | MessageType::ToolUseInProgress(_, _)
                    | MessageType::ToolUseComplete(_, _, _)
            );

            // Check if previous message was intermediate
            let prev_was_intermediate = if msg_idx > 0 {
                matches!(
                    messages.get(msg_idx - 1).map(|m| &m.message_type),
                    Some(MessageType::ThinkingInProgress(_))
                        | Some(MessageType::ThinkingComplete(_))
                        | Some(MessageType::ToolUseInProgress(_, _))
                        | Some(MessageType::ToolUseComplete(_, _, _))
                )
            } else {
                false
            };

            // Add blank line separator, but NOT if current is intermediate OR prev was intermediate
            if msg_idx > 0 && !is_intermediate && !prev_was_intermediate {
                lines.push(Line::from(""));
            }

            // Format timestamp
            let time_str = msg.timestamp.format("%H:%M:%S").to_string();

            // Get sender info
            let sender_name = msg.sender.name();
            let sender_color = msg.sender.color();

            // Handle different message types
            match &msg.message_type {
                MessageType::Normal(content) => {
                    // If agent message follows intermediate steps, add connector
                    if prev_was_intermediate && msg.sender == Sender::Agent {
                        lines.push(Line::from(vec![Span::styled(
                            "╰─▶ Final Response",
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::DIM),
                        )]));
                    }

                    // Message content - render as markdown
                    let markdown_text = tui_markdown::from_str(content);

                    // Always use header + content format for consistency
                    let header = Line::from(vec![
                        Span::styled(
                            format!("╭─ [{}] ", time_str),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            sender_name,
                            Style::default()
                                .fg(sender_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]);
                    lines.push(header);

                    // Add content lines with left border
                    for line in markdown_text.lines {
                        let mut bordered_line =
                            vec![Span::styled("│ ", Style::default().fg(Color::DarkGray))];
                        bordered_line.extend(line.spans);
                        lines.push(Line::from(bordered_line));
                    }

                    // Add bottom border
                    lines.push(Line::from(vec![Span::styled(
                        "╰─",
                        Style::default().fg(Color::DarkGray),
                    )]));
                }
                MessageType::ThinkingInProgress(_) | MessageType::ThinkingComplete(_) => {
                    // Render thinking messages as intermediate steps in the agent's process
                    let is_expanded = expanded.contains(&msg_idx);
                    let is_focused = focused == Some(msg_idx);

                    // Check if this is the first intermediate message (T-junction needed)
                    let is_first_intermediate = if msg_idx > 0 {
                        !matches!(
                            messages.get(msg_idx - 1).map(|m| &m.message_type),
                            Some(MessageType::ThinkingInProgress(_))
                                | Some(MessageType::ThinkingComplete(_))
                                | Some(MessageType::ToolUseInProgress(_, _))
                                | Some(MessageType::ToolUseComplete(_, _, _))
                        )
                    } else {
                        true
                    };

                    // Build title and content based on thinking state
                    let (status_icon, status_label, duration_text, content_text) = match &msg
                        .message_type
                    {
                        MessageType::ThinkingInProgress(start_time) => {
                            let elapsed = start_time.elapsed();
                            let duration = format!("{:.1}", elapsed.as_secs_f64());
                            let content =
                                "Analyzing user request and formulating response strategy...";
                            ("⟳", "thinking", duration, content)
                        }
                        MessageType::ThinkingComplete(duration) => {
                            let duration_str = format!("{:.1}", duration.as_secs_f64());
                            let content = "**Analysis Complete**\n\nSuccessfully processed the user's query and prepared a comprehensive response.\n\n**Approach:**\n- Understood context and requirements\n- Evaluated best response strategy\n- Prepared structured answer";
                            ("✓", "complete", duration_str, content)
                        }
                        _ => unreachable!(),
                    };

                    // Styling - make thinking steps subtle but readable
                    let (
                        pipe_style,
                        chevron_style,
                        icon_style,
                        label_style,
                        meta_style,
                        content_style,
                    ) = if is_focused {
                        (
                            Style::default().fg(Color::Yellow),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default().fg(Color::Black).bg(Color::Yellow),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::DIM),
                            Style::default().fg(Color::Yellow),
                        )
                    } else {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                            Style::default().fg(Color::Cyan),
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::DIM),
                            Style::default().fg(Color::Gray),
                        )
                    };

                    let chevron = if is_expanded { "▼" } else { "▶" };

                    // Use T-junction (├─) for first intermediate, pipe (│ ) for subsequent
                    let pipe_prefix = if is_first_intermediate {
                        "├─"
                    } else {
                        "│ "
                    };

                    // Compact step indicator line with full-width background when focused
                    let mut title_spans = vec![
                        Span::styled(
                            pipe_prefix,
                            if is_focused {
                                Style::default().fg(Color::Yellow).bg(Color::Yellow)
                            } else {
                                pipe_style
                            },
                        ),
                        Span::styled(chevron, chevron_style),
                        Span::styled(
                            " ",
                            if is_focused {
                                Style::default().bg(Color::Yellow)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(
                            status_icon,
                            if is_focused {
                                icon_style.bg(Color::Yellow)
                            } else {
                                icon_style
                            },
                        ),
                        Span::styled(
                            " ",
                            if is_focused {
                                Style::default().bg(Color::Yellow)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(status_label, label_style),
                        Span::styled(
                            format!(" ({}s)", duration_text),
                            if is_focused {
                                meta_style.bg(Color::Yellow)
                            } else {
                                meta_style
                            },
                        ),
                    ];

                    // Add background filler to extend highlight across full width when focused
                    if is_focused {
                        title_spans.push(Span::styled(
                            " ".repeat(80), // Fill rest of line with background
                            Style::default().bg(Color::Yellow),
                        ));
                    }

                    lines.push(Line::from(title_spans));

                    // Expanded content with proper indentation and spacing
                    if is_expanded {
                        lines.push(Line::from(vec![Span::styled("│", pipe_style)]));

                        let content_markdown = tui_markdown::from_str(content_text);
                        for line in content_markdown.lines.iter() {
                            let mut content_line = vec![Span::styled("│  ", pipe_style)];
                            content_line.extend(
                                line.spans
                                    .iter()
                                    .map(|s| Span::styled(s.content.clone(), content_style)),
                            );
                            lines.push(Line::from(content_line));
                        }

                        // Closing separator
                        lines.push(Line::from(vec![Span::styled("│", pipe_style)]));
                    } else {
                        // When collapsed, add connecting pipe only if next message is NOT intermediate
                        if msg_idx + 1 < messages.len() {
                            let next_is_intermediate = matches!(
                                messages.get(msg_idx + 1).map(|m| &m.message_type),
                                Some(MessageType::ThinkingInProgress(_))
                                    | Some(MessageType::ThinkingComplete(_))
                                    | Some(MessageType::ToolUseInProgress(_, _))
                                    | Some(MessageType::ToolUseComplete(_, _, _))
                            );
                            if !next_is_intermediate {
                                lines.push(Line::from(vec![Span::styled("│", pipe_style)]));
                            }
                        }
                    }
                }
                MessageType::ToolUseInProgress(tool_name, _)
                | MessageType::ToolUseComplete(tool_name, _, _) => {
                    // Render tool use messages as intermediate steps
                    let is_expanded = expanded.contains(&msg_idx);
                    let is_focused = focused == Some(msg_idx);

                    // Check if this is the first intermediate message (T-junction needed)
                    let is_first_intermediate = if msg_idx > 0 {
                        !matches!(
                            messages.get(msg_idx - 1).map(|m| &m.message_type),
                            Some(MessageType::ThinkingInProgress(_))
                                | Some(MessageType::ThinkingComplete(_))
                                | Some(MessageType::ToolUseInProgress(_, _))
                                | Some(MessageType::ToolUseComplete(_, _, _))
                        )
                    } else {
                        true
                    };

                    // Get status icon and label
                    let (status_icon, status_label) = match &msg.message_type {
                        MessageType::ToolUseInProgress(_, _) => ("⚙", "calling tool"),
                        MessageType::ToolUseComplete(_, _, _) => ("✓", "tool complete"),
                        _ => unreachable!(),
                    };

                    // Styling for tool messages
                    let (
                        pipe_style,
                        chevron_style,
                        icon_style,
                        label_style,
                        tool_name_style,
                        content_style,
                    ) = if is_focused {
                        (
                            Style::default().fg(Color::Yellow),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default().fg(Color::Black).bg(Color::Yellow),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::DIM),
                            Style::default().fg(Color::Yellow),
                        )
                    } else {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::DIM),
                            Style::default().fg(Color::Magenta),
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::DIM),
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::DIM),
                            Style::default().fg(Color::Gray),
                        )
                    };

                    let chevron = if is_expanded { "▼" } else { "▶" };

                    // Use T-junction (├─) for first intermediate, pipe (│ ) for subsequent
                    let pipe_prefix = if is_first_intermediate {
                        "├─"
                    } else {
                        "│ "
                    };

                    // Tool use indicator line with full-width background when focused
                    let mut title_spans = vec![
                        Span::styled(
                            pipe_prefix,
                            if is_focused {
                                Style::default().fg(Color::Yellow).bg(Color::Yellow)
                            } else {
                                pipe_style
                            },
                        ),
                        Span::styled(chevron, chevron_style),
                        Span::styled(
                            " ",
                            if is_focused {
                                Style::default().bg(Color::Yellow)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(
                            status_icon,
                            if is_focused {
                                icon_style.bg(Color::Yellow)
                            } else {
                                icon_style
                            },
                        ),
                        Span::styled(
                            " ",
                            if is_focused {
                                Style::default().bg(Color::Yellow)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(status_label, label_style),
                        Span::styled(
                            format!(" → {}", tool_name),
                            if is_focused {
                                tool_name_style.bg(Color::Yellow)
                            } else {
                                tool_name_style
                            },
                        ),
                    ];

                    // Add background filler when focused
                    if is_focused {
                        title_spans.push(Span::styled(
                            " ".repeat(80),
                            Style::default().bg(Color::Yellow),
                        ));
                    }

                    lines.push(Line::from(title_spans));

                    // Expanded content
                    if is_expanded {
                        lines.push(Line::from(vec![Span::styled("│", pipe_style)]));

                        // Get content directly in the loop like thinking messages do
                        match &msg.message_type {
                            MessageType::ToolUseInProgress(_, args) => {
                                let content_markdown = tui_markdown::from_str(args);
                                for line in content_markdown.lines.iter() {
                                    let mut content_line = vec![Span::styled("│  ", pipe_style)];
                                    content_line.extend(
                                        line.spans.iter().map(|s| {
                                            Span::styled(s.content.clone(), content_style)
                                        }),
                                    );
                                    lines.push(Line::from(content_line));
                                }
                            }
                            MessageType::ToolUseComplete(_, args, result) => {
                                // Show arguments
                                lines.push(Line::from(vec![
                                    Span::styled("│  ", pipe_style),
                                    Span::styled(
                                        "Arguments:",
                                        content_style.add_modifier(Modifier::BOLD),
                                    ),
                                ]));
                                let content_markdown = tui_markdown::from_str(args);
                                for line in content_markdown.lines.iter() {
                                    let mut content_line = vec![Span::styled("│    ", pipe_style)];
                                    content_line.extend(
                                        line.spans.iter().map(|s| {
                                            Span::styled(s.content.clone(), content_style)
                                        }),
                                    );
                                    lines.push(Line::from(content_line));
                                }

                                // Show result
                                lines.push(Line::from(vec![Span::styled("│", pipe_style)]));
                                lines.push(Line::from(vec![
                                    Span::styled("│  ", pipe_style),
                                    Span::styled(
                                        "Result:",
                                        content_style.add_modifier(Modifier::BOLD),
                                    ),
                                ]));
                                let result_markdown = tui_markdown::from_str(result);
                                for line in result_markdown.lines.iter() {
                                    let mut content_line = vec![Span::styled("│    ", pipe_style)];
                                    content_line.extend(
                                        line.spans.iter().map(|s| {
                                            Span::styled(s.content.clone(), content_style)
                                        }),
                                    );
                                    lines.push(Line::from(content_line));
                                }
                            }
                            _ => {}
                        }

                        // Closing separator
                        lines.push(Line::from(vec![Span::styled("│", pipe_style)]));
                    } else {
                        // When collapsed, add connecting pipe only if next message is NOT intermediate
                        if msg_idx + 1 < messages.len() {
                            let next_is_intermediate = matches!(
                                messages.get(msg_idx + 1).map(|m| &m.message_type),
                                Some(MessageType::ThinkingInProgress(_))
                                    | Some(MessageType::ThinkingComplete(_))
                                    | Some(MessageType::ToolUseInProgress(_, _))
                                    | Some(MessageType::ToolUseComplete(_, _, _))
                            );
                            if !next_is_intermediate {
                                lines.push(Line::from(vec![Span::styled("│", pipe_style)]));
                            }
                        }
                    }
                }
            }
        }

        // Calculate scroll offset
        let total_lines = lines.len();
        let max_scroll = total_lines.saturating_sub(available_height);
        let scroll_offset = (scroll_pos as usize).min(max_scroll);

        // Update scroll_position to the clamped value
        if scroll_pos as usize > max_scroll {
            self.scroll_position.set(max_scroll as u16);
        }

        // Render main paragraph with all content integrated
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((scroll_offset as u16, 0));

        f.render_widget(paragraph, area);

        // Render scrollbar if needed
        if total_lines > available_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray));

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
