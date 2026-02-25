# cc-gateway

> Pure Rust Claude API Gateway - OpenClaw 100%互換実装

Claude API に対応した高性能ゲートウェイ。Rust で実装され、18+チャネル、15+ツール、9層セキュリティを備えています。

## 機能

### チャネル (18+)

| カテゴリ | チャネル | ステータス |
|---------|---------|----------|
| **メッセージング** | Discord, Telegram, WhatsApp, Signal, Slack, LINE, iMessage | ✅ 実装済み |
| **SNS** | Instagram, Facebook, Twitter/X | ✅ 実装済み |
| **メール** | Email (SMTP/POP3) | ✅ 実装済み |
| **プラットフォーム** | CLI, HTTP API, WebSocket, Web Dashboard | ✅ 実装済み |
| **音声** | Voice (TTS/Whisper) | ✅ 実装済み |
| **データ** | Calendar (CalDAV), Contacts (CardDAV) | ✅ 実装済み |

### ツール (15+)

| ツール | 説明 |
|-------|------|
| **Bash** | シェルコマンド実行 |
| **Read** | ファイル読み取り |
| **Write** | ファイル作成/上書き |
| **Edit** | ファイル編集 |
| **Glob** | パターン一致ファイル検索 |
| **Grep** | ファイル内検索 |
| **ls** | ディレクトリ一覧 |
| **apply_patch** | パッチ適用 |
| **WebSearch** | Web検索 |
| **WebFetch** | Webコンテンツ取得 |
| **Browser** | ヘッドレスブラウザ操作 |
| **Memory** | 永続化メモリ |
| **Sessions** | セッション管理 |
| **Nodes** | ノード操作 |
| **Canvas** | キャンバス操作 |

### 自動化

- **Skills**: ユーザー定義自動化タスク
- **Sub-Agents**: タスク分散エージェント
- **WebSocket Gateway**: リアルタイム双方向通信
- **Scheduler**: cron形式定期実行

### セキュリティ (9層)

1. ツールポリシー (9層レベル)
2. 実行承認システム
3. DMセキュリティ
4. Tailscale認証
5. レート制限
6. 監査ログ
7. 暗号化
8. セッション隔離
9. MCP署名検証

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

### CLI モード

```bash
# 対話型 REPL を起動
cargo run -- --cli
```

```
cc-gateway CLI - Interactive Mode
Type your message and press Enter to chat.
Commands: /help, /exit, /clear, /history

> こんにちは
こんにちは！お手伝いできることがありましたら、お気軽にお聞きください。
```

### サーバーモード

```bash
# 全チャネル起動
cargo run
```

### HTTP API

```bash
# ヘルスチェック
curl http://localhost:3000/health

# チャット
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{"message": "Hello!"}'
```

## 設定

設定は以下の優先順位で読み込まれます:
1. 環境変数 (.env)
2. `cc-gateway.toml` 設定ファイル
3. デフォルト値

### TOML 設定ファイル

```bash
cp cc-gateway.toml.example cc-gateway.toml
```

```toml
[llm]
provider = "claude"
model = "claude-sonnet-4-20250514"
api_key = "${CLAUDE_API_KEY}"

[discord]
token = "${DISCORD_BOT_TOKEN}"
admin_user_ids = [123456789]

[telegram]
token = "${TELEGRAM_BOT_TOKEN}"

[whatsapp]
account_sid = "${TWILIO_ACCOUNT_SID}"
auth_token = "${TWILIO_AUTH_TOKEN}"

[signal]
phone_number = "${SIGNAL_PHONE_NUMBER}"
api_token = "${SIGNAL_API_TOKEN}"

[slack]
token = "${SLACK_BOT_TOKEN}"
app_token = "${SLACK_APP_TOKEN}"

[line]
channel_access_token = "${LINE_CHANNEL_ACCESS_TOKEN}"
channel_secret = "${LINE_CHANNEL_SECRET}"

[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_user = "${EMAIL_USER}"
smtp_password = "${EMAIL_PASSWORD}"

[instagram]
username = "${INSTAGRAM_USERNAME}"
password = "${INSTAGRAM_PASSWORD}"

[facebook]
page_id = "${FACEBOOK_PAGE_ID}"
access_token = "${FACEBOOK_ACCESS_TOKEN}"

[twitter]
api_key = "${TWITTER_API_KEY}"
api_secret = "${TWITTER_API_SECRET}"
access_token = "${TWITTER_ACCESS_TOKEN}"
access_token_secret = "${TWITTER_ACCESS_TOKEN_SECRET}"

[voice]
openai_api_key = "${OPENAI_API_KEY}"
twilio_account_sid = "${TWILIO_ACCOUNT_SID}"
twilio_auth_token = "${TWILIO_AUTH_TOKEN}"
twilio_phone_number = "${TWILIO_PHONE_NUMBER}"

[calendar]
caldav_url = "${CALDAV_URL}"
caldav_username = "${CALDAV_USERNAME}"
caldav_password = "${CALDAV_PASSWORD}"

[contacts]
carddav_url = "${CARDDAV_URL}"
carddav_username = "${CARDDAV_USERNAME}"
carddav_password = "${CARDDAV_PASSWORD}"

[api]
port = 3000
api_key = "${API_KEY}"

[security]
approval_required = true
tailscale_auth = true

[scheduler]
enabled = true
config_path = "schedule.toml"

[mcp]
enabled = true
config_path = "mcp.json"
```

## アーキテクチャ

```
cc-gateway (workspace)
├── crates/
│   ├── cc-core/          # コアライブラリ (Tool, LLM, Session, Memory)
│   ├── cc-tools/        # 組み込みツール
│   ├── cc-api/          # HTTP API (axum)
│   ├── cc-discord/      # Discord Gateway (poise)
│   ├── cc-telegram/     # Telegram Bot (teloxide)
│   ├── cc-whatsapp/     # WhatsApp (Twilio)
│   ├── cc-signal/       # Signal Bot
│   ├── cc-slack/        # Slack Bot
│   ├── cc-line/         # LINE Bot
│   ├── cc-imessage/     # iMessage (AppleScript)
│   ├── cc-email/        # Email (SMTP/POP3)
│   ├── cc-twitter/      # Twitter/X
│   ├── cc-instagram/    # Instagram
│   ├── cc-facebook/     # Facebook
│   ├── cc-voice/        # Voice (TTS/Whisper)
│   ├── cc-browser/      # Browser Automation
│   ├── cc-mcp/          # MCP Client
│   ├── cc-schedule/     # Scheduler
│   ├── cc-ws/           # WebSocket Gateway
│   ├── cc-dashboard/    # Web Dashboard
│   └── cc-gateway/      # メインバイナリ
```

## 対応プロバイダー

| プロバイダー | タイプ | ベース URL |
|------------|--------|-----------|
| Anthropic Claude | `claude` | `https://api.anthropic.com/v1` |
| OpenAI | `openai` | `https://api.openai.com/v1` |
| GLM (Z.ai) | `openai` | `https://api.z.ai/api/coding/paas/v4` |

## MCP 統合

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

```toml
[[schedules]]
name = "毎朝の挨拶"
cron = "0 9 * * *"
prompt = "おはようございます。今日の予定を教えてください。"
enabled = true
```

cron 形式: `分 時 日 月 曜日`

## セキュリティ

### 9層ツールポリシー

| レベル | ツール | 承認必要 |
|:------:|-------|:--------:|
| 1 | Read, Glob, Grep, ls | No |
| 2 | WebFetch, WebSearch | No |
| 3 | Edit, apply_patch | No |
| 4 | Write | 自分宛DM |
| 5 | Bash (読み取り専用) | 自分宛DM |
| 6 | Browser | 承認 |
| 7 | Bash (全コマンド) | 承認 |
| 8 | 外部API送信 | 承認 |
| 9 | セキュリティ設定変更 | 承認 |

### 承認フロー

1. ツール実行リクエスト受信
2. ポリシーレベル確認
3. 必要に応じて承認要求
4. ユーザー承認後実行
5. 監査ログに記録

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
- **HTTP クライアント/サーバー**: reqwest / axum
- **データベース**: rusqlite (bundled)
- **Discord**: poise
- **Telegram**: teloxide
- **MCP**: rmcp

## ドキュメント

- [ユーザーガイド](./docs/user-guide/)
- [チャネルガイド](./docs/user-guide/channels/)
- [ツールリファレンス](./docs/user-guide/tools.md)
- [設定ガイド](./docs/getting-started/configuration.md)

## ライセンス

MIT

## 謝辞

- [OpenClaw](https://openclaw.ai) - 本プロジェクトのインスピレーション
- [Anthropic](https://anthropic.com) - Claude API
