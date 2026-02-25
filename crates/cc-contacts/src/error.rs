//! Error types for cc-contacts

use thiserror::Error;

/// cc-contacts error type
#[derive(Error, Debug)]
pub enum ContactsError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("CardDAV error: {0}")]
    CarddavError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("XML parsing error: {0}")]
    XmlParseError(String),

    #[error("Contact not found: {0}")]
    ContactNotFound(String),

    #[error("Add error: {0}")]
    AddError(String),

    #[error("Delete error: {0}")]
    DeleteError(String),

    #[error("Update error: {0}")]
    UpdateError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid addressbook ID: {0}")]
    InvalidAddressbookId(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, ContactsError>;
