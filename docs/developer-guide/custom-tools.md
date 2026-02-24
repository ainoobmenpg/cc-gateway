# カスタムツール開発ガイド

このドキュメントでは、cc-gateway でカスタムツールを開発する方法について説明します。

## Tool trait の概要

すべてのツールは `cc_core::Tool` trait を実装する必要があります。

```rust
use async_trait::async_trait;
use cc_core::{Tool, ToolResult, Result};
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// ツール名（Claude API で使用される識別子）
    fn name(&self) -> &str;

    /// ツールの説明（Claude がツールを選択する際に参照）
    fn description(&self) -> &str;

    /// 入力パラメータの JSON Schema
    fn input_schema(&self) -> Value;

    /// ツールを実行する
    async fn execute(&self, input: Value) -> Result<ToolResult>;
}
```

## 基本的なツール実装

### 最小限の例

```rust
use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};

pub struct HelloTool;

#[async_trait]
impl Tool for HelloTool {
    fn name(&self) -> &str {
        "hello"
    }

    fn description(&self) -> &str {
        "挨拶を返すツール"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "挨拶する相手の名前"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        // 入力のパース
        let name = input["name"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution(
                "name は文字列である必要があります".to_string()
            ))?;

        // 処理実行
        let message = format!("こんにちは、{}さん！", name);

        // 結果返却
        Ok(ToolResult::success(message))
    }
}
```

## BashTool の詳細な実装例

より実践的な例として、`BashTool` の実装パターンを示します。

### 入力構造体の定義

```rust
use serde::{Deserialize, Serialize};

/// ツールへの入力パラメータ
#[derive(Debug, Deserialize)]
struct BashInput {
    /// 実行するコマンド
    command: String,
    /// タイムアウト（ミリ秒）
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

fn default_timeout() -> u64 {
    120_000 // デフォルト 120 秒
}
```

### 出力構造体の定義

```rust
/// ツールからの出力
#[derive(Debug, Serialize)]
struct BashOutput {
    /// 標準出力
    stdout: String,
    /// 標準エラー出力
    stderr: String,
    /// 終了コード
    exit_code: Option<i32>,
    /// タイムアウトしたかどうか
    timed_out: bool,
}
```

### 実装の完全な例

```rust
use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub struct BashTool;

#[derive(Debug, Deserialize)]
struct BashInput {
    command: String,
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

fn default_timeout() -> u64 {
    120_000
}

#[derive(Debug, Serialize)]
struct BashOutput {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    timed_out: bool,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command with optional timeout. Use this for terminal operations."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds (default: 120000)",
                    "default": 120000
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        // 1. 入力のパースと検証
        let bash_input: BashInput = serde_json::from_value(input)
            .map_err(|e| cc_core::Error::ToolExecution(format!("Invalid input: {}", e)))?;

        // 2. パラメータの制限（セキュリティ）
        let timeout_ms = bash_input.timeout_ms.min(600_000);
        let duration = Duration::from_millis(timeout_ms);

        // 3. ログ出力（デバッグ用）
        tracing::debug!(
            command = %bash_input.command,
            timeout_ms,
            "Executing bash command"
        );

        // 4. コマンド実行（タイムアウト付き）
        let result = timeout(
            duration,
            Command::new("bash")
                .arg("-c")
                .arg(&bash_input.command)
                .output(),
        ).await;

        // 5. 結果処理
        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let bash_output = BashOutput {
                    stdout,
                    stderr,
                    exit_code: output.status.code(),
                    timed_out: false,
                };

                let output_str = serde_json::to_string_pretty(&bash_output)
                    .unwrap_or_else(|_| format!("{:?}", bash_output));

                // 終了コードで成功/失敗を判定
                if output.status.success() {
                    Ok(ToolResult::success(output_str))
                } else {
                    Ok(ToolResult::error(output_str))
                }
            }
            Ok(Err(e)) => {
                Ok(ToolResult::error(format!("Failed to execute command: {}", e)))
            }
            Err(_) => {
                Ok(ToolResult::error(format!("Command timed out after {}ms", timeout_ms)))
            }
        }
    }
}
```

## ツールの登録方法

### ToolManager への登録

```rust
use cc_core::ToolManager;
use std::sync::Arc;

fn main() {
    let mut tool_manager = ToolManager::new();

    // ツールを登録
    tool_manager.register(Arc::new(BashTool));
    tool_manager.register(Arc::new(HelloTool));
    tool_manager.register(Arc::new(MyCustomTool));

    // 登録されたツールを確認
    println!("登録済みツール: {:?}", tool_manager.tool_names());
}
```

### cc-tools への追加

新しいツールを `cc-tools` crate に追加する場合：

1. `crates/cc-tools/src/` に新しいファイルを作成（例: `my_tool.rs`）
2. `lib.rs` でモジュールをエクスポート

```rust
// crates/cc-tools/src/lib.rs

pub mod my_tool;  // 追加

pub use my_tool::MyTool;  // 追加

pub fn register_default_tools(manager: &mut ToolManager) {
    // 既存のツール...
    manager.register(Arc::new(BashTool));
    manager.register(Arc::new(ReadTool));

    // 新しいツールを追加
    manager.register(Arc::new(MyTool));
}
```

## エラーハンドリング

### ToolResult の使い分け

```rust
use cc_core::ToolResult;

// 成功時
Ok(ToolResult::success("操作が成功しました".to_string()))

// 失敗時（エラーとして扱われる）
Ok(ToolResult::error("操作が失敗しました".to_string()))

// 重大なエラー（処理自体が失敗した場合）
Err(cc_core::Error::ToolExecution("入力が無効です".to_string()))
```

### エラーの伝搬

```rust
async fn execute(&self, input: Value) -> Result<ToolResult> {
    // 入力検証エラー
    let config: MyConfig = serde_json::from_value(input)
        .map_err(|e| cc_core::Error::ToolExecution(format!("入力パースエラー: {}", e)))?;

    // 処理エラー
    let result = my_async_operation()
        .await
        .map_err(|e| cc_core::Error::ToolExecution(format!("処理エラー: {}", e)))?;

    Ok(ToolResult::success(result))
}
```

## テスト方法

### 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_hello_tool() {
        let tool = HelloTool;
        let input = json!({"name": "世界"});

        let result = tool.execute(input).await.unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("世界"));
    }

    #[tokio::test]
    async fn test_hello_tool_missing_name() {
        let tool = HelloTool;
        let input = json!({});  // name がない

        let result = tool.execute(input).await;

        assert!(result.is_err());
    }
}
```

### 統合テスト

```rust
#[tokio::test]
async fn test_tool_registration() {
    let mut manager = ToolManager::new();
    manager.register(Arc::new(HelloTool));

    assert!(manager.contains("hello"));
    assert_eq!(manager.len(), 1);
}

#[tokio::test]
async fn test_tool_execution_via_manager() {
    let mut manager = ToolManager::new();
    manager.register(Arc::new(HelloTool));

    let input = json!({"name": "テスト"});
    let result = manager.execute("hello", input).await.unwrap();

    assert!(!result.is_error);
}
```

## 高度なパターン

### 状態を持つツール

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CounterTool {
    count: Arc<Mutex<usize>>,
}

impl CounterTool {
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl Tool for CounterTool {
    fn name(&self) -> &str {
        "counter"
    }

    fn description(&self) -> &str {
        "カウンターを増やす"
    }

    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {}})
    }

    async fn execute(&self, _input: Value) -> Result<ToolResult> {
        let mut count = self.count.lock().await;
        *count += 1;
        Ok(ToolResult::success(format!("カウント: {}", *count)))
    }
}
```

### 非同期ファイル操作

```rust
pub struct AsyncReadTool;

#[async_trait]
impl Tool for AsyncReadTool {
    fn name(&self) -> &str {
        "async_read"
    }

    fn description(&self) -> &str {
        "非同期でファイルを読み込む"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("path が必要です".to_string()))?;

        // tokio::fs で非同期読み込み
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| cc_core::Error::ToolExecution(format!("読み込みエラー: {}", e)))?;

        Ok(ToolResult::success(content))
    }
}
```

### 外部 API 呼び出し

```rust
pub struct WeatherTool {
    client: reqwest::Client,
    api_key: String,
}

impl WeatherTool {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "天気情報を取得する"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "city": {"type": "string"}
            },
            "required": ["city"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let city = input["city"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("city が必要です".to_string()))?;

        let url = format!(
            "https://api.weather.com/current?city={}&key={}",
            city, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| cc_core::Error::ToolExecution(format!("API エラー: {}", e)))?;

        let weather: serde_json::Value = response
            .json()
            .await
            .map_err(|e| cc_core::Error::ToolExecution(format!("JSON パースエラー: {}", e)))?;

        Ok(ToolResult::success(
            serde_json::to_string_pretty(&weather).unwrap()
        ))
    }
}
```

## ベストプラクティス

### 1. 入力検証

```rust
// 必須項目のチェック
let name = input["name"]
    .as_str()
    .ok_or_else(|| cc_core::Error::ToolExecution("name は必須です".to_string()))?;

// 値の範囲チェック
let count = input["count"]
    .as_u64()
    .ok_or_else(|| cc_core::Error::ToolExecution("count は数値である必要があります".to_string()))?
    .min(1000);  // 上限を設定
```

### 2. タイムアウト設定

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    long_running_operation(),
).await;

match result {
    Ok(Ok(value)) => Ok(ToolResult::success(value)),
    Ok(Err(e)) => Ok(ToolResult::error(format!("エラー: {}", e))),
    Err(_) => Ok(ToolResult::error("タイムアウトしました".to_string())),
}
```

### 3. ログ出力

```rust
use tracing::{debug, info, error, instrument};

#[instrument(skip(self))]
async fn execute(&self, input: Value) -> Result<ToolResult> {
    debug!(input = ?input, "ツール実行開始");

    // ... 処理 ...

    info!("ツール実行完了");
    Ok(ToolResult::success(result))
}
```

### 4. リソースクリーンアップ

```rust
use std::io::BufReader;
use std::fs::File;

async fn execute(&self, input: Value) -> Result<ToolResult> {
    let path = "some_file.txt";

    // スコープ内でリソースを管理
    let result = {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        // 処理...
    };  // ここで file が自動的にクローズされる

    Ok(ToolResult::success(result))
}
```

## トラブルシューティング

### よくある問題

| 問題 | 原因 | 解決策 |
|------|------|--------|
| ツールが呼ばれない | `name()` が一意でない | 他のツールと名前が被っていないか確認 |
| 入力がパースできない | JSON Schema と実際の入力が不一致 | Schema の `properties` を確認 |
| テストがハングする | タイムアウト設定がない | `tokio::time::timeout` を使用 |
| メモリリーク | 内部状態の参照サイクル | `Arc` の使用を見直す |
