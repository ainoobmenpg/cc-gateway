//! Session persistence using SQLite

use rusqlite::{Connection, params};
use crate::session::Session;
use crate::llm::Message;
use crate::{Error, Result};
use chrono::{DateTime, Utc};

/// SQLite-based session store
pub struct SessionStore {
    conn: Connection,
}

impl SessionStore {
    /// Create a new session store with the given database path
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    /// Create an in-memory session store (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    /// Initialize database tables
    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                messages TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create index for channel_id queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_channel_id ON sessions(channel_id)",
            [],
        )?;

        Ok(())
    }

    /// Save a session to the database
    pub fn save(&self, session: &Session) -> Result<()> {
        let messages_json = serde_json::to_string(&session.messages)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (id, channel_id, messages, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                session.id,
                session.channel_id,
                messages_json,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Load a session by ID
    pub fn load(&self, id: &str) -> Result<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, messages, created_at, updated_at FROM sessions WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            let messages_json: String = row.get(2)?;
            let messages: Vec<Message> = serde_json::from_str(&messages_json)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc);

            Ok(Session {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                messages,
                created_at,
                updated_at,
            })
        });

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Delete a session by ID
    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// List all sessions for a channel
    pub fn list_by_channel(&self, channel_id: &str) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, messages, created_at, updated_at FROM sessions
             WHERE channel_id = ?1 ORDER BY updated_at DESC"
        )?;

        let sessions = stmt.query_map(params![channel_id], |row| {
            let messages_json: String = row.get(2)?;
            let messages: Vec<Message> = serde_json::from_str(&messages_json)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc);

            Ok(Session {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                messages,
                created_at,
                updated_at,
            })
        })?;

        let mut result = Vec::new();
        for session in sessions {
            result.push(session?);
        }
        Ok(result)
    }

    /// Get the most recent session for a channel
    pub fn get_latest_by_channel(&self, channel_id: &str) -> Result<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, messages, created_at, updated_at FROM sessions
             WHERE channel_id = ?1 ORDER BY updated_at DESC LIMIT 1"
        )?;

        let result = stmt.query_row(params![channel_id], |row| {
            let messages_json: String = row.get(2)?;
            let messages: Vec<Message> = serde_json::from_str(&messages_json)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc);

            Ok(Session {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                messages,
                created_at,
                updated_at,
            })
        });

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Delete all sessions for a channel
    pub fn delete_by_channel(&self, channel_id: &str) -> Result<usize> {
        let affected = self.conn.execute(
            "DELETE FROM sessions WHERE channel_id = ?1",
            params![channel_id],
        )?;
        Ok(affected)
    }

    /// Count sessions for a channel
    pub fn count_by_channel(&self, channel_id: &str) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE channel_id = ?1",
            params![channel_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = SessionStore::in_memory().unwrap();
        assert!(store.list_by_channel("test").is_ok());
    }

    #[test]
    fn test_save_and_load() {
        let store = SessionStore::in_memory().unwrap();
        let mut session = Session::new("channel-123");
        session.add_message(Message::user("Hello"));

        store.save(&session).unwrap();
        let loaded = store.load(&session.id).unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.channel_id, "channel-123");
        assert_eq!(loaded.messages.len(), 1);
    }

    #[test]
    fn test_delete() {
        let store = SessionStore::in_memory().unwrap();
        let session = Session::new("channel-123");

        store.save(&session).unwrap();
        store.delete(&session.id).unwrap();

        let loaded = store.load(&session.id).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_list_by_channel() {
        let store = SessionStore::in_memory().unwrap();

        let session1 = Session::new("channel-123");
        let session2 = Session::new("channel-123");
        let session3 = Session::new("channel-456");

        store.save(&session1).unwrap();
        store.save(&session2).unwrap();
        store.save(&session3).unwrap();

        let sessions = store.list_by_channel("channel-123").unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
