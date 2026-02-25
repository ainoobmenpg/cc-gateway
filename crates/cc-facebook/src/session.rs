//! Session management for Facebook bot

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Re-export cc_core::Message
pub use cc_core::Message as CoreMessage;

/// In-memory session store for Facebook Messenger conversations
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

/// A single conversation session
#[derive(Debug, Clone)]
pub struct Session {
    pub messages: Vec<cc_core::Message>,
    pub psid: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for Session {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            messages: Vec::new(),
            psid: String::new(),
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

    /// Get or create a session for the given PSID (Page-Scoped ID)
    pub async fn get_or_create(&self, psid: &str) -> Session {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(psid) {
            return session.clone();
        }
        drop(sessions);

        let mut sessions = self.sessions.write().await;
        let session = Session {
            psid: psid.to_string(),
            ..Default::default()
        };
        sessions.insert(psid.to_string(), session.clone());
        session
    }

    /// Add a message to a session
    pub async fn add_message(&self, psid: &str, message: cc_core::Message) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(psid.to_string()).or_insert_with(|| Session {
            psid: psid.to_string(),
            ..Default::default()
        });
        session.messages.push(message);
        session.updated_at = chrono::Utc::now();
    }

    /// Clear a session
    pub async fn clear(&self, psid: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(psid);
    }

    /// Get all session keys (for debugging/admin)
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
        let session = store.get_or_create("test-psid").await;
        assert!(session.messages.is_empty());
        assert_eq!(session.psid, "test-psid");
    }

    #[tokio::test]
    async fn test_add_message() {
        let store = InMemorySessionStore::new();
        store
            .add_message("test-psid", cc_core::Message::user("Hello"))
            .await;

        let session = store.get_or_create("test-psid").await;
        assert_eq!(session.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_session() {
        let store = InMemorySessionStore::new();
        store
            .add_message("test-psid", cc_core::Message::user("Hello"))
            .await;

        store.clear("test-psid").await;

        let session = store.get_or_create("test-psid").await;
        assert!(session.messages.is_empty());
    }
}
