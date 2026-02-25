# チャネルガイド

cc-gateway は複数のチャネル（通信手段）を通じて AI アシスタントと対話できます。

## チャネルとは

チャネルは、あなたと cc-gateway の間の通信手段を定義します。各チャネルは異なるインターフェースと機能を提供します。

## 利用可能なチャネル一覧

| チャネル | ステータス | 説明 |
|---------|----------|------|
| **CLI** | ✅ 実装済み | ターミナルでの対話型 REPL |
| **Discord** | ✅ 実装済み | Discord Bot による対話 |
| **HTTP API** | ✅ 実装済み | RESTful API サーバー |
| **WebSocket** | ✅ 実装済み | リアルタイム双方向通信 |
| **Telegram** | ✅ 実装済み | Telegram Bot による対話 |
| **WhatsApp** | ✅ 実装済み | WhatsApp Business API |
| **Signal** | ✅ 実装済み | Signal Bot |
| **Slack** | ✅ 実装済み | Slack アプリとの連携 |
| **LINE** | ✅ 実装済み | LINE Messaging API |
| **iMessage** | ✅ 実装済み | macOS iMessage (AppleScript) |
| **Email** | ✅ 実装済み | SMTP/POP3 メール |
| **Twitter/X** | ✅ 実装済み | Twitter API v2 |
| **Instagram** | ✅ 実装済み | Instagram Graph API |
| **Facebook** | ✅ 実装済み | Facebook Messenger API |
| **Voice** | ✅ 実装済み | TTS/Whisper/電話 |
| **Calendar** | ✅ 実装済み | CalDAV 連携 |
| **Contacts** | ✅ 実装済み | CardDAV 連携 |
| **Browser** | ✅ 実装済み | ヘッドレスブラウザ |
| **Dashboard** | ✅ 実装済み | Web 管理コンソール |

---

## クイックスタート

### CLI モード

```bash
cargo run -- --cli
```

### Discord Bot

```bash
cargo run
# Discord Bot と HTTP API が起動
```

### WebSocket

```bash
# ws://localhost:3001 に接続
```

---

## チャネル固有ドキュメント

### メッセージング

- [Discord](./discord.md)
- [Telegram](./telegram.md)
- [WhatsApp](./whatsapp.md)
- [Signal](./signal.md)
- [Slack](./slack.md)
- [LINE](./line.md)
- [iMessage](./imessage.md)
- [Email](./email.md)

### SNS

- [Twitter/X](./twitter.md)
- [Instagram](./instagram.md)
- [Facebook](./facebook.md)

### プラットフォーム

- [CLI](../cli.md)
- [WebSocket](./websocket.md)
- [Voice](./voice.md)

---

## 共通設定

### LLM 設定

```toml
[llm]
provider = "claude"
model = "claude-sonnet-4-20250514"
api_key = "${CLAUDE_API_KEY}"
```

### セキュリティ設定

```toml
[security]
approval_required = true
tailscale_auth = false
```

---

## チャネルの選択ガイド

| ユースケース | 推奨チャネル |
|-------------|-------------|
| 開発・デバッグ | CLI |
| チームでの共有 | Discord / Slack |
| アプリケーション連携 | HTTP API / WebSocket |
| モバイルからのアクセス | WhatsApp / LINE / Telegram |
| 音声対話 | Voice |
| SNS 分析 | Twitter / Instagram |
| メール自動化 | Email |

---

## マルチチャネル運用

複数のチャネルを同時に使用できます：

```bash
# 全チャネル起動
cargo run
```

各チャネルは独立したセッションを持ち、異なるコンテキストで対話できます。
