//! Instagram Graph API client implementation

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::error::{InstagramError, Result};

/// Instagram API base URL
const INSTAGRAM_GRAPH_API_URL: &str = "https://graph.instagram.com";

/// Instagram Graph API client
#[derive(Clone)]
pub struct InstagramApi {
    client: Client,
    access_token: String,
    #[allow(dead_code)]
    app_secret: Option<String>,
    page_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub recipient: Recipient,
    pub message: MessagePayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipient {
    #[serde(rename = "id")]
    pub psid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagePayload {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub recipient_id: String,
    pub message_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "account_type")]
    pub account_type: Option<String>,
    #[serde(rename = "media_count")]
    pub media_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEntry {
    pub id: String,
    pub messaging: Option<Vec<WebhookMessagingEvent>>,
    pub changes: Option<Vec<WebhookChange>>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookMessagingEvent {
    pub sender: WebhookSender,
    pub recipient: WebhookRecipient,
    pub timestamp: String,
    pub message: Option<WebhookMessage>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookSender {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookRecipient {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookMessage {
    pub mid: Option<String>,
    pub text: Option<String>,
    pub attachments: Option<Vec<WebhookAttachment>>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookAttachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub payload: WebhookPayload,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookChange {
    pub field: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayloadInstagram {
    pub object: String,
    pub entry: Vec<WebhookEntry>,
}

impl InstagramApi {
    /// Create a new Instagram API client
    pub fn new(access_token: String, page_id: String, app_secret: Option<String>) -> Self {
        let client = Client::new();
        Self {
            client,
            access_token,
            app_secret,
            page_id,
        }
    }

    /// Send a message to a user via Instagram Direct Message
    pub async fn send_message(&self, psid: &str, text: &str) -> Result<SendMessageResponse> {
        info!("Sending message to PSID: {}", psid);

        let url = format!("{}/v21.0/me/messages", INSTAGRAM_GRAPH_API_URL);

        let request_body = SendMessageRequest {
            recipient: Recipient {
                psid: psid.to_string(),
            },
            message: MessagePayload {
                text: text.to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .query(&[("access_token", &self.access_token)])
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        debug!("Instagram API response: {} - {}", status, body);

        if !status.is_success() {
            error!("Instagram API error: {} - {}", status, body);
            return Err(InstagramError::GraphApi(format!(
                "Status: {}, Body: {}",
                status, body
            )));
        }

        let response_json: serde_json::Value = serde_json::from_str(&body)?;

        let recipient_id = response_json["recipient_id"]
            .as_str()
            .unwrap_or(psid)
            .to_string();

        let message_id = response_json["message_id"].as_str().map(String::from);

        Ok(SendMessageResponse {
            recipient_id,
            message_id,
        })
    }

    /// Get user profile information
    pub async fn get_user_profile(&self, psid: &str) -> Result<UserProfileResponse> {
        info!("Getting user profile for PSID: {}", psid);

        let url = format!("{}/{}", INSTAGRAM_GRAPH_API_URL, psid);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("fields", "id,name,account_type,media_count"),
                ("access_token", &self.access_token),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        debug!("Instagram API response: {} - {}", status, body);

        if !status.is_success() {
            error!("Instagram API error: {} - {}", status, body);
            return Err(InstagramError::GraphApi(format!(
                "Status: {}, Body: {}",
                status, body
            )));
        }

        let profile: UserProfileResponse = serde_json::from_str(&body)?;
        Ok(profile)
    }

    /// Handle incoming webhook
    pub fn handle_webhook(&self, payload: &str) -> Result<Vec<WebhookMessagingEvent>> {
        let webhook: WebhookPayloadInstagram = serde_json::from_str(payload)?;

        let mut events = Vec::new();

        for entry in webhook.entry {
            if let Some(messaging) = entry.messaging {
                for event in messaging {
                    // Only handle messages sent to our page
                    if event.recipient.id == self.page_id {
                        events.push(event);
                    }
                }
            }
        }

        Ok(events)
    }

    /// Verify webhook challenge (for initial setup)
    pub fn verify_webhook_challenge(
        &self,
        mode: &str,
        _token: &str,
        challenge: &str,
    ) -> Result<String> {
        // Verify the webhook subscription
        if mode != "subscribe" {
            return Err(InstagramError::WebhookVerificationFailed);
        }

        // In production, you would verify the token matches your app secret
        // For now, just return the challenge
        Ok(challenge.to_string())
    }

    /// Get the page ID
    pub fn page_id(&self) -> &str {
        &self.page_id
    }

    /// Get the access token
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

impl Default for InstagramApi {
    fn default() -> Self {
        Self {
            client: Client::new(),
            access_token: String::new(),
            app_secret: None,
            page_id: String::new(),
        }
    }
}
