//! Slack Web API client
//!
//! Communicates with Slack Web API

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::error::{Result, SlackError};
use crate::types::*;

/// Slack Web API client
#[derive(Clone)]
pub struct SlackApiClient {
    client: Client,
    bot_token: String,
    base_url: String,
}

impl SlackApiClient {
    /// Create a new Slack API client
    pub fn new(bot_token: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(SlackError::HttpError)?;

        Ok(Self {
            client,
            bot_token: bot_token.to_string(),
            base_url: "https://slack.com/api".to_string(),
        })
    }

    /// Add authorization header
    fn add_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.bearer_auth(&self.bot_token)
    }

    /// Test authentication
    pub async fn auth_test(&self) -> Result<AuthTestResponse> {
        let url = format!("{}/auth.test", self.base_url);

        debug!("Testing Slack authentication");

        let response = self
            .add_auth(self.client.post(&url))
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        let result: SlackResponse<AuthTestResponse> = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            return Err(SlackError::ApiError(result.error.unwrap_or("Unknown error".to_string())));
        }

        info!("Slack auth test successful for team: {:?}", result.data.as_ref().map(|r| &r.team));
        Ok(result.data.unwrap())
    }

    /// Post a message to a channel
    pub async fn post_message(&self, message: &PostMessage) -> Result<PostMessageResponse> {
        let url = format!("{}/chat.postMessage", self.base_url);

        debug!("Posting message to channel: {}", message.channel);

        let response = self
            .add_auth(self.client.post(&url).json(message))
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Post message failed: {} - {}", status, error_text);
            return Err(SlackError::ApiError(format!("{}: {}", status, error_text)));
        }

        let result: SlackResponse<PostMessageResponse> = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            return Err(SlackError::ApiError(result.error.unwrap_or("Unknown error".to_string())));
        }

        Ok(result.data.unwrap())
    }

    /// Post a simple text message
    pub async fn send_message(&self, channel: &str, text: &str, thread_ts: Option<&str>) -> Result<PostMessageResponse> {
        let message = PostMessage {
            channel: channel.to_string(),
            text: text.to_string(),
            thread_ts: thread_ts.map(|s| s.to_string()),
            blocks: None,
        };

        self.post_message(&message).await
    }

    /// Get list of conversations (channels)
    pub async fn conversations_list(&self, types: Option<&str>) -> Result<Vec<SlackChannel>> {
        let url = format!("{}/conversations.list", self.base_url);

        let mut params = vec![("limit", "200")];
        if let Some(types) = types {
            params.push(("types", types));
        }

        debug!("Getting conversations list");

        let response = self
            .add_auth(self.client.get(&url).query(&params))
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        let result: SlackResponse<ConversationsListResponse> = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            return Err(SlackError::ApiError(result.error.unwrap_or("Unknown error".to_string())));
        }

        Ok(result.data.map(|d| d.channels).unwrap_or_default())
    }

    /// Get user info
    pub async fn users_info(&self, user_id: &str) -> Result<SlackUser> {
        let url = format!("{}/users.info", self.base_url);

        debug!("Getting user info for: {}", user_id);

        let response = self
            .add_auth(self.client.get(&url).query(&[("user", user_id)]))
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        #[derive(Debug, Serialize, Deserialize)]
        struct UsersInfoResponse {
            user: SlackUser,
        }

        let result: SlackResponse<UsersInfoResponse> = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            return Err(SlackError::ApiError(result.error.unwrap_or("Unknown error".to_string())));
        }

        Ok(result.data.unwrap().user)
    }

    /// Add a reaction to a message
    pub async fn reactions_add(&self, channel: &str, timestamp: &str, name: &str) -> Result<()> {
        let url = format!("{}/reactions.add", self.base_url);

        let body = serde_json::json!({
            "channel": channel,
            "timestamp": timestamp,
            "name": name,
        });

        let response = self
            .add_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        let result: SlackResponse<()> = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            // Don't fail on reaction errors, just log
            debug!("Reaction failed: {:?}", result.error);
        }

        Ok(())
    }

    /// Open a DM channel with a user
    pub async fn im_open(&self, user_id: &str) -> Result<String> {
        let url = format!("{}/conversations.open", self.base_url);

        let body = serde_json::json!({
            "users": user_id,
        });

        let response = self
            .add_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        #[derive(Debug, Serialize, Deserialize)]
        struct ImOpenResponse {
            channel: SlackChannel,
        }

        let result: SlackResponse<ImOpenResponse> = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            return Err(SlackError::ApiError(result.error.unwrap_or("Unknown error".to_string())));
        }

        Ok(result.data.unwrap().channel.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = SlackApiClient::new("xoxb-test-token");
        assert!(client.is_ok());
    }
}
