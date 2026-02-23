//! Error types for cc-core

use thiserror::Error;

/// Main error type for cc-core
#[derive(Error, Debug)]
pub enum Error {
    #[error("Claude API error: {0}")]
    ClaudeApi(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for cc-core
pub type Result<T> = std::result::Result<T, Error>;
