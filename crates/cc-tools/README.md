# cc-tools

cc-gateway の組み込みツールを提供するクレートです。

## 概要

`cc-tools` は cc-gateway で使用する基本的なツールの実装を提供します。これらのツールは `cc_core::Tool` trait を実装しており、`ToolManager` に登録して使用します。

## 提供するツール

| ツール | 名前 | 機能 |
|-------|------|------|
| **BashTool** | `bash` | シェルコマンドを実行 |
| **ReadTool** | `read` | ファイルを読み込み |
| **WriteTool** | `write` | ファイルを書き込み |
| **EditTool** | `edit` | ファイルを編集（文字列置換） |
| **GlobTool** | `glob` | ファイルパターンマッチング |
| **GrepTool** | `grep` | ファイル内容の検索 |
| **WebSearchTool** | `web_search` | Web 検索 |
| **WebFetchTool** | `web_fetch` | Web ページの取得 |

## 使用方法

### すべてのツールを登録

```rust
use cc_core::ToolManager;
use cc_tools::register_default_tools;

let mut tool_manager = ToolManager::new();
register_default_tools(&mut tool_manager);
```

### 個別にツールを登録

```rust
use cc_core::ToolManager;
use cc_tools::{BashTool, ReadTool};
use std::sync::Arc;

let mut tool_manager = ToolManager::new();
tool_manager.register(Arc::new(BashTool));
tool_manager.register(Arc::new(ReadTool));
```

## ツール詳細

### BashTool

シェルコマンドを実行します。

```rust
use cc_tools::BashTool;
use serde_json::json;

let tool = BashTool;
let input = json!({
    "command": "echo hello",
    "timeout_ms": 5000  // オプション、デフォルト 120000ms
});
let result = tool.execute(input).await?;
```

**出力形式:**
```json
{
  "stdout": "hello\n",
  "stderr": "",
  "exit_code": 0,
  "timed_out": false
}
```

### ReadTool

ファイルを読み込みます。

```rust
use cc_tools::ReadTool;
use serde_json::json;

let tool = ReadTool;
let input = json!({
    "file_path": "/path/to/file.txt",
    "offset": 0,     // オプション、開始行
    "limit": 100     // オプション、読み込み行数
});
let result = tool.execute(input).await?;
```

### WriteTool

ファイルを書き込みます（上書き）。

```rust
use cc_tools::WriteTool;
use serde_json::json;

let tool = WriteTool;
let input = json!({
    "file_path": "/path/to/file.txt",
    "content": "Hello, world!"
});
let result = tool.execute(input).await?;
```

**注意:** 既存ファイルを書き込む前に、事前に `ReadTool` で読み込む必要があります。

### EditTool

ファイル内の文字列を置換します。

```rust
use cc_tools::EditTool;
use serde_json::json;

let tool = EditTool;
let input = json!({
    "file_path": "/path/to/file.txt",
    "old_string": "old text",
    "new_string": "new text"
});
let result = tool.execute(input).await?;
```

### GlobTool

ファイルパターンにマッチするファイルを検索します。

```rust
use cc_tools::GlobTool;
use serde_json::json;

let tool = GlobTool;
let input = json!({
    "pattern": "**/*.rs",
    "path": "/path/to/search"  // オプション、デフォルトはカレントディレクトリ
});
let result = tool.execute(input).await?;
```

**出力形式:**
```json
{
  "matches": [
    "/path/to/file1.rs",
    "/path/to/file2.rs"
  ]
}
```

### GrepTool

ファイル内のテキストを検索します。

```rust
use cc_tools::GrepTool;
use serde_json::json;

let tool = GrepTool;
let input = json!({
    "pattern": "TODO",
    "path": "/path/to/search",
    "glob": "*.rs",        // オプション、ファイルパターン
    "output_mode": "content"  // "content" | "files_with_matches" | "count"
});
let result = tool.execute(input).await?;
```

### WebSearchTool

Web 検索を実行します。

```rust
use cc_tools::WebSearchTool;
use serde_json::json;

let tool = WebSearchTool::new();
let input = json!({
    "query": "Rust async programming"
});
let result = tool.execute(input).await?;
```

### WebFetchTool

Web ページの内容を取得します。

```rust
use cc_tools::WebFetchTool;
use serde_json::json;

let tool = WebFetchTool::new();
let input = json!({
    "url": "https://example.com"
});
let result = tool.execute(input).await?;
```

## 新しいツールの追加方法

1. `crates/cc-tools/src/` に新しいファイルを作成（例: `my_tool.rs`）

```rust
//! My Tool

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "Description of my tool"
    }

    fn input_schema(&self) -> Value {
        json!({
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

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        // 実装
        Ok(ToolResult::success("Done!"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_tool() {
        let tool = MyTool;
        let input = json!({"param": "test"});
        let result = tool.execute(input).await.unwrap();
        assert!(!result.is_error);
    }
}
```

2. `lib.rs` でモジュールをエクスポート

```rust
pub mod my_tool;
pub use my_tool::MyTool;
```

3. `register_default_tools` に追加

```rust
pub fn register_default_tools(manager: &mut ToolManager) {
    // 既存のツール...
    manager.register(Arc::new(BashTool));

    // 新しいツールを追加
    manager.register(Arc::new(MyTool));
}
```

4. `Cargo.toml` に必要な依存を追加

```toml
[dependencies]
# 必要に応じて追加
```

## テスト

```bash
# 全テスト実行
cargo test -p cc-tools

# 特定のツールのテスト
cargo test -p cc-tools test_bash

# ドキュメンテーションテスト
cargo test -p cc-tools --doc
```

## 依存クレート

- `cc-core`: Tool trait と共通型
- `tokio`: 非同期ランタイム
- `serde`/`serde_json`: シリアライゼーション
- `glob`: ファイルパターンマッチング
- `grep-regex`: ripgrep ライブラリ

## ライセンス

MIT
