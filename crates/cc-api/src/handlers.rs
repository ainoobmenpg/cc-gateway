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
use cc_core::session::Session;
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

// ============================================================================
// Session Management API
// ============================================================================

/// Create session request
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Channel ID for the session
    pub channel_id: String,
}

/// Session detail response
#[derive(Debug, Serialize)]
pub struct SessionDetailResponse {
    pub id: String,
    pub channel_id: String,
    pub message_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

/// Sessions list response
#[derive(Debug, Serialize)]
pub struct SessionsListResponse {
    pub sessions: Vec<SessionDetailResponse>,
    pub total: usize,
}

impl From<Session> for SessionDetailResponse {
    fn from(session: Session) -> Self {
        let message_count = session.message_count();
        Self {
            id: session.id.clone(),
            channel_id: session.channel_id.clone(),
            message_count,
            created_at: session.created_at.to_rfc3339(),
            updated_at: session.updated_at.to_rfc3339(),
        }
    }
}

/// Create a new session
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionDetailResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Create session request: channel_id={}", req.channel_id);

    match state.session_manager.get_or_create(&req.channel_id).await {
        Ok(session) => {
            info!("Created session: {} for channel: {}", session.id, req.channel_id);
            Ok(Json(SessionDetailResponse::from(session)))
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create session: {}", e),
                }),
            ))
        }
    }
}

/// Get session by ID
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionDetailResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Get session request: id={}", session_id);

    // SessionManager の公開メソッドを使用
    match state.session_manager.get_cached_session(&session_id).await {
        Some(session) => Ok(Json(SessionDetailResponse::from(session))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Session not found: {}", session_id),
            }),
        )),
    }
}

/// Delete session by ID
pub async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    debug!("Delete session request: id={}", session_id);

    // SessionManager の公開メソッドを使用
    match state.session_manager.remove_from_cache(&session_id).await {
        Some(channel_id) => {
            // ストアからも削除
            match state.session_manager.delete_session(&channel_id).await {
                Ok(()) => {
                    info!("Deleted session: {}", session_id);
                    Ok(StatusCode::NO_CONTENT)
                }
                Err(e) => {
                    error!("Failed to delete session from store: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to delete session: {}", e),
                        }),
                    ))
                }
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Session not found: {}", session_id),
            }),
        )),
    }
}

/// List all sessions
pub async fn list_sessions(
    State(state): State<AppState>,
) -> Json<SessionsListResponse> {
    debug!("List sessions request");

    let sessions = state.session_manager.list_cached_sessions().await;
    let session_responses: Vec<SessionDetailResponse> = sessions
        .into_iter()
        .map(SessionDetailResponse::from)
        .collect();

    Json(SessionsListResponse {
        total: session_responses.len(),
        sessions: session_responses,
    })
}

// ============================================================================
// Tools API
// ============================================================================

/// Tool definition response
#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tools list response
#[derive(Debug, Serialize)]
pub struct ToolsListResponse {
    pub tools: Vec<ToolInfo>,
    pub total: usize,
}

/// Execute tool request
#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub input: serde_json::Value,
}

/// Tool execution response
#[derive(Debug, Serialize)]
pub struct ToolExecutionResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// List all available tools
pub async fn list_tools(
    State(state): State<AppState>,
) -> Json<ToolsListResponse> {
    debug!("List tools request");

    let definitions = state.tool_manager.definitions();
    let tools: Vec<ToolInfo> = definitions
        .into_iter()
        .map(|d| ToolInfo {
            name: d.name,
            description: d.description,
            input_schema: d.input_schema,
        })
        .collect();

    Json(ToolsListResponse {
        total: tools.len(),
        tools,
    })
}

/// Execute a tool by name
pub async fn execute_tool(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
    Json(req): Json<ExecuteToolRequest>,
) -> Json<ToolExecutionResponse> {
    debug!("Execute tool request: tool={}", tool_name);

    match state.tool_manager.execute(&tool_name, req.input).await {
        Ok(result) => {
            info!("Tool executed successfully: {}", tool_name);
            Json(ToolExecutionResponse {
                success: true,
                result: Some(result.output),
                error: None,
            })
        }
        Err(e) => {
            error!("Tool execution failed: {}", e);
            Json(ToolExecutionResponse {
                success: false,
                result: None,
                error: Some(format!("Tool execution failed: {}", e)),
            })
        }
    }
}

// ============================================================================
// Schedules API (Stub)
// ============================================================================

/// Schedule information
#[derive(Debug, Serialize)]
pub struct ScheduleInfo {
    pub id: String,
    pub name: String,
    pub cron_expression: String,
    pub enabled: bool,
}

/// Schedules list response
#[derive(Debug, Serialize)]
pub struct SchedulesListResponse {
    pub schedules: Vec<ScheduleInfo>,
    pub total: usize,
}

/// List all schedules (stub implementation)
pub async fn list_schedules() -> Json<SchedulesListResponse> {
    debug!("List schedules request (stub)");

    // スケジューラーは未実装のため空リストを返す
    Json(SchedulesListResponse {
        schedules: Vec::new(),
        total: 0,
    })
}
