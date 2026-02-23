# cc-gateway

> Pure Rust Claude API Gateway - OpenClaw alternative with GLM support

A high-performance gateway for Claude API and OpenAI-compatible APIs (GLM Coding Plan), written in Rust.

## Features

- **Multiple LLM Providers**: Supports both Anthropic Claude API and OpenAI-compatible APIs (GLM, etc.)
- **CLI Interactive Mode**: OpenClaw-like REPL for direct interaction
- **HTTP API**: RESTful API server with authentication
- **Discord Bot**: Full-featured Discord integration with slash commands
- **MCP Integration**: Model Context Protocol support for external tools
- **Built-in Tools**: bash, read, write, edit, glob, grep

## Installation

```bash
# Clone
git clone https://github.com/ainoobmenpg/cc-gateway.git
cd cc-gateway

# Build
cargo build --release

# Run
./target/release/cc-gateway --help
```

## Usage

### CLI Mode (OpenClaw-like)

```bash
# Start interactive REPL
cargo run -- --cli
```

```
╔════════════════════════════════════════════════════════════╗
║          🤖 cc-gateway CLI - Interactive Mode              ║
╠════════════════════════════════════════════════════════════╣
║  Type your message and press Enter to chat.                ║
║  Commands: /help, /exit, /clear, /history                  ║
╚════════════════════════════════════════════════════════════╝

> こんにちは
こんにちは！お手伝いできることがありましたら、お気軽にお聞きください。

> /help
📖 Available Commands:
  /help, /?     - Show this help message
  /exit, /quit  - Exit the program
  /clear        - Clear conversation history
  /history      - Show conversation history
```

### Server Mode

```bash
# Start HTTP API + Discord Bot
cargo run
```

### HTTP API

```bash
# Health check
curl http://localhost:3000/health

# Chat
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!"}'
```

## Configuration

Create a `.env` file:

```bash
# LLM Configuration (Required)
LLM_API_KEY=your-api-key
LLM_MODEL=glm-4.7
LLM_PROVIDER=openai  # claude or openai
LLM_BASE_URL=https://api.z.ai/api/coding/paas/v4

# Discord Bot (Optional)
DISCORD_BOT_TOKEN=your-bot-token
ADMIN_USER_IDS=123456789,987654321

# HTTP API (Optional)
API_KEY=your-api-key
API_PORT=3000

# MCP Integration (Optional)
MCP_ENABLED=true
MCP_CONFIG_PATH=mcp.json
```

## Architecture

```
cc-gateway (workspace)
├── crates/
│   ├── cc-core/        # Core library (Tool trait, LLM client, Session, Memory)
│   ├── cc-tools/       # Built-in tools (Bash, Read, Write, Edit, Glob, Grep)
│   ├── cc-mcp/         # MCP client integration (rmcp)
│   ├── cc-discord/     # Discord Gateway (Serenity)
│   ├── cc-api/         # HTTP API (axum)
│   └── cc-gateway/     # Main binary
```

## Supported Providers

| Provider | Type | Base URL |
|----------|------|----------|
| Anthropic Claude | `claude` | `https://api.anthropic.com/v1` |
| GLM Coding Plan | `openai` | `https://api.z.ai/api/coding/paas/v4` |
| OpenAI | `openai` | `https://api.openai.com/v1` |

## MCP Integration

Create `mcp.json`:

```json
{
  "servers": [
    {
      "name": "git",
      "command": "uvx mcp-server-git",
      "enabled": true
    }
  ]
}
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt
```

## Tech Stack

- **Language**: Rust 2024 Edition (rustc 1.85+)
- **Async Runtime**: tokio
- **HTTP Client**: reqwest (rustls-tls)
- **HTTP Server**: axum
- **Discord**: serenity
- **MCP**: rmcp
- **Database**: rusqlite (bundled)

## License

MIT

## Acknowledgments

- [OpenClaw](https://openclaw.ai) - Inspiration for this project
- [Anthropic](https://anthropic.com) - Claude API
- [Z.ai](https://z.ai) - GLM Coding Plan
