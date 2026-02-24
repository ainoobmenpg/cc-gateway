//! Error types for cc-ws

use thiserror::Error;

/// WebSocket error type
#[derive(Error, Debug)]
pub enum WsError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Core error: {0}")]
    Core(#[from] cc_core::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Channel send error: {0}")]
    ChannelSend(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for cc-ws
pub type Result<T> = std::result::Result<T, WsError>;
