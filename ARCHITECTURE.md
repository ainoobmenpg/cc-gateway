# cc-gateway Architecture

> Pure Rust Claude API Gateway - OpenClaw 100% Compatible Implementation
>
> Updated: 2026-02-25

---

## Overview

cc-gateway is a pure Rust implementation of a Claude API Gateway that provides 18+ communication channels, 15+ tools, and 9-layer security.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           cc-gateway                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Discord  │ │ Telegram │ │ WhatsApp │ │  Signal  │ │  Slack   │          │
│  │  (poise)│ │teloxide  │ │ Twilio   │ │signal-cli│ │ slack-rs │          │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘          │
│       │            │            │            │            │                 │
│  ┌────┴────────────┴────────────┴────────────┴────────────┴────┐           │
│  │                    cc-core (Agent Loop)                     │           │
│  │  ┌──────────────────────────────────────────────────────┐  │           │
│  │  │              LLM Client (Claude/OpenAI/GLM)           │  │           │
│  │  └──────────────────────────────────────────────────────┘  │           │
│  │                           │                                 │           │
│  │  ┌──────────────────────────────────────────────────────┐  │           │
│  │  │                  Tool System                          │  │           │
│  │  │  Bash | Read | Write | Edit | Glob | Grep | ls ...   │  │           │
│  │  └──────────────────────────────────────────────────────┘  │           │
│  │                           │                                 │           │
│  │  ┌───────────┐  ┌──────────┴───────────┐  ┌─────────────┐  │           │
│  │  │  Session  │  │      Memory         │  │   Skills    │  │           │
│  │  │ Manager   │  │   (SQLite Store)   │  │  (inject)   │  │           │
│  │  └───────────┘  └─────────────────────┘  └─────────────┘  │           │
│  └──────────────────────────────────────────────────────────┘           │
│                           │                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │  iMessage │ │   LINE   │ │  Email   │ │  Twitter │ │Instagram │       │
│  │AppleScript│ │    API   │ │  SMTP    │ │ API v2   │ │   API    │       │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘       │
│       │            │            │            │            │              │
│  ┌────┴────────────┴────────────┴────────────┴────────────┴────┐         │
│  │                    Platform Layer                          │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │         │
│  │  │    CLI   │  │ HTTP API │  │WebSocket │  │  Dashboard │  │         │
│  │  │   (REPL) │  │  (axum)  │  │(tungstenite)│ │  (static)  │  │         │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────┘  │         │
│  │                                                              │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │         │
│  │  │  Voice   │  │ Calendar │  │ Contacts │  │  Browser   │  │         │
│  │  │(TTS/WSP) │  │ CalDAV   │  │ CardDAV  │  │headless-chr│  │         │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────┘  │         │
│  └────────────────────────────────────────────────────────────┘         │
│                           │                                               │
│  ┌────────────────────────┴────────────────────────────────────────┐    │
│  │                     Support Systems                              │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐       │    │
│  │  │ Scheduler│  │    MCP   │  │ Security │  │  Approval  │       │    │
│  │  │  (cron)  │  │ (rmcp)   │  │ (9-layer)│  │  System   │       │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────┘       │    │
│  └──────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Project Structure

```
cc-gateway/
├── Cargo.toml                           # Workspace root
├── cc-gateway.toml.example              # Configuration template
├── .env.example                         # Environment variables template
│
├── crates/
│   ├── cc-core/                         # Core library
│   │   ├── src/
│   │   │   ├── lib.rs                  # Main exports
│   │   │   ├── config.rs               # Configuration
│   │   │   ├── llm/                    # LLM clients
│   │   │   │   ├── client.rs            # HTTP client
│   │   │   │   ├── types.rs             # Request/Response types
│   │   │   │   └── mod.rs
│   │   │   ├── tool/                   # Tool system
│   │   │   │   ├── definition.rs        # Tool trait
│   │   │   │   ├── manager.rs          # Tool manager
│   │   │   │   ├── policy.rs           # 9-layer policy
│   │   │   │   ├── approval.rs         # Approval system
│   │   │   │   └── mod.rs
│   │   │   ├── session/                # Session management
│   │   │   │   ├── manager.rs          # Session manager
│   │   │   │   ├── store.rs            # SQLite store
│   │   │   │   ├── types.rs            # Session types
│   │   │   │   └── mod.rs
│   │   │   ├── memory/                 # Memory system
│   │   │   │   ├── store.rs            # Memory store
│   │   │   │   ├── compaction.rs       # Memory compaction
│   │   │   │   └── mod.rs
│   │   │   ├── skills/                 # Skills system
│   │   │   │   ├── loader.rs           # Skill loader
│   │   │   │   ├── inject_prompts.rs   # Prompt injection
│   │   │   │   └── mod.rs
│   │   │   ├── agents/                 # Agent system
│   │   │   │   ├── manager.rs          # Agent manager
│   │   │   │   ├── delegation.rs       # Task delegation
│   │   │   │   ├── types.rs            # Agent types
│   │   │   │   └── mod.rs
│   │   │   ├── audit/                  # Audit system
│   │   │   │   ├── logger.rs           # Audit logger
│   │   │   │   ├── crypto.rs           # Cryptography
│   │   │   │   └── mod.rs
│   │   │   └── mod.rs
│   │   └── Cargo.toml
│   │
│   ├── cc-tools/                       # Built-in tools
│   │   ├── src/
│   │   │   ├── lib.rs                  # Tool registration
│   │   │   ├── bash.rs                 # Command execution
│   │   │   ├── read.rs                 # File read
│   │   │   ├── write.rs                # File write
│   │   │   ├── edit.rs                 # File edit
│   │   │   ├── glob.rs                 # Pattern search
│   │   │   ├── grep.rs                 # Content search
│   │   │   ├── ls.rs                   # Directory listing
│   │   │   ├── apply_patch.rs          # Patch application
│   │   │   ├── web_search.rs           # Web search
│   │   │   ├── web_fetch.rs            # Web fetch
│   │   │   ├── nodes.rs                # Node operations
│   │   │   └── canvas.rs               # Canvas operations
│   │   └── Cargo.toml
│   │
│   ├── cc-api/                        # HTTP API server
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── server.rs               # Axum server
│   │   │   ├── routes.rs               # API routes
│   │   │   ├── handlers.rs             # Request handlers
│   │   │   └── middleware/
│   │   │       ├── auth.rs             # Authentication
│   │   │       ├── rate_limit.rs      # Rate limiting
│   │   │       └── tailscale.rs        # Tailscale auth
│   │   └── Cargo.toml
│   │
│   ├── cc-discord/                    # Discord gateway
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── bot.rs                  # Bot setup
│   │   │   ├── handler.rs              # Event handler
│   │   │   ├── commands/
│   │   │   │   ├── help.rs
│   │   │   │   └── clear.rs
│   │   │   └── session.rs             # Discord session
│   │   └── Cargo.toml
│   │
│   ├── cc-telegram/                   # Telegram gateway
│   ├── cc-whatsapp/                   # WhatsApp gateway (Twilio)
│   ├── cc-signal/                     # Signal gateway
│   ├── cc-slack/                      # Slack gateway
│   ├── cc-line/                       # LINE gateway
│   ├── cc-imessage/                   # iMessage gateway
│   ├── cc-email/                      # Email gateway (SMTP/POP3)
│   ├── cc-twitter/                    # Twitter/X gateway
│   ├── cc-instagram/                  # Instagram gateway
│   ├── cc-facebook/                   # Facebook gateway
│   ├── cc-voice/                      # Voice gateway (TTS/Whisper)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tts.rs                 # Text-to-Speech
│   │   │   ├── whisper.rs             # Speech-to-Text
│   │   │   └── phone.rs               # Phone calls (Twilio)
│   │   └── Cargo.toml
│   │
│   ├── cc-browser/                    # Browser automation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── browser.rs             # Headless Chrome
│   │   └── Cargo.toml
│   │
│   ├── cc-ws/                        # WebSocket gateway
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── server.rs              # WebSocket server
│   │   │   ├── handler.rs             # Message handler
│   │   │   ├── session.rs             # Session management
│   │   │   ├── canvas.rs              # Canvas operations
│   │   │   └── message.rs             # Message types
│   │   └── Cargo.toml
│   │
│   ├── cc-dashboard/                  # Web dashboard
│   ├── cc-mcp/                        # MCP client
│   ├── cc-schedule/                   # Task scheduler
│   │
│   └── cc-gateway/                    # Main binary
│       ├── src/
│       │   ├── lib.rs
│       │   ├── main.rs                # Entry point
│       │   └── cli.rs                 # CLI options
│       └── Cargo.toml
│
└── docs/                              # Documentation
    ├── user-guide/
    │   ├── channels/                   # Channel guides
    │   ├── tools.md                   # Tool reference
    │   ├── skills.md                  # Skills guide
    │   └── ...
    ├── developer-guide/
    └── reference/
```

---

## Data Flow

### 1. Message Reception Flow

```
Channel (Discord/Telegram/etc.)
    ↓
Event Handler (crate-specific)
    ↓
Session Manager (lookup/create session)
    ↓
Core Agent Loop
```

### 2. Agent Loop

```
User Message
    ↓
Build Request (with session context)
    ↓
LLM API (Claude/OpenAI/GLM)
    ↓
┌──────────────────────────────────────────┐
│  Check stop_reason:                      │
│  - "end_turn" → Return response         │
│  - "tool_use" → Execute tool → Loop     │
└──────────────────────────────────────────┘
    ↓
Tool Execution (with policy check)
    ↓
Tool Result → LLM API (continue loop)
```

### 3. Security Flow

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

---

## Key Components

### Tool System (9-Layer Policy)

| Level | Tools | Approval Required |
|:-----:|-------|:-----------------:|
| 1 | Read, Glob, Grep, ls | No |
| 2 | WebFetch, WebSearch | No |
| 3 | Edit, apply_patch | No |
| 4 | Write | DM |
| 5 | Bash (read-only) | DM |
| 6 | Browser | Yes |
| 7 | Bash (full) | Yes |
| 8 | External API | Yes |
| 9 | Security config | Yes |

### Session Management

- SQLite-backed persistent sessions
- Per-channel session isolation
- Conversation history management
- Cost tracking per session

### Memory System

- Vector-based semantic search (planned)
- SQLite-backed storage
- Memory compaction
- Per-session memory isolation

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2024 Edition |
| Async Runtime | tokio |
| HTTP Client | reqwest (rustls-tls) |
| HTTP Server | axum |
| WebSocket | tokio-tungstenite |
| Database | rusqlite (bundled) |
| Discord | poise |
| Telegram | teloxide |
| MCP | rmcp |
| Browser | headless-chrome |

---

## Configuration Priority

1. Environment variables (.env)
2. TOML configuration file
3. Default values

---

## Environment Variables

```bash
# Required
CLAUDE_API_KEY=sk-ant-...
LLM_PROVIDER=claude  # or openai
LLM_MODEL=claude-sonnet-4-20250514

# Optional - Channels
DISCORD_BOT_TOKEN=...
TELEGRAM_BOT_TOKEN=...
TWILIO_ACCOUNT_SID=...
SLACK_BOT_TOKEN=...
LINE_CHANNEL_ACCESS_TOKEN=...
# ... etc

# Optional - Platform
API_PORT=3000
API_KEY=...

# Optional - Security
TAILSCALE_AUTH_ENABLED=true
APPROVAL_REQUIRED=true
```
