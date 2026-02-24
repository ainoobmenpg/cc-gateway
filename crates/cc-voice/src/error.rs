//! Error types for cc-voice

use thiserror::Error;

/// cc-voice error type
#[derive(Error, Debug)]
pub enum VoiceError {
    #[error("Speech recognition failed: {0}")]
    RecognitionFailed(String),

    #[error("Speech synthesis failed: {0}")]
    SynthesisFailed(String),

    #[error("Audio encoding error: {0}")]
    EncodingError(String),

    #[error("Audio decoding error: {0}")]
    DecodingError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, VoiceError>;
