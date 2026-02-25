# iMessage チャネルガイド

iMessage を通じて AI アシスタントと対話できます（macOS のみ）。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | AppleScript |
| crate | cc-imessage |
| ステータス | ✅ 実装済み |
| 前提条件 | macOS |

## 設定

```toml
[imessage]
enabled = true
recipient = "+1234567890"
```

### 環境変数

```bash
IMESSAGE_RECIPIENT=+1234567890
```

## 前提条件

- macOS が動作するサーバー
- macOS 10.14 以上
- iMessage へのログイン

## 使用方法

AppleScript 経由で iMessage を操作します：

```applescript
tell application "Messages"
    send "Hello" to buddy "+1234567890" of service "iMessage"
end tell
```

## 機能

- テキストメッセージの送信
- 画像送信対応
- グループメッセージ対応

## 制約

- macOS 专用
- AppleScript が必要
- サーバーマシンで iMessage にログイン必要

## Automator サービス

macOS Automator を使ってサービスとして登録すると、より柔軟な自動化が可能です。
