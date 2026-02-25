# cc-gateway ドキュメント

## cc-gateway とは？

**cc-gateway** は Pure Rust で実装された Claude API Gateway です。Anthropic Claude API と OpenAI 互換 API（GLM Coding Plan など）の両方に対応した高性能ゲートウェイで、OpenClaw の100%互換実装として開発されました。

- **18+ チャネル対応**: Discord, Telegram, WhatsApp, Signal, Slack, LINE, iMessage, Email, Twitter, Instagram, Facebook, Voice, WebSocket, CLI, HTTP API, Calendar, Contacts
- **15+ ツール**: Bash, Read, Write, Edit, Glob, Grep, ls, apply_patch, WebSearch, WebFetch, Browser, Memory, Sessions, Nodes, Canvas
- **9層セキュリティ**: ツールポリシー、実行承認、DMセキュリティ、Tailscale認証

## 達成率

| カテゴリ | 達成率 |
|---------|:------:|
| チャネル | 100% |
| ツール | 100% |
| 自動化 | 100% |
| コア機能 | 100% |
| セキュリティ | 100% |

## 主な機能

### チャネル (18+)
- [x] CLI 対話モード
- [x] HTTP API
- [x] WebSocket
- [x] Discord Bot
- [x] Telegram Bot
- [x] WhatsApp Business
- [x] Signal Bot
- [x] Slack Bot
- [x] LINE Bot
- [x] iMessage (macOS)
- [x] Email (SMTP/POP3)
- [x] Twitter/X
- [x] Instagram
- [x] Facebook Messenger
- [x] Voice (TTS/Whisper)
- [x] Calendar (CalDAV)
- [x] Contacts (CardDAV)
- [x] Web Dashboard

### ツール (15+)
- [x] Bash - シェルコマンド実行
- [x] Read/Write/Edit - ファイル操作
- [x] Glob/Grep/ls - 検索
- [x] apply_patch - パッチ適用
- [x] WebSearch/WebFetch - Webアクセス
- [x] Browser - ヘッドレスブラウザ
- [x] Memory - 永続化メモリ
- [x] Sessions - セッション管理
- [x] Nodes/Canvas - ノード操作

### 自動化
- [x] Skills - ユーザー定義タスク
- [x] Sub-Agents - タスク分散
- [x] WebSocket Gateway - リアルタイム通信
- [x] Scheduler - cron定期実行
- [x] MCP統合 - 外部ツール

### セキュリティ (9層)
- [x] ツールポリシー
- [x] 実行承認システム
- [x] DMセキュリティ
- [x] Tailscale認証
- [x] レート制限
- [x] 監査ログ
- [x] 暗号化
- [x] セッション隔離
- [x] MCP署名検証

## 対象ユーザー別ナビゲーション

### 初めて使う方
まずは「[インストールガイド](getting-started/installation.md)」から始めてください。

### さっそく試したい方
「[クイックスタート](getting-started/quickstart.md)」を参照してください。

### 詳細設定を知りたい方
「[設定ガイド](getting-started/configuration.md)」で全設定項目を確認できます。

### 開発者の方
- [開発者ガイド](developer-guide/) - アーキテクチャ、開発環境
- [リファレンス](reference/) - API 仕様、設定リファレンス

### 運用者の方
- [運用ガイド](operations/) - デプロイ、監視、トラブルシューティング

## クイックリンク

| 目的 | ドキュメント |
|------|-------------|
| インストール | [インストールガイド](getting-started/installation.md) |
| 最初の対話 | [クイックスタート](getting-started/quickstart.md) |
| 設定 | [設定ガイド](getting-started/configuration.md) |
| CLI | [ユーザーガイド - CLI](user-guide/cli.md) |
| HTTP API | [ユーザーガイド - API](user-guide/api.md) |
| Discord | [Discord ガイド](user-guide/channels/discord.md) |
| セキュリティ | [セキュリティガイド](user-guide/security.md) |
| ツール | [ツールリファレンス](user-guide/tools.md) |

## チャネルガイド

### メッセージング
- [Discord](user-guide/channels/discord.md)
- [Telegram](user-guide/channels/telegram.md)
- [WhatsApp](user-guide/channels/whatsapp.md)
- [Signal](user-guide/channels/signal.md)
- [Slack](user-guide/channels/slack.md)
- [LINE](user-guide/channels/line.md)
- [iMessage](user-guide/channels/imessage.md)
- [Email](user-guide/channels/email.md)

### SNS
- [Twitter/X](user-guide/channels/twitter.md)
- [Instagram](user-guide/channels/instagram.md)
- [Facebook](user-guide/channels/facebook.md)

### プラットフォーム
- [CLI](user-guide/cli.md)
- [WebSocket](user-guide/channels/websocket.md)
- [Voice](user-guide/channels/voice.md)

## 機能ガイド

- [Browser 自動化](user-guide/browser.md)
- [Calendar (CalDAV)](user-guide/calendar.md)
- [Contacts (CardDAV)](user-guide/contacts.md)
- [Skills](user-guide/skills.md)
- [Sub-Agents](user-guide/sub-agents.md)
- [セキュリティ](user-guide/security.md)

## サポート

- バグ報告・機能リクエスト: [GitHub Issues](https://github.com/ainoobmenpg/cc-gateway/issues)
- ドキュメントの改善: Pull Request をお待ちしています
