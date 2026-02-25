# Facebook チャネルガイド

Facebook ページを通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | Facebook Messenger API |
| crate | cc-facebook |
| ステータス | ✅ 実装済み |

## 設定

```toml
[facebook]
page_id = "${FACEBOOK_PAGE_ID}"
access_token = "${FACEBOOK_ACCESS_TOKEN}"
app_secret = "${FACEBOOK_APP_SECRET}"
verify_token = "${FACEBOOK_VERIFY_TOKEN}"
```

### 環境変数

```bash
FACEBOOK_PAGE_ID=123456789
FACEBOOK_ACCESS_TOKEN=...
FACEBOOK_APP_SECRET=...
FACEBOOK_VERIFY_TOKEN=your_verify_token
```

## Facebook App の作成

1. https://developers.facebook.com でアプリを作成
2. Messenger 製品を追加
3. ページをリンク
4. アクセストークンを取得
5. Webhook を設定

## 機能

- ダイレクトメッセージの送受信
- クイックリプライ
- テンプレートメッセージ
- ペイロード付きボタン
- シェア extension

## 対応メッセージタイプ

| タイプ | 説明 |
|-------|------|
| text | テキストメッセージ |
| image | 画像 |
| audio | 音声 |
| video | 動画 |
| file | ファイル |
| template | テンプレート |

## クイックリプライ

```json
{
  "text": "What would you like to do?",
  "quick_replies": [
    {"content_type": "text", "title": "Help", "payload": "HELP"},
    {"content_type": "text", "title": "Info", "payload": "INFO"}
  ]
}
```

## 制約

- Facebook の審査が必要
- レート制限あり
- ページ管理者が必要
