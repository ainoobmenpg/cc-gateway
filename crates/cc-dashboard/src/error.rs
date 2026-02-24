//! Error types for cc-dashboard

use thiserror::Error;

/// cc-dashboard error type
#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, DashboardError>;
