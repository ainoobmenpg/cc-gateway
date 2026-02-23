//! HTTP API handlers
//!
//! Request handlers for Claude API and session management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use cc_core::llm::{Message, MessageContent, MessagesRequest};
use crate::server::AppState;

// ============================================================================
// Request/Response types
// ============================================================================

/// Chat request payload
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// User message
    pub message: String,
    /// Session ID for conversation continuity
    pub session_id: Option<String>,
    /// System prompt override
    pub system: Option<String>,
    /// Max tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u64,
}

fn default_max_tokens() -> u64 {
    4096
}

/// Token usage information
#[derive(Debug, Serialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Chat response payload
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    /// Claude's response text
    pub response: String,
    /// Session ID (for subsequent requests)
    pub session_id: String,
    /// Token usage
    pub tokens_used: Option<TokenUsage>,
}

/// Session info response
#[derive(Debug, Serialize)]
pub struct SessionInfoResponse {
    pub session_id: String,
    pub message_count: usize,
    pub created_at: String,
}

/// Memory request payload
#[derive(Debug, Deserialize)]
pub struct MemoryRequest {
    pub key: String,
    pub value: String,
}

/// Memory response payload
#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    pub success: bool,
    pub message: String,
}

/// Generic API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============================================================================
// Handler functions
// ============================================================================

/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}

/// Chat endpoint - send message to Claude
pub async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Chat request: {:?}", req);

    let session_id = req.session_id.unwrap_or_else(|| {
        uuid::Uuid::new_v4().to_string()
    });

    // Get the model from client
    let model = state.claude_client.model().to_string();

    // Build the messages request
    let messages_request = MessagesRequest {
        model,
        max_tokens: req.max_tokens,
        system: req.system,
        messages: vec![Message::user(&req.message)],
        tools: None,
    };

    // Call Claude API
    match state.claude_client.messages(messages_request).await {
        Ok(response) => {
            // Extract text from response
            let response_text = response.content
                .iter()
                .filter_map(|block| {
                    if let MessageContent::Text { text } = block {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            let tokens_used = response.usage.as_ref().map(|u| TokenUsage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
            });

            info!("Chat response: {} tokens", response_text.len());

            Ok(Json(ChatResponse {
                response: response_text,
                session_id,
                tokens_used,
            }))
        }
        Err(e) => {
            error!("Claude API error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Claude API error: {}", e),
                }),
            ))
        }
    }
}

/// Get session info
pub async fn session_info(
    Path(session_id): Path<String>,
) -> Result<Json<SessionInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Session info request: {}", session_id);

    // For now, return basic info (session management would need more implementation)
    Ok(Json(SessionInfoResponse {
        session_id,
        message_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Clear session
pub async fn clear_session(
    Path(session_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    info!("Clearing session: {}", session_id);
    Ok(StatusCode::NO_CONTENT)
}

/// Memory endpoint - save/recall memory
pub async fn memory(
    Json(req): Json<MemoryRequest>,
) -> Result<Json<MemoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Memory request: key={}", req.key);

    // Basic implementation - just acknowledge
    Ok(Json(MemoryResponse {
        success: true,
        message: format!("Memory saved: {} = {}", req.key, req.value),
    }))
}
