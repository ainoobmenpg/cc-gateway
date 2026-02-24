//! エラー型定義 (cc-line)

use thiserror::Error;

/// cc-line のエラー型
#[derive(Error, Debug)]
pub enum LineError {
    #[error("LINE API error: {0}")]
    ApiError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Channel secret not configured")]
    ChannelSecretNotConfigured,

    #[error("Channel access token not configured")]
    AccessTokenNotConfigured,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Webhook error: {0}")]
    Webhook(String),
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, LineError>;
