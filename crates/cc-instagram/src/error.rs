//! Error types for cc-instagram

use thiserror::Error;

/// cc-instagram error type
#[derive(Error, Debug)]
pub enum InstagramError {
    #[error("Instagram access token not set")]
    AccessTokenNotSet,

    #[error("Instagram app secret not set")]
    AppSecretNotSet,

    #[error("Instagram API error: {0}")]
    Api(String),

    #[error("Instagram Graph API error: {0}")]
    GraphApi(String),

    #[error("Webhook verification failed")]
    WebhookVerificationFailed,

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Claude API error: {0}")]
    ClaudeApi(String),
}

impl From<cc_core::Error> for InstagramError {
    fn from(err: cc_core::Error) -> Self {
        InstagramError::ClaudeApi(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, InstagramError>;
