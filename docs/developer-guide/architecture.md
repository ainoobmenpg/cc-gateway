# cc-gateway アーキテクチャ

このドキュメントでは cc-gateway の全体アーキテクチャについて説明します。

## 全体構成図

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              cc-gateway System                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌────────────────┐      ┌─────────────────────────────────────────────────┐  │
│  │   入力チャネル  │──────▶│                     ToolManager                 │  │
│  │                │      │  ┌───────┐ ┌───────┐ ┌───────┐ ┌─────────────┐  │  │
│  │ • CLI (REPL)   │      │  │ Bash  │ │ Read  │ │ Write │ │  MCP Tools  │  │  │
│  │ • HTTP API     │      │  └───────┘ └───────┘ └───────┘ └─────────────┘  │  │
│  │ • Discord Bot  │      │  ┌───────┐ ┌───────┐ ┌───────┐ ┌─────────────┐  │  │
│  │ • WebSocket    │      │  │ Edit  │ │ Glob  │ │ Grep  │ │ Custom Tools│  │  │
│  │ • Telegram     │      │  └───────┘ └───────┘ └───────┘ └─────────────┘  │  │
│  │ • LINE         │      └─────────────────────────────────────────────────┘  │
│  │ • Slack        │                                                        │
│  └────────────────┘                                                        │
│                                  │                                           │
│                                  ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                          Agent Loop (llm)                                │  │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Message → LLM → tool_use → ToolManager → ToolResult → Response  │  │  │
│  │  └──────────────────────────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                  │                                           │
│                                  ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                       永続化レイヤー                                       │  │
│  │  • SessionManager (SQLite)  • MemoryStore (SQLite)  • AuditLogger        │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## ワークスペース構成

cc-gateway は 17 個の crate で構成される Rust ワークスペースです。

```
cc-gateway/
├── crates/
│   ├── cc-core/         ★ コアライブラリ
│   ├── cc-tools/        ★ 組み込みツール
│   ├── cc-mcp/          ★ MCP クライアント統合
│   ├── cc-schedule/     タスクスケジューラー
│   ├── cc-api/          HTTP API サーバー
│   ├── cc-ws/           WebSocket ゲートウェイ
│   ├── cc-discord/      Discord Bot
│   ├── cc-telegram/     Telegram Bot
│   ├── cc-whatsapp/     WhatsApp Bot
│   ├── cc-line/         LINE Bot
│   ├── cc-slack/        Slack Bot
│   ├── cc-imessage/     iMessage ゲートウェイ (macOS のみ)
│   ├── cc-signal/       Signal Bot
│   ├── cc-browser/      ブラウザ操作ツール
│   ├── cc-voice/        音声認識/合成 (Whisper, TTS)
│   ├── cc-dashboard/    Web ダッシュボード
│   ├── cc-email/        Email ゲートウェイ
│   └── cc-gateway/      ★ メインバイナリ
├── docs/
├── Cargo.toml
└── README.md
```

### Crate 詳細

| Crate | 役割 | 主要機能 |
|-------|------|---------|
| **cc-core** | コアライブラリ | Tool trait, LLM クライアント, Session, Memory, Agents, Audit |
| **cc-tools** | 組み込みツール | Bash, Read, Write, Edit, Glob, Grep, WebSearch, WebFetch |
| **cc-mcp** | MCP 統合 | MCP サーバーとの通信, McpToolAdapter |
| **cc-schedule** | スケジューラー | cron 形式のタスク定期実行 |
| **cc-api** | HTTP サーバー | RESTful API, axum ベース |
| **cc-ws** | WebSocket | WebSocket ゲートウェイ |
| **cc-discord** | Discord | serenity/poise ベースの Discord Bot |
| **cc-telegram** | Telegram | Teloxide ベースの Telegram Bot |
| **cc-whatsapp** | WhatsApp | WhatsApp Business API |
| **cc-line** | LINE | LINE Messaging API |
| **cc-slack** | Slack | Slack API |
| **cc-imessage** | iMessage | AppleScript による iMessage 送信 (macOS) |
| **cc-signal** | Signal | Signal Protocol |
| **cc-browser** | ブラウザ | Playwright によるブラウザ操作 |
| **cc-voice** | 音声 | Whisper (認識), TTS (合成) |
| **cc-dashboard** | ダッシュボード | Web UI |
| **cc-email** | Email | SMTP/IMAP |
| **cc-gateway** | バイナリ | メインエントリーポイント |

## データフロー

### 1. 基本的なリクエストフロー

```
┌──────────┐     ┌─────────────┐     ┌──────────────┐     ┌──────────┐
│  ユーザー  │────▶│ 入力チャネル  │────▶│ ClaudeClient │────▶│   LLM    │
│  入力     │     │ (CLI/API等)  │     │              │     │  (API)   │
└──────────┘     └─────────────┘     └──────────────┘     └────┬─────┘
                                                                   │
                              tool_use レスポンス                    │
                                                                   │
                              ┌──────────────┐                      │
                              │ ToolManager   │◀─────────────────────┘
                              │              │
                              │  • ツール選択  │
                              │  • 実行      │
                              │  • 結果集約  │
                              └──────┬───────┘
                                     │
                         ┌───────────┴───────────┐
                         │                       │
                         ▼                       ▼
                  ┌─────────────┐         ┌─────────────┐
                  │ 組み込みツール │         │  MCP ツール   │
                  │ (Bash等)    │         │  (外部)      │
                  └─────────────┘         └─────────────┘
                         │                       │
                         └───────────┬───────────┘
                                     │
                              ToolResult
                                     │
                              ┌──────▼──────┐
                              │ClaudeClient │
                              │ (結果を送信) │
                              └──────┬──────┘
                                     │
                              ┌──────▼──────┐
                              │ ユーザー応答  │
                              └─────────────┘
```

### 2. セッション管理フロー

```
┌──────────────┐     ┌────────────────┐     ┌──────────────┐
│ 新規リクエスト  │────▶│ SessionManager │────▶│  SessionStore │
│              │     │                │     │   (SQLite)   │
└──────────────┘     └────────────────┘     └──────────────┘
                            │
                            ▼
                   ┌────────────────┐
                   │ セッションID発行 │
                   │ または既存セッション│
                   │    の復元       │
                   └────────────────┘
```

### 3. MCP 統合フロー

```
┌──────────────┐     ┌────────────────┐     ┌──────────────┐
│  ToolManager  │────▶│ McpToolAdapter │────▶│  McpClient   │
│              │     │                │     │              │
└──────────────┘     └────────────────┘     └──────┬───────┘
                                                   │
                                                   ▼
                                          ┌────────────────┐
                                          │  MCP サーバー   │
                                          │  (子プロセス)   │
                                          └────────────────┘
```

## 主要コンポーネント

### cc-core

コア機能を提供するライブラリ crate です。

#### モジュール構成

| モジュール | 説明 |
|----------|------|
| `tool` | Tool trait と ToolManager |
| `llm` | Claude API クライアントと Agent Loop |
| `session` | セッション管理 (SQLite 永続化) |
| `memory` | メモリシステム (SQLite) |
| `agents` | サブエージェント機能 |
| `audit` | 監査ログと暗号化 |
| `skills` | スキルシステム |
| `config` | 設定管理 |
| `error` | エラー型定義 |

#### Tool trait

```rust
#[async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> JsonValue;
    async fn execute(&self, input: JsonValue) -> Result<ToolResult>;
}
```

### cc-tools

組み込みツールを提供します。

| ツール | 機能 |
|-------|------|
| `BashTool` | シェルコマンド実行 |
| `ReadTool` | ファイル読み込み |
| `WriteTool` | ファイル書き込み |
| `EditTool` | ファイル編集 |
| `GlobTool` | ファイルパターンマッチング |
| `GrepTool` | ファイル内容検索 |
| `WebSearchTool` | Web 検索 |
| `WebFetchTool` | Web ページ取得 |

### cc-mcp

Model Context Protocol (MCP) 統合を提供します。

#### 主要コンポーネント

- `McpClient`: MCP サーバーとの通信
- `McpToolAdapter`: MCP ツールを cc-core Tool trait に適合
- `McpRegistry`: MCP サーバーの登録と管理

## 依存関係図

```
                    ┌─────────────────┐
                    │   cc-gateway    │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   cc-api      │    │   cc-discord  │    │   cc-tools    │
└───────┬───────┘    └───────┬───────┘    └───────┬───────┘
        │                    │                    │
        └────────────────────┼────────────────────┤
                             │                    │
                             ▼                    ▼
                    ┌──────────────────────────────┐
                    │           cc-core            │
                    │  (Tool, LLM, Session, etc.)  │
                    └──────────────────────────────┘
                             │
                             ▼
                    ┌──────────────────────────────┐
                    │         tokio / reqwest      │
                    │      (async runtime/HTTP)    │
                    └──────────────────────────────┘
```

## 非同期処理モデル

cc-gateway は `tokio` を非同期ランタイムとして使用します。

- **Executor**: Multi-threaded executor
- **I/O**: 非同期 I/O (tokio::net, reqwest)
- **並列処理**: `tokio::spawn` によるタスク並列化

## 設定の優先順位

1. 環境変数 (最優先)
2. `cc-gateway.toml` 設定ファイル
3. デフォルト値

## エラー処理

- `cc-core::Error`: 共通エラー型
- `thiserror` によるエラー定義
- エラーは `Result<T>` で伝搬
