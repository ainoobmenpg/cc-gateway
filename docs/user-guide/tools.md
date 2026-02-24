# ツールガイド

cc-gateway には、AI アシスタントが様々なタスクを実行するための組み込みツールが含まれています。

## ツール一覧

| ツール名 | 説明 | 主なパラメータ |
|---------|------|--------------|
| `bash` | シェルコマンドを実行 | `command`, `timeout_ms` |
| `read` | ファイルを読み込む | `path`, `offset`, `limit` |
| `write` | ファイルを書き込む | `path`, `content` |
| `edit` | ファイルを編集（文字列置換） | `path`, `old_string`, `new_string` |
| `glob` | ファイルパターンで検索 | `pattern`, `path` |
| `grep` | ファイル内容を正規表現で検索 | `pattern`, `path`, `glob` |
| `web_search` | Web 検索 | `query`, `limit` |
| `web_fetch` | Web ページを取得 | `url`, `max_chars` |

---

## Bash

シェルコマンドを実行します。git、npm、docker などのコマンドラインツールの操作に使用します。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `command` | string | ✓ | - | 実行するコマンド |
| `timeout_ms` | integer | - | 120000 | タイムアウト（ミリ秒、最大600000） |

### 使用例

```bash
# ディレクトリ一覧
bash("ls -la")

# Git ステータス確認
bash("git status")

# Docker コンテナ一覧
bash("docker ps")

# タイムアウトを指定（30秒）
bash("sleep 60", timeout_ms=30000)  # タイムアウトエラーになる
```

### 実行結果

```json
{
  "stdout": "total 24\ndrwxr-xr-x  5 user  staff  160 Feb 24 10:00 .\n",
  "stderr": "",
  "exit_code": 0,
  "timed_out": false
}
```

---

## Read

ファイルの内容を読み込みます。行範囲を指定して部分的に読むことも可能です。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `path` | string | ✓ | - | ファイルの絶対パス |
| `offset` | integer | - | 1 | 読み込み開始行（1-indexed） |
| `limit` | integer | - | - | 読み込む最大行数 |

### 使用例

```bash
# ファイル全体を読み込み
read("/path/to/file.txt")

# 10行目から読み込み
read("/path/to/file.txt", offset=10)

# 最初の100行のみ読み込み
read("/path/to/file.txt", limit=100)

# 50行目から100行読み込み
read("/path/to/file.txt", offset=50, limit=100)
```

### 実行結果

```
1: # README
2:
3: This is a sample file.
4: It contains multiple lines.
```

---

## Write

ファイルに内容を書き込みます。ファイルが存在しない場合は新規作成、存在する場合は上書きします。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `path` | string | ✓ | - | ファイルの絶対パス |
| `content` | string | ✓ | - | 書き込む内容 |

### 使用例

```bash
# 新規ファイル作成
write("/tmp/hello.txt", "Hello, World!")

# コードを書き込み
write("/src/main.rs", """
fn main() {
    println!("Hello, cc-gateway!");
}
""")
```

### 実行結果

```
Successfully wrote 13 bytes to '/tmp/hello.txt'
```

### 注意点

- **上書き**: 既存のファイルは完全に上書きされます
- **ディレクトリ作成**: 親ディレクトリが存在しない場合は自動作成されます

---

## Edit

ファイル内の文字列を正確に置換します。既存のファイルを安全に編集する場合に使用します。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `path` | string | ✓ | - | ファイルの絶対パス |
| `old_string` | string | ✓ | - | 検索する文字列（一意である必要あり） |
| `new_string` | string | ✓ | - | 置換後の文字列 |
| `replace_all` | boolean | - | false | すべての出現を置換 |

### 使用例

```bash
# 単一の文字列を置換
edit("/src/main.rs",
     old_string="println!(\"Hello\");",
     new_string="println!(\"Hello, cc-gateway!\");")

# すべての出現を置換
edit("/src/config.rs",
     old_string="TODO",
     new_string="FIXME",
     replace_all=true)
```

### 実行結果

```
Successfully replaced 1 occurrence(s) in '/src/main.rs'
```

### 注意点

- **一意性**: `old_string` はファイル内で一意である必要があります
- **複数マッチ**: `replace_all=false` の場合、複数マッチするとエラーになります

---

## Glob

ファイルパターンに一致するファイルを検索します。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `pattern` | string | ✓ | - | グロブパターン（例: `**/*.rs`） |
| `path` | string | - | "." | 検索対象のベースディレクトリ |

### 使用例

```bash
# すべての Rust ファイルを検索
glob("**/*.rs")

# カレントディレクトリの Markdown ファイル
glob("*.md")

# 特定のディレクトリ内の JSON ファイル
glob("**/*.json", path="/config")
```

### 実行結果

```
/src/main.rs
/src/lib.rs
/tests/test.rs
```

### パターン例

| パターン | 説明 |
|---------|------|
| `*.txt` | カレントディレクトリの .txt ファイル |
| `**/*.rs` | サブディレクトリを含むすべての .rs ファイル |
| `src/**/*.rs` | src ディレクトリ以下のすべての .rs ファイル |
| `test_*.rs` | `test_` で始まる .rs ファイル |

---

## Grep

ファイル内容を正規表現で検索します。ripgrep (rg) が使用されます。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `pattern` | string | ✓ | - | 正規表現パターン |
| `path` | string | - | "." | 検索対象のファイル/ディレクトリ |
| `glob` | string | - | - | ファイルパターン（例: `*.rs`） |
| `ignore_case` | boolean | - | false | 大文字小文字を区別しない |

### 使用例

```bash
# 単語を検索
grep("TODO")

# 大文字小文字を区別せずに検索
grep("error", ignore_case=true)

# 特定の拡張子のファイルのみ検索
grep("fn main", glob="*.rs")

# 正規表現で検索
grep(r"(async|await) fn")
```

### 実行結果

```
src/main.rs:10:async fn run() {
src/main.rs:25:    await future;
src/lib.rs:5:async fn process() {
```

---

## WebSearch

Web を検索して情報を取得します。Exa API（推奨）または DuckDuckGo フォールバックを使用します。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `query` | string | ✓ | - | 検索クエリ |
| `limit` | integer | - | 5 | 結果の最大数（最大10） |
| `use_exa` | boolean | - | true | Exa API を使用するか |

### 使用例

```bash
# 基本的な検索
web_search("Rust programming")

# 結果数を指定
web_search("Claude API", limit=3)

# DuckDuckGo を強制使用
web_search("weather Tokyo", use_exa=false)
```

### 実行結果

```
Search results for: "Rust programming"

## [1] The Rust Programming Language
URL: https://www.rust-lang.org/
Published: 2024-12-15

Rust is a systems programming language that runs blazingly fast...
...
Found 5 results.
```

---

## WebFetch

Web ページを取得して解析します。メインコンテンツを抽出し、読みやすい形式で返します。

### パラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|-----------|------|------|-----------|------|
| `url` | string | ✓ | - | 取得する URL |
| `extract_main` | boolean | - | true | メインコンテンツのみ抽出 |
| `include_links` | boolean | - | false | リンクを含める |
| `max_chars` | integer | - | 10000 | 最大文字数（1000-50000） |

### 使用例

```bash
# 基本的な取得
web_fetch("https://example.com")

# リンクを含めて取得
web_fetch("https://example.com", include_links=true)

# 最大文字数を指定
web_fetch("https://example.com", max_chars=5000)

# 全テキストを取得
web_fetch("https://example.com", extract_main=false)
```

### 実行結果

```
Title: Example Domain

This domain is for use in illustrative examples in documents...
```

### 注意点

- **対応 URL**: HTTP/HTTPS のみ対応
- **サイズ制限**: デフォルトで 1MB まで
- **Content-Type**: HTML 以外は生テキストで返されます
