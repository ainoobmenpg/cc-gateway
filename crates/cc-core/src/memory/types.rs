//! Memory type definitions for cc-core

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// A memory entry stored in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier for the memory
    pub id: String,
    /// The content of the memory
    pub content: String,
    /// Optional metadata associated with the memory
    pub metadata: JsonValue,
    /// When the memory was created
    pub created_at: DateTime<Utc>,
}

impl Memory {
    /// Create a new memory with the given content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: JsonValue::Null,
            created_at: Utc::now(),
        }
    }

    /// Create a new memory with a specific ID
    pub fn with_id(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: JsonValue::Null,
            created_at: Utc::now(),
        }
    }

    /// Add metadata to the memory
    pub fn with_metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the creation timestamp
    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_memory_new() {
        let memory = Memory::new("Test content");
        assert!(!memory.id.is_empty());
        assert_eq!(memory.content, "Test content");
        assert!(memory.metadata.is_null());
    }

    #[test]
    fn test_memory_with_metadata() {
        let memory = Memory::new("Test content").with_metadata(json!({
            "source": "test",
            "importance": 5
        }));
        assert_eq!(memory.metadata["source"], "test");
        assert_eq!(memory.metadata["importance"], 5);
    }
}
