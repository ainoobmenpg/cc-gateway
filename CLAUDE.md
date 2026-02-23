# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

**cc-gateway** は Pure Rust で実装された Claude API Gateway です。OpenClaw の代替として Claude Code と同等の機能を提供します。

## 開発コマンド

```bash
# ビルド
cargo build

# リリースビルド
cargo build --release

# 全テスト実行
cargo test

# 特定のテストのみ実行
cargo test test_name

# 特定の crate のテスト
cargo test -p cc-core

# Lint (clippy)
cargo clippy --all-targets --all-features -- -D warnings

# フォーマット
cargo fmt

# 実行
cargo run
```

## アーキテクチャ

```
cc-gateway (workspace)
├── crates/
│   ├── cc-core/        # コアライブラリ (Tool trait, Claude client, Session, Memory)
│   ├── cc-tools/       # 組み込みツール (Bash, Read, Write, Edit, Glob, Grep)
│   ├── cc-mcp/         # MCPクライアント (未実装)
│   ├── cc-discord/     # Discord Gateway (未実装)
│   ├── cc-api/         # HTTP API (未実装)
│   └── cc-gateway/     # メインバイナリ
```

### 主要コンポーネント

**cc-core** (`crates/cc-core/src/`):
- `tool/` - Tool trait と ToolManager。新しいツールは `Tool` trait を実装する
- `llm/` - Claude API HTTP クライアントと Agent Loop
- `session/` - セッション管理 (SQLite 永続化)
- `memory/` - メモリシステム (SQLite)

**cc-tools** (`crates/cc-tools/src/`):
- 組み込みツール実装。`register_default_tools()` で一括登録

### Tool の実装パターン

```rust
use cc_core::{Tool, ToolResult};
use async_trait::async_trait;

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Tool description" }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "param": { "type": "string" } }
        })
    }
    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult> {
        // 実装
        Ok(ToolResult::success("result"))
    }
}
```

## 環境変数

```bash
CLAUDE_API_KEY=sk-ant-...    # 必須
CLAUDE_MODEL=claude-sonnet-4-20250514  # デフォルト
DB_PATH=data/cc-gateway.db   # SQLite パス
DISCORD_BOT_TOKEN=...        # オプション (Phase 4)
API_KEY=...                  # オプション (Phase 5)
API_PORT=3000
```

## 技術スタック

- Rust 2024 Edition (rustc 1.85+)
- 非同期ランタイム: tokio
- HTTP クライアント: reqwest (rustls-tls)
- データベース: rusqlite (bundled)
- Discord: serenity (Phase 4)
- MCP: rmcp (Phase 3)
