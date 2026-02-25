//! Error types for cc-browser

use thiserror::Error;

/// cc-browser error type
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Browser initialization failed: {0}")]
    Initialization(String),

    #[error("Navigation failed: {0}")]
    Navigation(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Interaction failed: {0}")]
    Interaction(String),

    #[error("Screenshot failed: {0}")]
    Screenshot(String),

    #[error("Extraction failed: {0}")]
    Extraction(String),

    #[error("Tab error: {0}")]
    TabError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Cookie error: {0}")]
    Cookie(String),

    #[error("Download error: {0}")]
    Download(String),

    #[error("Frame error: {0}")]
    Frame(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, BrowserError>;
