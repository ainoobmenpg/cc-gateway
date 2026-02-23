//! Route definitions
//!
//! Defines all HTTP API endpoints.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::{
    chat, clear_session, health, memory, session_info,
    // Session management
    create_session, get_session, delete_session, list_sessions,
    // Tools
    list_tools, execute_tool,
    // Schedules
    list_schedules,
};
use crate::server::AppState;

/// Create the API router
pub fn routes() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Chat endpoint
        .route("/api/chat", post(chat))
        // Session management (legacy endpoints)
        .route("/api/session/:session_id", get(session_info))
        .route("/api/session/:session_id", delete(clear_session))
        // Memory endpoint
        .route("/api/memory", post(memory))
        // Session management API
        .route("/api/sessions", post(create_session))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id", delete(delete_session))
        // Tools API
        .route("/api/tools", get(list_tools))
        .route("/api/tools/:name", post(execute_tool))
        // Schedules API
        .route("/api/schedules", get(list_schedules))
}
