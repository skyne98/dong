use std::collections::HashMap;
use crate::vue::Val;

/// A chat session
#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub message_count: usize,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub last_modified: chrono::DateTime<chrono::Local>,
}

impl Session {
    pub fn new(title: String) -> Self {
        let now = chrono::Local::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            message_count: 0,
            created_at: now,
            last_modified: now,
        }
    }
}

/// Session service - manages chat sessions
pub struct SessionService {
    pub current_session: Val<Option<Session>>,
    pub sessions: Val<HashMap<String, Session>>,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            current_session: Val::new(None),
            sessions: Val::new(HashMap::new()),
        }
    }

    /// Create a new session
    pub fn create_session(&self, title: impl Into<String>) -> String {
        let session = Session::new(title.into());
        let id = session.id.clone();

        // For now, just set current session without updating the map
        // to avoid potential reactive issues
        self.current_session.set(Some(session));

        id
    }

    /// Switch to an existing session
    pub fn switch_session(&self, id: &str) -> bool {
        if let Some(session) = self.sessions.value().get(id).cloned() {
            self.current_session.set(Some(session));
            true
        } else {
            false
        }
    }

    /// Get all sessions
    pub fn all_sessions(&self) -> Vec<Session> {
        self.sessions.value().values().cloned().collect()
    }

    /// Update message count for current session
    pub fn increment_message_count(&self) {
        if let Some(mut session) = self.current_session.value().clone() {
            session.message_count += 1;
            session.last_modified = chrono::Local::now();

            // Update in sessions map
            let mut sessions = self.sessions.value().clone();
            sessions.insert(session.id.clone(), session.clone());
            self.sessions.set(sessions);

            // Update current
            self.current_session.set(Some(session));
        }
    }

    /// Get current session ID
    pub fn current_session_id(&self) -> Option<String> {
        self.current_session.value().as_ref().map(|s| s.id.clone())
    }

    /// Get current session title
    pub fn current_session_title(&self) -> Option<String> {
        self.current_session.value().as_ref().map(|s| s.title.clone())
    }
}

thread_local! {
    /// Global session service singleton
    pub static SESSION_SERVICE: std::cell::RefCell<SessionService> = std::cell::RefCell::new(SessionService::new());
}
