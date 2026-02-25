//! Data models for calendar integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Calendar configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarConfig {
    /// CalDAV server URL
    pub server_url: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Calendar ID (optional, defaults to primary calendar)
    #[serde(default)]
    pub calendar_id: Option<String>,
}

impl CalendarConfig {
    /// Create a new calendar config
    pub fn new(server_url: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            username: username.into(),
            password: password.into(),
            calendar_id: None,
        }
    }

    /// Set the calendar ID
    pub fn with_calendar_id(mut self, calendar_id: impl Into<String>) -> Self {
        self.calendar_id = Some(calendar_id.into());
        self
    }
}

/// Calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Event unique identifier (ETag)
    #[serde(default)]
    pub uid: Option<String>,
    /// Event summary/title
    pub summary: String,
    /// Event description
    #[serde(default)]
    pub description: Option<String>,
    /// Event start time
    pub start: DateTime<Utc>,
    /// Event end time
    pub end: DateTime<Utc>,
    /// Event location
    #[serde(default)]
    pub location: Option<String>,
    /// Event organizer
    #[serde(default)]
    pub organizer: Option<String>,
    /// Event attendees
    #[serde(default)]
    pub attendees: Vec<String>,
    /// All-day event flag
    #[serde(default)]
    pub all_day: bool,
    /// Recurrence rule (iCal format)
    #[serde(default)]
    pub rrule: Option<String>,
    /// Last modification time
    #[serde(default)]
    pub modified: Option<DateTime<Utc>>,
}

impl Default for CalendarEvent {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            uid: None,
            summary: String::new(),
            description: None,
            start: now,
            end: now + chrono::Duration::hours(1),
            location: None,
            organizer: None,
            attendees: Vec::new(),
            all_day: false,
            rrule: None,
            modified: None,
        }
    }
}

impl CalendarEvent {
    /// Create a new calendar event
    pub fn new(summary: impl Into<String>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            summary: summary.into(),
            start,
            end,
            ..Default::default()
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}
