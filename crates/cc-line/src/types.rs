//! LINE Messaging API types

use serde::{Deserialize, Serialize};

/// LINE user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineProfile {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(default)]
    pub picture_url: Option<String>,
    #[serde(default)]
    pub status_message: Option<String>,
}

/// LINE message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub id: String,
    #[serde(default)]
    pub text: Option<String>,
}

/// LINE source (user, group, or room)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub room_id: Option<String>,
}

/// LINE event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "replyToken")]
    pub reply_token: Option<String>,
    pub timestamp: i64,
    pub source: LineSource,
    pub message: Option<LineMessage>,
}

/// Webhook request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBody {
    pub destination: String,
    pub events: Vec<LineEvent>,
}

/// Reply message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyMessage {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    pub messages: Vec<MessageContent>,
}

/// Push message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushMessage {
    pub to: String,
    pub messages: Vec<MessageContent>,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum MessageContent {
    Text { text: String },
}

/// API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineApiResponse {
    pub ok: Option<bool>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub details: Option<Vec<ErrorDetail>>,
}

/// Error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    pub property: Option<String>,
}
