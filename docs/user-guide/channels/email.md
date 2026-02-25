# Email チャネルガイド

Email を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | SMTP / POP3 |
| crate | cc-email |
| ステータス | ✅ 実装済み |

## 設定

### SMTP (送信)

```toml
[email.smtp]
host = "smtp.gmail.com"
port = 587
use_tls = true
user = "${EMAIL_USER}"
password = "${EMAIL_PASSWORD}"
```

### POP3 (受信)

```toml
[email.pop3]
host = "pop.gmail.com"
port = 995
use_ssl = true
user = "${EMAIL_USER}"
password = "${EMAIL_PASSWORD}"
```

### 環境変数

```bash
EMAIL_USER=your-email@gmail.com
EMAIL_PASSWORD=app-specific-password
```

## 機能

- SMTP によるメール送信
- POP3 によるメール受信
- IMAP 対応（予定）
- 添付ファイル対応
- HTML メール対応

## セキュリティ

- App Password の使用を推奨（Gmail の2段階認証有効時）
- TLS/SSL による暗号化

## メールアドレス

| 用途 | アドレス形式 |
|------|------------|
| 受信 | bot@yourdomain.com |
| 送信 | from: bot@yourdomain.com |

## 定期チェック

```toml
[email]
poll_interval = 60  # 秒
```

ポーリング間隔でメールボックスをチェックし、新着メールに対して AI が応答します。
