# Contacts Guide (CardDAV)

Contacts integration via CardDAV protocol for reading and managing address books.

## Overview

| Component | Protocol |
|-----------|----------|
| crate | (built-in) |
| Standard | CardDAV (RFC 6352) |

## Configuration

```toml
[contacts]
carddav_url = "https://carddav.example.com"
carddav_username = "${CARDDAV_USERNAME}"
carddav_password = "${CARDDAV_PASSWORD}"
```

### Environment Variables

```bash
CARDDAV_URL=https://carddav.googleapis.com
CARDDAV_USERNAME=your@email.com
CARDDAV_PASSWORD=app-password
```

## Supported Providers

| Provider | URL Example |
|----------|------------|
| Google Contacts | https://carddav.googleapis.com |
| Apple iCloud | https://contacts.icloud.com |
| Nextcloud | https://nextcloud.example.com/remote.php/dav/contacts |
| OwnCloud | https://owncloud.example.com/remote.php/dav/addressbooks |

## Features

- List address books
- Search contacts
- Get contact details
- Create contacts
- Update contacts
- Delete contacts

## Usage

### List Address Books

```json
{
    "type": "contacts",
    "action": "list_addressbooks"
}
```

### Search Contacts

```json
{
    "type": "contacts",
    "action": "search",
    "query": "John"
}
```

### Get Contact

```json
{
    "type": "contacts",
    "action": "get",
    "uid": "contact-uid"
}
```

### Create Contact

```json
{
    "type": "contacts",
    "action": "create",
    "name": "John Doe",
    "email": "john@example.com",
    "phone": "+1234567890",
    "organization": "Example Corp"
}
```

## Contact Fields

| Field | vCard Property |
|-------|----------------|
| Name | FN, N |
| Email | EMAIL |
| Phone | TEL |
| Organization | ORG |
| Title | TITLE |
| Address | ADR |
| URL | URL |
| Note | NOTE |
| Photo | PHOTO |

## Authentication

### App Password (Google)

1. Enable 2-Step Verification
2. Go to Google Account → Security
3. App passwords → Generate new app password
4. Use the generated password

## Example

```toml
[contacts]
carddav_url = "https://carddav.googleapis.com"
carddav_username = "user@gmail.com"
# Use App Password, not regular password
carddav_password = "${GOOGLE_APP_PASSWORD}"
```

## Data Format

Contacts are stored in vCard format:

```
BEGIN:VCARD
VERSION:3.0
FN:John Doe
EMAIL:john@example.com
TEL:+1234567890
ORG:Example Corp
END:VCARD
```
