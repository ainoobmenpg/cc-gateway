//! Route definitions
//!
//! Defines all HTTP API endpoints.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::{chat, clear_session, health, memory, session_info};
use crate::server::AppState;

/// Create the API router
pub fn routes() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Chat endpoint
        .route("/api/chat", post(chat))
        // Session management
        .route("/api/session/{session_id}", get(session_info))
        .route("/api/session/{session_id}", delete(clear_session))
        // Memory endpoint
        .route("/api/memory", post(memory))
}
