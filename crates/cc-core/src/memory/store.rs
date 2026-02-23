//! Memory storage implementation using SQLite

use rusqlite::{Connection, params};
use crate::memory::Memory;
use crate::Result;
use serde_json::Value as JsonValue;
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// SQLite-based storage for memories
pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    /// Create a new MemoryStore with the given database path
    pub fn new(db_path: &str) -> Result<Self> {
        debug!("Opening memory database at: {}", db_path);
        let conn = Connection::open(db_path)?;
        let store = Self { conn };
        store.init_tables()?;
        info!("MemoryStore initialized successfully");
        Ok(store)
    }

    /// Create an in-memory MemoryStore (useful for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    /// Initialize database tables
    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Try to create FTS5 virtual table for full-text search
        // This may fail if FTS5 is not compiled into SQLite
        let fts_result = self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                id UNINDEXED,
                content,
                content='memories',
                content_rowid=rowid
            )",
            [],
        );

        match fts_result {
            Ok(_) => debug!("FTS5 full-text search enabled"),
            Err(e) => debug!("FTS5 not available, falling back to LIKE search: {}", e),
        }

        Ok(())
    }

    /// Save a memory to the store
    pub fn save(&self, memory: &Memory) -> Result<()> {
        let metadata_json = serde_json::to_string(&memory.metadata)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO memories (id, content, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                memory.id,
                memory.content,
                metadata_json,
                memory.created_at.to_rfc3339(),
            ],
        )?;

        // Also update FTS index if available
        self.conn.execute(
            "INSERT OR REPLACE INTO memories_fts (rowid, id, content)
             SELECT rowid, id, content FROM memories WHERE id = ?1",
            params![memory.id],
        ).ok();

        debug!("Saved memory with id: {}", memory.id);
        Ok(())
    }

    /// Load a memory by ID
    pub fn load(&self, id: &str) -> Result<Option<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, metadata, created_at FROM memories WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let metadata_str: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let metadata: JsonValue = serde_json::from_str(&metadata_str)
                .unwrap_or(JsonValue::Null);
            let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Memory {
                id,
                content,
                metadata,
                created_at,
            })
        });

        match result {
            Ok(memory) => Ok(Some(memory)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Search memories by content (using LIKE or FTS if available)
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        // Try FTS first, fall back to LIKE search if FTS fails
        let memories = match self.search_with_fts(query, limit) {
            Ok(results) => results,
            Err(_) => self.search_with_like(query, limit)?,
        };
        debug!("Found {} memories matching query: {}", memories.len(), query);
        Ok(memories)
    }

    /// Search using FTS5 (if available)
    fn search_with_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.content, m.metadata, m.created_at
             FROM memories m
             JOIN memories_fts fts ON m.id = fts.id
             WHERE memories_fts MATCH ?1
             ORDER BY m.created_at DESC
             LIMIT ?2"
        )?;

        let memories = stmt.query_map(params![query, limit as i32], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let metadata_str: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let metadata: JsonValue = serde_json::from_str(&metadata_str)
                .unwrap_or(JsonValue::Null);
            let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Memory {
                id,
                content,
                metadata,
                created_at,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    /// Fallback search using LIKE
    fn search_with_like(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, metadata, created_at FROM memories
             WHERE content LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2"
        )?;

        let pattern = format!("%{}%", query);
        let memories = stmt.query_map(params![pattern, limit as i32], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let metadata_str: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let metadata: JsonValue = serde_json::from_str(&metadata_str)
                .unwrap_or(JsonValue::Null);
            let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Memory {
                id,
                content,
                metadata,
                created_at,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    /// Delete a memory by ID
    pub fn delete(&self, id: &str) -> Result<()> {
        // Delete from FTS index first
        self.conn.execute(
            "DELETE FROM memories_fts WHERE id = ?1",
            params![id],
        ).ok();

        // Then delete from main table
        let rows_affected = self.conn.execute(
            "DELETE FROM memories WHERE id = ?1",
            params![id],
        )?;

        if rows_affected > 0 {
            debug!("Deleted memory with id: {}", id);
        }
        Ok(())
    }

    /// List recent memories
    pub fn list_recent(&self, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, metadata, created_at FROM memories
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let memories = stmt.query_map(params![limit as i32], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let metadata_str: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let metadata: JsonValue = serde_json::from_str(&metadata_str)
                .unwrap_or(JsonValue::Null);
            let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Memory {
                id,
                content,
                metadata,
                created_at,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        debug!("Listed {} recent memories", memories.len());
        Ok(memories)
    }

    /// Count total memories
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memories",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Clear all memories
    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM memories_fts", [])?;
        self.conn.execute("DELETE FROM memories", [])?;
        info!("Cleared all memories");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_memory_store_basic() -> Result<()> {
        let store = MemoryStore::in_memory()?;

        // Create and save a memory
        let memory = Memory::new("This is a test memory about Rust programming");
        store.save(&memory)?;

        // Load it back
        let loaded = store.load(&memory.id)?;
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.content, memory.content);

        Ok(())
    }

    #[test]
    fn test_memory_search() -> Result<()> {
        let store = MemoryStore::in_memory()?;

        // Save multiple memories
        store.save(&Memory::new("Rust is a systems programming language"))?;
        store.save(&Memory::new("Python is great for scripting"))?;
        store.save(&Memory::new("JavaScript runs in the browser"))?;

        // Search for Rust-related memories
        let results = store.search("Rust", 10)?;
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("Rust"));

        Ok(())
    }

    #[test]
    fn test_memory_with_metadata() -> Result<()> {
        let store = MemoryStore::in_memory()?;

        let memory = Memory::new("Important note")
            .with_metadata(json!({
                "priority": "high",
                "tags": ["important", "note"]
            }));
        store.save(&memory)?;

        let loaded = store.load(&memory.id)?.unwrap();
        assert_eq!(loaded.metadata["priority"], "high");

        Ok(())
    }

    #[test]
    fn test_list_recent() -> Result<()> {
        let store = MemoryStore::in_memory()?;

        store.save(&Memory::new("Memory 1"))?;
        store.save(&Memory::new("Memory 2"))?;
        store.save(&Memory::new("Memory 3"))?;

        let recent = store.list_recent(2)?;
        assert_eq!(recent.len(), 2);

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<()> {
        let store = MemoryStore::in_memory()?;

        let memory = Memory::new("To be deleted");
        store.save(&memory)?;

        assert!(store.load(&memory.id)?.is_some());

        store.delete(&memory.id)?;
        assert!(store.load(&memory.id)?.is_none());

        Ok(())
    }
}
