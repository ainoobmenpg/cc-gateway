//! In-memory session management for Signal bot
//!
//! Thread-safe session storage using DashMap

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::time::interval;
use tracing::info;

use cc_core::{Message, Session};

/// In-memory session store for Signal chats
#[derive(Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<DashMap<String, Session>>,
    #[allow(dead_code)]
    max_sessions: usize,
    session_timeout_secs: u64,
}

impl InMemorySessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            max_sessions: 1000,
            session_timeout_secs: 3600, // 1 hour
        }
    }

    /// Create with custom settings
    pub fn with_settings(max_sessions: usize, session_timeout_secs: u64) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            max_sessions,
            session_timeout_secs,
        }
    }

    /// Get or create a session for a chat
    pub fn get_or_create(&self, chat_id: &str) -> Session {
        if let Some(session) = self.sessions.get(chat_id) {
            return session.clone();
        }

        let session = Session::new(chat_id);
        self.sessions.insert(chat_id.to_string(), session.clone());
        session
    }

    /// Get a session if it exists
    pub fn get(&self, chat_id: &str) -> Option<Session> {
        self.sessions.get(chat_id).map(|s| s.clone())
    }

    /// Update a session
    pub fn update(&self, chat_id: &str, session: Session) {
        self.sessions.insert(chat_id.to_string(), session);
    }

    /// Add a message to a session
    pub fn add_message(&self, chat_id: &str, message: Message) {
        if let Some(mut session) = self.sessions.get_mut(chat_id) {
            session.add_message(message);
        }
    }

    /// Clear a session's messages
    pub fn clear(&self, chat_id: &str) -> bool {
        if let Some(mut session) = self.sessions.get_mut(chat_id) {
            session.clear_messages();
            true
        } else {
            false
        }
    }

    /// Remove a session entirely
    pub fn remove(&self, chat_id: &str) -> Option<Session> {
        self.sessions.remove(chat_id).map(|(_, s)| s)
    }

    /// Get session count
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Start a background task to clean up expired sessions
    pub fn start_cleanup_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Check every 5 minutes
            loop {
                interval.tick().await;
                self.cleanup_expired();
            }
        })
    }

    /// Clean up expired sessions
    fn cleanup_expired(&self) {
        let now = chrono::Utc::now();
        let timeout = chrono::Duration::seconds(self.session_timeout_secs as i64);

        let expired: Vec<String> = self
            .sessions
            .iter()
            .filter(|entry| {
                let session = entry.value();
                now - session.updated_at > timeout
            })
            .map(|entry| entry.key().clone())
            .collect();

        for chat_id in expired {
            self.sessions.remove(&chat_id);
            info!("Cleaned up expired session for chat: {}", chat_id);
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_store() {
        let store = InMemorySessionStore::new();

        let session = store.get_or_create("+1234567890");
        assert!(session.is_empty());

        store.add_message("+1234567890", Message::user("Hello"));
        let updated = store.get("+1234567890").unwrap();
        assert_eq!(updated.message_count(), 1);
    }

    #[test]
    fn test_clear_session() {
        let store = InMemorySessionStore::new();
        store.get_or_create("+1234567890");
        store.add_message("+1234567890", Message::user("Hello"));

        assert!(store.clear("+1234567890"));

        let session = store.get("+1234567890").unwrap();
        assert!(session.is_empty());
    }

    #[test]
    fn test_remove_session() {
        let store = InMemorySessionStore::new();
        store.get_or_create("+1234567890");
        store.add_message("+1234567890", Message::user("Hello"));

        let removed = store.remove("+1234567890");
        assert!(removed.is_some());
        assert!(store.get("+1234567890").is_none());
    }
}
