# cc-mcp

cc-gateway の MCP (Model Context Protocol) 統合を提供するクレートです。

## 概要

`cc-mcp` は MCP サーバーと通信し、MCP ツールを cc-gateway のツールシステム（`cc_core::Tool` trait）に統合する機能を提供します。

## 機能

- **MCP クライアント**: MCP サーバー（子プロセス）との通信
- **ツールアダプター**: MCP ツールを cc-core Tool に変換
- **サーバーレジストリ**: 複数の MCP サーバーを管理

## 使用方法

### 基本的なセットアップ

```rust
use cc_mcp::{McpClient, McpConfig, McpRegistry};
use cc_core::ToolManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // MCP 設定
    let config = McpConfig {
        servers: vec![
            McpServerConfig {
                name: "git".to_string(),
                command: "uvx".to_string(),
                args: vec!["mcp-server-git".to_string()],
                enabled: true,
            }
        ],
    };

    // ツールマネージャーに MCP ツールを登録
    let mut tool_manager = ToolManager::new();
    initialize_mcp_tools(&mut tool_manager, &config).await?;

    Ok(())
}
```

### 設定ファイルを使用

`mcp.json` を作成:

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
      "command": "uvx",
      "args": ["mcp-server-filesystem", "/path/to/allowed"],
      "enabled": true
    },
    {
      "name": "postgres",
      "command": "uvx",
      "args": ["mcp-server-postgres", "postgresql://..."],
      "enabled": false
    }
  ]
}
```

設定を読み込んで初期化:

```rust
use cc_mcp::{initialize_mcp_tools_from_file, McpConfig};

// ファイルから設定を読み込み
let config = McpConfig::from_file("mcp.json")?;

// ツールを登録
let mut tool_manager = ToolManager::new();
initialize_mcp_tools_from_file(&mut tool_manager, "mcp.json").await?;
```

## API

### McpClient

MCP サーバーとの通信を管理します。

```rust
use cc_mcp::McpClient;

// クライアントを作成
let client = McpClient::new(server_config).await?;

// 利用可能なツールを取得
let tools = client.list_tools().await?;

// ツールを実行
let result = client.call_tool("tool_name", json!({"param": "value"})).await?;
```

### McpToolAdapter

MCP ツールを cc-core Tool に適合させます。

```rust
use cc_mcp::McpToolAdapter;
use std::sync::Arc;

// アダプターを作成
let adapter = McpToolAdapter::new(
    Arc::new(client),
    mcp_tool
);

// Tool trait を実装しているので、ToolManager に登録可能
tool_manager.register(Arc::new(adapter));
```

### McpRegistry

複数の MCP サーバーを管理します。

```rust
use cc_mcp::McpRegistry;

let registry = McpRegistry::new();

// サーバーを登録
registry.add_server(client).await?;

// 全サーバーのツールを取得
let all_tools = registry.list_all_tools().await?;

// 特定のツールを実行
let result = registry.execute_tool("server_name", "tool_name", input).await?;
```

### ユーティリティ関数

```rust
use cc_mcp::initialize_mcp_tools;

// 設定から MCP ツールを初期化して登録
initialize_mcp_tools(&mut tool_manager, &mcp_config).await?;

// ファイルから設定を読み込んで初期化
initialize_mcp_tools_from_file(&mut tool_manager, "mcp.json").await?;
```

## MCP サーバー設定

### サーバー設定オプション

```rust
use cc_mcp::McpServerConfig;

let config = McpServerConfig {
    name: "my_server".to_string(),      // サーバー名（一意）
    command: "uvx".to_string(),          // 実行コマンド
    args: vec!["mcp-server-xxx".to_string()],  // コマンド引数
    env: vec![  // 環境変数（オプション）
        ("API_KEY".to_string(), "xxx".to_string())
    ],
    enabled: true,                       // 有効/無効
};
```

### 対応 MCP サーバー例

| サーバー | コマンド | 説明 |
|---------|---------|------|
| git | `uvx mcp-server-git` | Git リポジトリ操作 |
| filesystem | `uvx mcp-server-filesystem` | ファイルシステム操作 |
| postgres | `uvx mcp-server-postgres` | PostgreSQL データベース |
| sqlite | `uvx mcp-server-sqlite` | SQLite データベース |
| brave-search | `uvx mcp-server-brave-search` | Brave 検索 |
| puppeteer | `uvx mcp-server-puppeteer` | ブラウザ操作 |

## データフロー

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  ToolManager    │────▶│ McpToolAdapter  │────▶│   McpClient     │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                           │
                                                           ▼
                                                   ┌─────────────────┐
                                                   │  MCP サーバー    │
                                                   │  (子プロセス)    │
                                                   └─────────────────┘
```

## エラーハンドリング

```rust
use cc_mcp::McpError;

match initialize_mcp_tools(&mut tool_manager, &config).await {
    Ok(_) => println!("MCP ツールを登録しました"),
    Err(McpError::ServerStartFailed(e)) => {
        eprintln!("サーバー起動エラー: {}", e);
    }
    Err(McpError::ToolExecution(e)) => {
        eprintln!("ツール実行エラー: {}", e);
    }
    Err(e) => {
        eprintln!("その他のエラー: {}", e);
    }
}
```

## トラブルシューティング

### サーバーが起動しない

- コマンドがインストールされているか確認
- `uvx` が使用可能か確認: `uvx --version`
- 手動でサーバーを実行してエラーを確認

```bash
# 手動実行テスト
uvx mcp-server-git
```

### ツールが実行できない

- 入力パラメータが正しいか確認
- MCP サーバーのログを確認

### デバッグモード

```rust
use tracing::Level;

tracing::subscriber::set_global_default(
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish()
)?;
```

## 依存クレート

- `cc-core`: Tool trait と共通型
- `rmcp`: MCP クライアント実装
- `tokio`: 非同期ランタイム
- `serde`/`serde_json`: シリアライゼーション

## ライセンス

MIT
