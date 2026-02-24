# Plans.md - cc-gateway 実装計画

> Pure Rust Claude Gateway - OpenClaw代替実装
>
> 作成日: 2026-02-23 | 最終更新: 2026-02-24

---

## 📊 進捗サマリー

| Phase | 状態 | 内容 |
|-------|------|------|
| Phase 1-21 | ✅ 完了 | コア機能 + セキュリティ/安定性/品質/Discord/MCP/マルチチャネル/自動化 |
| Phase 22 | ✅ 完了 | Critical: WebSocket / 画像 / WebSearch / WebFetch / Skills |
| Phase 23 | 🔴 未着手 | High: iMessage / Signal / Slack / Sub-Agents |
| Phase 24 | 🔴 未着手 | Medium: 完全ブラウザ / Voice / Web Dashboard |
| Phase 25 | 🔴 未着手 | Superior: Rust最適化 / バイナリ配布 |

> 📦 過去の完了タスク: `.claude/memory/archive/Plans-2026-02-24.md`
>
> 📊 **OpenClaw パリティ達成率**: ~50% → 詳細は `OPENCLAW_COMPARISON.md`

---

## ✅ 完了フェーズ

> Phase 1-21 の詳細は `.claude/memory/archive/Plans-2026-02-24.md` を参照

| Phase | 内容 |
|-------|------|
| 1-8 | Core Library, Tools, MCP Client, Discord, HTTP API, CLI, Scheduler |
| 9-13 | TOML設定, エラーハンドリング, CLI非対話, HTTP API拡張, テスト追加 |
| 14-16 | セキュリティ, 安定性, 品質改善 |
| 17-18 | (reserved) |
| 19 | MCP統合 (McpRegistry) |
| 20 | マルチチャネル (Telegram, WhatsApp) |
| 21 | 自動化機能 (Browser, Email stubs) |

---

## 📌 技術スタック

- Rust 2024 Edition (rustc 1.85+)
- 非同期ランタイム: tokio
- HTTP client: reqwest (rustls-tls)
- SQLite: rusqlite (bundled)
- Discord: poise 0.6
- Telegram: teloxide 0.13 ✅
- WhatsApp: Twilio API (axum webhook) ✅
- Browser/Email: stub ✅
- **WebSocket: axum + tokio-tungstenite** ✅
- **WebSearch/WebFetch: reqwest + scraper** ✅
- **Skills: config + dynamic loader** ✅

---

## 🔴 Phase 22: Critical Features

> **目標**: 基本的な機能パリティ達成、達成率 ~50%

| タスク | 技術スタック | 内容 |
|--------|------------|------|
| 22.1 WebSocket | axum + tokio-tungstenite | cc-ws crate, WS server, セッション統合, 簡易Web UI |
| 22.2 画像 | base64, image | Claude API画像入力, マルチモーダル処理, 画像生成 |
| 22.3 WebSearch | reqwest + API | Exa/SerpAPI/DuckDuckGo, 結果フィルタリング |
| 22.4 WebFetch | reqwest + scraper | HTMLパース, テキスト抽出, JS レンダリング(オプション) |
| 22.5 Skills | config + loader | 設計, ローダー, カスタムツール登録, 監視 |

---

## 🔴 Phase 23: High Priority

> **目標**: 主要チャネル・機能追加、達成率 ~65%

| タスク | 技術スタック | 内容 |
|--------|------------|------|
| 23.1 iMessage | Apple Script | cc-imessage crate, osascript連携, 送受信 |
| 23.2 Signal | signal-cli REST | cc-signal crate, API連携, 送受信 |
| 23.3 Slack | slack-api/reqwest | cc-slack crate, Events API, Socket Mode |
| 23.4 Sub-Agents | Task delegation | アーキテクチャ, 分散ロジック, 結果集約 |
| 23.5 Thinking | Claude extended | API対応, level設定, 出力処理 |

---

## 🔴 Phase 24: Medium Priority

> **目標**: 完全パリティ、達成率 ~80%

| タスク | 技術スタック | 内容 |
|--------|------------|------|
| 24.1 Browser | headless_chrome/fantoccini | stub→実装, スクショ, フォーム操作 |
| 24.2 Voice | Whisper/TTS API | 音声認識, 音声合成, Twilio Voice |
| 24.3 LINE | LINE Messaging API | cc-line crate, Webhook |
| 24.4 Dashboard | axum + static | UI, セッション履歴, コスト表示 |

---

## 🔴 Phase 25: Superior Features

> **目標**: cc-gateway 独自の優位性確立

| タスク | 内容 |
|--------|------|
| 25.1 Performance | ベンチマーク, メモリ最適化, 並列処理 |
| 25.2 Distribution | cross-compilation, GitHub Releases, Homebrew |
| 25.3 Security | 監査ログ, 暗号化オプション |

---

## 🚀 拡張可能性

| 機能 | 現状 | 拡張方法 |
|------|------|---------|
| ブラウザ自動化 | stub | headless_chrome / fantoccini |
| メール送受信 | stub | lettre / async-imap feature |
| iMessage | 未実装 | Apple Script 連携 |
| Signal | 未実装 | signal-cli 連携 |
| Slack | 未実装 | Slack API |
