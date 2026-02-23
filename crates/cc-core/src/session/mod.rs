//! Session management module
//!
//! Provides session persistence and management for conversation history.

mod manager;
mod store;
mod types;

pub use manager::SessionManager;
pub use store::SessionStore;
pub use types::Session;
