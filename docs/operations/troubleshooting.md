# トラブルシューティング

このガイドでは、cc-gateway の使用中に発生する可能性のある一般的な問題とその解決方法について説明します。

## 目次

- [ビルドエラー](#ビルドエラー)
- [起動エラー](#起動エラー)
- [接続エラー](#接続エラー)
- [ツール実行エラー](#ツール実行エラー)
- [Discord 接続エラー](#discord-接続エラー)
- [MCP 接続エラー](#mcp-接続エラー)
- [ログの確認方法](#ログの確認方法)
- [デバッグモードの有効化](#デバッグモードの有効化)

---

## ビルドエラー

### Rust バージョンが古い

**エラー例:**
```
error[E0658]: use of unstable library feature 'future_join'
```

**原因:** cc-gateway は Rust 2024 Edition (rustc 1.85+) を必要とします。

**解決方法:**

```bash
# Rust バージョンを確認
rustc --version

# Rustup を使用して最新版に更新
rustup update stable

# 特定のバージョンをインストール
rustup install stable
rustup default stable
```

### 依存関係のビルドエラー

**エラー例:**
```
error: failed to compile cc-gateway
```

**解決方法:**

```bash
# キャッシュをクリアして再ビルド
cargo clean
cargo build --release

# 依存関係を更新
cargo update
```

### OpenSSL 関連のエラー (Linux)

**エラー例:**
```
error: linking with `cc` failed: /usr/bin/ld: cannot find -lssl
```

**解決方法:**

```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install openssl-devel pkg-config

# Arch Linux
sudo pacman -S openssl pkg-config
```

---

## 起動エラー

### API キーが設定されていない

**エラー例:**
```
Error: LLM API key is not configured
```

**解決方法:**

1. 環境変数で設定:
```bash
export LLM_API_KEY=your-api-key-here
```

2. または `.env` ファイルで設定:
```bash
echo "LLM_API_KEY=your-api-key-here" > .env
```

3. または `cc-gateway.toml` で設定:
```toml
[llm]
api_key = "your-api-key-here"
```

### 設定ファイルが見つからない

**エラー例:**
```
Error: Configuration file not found
```

**解決方法:**

```bash
# サンプル設定をコピー
cp cc-gateway.toml.example cc-gateway.toml

# 必要な項目を編集
vim cc-gateway.toml
```

### ポートが既に使用中

**エラー例:**
```
Error: Address already in use (os error 48)
```

**解決方法:**

```bash
# 使用中のポートを確認
lsof -i :3000

# プロセスを終了
kill -9 <PID>

# または別のポートを使用
export API_PORT=3001
```

---

## 接続エラー

### ネットワークエラー

**エラー例:**
```
Error: Failed to connect to API endpoint
```

**解決方法:**

1. インターネット接続を確認:
```bash
ping -c 3 api.anthropic.com
# または
ping -c 3 api.openai.com
```

2. プロキシ設定を確認:
```bash
export HTTP_PROXY=http://your-proxy:port
export HTTPS_PROXY=http://your-proxy:port
```

3. ファイアウォール設定を確認

### API タイムアウト

**エラー例:**
```
Error: Request timed out after 30 seconds
```

**解決方法:**

1. ネットワーク接続の速度を確認
2. API エンドポイントの URL が正しいか確認
3. リクエストを簡略化する

### SSL/TLS 証明書エラー

**エラー例:**
```
Error: invalid peer certificate details
```

**解決方法:**

```bash
# 証明書ストアを更新 (macOS)
brew install ca-certificates

# 証明書ストアを更新 (Linux)
sudo update-ca-certificates
```

---

## ツール実行エラー

### 権限エラー

**エラー例:**
```
Error: Permission denied (os error 13)
```

**解決方法:**

```bash
# ファイルのパーミッションを確認
ls -la /path/to/file

# 実行権限を追加
chmod +x script.sh

# ディレクトリへの書き込み権限を確認
ls -ld /path/to/directory
```

### パスが見つからない

**エラー例:**
```
Error: No such file or directory (os error 2)
```

**解決方法:**

1. 絶対パスを使用
2. カレントディレクトリを確認:
```bash
pwd
ls -la
```
3. 環境変数 PATH を確認:
```bash
echo $PATH
```

### ツールが見つからない

**エラー例:**
```
Error: Unknown tool: custom_tool
```

**解決方法:**

1. 組み込みツールのリストを確認:
```bash
# CLI モードで
> /help
```

2. カスタム MCP サーバーが有効になっているか確認:
```bash
cat mcp.json
```

---

## Discord 接続エラー

### Bot トークンが無効

**エラー例:**
```
Error: Failed to authenticate with Discord
```

**解決方法:**

1. Discord Developer Portal でトークンを再生成
2. 環境変数を更新:
```bash
export DISCORD_BOT_TOKEN=your-new-token
```
3. Bot の権限を確認

### Bot がギルドに参加していない

**解決方法:**

1. Discord Developer Portal で OAuth2 URL を生成
2. 必要なスコープを追加: `bot`, `applications.commands`
3. URL にアクセスして Bot を招待

### スラッシュコマンドが表示されない

**解決方法:**

1. Bot に `applications.commands` スコープが付与されているか確認
2. ギルドコマンドは登録に数分かかる場合があります
3. Discord クライアントを再起動

### 権限エラー

**エラー例:**
```
Error: Missing required permissions
```

**解決方法:**

1. `ADMIN_USER_IDS` に自分のユーザー ID を追加
2. Bot のロール権限を確認
3. チャンネルの権限設定を確認

### ユーザー ID の確認方法

```bash
# Discord で開発者モードを有効にする
# 設定 > 詳細設定 > 開発者モード

# ユーザーを右クリック > ID をコピー
```

---

## MCP 接続エラー

### MCP サーバーが起動しない

**エラー例:**
```
Error: Failed to start MCP server: uvx
```

**解決方法:**

1. uvx がインストールされているか確認:
```bash
uvx --version
```

2. Python がインストールされているか確認:
```bash
python --version
```

3. uv をインストール:
```bash
pip install uv
```

### MCP 設定ファイルのエラー

**エラー例:**
```
Error: Failed to parse mcp.json
```

**解決方法:**

1. JSON の構文を確認:
```bash
cat mcp.json | jq .
```

2. サンプル設定を確認:
```bash
cat mcp.json.example
```

### MCP サーバーが応答しない

**解決方法:**

1. サーバーが有効になっているか確認:
```bash
cat mcp.json | jq '.servers[].enabled'
```

2. MCP ログを確認（デバッグモード）

---

## ログの確認方法

### 標準ログ

```bash
# ログを表示しながら実行
cargo run

# ログをファイルに保存
cargo run > logs/cc-gateway.log 2>&1
```

### RUST_LOG で詳細度を変更

```bash
# エラーのみ
RUST_LOG=error cargo run

# 情報レベル（デフォルト）
RUST_LOG=info cargo run

# デバッグレベル
RUST_LOG=debug cargo run

# トレースレベル（最も詳細）
RUST_LOG=trace cargo run

# 特定のモジュールのみ
RUST_LOG=cc_core=debug,cc_discord=info cargo run
```

### SQLite データベースのログ

```bash
# データベースファイルの場所を確認
echo $DB_PATH  # デフォルト: data/cc-gateway.db

# SQLite CLI で開く
sqlite3 data/cc-gateway.db

# ログテーブルを確認
sqlite> .tables
sqlite> SELECT * FROM conversations ORDER BY created_at DESC LIMIT 10;
```

---

## デバッグモードの有効化

### 環境変数で有効化

```bash
# デバッグログを有効化
export RUST_LOG=debug
export RUST_BACKTRACE=1  # バックトレースを有効化

cargo run
```

### cargo の詳細出力

```bash
# ビルド時の詳細出力
cargo build --verbose

# テスト時の詳細出力
cargo test -- --nocapture

# 特定のテストのみ実行
cargo test test_name -- --exact --nocapture
```

### トレースログの取得

```bash
# フルバックトレース
export RUST_BACKTRACE=full

cargo run
```

### ログファイルの出力先

```bash
# ログディレクトリを作成
mkdir -p logs

# ログをローテートしながら保存
cargo run 2>&1 | tee -a logs/cc-gateway-$(date +%Y%m%d).log
```

---

## その他の問題

### メモリ使用量が高い

**解決方法:**

1. セッション履歴をクリア:
```bash
# データベースから古い履歴を削除
sqlite3 data/cc-gateway.db "DELETE FROM messages WHERE created_at < datetime('now', '-30 days');"
```

2. 会話履歴をリセット（CLI モード）:
```
> /clear
```

### データベースが破損している

**解決方法:**

```bash
# データベースの整合性をチェック
sqlite3 data/cc-gateway.db "PRAGMA integrity_check;"

# 破損している場合はバックアップから復元
cp data/cc-gateway.db.backup data/cc-gateway.db
```

---

## 問題が解決しない場合

1. 最新バージョンに更新:
```bash
git pull origin main
cargo build --release
```

2. GitHub Issues で同様の問題を検索

3. デバッグログを添えて Issue を作成:
```bash
RUST_LOG=debug cargo run 2>&1 | tee debug.log
```
