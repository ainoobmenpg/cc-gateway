//! Signal CLI REST API client
//!
//! Communicates with signal-cli-rest-api server

use reqwest::Client;
use tracing::{debug, error, info};

use crate::error::{Result, SignalError};
use crate::types::*;

/// Signal CLI REST API client
#[derive(Clone)]
pub struct SignalApiClient {
    client: Client,
    base_url: String,
    phone_number: String,
}

impl SignalApiClient {
    /// Create a new Signal API client
    pub fn new(base_url: &str, phone_number: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(SignalError::HttpError)?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            phone_number: phone_number.to_string(),
        })
    }

    /// Check if the API is available
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/v1/about", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                debug!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Send a text message to a recipient
    pub async fn send_message(&self, recipient: &str, message: &str) -> Result<SendResponse> {
        let url = format!("{}/v2/send", self.base_url);

        let body = serde_json::json!({
            "number": self.phone_number,
            "recipients": [recipient],
            "message": message,
        });

        debug!("Sending message to {}", recipient);

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(SignalError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Send message failed: {} - {}", status, error_text);
            return Err(SignalError::ApiError(format!("{}: {}", status, error_text)));
        }

        let send_response: SendResponse = response
            .json()
            .await
            .map_err(|e| SignalError::ParseError(e.to_string()))?;

        info!("Message sent to {} at timestamp {}", recipient, send_response.timestamp);
        Ok(send_response)
    }

    /// Send a message with attachments
    pub async fn send_message_with_attachments(
        &self,
        recipient: &str,
        message: &str,
        attachments: Vec<String>,
    ) -> Result<SendResponse> {
        let url = format!("{}/v2/send", self.base_url);

        let body = serde_json::json!({
            "number": self.phone_number,
            "recipients": [recipient],
            "message": message,
            "base64_attachments": attachments,
        });

        debug!("Sending message with {} attachments to {}", attachments.len(), recipient);

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(SignalError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Send message failed: {} - {}", status, error_text);
            return Err(SignalError::ApiError(format!("{}: {}", status, error_text)));
        }

        let send_response: SendResponse = response
            .json()
            .await
            .map_err(|e| SignalError::ParseError(e.to_string()))?;

        info!("Message with attachments sent to {}", recipient);
        Ok(send_response)
    }

    /// Receive messages
    pub async fn receive_messages(&self) -> Result<Vec<SignalMessage>> {
        let url = format!("{}/v1/receive/{}", self.base_url, self.phone_number);

        debug!("Receiving messages for {}", self.phone_number);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(SignalError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Receive messages failed: {} - {}", status, error_text);
            return Err(SignalError::ApiError(format!("{}: {}", status, error_text)));
        }

        // The API returns messages directly or an empty array
        let messages: Vec<SignalMessage> = response
            .json()
            .await
            .map_err(|e| SignalError::ParseError(e.to_string()))?;

        debug!("Received {} messages", messages.len());
        Ok(messages)
    }

    /// Get account information
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        let url = format!("{}/v1/accounts/{}", self.base_url, self.phone_number);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(SignalError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SignalError::ApiError(format!("{}: {}", status, error_text)));
        }

        let account_info: AccountInfo = response
            .json()
            .await
            .map_err(|e| SignalError::ParseError(e.to_string()))?;

        Ok(account_info)
    }

    /// Get list of groups
    pub async fn get_groups(&self) -> Result<Vec<GroupInfo>> {
        let url = format!("{}/v1/groups/{}", self.base_url, self.phone_number);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(SignalError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SignalError::ApiError(format!("{}: {}", status, error_text)));
        }

        let groups: Vec<GroupInfo> = response
            .json()
            .await
            .map_err(|e| SignalError::ParseError(e.to_string()))?;

        Ok(groups)
    }

    /// Send a reaction to a message
    pub async fn send_reaction(
        &self,
        recipient: &str,
        target_timestamp: u64,
        emoji: &str,
    ) -> Result<()> {
        let url = format!("{}/v1/reactions/{}", self.base_url, self.phone_number);

        let body = serde_json::json!({
            "recipient": recipient,
            "timestamp": target_timestamp,
            "emoji": emoji,
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(SignalError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SignalError::ApiError(format!("{}: {}", status, error_text)));
        }

        Ok(())
    }

    /// Start typing indicator (if supported)
    pub async fn start_typing(&self, _recipient: &str) -> Result<()> {
        // Note: typing indicators may not be supported by all signal-cli implementations
        debug!("Typing indicator not supported");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = SignalApiClient::new("http://localhost:8080", "+1234567890");
        assert!(client.is_ok());
    }
}
