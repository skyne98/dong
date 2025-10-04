use crate::vue::Val;

/// Status bar information
#[derive(Clone, Debug)]
pub struct StatusInfo {
    pub version: String,
    pub current_model: String,
    pub session_name: Option<String>,
    pub working: bool,
    pub interrupt_hint: bool,
}

impl Default for StatusInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            current_model: "gpt-4o".to_string(),
            session_name: None,
            working: false,
            interrupt_hint: false,
        }
    }
}

/// Status service - manages status bar state
pub struct StatusService {
    pub info: Val<StatusInfo>,
}

impl StatusService {
    pub fn new() -> Self {
        Self {
            info: Val::new(StatusInfo::default()),
        }
    }

    /// Set the current AI model
    pub fn set_model(&self, model: impl Into<String>) {
        let mut info = self.info.value().clone();
        info.current_model = model.into();
        self.info.set(info);
    }

    /// Set the session name
    pub fn set_session(&self, name: Option<String>) {
        let mut info = self.info.value().clone();
        info.session_name = name;
        self.info.set(info);
    }

    /// Set working state
    pub fn set_working(&self, working: bool) {
        let mut info = self.info.value().clone();
        info.working = working;
        self.info.set(info);
    }

    /// Set interrupt hint
    pub fn set_interrupt_hint(&self, hint: bool) {
        let mut info = self.info.value().clone();
        info.interrupt_hint = hint;
        self.info.set(info);
    }

    /// Get the formatted status text
    pub fn status_text(&self) -> String {
        let info = self.info.value();
        let mut parts = vec![];

        // Version
        parts.push(format!("dong v{}", info.version));

        // Session info
        if let Some(session) = &info.session_name {
            parts.push(format!("~/{}:{}", session, info.current_model));
        } else {
            parts.push(format!("~/new:{}", info.current_model));
        }

        // Working indicator
        if info.working {
            if info.interrupt_hint {
                parts.push("working... esc interrupt".to_string());
            } else {
                parts.push("working...".to_string());
            }
        } else {
            parts.push("ready".to_string());
        }

        parts.join("    ")
    }
}

thread_local! {
    /// Global status service singleton
    pub static STATUS_SERVICE: std::cell::RefCell<StatusService> = std::cell::RefCell::new(StatusService::new());
}
