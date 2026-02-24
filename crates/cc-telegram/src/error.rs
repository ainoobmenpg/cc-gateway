//! Error types for cc-telegram

use thiserror::Error;

/// cc-telegram error type
#[derive(Error, Debug)]
pub enum TelegramError {
    #[error("Telegram token not set")]
    TokenNotSet,

    #[error("Telegram API error: {0}")]
    Api(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Teloxide error: {0}")]
    Teloxide(#[from] teloxide::ApiError),

    #[error("Request error: {0}")]
    Request(String),

    #[error("Download error: {0}")]
    Download(String),
}

impl From<teloxide::RequestError> for TelegramError {
    fn from(err: teloxide::RequestError) -> Self {
        match err {
            teloxide::RequestError::Api(api_err) => TelegramError::Teloxide(api_err),
            _ => TelegramError::Request(err.to_string()),
        }
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, TelegramError>;
