//! Facebook Messenger API client

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::error::{FacebookError, Result};

/// Facebook Messenger API base URL
const FACEBOOK_API_URL: &str = "https://graph.facebook.com/v18.0";

/// Facebook API client
#[derive(Clone)]
pub struct FacebookApi {
    client: Client,
    page_id: String,
    access_token: String,
    verify_token: String,
}

impl FacebookApi {
    /// Create a new Facebook API client
    pub fn new(page_id: &str, access_token: &str, verify_token: &str) -> Self {
        Self {
            client: Client::new(),
            page_id: page_id.to_string(),
            access_token: access_token.to_string(),
            verify_token: verify_token.to_string(),
        }
    }

    /// Send a message to a user via Facebook Messenger
    pub async fn send_message(&self, recipient_id: &str, message: &str) -> Result<MessageResponse> {
        let url = format!("{}/{}/messages", FACEBOOK_API_URL, self.page_id);

        let payload = SendMessagePayload {
            messaging_type: "RESPONSE".to_string(),
            recipient: Recipient {
                id: recipient_id.to_string(),
            },
            message: MessageText {
                text: message.to_string(),
            },
        };

        debug!("Sending message to {}: {}", recipient_id, message);

        let response = self
            .client
            .post(&url)
            .query(&[("access_token", &self.access_token)])
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Facebook API error: {} - {}", status, body);
            return Err(FacebookError::Api(format!("{} - {}", status, body)));
        }

        let message_response: MessageResponse = response.json().await?;
        info!("Message sent successfully: {:?}", message_response.message_id);

        Ok(message_response)
    }

    /// Get user profile information
    pub async fn get_user_profile(&self, user_id: &str) -> Result<UserProfile> {
        let url = format!("{}/{}", FACEBOOK_API_URL, user_id);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("fields", "first_name,last_name,profile_pic"),
                ("access_token", &self.access_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Facebook API error: {} - {}", status, body);
            return Err(FacebookError::Api(format!("{} - {}", status, body)));
        }

        let profile: UserProfile = response.json().await?;
        debug!("Got user profile: {:?}", profile);

        Ok(profile)
    }

    /// Verify webhook challenge for Facebook webhook setup
    pub fn verify_webhook(&self, mode: &str, token: &str, challenge: &str) -> Result<String> {
        // Verify the webhook verify token matches what we configured
        if mode == "subscribe" && token == self.verify_token {
            info!("Webhook verified successfully");
            Ok(challenge.to_string())
        } else {
            error!("Webhook verification failed: invalid mode or token");
            Err(FacebookError::WebhookVerificationFailed)
        }
    }

    /// Handle incoming webhook payload
    pub fn handle_webhook(&self, payload: &str) -> Result<Vec<WebhookEntry>> {
        let webhook_payload: WebhookPayload = serde_json::from_str(payload)
            .map_err(|e| FacebookError::InvalidPayload(e.to_string()))?;

        if let Some(entries) = webhook_payload.entry {
            Ok(entries)
        } else {
            Err(FacebookError::InvalidPayload("No entries in webhook payload".to_string()))
        }
    }
}

// =============================================================================
// Data structures for Facebook Messenger API
// =============================================================================

#[derive(Debug, Serialize)]
struct SendMessagePayload {
    messaging_type: String,
    recipient: Recipient,
    message: MessageText,
}

#[derive(Debug, Serialize)]
struct Recipient {
    id: String,
}

#[derive(Debug, Serialize)]
struct MessageText {
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub recipient_id: Option<String>,
    pub message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_pic: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub object: Option<String>,
    pub entry: Option<Vec<WebhookEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEntry {
    pub id: Option<String>,
    pub time: Option<i64>,
    pub messaging: Option<Vec<WebhookMessaging>>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookMessaging {
    pub sender: Option<WebhookSender>,
    pub recipient: Option<WebhookRecipient>,
    pub timestamp: Option<i64>,
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
    pub quick_reply: Option<WebhookQuickReply>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookQuickReply {
    pub payload: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_parsing() {
        let payload = r#"{
            "object": "page",
            "entry": [{
                "id": "123456789",
                "messaging": [{
                    "sender": {"id": "user123"},
                    "recipient": {"id": "page123"},
                    "timestamp": 1234567890,
                    "message": {"mid": "mid.123", "text": "Hello"}
                }]
            }]
        }"#;

        let result = FacebookApi::new("test", "test", "verify").handle_webhook(payload);
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_verify_webhook() {
        let api = FacebookApi::new("test", "test", "verify");
        let result = api.verify_webhook("subscribe", "verify", "challenge");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "challenge");
    }
}
