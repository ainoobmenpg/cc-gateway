//! WebSocket message types
//!
//! Defines the JSON message format for WebSocket communication.

use serde::{Deserialize, Serialize};

/// Message from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a text message to Claude
    Chat {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<ImageData>,
    },

    /// Clear conversation history
    Clear,

    /// Request current session info
    SessionInfo,

    /// Ping for keepalive
    Ping,
}

/// Message from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Chat response from Claude
    ChatResponse {
        response: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tokens_used: Option<TokenUsage>,
    },

    /// Streaming text chunk
    StreamChunk {
        chunk: String,
        done: bool,
    },

    /// Error message
    Error {
        message: String,
    },

    /// Session information
    SessionInfo {
        session_id: String,
        message_count: usize,
    },

    /// Session cleared notification
    SessionCleared,

    /// Pong response
    Pong,

    /// Tool being executed (for UI feedback)
    ToolExecuting {
        name: String,
    },

    /// Tool execution result
    ToolResult {
        name: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
}

/// Image data for multimodal input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// MIME type (e.g., "image/png", "image/jpeg")
    pub media_type: String,
    /// Base64-encoded image data
    pub data: String,
}

impl ImageData {
    /// Create new image data
    pub fn new(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            media_type: media_type.into(),
            data: data.into(),
        }
    }

    /// Create from raw bytes (encodes to base64)
    pub fn from_bytes(media_type: impl Into<String>, bytes: &[u8]) -> Self {
        Self {
            media_type: media_type.into(),
            data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes),
        }
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl From<cc_core::llm::Usage> for TokenUsage {
    fn from(usage: cc_core::llm::Usage) -> Self {
        Self {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_client_message() {
        let msg = ClientMessage::Chat {
            message: "Hello".to_string(),
            image: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"chat"#));
        assert!(json.contains(r#""message":"Hello"#));
    }

    #[test]
    fn test_deserialize_server_message() {
        let json = r#"{"type":"chat_response","response":"Hi there!"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::ChatResponse { response, .. } => {
                assert_eq!(response, "Hi there!");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_image_data_base64() {
        let bytes = b"test image data";
        let img = ImageData::from_bytes("image/png", bytes);
        assert_eq!(img.media_type, "image/png");
        // base64 encoded
        assert!(!img.data.is_empty());
    }
}
