//! Error types for cc-calendar

use thiserror::Error;

/// cc-calendar error type
#[derive(Error, Debug)]
pub enum CalendarError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("CalDAV error: {0}")]
    CaldavError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("XML parsing error: {0}")]
    XmlParseError(String),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Create error: {0}")]
    CreateError(String),

    #[error("Delete error: {0}")]
    DeleteError(String),

    #[error("Update error: {0}")]
    UpdateError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid calendar ID: {0}")]
    InvalidCalendarId(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, CalendarError>;
