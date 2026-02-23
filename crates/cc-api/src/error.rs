//! エラー型定義 (cc-api)

use thiserror::Error;

/// cc-api のエラー型
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Core error: {0}")]
    Core(#[from] cc_core::Error),
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, ApiError>;
