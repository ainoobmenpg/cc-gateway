//! CalDAV client implementation

use crate::error::{CalendarError, Result};
use crate::models::{CalendarConfig, CalendarEvent};
use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use tracing::{debug, error, info};

/// CalDAV client for calendar operations
pub struct CalendarClient {
    client: Client,
    config: CalendarConfig,
    base_url: String,
}

impl CalendarClient {
    /// Create a new CalDAV client
    pub async fn new(config: CalendarConfig) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| CalendarError::Configuration(e.to_string()))?;

        let base_url = if config.server_url.ends_with('/') {
            config.server_url.trim_end_matches('/').to_string()
        } else {
            config.server_url.clone()
        };

        info!("Calendar client initialized for: {}", base_url);

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    /// Get calendar events within a date range
    pub async fn get_events(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        let calendar_path = self.calendar_path();
        let url = format!("{}/{}", self.base_url, calendar_path);

        let start_str = start.format("%Y%m%dT%H%M%SZ").to_string();
        let end_str = end.format("%Y%m%dT%H%M%SZ").to_string();

        let body = format!(
            r#"<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:prop>
        <D:getetag/>
        <C:calendar-data/>
    </D:prop>
    <C:filter>
        <C:comp-filter name="VCALENDAR">
            <C:comp-filter name="VEVENT">
                <C:time-range start="{}" end="{}"/>
            </C:comp-filter>
        </C:comp-filter>
    </C:filter>
</C:calendar-query>"#,
            start_str, end_str
        );

        debug!("Fetching events from: {}", url);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), &url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body)
            .send()
            .await
            .map_err(|e| CalendarError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("CalDAV request failed: {} - {}", status, error_text);
            return Err(CalendarError::CaldavError(format!(
                "Request failed: {} - {}",
                status, error_text
            )));
        }

        let text = response.text().await.map_err(|e| CalendarError::HttpError(e.to_string()))?;
        let events = self.parse_calendar_response(&text)?;

        info!("Fetched {} events", events.len());
        Ok(events)
    }

    /// Create a new calendar event
    pub async fn create_event(&self, event: CalendarEvent) -> Result<CalendarEvent> {
        let calendar_path = self.calendar_path();
        let url = format!("{}/{}", self.base_url, calendar_path);

        let uid = event.uid.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let ical = self.event_to_ical(&event, &uid);

        debug!("Creating event: {}", event.summary);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PUT").unwrap(), format!("{}/{}.ics", url, uid))
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "text/calendar; charset=utf-8")
            .header("If-None-Match", "*")
            .body(ical)
            .send()
            .await
            .map_err(|e| CalendarError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Create event failed: {} - {}", status, error_text);
            return Err(CalendarError::CreateError(format!(
                "Failed to create event: {} - {}",
                status, error_text
            )));
        }

        info!("Created event: {}", uid);

        let mut created_event = event;
        created_event.uid = Some(uid);
        Ok(created_event)
    }

    /// Delete a calendar event
    pub async fn delete_event(&self, uid: &str) -> Result<()> {
        let calendar_path = self.calendar_path();
        let url = format!("{}/{}.ics", self.base_url, calendar_path);

        debug!("Deleting event: {}", uid);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"DELETE").unwrap(), format!("{}/{}", url, uid))
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("If-Match", "*")
            .send()
            .await
            .map_err(|e| CalendarError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Delete event failed: {} - {}", status, error_text);
            return Err(CalendarError::DeleteError(format!(
                "Failed to delete event: {} - {}",
                status, error_text
            )));
        }

        info!("Deleted event: {}", uid);
        Ok(())
    }

    /// Get list of available calendars
    pub async fn list_calendars(&self) -> Result<Vec<String>> {
        let url = format!("{}/", self.base_url);

        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
    <D:prop>
        <D:displayname/>
        <C:calendar-home-set/>
    </D:prop>
</D:propfind>"#;

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body)
            .send()
            .await
            .map_err(|e| CalendarError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CalendarError::CaldavError(format!(
                "Failed to list calendars: {}",
                response.status()
            )));
        }

        let text = response.text().await.map_err(|e| CalendarError::HttpError(e.to_string()))?;
        let calendars = self.parse_calendars_response(&text)?;

        Ok(calendars)
    }

    fn calendar_path(&self) -> String {
        self.config
            .calendar_id
            .clone()
            .unwrap_or_else(|| "calendars".to_string())
    }

    fn parse_calendar_response(&self, response: &str) -> Result<Vec<CalendarEvent>> {
        let mut events = Vec::new();
        let mut reader = Reader::from_str(response);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_calendar_data = false;
        let mut current_calendar_data = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"calendar-data" => {
                    in_calendar_data = true;
                    current_calendar_data.clear();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"calendar-data" => {
                    in_calendar_data = false;
                    if let Ok(event) = self.parse_icalendar(&current_calendar_data) {
                        events.push(event);
                    }
                }
                Ok(Event::Text(ref e)) if in_calendar_data => {
                    current_calendar_data.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(CalendarError::XmlParseError(e.to_string()));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(events)
    }

    fn parse_calendars_response(&self, response: &str) -> Result<Vec<String>> {
        let mut calendars = Vec::new();
        let mut reader = Reader::from_str(response);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_href = false;
        let mut current_href = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"href" => {
                    in_href = true;
                    current_href.clear();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"href" => {
                    in_href = false;
                    let path = current_href.trim().to_string();
                    if path.contains("calendars/") || path.ends_with("/") {
                        calendars.push(path);
                    }
                }
                Ok(Event::Text(ref e)) if in_href => {
                    current_href.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(CalendarError::XmlParseError(e.to_string()));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(calendars)
    }

    fn parse_icalendar(&self, ical: &str) -> Result<CalendarEvent> {
        let mut summary = String::new();
        let mut description = None;
        let mut start = Utc::now();
        let mut end = Utc::now();
        let mut location = None;
        let mut uid = None;
        let mut attendees = Vec::new();
        let mut all_day = false;

        for line in ical.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("SUMMARY:") {
                summary = val.to_string();
            } else if let Some(val) = line.strip_prefix("DESCRIPTION:") {
                description = Some(val.to_string());
            } else if line.starts_with("DTSTART") {
                if let Some(dt) = self.parse_ical_date(line) {
                    start = dt;
                }
            } else if line.starts_with("DTEND") {
                if let Some(dt) = self.parse_ical_date(line) {
                    end = dt;
                }
            } else if let Some(val) = line.strip_prefix("LOCATION:") {
                location = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("UID:") {
                uid = Some(val.to_string());
            } else if let Some(email) = line.strip_prefix("ATTENDEE") {
                if let Some(pos) = email.find(':') {
                    attendees.push(email[pos + 1..].to_string());
                }
            } else if line.contains("VALUE=DATE") {
                all_day = true;
            }
        }

        Ok(CalendarEvent {
            uid,
            summary,
            description,
            start,
            end,
            location,
            organizer: None,
            attendees,
            all_day,
            rrule: None,
            modified: None,
        })
    }

    fn parse_ical_date(&self, line: &str) -> Option<DateTime<Utc>> {
        let colon_pos = line.find(':')?;
        let date_str = &line[colon_pos + 1..];

        if date_str.contains('T') {
            // Date-time format: YYYYMMDDTHHMMSSZ
            DateTime::parse_from_str(date_str, "%Y%m%dT%H%M%SZ")
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        } else {
            // Date only format: YYYYMMDD
            chrono::NaiveDate::parse_from_str(date_str, "%Y%m%d")
                .ok()
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
        }
    }

    fn event_to_ical(&self, event: &CalendarEvent, uid: &str) -> String {
        let mut ical = String::new();

        ical.push_str("BEGIN:VCALENDAR\r\n");
        ical.push_str("VERSION:2.0\r\n");
        ical.push_str("PRODID:-//cc-gateway//calendar//EN\r\n");
        ical.push_str("CALSCALE:GREGORIAN\r\n");
        ical.push_str("METHOD:PUBLISH\r\n");
        ical.push_str("BEGIN:VEVENT\r\n");

        ical.push_str(&format!("UID:{}\r\n", uid));
        ical.push_str(&format!("DTSTAMP:{}\r\n", Utc::now().format("%Y%m%dT%H%M%SZ")));

        let start_str = if event.all_day {
            event.start.format("%Y%m%d").to_string()
        } else {
            event.start.format("%Y%m%dT%H%M%SZ").to_string()
        };
        let end_str = if event.all_day {
            event.end.format("%Y%m%d").to_string()
        } else {
            event.end.format("%Y%m%dT%H%M%SZ").to_string()
        };

        ical.push_str(&format!("DTSTART:{}\r\n", start_str));
        ical.push_str(&format!("DTEND:{}\r\n", end_str));

        ical.push_str(&format!("SUMMARY:{}\r\n", event.summary));

        if let Some(ref desc) = event.description {
            ical.push_str(&format!("DESCRIPTION:{}\r\n", desc));
        }

        if let Some(ref loc) = event.location {
            ical.push_str(&format!("LOCATION:{}\r\n", loc));
        }

        for attendee in &event.attendees {
            ical.push_str(&format!("ATTENDEE:mailto:{}\r\n", attendee));
        }

        if let Some(ref rrule) = event.rrule {
            ical.push_str(&format!("RRULE:{}\r\n", rrule));
        }

        ical.push_str("END:VEVENT\r\n");
        ical.push_str("END:VCALENDAR\r\n");

        ical
    }
}
