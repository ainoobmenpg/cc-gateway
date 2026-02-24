//! エラー型定義 (cc-signal)

use thiserror::Error;

/// cc-signal のエラー型
#[derive(Error, Debug)]
pub enum SignalError {
    #[error("Signal CLI REST API error: {0}")]
    ApiError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Signal CLI not available")]
    NotAvailable,

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Attachment error: {0}")]
    Attachment(String),

    #[error("Timeout waiting for response")]
    Timeout,
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, SignalError>;
