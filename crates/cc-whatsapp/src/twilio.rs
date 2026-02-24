//! Twilio API client for WhatsApp

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{Result, WhatsAppError};

/// Twilio API client
#[derive(Debug, Clone)]
pub struct TwilioClient {
    client: Client,
    account_sid: String,
    auth_token: String,
    phone_number: String,
    base_url: String,
}

/// Incoming WhatsApp message from Twilio webhook
#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub from: String,
    pub to: String,
    pub body: String,
    #[serde(rename = "MessageSid")]
    pub message_sid: String,
    #[serde(rename = "AccountSid")]
    pub account_sid: String,
}

/// Outgoing message payload
#[derive(Debug, Serialize)]
struct SendMessagePayload {
    from: String,
    to: String,
    body: String,
}

impl TwilioClient {
    /// Create a new Twilio client
    pub fn new(account_sid: String, auth_token: String, phone_number: String) -> Self {
        Self {
            client: Client::new(),
            account_sid,
            auth_token,
            phone_number,
            base_url: "https://api.twilio.com".to_string(),
        }
    }

    /// Send a WhatsApp message
    pub async fn send_message(&self, to: &str, body: &str) -> Result<String> {
        info!("Sending WhatsApp message to {}", to);

        let url = format!(
            "{}/2010-04-01/Accounts/{}/Messages.json",
            self.base_url, self.account_sid
        );

        let payload = SendMessagePayload {
            from: format!("whatsapp:{}", self.phone_number),
            to: to.to_string(),
            body: body.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WhatsAppError::Api(format!(
                "Failed to send message: {} - {}",
                status, text
            )));
        }

        #[derive(Deserialize)]
        struct SendMessageResponse {
            sid: String,
        }

        let result: SendMessageResponse = response.json().await?;
        Ok(result.sid)
    }

    /// Verify webhook signature
    pub fn verify_signature(&self, url: &str, params: &str, signature: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = match HmacSha256::new_from_slice(self.auth_token.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let data = format!("{}{}", url, params);
        mac.update(data.as_bytes());

        let expected = mac.finalize().into_bytes();
        let expected_hex = hex::encode(expected);

        expected_hex == signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TwilioClient::new(
            "AC123".to_string(),
            "token123".to_string(),
            "+1234567890".to_string(),
        );
        assert_eq!(client.account_sid, "AC123");
    }
}
