# Plans.md - cc-gateway 実装計画

> Pure Rust Claude Gateway - OpenClaw代替実装
>
> 作成日: 2026-02-23 | 最終更新: 2026-02-24

---

## 📊 進捗サマリー

| Phase | 状態 | 内容 |
|-------|------|------|
| Phase 1-8 | ✅ 完了 | コア機能（CLI/API/Discord/MCP/スケジューラー） |
| Phase 9 | ⬜ 未着手 | TOML設定ファイル対応 |
| Phase 10 | ⬜ 未着手 | エラーハンドリング改善 |
| Phase 11 | ⬜ 未着手 | CLI非対話モード |
| Phase 12 | ⬜ 未着手 | HTTP API拡張 |
| Phase 13 | ⬜ 未着手 | テスト追加 |

---

## 🎯 優先度マトリックス（改善フェーズ）

| 優先度 | Phase | 内容 | 理由 |
|--------|-------|------|------|
| **必須** | 9 | TOML設定ファイル | 型安全性、ドキュメント性 |
| **必須** | 10 | thiserror導入 | エラー処理の統一 |
| **推奨** | 11 | CLI非対話モード | スクリプト連携 |
| **推奨** | 12 | HTTP API拡張 | セッション管理 |
| **推奨** | 13 | テスト追加 | 品質保証 |

---

## Phase 9: TOML設定ファイル対応 [⬜ 未着手]

### タスク

- [ ] 9.1 `cc-gateway.toml` 形式定義
- [ ] 9.2 `config` crate 統合
- [ ] 9.3 環境変数展開（`${VAR}`）
- [ ] 9.4 .env からの移行
- [ ] 9.5 `cc-gateway.toml.example` 作成

### 設定ファイル仕様

```toml
[llm]
provider = "openai"
model = "glm-4.7"
base_url = "https://api.z.ai/api/coding/paas/v4"
api_key = "${LLM_API_KEY}"

[discord]
token = "${DISCORD_BOT_TOKEN}"
admin_user_ids = [123456789]

[api]
port = 3000

[scheduler]
enabled = true
config_path = "schedule.toml"

[mcp]
enabled = true
config_path = "mcp.json"
```

---

## Phase 10: エラーハンドリング改善 [⬜ 未着手]

### タスク

- [ ] 10.1 `cc-core/src/error.rs` 作成
- [ ] 10.2 thiserror でエラー型定義
- [ ] 10.3 anyhow から移行

### エラー型定義

```rust
#[derive(Error, Debug)]
pub enum CcError {
    #[error("LLM API error: {0}")]
    LlmApi(String),
    #[error("Tool execution error: {0}")]
    ToolExecution(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Configuration error: {0}")]
    Config(String),
}
```

---

## Phase 11: CLI非対話モード [⬜ 未着手]

### タスク

- [ ] 11.1 `--execute "プロンプト"` オプション
- [ ] 11.2 `--file prompt.txt` オプション
- [ ] 11.3 `--session-id ID` オプション
- [ ] 11.4 終了コード設定

### 使用例

```bash
# ワンショット実行
cc-gateway --execute "今日の天気は？"

# ファイルから実行
cc-gateway --file prompt.txt

# セッション継続
cc-gateway --session-id abc123 --cli
```

---

## Phase 12: HTTP API拡張 [⬜ 未着手]

### タスク

- [ ] 12.1 `POST /api/sessions` - セッション作成
- [ ] 12.2 `GET /api/sessions/:id` - セッション取得
- [ ] 12.3 `DELETE /api/sessions/:id` - セッション削除
- [ ] 12.4 `GET /api/tools` - ツール一覧
- [ ] 12.5 `POST /api/tools/:name` - ツール実行
- [ ] 12.6 `GET /api/schedules` - スケジュール一覧

---

## Phase 13: テスト追加 [⬜ 未着手]

### タスク

- [ ] 13.1 ツール単体テスト（bash, read, write）
- [ ] 13.2 セッション永続化テスト
- [ ] 13.3 LLMクライアントモックテスト
- [ ] 13.4 HTTP API統合テスト

---

## 🚀 使用方法（現状）

```bash
# CLI対話モード
cargo run -- --cli

# サーバーモード
cargo run

# ヘルプ
cargo run -- --help
```

---

## 📌 備考

- Rust 2024 Edition (rustc 1.85+)
- 非同期ランタイム: tokio
- HTTP client: reqwest (rustls-tls)
- SQLite: rusqlite (bundled)
