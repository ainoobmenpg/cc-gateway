//! Error types for cc-facebook

use thiserror::Error;

/// cc-facebook error type
#[derive(Error, Debug)]
pub enum FacebookError {
    #[error("Facebook page access token not set")]
    AccessTokenNotSet,

    #[error("Facebook app secret not set")]
    AppSecretNotSet,

    #[error("Facebook verify token not set")]
    VerifyTokenNotSet,

    #[error("Facebook API error: {0}")]
    Api(String),

    #[error("Facebook API request failed: {0}")]
    Request(String),

    #[error("Facebook webhook verification failed")]
    WebhookVerificationFailed,

    #[error("Invalid webhook payload: {0}")]
    InvalidPayload(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("JSON serialization error: {0}")]
    Serialization(String),
}

impl From<reqwest::Error> for FacebookError {
    fn from(err: reqwest::Error) -> Self {
        FacebookError::Request(err.to_string())
    }
}

impl From<serde_json::Error> for FacebookError {
    fn from(err: serde_json::Error) -> Self {
        FacebookError::Serialization(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, FacebookError>;
