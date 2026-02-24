# cc-gateway ドキュメント

## cc-gateway とは？

**cc-gateway** は Pure Rust で実装された Claude API Gateway です。Anthropic Claude API と OpenAI 互換 API（GLM Coding Plan など）の両方に対応した高性能ゲートウェイで、OpenClaw の代替として開発されました。

- **マルチプロバイダー対応**: Claude / GLM / OpenAI など、複数の LLM プロバイダーをシームレスに利用可能
- **多彩なインターフェース**: CLI 対話モード、HTTP API、Discord Bot から利用できます
- **拡張可能なアーキテクチャ**: MCP（Model Context Protocol）による外部ツール統合、スケジューラー機能を搭載

## 主な機能

### 基本機能
- [x] **CLI 対話モード** - OpenClaw 風の REPL で直接対話
- [x] **HTTP API** - 認証付き RESTful API サーバー
- [x] **Discord Bot** - スラッシュコマンド対応のフル機能 Discord 連携

### LLM 機能
- [x] **マルチ LLM プロバイダー** - Anthropic Claude API と OpenAI 互換 API（GLM 等）の両方に対応
- [x] **ストリーミング応答** - リアルタイムでの応答表示
- [x] **セッション管理** - SQLite による会話履歴の永続化

### 拡張機能
- [x] **組み込みツール** - bash, read, write, edit, glob, grep
- [x] **MCP 統合** - Model Context Protocol による外部ツール対応
- [x] **スケジューラー** - cron 形式でタスクを定期実行

## 対象ユーザー別ナビゲーション

### 初めて使う方
まずは「[インストールガイド](getting-started/installation.md)」から始めてください。Rust のインストールから cc-gateway のビルドまで、順を追って説明しています。

### さっそく試したい方
Rust が既にインストールされている方は、「[クイックスタート](getting-started/quickstart.md)」を参照して、最小構成で起動してみましょう。

### 詳細設定を知りたい方
「[設定ガイド](getting-started/configuration.md)」で、全設定項目とプロバイダー別の設定例を確認できます。

### 開発者の方
- [開発者ガイド](developer-guide/) - アーキテクチャ、開発環境セットアップ、コードベースの理解
- [リファレンス](reference/) - API 仕様、設定リファレンス

### 運用者の方
- [運用ガイド](operations/) - デプロイ、監視、トラブルシューティング

## 学習パス

```
                    ┌─────────────────────────────────────┐
                    │      cc-gateway ドキュメント          │
                    └─────────────────────────────────────┘
                                       │
                    ┌──────────────────┴──────────────────┐
                    │                                     │
            ┌───────▼────────┐                  ┌────────▼────────┐
            │   初めて使う方   │                  │   Rust既経験者   │
            └───────┬────────┘                  └────────┬────────┘
                    │                                     │
            ┌───────▼─────────────────────────────────────▼────────┐
            │         1. インストールガイド                          │
            │            (getting-started/installation.md)          │
            └───────┬─────────────────────────────────────────────┬─┘
                    │                                             │
            ┌───────▼────────┐                          ┌────────▼────────┐
            │   基本的な使い方   │                          │   詳細設定を知りたい │
            └───────┬────────┘                          └────────┬────────┘
                    │                                             │
    ┌───────────────┼───────────────┐                 ┌───────────▼───────────┐
    │               │               │                 │   2. 設定ガイド        │
┌───▼───────┐ ┌─────▼─────┐ ┌─────▼─────┐           │ (getting-started/     │
│ CLI モード  │ │ HTTP API  │ │ Discord   │           │  configuration.md)   │
│ で使う     │ │ で使う    │ │ Bot で使う│           └───────────┬───────────┘
└───┬───────┘ └─────┬─────┘ └─────┬─────┘                       │
    │               │               │               ┌───────────▼───────────┐
    │               │               │               │   3. ユーザーガイド     │
    └───────────────┴───────────────┴───────────────►  (user-guide/)        │
                                                    └───────────┬───────────┘
                                                                │
                                                    ┌───────────▼───────────┐
                                                    │   4. 開発者ガイド       │
                                                    │  (developer-guide/)    │
                                                    │   やリファレンス        │
                                                    │   (reference/)         │
                                                    └───────────────────────┘
```

## クイックリンク

| 目的 | ドキュメント |
|------|-------------|
| インストール方法 | [インストールガイド](getting-started/installation.md) |
| 最初の対話 | [クイックスタート](getting-started/quickstart.md) |
| 設定ファイル | [設定ガイド](getting-started/configuration.md) |
| CLI コマンド | [ユーザーガイド - CLI](user-guide/cli.md) |
| HTTP API | [ユーザーガイド - API](user-guide/api.md) |
| Discord Bot | [ユーザーガイド - Discord](user-guide/discord.md) |
| 開発環境 | [開発者ガイド](developer-guide/) |
| API 仕様 | [リファレンス](reference/) |

## サポート

- バグ報告・機能リクエスト: [GitHub Issues](https://github.com/ainoobmenpg/cc-gateway/issues)
- ドキュメントの改善: Pull Request をお待ちしています
