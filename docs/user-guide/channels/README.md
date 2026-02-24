# チャネルガイド

cc-gateway は複数のチャネル（通信手段）を通じて AI アシスタントと対話できます。

## チャネルとは

チャネルは、あなたと cc-gateway の間の通信手段を定義するものです。各チャネルは異なるインターフェースと機能を提供します。

## 利用可能なチャネル一覧

| チャネル | ステータス | 説明 |
|---------|----------|------|
| **CLI** | ✓ 実装済み | ターミナルでの対話型 REPL |
| **Discord** | ✓ 実装済み | Discord Bot による対話 |
| **HTTP API** | ✓ 実装済み | RESTful API サーバー |
| **WebSocket** | 🚧 計画中 | リアルタイム双方向通信 |
| **Telegram** | 🚧 計画中 | Telegram Bot による対話 |
| **Slack** | 🚧 計画中 | Slack アプリとの連携 |
| **Voice** | 🚧 計画中 | 音声対話インターフェース |
| **Browser** | 🚧 計画中 | Web ブラウザ操作 |
| **Dashboard** | 🚧 計画中 | Web 管理コンソール |

---

## 実装済みチャネル

### CLI モード

ターミナルで直接対話できる最もシンプルなチャネルです。

```bash
cargo run -- --cli
```

- **特徴**: シンプル、高速、ツールの完全なアクセス
- **適した用途**: 開発、デバッグ、ローカルでの使用
- **詳細**: [cli.md](../cli.md)

### Discord Bot

Discord サーバーから AI アシスタントと対話できます。

```toml
[discord]
token = "${DISCORD_BOT_TOKEN}"
admin_user_ids = [123456789]
```

- **特徴**: スラッシュコマンド、セッション管理、マルチユーザー対応
- **適した用途**: チームでの共有、リモートアクセス
- **詳細**: [discord.md](./discord.md)

### HTTP API

RESTful API を通じてプログラム的にアクセスできます。

```bash
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!"}'
```

- **特徴**: RESTful、認証対応、軽量
- **適した用途**: アプリケーション連携、自動化

---

## 計画中のチャネル

### WebSocket

リアルタイム双方向通信を実現します。ブラウザやネイティブアプリからのストリーミング応答が可能になります。

### Telegram

Telegram Bot と連携し、Telegram アプリから対話できます。

### Slack

Slack ワークスペースと統合し、チームのコラボレーションフローに組み込めます。

### LINE / WhatsApp / Signal / iMessage

一般的なメッセージングアプリとの連携を予定しています。

### Voice

音声対話インターフェースを提供し、話しかけて操作できるようになります。

### Browser

AI が Web ブラウザを操作できるようになり、自動化タスクを実行できます。

### Dashboard

Web ベースの管理コンソールで、視覚的に設定・管理できるようになります。

---

## 共通の設定パターン

すべてのチャネルは以下の共通設定を使用します：

### LLM 設定

```toml
[llm]
provider = "openai"  # または "claude"
model = "glm-4.7"
api_key = "${LLM_API_KEY}"
```

### ツール設定

組み込みツールはすべてのチャネルで利用可能です：

- Bash - コマンド実行
- Read/Write/Edit - ファイル操作
- Glob/Grep - ファイル検索
- WebSearch/WebFetch - Web アクセス

### MCP 統合

外部 MCP サーバーはすべてのチャネルで共通して使用できます：

```json
{
  "servers": [
    {
      "name": "git",
      "command": "uvx mcp-server-git",
      "enabled": true
    }
  ]
}
```

### スケジューラー

定期タスクはチャネルに依存せず実行されますが、結果を特定のチャネルに送信できます：

```toml
[[schedules]]
name = "日次レポート"
cron = "0 18 * * *"
discord_channel = "reports"  # Discord に結果を送信
```

---

## チャネルの選択ガイド

| ユースケース | 推奨チャネル |
|-------------|-------------|
| 開発・デバッグ | CLI |
| チームでの共有 | Discord |
| アプリケーション連携 | HTTP API |
| 自動化スクリプト | HTTP API |
| モバイルからのアクセス | Discord / HTTP API |
| リアルタイム監視 | WebSocket (計画中) |
| 音声操作 | Voice (計画中) |

---

## マルチチャネル運用

複数のチャネルを同時に使用できます：

```bash
# HTTP API + Discord Bot を同時に起動
cargo run
```

各チャネルは独立したセッションを持ち、異なるコンテキストで対話できます。
