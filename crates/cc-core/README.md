# cc-core

cc-gateway のコアライブラリです。Claude API との通信、ツールシステム、セッション管理、メモリシステムなどの基本的な機能を提供します。

## 概要

`cc-core` は cc-gateway プロジェクトの中核となるライブラリで、以下の機能を提供します：

- **Tool System**: LLM から呼び出し可能なツールの定義と実行
- **LLM Client**: Claude API および OpenAI 互換 API との通信
- **Session Management**: チャットセッションの永続化
- **Memory System**: 長期記憶の保存と検索
- **Sub-Agents**: タスクの委譲と並列実行
- **Audit Logging**: 操作ログと暗号化
- **Skills**: スキルファイルのロードと実行

## 提供する機能

### Tool System

ツールの登録、検索、実行を管理します。

```rust
use cc_core::{Tool, ToolManager, ToolResult};
use std::sync::Arc;

// ツールを登録
let mut manager = ToolManager::new();
manager.register(Arc::new(MyTool));

// ツールを定義形式で取得
let definitions = manager.definitions();

// ツールを実行
let result = manager.execute("my_tool", json!({"param": "value"})).await?;
```

### LLM Client

Claude API および OpenAI 互換 API と通信します。

```rust
use cc_core::{ClaudeClient, LlmConfig, LlmProvider, Message, MessageContent};

let config = LlmConfig {
    provider: LlmProvider::Claude,
    api_key: "sk-ant-...".to_string(),
    model: "claude-sonnet-4-20250514".to_string(),
    ..Default::default()
};

let client = ClaudeClient::new(config);

let response = client.messages(MessagesRequest {
    model: "claude-sonnet-4-20250514".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: MessageContent::Text("Hello!".to_string()),
        }
    ],
    max_tokens: 4096,
    ..Default::default()
}).await?;
```

### Session Management

チャットセッションを SQLite に保存します。

```rust
use cc_core::{Session, SessionManager, SessionStore};

// セッションマネージャーの作成
let store = SessionStore::new("data/sessions.db")?;
let manager = SessionManager::new(store);

// 新規セッションの作成
let session = manager.create("user123").await?;

// メッセージの追加
session.add_message(Message { ... }).await?;

// セッションの取得
let session = manager.get("session_id").await?;
```

### Memory System

長期記憶の保存とベクトル検索をサポートします。

```rust
use cc_core::{Memory, MemoryStore};

let store = MemoryStore::new("data/memories.db")?;

// 記憶を保存
let memory = Memory {
    content: "プロジェクトの設定ファイルは config.toml にあります".to_string(),
    embedding: None,  // 自動計算されます
    metadata: Some(json!({"project": "cc-gateway"})),
};
store.add(memory).await?;

// 記憶を検索
let results = store.search("設定").await?;
```

### Sub-Agents

タスクをサブエージェントに委譲して並列実行します。

```rust
use cc_core::{SubAgentManager, SubAgentTask, TaskPriority};

let manager = SubAgentManager::new();

// タスクを作成
let task = SubAgentTask::builder()
    .prompt("README.md を読んで要約してください")
    .priority(TaskPriority::High)
    .build();

// タスクを委譲
let task_id = manager.delegate(task).await?;

// 結果を取得
let result = manager.await_result(task_id).await?;
```

### Audit Logging

操作ログを記録し、暗号化して保存します。

```rust
use cc_core::{AuditLogger, AuditConfig, AuditEntry};

let logger = AuditLogger::new(AuditConfig {
    log_path: "data/audit.log".to_string(),
    encryption_key: Some("secret-key".to_string()),
    ..Default::default()
});

// ログを記録
logger.log(AuditEntry {
    event_type: AuditEventType::ToolExecution,
    source: AuditSource::User("user123".to_string()),
    target: AuditTarget::Tool("bash".to_string()),
    ..Default::default()
}).await?;
```

## モジュール構成

| モジュール | 説明 |
|----------|------|
| `tool` | Tool trait、ToolManager、ToolResult |
| `llm` | ClaudeClient、Message、ToolDefinition |
| `session` | Session、SessionManager、SessionStore |
| `memory` | Memory、MemoryStore |
| `agents` | SubAgent、SubAgentManager、TaskDelegator |
| `audit` | AuditLogger、AuditEntry、EncryptionConfig |
| `skills` | Skill、SkillConfig、SkillLoader |
| `config` | Config、ApiConfig、LlmConfig、McpConfig |
| `error` | Error、Result |

## 使用例

### 基本的なセットアップ

```rust
use cc_core::{ClaudeClient, Config, SessionManager, ToolManager};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 設定をロード
    let config = Config::from_file("cc-gateway.toml")?;

    // LLM クライアントを作成
    let client = ClaudeClient::new(config.llm);

    // セッションマネージャーを作成
    let store = SessionStore::new("data/sessions.db")?;
    let session_manager = SessionManager::new(store);

    // ツールマネージャーを作成
    let mut tool_manager = ToolManager::new();
    cc_tools::register_default_tools(&mut tool_manager);

    Ok(())
}
```

### カスタムツールの実装

```rust
use async_trait::async_trait;
use cc_core::{Tool, ToolResult};
use serde_json::json;

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "My custom tool"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "param": {"type": "string"}
            }
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult> {
        Ok(ToolResult::success("Done!"))
    }
}
```

## 依存クレート

- `tokio`: 非同期ランタイム
- `reqwest`: HTTP クライアント
- `serde`/`serde_json`: シリアライゼーション
- `rusqlite`: SQLite データベース
- `async-trait`: 非同期 trait
- `thiserror`: エラー処理
- `tracing`: ロギング

## ライセンス

MIT
