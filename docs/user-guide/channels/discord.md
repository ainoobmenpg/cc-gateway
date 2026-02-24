# Discord チャネルガイド

cc-gateway を Discord Bot として使用することで、Discord サーバーから AI アシスタントと対話できます。

## Discord Bot の作成手順

### 1. Discord Developer Portal にアクセス

https://discord.com/developers/applications にアクセスし、ログインします。

### 2. 新しいアプリケーションを作成

1. 右上の「New Application」をクリック
2. アプリケーション名を入力（例: `cc-gateway`）
3. 「Create」をクリック

### 3. Bot を作成

1. 左側のメニューから「Bot」を選択
2. 「Add Bot」をクリック
3. 「Yes, do it!」をクリック

### 4. トークンを取得

1. 「Reset Token」をクリックして新しいトークンを生成
2. トークンをコピーして安全な場所に保存
3. ⚠️ **注意**: トークンは他人に教えないでください

### 5. 権限を設定

1. 左側のメニューから「OAuth2」→「URL Generator」を選択
2. 以下のスコープにチェック：
   - `bot`
3. 以下の Bot Permissions にチェック：
   - `Send Messages`
   - `Read Messages/View Channels`
   - `Read Message History`
   - `Use Slash Commands`
   - `Message Content Intent`（特権扱い）
4. 生成された URL を使って Bot をサーバーに招待

### 6. Message Content Intent を有効化

1. 「Bot」セクションに戻る
2. **「MESSAGE CONTENT INTENT」** をオンにする
3. 「Save Changes」をクリック

---

## 設定ファイル

`cc-gateway.toml` に Discord 設定を追加します：

```toml
[discord]
# Discord Bot トークン（必須）
token = "${DISCORD_BOT_TOKEN}"

# 管理者ユーザー ID リスト（コマンド実行権限を持つユーザー）
admin_user_ids = ["123456789012345678", "987654321098765432"]
```

### 環境変数での設定

`.env` ファイルを作成：

```bash
DISCORD_BOT_TOKEN=your-bot-token-here
ADMIN_USER_IDS=123456789012345678,987654321098765432
```

### ユーザー ID の取得方法

Discord でユーザー ID を取得するには：

1. Discord を開く
2. 設定 → 詳細設定 → 「開発者モード」をオン
3. 対象ユーザーを右クリック
4. 「ID をコピー」を選択

---

## スラッシュコマンド一覧

cc-gateway の Discord Bot は以下のスラッシュコマンドを提供します：

| コマンド | 説明 | 権限 |
|---------|------|------|
| `/ask` | AI に質問する | すべてのユーザー |
| `/clear` | 会話履歴をクリア | すべてのユーザー |
| `/help` | ヘルプを表示 | すべてのユーザー |

### `/ask` - AI に質問

AI アシスタントに質問やタスクを依頼します。

```
/ask 今日の天気は？
/ask src/main.rs の内容を分析して
/ask ディレクトリ内の .rs ファイルを一覧して
```

### `/clear` - 履歴クリア

現在のチャンネルでの会話履歴をクリアします。

```
/clear
```

### `/help` - ヘルプ表示

利用可能なコマンドの一覧を表示します。

```
/help
```

---

## 使用例

### 基本的な対話

```
ユーザー: /ask こんにちは！
Bot: こんにちは！お手伝いできることがありましたら、お気軽にお聞きください。
```

### ツールを使用したタスク

```
ユーザー: /ask このリポジトリのファイル構造を教えて
Bot: ファイル構造を確認します。

[ツール実行: bash]
Command: find . -type f -name "*.rs" | head -20

[結果]
./src/main.rs
./src/lib.rs
./tests/integration_test.rs
...
```

### コード分析

```
ユーザー: /ask src/main.rs のバグを探して
Bot: ソースコードを読み込みます。

[ツール実行: read]
File: src/main.rs

[分析結果]
潜在的な問題点：
1. エラーハンドリングが不足しています
2. 未使用の変数 `x` があります
...
```

### Web 検索

```
ユーザー: /ask Rust の最新バージョンは？
Bot: 最新情報を検索します。

[ツール実行: web_search]
Query: Rust latest version 2025

[結果]
Rust 1.85 が最新リリースです（2025年2月）。
...
```

---

## セッション管理

Discord Bot は各ユーザー/チャンネルごとに独立したセッションを保持します：

- **DM**: ユーザーごとのプライベートセッション
- **サーバーチャンネル**: チャンネルごとの共有セッション
- **セッション保持時間**: デフォルトで1時間（最後の対話から）

### セッションのクリア

```
/clear
```

---

## 管理者機能

`admin_user_ids` に設定されたユーザーは、追加の権限を持ちます：

- すべてのツールへの完全アクセス
- システムレベルのコマンド実行
- 設定の変更（将来実装予定）

---

## 起動方法

```bash
# cc-gateway を起動（Discord Bot が含まれます）
cargo run

# または Discord のみを起動（将来実装予定）
cargo run -- --discord
```

### 起動時のログ

```
INFO Starting Discord bot with poise framework...
INFO "cc-gateway#1234" is connected!
INFO Registered 3 slash commands globally.
```

---

## トラブルシューティング

### Bot が応答しない

1. Bot がオンラインか確認（Discord 上で Bot のステータスを確認）
2. `cc-gateway.toml` のトークンが正しいか確認
3. Bot に「Message Content Intent」が付与されているか確認

### スラッシュコマンドが表示されない

1. コマンドの登録には最大1時間かかる場合があります
2. 即座に反映させるには、Bot を再起動してください
3. Bot に「Use Slash Commands」権限があるか確認

### 権限エラー

1. チャンネルで Bot に「SendMessage」「Read Messages」権限があるか確認
2. 「Message Content Intent」が有効になっているか確認

### セッションが保持されない

1. データベースファイル `data/cc-gateway.db` に書き込み権限があるか確認
2. ディスク容量に余裕があるか確認

---

## ヒント

- **DM とサーバー**: プライベートな質問は DM、共有したいタスクはサーバーチャンネル
- **メンション**: Bot はメンションなしでも応答します
- **複数のサーバー**: 1つの Bot で複数の Discord サーバーに参加できます
- **セキュリティ**: トークンは必ず環境変数または `.env` ファイルで管理してください
