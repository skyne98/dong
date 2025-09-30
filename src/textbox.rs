use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::{Input, TextArea};

use crate::vue::{Computed, Val, computed, val};

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
    /// Validation function
    pub validator: Option<Box<dyn Fn(&Vec<String>) -> (bool, String) + 'static>>,
    /// Reactive validation state (is_valid, error_message)
    pub is_valid: Computed<bool>,
    pub validation_message: Computed<String>,
}

impl ReactiveTextbox {
    /// Create a new reactive textbox
    pub fn new(placeholder: impl Into<String>) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text(placeholder);
        textarea.set_block(Block::default().borders(Borders::ALL));

        let is_focused = val(false);
        let text = val(vec![String::new()]);

        // Create computed validation state
        let is_valid = computed({
            let text = text.clone();
            move || {
                let lines = (*text.value()).clone();
                !lines.is_empty() && !(lines.len() == 1 && lines[0].is_empty())
            }
        });

        let validation_message = computed({
            let text = text.clone();
            move || {
                let lines = (*text.value()).clone();
                if lines.is_empty() || (lines.len() == 1 && lines[0].is_empty()) {
                    "Text cannot be empty".to_string()
                } else {
                    "Valid".to_string()
                }
            }
        });

        Self {
            textarea,
            text,
            is_focused,
            on_submit: None,
            validator: None,
            is_valid,
            validation_message,
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

    /// Set a custom validator and rebuild validation computed values
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Vec<String>) -> (bool, String) + 'static,
    {
        let text = self.text.clone();
        let validator_box = Box::new(validator);
        let validator_rc = std::rc::Rc::new(validator_box);

        // Rebuild is_valid with custom validator
        self.is_valid = computed({
            let text = text.clone();
            let validator = validator_rc.clone();
            move || {
                let lines = (*text.value()).clone();
                let (is_valid, _) = validator(&lines);
                is_valid
            }
        });

        // Rebuild validation_message with custom validator
        self.validation_message = computed({
            let text = text.clone();
            let validator = validator_rc.clone();
            move || {
                let lines = (*text.value()).clone();
                let (_, message) = validator(&lines);
                message
            }
        });

        // We can't store the Rc-wrapped validator in the Option field, so we leave it as None
        // The actual validation logic is now in the computed properties
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
            // Only submit if validation passes
            if *self.is_valid.value() {
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
            }
            // If validation fails, do nothing (validation error is shown in the title)
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
        let is_valid = *self.is_valid.value();
        let validation_msg = self.validation_message.value().clone();

        // Choose border color based on validation state and focus
        let border_color = if !is_valid {
            if is_focused {
                Color::Red // Focused and invalid - bright red
            } else {
                Color::DarkGray // Unfocused and invalid - dark gray to show "inactive but still wrong"
            }
        } else if is_focused {
            Color::Yellow // Focused and valid
        } else {
            Color::White // Unfocused and valid
        };

        // Create title with validation message if invalid (show in both focused and unfocused states)
        let display_title = if !is_valid {
            format!(" {} - {} ", title, validation_msg)
        } else {
            format!(" {} ", title)
        };

        // Create block with appropriate style
        let block = Block::default()
            .title(display_title)
            .title_alignment(ratatui::layout::Alignment::Left)
            .borders(Borders::ALL)
            .style(Style::default().fg(border_color));

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
