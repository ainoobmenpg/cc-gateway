# CLI モードユーザーガイド

cc-gateway の CLI モードは、ターミナルで直接 AI アシスタントと対話できる対話型 REPL 環境を提供します。

## 起動方法

### 基本的な起動

```bash
# CLI モードで起動
cargo run -- --cli

# またはリリースビルドを使用
./target/release/cc-gateway --cli
```

### 起動時の表示

```
╔════════════════════════════════════════════════════════════╗
║          🤖 cc-gateway CLI - Interactive Mode              ║
╠════════════════════════════════════════════════════════════╣
║  Type your message and press Enter to chat.                ║
║  Commands: /help, /exit, /clear, /history                  ║
╚════════════════════════════════════════════════════════════╝
```

## REPL コマンド一覧

| コマンド | 省略形 | 説明 |
|---------|--------|------|
| `/help` | `/?` | ヘルプメッセージを表示 |
| `/exit` | `/quit` | CLI を終了 |
| `/clear` | - | 会話履歴をクリア |
| `/history` | - | 会話履歴を表示 |

### `/help` - ヘルプ表示

利用可能なコマンドの一覧を表示します。

```
> /help
📖 Available Commands:
  /help, /?     - Show this help message
  /exit, /quit  - Exit the program
  /clear        - Clear conversation history
  /history      - Show conversation history
```

### `/exit` - 終了

CLI モードを終了します。

```
> /exit
👋 Goodbye!
```

### `/clear` - 履歴クリア

現在のセッションの会話履歴をクリアします。新しい会話を始めたい場合に使用します。

```
> /clear
🗑️ Conversation history cleared.
```

### `/history` - 履歴表示

現在のセッションの会話履歴を表示します。

```
> /history
📜 Conversation History:
[1] User: こんにちは
[1] Assistant: こんにちは！お手伝いできることがありましたら、お気軽にお聞きください。
[2] User: 今日の天気は？
[2] Assistant: Web検索ツールを使って今日の天気を調べましょう。
```

## 対話の例

### 基本的な対話

```
> こんにちは
こんにちは！お手伝いできることがありましたら、お気軽にお聞きください。
```

### ツールを使用した対話

```
> カレントディレクトリのファイルを一覧して
ツール: bash
コマンド: ls -la

[実行結果]
total 24
drwxr-xr-x   5 user  staff   160 Feb 24 10:00 .
drwxr-xr-x  10 user  staff   320 Feb 24 10:00 ..
-rw-r--r--   1 user  staff  1024 Feb 24 10:00 README.md
-rw-r--r--   1 user  staff  2048 Feb 24 10:00 main.rs

カレントディレクトリには2つのファイルがあります：
- README.md (1024 bytes)
- main.rs (2048 bytes)
```

### ファイル操作

```
> README.md の内容を読んで
ツール: read
ファイル: README.md

[実行結果]
# cc-gateway

Pure Rust で実装された Claude API Gateway です。
...
```

### Web 検索

```
> Rust の最新バージョンを調べて
ツール: web_search
クエリ: Rust latest version 2025

[実行結果]
Search results for: "Rust latest version 2025"

## [1] Rust 1.85 Release Notes
URL: https://blog.rust-lang.org/2025/02/20/Rust-1.85.html
Published: 2025-02-20

Rust 1.85 がリリースされました。このバージョンでは...
```

## 終了方法

以下のいずれかの方法で終了できます：

1. `/exit` または `/quit` コマンドを入力
2. `Ctrl + D`（EOF）
3. `Ctrl + C`（強制終了）

## 設定

CLI モードは以下の設定ファイルを読み込みます：

- `cc-gateway.toml` - メインの設定ファイル
- `mcp.json` - MCP サーバー設定
- `schedule.toml` - スケジューラー設定

環境変数でも設定できます。詳細はメインの README.md を参照してください。

## ヒント

- **長いプロンプト**: 複数行に渡る入力も可能です
- **エイリアス**: `/?` は `/help` の短縮形です
- **履歴の保存**: セッションはデータベースに保存され、次回起動時に復元されます
- **ツールの使用**: AI が適切なツールを自動的に選択して実行します
