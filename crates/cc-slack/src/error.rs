//! エラー型定義 (cc-slack)

use thiserror::Error;

/// cc-slack のエラー型
#[derive(Error, Debug)]
pub enum SlackError {
    #[error("Slack API error: {0}")]
    ApiError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Slack token not configured")]
    TokenNotConfigured,

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid signature")]
    InvalidSignature,
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, SlackError>;
