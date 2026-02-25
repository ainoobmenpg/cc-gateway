# Slack チャネルガイド

Slack ワークスペースを通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | Slack API |
| crate | cc-slack |
| ステータス | ✅ 実装済み |

## 設定

```toml
[slack]
token = "${SLACK_BOT_TOKEN}"
app_token = "${SLACK_APP_TOKEN}"
signing_secret = "${SLACK_SIGNING_SECRET}"
```

### 環境変数

```bash
SLACK_BOT_TOKEN=xoxb-...
SLACK_APP_TOKEN=xapp-...
SLACK_SIGNING_SECRET=...
```

## Slack App の作成

1. https://api.slack.com/apps で新規アプリを作成
2. Bot Token Scopes を設定:
   - `chat:write`
   - `channels:read`
   - `groups:read`
   - `im:read`
   - `mpim:read`
3. アプリをワークスペースにインストール

## 機能

- @メンションで対話
- スラッシュコマンド対応
- チャンネル/DM対応
- インタラクティブメッセージ
- Socket Mode 対応

## スラッシュコマンド

| コマンド | 説明 |
|---------|------|
| `/ask [メッセージ]` | AI に質問 |
| `/help` | ヘルプ表示 |
| `/clear` | 会話をクリア |

## レート制限

Slack API のレート制限に注意してください。
