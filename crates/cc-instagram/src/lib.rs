//! cc-instagram: Instagram Gateway for Claude Code
//!
//! This crate provides Instagram integration for cc-gateway,
//! allowing users to interact with Claude through Instagram Direct Messages.
//! Uses Instagram Graph API for messaging.

pub mod api;
pub mod error;
pub mod handler;
pub mod session;

pub use api::InstagramApi;
pub use error::{InstagramError, Result};
pub use handler::InstagramHandler;
pub use session::InMemorySessionStore;
