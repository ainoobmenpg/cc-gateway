# WebSocket チャネルガイド

WebSocket を通じてリアルタイムに AI アシスタントと対話できます。

## 概要

| 項目 | 値 |
|------|-----|
| プロバイダー | tokio-tungstenite |
| crate | cc-ws |
| ステータス | ✅ 実装済み |

## 設定

```toml
[websocket]
enabled = true
host = "0.0.0.0"
port = 3001
tls_enabled = false
tls_cert_path = ""
tls_key_path = ""
```

### 環境変数

```bash
WS_HOST=0.0.0.0
WS_PORT=3001
WS_TLS_ENABLED=false
```

## 接続

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

// 認証
ws.onopen = () => {
    ws.send(JSON.stringify({
        type: 'auth',
        token: 'your-api-key'
    }));
};

// メッセージ送信
ws.send(JSON.stringify({
    type: 'message',
    content: 'Hello!'
}));

// メッセージ受信
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log(data.content);
};
```

## メッセージフォーマット

### クライアント → サーバー

```json
{
    "type": "message",
    "content": "メッセージ内容",
    "session_id": "optional-session-id"
}
```

```json
{
    "type": "auth",
    "token": "API_KEY"
}
```

### サーバー → クライアント

```json
{
    "type": "message",
    "content": "応答内容",
    "session_id": "session-id"
}
```

```json
{
    "type": "tool_use",
    "tool": "bash",
    "input": {...}
}
```

```json
{
    "type": "tool_result",
    "tool": "bash",
    "result": "結果"
}
```

```json
{
    "type": "error",
    "message": "エラーメッセージ"
}
```

## 機能

- 双方向リアルタイム通信
- ストリーミング応答
- ツール実行通知
- セッション管理
- 認証/認可

## セキュリティ

- API キー認証必須
- TLS/SSL 対応可能
- レート制限

## Canvas機能

WebSocket 経由で Canvas 操作が可能です：

```json
{
    "type": "canvas",
    "action": "draw",
    "data": {
        "type": "rect",
        "x": 10,
        "y": 10,
        "width": 100,
        "height": 100
    }
}
```

## 制約

- ブラウザの WebSocket 制限に注意
- 長時間接続は自動的に切断される場合あり
