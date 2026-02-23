//! Session management

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::session::{Session, SessionStore};
use crate::llm::Message;
use crate::{Error, Result};

/// Session manager that handles session lifecycle
pub struct SessionManager {
    /// Persistent storage (wrapped in Mutex for thread safety)
    store: Arc<Mutex<SessionStore>>,
    /// In-memory cache for active sessions
    cache: Arc<RwLock<HashMap<String, Session>>>,
    /// Maximum messages per session (0 = unlimited)
    max_messages: usize,
}

impl SessionManager {
    /// Create a new session manager with a database path
    pub fn new(db_path: &str) -> Result<Self> {
        let store = SessionStore::new(db_path)?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_messages: 0,
        })
    }

    /// Create a new session manager with custom settings
    pub fn with_options(db_path: &str, max_messages: usize) -> Result<Self> {
        let store = SessionStore::new(db_path)?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_messages,
        })
    }

    /// Create an in-memory session manager (for testing)
    pub fn in_memory() -> Result<Self> {
        let store = SessionStore::in_memory()?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_messages: 0,
        })
    }

    /// Get or create a session for a channel
    pub async fn get_or_create(&self, channel_id: &str) -> Result<Session> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(session) = cache.get(channel_id) {
                debug!("Session found in cache for channel: {}", channel_id);
                return Ok(session.clone());
            }
        }

        // Try to load from store
        {
            let store = self.store.lock().unwrap();
            if let Some(session) = store.get_latest_by_channel(channel_id)? {
                debug!("Session loaded from store for channel: {}", channel_id);
                // Add to cache
                let mut cache = self.cache.write().await;
                cache.insert(channel_id.to_string(), session.clone());
                return Ok(session);
            }
        }

        // Create new session
        info!("Creating new session for channel: {}", channel_id);
        let session = Session::new(channel_id);
        {
            let store = self.store.lock().unwrap();
            store.save(&session)?;
        }

        // Add to cache
        let mut cache = self.cache.write().await;
        cache.insert(channel_id.to_string(), session.clone());

        Ok(session)
    }

    /// Add a message to a session
    pub async fn add_message(&self, channel_id: &str, message: Message) -> Result<()> {
        let mut cache = self.cache.write().await;

        let session = cache
            .get_mut(channel_id)
            .ok_or_else(|| Error::SessionNotFound(channel_id.to_string()))?;

        session.add_message(message);

        // Enforce message limit if set
        if self.max_messages > 0 && session.messages.len() > self.max_messages {
            let excess = session.messages.len() - self.max_messages;
            session.messages.drain(0..excess);
            debug!("Trimmed {} old messages from session", excess);
        }

        // Persist to store
        let store = self.store.lock().unwrap();
        store.save(session)?;

        Ok(())
    }

    /// Get all messages for a channel
    pub async fn get_messages(&self, channel_id: &str) -> Result<Vec<Message>> {
        let cache = self.cache.read().await;

        let session = cache
            .get(channel_id)
            .ok_or_else(|| Error::SessionNotFound(channel_id.to_string()))?;

        Ok(session.messages.clone())
    }

    /// Clear messages for a channel
    pub async fn clear_messages(&self, channel_id: &str) -> Result<()> {
        let mut cache = self.cache.write().await;

        if let Some(session) = cache.get_mut(channel_id) {
            session.clear_messages();
            let store = self.store.lock().unwrap();
            store.save(session)?;
            info!("Cleared messages for channel: {}", channel_id);
        }

        Ok(())
    }

    /// Delete a session completely
    pub async fn delete_session(&self, channel_id: &str) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(channel_id);
        }

        // Remove from store
        {
            let store = self.store.lock().unwrap();
            store.delete_by_channel(channel_id)?;
        }
        info!("Deleted session for channel: {}", channel_id);

        Ok(())
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Sync cache to persistent storage
    pub async fn sync(&self) -> Result<()> {
        let cache = self.cache.read().await;
        let store = self.store.lock().unwrap();
        for session in cache.values() {
            store.save(session)?;
        }
        debug!("Synced {} sessions to storage", cache.len());
        Ok(())
    }

    /// Load all active sessions from storage into cache
    pub async fn load_active_sessions(&self) -> Result<()> {
        // This would need a list_all method in store, for now we skip
        debug!("Loading active sessions not implemented yet");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_or_create() {
        let manager = SessionManager::in_memory().unwrap();

        let session1 = manager.get_or_create("channel-123").await.unwrap();
        let session2 = manager.get_or_create("channel-123").await.unwrap();

        assert_eq!(session1.id, session2.id);
    }

    #[tokio::test]
    async fn test_add_message() {
        let manager = SessionManager::in_memory().unwrap();

        manager.get_or_create("channel-123").await.unwrap();
        manager.add_message("channel-123", Message::user("Hello")).await.unwrap();

        let messages = manager.get_messages("channel-123").await.unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_messages() {
        let manager = SessionManager::in_memory().unwrap();

        manager.get_or_create("channel-123").await.unwrap();
        manager.add_message("channel-123", Message::user("Hello")).await.unwrap();

        manager.clear_messages("channel-123").await.unwrap();

        let messages = manager.get_messages("channel-123").await.unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn test_message_limit() {
        let manager = SessionManager::with_options(":memory:", 3).unwrap();

        manager.get_or_create("channel-123").await.unwrap();

        for i in 0..5 {
            manager.add_message("channel-123", Message::user(format!("Message {}", i))).await.unwrap();
        }

        let messages = manager.get_messages("channel-123").await.unwrap();
        assert_eq!(messages.len(), 3);
    }
}
