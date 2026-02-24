# クイックスタート

このガイドでは、cc-gateway を最小限の設定で起動し、最初の対話を試すまでの手順を説明します。

## 事前準備

まだインストールが完了していない場合は、先に「[インストールガイド](installation.md)」を参照してください。

このガイドでは、以下が既に完了していることを前提としています：

- [ ] Rust がインストールされている（バージョン 1.85 以上）
- [ ] cc-gateway のソースコードがクローンされている
- [ ] ビルドが完了している

## ステップ 1：API キーの取得

cc-gateway を使用するには、LLM プロバイダーの API キーが必要です。以下のいずれかから取得してください。

### Anthropic Claude API を使用する場合

1. [Anthropic Console](https://console.anthropic.com/) にアクセス
2. アカウントを作成またはログイン
3. 「API Keys」メニューから「Create Key」をクリック
4. API キーを安全な場所に保存（`sk-ant-...` の形式）

**利用料金について**：Claude API は従量課金です。詳細は [Anthropic の pricing ページ](https://www.anthropic.com/pricing) をご確認ください。

### GLM Coding Plan を使用する場合（オープン互換）

1. [Z.ai](https://z.ai) にアクセス
2. アカウントを作成またはログイン
3. API キーを取得
4. ベース URL を確認（通常：`https://api.z.ai/api/coding/paas/v4`）

### OpenAI API を使用する場合

1. [OpenAI Platform](https://platform.openai.com/) にアクセス
2. アカウントを作成またはログイン
3. 「API Keys」メニューから「Create new secret key」をクリック
4. API キーを安全な場所に保存（`sk-...` の形式）

## ステップ 2：環境変数の設定

最も簡単な方法は、環境変数で API キーを設定することです。

### 一時的に設定する場合（現在のターミナルセッションのみ）

```bash
# Anthropic Claude を使用する場合
export LLM_PROVIDER="claude"
export LLM_API_KEY="sk-ant-your-api-key-here"
export LLM_MODEL="claude-sonnet-4-20250514"

# GLM を使用する場合
export LLM_PROVIDER="openai"
export LLM_API_KEY="your-glm-api-key-here"
export LLM_MODEL="glm-4.7"
export LLM_BASE_URL="https://api.z.ai/api/coding/paas/v4"

# OpenAI を使用する場合
export LLM_PROVIDER="openai"
export LLM_API_KEY="sk-your-openai-api-key-here"
export LLM_MODEL="gpt-4"
export LLM_BASE_URL="https://api.openai.com/v1"
```

### 恒久的に設定する場合（推奨）

プロジェクトルートに `.env` ファイルを作成します：

```bash
# .env ファイルの内容（以下をコピーして API キーを書き換えてください）

# Anthropic Claude の場合
LLM_PROVIDER=claude
LLM_API_KEY=sk-ant-your-api-key-here
LLM_MODEL=claude-sonnet-4-20250514

# GLM の場合（以下の行のコメントを外して使用）
# LLM_PROVIDER=openai
# LLM_API_KEY=your-glm-api-key-here
# LLM_MODEL=glm-4.7
# LLM_BASE_URL=https://api.z.ai/api/coding/paas/v4

# OpenAI の場合（以下の行のコメントを外して使用）
# LLM_PROVIDER=openai
# LLM_API_KEY=sk-your-openai-api-key-here
# LLM_MODEL=gpt-4
# LLM_BASE_URL=https://api.openai.com/v1
```

**重要**：`.env` ファイルは機密情報を含むため、Git にコミットしないよう注意してください。

## ステップ 3：CLI モードで起動

準備ができたら、CLI モード（対話型）で cc-gateway を起動します。

```bash
cargo run -- --cli
```

または、ビルド済みのバイナリを使用する場合：

```bash
./target/release/cc-gateway --cli
```

### 起動成功のサイン

以下のような画面が表示されれば起動成功です：

```
╔════════════════════════════════════════════════════════════╗
║          🤖 cc-gateway CLI - Interactive Mode              ║
╠════════════════════════════════════════════════════════════╣
║  Type your message and press Enter to chat.                ║
║  Commands: /help, /exit, /clear, /history                  ║
╚════════════════════════════════════════════════════════════╝

>
```

## ステップ 4：最初の対話

プロンプト `>` が表示されたら、メッセージを入力してみましょう。

### 基本的な会話

```
> こんにちは
こんにちは！何かお手伝いできることはありますか？
```

### ツールを使用する例

cc-gateway はファイル操作ツールを備えています。試してみましょう：

```
> カレントディレクトリにあるファイルを一覧してください
```

cc-gateway が自動的に `bash` ツールを使用して `ls` コマンドを実行し、結果を返します。

### ファイルを読み込む例

```
> README.md の内容を読んでください
```

`read` ツールが自動的に使用され、ファイル内容が表示されます。

## ステップ 5：CLI コマンドの使用

CLI モードでは、以下のコマンドが使用できます：

| コマンド | 説明 |
|---------|------|
| `/help` または `/?` | ヘルプメッセージを表示 |
| `/exit` または `/quit` | プログラムを終了 |
| `/clear` | 会話履歴をクリア |
| `/history` | 会話履歴を表示 |

### 使用例

```
> /history
[1] User: こんにちは
[1] Assistant: こんにちは！何かお手伝いできることはありますか？
[2] User: カレントディレクトリにあるファイルを一覧してください
[2] Assistant: [Tool execution: bash] ...

> /clear
会話履歴をクリアしました。

> /exit
さようなら！
```

## よくある質問

### Q: API キーを誤って設定している可能性があります

A: 起動時にエラーが表示される場合は、環境変数を確認してください：

```bash
echo $LLM_API_KEY
echo $LLM_PROVIDER
```

正しく設定されていない場合は、再度環境変数を設定するか、`.env` ファイルを確認してください。

### Q: コマンド入力中にキャンセルしたいです

A: `Ctrl + C` を押すと、現在の入力をキャンセルできます。続けてもう一度押すとプログラムが終了します。

### Q: 日本語がうまく表示されません

A: ターミナルの文字エンコーディングが UTF-8 であることを確認してください。macOS や最新の Linux ディストリビューションでは、通常は問題ありません。

## 次のステップ

基本的な使い方が理解できたら、次は以下のドキュメントを参照してください：

- [設定ガイド](configuration.md) - 詳細な設定方法とプロバイダー別の設定例
- [ユーザーガイド - CLI](../user-guide/cli.md) - CLI モードの詳細な使い方
- [ユーザーガイド - API](../user-guide/api.md) - HTTP API の使用方法

## サンプル対話セッション

以下は、実際の対話のサンプルです：

```
> Rust で Hello World を書いてください

```rust
fn main() {
    println!("Hello, world!");
}
```

このプログラムは、文字列 "Hello, world!" を標準出力に出力します。
実行するには、`rustc` コンパイラを使用します：

```bash
rustc hello.rs
./hello
```

> このコードを main.rs として保存してください

[Tool: write]
File: main.rs
Content:
```rust
fn main() {
    println!("Hello, world!");
}
```

ファイルを保存しました：main.rs

> /exit
さようなら！
```
