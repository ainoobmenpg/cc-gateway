# Browser Automation Guide

Browser automation enables AI to interact with web pages programmatically.

## Overview

| Component | Technology |
|-----------|------------|
| Browser Engine | headless Chrome |
| crate | cc-browser |

## Configuration

```toml
[browser]
enabled = true
headless = true
window_size = { width = 1920, height = 1080 }
user_agent = "Mozilla/5.0 ..."
```

## Usage

### Navigation

```json
{
    "type": "browser",
    "action": "navigate",
    "url": "https://example.com"
}
```

### Interactions

```json
{
    "type": "browser",
    "action": "click",
    "selector": "#submit-button"
}
```

```json
{
    "type": "browser",
    "action": "type",
    "selector": "#search-input",
    "value": "search query"
}
```

### Screenshot

```json
{
    "type": "browser",
    "action": "screenshot",
    "full_page": true
}
```

## Supported Actions

| Action | Description |
|--------|-------------|
| navigate | Go to URL |
| click | Click element |
| type | Input text |
| select | Select dropdown |
| screenshot | Capture screen |
| evaluate | Execute JavaScript |
| wait | Wait for element |
| scroll | Scroll page |

## Security

- Browser automation requires Level 6 approval
- Runs in isolated environment
- Limited to safe domains (configurable)

## Timeout Settings

```toml
[browser.timeouts]
page_load = 30000
script = 10000
element = 5000
```

## Example Workflow

```
1. Navigate to login page
2. Type username
3. Type password
4. Click submit
5. Wait for dashboard
6. Navigate to target page
7. Extract data
```

## Best Practices

1. Use explicit waits instead of sleep
2. Handle popups carefully
3. Clean up resources properly
4. Monitor memory usage
