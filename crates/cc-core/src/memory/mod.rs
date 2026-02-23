//! Memory system for cc-core
//!
//! This module provides persistent storage for memories/conversations
//! using SQLite as the backend with optional FTS5 full-text search.

mod store;
mod types;

pub use store::MemoryStore;
pub use types::Memory;
