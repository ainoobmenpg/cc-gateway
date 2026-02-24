//! Signal message types

use serde::{Deserialize, Serialize};

/// Received Signal message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    /// Sender phone number
    pub sender: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: u64,
    /// Group ID (if group message)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Attachments
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// Attachment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Content type
    pub content_type: String,
    /// File name
    pub filename: Option<String>,
    /// Base64 encoded data (for received attachments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Message to send via Signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessage {
    /// Recipient phone number or group ID
    pub recipient: String,
    /// Message content
    pub message: String,
    /// Base64 encoded attachments (optional)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub base64_attachments: Vec<String>,
}

/// Signal CLI REST API response for sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResponse {
    /// Timestamp of sent message
    pub timestamp: u64,
}

/// Signal CLI REST API response for receiving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveResponse {
    /// List of received messages
    pub messages: Vec<SignalMessage>,
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Phone number
    pub number: String,
    /// Whether the account is registered
    pub registered: bool,
    /// Whether safety number has been approved
    pub safety_number: Option<String>,
}

/// Group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    /// Group ID (base64 encoded)
    pub id: String,
    /// Group name
    pub name: String,
    /// Group members
    pub members: Vec<String>,
    /// Whether the bot is in the group
    pub is_member: bool,
}
