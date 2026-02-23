//! cc-api: HTTP API for Claude Code Gateway
//!
//! Provides REST API endpoints for interacting with Claude AI.
//! Built with axum for async HTTP handling.

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod server;

pub use error::{ApiError, Result};
pub use server::start_server;
