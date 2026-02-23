//! Session types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::llm::Message;

/// Represents a conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Discord channel ID this session belongs to
    pub channel_id: String,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session for a channel
    pub fn new(channel_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channel_id: channel_id.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a message to the session
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Clear all messages in the session
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.updated_at = Utc::now();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if session is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("channel-123");
        assert!(!session.id.is_empty());
        assert_eq!(session.channel_id, "channel-123");
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut session = Session::new("channel-123");
        session.add_message(Message::user("Hello"));
        assert_eq!(session.messages.len(), 1);
    }
}
