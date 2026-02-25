# Signal チャネルガイド

Signal を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | signal-cli REST API |
| crate | cc-signal |
| ステータス | ✅ 実装済み |

## 設定

```toml
[signal]
phone_number = "${SIGNAL_PHONE_NUMBER}"
api_token = "${SIGNAL_API_TOKEN}"
signal_cli_path = "signal-cli"
```

### 環境変数

```bash
SIGNAL_PHONE_NUMBER=+1234567890
SIGNAL_API_TOKEN=...
```

## 前提条件

1. signal-cli をインストール: `brew install signal-cli` または `npm install -g signal-cli`
2. 信号番号でアカウントを登録
3. REST API モードで起動

## 使用方法

```bash
# signal-cli REST API 起動
signal-cli rest-api -p 8080
```

```toml
[signal]
phone_number = "+1234567890"
api_token = "your-auth-token"
rest_api_url = "http://localhost:8080"
```

## 機能

- テキストメッセージの送受信
- グループメッセージ対応
- 添付ファイル対応
- エンドツーエンド暗号化

## セキュリティ

Signal はエンドツーエンド暗号化を提供するため、セキュリティの高い通信が可能です。
