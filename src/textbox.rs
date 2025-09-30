use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use tui_textarea::{Input, TextArea};

use crate::vue::{Val, val};

/// A reactive textbox component that wraps tui-textarea and integrates with the Vue-like reactivity system
pub struct ReactiveTextbox {
    /// The underlying tui-textarea widget
    textarea: TextArea<'static>,
    /// The reactive text content (updated on every change)
    pub text: Val<Vec<String>>,
    /// Whether the textbox is focused
    pub is_focused: Val<bool>,
    /// Submit callback
    pub on_submit: Option<Box<dyn Fn(&Vec<String>) + 'static>>,
}

impl ReactiveTextbox {
    /// Create a new reactive textbox
    pub fn new(placeholder: impl Into<String>) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text(placeholder);
        textarea.set_block(Block::default().borders(Borders::ALL));

        let is_focused = val(false);
        let text = val(vec![String::new()]);

        Self {
            textarea,
            text,
            is_focused,
            on_submit: None,
        }
    }

    /// Get the current text content
    pub fn text(&self) -> Vec<String> {
        self.textarea.lines().to_vec()
    }

    /// Set the submit callback
    pub fn on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Vec<String>) + 'static,
    {
        self.on_submit = Some(Box::new(callback));
        self
    }

    /// Focus the textbox
    pub fn focus(&mut self) {
        self.is_focused.set(true);
    }

    /// Unfocus the textbox
    pub fn unfocus(&mut self) {
        self.is_focused.set(false);
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) {
        if !*self.is_focused.value() {
            return;
        }

        // Check for submit with Ctrl+D
        if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
            if let Some(ref callback) = self.on_submit {
                let lines = self.textarea.lines().to_vec();
                callback(&lines);

                // Clear the textarea after submit for visual feedback
                while !self.textarea.lines().is_empty() {
                    self.textarea.delete_line_by_head();
                    self.textarea.move_cursor(tui_textarea::CursorMove::Head);
                    if self.textarea.lines().len() == 1 && self.textarea.lines()[0].is_empty() {
                        break;
                    }
                }
                self.text.set(vec![String::new()]);
            }
            return;
        }

        // Let tui-textarea handle all other keys (including Enter for newlines)
        let input: Input = key.into();
        self.textarea.input(input);

        // Update reactive text after input
        let current_lines = self.textarea.lines().to_vec();
        self.text.set(current_lines);
    }

    /// Render the textbox
    pub fn render(&self, f: &mut Frame, area: Rect, title: &str) {
        let is_focused = *self.is_focused.value();

        // Create block with appropriate style
        let block = Block::default()
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Left)
            .borders(Borders::ALL)
            .style(if is_focused {
                ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
            } else {
                ratatui::style::Style::default().fg(ratatui::style::Color::White)
            });

        // Clone the textarea and set the block on the clone
        let mut textarea_with_block = self.textarea.clone();
        textarea_with_block.set_block(block);
        f.render_widget(&textarea_with_block, area);
    }
}

impl Default for ReactiveTextbox {
    fn default() -> Self {
        Self::new("")
    }
}
