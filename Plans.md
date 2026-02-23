# Plans.md - cc-gateway 実装計画

> Pure Rust Claude Gateway - OpenClaw代替実装
>
> 作成日: 2026-02-23 | 最終更新: 2026-02-24

---

## 📊 進捗サマリー

| Phase | 状態 | 内容 |
|-------|------|------|
| Phase 1-8 | ✅ 完了 | コア機能（CLI/API/Discord/MCP/スケジューラー） |
| Phase 9 | ✅ 完了 | TOML設定ファイル対応 |
| Phase 10 | ✅ 完了 | エラーハンドリング改善（thiserror） |
| Phase 11 | ✅ 完了 | CLI非対話モード |
| Phase 12 | ✅ 完了 | HTTP API拡張 |
| Phase 13 | ✅ 完了 | テスト追加 |
| Phase 14 | ✅ 完了 | セキュリティ修正（認証・CORS） |
| Phase 15 | ✅ 完了 | 安定性修正（MutexGuard・エラー処理） |
| Phase 16 | ✅ 完了 | 品質改善（clippy修正） |

---

## 🎯 優先度マトリックス（レビュー指摘対応）

| 優先度 | Phase | 内容 | 影響 |
|--------|-------|------|------|
| **Required** | 14 | セキュリティ修正 | 認証未適用、CORS 全許可 |
| **Recommended** | 15 | 安定性修正 | デッドロック、パニック |
| **Optional** | 16 | 品質改善 | clippy 警告、テスト不足 |

---

## Phase 14: セキュリティ修正（即時）[feature:security] [✅ 完了]

> コミット: 1e71eb4

### タスク 14.1: 認証ミドルウェアの適用

- [ ] `cc-api/src/server.rs` で `auth_middleware` をルートに適用
- [ ] `/health` エンドポイントは認証除外
- [ ] 設定: `API_KEY` 環境変数で制御
- [ ] テスト: 認証なしで保護エンドポイントにアクセスすると 401

### タスク 14.2: CORS 設定の制限

- [ ] `CorsLayer::permissive()` を制限的設定に変更
- [ ] 設定ファイルから許可オリジンを読み込み（`[api].allowed_origins`）
- [ ] デフォルト: `["http://localhost:*"]` のみ許可
- [ ] テスト: 不正オリジンからのリクエストを拒否

### 検証

```bash
# API_KEY 設定時: 認証なしで 401
curl -X POST http://localhost:3000/api/chat -d '{"message":"test"}'
# 期待: 401 Unauthorized

# /health は認証不要
curl http://localhost:3000/health
# 期待: OK
```

---

## Phase 15: 安定性修正（短期）[✅ 完了]

> レビュー指摘: MAJOR - MutexGuard await 問題、環境変数サイレント失敗、パニック

### タスク 15.1: MutexGuard の await 問題修正

- [ ] `cc-core/session/manager.rs` で MutexGuard を await 前にドロップ
- [ ] パターン: スコープで囲んで早期解放

```rust
// Before (問題あり)
let store = self.store.lock().unwrap();
let session = store.get_latest_by_channel(channel_id)?;
let mut cache = self.cache.write().await; // MutexGuard 保持したまま await

// After (修正済み)
let session = {
    let store = self.store.lock().unwrap();
    store.get_latest_by_channel(channel_id)?
}; // MutexGuard ここで解放
let mut cache = self.cache.write().await; // 安全
```

- [ ] テスト: 並行アクセスでデッドロックしないことを確認

### タスク 15.2: 必須環境変数のバリデーション

- [ ] `cc-core/src/config.rs` で必須環境変数チェック追加
- [ ] `${LLM_API_KEY}` が未設定の場合は **警告ログ + エラー**
- [ ] 設定ファイルに `required_env_vars = ["LLM_API_KEY"]` オプション追加
- [ ] テスト: 必須変数未設定で起動エラーになることを確認

### タスク 15.3: Discord Bot エラーハンドリング

- [ ] `cc-discord/src/bot.rs` の `unwrap()` を適切なエラーハンドリングに変更

```rust
// Before (パニックのリスク)
tokio::spawn(async move {
    store_clone.start_cleanup_task().await.unwrap();
});

// After (安全)
tokio::spawn(async move {
    if let Err(e) = store_clone.start_cleanup_task().await {
        tracing::error!("Cleanup task failed: {}", e);
    }
});
```

- [ ] テスト: クリーンアップタスクでエラーが起きても bot が継続

---

## Phase 16: 品質改善（中期）[✅ 完了]

> レビュー指摘: MINOR - clippy 警告、テストカバレッジ不足

### タスク 16.1: clippy 警告の解消

- [ ] `cc-discord` の `impl Clone` を derive に変更
- [ ] Error enum のサイズ削減（`Box<dyn Error>` 化検討）
- [ ] ドキュメント後の空行削除
- [ ] `cargo clippy --fix` で自動修正
- [ ] CI で `cargo clippy -- -D warnings` を通す

### タスク 16.2: テストカバレッジ向上

#### cc-api ハンドラーテスト

- [ ] `chat` handler テスト（正常系・エラー系）
- [ ] `create_session` / `get_session` / `delete_session` テスト
- [ ] `list_tools` / `execute_tool` テスト

#### cc-discord テスト

- [ ] `/ask` コマンドテスト
- [ ] `/clear` コマンドテスト
- [ ] `/help` コマンドテスト

#### cc-schedule テスト

- [ ] cron パース追加テスト（境界値）
- [ ] スケジュール実行テスト（モック使用）

---

## ✅ 完了フェーズ概要

### Phase 9-13（2026-02-24 完了）

| Phase | 内容 | コミット |
|-------|------|---------|
| 9 | TOML設定ファイル + 環境変数展開 | 5866b1e |
| 10 | thiserror エラー型定義 | 5866b1e |
| 11 | `--execute` / `--file` オプション | 5866b1e |
| 12 | Sessions/Tools/Schedules API | 5866b1e, 7631df3 |
| 13 | 56 テスト追加 | 800819e |

---

## 🚀 実行順序

```
Phase 14 (セキュリティ) → Phase 15 (安定性) → Phase 16 (品質)
```

---

## 📌 備考

- Rust 2024 Edition (rustc 1.85+)
- 非同期ランタイム: tokio
- HTTP client: reqwest (rustls-tls)
- SQLite: rusqlite (bundled)
