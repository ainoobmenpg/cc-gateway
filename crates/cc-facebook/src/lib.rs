//! cc-facebook: Facebook Messenger Gateway for cc-gateway
//!
//! This crate provides Facebook Messenger bot integration for cc-gateway,
//! allowing users to interact with Claude through Facebook Messenger.

pub mod api;
pub mod error;
pub mod handler;
pub mod session;

pub use api::FacebookApi;
pub use error::{FacebookError, Result};
pub use handler::FacebookHandler;
pub use session::InMemorySessionStore;
