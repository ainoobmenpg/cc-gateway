# インストールガイド

このガイドでは、cc-gateway を初めてインストールして実行するまでの手順を説明します。Rust の経験がなくても、このドキュメントに沿って進めればセットアップできます。

## 前提条件

### 必要な環境

cc-gateway を実行するには、以下の環境が必要です：

| 項目 | 必須バージョン | 確認コマンド |
|------|--------------|-------------|
| OS | macOS / Linux / Windows (WSL) | - |
| Rust | 1.85 以上 | `rustc --version` |
| Git | 任意 | `git --version` |

### Rust がインストールされているか確認

ターミナル（macOS/Linux）またはコマンドプロンプト（Windows）で以下を実行してください：

```bash
rustc --version
```

**バージョンが表示される場合**：
```
rustc 1.85.0 (またはそれ以上)
```
Rust は既にインストールされています。次の「[ソースコードの取得](#ソースコードの取得)」に進んでください。

**コマンドが見つからない場合**：
```
command not found: rustc
```
次の「[Rust のインストール](#rust-のインストール)」に進んでください。

## Rust のインストール

Rust は公式のインストールツール「rustup」を使用してインストールします。rustup は Rust のバージョン管理ツールで、複数のバージョンの Rust を切り替えて使用できます。

### macOS / Linux の場合

1. ターミナルを開き、以下のコマンドを実行します：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. インストールが開始されます。画面の指示に従って進めてください：

```
Welcome to Rust!

This will download and install the official compiler for the Rust
programming language, and its package manager, Cargo.

It will add the cargo, rustc, rustup and other commands to
Cargo's bin directory, located at:

  /home/yourusername/.cargo/bin

You can uninstall at any time with rustup self uninstall.

Proceed with installation? (y/N)
```

`y` を入力して Enter を押します。

3. インストールが完了したら、ターミナルを再読み込みします：

```bash
source $HOME/.cargo/env
```

または、新しいターミナルウィンドウを開いても構いません。

4. インストールを確認します：

```bash
rustc --version
cargo --version
```

バージョンが表示されればインストール成功です。

### Windows の場合

1. [rustup.rs のウェブサイト](https://rustup.rs/) にアクセスし、インストーラーをダウンロードします。

2. ダウンロードした `rustup-init.exe` を実行します。

3. インストーラーの指示に従ってインストールを進めます。

4. インストール完了後、新しいコマンドプロンプトまたは PowerShell を開き、以下で確認します：

```powershell
rustc --version
cargo --version
```

## ソースコードの取得

cc-gateway のソースコードを GitHub からクローンします。

### クローン

```bash
# プロジェクトディレクトリに移動（任意の場所で構いません）
cd ~/GitHub

# リポジトリをクローン
git clone https://github.com/ainoobmenpg/cc-gateway.git

# 作成されたディレクトリに移動
cd cc-gateway
```

クローンが完了すると、以下のようなファイル・ディレクトリ構成になっています：

```
cc-gateway/
├── Cargo.toml              # ワークスペース設定
├── cc-gateway.toml.example # 設定ファイルのテンプレート
├── README.md               # プロジェクトの説明
├── CLAUDE.md               # 開発者向けガイド
├── crates/                 # クレート（パッケージ）のディレクトリ
│   ├── cc-core/           # コアライブラリ
│   ├── cc-tools/          # 組み込みツール
│   ├── cc-mcp/            # MCP 統合
│   ├── cc-discord/        # Discord Bot
│   ├── cc-api/            # HTTP API
│   └── cc-gateway/        # メインバイナリ
└── docs/                  # ドキュメント
```

## ビルド

Rust のプロジェクトは `cargo` コマンドを使ってビルドします。

### 依存関係の確認

cc-gateway は SQLite を使用しますが、SQLite ライブラリはバイナリにバンドルされているため、追加のシステム依存関係はありません。

### ビルド実行

プロジェクトのルートディレクトリ（`cc-gateway`）で以下を実行します：

```bash
cargo build --release
```

**オプションの説明**：
- `--release`：最適化された本番用バイナリをビルドします。これを指定しないとデバッグビルドになり、実行速度が遅くなります。

### ビルド時間について

初回ビルドは依存関係のダウンロードとコンパイルがあるため、数分〜十数分かかることがあります。

```
Compiling cc-core v0.1.0 (/path/to/cc-gateway/crates/cc-core)
Compiling cc-tools v0.1.0 (/path/to/cc-gateway/crates/cc-tools)
Compiling cc-gateway v0.1.0 (/path/to/cc-gateway/crates/cc-gateway)
    Finished `release` profile [optimized] target(s) in 2m 45s
```

`Finished` と表示されればビルド成功です。

## ビルドエラーの対処法

### エラー: rustc バージョンが古い

```
error: package `cc-gateway v0.1.0` cannot be built because it requires rustc 1.85 or newer
```

**対処法**：Rust を更新してください

```bash
rustup update stable
```

### エラー: リンクエラー（macOS）

```
error: linking with `cc` failed
```

**対処法**：Xcode コマンドラインツールをインストールしてください

```bash
xcode-select --install
```

### エラー: メモリ不足

```
error: could not compile cc-gateway
error: aborting due to previous error
```

**対処法**：ビルド時にメモリ使用量を抑えるパラレルジョブ数を指定してください

```bash
cargo build --release -j 2
```

### エラー: ネットワーク関連

依存関係のダウンロードでエラーが発生する場合：

```
error: failed to download ...
```

**対処法**：
1. インターネット接続を確認してください
2. プロキシ設定が必要な場合は、以下の環境変数を設定してください

```bash
export HTTP_PROXY=http://your-proxy:port
export HTTPS_PROXY=http://your-proxy:port
```

## 動作確認

ビルドが成功したら、正常に動作するか確認します。

### ヘルプ表示

```bash
./target/release/cc-gateway --help
```

以下のような出力が表示されれば成功です：

```
cc-gateway 0.1.0
Pure Rust Claude API Gateway

USAGE:
    cc-gateway [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
    -c, --config <CONFIG>    Config file path [default: cc-gateway.toml]
    -v, --verbose    Increase verbosity

SUBCOMMANDS:
    cli              Start interactive CLI mode
```

### バージョン確認

```bash
./target/release/cc-gateway --version
```

```
cc-gateway 0.1.0
```

## 次のステップ

インストールが完了したら、次は「[クイックスタート](quickstart.md)」を参考にして、最初の対話を試してみましょう。

## トラブルシューティング

### Q: ビルド後に実行すると「コマンドが見つかりません」と言われます

A: 実行パスが正しいか確認してください。プロジェクトのルートディレクトリから実行している場合は：

```bash
# 正しい例
./target/release/cc-gateway --help

# 間違った例
target/release/cc-gateway --help  # 先頭の ./ が必要
```

### Q: 毎回 ./target/release/cc-gateway と入力するのは面倒です

A: 以下のいずれかの方法で便利に使えます：

**方法1：グローバルインストール**

```bash
cargo install --path .
```

これでどこからでも `cc-gateway` コマンドで実行できます。

**方法2：エイリアス設定**

`~/.bashrc` または `~/.zshrc` に以下を追加：

```bash
alias cc-gateway="$HOME/GitHub/cc-gateway/target/release/cc-gateway"
```

### Q: Windows で実行できません

A: Windows の場合は以下を試してください：

```powershell
.\target\release\cc-gateway.exe --help
```

それでも動かない場合は、WSL（Windows Subsystem for Linux）の使用を推奨します。
