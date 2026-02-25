# Instagram チャネルガイド

Instagram を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | Instagram Graph API |
| crate | cc-instagram |
| ステータス | ✅ 実装済み |

## 設定

```toml
[instagram]
username = "${INSTAGRAM_USERNAME}"
password = "${INSTAGRAM_PASSWORD}"
page_id = "${INSTAGRAM_PAGE_ID}"
access_token = "${INSTAGRAM_ACCESS_TOKEN}"
```

### 環境変数

```bash
INSTAGRAM_USERNAME=your_username
INSTAGRAM_PASSWORD=your_password
INSTAGRAM_PAGE_ID=178414...
INSTAGRAM_ACCESS_TOKEN=...
```

## Instagram Business アカウントの設定

1. Facebook ページを作成
2. Instagram ビジネスアカウントに変換
3. Meta for Developers でアプリを作成
4. Instagram Basic Display API を追加
5. アクセストークンを取得

## 機能

- ダイレクトメッセージの送受信
- ストーリーに応答
- コメント応答
- 画像/動画対応

## 対応可能なコンテンツ

| コンテンツタイプ | 対応 |
|-----------------|:----:|
| テキスト DM | ✅ |
| 画像 DM | ✅ |
| 動画 DM | ✅ |
| ストーリーへのコメント | ✅ |
| 投稿へのコメント | ✅ |

## 制約

- Instagram API の利用には審査が必要
- レート制限あり
- ビジネスアカウント必須
