# MCP (Model Context Protocol) ガイド

MCP (Model Context Protocol) は、AI アシスタントと外部ツールやデータソースを接続するための標準プロトコルです。cc-gateway は MCP を使用して機能を拡張できます。

## MCP とは

MCP は以下を実現するためのプロトコルです：

- 🔌 **外部ツール統合**: Git、データベース、API などと接続
- 📁 **データソースアクセス**: ファイルシステム、クラウドストレージ
- 🛠️ **機能拡張**: 新しいツールをサーバーレスで追加

### MCP の仕組み

```
cc-gateway (MCP クライアント) ←→ MCP サーバー
                                  ↓
                            外部リソース
```

---

## 設定ファイル

MCP 設定は `mcp.json` で行います：

```toml
# cc-gateway.toml で設定ファイルパスを指定
[mcp]
enabled = true
config_path = "mcp.json"
```

### 環境変数

```bash
MCP_ENABLED=true
MCP_CONFIG_PATH=mcp.json
```

---

## mcp.json の基本形式

```json
{
  "servers": [
    {
      "name": "サーバー名",
      "command": "起動コマンド",
      "args": ["追加引数"],
      "env": {
        "環境変数": "値"
      },
      "enabled": true
    }
  ]
}
```

---

## 利用可能な MCP サーバー

### Git MCP サーバー

Git リポジトリの操作を可能にします。

```json
{
  "servers": [
    {
      "name": "git",
      "command": "uvx",
      "args": ["mcp-server-git"],
      "enabled": true
    }
  ]
}
```

**使用可能なツール:**
- `git_clone` - リポジトリのクローン
- `git_log` - コミットログの取得
- `git_diff` - 変更の差分表示
- `git_status` - リポジトリの状態確認

### Filesystem MCP サーバー

ファイルシステムへの高度なアクセスを提供します。

```json
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/allowed/path"],
      "enabled": true
    }
  ]
}
```

### SQLite MCP サーバー

SQLite データベースへのクエリを実行できます。

```json
{
  "servers": [
    {
      "name": "sqlite",
      "command": "uvx",
      "args": ["mcp-server-sqlite", "--db-path", "./data.db"],
      "enabled": true
    }
  ]
}
```

### Brave Search MCP サーバー

Brave Search API を使用した Web 検索を提供します。

```json
{
  "servers": [
    {
      "name": "brave-search",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "your-api-key"
      },
      "enabled": true
    }
  ]
}
```

### GitHub MCP サーバー

GitHub リポジトリとの連携を可能にします。

```json
{
  "servers": [
    {
      "name": "github",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "your-github-token"
      },
      "enabled": true
    }
  ]
}
```

### Postgres MCP サーバー

PostgreSQL データベースへの接続を提供します。

```json
{
  "servers": [
    {
      "name": "postgres",
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-postgres",
        "postgresql://user:password@localhost:5432/dbname"
      ],
      "enabled": true
    }
  ]
}
```

### Puppeteer MCP サーバー

ヘッドレスブラウザ操作を可能にします。

```json
{
  "servers": [
    {
      "name": "puppeteer",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-puppeteer"],
      "enabled": true
    }
  ]
}
```

---

## 完全な設定例

### 複数の MCP サーバーを使用

```json
{
  "servers": [
    {
      "name": "git",
      "command": "uvx",
      "args": ["mcp-server-git"],
      "enabled": true
    },
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/projects"],
      "enabled": true
    },
    {
      "name": "brave-search",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "${BRAVE_API_KEY}"
      },
      "enabled": true
    },
    {
      "name": "sqlite",
      "command": "uvx",
      "args": ["mcp-server-sqlite", "--db-path", "./data/app.db"],
      "enabled": true
    }
  ]
}
```

---

## 使用例

### Git 操作

```
ユーザー: リポジトリの変更履歴を確認して
AI: ツール: git_log
リポジトリのコミット履歴を取得します...

[結果]
commit abc123 (HEAD -> main)
Author: user <user@example.com>
Date: Mon Feb 24 10:00:00 2025

    Add new feature

commit def456
Author: user <user@example.com>
Date: Sun Feb 23 15:30:00 2025

    Fix bug in parser
```

### データベースクエリ

```
ユーザー: users テーブルの全データを取得して
AI: ツール: sqlite_query
クエリ: SELECT * FROM users;

[結果]
| id | name      | email               |
|----|-----------|---------------------|
| 1  | Alice     | alice@example.com   |
| 2  | Bob       | bob@example.com     |
```

### Web 検索（Brave）

```
ユーザー: Rust の最新バージョンは？
AI: ツール: brave_search
クエリ: Rust latest version 2025

[結果]
Rust 1.85 が最新バージョンです（2025年2月20日リリース）。
```

---

## MCP サーバーのインストール

### uvx を使用する場合（推奨）

```bash
# uv をインストール
curl -LsSf https://astral.sh/uv/install.sh | sh

# MCP サーバーを実行
uvx mcp-server-git
```

### npx を使用する場合

```bash
# Node.js がインストールされている必要があります
npx -y @modelcontextprotocol/server-filesystem
```

### Docker を使用する場合

```bash
docker run -it --rm \
  -v /path/to/repo:/repo \
  mcp-server-git
```

---

## 環境変数の使用

API キーなどの機密情報は環境変数で管理します：

```json
{
  "servers": [
    {
      "name": "brave-search",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "${BRAVE_API_KEY}"
      },
      "enabled": true
    }
  ]
}
```

`.env` ファイル：

```bash
BRAVE_API_KEY=your-actual-api-key
```

---

## カスタム MCP サーバーの作成

### TypeScript での実装例

```typescript
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

const server = new Server(
  {
    name: 'my-custom-server',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// ツールを登録
server.setRequestHandler('tools/list', async () => ({
  tools: [
    {
      name: 'my_tool',
      description: 'My custom tool',
      inputSchema: {
        type: 'object',
        properties: {
          param: { type: 'string' },
        },
        required: ['param'],
      },
    },
  ],
}));

// ツールの実行
server.setRequestHandler('tools/call', async (request) => {
  if (request.params.name === 'my_tool') {
    return {
      content: [{
        type: 'text',
        text: `Result: ${request.params.arguments.param}`,
      }],
    };
  }
});

// サーバーを起動
const transport = new StdioServerTransport();
await server.connect(transport);
```

### cc-gateway から使用

```json
{
  "servers": [
    {
      "name": "my-custom",
      "command": "node",
      "args": ["dist/my-server.js"],
      "enabled": true
    }
  ]
}
```

---

## トラブルシューティング

### MCP サーバーが起動しない

1. コマンドパスが正しいか確認
2. 必要な依存関係がインストールされているか確認
3. ログでエラーメッセージを確認

```bash
# MCP サーバーを単体でテスト
uvx mcp-server-git
```

### 環境変数が読み込まれない

1. `.env` ファイルが正しい場所にあるか確認
2. 環境変数名が正しいか確認
3. `${VAR_NAME}` 形式を使用しているか確認

### ツールが見つからない

1. MCP サーバーが正常に起動しているか確認
2. `mcp.json` で `enabled: true` になっているか確認
3. サーバー名が正しいか確認

---

## ベストプラクティス

### 最小限の権限

```json
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/specific/path/only"  // 必要なパスのみ
      ],
      "enabled": true
    }
  ]
}
```

### エラーハンドリング

MCP サーバーが利用できない場合、cc-gateway は組み込みツールにフォールバックします。

### ログの確認

```bash
# MCP 関連のログをフィルタリング
cargo run 2>&1 | grep -i mcp
```

---

## 公式 MCP サーバー一覧

| サーバー | 説明 | インストール |
|---------|------|-------------|
| `mcp-server-git` | Git 操作 | `uvx mcp-server-git` |
| `@modelcontextprotocol/server-filesystem` | ファイルシステム | `npx -y @modelcontextprotocol/server-filesystem` |
| `@modelcontextprotocol/server-brave-search` | Brave 検索 | `npx -y @modelcontextprotocol/server-brave-search` |
| `@modelcontextprotocol/server-github` | GitHub 統合 | `npx -y @modelcontextprotocol/server-github` |
| `@modelcontextprotocol/server-postgres` | PostgreSQL | `npx -y @modelcontextprotocol/server-postgres` |
| `@modelcontextprotocol/server-sqlite` | SQLite | `uvx mcp-server-sqlite` |
| `@modelcontextprotocol/server-puppeteer` | ブラウザ操作 | `npx -y @modelcontextprotocol/server-puppeteer` |

詳細は [MCP 公式リポジトリ](https://github.com/modelcontextprotocol) を参照してください。
