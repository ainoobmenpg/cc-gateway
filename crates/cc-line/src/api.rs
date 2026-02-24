//! LINE Messaging API client
//!
//! Communicates with LINE Messaging API

use reqwest::Client;
use tracing::{debug, error, info};

use crate::error::{LineError, Result};
use crate::types::*;

/// LINE Messaging API client
#[derive(Clone)]
pub struct LineApiClient {
    client: Client,
    channel_access_token: String,
    base_url: String,
}

impl LineApiClient {
    /// Create a new LINE API client
    pub fn new(channel_access_token: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(LineError::HttpError)?;

        Ok(Self {
            client,
            channel_access_token: channel_access_token.to_string(),
            base_url: "https://api.line.me/v2".to_string(),
        })
    }

    /// Add authorization header
    fn add_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.bearer_auth(&self.channel_access_token)
    }

    /// Get user profile
    pub async fn get_profile(&self, user_id: &str) -> Result<LineProfile> {
        let url = format!("{}/profile/{}", self.base_url, user_id);

        debug!("Getting profile for user: {}", user_id);

        let response = self
            .add_auth(self.client.get(&url))
            .send()
            .await
            .map_err(LineError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Get profile failed: {} - {}", status, error_text);
            return Err(LineError::ApiError(format!("{}: {}", status, error_text)));
        }

        let profile: LineProfile = response
            .json()
            .await
            .map_err(|e| LineError::ParseError(e.to_string()))?;

        info!("Got profile for user: {}", profile.display_name);
        Ok(profile)
    }

    /// Reply to a message
    pub async fn reply_message(&self, reply_token: &str, text: &str) -> Result<()> {
        let url = format!("{}/bot/message/reply", self.base_url);

        let body = ReplyMessage {
            reply_token: reply_token.to_string(),
            messages: vec![MessageContent::Text { text: text.to_string() }],
        };

        debug!("Replying to message");

        let response = self
            .add_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(LineError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Reply message failed: {} - {}", status, error_text);
            return Err(LineError::ApiError(format!("{}: {}", status, error_text)));
        }

        Ok(())
    }

    /// Push a message to a user/group/room
    pub async fn push_message(&self, to: &str, text: &str) -> Result<()> {
        let url = format!("{}/bot/message/push", self.base_url);

        let body = PushMessage {
            to: to.to_string(),
            messages: vec![MessageContent::Text { text: text.to_string() }],
        };

        debug!("Pushing message to: {}", to);

        let response = self
            .add_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(LineError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Push message failed: {} - {}", status, error_text);
            return Err(LineError::ApiError(format!("{}: {}", status, error_text)));
        }

        Ok(())
    }

    /// Send multiple messages (for long responses)
    pub async fn push_messages(&self, to: &str, texts: &[String]) -> Result<()> {
        // LINE allows up to 5 messages per API call
        for chunk in texts.chunks(5) {
            let url = format!("{}/bot/message/push", self.base_url);

            let messages: Vec<MessageContent> = chunk
                .iter()
                .map(|text| MessageContent::Text { text: text.clone() })
                .collect();

            let body = PushMessage {
                to: to.to_string(),
                messages,
            };

            let response = self
                .add_auth(self.client.post(&url).json(&body))
                .send()
                .await
                .map_err(LineError::HttpError)?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                error!("Push messages failed: {} - {}", status, error_text);
                return Err(LineError::ApiError(format!("{}: {}", status, error_text)));
            }

            // Small delay between chunks
            if chunk.len() == 5 && texts.len() > 5 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Get group member profile
    pub async fn get_group_member_profile(&self, group_id: &str, user_id: &str) -> Result<LineProfile> {
        let url = format!("{}/bot/group/{}/member/{}", self.base_url, group_id, user_id);

        let response = self
            .add_auth(self.client.get(&url))
            .send()
            .await
            .map_err(LineError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LineError::ApiError(format!("{}: {}", status, error_text)));
        }

        let profile: LineProfile = response
            .json()
            .await
            .map_err(|e| LineError::ParseError(e.to_string()))?;

        Ok(profile)
    }

    /// Leave a group
    pub async fn leave_group(&self, group_id: &str) -> Result<()> {
        let url = format!("{}/bot/group/{}/leave", self.base_url, group_id);

        let response = self
            .add_auth(self.client.post(&url))
            .send()
            .await
            .map_err(LineError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LineError::ApiError(format!("{}: {}", status, error_text)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = LineApiClient::new("test-token");
        assert!(client.is_ok());
    }
}
