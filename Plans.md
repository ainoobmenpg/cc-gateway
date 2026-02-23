# Plans.md - cc-gateway 実装計画

> Pure Rust Claude Gateway - OpenClaw代替実装
>
> 作成日: 2026-02-23 | 最終更新: 2026-02-24

---

## 📊 進捗サマリー

| Phase | 状態 | 内容 |
|-------|------|------|
| Phase 1-16 | ✅ 完了 | コア機能 + セキュリティ/安定性/品質修正 |
| Phase 17 | 🔲 未着手 | Discord Bot改善（poise移行） |
| Phase 18 | 🔲 検討中 | スケジューラー改善 |
| Phase 19 | ✅ 完了 | MCP統合（McpRegistry実装済み） |

> 📦 過去の完了タスク: `.claude/memory/archive/Plans-2026-02-24.md`

---

## 🎯 優先度マトリックス

| 優先度 | Phase | 内容 | 判定 |
|--------|-------|------|------|
| **Recommended** | 17 | Discord Bot poise移行 | 実装推奨 |
| **Optional** | 18 | スケジューラー改善 | 現状維持推奨 |

---

## Phase 17: Discord Bot改善（poise移行）[feature:tdd] [🔲 未着手]

> 外部フィードバック提案: serenity → poise でスラッシュコマンド改善

### 現状分析

| 項目 | 現状 | poise導入後 |
|------|------|------------|
| コマンド登録 | 手動 `CreateCommand` | derive macro自動生成 |
| ハンドラー | `match` で分岐 | 関数属性で自動ルーティング |
| 型安全性 | 低い（文字列ベース） | 高い（型付き引数） |

### タスク

- [ ] 17.1: `cc-discord/Cargo.toml` に `poise = "0.6"` 追加
- [ ] 17.2: `/ask` コマンドを poise で再実装
- [ ] 17.3: `/clear` コマンドを poise で再実装
- [ ] 17.4: `/help` コマンドを poise で再実装
- [ ] 17.5: `handler.rs` を poise Framework に置き換え

### 検証

```bash
cargo test -p cc-discord
```

---

## Phase 18: スケジューラー改善 [🔲 検討中]

> 外部フィードバック提案: tokio-cron-scheduler 導入

### 推奨: **現状維持**

理由:
1. 現在の実装（cron crate）は安定して動作
2. 機能要件を満たしている
3. tokio-cron-scheduler の追加機能が不要
4. 依存関係を増やしたくない

### タスク

- [ ] 18.1: 機能不足がないか最終確認 → 見送り判定予定

---

## 🚀 次のアクション

```bash
# Phase 17 を実装
/work 17

# 全タスク実行
/work all
```

---

## 📌 技術スタック

- Rust 2024 Edition (rustc 1.85+)
- 非同期ランタイム: tokio
- HTTP client: reqwest (rustls-tls)
- SQLite: rusqlite (bundled)
