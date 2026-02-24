//! Audit logging error types

use thiserror::Error;

/// Audit-related errors
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Failed to write audit log: {0}")]
    WriteError(#[from] std::io::Error),

    #[error("Failed to serialize audit entry: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to rotate audit log: {0}")]
    RotationError(String),

    #[error("Invalid audit configuration: {0}")]
    ConfigurationError(String),

    #[error("Audit storage error: {0}")]
    StorageError(String),
}

pub type AuditResult<T> = Result<T, AuditError>;
