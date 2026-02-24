# 設定ガイド

このガイドでは、cc-gateway の詳細な設定方法について説明します。設定ファイルの全項目解説、環境変数による上書きルール、プロバイダー別の設定例をカバーしています。

## 設定の仕組み

cc-gateway の設定は以下の優先順位で読み込まれます：

```
優先度: 高 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━> 低
       環境変数      >    TOML 設定ファイル    >   デフォルト値
```

つまり：
1. 環境変数が設定されている場合は、設定ファイルよりも優先されます
2. 設定ファイルが存在しない場合は、デフォルト値が使用されます
3. 環境変数で上書きされていない項目のみ、設定ファイルの値が使用されます

## 設定ファイルの基本

### 設定ファイルの作成

プロジェクトルートにあるテンプレートをコピーして使用します：

```bash
cp cc-gateway.toml.example cc-gateway.toml
```

### 環境変数の展開

設定ファイル内の `${VAR_NAME}` は、対応する環境変数の値に置換されます：

```toml
[llm]
api_key = "${LLM_API_KEY}"  # 環境変数 LLM_API_KEY の値に置換
```

これは、API キーのような機密情報を設定ファイルに直接書かずに済むため、推奨される方法です。

## 設定項目の詳細

### LLM 設定 (`[llm]`)

LLM（大規模言語モデル）プロバイダーの設定です。

| 項目 | 型 | デフォルト値 | 説明 |
|------|----|-------------|------|
| `provider` | string | `"claude"` | API プロバイダー（`"claude"` または `"openai"`） |
| `model` | string | `"claude-sonnet-4-20250514"` | 使用するモデル名 |
| `api_key` | string | - | API キー（環境変数推奨） |
| `base_url` | string | - | カスタム API エンドポイント |

**provider の値**：
- `"claude"` - Anthropic Claude API（デフォルトのベース URL: `https://api.anthropic.com/v1`）
- `"openai"` - OpenAI 互換 API

### Discord 設定 (`[discord]`)

Discord Bot の設定です。

| 項目 | 型 | デフォルト値 | 説明 |
|------|----|-------------|------|
| `token` | string | - | Discord Bot トークン |
| `admin_user_ids` | array of int | `[]` | 管理者ユーザー ID リスト |

### HTTP API 設定 (`[api]`)

HTTP API サーバーの設定です。

| 項目 | 型 | デフォルト値 | 説明 |
|------|----|-------------|------|
| `port` | int | `3000` | API サーバーのポート番号 |
| `key` | string | - | API 認証キー（オプション） |
| `allowed_origins` | array of string | `["*"]` | CORS 許可オリジン |

### メモリ設定 (`[memory]`)

会話履歴とメモリの永続化設定です。

| 項目 | 型 | デフォルト値 | 説明 |
|------|----|-------------|------|
| `db_path` | string | `"data/cc-gateway.db"` | SQLite データベースファイルのパス |

### MCP 設定 (`[mcp]`)

Model Context Protocol（MCP）統合の設定です。

| 項目 | 型 | デフォルト値 | 説明 |
|------|----|-------------|------|
| `enabled` | bool | `true` | MCP 統合を有効にするかどうか |
| `config_path` | string | `"mcp.json"` | MCP 設定ファイルのパス |

### スケジューラー設定 (`[scheduler]`)

定期実行タスクの設定です。

| 項目 | 型 | デフォルト値 | 説明 |
|------|----|-------------|------|
| `enabled` | bool | `true` | スケジューラーを有効にするかどうか |
| `config_path` | string | `"schedule.toml"` | スケジュール設定ファイルのパス |

## 環境変数による設定

設定ファイルの各項目は、環境変数で上書きできます。環境変数の命名規則は以下の通りです：

```
<セクション名>_<項目名>
```

すべて大文字にする必要があります。

### 環境変数一覧

| 環境変数 | 対応する設定項目 | デフォルト値 |
|---------|----------------|-------------|
| `LLM_PROVIDER` | `[llm].provider` | `"claude"` |
| `LLM_MODEL` | `[llm].model` | `"claude-sonnet-4-20250514"` |
| `LLM_API_KEY` | `[llm].api_key` | - |
| `LLM_BASE_URL` | `[llm].base_url` | - |
| `DISCORD_BOT_TOKEN` | `[discord].token` | - |
| `ADMIN_USER_IDS` | `[discord].admin_user_ids` | `[]` |
| `API_PORT` | `[api].port` | `3000` |
| `API_KEY` | `[api].key` | - |
| `API_ALLOWED_ORIGINS` | `[api].allowed_origins` | `["*"]` |
| `DB_PATH` | `[memory].db_path` | `"data/cc-gateway.db"` |
| `MCP_ENABLED` | `[mcp].enabled` | `true` |
| `MCP_CONFIG_PATH` | `[mcp].config_path` | `"mcp.json"` |
| `SCHEDULE_ENABLED` | `[scheduler].enabled` | `true` |
| `SCHEDULE_CONFIG_PATH` | `[scheduler].config_path` | `"schedule.toml"` |

### 配列項目の環境変数

配列型の設定項目（`admin_user_ids` など）は、カンマ区切りで指定します：

```bash
export ADMIN_USER_IDS="123456789,987654321"
```

## プロバイダー別設定例

### Anthropic Claude API の設定

```toml
[llm]
provider = "claude"
model = "claude-sonnet-4-20250514"  # または "claude-opus-4-20250514"
api_key = "${ANTHROPIC_API_KEY}"
# base_url は省略可能（デフォルト: https://api.anthropic.com/v1）
```

**利用可能なモデル**（2025年2月現在）：
- `claude-opus-4-20250514` - 最高品質
- `claude-sonnet-4-20250514` - バランス型
- `claude-haiku-4-20250514` - 高速・低コスト

### GLM Coding Plan の設定

```toml
[llm]
provider = "openai"
model = "glm-4.7"
base_url = "https://api.z.ai/api/coding/paas/v4"
api_key = "${GLM_API_KEY}"
```

### OpenAI API の設定

```toml
[llm]
provider = "openai"
model = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "${OPENAI_API_KEY}"
```

### その他の OpenAI 互換 API

OpenAI 互換の形式であれば、様々なプロバイダーを使用できます：

```toml
[llm]
provider = "openai"
model = "your-model-name"
base_url = "https://your-custom-endpoint.com/v1"
api_key = "${YOUR_API_KEY}"
```

## 設定ファイルのサンプル

### 最小構成（Claude デフォルト）

```toml
[llm]
provider = "claude"
model = "claude-sonnet-4-20250514"
api_key = "${ANTHROPIC_API_KEY}"
```

### 全機能有効化

```toml
[llm]
provider = "claude"
model = "claude-sonnet-4-20250514"
api_key = "${ANTHROPIC_API_KEY}"

[discord]
token = "${DISCORD_BOT_TOKEN}"
admin_user_ids = [123456789, 987654321]

[api]
port = 3000
key = "${API_KEY}"
allowed_origins = ["*"]

[memory]
db_path = "data/cc-gateway.db"

[mcp]
enabled = true
config_path = "mcp.json"

[scheduler]
enabled = true
config_path = "schedule.toml"
```

## 設定の確認方法

設定が正しく適用されているか確認するには：

```bash
# デバッグモードで起動（設定情報が表示されます）
cc-gateway --verbose --cli
```

起動時の出力に、読み込まれた設定値が表示されます。

## トラブルシューティング

### Q: 設定ファイルが読み込まれていないようです

A: 以下を確認してください：

1. 設定ファイルがプロジェクトルートにあるか
2. ファイル名が `cc-gateway.toml` であるか（`cc-gateway.toml.example` ではない）
3. 起動時に `-c` オプションで明示的に指定することも可能：

```bash
cc-gateway -c /path/to/your-config.toml --cli
```

### Q: 環境変数が反映されません

A: 以下を確認してください：

1. 環境変数を設定した後、ターミナルを再起動したか
2. `.env` ファイルを使用している場合、そのファイルが正しい場所にあるか
3. 環境変数名が正しいか（大文字・小文字の区別）

### Q: API キーを設定しているが認証エラーが発生します

A: 以下を確認してください：

1. API キーが正しいか（余分なスペースが含まれていないか）
2. プロバイダー設定が正しいか（`provider` の値）
3. `base_url` が正しいか（プロバイダーごとに異なる）

### Q: 複数のプロバイダーを併用したいです

A: 現在のバージョンでは、同時に使用できるプロバイダーは1つのみです。複数のプロバイダーを併用したい場合は、別の設定ファイルを作成して切り替えて使用してください：

```bash
# Claude 用設定
cc-gateway -c cc-gateway-claude.toml --cli

# GLM 用設定
cc-gateway -c cc-gateway-glm.toml --cli
```

## 次のステップ

設定が完了したら、以下のドキュメントを参照してください：

- [ユーザーガイド - CLI](../user-guide/cli.md) - CLI モードの詳細な使い方
- [ユーザーガイド - API](../user-guide/api.md) - HTTP API の使用方法
- [ユーザーガイド - Discord](../user-guide/discord.md) - Discord Bot の設定と使い方
