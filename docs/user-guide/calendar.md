# Calendar Guide (CalDAV)

Calendar integration via CalDAV protocol for reading and managing events.

## Overview

| Component | Protocol |
|-----------|----------|
| crate | (built-in) |
| Standard | CalDAV (RFC 4791) |

## Configuration

```toml
[calendar]
caldav_url = "https://caldav.example.com"
caldav_username = "${CALDAV_USERNAME}"
caldav_password = "${CALDAV_PASSWORD}"
```

### Environment Variables

```bash
CALDAV_URL=https://caldav.googleapis.com
CALDAV_USERNAME=your@email.com
CALDAV_PASSWORD=app-password
```

## Supported Providers

| Provider | URL Example |
|----------|------------|
| Google Calendar | https://caldav.googleapis.com |
| Apple iCloud | https://caldav.icloud.com |
| Nextcloud | https://nextcloud.example.com/remote.php/dav |
| OwnCloud | https://owncloud.example.com/remote.php/dav |

## Features

- List calendars
- Fetch events
- Create events
- Update events
- Delete events
- Recurring events

## Usage

### List Calendars

```json
{
    "type": "calendar",
    "action": "list_calendars"
}
```

### Get Events

```json
{
    "type": "calendar",
    "action": "get_events",
    "start": "2026-02-25T00:00:00Z",
    "end": "2026-02-26T00:00:00Z"
}
```

### Create Event

```json
{
    "type": "calendar",
    "action": "create_event",
    "summary": "Meeting",
    "start": "2026-02-25T10:00:00Z",
    "end": "2026-02-25T11:00:00Z",
    "description": "Team meeting",
    "location": "Conference Room"
}
```

## Time Zones

```toml
[calendar]
default_timezone = "Asia/Tokyo"
```

## Authentication

### App Password (Google)

1. Enable 2-Step Verification
2. Go to Google Account → Security
3. App passwords → Generate new app password
4. Use the generated password

### OAuth2 (Future)

OAuth2 authentication is planned for future releases.

## Example

```toml
[calendar]
caldav_url = "https://caldav.googleapis.com"
caldav_username = "user@gmail.com"
# Use App Password, not regular password
caldav_password = "${GOOGLE_APP_PASSWORD}"
```
