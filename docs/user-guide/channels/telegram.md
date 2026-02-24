# Telegram チャネルガイド

> **注意**: Telegram チャネルは現在**計画中**です。このドキュメントは将来の実装に向けた設計案です。

cc-gateway を Telegram Bot として使用することで、Telegram アプリから AI アシスタントと対話できます。

## 機能概要

Telegram Bot チャネルでは以下の機能を提供予定です：

- 💬 **対話型チャット**: メッセージによる対話
- 🔧 **ツール統合**: ファイル操作、コマンド実行、Web検索
- 📎 **ファイル対応**: ドキュメントのアップロード・解析
- 🔄 **セッション管理**: 会話履歴の保持
- ⚙️ **設定管理**: チャット内での動作設定

---

## Telegram Bot の作成手順

### 1. BotFather と対話

1. Telegram で `@BotFather` を検索
2. `/newbot` コマンドを送信
3. Bot の名前を入力（例: `cc-gateway`）
4. Bot のユーザー名を入力（例: `cc_gateway_bot`）

### 2. トークンを取得

BotFather から提供される API トークンを保存します：

```
123456789:ABCdefGHIjklMNOpqrsTUVwxyz
```

### 3. Bot を設定

BotFather で以下の設定が可能です：

- 説明文の設定
- About テキストの設定
- プロフィール写真の設定
- コマンドリストの設定

---

## 設定ファイル

`cc-gateway.toml` に Telegram 設定を追加します（将来実装予定）：

```toml
[telegram]
# Telegram Bot トークン（必須）
token = "${TELEGRAM_BOT_TOKEN}"

# 管理者ユーザー ID リスト
admin_user_ids = [123456789, 987654321]

# Webhook モード（オプション）
use_webhook = true
webhook_url = "https://your-domain.com/telegram/webhook"
webhook_port = 8443
```

### 環境変数での設定

```bash
TELEGRAM_BOT_TOKEN=your-bot-token-here
TELEGRAM_ADMIN_IDS=123456789,987654321
```

### ユーザー ID の取得方法

1. `@userinfobot` にメッセージを送信
2. 返信メッセージにユーザー ID が表示されます

---

## コマンド一覧（予定）

| コマンド | 説明 |
|---------|------|
| `/start` | Bot を開始 |
| `/help` | ヘルプを表示 |
| `/ask` | AI に質問 |
| `/clear` | 会話履歴をクリア |
| `/settings` | 設定を表示・変更 |
| `/image` | 画像生成・解析 |
| `/file` | ファイルをアップロードして処理 |

---

## 使用例（予定）

### 基本的な対話

```
ユーザー: /start
Bot: こんにちは！私は cc-gateway AI アシスタントです。
      何でもお手伝いします。

ユーザー: こんにちは
Bot: こんにちは！お手伝いできることがありましたら、
      お気軽にお聞きください。
```

### ファイル操作

```
ユーザー: /file main.rs
Bot: ファイルを受信しました。内容を分析します...

[分析結果]
この Rust ファイルには：
- 1つの struct 定義
- 3つの関数
- 使用クレート: tokio, serde
```

### コード生成

```
ユーザー: FizzBuzz を Rust で書いて
Bot: わかりました。FizzBuzz の実装を作成します。

```rust
fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}
```
```

---

## Webhook vs Polling

### Polling モード（デフォルト）

```toml
[telegram]
token = "${TELEGRAM_BOT_TOKEN}"
use_webhook = false
```

- サーバーが必要ない
- 遅延が発生する場合がある
- 小規模向け

### Webhook モード（推奨）

```toml
[telegram]
token = "${TELEGRAM_BOT_TOKEN}"
use_webhook = true
webhook_url = "https://your-domain.com/telegram/webhook"
webhook_port = 8443
```

- リアルタイム応答
- HTTPS サーバーが必要
- 大規模向け

---

## グループチャットでの使用

Bot をグループチャットに追加できます：

1. グループに Bot を追加
2. Bot に管理者権限を付与
3. メンションで Bot に話しかけ：`@cc_gateway_bot 天気は？`

---

## セキュリティ

### プライベートチャットのみ

```toml
[telegram]
allow_group_chats = false
allow_private_only = true
```

### ユーザー制限

```toml
[telegram]
allowed_user_ids = [123456789, 987654321]
```

---

## 制限事項

Telegram Bot API の制限事項：

- メッセージサイズ: 最大 4096 文字
- ファイルサイズ: 最大 50 MB
- 写真サイズ: 最大 10 MB
- 動画サイズ: 最大 50 MB
- レート制限: 1秒間に約30メッセージ

---

## 開発ロードマップ

- [ ] 基本的な Bot 機能の実装
- [ ] コマンド処理
- [ ] セッション管理
- [ ] ファイルアップロード対応
- [ ] Webhook サポート
- [ ] インラインキーボード
- [ ] リッチメッセージフォーマット
- [ ] グループチャット対応
