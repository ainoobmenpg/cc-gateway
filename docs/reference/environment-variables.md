# 環境変数リファレンス

cc-gateway で使用できる環境変数の一覧です。

## 設定の優先順位

設定値は以下の順序で評価されます。上位の設定が優先されます。

1. **環境変数** - 最も優先度が高い
2. `cc-gateway.toml` 設定ファイル
3. **デフォルト値** - 最も優先度が低い

### 設定例

```bash
# 環境変数で設定（最優先）
export LLM_MODEL=claude-sonnet-4-20250514
```

```toml
# cc-gateway.toml で設定（環境変数がない場合に使用）
[llm]
model = "claude-sonnet-4-20250514"
```

---

## LLM 設定

### LLM_API_KEY

- **説明**: LLM プロバイダーの API 認証キー
- **デフォルト値**: なし
- **必須**: ○

```bash
export LLM_API_KEY=sk-ant-your-key-here
```

### LLM_MODEL

- **説明**: 使用する LLM モデル名
- **デフォルト値**: `claude-sonnet-4-20250514`
- **必須**: -

```bash
# GLM モデル
export LLM_MODEL=glm-4.7

# Claude モデル
export LLM_MODEL=claude-sonnet-4-20250514
export LLM_MODEL=claude-opus-4-20250514

# OpenAI モデル
export LLM_MODEL=gpt-4o
export LLM_MODEL=gpt-4o-mini
```

### LLM_PROVIDER

- **説明**: LLM プロバイダーの種類
- **デフォルト値**: `claude`
- **必須**: -
- **選択可能な値**:
  - `openai` - OpenAI 互換 API（GLM、OpenAI など）
  - `claude` - Anthropic Claude API

```bash
# OpenAI 互換 API（GLM 等）
export LLM_PROVIDER=openai

# Anthropic Claude API
export LLM_PROVIDER=claude
```

### LLM_BASE_URL

- **説明**: カスタム API エンドポイント URL
- **デフォルト値**:
  - `openai` プロバイダー: `https://api.openai.com/v1`
  - `claude` プロバイダー: `https://api.anthropic.com/v1`
- **必須**: -

```bash
# GLM Coding Plan
export LLM_BASE_URL=https://api.z.ai/api/coding/paas/v4

# ローカル LLM サーバー
export LLM_BASE_URL=http://localhost:11434/v1
```

---

## Discord 設定

### DISCORD_BOT_TOKEN

- **説明**: Discord Bot のトークン
- **デフォルト値**: なし
- **必須**: Discord 機能を使用する場合

```bash
export DISCORD_BOT_TOKEN=MTIzNDU2Nzg5MA.GhIjKl.MnOpQrStUvWxYzAbCdEfGhIjKlMnOpQrStUvWx
```

**トークンの取得方法:**

1. [Discord Developer Portal](https://discord.com/developers/applications) にアクセス
2. アプリケーションを作成または選択
3. 「Bot」セクションで「Reset Token」をクリック
4. 生成されたトークンをコピー

### ADMIN_USER_IDS

- **説明**: コマンド実行権限を持つ管理者ユーザー ID のリスト（カンマ区切り）
- **デフォルト値**: なし
- **必須**: -

```bash
export ADMIN_USER_IDS=123456789,987654321
```

**ユーザー ID の確認方法:**

1. Discord の「設定」>「詳細設定」>「開発者モード」を有効化
2. ユーザーを右クリック>「ID をコピー」

### DISCORD_MESSAGE_CACHE_SIZE

- **説明**: メッセージキャッシュのサイズ
- **デフォルト値**: `100`
- **必須**: -

```bash
export DISCORD_MESSAGE_CACHE_SIZE=200
```

---

## HTTP API 設定

### API_KEY

- **説明**: HTTP API の認証キー（設定した場合のみ認証が必要）
- **デフォルト値**: なし（認証なし）
- **必須**: -

```bash
export API_KEY=your-secure-api-key
```

**使用例:**

```bash
# API キーあり
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secure-api-key" \
  -d '{"message": "Hello"}'

# API キーなし
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello"}'
```

### API_PORT

- **説明**: HTTP API サーバーのポート番号
- **デフォルト値**: `3000`
- **必須**: -

```bash
export API_PORT=8080
```

### API_HOST

- **説明**: HTTP API サーバーのバインドアドレス
- **デフォルト値**: `0.0.0.0`（すべてのインターフェース）
- **必須**: -

```bash
export API_HOST=127.0.0.1  # ローカルホストのみ
```

### API_ALLOWED_ORIGINS

- **説明**: CORS 許可オリジンのリスト（カンマ区切り）
- **デフォルト値**: `*`（すべて許可）
- **必須**: -

```bash
export API_ALLOWED_ORIGINS=http://localhost:3000,https://example.com
```

---

## MCP 設定

### MCP_ENABLED

- **説明**: MCP（Model Context Protocol）統合を有効にするかどうか
- **デフォルト値**: `true`
- **必須**: -

```bash
export MCP_ENABLED=true
export MCP_ENABLED=false
```

### MCP_CONFIG_PATH

- **説明**: MCP 設定ファイルのパス
- **デフォルト値**: `mcp.json`
- **必須**: -

```bash
export MCP_CONFIG_PATH=/path/to/custom/mcp.json
```

---

## スケジューラー設定

### SCHEDULE_ENABLED

- **説明**: スケジューラー機能を有効にするかどうか
- **デフォルト値**: `true`
- **必須**: -

```bash
export SCHEDULE_ENABLED=true
export SCHEDULE_ENABLED=false
```

### SCHEDULE_CONFIG_PATH

- **説明**: スケジュール設定ファイルのパス
- **デフォルト値**: `schedule.toml`
- **必須**: -

```bash
export SCHEDULE_CONFIG_PATH=/path/to/custom/schedule.toml
```

---

## メモリ・データベース設定

### DB_PATH

- **説明**: SQLite データベースファイルのパス
- **デフォルト値**: `data/cc-gateway.db`
- **必須**: -

```bash
export DB_PATH=/path/to/custom/database.db
```

### MEMORY_ENABLED

- **説明**: メモリ機能を有効にするかどうか
- **デフォルト値**: `true`
- **必須**: -

```bash
export MEMORY_ENABLED=false
```

---

## その他

### RUST_LOG

- **説明**: ログレベルの設定
- **デフォルト値**: `info`
- **必須**: -

```bash
# エラーのみ
export RUST_LOG=error

# 通常ログ（デフォルト）
export RUST_LOG=info

# デバッグログ
export RUST_LOG=debug

# 最も詳細
export RUST_LOG=trace

# 特定のモジュールのみ
export RUST_LOG=cc_core=debug,cc_discord=info
```

### RUST_BACKTRACE

- **説明**: エラー時のバックトレース表示
- **デフォルト値**: `0`（無効）
- **必須**: -

```bash
# バックトレース有効
export RUST_BACKTRACE=1

# フルバックトレース
export RUST_BACKTRACE=full
```

---

## 環境変数の一覧表

| 変数名 | 説明 | デフォルト値 | 必須 | 設定セクション |
|--------|------|-------------|------|---------------|
| LLM_API_KEY | API認証キー | - | ○ | LLM |
| LLM_MODEL | モデル名 | claude-sonnet-4-20250514 | - | LLM |
| LLM_PROVIDER | プロバイダー | claude | - | LLM |
| LLM_BASE_URL | APIエンドポイント | （注1） | - | LLM |
| DISCORD_BOT_TOKEN | Discord Botトークン | - | - | Discord |
| ADMIN_USER_IDS | 管理者ユーザーID | - | - | Discord |
| DISCORD_MESSAGE_CACHE_SIZE | メッセージキャッシュサイズ | 100 | - | Discord |
| API_KEY | HTTP API認証キー | - | - | API |
| API_PORT | HTTP APIポート | 3000 | - | API |
| API_HOST | HTTP APIホスト | 0.0.0.0 | - | API |
| API_ALLOWED_ORIGINS | CORS許可オリジン | * | - | API |
| MCP_ENABLED | MCP有効フラグ | true | - | MCP |
| MCP_CONFIG_PATH | MCP設定ファイルパス | mcp.json | - | MCP |
| SCHEDULE_ENABLED | スケジューラー有効フラグ | true | - | スケジューラー |
| SCHEDULE_CONFIG_PATH | スケジュール設定パス | schedule.toml | - | スケジューラー |
| DB_PATH | データベースファイルパス | data/cc-gateway.db | - | メモリ |
| MEMORY_ENABLED | メモリ有効フラグ | true | - | メモリ |
| RUST_LOG | ログレベル | info | - | その他 |
| RUST_BACKTRACE | バックトレース表示 | 0 | - | その他 |

**注1:** `LLM_PROVIDER` によってデフォルト値が異なります。
- `openai` プロバイダー: `https://api.openai.com/v1`
- `claude` プロバイダー: `https://api.anthropic.com/v1`

---

## .env ファイルの使用

環境変数は `.env` ファイルで管理することもできます。

```bash
# .env
LLM_API_KEY=your-api-key
LLM_MODEL=claude-sonnet-4-20250514
LLM_PROVIDER=claude

DISCORD_BOT_TOKEN=your-bot-token
ADMIN_USER_IDS=123456789,987654321

API_PORT=3000
API_ALLOWED_ORIGINS=*
```

`.env` ファイルはプロジェクトルートディレクトリに配置すると自動的に読み込まれます。

**重要:** `.env` ファイルは機密情報を含むため、`.gitignore` に追加してください。
