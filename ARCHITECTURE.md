# 計画: cc-gateway (Pure Rust) 実装

## Context

ユーザーは **OpenClawの機能をClaude Codeで置き換え**、新規リポジトリ **cc-gateway** をPure Rustで開発したい。

### 要件
- **リポジトリ名**: `cc-gateway`
- **場所**: `~/GitHub/cc-gateway/`
- **技術スタック**: Rust 2024 Edition のみ（TypeScript/Node.js不使用）
- **目的**: Claude APIを直接使用し、OpenClaw相当の機能をRustで実現

### 技術的実現可能性: **高い**
- Claude Agent SDKはTypeScript専用だが、**HTTP REST APIを直接叩けばRustで実装可能**
- 既存cc-discord-botのパターンを再利用

---

## アーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│                       Discord                           │
└────────────────────────┬────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│                    cc-gateway                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │  cc-discord (Serenity)  │  cc-api (axum)        │   │
│  │  - Slash Commands       │  - REST API           │   │
│  │  - Message Handler      │  - 認証/レートリミット │   │
│  └────────────┬────────────┴───────────────┬───────┘   │
│               │                            │           │
│               └──────────┬─────────────────┘           │
│                          ↓                             │
│  ┌───────────────────────────────────────────────────┐ │
│  │                 cc-core                           │ │
│  │  ┌─────────────┐  ┌──────────┐  ┌─────────────┐  │ │
│  │  │ Claude API  │  │  Agent   │  │   Session   │  │ │
│  │  │ HTTP Client │  │  Loop    │  │   Manager   │  │ │
│  │  └──────┬──────┘  └────┬─────┘  └─────────────┘  │ │
│  │         │              │                         │ │
│  │  ┌──────┴──────────────┴──────────────────┐     │ │
│  │  │            Tool System                  │     │ │
│  │  │  Bash | Read | Write | Edit | Glob ... │     │ │
│  │  └────────────────────────────────────────┘     │ │
│  └───────────────────────────────────────────────────┘ │
│                          │                             │
│  ┌───────────────────────┴───────────────────────┐    │
│  │  cc-mcp (rmcp)  │  Memory (SQLite)  │  Schedule │   │
│  └───────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

---

## プロジェクト構造

```
cc-gateway/
├── Cargo.toml                    # ワークスペース
├── crates/
│   ├── cc-core/                  # コアライブラリ
│   │   └── src/
│   │       ├── llm/              # Claude API クライアント
│   │       ├── tool/             # Tool trait & Manager
│   │       ├── session/          # セッション管理
│   │       └── memory/           # メモリシステム
│   │
│   ├── cc-tools/                 # 組み込みツール
│   │   └── src/
│   │       ├── bash.rs           # コマンド実行
│   │       ├── read_file.rs
│   │       ├── write_file.rs
│   │       ├── edit.rs
│   │       ├── glob.rs
│   │       └── grep.rs
│   │
│   ├── cc-mcp/                   # MCPクライアント
│   ├── cc-discord/               # Discord Gateway
│   └── cc-api/                   # HTTP API
└── data/                         # SQLite データ
```

---

## Claude API 実装（核心）

### HTTP直接呼び出し
```rust
// Claude APIエンドポイント
POST https://api.anthropic.com/v1/messages

// ヘッダー
x-api-key: {CLAUDE_API_KEY}
anthropic-version: 2023-06-01
```

### エージェントループ
```
1. ユーザーメッセージ受信
2. Claude APIにリクエスト送信
3. stop_reason で分岐:
   - "end_turn" → 応答を返す
   - "tool_use" → ツール実行 → 結果を送信 → 2に戻る
```

### Tool Useフロー
```json
// Claude応答 (tool_use)
{
  "content": [{"type": "tool_use", "name": "read_file", "input": {...}}],
  "stop_reason": "tool_use"
}

// ツール実行後の送信
{
  "messages": [
    {"role": "user", "content": "Read file"},
    {"role": "assistant", "content": [tool_use]},
    {"role": "user", "content": [{"type": "tool_result", "content": "..."}]}
  ]
}
```

---

## 実装フェーズ

### Phase 1: コアライブラリ (Week 1-2)
- [ ] Claude API HTTPクライアント実装
- [ ] エージェントループ実装
- [ ] Tool trait & ToolManager
- [ ] セッション管理（SQLite永続化）
- [ ] メモリシステム

### Phase 2: ツールシステム (Week 2-3)
- [ ] Bashツール
- [ ] Read/Write/Editファイル
- [ ] Glob/Grep
- [ ] WebFetch
- [ ] Remember/Recall

### Phase 3: MCP統合 (Week 3)
- [ ] rmcpクライアント統合
- [ ] Tool traitアダプター

### Phase 4: Discord Gateway (Week 3-4)
- [ ] Serenityイベントハンドラー
- [ ] Slash Commands (/ask, /memory, /schedule)
- [ ] メッセージ監視モード

### Phase 5: HTTP API (Week 4)
- [ ] axum REST API
- [ ] 認証ミドルウェア
- [ ] レートリミット

---

## 主要Crate

```toml
[workspace.dependencies]
tokio = "1"
reqwest = { version = "0.12", features = ["json", "stream"] }
axum = "0.8"
serenity = "0.12"
rusqlite = "0.32"
rmcp = "0.16"
cron = "0.15"
serde = "1"
serde_json = "1"
```

---

## 既存コードからの再利用

| cc-discord-bot | cc-gateway |
|----------------|------------|
| `src/tool.rs` | Tool trait設計 |
| `src/session.rs` | セッション管理パターン |
| `src/api.rs` | axum API構造 |
| `src/mcp_client.rs` | MCPクライアント |
| `src/tools/*.rs` | ツール実装 |

---

## 環境変数

```bash
CLAUDE_API_KEY=sk-ant-...
CLAUDE_MODEL=claude-sonnet-4-20250514
DISCORD_BOT_TOKEN=...
ADMIN_USER_IDS=...
API_KEY=...  # HTTP API認証
API_PORT=3000
```

---

## 検証方法

1. **Phase 1完了**: 単体テストでClaude API呼び出し確認
2. **Phase 4完了**: Discordから`/ask`で応答確認
3. **統合テスト**: ファイル操作、ツール実行確認
4. **最終**: cc-discord-botと同等機能の動作確認

---

## 次のアクション

1. `~/GitHub/cc-gateway/` ディレクトリ作成
2. Cargo.toml ワークスペース初期化
3. cc-core crate 作成
4. Claude APIクライアント実装開始
