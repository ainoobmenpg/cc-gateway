# WhatsApp チャネルガイド

WhatsApp を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | Twilio API |
| crate | cc-whatsapp |
| ステータス | ✅ 実装済み |

## 設定

```toml
[whatsapp]
account_sid = "${TWILIO_ACCOUNT_SID}"
auth_token = "${TWILIO_AUTH_TOKEN}"
phone_number = "+1234567890"
```

### 環境変数

```bash
TWILIO_ACCOUNT_SID=AC...
TWILIO_AUTH_TOKEN=...
TWILIO_PHONE_NUMBER=+1234567890
```

## 使用方法

1. Twilio で WhatsApp Business アカウントを作成
2. Twilio から phone_number を取得
3. 設定ファイルに認証情報を追加
4. cc-gateway を起動

## 機能

- テキストメッセージの送受信
- セッション管理
- ツール実行（9層ポリシー）
- 画像対応（ Twilio Media URL）

## 制約

- Twilio のレート制限に従う
- WhatsApp Business API の利用料が発生
