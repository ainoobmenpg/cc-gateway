# Plans.md - cc-gateway 実装計画

> Pure Rust Claude Gateway - OpenClaw代替実装
>
> 作成日: 2026-02-23 | 最終更新: 2026-02-24

---

## 📊 進捗サマリー

| Phase | 状態 | 進捗 |
|-------|------|------|
| Phase 1: Core Library | ✅ 完了 | 100% |
| Phase 2: Tools | ✅ 完了 | 100% |
| Phase 3: MCP統合 | ✅ 完了 | 100% |
| Phase 4: Discord Gateway | ✅ 完了 | 100% |
| Phase 5: HTTP API | ✅ 完了 | 100% |
| Phase 6: Main Binary | ✅ 完了 | 100% |
| Phase 7: CLI対話モード | ✅ 完了 | 100% |
| Phase 8: スケジューラー | ✅ 完了 | 100% |

**🎉 全Phase完了！**

---

## Phase 1: Core Library [✅ 完了]

- [x] Tool System (trait, manager, definition)
- [x] Session Management (manager, SQLite store)
- [x] Memory System (store, types)
- [x] Claude Client (HTTP client, types, agent loop)
- [x] Config & Error handling

---

## Phase 2: Built-in Tools [✅ 完了]

- [x] bash.rs - コマンド実行
- [x] read.rs - ファイル読み込み
- [x] write.rs - ファイル書き込み
- [x] edit.rs - ファイル編集
- [x] glob.rs - ファイルパターン検索
- [x] grep.rs - 内容検索

---

## Phase 3: MCP Integration [✅ 完了]

- [x] client.rs - rmcp統合
- [x] adapter.rs - Tool traitアダプター
- [x] config.rs - MCP設定読み込み
- [x] registry.rs - McpRegistry実装
- [x] main.rs - MCP初期化統合

---

## Phase 4: Discord Gateway [✅ 完了]

- [x] bot.rs - Serenity Bot
- [x] handler.rs - イベントハンドラー
- [x] session.rs - インメモリセッション
- [x] commands/ - /ask, /clear, /help コマンド

---

## Phase 5: HTTP API [✅ 完了]

- [x] server.rs - axum サーバー
- [x] routes.rs - ルート定義
- [x] handlers.rs - ハンドラー
- [x] middleware/auth.rs - 認証
- [x] middleware/rate_limit.rs - レートリミット

---

## Phase 6: Main Binary [✅ 完了]

- [x] main.rs - エントリーポイント
- [x] Discord Bot統合
- [x] HTTP API統合
- [x] MCP統合
- [x] GLM Coding Plan対応

---

## Phase 7: CLI対話モード [✅ 完了]

### 7.1 CLI引数処理 [✅] 完了

- [x] `std::env::args` で `--cli` フラグ処理
- [x] `--help`, `--version` オプション

### 7.2 REPL実装 [✅] 完了

- [x] `crates/cc-gateway/src/cli.rs` 作成
- [x] ユーザー入力ループ (stdin)
- [x] 出力フォーマット
- [x] スペシャルコマンド (`/exit`, `/clear`, `/help`, `/history`)

### 7.3 Agent Loop統合 [✅] 完了

- [x] ツール実行対応
- [x] ツール実行ログ表示

### 7.4 セッション管理 [✅] 完了

- [x] 会話履歴の保持
- [x] `/history` コマンド

---

## Phase 8: スケジューラー [✅ 完了]

### 8.1 スケジュール Crate [✅] 完了

- [x] `crates/cc-schedule/` 作成
- [x] `config.rs` - TOML 設定読み込み
- [x] `scheduler.rs` - cron ベースのタスク実行

### 8.2 設定ファイル [✅] 完了

- [x] `schedule.toml` 形式定義
- [x] `schedule.toml.example` サンプル作成

### 8.3 main.rs 統合 [✅] 完了

- [x] スケジューラー初期化
- [x] グレースフルシャットダウン

---

## 🎯 優先度マトリックス (Phase 7) - 完了

| 優先度 | タスク | 状態 |
|-------|--------|------|
| **必須** | 7.1 CLI引数 | ✅ |
| **必須** | 7.2 REPL | ✅ |
| **必須** | 7.3 Agent Loop統合 | ✅ |
| **推奨** | 7.4 セッション管理 | ✅ |

---

## 🔧 環境変数

```bash
# LLM設定 (必須)
LLM_API_KEY=your-api-key
LLM_MODEL=glm-4.7
LLM_PROVIDER=openai  # claude or openai
LLM_BASE_URL=https://api.z.ai/api/coding/paas/v4

# 旧形式 (後方互換)
CLAUDE_API_KEY=sk-ant-...
CLAUDE_MODEL=claude-sonnet-4-20250514

# Discord Bot (オプション)
DISCORD_BOT_TOKEN=...
ADMIN_USER_IDS=...

# HTTP API (オプション)
API_KEY=...  # HTTP API認証
API_PORT=3000

# データベース
DB_PATH=data/cc-gateway.db

# MCP統合
MCP_CONFIG_PATH=mcp.json
MCP_ENABLED=true

# スケジューラー (オプション)
SCHEDULE_ENABLED=true
SCHEDULE_CONFIG_PATH=schedule.toml
```

## 🚀 使用方法

```bash
# CLI対話モード (OpenClaw風)
cargo run -- --cli

# サーバーモード (HTTP API + Discord Bot + スケジューラー)
cargo run

# ヘルプ
cargo run -- --help
```

## 📅 スケジュール設定

`schedule.toml` で定期実行タスクを設定：

```toml
# 毎朝の挨拶
[[schedules]]
name = "毎朝の挨拶"
cron = "0 9 * * *"        # 毎日 9:00
prompt = "おはようございます。今日の予定を教えてください。"
enabled = true

# 日次レポート
[[schedules]]
name = "日次レポート"
cron = "0 18 * * *"       # 毎日 18:00
prompt = "今日の作業ログをまとめてください。"
tools = ["read", "glob"]  # 使用ツールを制限
discord_channel = "reports"  # Discord に投稿
enabled = true
```

cron 形式: `分 時 日 月 曜日`

---

## 📌 備考

- Rust 2024 Edition (rustc 1.85+)
- SQLite はbundled feature使用
- 非同期ランタイム: tokio
- HTTP client: reqwest (rustls-tls)
