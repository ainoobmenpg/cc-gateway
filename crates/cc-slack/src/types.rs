//! Slack API types

use serde::{Deserialize, Serialize};

/// Slack user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackUser {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub real_name: Option<String>,
    #[serde(default)]
    pub profile: Option<UserProfile>,
}

/// User profile details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

/// Slack channel info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_channel: bool,
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub is_im: bool,
    #[serde(default)]
    pub is_private: bool,
}

/// Slack message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub user: Option<String>,
    pub text: String,
    pub ts: String,
    #[serde(default)]
    pub thread_ts: Option<String>,
    #[serde(default)]
    pub bot_id: Option<String>,
    #[serde(default)]
    pub subtype: Option<String>,
}

/// Slack event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub ts: Option<String>,
    #[serde(default)]
    pub thread_ts: Option<String>,
    #[serde(default)]
    pub bot_id: Option<String>,
    #[serde(default)]
    pub subtype: Option<String>,
}

/// Socket Mode hello event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketModeHello {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub num_connections: Option<i32>,
    #[serde(default)]
    pub connection_info: Option<ConnectionInfo>,
}

/// Connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    #[serde(default)]
    pub app_id: Option<String>,
}

/// Slash command payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
    pub trigger_id: String,
}

/// Message to send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMessage {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<serde_json::Value>>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackResponse<T> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_metadata: Option<ResponseMetadata>,
    #[serde(flatten)]
    pub data: Option<T>,
}

/// Response metadata (for pagination, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Auth test response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTestResponse {
    pub url: String,
    pub team: String,
    pub user: String,
    pub team_id: String,
    pub user_id: String,
    pub bot_id: Option<String>,
}

/// Conversations list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationsListResponse {
    pub channels: Vec<SlackChannel>,
    #[serde(default)]
    pub response_metadata: Option<ResponseMetadata>,
}

/// Post message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMessageResponse {
    pub ts: String,
    pub channel: String,
    pub message: Option<SlackMessage>,
}
