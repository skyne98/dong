use crate::vue::Computed;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

/// A reactive markdown renderer that displays markdown content
/// The content is reactive - it updates automatically when the source changes
pub struct ReactiveMarkdown {
    /// Computed markdown source
    source: Computed<String>,
}

impl ReactiveMarkdown {
    /// Create a new reactive markdown widget from a computed string source
    pub fn new(source: Computed<String>) -> Self {
        Self { source }
    }

    /// Render the markdown widget in the given frame area
    pub fn render(&self, f: &mut ratatui::Frame, area: Rect, title: &str) {
        let markdown_text = self.source.value();

        // Convert markdown to Text
        let text = tui_markdown::from_str(&markdown_text);

        // Create a block for the markdown
        let block = Block::default()
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Magenta));

        // Create a paragraph with the markdown text
        let paragraph = Paragraph::new(text).block(block);

        f.render_widget(paragraph, area);
    }
}
