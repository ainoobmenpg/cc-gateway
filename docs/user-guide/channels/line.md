# LINE チャネルガイド

LINE を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | LINE Messaging API |
| crate | cc-line |
| ステータス | ✅ 実装済み |

## 設定

```toml
[line]
channel_access_token = "${LINE_CHANNEL_ACCESS_TOKEN}"
channel_secret = "${LINE_CHANNEL_SECRET}"
```

### 環境変数

```bash
LINE_CHANNEL_ACCESS_TOKEN=...
LINE_CHANNEL_SECRET=...
```

## LINE Developers の設定

1. https://developers.line.biz/ でMessaging APIチャンネルを作成
2. チャネルアクセストークンを取得
3. Webhook URL を設定: `https://your-server/line/webhook`

## 機能

- テキストメッセージの送受信
- 画像/音声対応
- グループトーク対応
- テンプレートメッセージ
- クイックリプライ
- Flex Message

## 対応イベント

| イベントタイプ | 説明 |
|--------------|------|
| message | テキスト、画像、音声、ビデオ |
| follow | 友達追加 |
| unfollow | ブロック |
| join | グループ参加 |
| leave | グループ退出 |

## Rich Menu

LINE の Rich Menu 機能を使って、クイックアクションを設定できます。
