# よくある質問（FAQ）

cc-gateway に関するよくある質問と回答です。

---

## 一般的な質問

### Q: GLM と Claude、どちらを使うべき？

**A:** 使用シナリアによって異なります。

| 特徴 | GLM Coding Plan | Claude |
|------|-----------------|--------|
| **提供元** | Z.ai | Anthropic |
| **対応モデル** | glm-4.7 等 | Claude 4 Sonnet/Opus/Haiku |
| **プロバイダー設定** | `LLM_PROVIDER=openai` | `LLM_PROVIDER=claude` |
| **ベース URL** | `https://api.z.ai/api/coding/paas/v4` | `https://api.anthropic.com/v1` |
| **特徴** | コーディングタスクに特化 | 一般的なタスクに対応 |

**推奨設定:**

```bash
# GLM Coding Plan（コーディング向け）
export LLM_PROVIDER=openai
export LLM_MODEL=glm-4.7
export LLM_BASE_URL=https://api.z.ai/api/coding/paas/v4

# Anthropic Claude（汎用）
export LLM_PROVIDER=claude
export LLM_MODEL=claude-sonnet-4-20250514
```

---

### Q: Rust がインストールできない

**A:** 以下の方法を試してください。

**macOS / Linux:**

```bash
# rustup インストーラーを使用
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# シェルを再読み込み
source $HOME/.cargo/env

# バージョン確認（1.85+が必要）
rustc --version
```

**Windows:**

1. [rustup.rs](https://rustup.rs/) からインストーラーをダウンロード
2. インストーラーを実行
3. コマンドプロンプトを再起動

**トラブルシューティング:**

```bash
# プロキシ環境下の場合
export https_proxy=http://your-proxy:port
export http_proxy=http://your-proxy:port

# 既存の Rust を削除して再インストール
rustup self uninstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

### Q: API キーはどこで取得できる？

**A:** 使用するプロバイダーによって異なります。

**GLM Coding Plan (Z.ai):**

1. [Z.ai](https://z.ai) にアクセス
2. アカウントを作成またはログイン
3. API キーを発行

**Anthropic Claude:**

1. [Anthropic Console](https://console.anthropic.com) にアクセス
2. アカウントを作成またはログイン
3. 「API Keys」セクションでキーを作成

**OpenAI:**

1. [OpenAI Platform](https://platform.openai.com) にアクセス
2. アカウントを作成またはログイン
3. 「API Keys」セクションでキーを作成

---

### Q: Discord Bot が反応しない

**A:** 以下を確認してください。

1. **トークンが正しいか確認:**

```bash
echo $DISCORD_BOT_TOKEN
```

2. **Bot がサーバーに招待されているか確認:**

Discord Developer Portal で OAuth2 URL を生成し、Bot を招待してください。
必要なスコープ: `bot`, `applications.commands`

3. **管理者ユーザー ID が設定されているか確認:**

```bash
echo $ADMIN_USER_IDS
```

4. **ログを確認:**

```bash
RUST_LOG=debug cargo run
```

5. **Bot の意図を確認:**

Discord Developer Portal で「Bot」セクションの「Presence Intent」および「Server Members Intent」が有効になっていることを確認してください。

---

### Q: ツールが実行されない

**A:** 以下の点を確認してください。

1. **ツール名が正しいか確認:**

組み込みツール: `bash`, `read`, `write`, `edit`, `glob`, `grep`

2. **入力パラメータが正しいか確認:**

```
# 正しい例
> bash で現在のディレクトリのファイルを一覧表示して

# 間違った例
> ファイル一覧表示
```

3. **権限があるか確認:**

```bash
# ファイルの読み取り権限
ls -la /path/to/file

# ディレクトリの書き込み権限
ls -ld /path/to/directory
```

4. **MCP サーバーの設定を確認（カスタムツールの場合）:**

```bash
cat mcp.json | jq .
```

---

### Q: メモリはどこに保存される？

**A:** SQLite データベースに保存されます。

**デフォルトの場所:**

```bash
data/cc-gateway.db
```

**場所を変更する場合:**

```bash
export DB_PATH=/custom/path/database.db
```

**データベースの中身を確認:**

```bash
sqlite3 data/cc-gateway.db

# テーブル一覧
.tables

# 会話履歴を確認
SELECT * FROM conversations ORDER BY created_at DESC LIMIT 10;

# メッセージを確認
SELECT * FROM messages WHERE conversation_id = 1;

# メモリを確認
SELECT * FROM memories;
```

---

### Q: 複数のチャネルを同時に使える？

**A:** はい、複数のチャンネルで同時に使用できます。

**Discord の場合:**

```toml
# cc-gateway.toml
[discord]
token = "${DISCORD_BOT_TOKEN}"
admin_user_ids = [123456789, 987654321]
```

同じ Bot を複数のサーバー（ギルド）に招待することで、各サーバーで独立して動作します。

**HTTP API の場合:**

複数のクライアントから同時にリクエストを送信できます。

```bash
# クライアント1
curl -X POST http://localhost:3000/api/chat -H "Content-Type: application/json" -d '{"message": "Hello"}'

# クライアント2
curl -X POST http://localhost:3000/api/chat -H "Content-Type: application/json" -d '{"message": "Hi"}'
```

**CLI モードの場合:**

CLI モードは単一セッションです。複数の同時対話を行う場合は、サーバーモードを使用してください。

---

### Q: カスタムツールを作成したい

**A:** Rust で独自のツールを実装できます。

**基本的な手順:**

1. `cc-core` の `Tool` trait を実装する構造体を作成:

```rust
use cc_core::{Tool, ToolResult};
use async_trait::async_trait;

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "My custom tool description"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "param": {
                    "type": "string",
                    "description": "Parameter description"
                }
            },
            "required": ["param"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult> {
        let param = input["param"].as_str().unwrap_or("");

        // ツールの処理を実装
        let result = format!("Processed: {}", param);

        Ok(ToolResult::success(result))
    }
}
```

2. ツールを登録:

```rust
use cc_core::ToolManager;

let mut tool_manager = ToolManager::new();
tool_manager.register_tool(MyTool);
```

詳細は [開発者ガイド](../developer-guide/index.md) を参照してください。

---

### Q: スケジューラーはどのように設定する？

**A:** `schedule.toml` ファイルで設定します。

**基本設定:**

```toml
[[schedules]]
name = "毎朝の挨拶"
cron = "0 9 * * *"
prompt = "おはようございます。今日の予定を教えてください。"
enabled = true

[[schedules]]
name = "日次レポート"
cron = "0 18 * * *"
prompt = "今日の作業ログをまとめてください。"
tools = ["read", "glob"]
enabled = true
```

**cron 形式:** `分 時 日 月 曜日`

| フィールド | 値の範囲 |
|-----------|---------|
| 分 | 0-59 |
| 時 | 0-23 |
| 日 | 1-31 |
| 月 | 1-12 |
| 曜日 | 0-6（0 = 日曜日） |

**例:**

```bash
# 毎時0分
cron = "0 * * * *"

# 毎日9:00
cron = "0 9 * * *"

# 毎週月曜日10:00
cron = "0 10 * * 1"

# 毎月1日0:00
cron = "0 0 1 * *"
```

---

### Q: Docker で動かしたい

**A:** Docker イメージが利用可能です。

**使用方法:**

```bash
# ビルド
docker build -t cc-gateway .

# 実行
docker run -d \
  -e LLM_API_KEY=your-api-key \
  -e DISCORD_BOT_TOKEN=your-bot-token \
  -v $(pwd)/data:/app/data \
  -p 3000:3000 \
  cc-gateway
```

**docker-compose の場合:**

```bash
docker-compose up -d
```

---

### Q: パフォーマンスを向上させたい

**A:** 以下の方法を試してください。

1. **リリースビルドを使用:**

```bash
cargo build --release
./target/release/cc-gateway
```

2. **メモリ使用量を最適化:**

古い会話履歴を定期的に削除してください。

```bash
sqlite3 data/cc-gateway.db "DELETE FROM messages WHERE created_at < datetime('now', '-30 days');"
```

3. **ログレベルを調整:**

```bash
export RUST_LOG=warn  # ログを減らす
```

---

### Q: セキュリティ上の注意点は？

**A:** 以下の点に注意してください。

1. **API キーを公開しない:**

```bash
# .gitignore に追加
.env
cc-gateway.toml
data/
```

2. **API 認証を有効にする:**

```bash
export API_KEY=your-secure-api-key
```

3. **管理者権限を制限する:**

```bash
export ADMIN_USER_IDS=123456789  # 必要なユーザーのみ
```

4. **ファイアウォールを設定する:**

```bash
# ローカルホストのみでリッスン
export API_HOST=127.0.0.1
```

---

## まだ問題が解決しない場合

1. [トラブルシューティングガイド](../operations/troubleshooting.md) を確認してください
2. [GitHub Issues](https://github.com/ainoobmenpg/cc-gateway/issues) で同様の問題を検索してください
3. デバッグログを添えて新しい Issue を作成してください:

```bash
RUST_LOG=debug cargo run 2>&1 | tee debug.log
```
