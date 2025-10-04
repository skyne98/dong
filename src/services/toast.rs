use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::cell::RefCell;
use crate::vue::Val;

/// Toast notification type
#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Success,
    Info,
    Warning,
    Error,
}

/// A single toast notification
#[derive(Clone, Debug)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: String, toast_type: ToastType) -> Self {
        static mut NEXT_ID: usize = 0;
        let id = unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            id
        };

        Self {
            id,
            message,
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3), // Default 3 seconds
        }
    }

    /// Check if the toast should be dismissed
    pub fn should_dismiss(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Get the color for this toast type
    pub fn color(&self) -> ratatui::style::Color {
        use crate::services;
        services::theme(|theme| {
            match self.toast_type {
                ToastType::Success => theme.success_color(),
                ToastType::Info => theme.info_color(),
                ToastType::Warning => theme.warning_color(),
                ToastType::Error => theme.error_color(),
            }
        })
    }

    /// Get the icon for this toast type
    pub fn icon(&self) -> &'static str {
        match self.toast_type {
            ToastType::Success => "✓",
            ToastType::Info => "ℹ",
            ToastType::Warning => "⚠",
            ToastType::Error => "✗",
        }
    }
}

/// Toast service - manages app-wide notifications
pub struct ToastService {
    pub toasts: Val<VecDeque<Toast>>,
    max_toasts: usize,
}

impl ToastService {
    pub fn new() -> Self {
        Self {
            toasts: Val::new(VecDeque::new()),
            max_toasts: 5, // Maximum number of toasts to show
        }
    }

    /// Show a success toast
    pub fn success(&self, message: impl Into<String>) {
        self.show(message.into(), ToastType::Success);
    }

    /// Show an info toast
    pub fn info(&self, message: impl Into<String>) {
        self.show(message.into(), ToastType::Info);
    }

    /// Show a warning toast
    pub fn warning(&self, message: impl Into<String>) {
        self.show(message.into(), ToastType::Warning);
    }

    /// Show an error toast
    pub fn error(&self, message: impl Into<String>) {
        self.show(message.into(), ToastType::Error);
    }

    /// Show a toast with custom type
    pub fn show(&self, message: String, toast_type: ToastType) {
        let toast = Toast::new(message, toast_type);
        let mut toasts = self.toasts.value().clone();

        // Add new toast
        toasts.push_back(toast);

        // Remove old toasts if we exceed the limit
        while toasts.len() > self.max_toasts {
            toasts.pop_front();
        }

        self.toasts.set(toasts);
    }

    /// Dismiss a specific toast by ID
    pub fn dismiss(&self, id: usize) {
        let mut toasts = self.toasts.value().clone();
        toasts.retain(|toast| toast.id != id);
        self.toasts.set(toasts);
    }

    /// Clean up expired toasts
    pub fn cleanup(&self) {
        let mut toasts = self.toasts.value().clone();
        let original_len = toasts.len();
        toasts.retain(|toast| !toast.should_dismiss());

        if toasts.len() != original_len {
            self.toasts.set(toasts);
        }
    }

    /// Get current toasts (reactive)
    pub fn current_toasts(&self) -> std::cell::Ref<VecDeque<Toast>> {
        self.toasts.value()
    }
}

thread_local! {
    /// Global toast service singleton
    pub static TOAST_SERVICE: RefCell<ToastService> = RefCell::new(ToastService::new());
}
