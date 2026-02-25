# Twitter/X チャネルガイド

Twitter/X を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | Twitter API v2 |
| crate | cc-twitter |
| ステータス | ✅ 実装済み |

## 設定

```toml
[twitter]
api_key = "${TWITTER_API_KEY}"
api_secret = "${TWITTER_API_SECRET}"
access_token = "${TWITTER_ACCESS_TOKEN}"
access_token_secret = "${TWITTER_ACCESS_TOKEN_SECRET}"
bearer_token = "${TWITTER_BEARER_TOKEN}"
webhook_url = "https://your-server/twitter/webhook"
```

### 環境変数

```bash
TWITTER_API_KEY=...
TWITTER_API_SECRET=...
TWITTER_ACCESS_TOKEN=...
TWITTER_ACCESS_TOKEN_SECRET=...
TWITTER_BEARER_TOKEN=...
```

## Twitter Developer アカウントの設定

1. https://developer.twitter.com で開発者アカウントを申請
2. プロジェクトとアプリを作成
3. API Keys と Access Tokens を取得
4. OAuth 1.0a または OAuth 2.0 を設定

## 機能

| 機能 | 説明 |
|------|------|
| メンション応答 | @への返信を自動生成 |
| DM 応答 | ダイレクトメッセージに返信 |
| トレンド取得 | トレンドトピックを取得 |
| 検索 | Twitter 検索結果を分析 |
| ポスト | 自動投稿 |

## 対応イベント

| イベント | 説明 |
|---------|------|
| mention | メンション付きツイート |
| reply | 返信 |
| dm | ダイレクトメッセージ |
| like | いいね |
| retweet | リツイート |

## 使用例

### メンションへの自動応答

```toml
[twitter]
auto_reply = true
reply_mention = true
auto_like = false
```

### 検索結果の分析

```json
{
    "type": "twitter_search",
    "query": "rust programming",
    "max_results": 10
}
```

## レート制限

Twitter API v2 のレート制限に注意：

| エンドポイント | レート |
|--------------|--------|
| 投稿 | 200/15分 |
| 読み取り | 900/15分 |
| DM | 200/15分 |

## 制約

- Twitter API v2 が必要
- 開発者アカウント審査が必要
- レート制限が厳しい
- 最近の変更で DM 機能が制限の場合あり
