use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// A collapsible card that can be expanded/collapsed
#[derive(Clone, Debug)]
pub struct CollapsibleCard {
    /// Title of the card
    pub title: String,
    /// Content when expanded
    pub content: Vec<Line<'static>>,
    /// Whether the card is expanded
    pub is_expanded: bool,
    /// Whether the card is focused/selected
    pub is_focused: bool,
    /// Style for the card
    pub style: Style,
}

impl CollapsibleCard {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            is_expanded: false,
            is_focused: false,
            style: Style::default(),
        }
    }

    pub fn content(mut self, content: Vec<Line<'static>>) -> Self {
        self.content = content;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    /// Render the card and return the number of lines it takes
    pub fn height(&self) -> usize {
        if self.is_expanded {
            1 + self.content.len() // Title line + content lines
        } else {
            1 // Just the title line
        }
    }
}

impl Widget for CollapsibleCard {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chevron = if self.is_expanded { "▼" } else { "▶" };

        // Create a much better visual design with thinner lines
        let (title_style, content_style) = if self.is_focused {
            (
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            )
        } else {
            (
                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            )
        };

        // Render title line with simpler formatting (thinner lines)
        let prefix = if self.is_focused { "│ " } else { "│ " };
        let title_line = Line::from(vec![
            Span::styled(prefix, title_style),
            Span::styled(chevron, title_style),
            Span::raw(" "),
            Span::styled(&self.title, title_style),
        ]);

        let mut lines = vec![title_line];

        // Add content if expanded with simpler visual hierarchy
        if self.is_expanded {
            for (idx, line) in self.content.iter().enumerate() {
                let is_last = idx == self.content.len() - 1;
                let line_prefix = if is_last { "└ " } else { "├ " };

                let mut indented_line = vec![Span::styled(line_prefix, content_style)];
                indented_line.extend(
                    line.spans
                        .iter()
                        .map(|s| Span::styled(s.content.clone(), content_style)),
                );
                lines.push(Line::from(indented_line));
            }
        }

        // Clear the background first to prevent content bleeding through
        let clear_style = Style::default().bg(Color::Reset);
        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buf.cell_mut((area.x + x, area.y + y)).unwrap();
                cell.set_symbol(" ");
                cell.set_style(clear_style);
            }
        }

        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}
