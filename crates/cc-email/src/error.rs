//! Error types for cc-email

use thiserror::Error;

/// cc-email error type
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("SMTP configuration error: {0}")]
    SmtpConfig(String),

    #[error("SMTP send error: {0}")]
    SmtpSend(String),

    #[error("IMAP configuration error: {0}")]
    ImapConfig(String),

    #[error("IMAP connection error: {0}")]
    ImapConnection(String),

    #[error("Email parsing error: {0}")]
    Parsing(String),

    #[error("Invalid email address: {0}")]
    InvalidAddress(String),

    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, EmailError>;
