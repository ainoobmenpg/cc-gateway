//! Session management for WhatsApp bot

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory session store for WhatsApp conversations
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

/// A single conversation session
#[derive(Debug, Clone)]
pub struct Session {
    pub messages: Vec<cc_core::Message>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for Session {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl InMemorySessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a session for the given key (phone number)
    pub async fn get_or_create(&self, key: &str) -> Session {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(key) {
            return session.clone();
        }
        drop(sessions);

        let mut sessions = self.sessions.write().await;
        let session = Session::default();
        sessions.insert(key.to_string(), session.clone());
        session
    }

    /// Add a message to a session
    pub async fn add_message(&self, key: &str, message: cc_core::Message) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(key.to_string()).or_default();
        session.messages.push(message);
        session.updated_at = chrono::Utc::now();
    }

    /// Clear a session
    pub async fn clear(&self, key: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(key);
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Clone for InMemorySessionStore {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let store = InMemorySessionStore::new();
        let session = store.get_or_create("+1234567890").await;
        assert!(session.messages.is_empty());
    }

    #[tokio::test]
    async fn test_add_message() {
        let store = InMemorySessionStore::new();
        store
            .add_message("+1234567890", cc_core::Message::user("Hello"))
            .await;

        let session = store.get_or_create("+1234567890").await;
        assert_eq!(session.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_session() {
        let store = InMemorySessionStore::new();
        store
            .add_message("+1234567890", cc_core::Message::user("Hello"))
            .await;

        store.clear("+1234567890").await;

        let session = store.get_or_create("+1234567890").await;
        assert!(session.messages.is_empty());
    }
}
