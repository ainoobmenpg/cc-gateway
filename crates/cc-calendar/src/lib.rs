//! cc-calendar: Calendar integration for cc-gateway
//!
//! This crate provides CalDAV calendar access capabilities.
//!
//! ## Features
//!
//! - CalDAV client for calendar access
//! - Event creation, retrieval, and deletion
//! - Support for multiple calendar providers
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cc_calendar::{CalendarClient, CalendarConfig, CalendarEvent};
//!
//! let config = CalendarConfig {
//!     server_url: "https://caldav.example.com".to_string(),
//!     username: "user".to_string(),
//!     password: "password".to_string(),
//!     calendar_id: Some("primary".to_string()),
//! };
//! let client = CalendarClient::new(config).await?;
//!
//! // Get events
//! let events = client.get_events(
//!     chrono::Utc::now() - chrono::Duration::days(7),
//!     chrono::Utc::now() + chrono::Duration::days(30),
//! ).await?;
//!
//! // Create event
//! let event = CalendarEvent {
//!     summary: "Meeting".to_string(),
//!     description: Some("Team meeting".to_string()),
//!     start: chrono::Utc::now() + chrono::Duration::hours(1),
//!     end: chrono::Utc::now() + chrono::Duration::hours(2),
//!     location: None,
//!     ..Default::default()
//! };
//! client.create_event(event).await?;
//! ```

pub mod client;
pub mod error;
pub mod models;

pub use client::CalendarClient;
pub use error::{CalendarError, Result};
pub use models::{CalendarConfig, CalendarEvent};

/// Re-export models for easy use
pub mod prelude {
    pub use super::{CalendarClient, CalendarConfig, CalendarEvent};
}
