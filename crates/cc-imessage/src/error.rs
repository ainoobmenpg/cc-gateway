//! エラー型定義 (cc-imessage)

use thiserror::Error;

/// cc-imessage のエラー型
#[derive(Error, Debug)]
pub enum IMessageError {
    #[error("Apple Script execution failed: {0}")]
    ScriptError(String),

    #[error("Failed to parse Apple Script output: {0}")]
    ParseError(String),

    #[error("iMessage not available (requires macOS)")]
    NotAvailable,

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Timeout waiting for response")]
    Timeout,
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, IMessageError>;
