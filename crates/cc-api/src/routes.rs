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
    delete_session, get_session, list_sessions,
    // Tools
    list_tools,
    // Schedules
    list_schedules,
};
use crate::server::AppState;

/// Create the API router (unprotected routes only)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        // Health check - no authentication required
        .route("/health", get(health))
}

/// Create the protected API router (requires authentication)
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        // Chat endpoint
        .route("/api/chat", post(chat))
        // Session management (legacy endpoints)
        .route("/api/session/:session_id", get(session_info))
        .route("/api/session/:session_id", delete(clear_session))
        // Memory endpoint
        .route("/api/memory", post(memory))
        // Session management API (GET/DELETE only for now - POST has axum 0.8 compatibility issues)
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id", delete(delete_session))
        // Tools API (GET only for now)
        .route("/api/tools", get(list_tools))
        // Schedules API
        .route("/api/schedules", get(list_schedules))
}

/// Create the full API router (for backward compatibility without auth)
pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(public_routes())
        .merge(protected_routes())
}
