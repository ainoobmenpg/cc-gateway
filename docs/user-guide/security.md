# Security Guide

cc-gateway provides comprehensive 9-layer security for tool execution and channel access.

## 9-Layer Security Policy

| Layer | Tools | Approval Required | Description |
|:-----:|-------|:-----------------:|-------------|
| 1 | Read, Glob, Grep, ls | No | Read-only operations |
| 2 | WebFetch, WebSearch | No | Web access |
| 3 | Edit, apply_patch | No | Local modifications |
| 4 | Write | DM | File creation |
| 5 | Bash (read-only) | DM | Safe command execution |
| 6 | Browser | Yes | Browser automation |
| 7 | Bash (full) | Yes | Full shell access |
| 8 | External API | Yes | Network calls |
| 9 | Security config | Yes | Security settings |

## Approval Flow

```
Tool Request
    ↓
Policy Check (9-layer)
    ↓
┌─────────────────────────────────────┐
│  Level 1-3: Auto-approve           │
│  Level 4-5: DM confirmation        │
│  Level 6-9: Explicit approval      │
└─────────────────────────────────────┘
    ↓
Execution (if approved)
    ↓
Audit Log
```

## Configuration

```toml
[security]
approval_required = true
auto_approve_levels = [1, 2, 3]
dm_channels = ["discord", "telegram"]
```

## DM Security

DM (Direct Message) channels provide an additional layer of confirmation:

- Level 4-5 tools require DM confirmation
- Available channels: Discord DM, Telegram, LINE

## Tailscale Authentication

```toml
[security.tailscale]
enabled = true
auth_key = "${TAILSCALE_AUTH_KEY}"
```

Only allow access from Tailscale network users.

## Rate Limiting

```toml
[security.rate_limit]
enabled = true
requests_per_minute = 60
burst = 10
```

## Audit Logging

All tool executions are logged:

```json
{
    "timestamp": "2026-02-25T10:00:00Z",
    "user_id": "user123",
    "channel": "discord",
    "tool": "bash",
    "level": 7,
    "approved": true,
    "approved_by": "user123",
    "input": "ls -la",
    "output": "..."
}
```

## Session Isolation

- Per-channel session isolation
- Per-user context separation
- Session timeout: 30 minutes (configurable)

## MCP Security

```toml
[mcp]
require_signature = true
allowed_servers = ["git", "filesystem"]
```
