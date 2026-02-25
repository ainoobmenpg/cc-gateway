# Voice チャネルガイド

音声を通じて AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | OpenAI Whisper / TTS / Twilio |
| crate | cc-voice |
| ステータス | ✅ 実装済み |

## コンポーネント

### Speech-to-Text (Whisper)

音声ファイルをテキストに変換します。

```toml
[voice.whisper]
model = "whisper-1"
language = "ja"  # 日本語
```

### Text-to-Speech (TTS)

テキストを音声に変換します。

```toml
[voice.tts]
model = "tts-1"
voice = "alloy"  # alloy, echo, fable, onyx, nova, shimmer
```

### Phone (Twilio)

電話を通じて対話できます。

```toml
[voice.phone]
enabled = true
twilio_account_sid = "${TWILIO_ACCOUNT_SID}"
twilio_auth_token = "${TWILIO_AUTH_TOKEN}"
twilio_phone_number = "+1234567890"
```

### 環境変数

```bash
OPENAI_API_KEY=sk-...
TWILIO_ACCOUNT_SID=AC...
TWILIO_AUTH_TOKEN=...
TWILIO_PHONE_NUMBER=+1234567890
```

## 機能

| 機能 | 説明 |
|------|------|
| 音声入力 | Whisper で文字起こし |
| 音声出力 | TTS で応答を音声化 |
| 電話応答 | Twilio で着信応答 |
| 音声ファイル | MP3/WAV 対応 |

## 使用例

### 電話応答

1. Twilio で電話番号を購入
2. Webhook URL を設定: `https://your-server/voice/webhook`
3. 着信時に AI が応答

```xml
<!-- Twilio Webhook Response -->
<Response>
    <Gather numDigits="1" action="/voice/gather">
        <Say>こんにちは。メッセージを残してください。</Say>
    </Gather>
</Response>
```

## 対応言語

Whisper/TTS は多言語に対応しています：

- 日本語 (ja)
- 英語 (en)
- 中国語 (zh)
- 韓国語 (ko)
- など

## 品質設定

```toml
[voice.tts]
model = "tts-1-hd"  # 高品質
speed = 1.0
```

## コスト

| サービス | コスト |
|---------|-------|
| Whisper | $0.006/分 |
| TTS | $0.015/1K 文字 |
| Twilio | 通話料金 |
