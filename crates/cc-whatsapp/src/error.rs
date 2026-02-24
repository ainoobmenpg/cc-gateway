//! Error types for cc-whatsapp

use thiserror::Error;

/// cc-whatsapp error type
#[derive(Error, Debug)]
pub enum WhatsAppError {
    #[error("Twilio credentials not set")]
    CredentialsNotSet,

    #[error("Webhook signature verification failed")]
    SignatureVerificationFailed,

    #[error("Invalid webhook payload: {0}")]
    InvalidPayload(String),

    #[error("Twilio API error: {0}")]
    Api(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<reqwest::Error> for WhatsAppError {
    fn from(err: reqwest::Error) -> Self {
        WhatsAppError::Http(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, WhatsAppError>;
