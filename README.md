# cc-gateway

> Pure Rust Claude API Gateway - OpenClaw 代替実装（GLM 対応）

Claude API と OpenAI 互換 API（GLM Coding Plan など）に対応した高性能ゲートウェイ。Rust で実装されています。

## 機能

- **マルチ LLM プロバイダー**: Anthropic Claude API と OpenAI 互換 API（GLM 等）の両方に対応
- **CLI 対話モード**: OpenClaw 風の REPL で直接対話
- **HTTP API**: 認証付き RESTful API サーバー
- **Discord Bot**: スラッシュコマンド対応のフル機能 Discord 連携
- **MCP 統合**: Model Context Protocol による外部ツール対応
- **組み込みツール**: bash, read, write, edit, glob, grep
- **スケジューラー**: cron 形式でタスクを定期実行

## インストール

```bash
# クローン
git clone https://github.com/ainoobmenpg/cc-gateway.git
cd cc-gateway

# ビルド
cargo build --release

# 実行
./target/release/cc-gateway --help
```

## 使用方法

### CLI モード（OpenClaw 風）

```bash
# 対話型 REPL を起動
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

### サーバーモード

```bash
# HTTP API + Discord Bot を起動
cargo run
```

### HTTP API

```bash
# ヘルスチェック
curl http://localhost:3000/health

# チャット
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!"}'
```

## 設定

設定は以下の優先順位で読み込まれます:
1. 環境変数
2. `cc-gateway.toml` 設定ファイル
3. デフォルト値

### TOML 設定ファイル（推奨）

`cc-gateway.toml.example` をコピーして使用してください：

```bash
cp cc-gateway.toml.example cc-gateway.toml
```

設定ファイル内の `${VAR_NAME}` は環境変数の値に置換されます。

```toml
[llm]
provider = "openai"
model = "glm-4.7"
base_url = "https://api.z.ai/api/coding/paas/v4"
api_key = "${LLM_API_KEY}"

[discord]
token = "${DISCORD_BOT_TOKEN}"
admin_user_ids = [123456789]

[api]
port = 3000

[scheduler]
enabled = true
config_path = "schedule.toml"

[mcp]
enabled = true
config_path = "mcp.json"
```

### 環境変数設定（.env ファイル）

`.env` ファイルを作成（TOML 設定ファイル内の値を上書きします）：

```bash
# LLM 設定（必須）
LLM_API_KEY=your-api-key
LLM_MODEL=glm-4.7
LLM_PROVIDER=openai  # claude または openai
LLM_BASE_URL=https://api.z.ai/api/coding/paas/v4

# Discord Bot（オプション）
DISCORD_BOT_TOKEN=your-bot-token
ADMIN_USER_IDS=123456789,987654321

# HTTP API（オプション）
API_KEY=your-api-key
API_PORT=3000

# MCP 統合（オプション）
MCP_ENABLED=true
MCP_CONFIG_PATH=mcp.json
```

## アーキテクチャ

```
cc-gateway (workspace)
├── crates/
│   ├── cc-core/        # コアライブラリ (Tool trait, LLM client, Session, Memory)
│   ├── cc-tools/       # 組み込みツール (Bash, Read, Write, Edit, Glob, Grep)
│   ├── cc-mcp/         # MCP クライアント統合 (rmcp)
│   ├── cc-discord/     # Discord Gateway (Serenity)
│   ├── cc-api/         # HTTP API (axum)
│   └── cc-gateway/     # メインバイナリ
```

## 対応プロバイダー

| プロバイダー | タイプ | ベース URL |
|------------|--------|-----------|
| Anthropic Claude | `claude` | `https://api.anthropic.com/v1` |
| GLM Coding Plan | `openai` | `https://api.z.ai/api/coding/paas/v4` |
| OpenAI | `openai` | `https://api.openai.com/v1` |

## MCP 統合

`mcp.json` を作成：

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

## スケジューラー

`schedule.toml` で定期実行タスクを設定：

```toml
# 毎朝の挨拶
[[schedules]]
name = "毎朝の挨拶"
cron = "0 9 * * *"        # 毎日 9:00
prompt = "おはようございます。今日の予定を教えてください。"
enabled = true

# 日次レポート
[[schedules]]
name = "日次レポート"
cron = "0 18 * * *"       # 毎日 18:00
prompt = "今日の作業ログをまとめてください。"
tools = ["read", "glob"]  # 使用ツールを制限（オプション）
discord_channel = "reports"  # Discord に投稿（オプション）
enabled = true
```

cron 形式: `分 時 日 月 曜日`

| 環境変数 | 説明 | デフォルト |
|---------|------|----------|
| `SCHEDULE_ENABLED` | スケジューラー有効/無効 | `true` |
| `SCHEDULE_CONFIG_PATH` | 設定ファイルパス | `schedule.toml` |

## 開発

```bash
# ビルド
cargo build

# テスト
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# フォーマット
cargo fmt
```

## 技術スタック

- **言語**: Rust 2024 Edition (rustc 1.85+)
- **非同期ランタイム**: tokio
- **HTTP クライアント**: reqwest (rustls-tls)
- **HTTP サーバー**: axum
- **Discord**: serenity
- **MCP**: rmcp
- **データベース**: rusqlite (bundled)

## ライセンス

MIT

## 謝辞

- [OpenClaw](https://openclaw.ai) - 本プロジェクトのインスピレーション
- [Anthropic](https://anthropic.com) - Claude API
- [Z.ai](https://z.ai) - GLM Coding Plan
