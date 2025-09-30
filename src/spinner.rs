use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};
use std::time::{Duration, Instant};

use crate::vue::{Computed, Val};

/// A reusable spinner widget that can animate and display loading states
pub struct ReactiveSpinner {
    frame: usize,
    last_update: Instant,
    style: SpinnerStyle,
    message: Computed<String>,
}

#[derive(Clone)]
pub struct SpinnerStyle {
    pub spinner_chars: Vec<&'static str>,
    pub fg_color: Color,
    pub bg_color: Option<Color>,
    pub update_interval_ms: u64,
}

impl Default for SpinnerStyle {
    fn default() -> Self {
        Self {
            spinner_chars: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"],
            fg_color: Color::Green,
            bg_color: Some(Color::Black),
            update_interval_ms: 80,
        }
    }
}

impl ReactiveSpinner {
    /// Create a new reactive spinner with a computed message
    pub fn new(message: Computed<String>) -> Self {
        Self {
            frame: 0,
            last_update: Instant::now(),
            style: SpinnerStyle::default(),
            message,
        }
    }

    /// Update the spinner animation
    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update)
            >= Duration::from_millis(self.style.update_interval_ms)
        {
            self.frame = (self.frame + 1) % self.style.spinner_chars.len();
            self.last_update = now;
        }
    }

    /// Get the current spinner character
    pub fn get_spinner_char(&self) -> &'static str {
        self.style
            .spinner_chars
            .get(self.frame)
            .copied()
            .unwrap_or("⠋")
    }

    /// Render the spinner widget to the frame (content only, no block)
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let message = self.message.value().clone();
        let spinner_text = format!("{} {}", self.get_spinner_char(), message);
        let mut style = Style::default().fg(self.style.fg_color);
        if let Some(bg_color) = self.style.bg_color {
            style = style.bg(bg_color);
        }

        let spinner_paragraph = Paragraph::new(spinner_text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(style);

        f.render_widget(spinner_paragraph, area);
    }
}

/// Legacy non-reactive spinner for backward compatibility
pub struct Spinner {
    frame: usize,
    last_update: Instant,
    style: SpinnerStyle,
    message: String,
}

impl Spinner {
    /// Create a new spinner with default style
    pub fn new() -> Self {
        Self::with_message("Processing...")
    }

    /// Create a new spinner with a custom message
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            frame: 0,
            last_update: Instant::now(),
            style: SpinnerStyle::default(),
            message: message.into(),
        }
    }

    /// Update the spinner animation
    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update)
            >= Duration::from_millis(self.style.update_interval_ms)
        {
            self.frame = (self.frame + 1) % self.style.spinner_chars.len();
            self.last_update = now;
        }
    }

    /// Update the spinner message
    pub fn update_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Get the current spinner character
    pub fn get_spinner_char(&self) -> &'static str {
        self.style
            .spinner_chars
            .get(self.frame)
            .copied()
            .unwrap_or("⠋")
    }

    /// Render the spinner widget to the frame (content only, no block)
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let spinner_text = format!("{} {}", self.get_spinner_char(), self.message);
        let mut style = Style::default().fg(self.style.fg_color);
        if let Some(bg_color) = self.style.bg_color {
            style = style.bg(bg_color);
        }

        let spinner_paragraph = Paragraph::new(spinner_text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(style);

        f.render_widget(spinner_paragraph, area);
    }
}
