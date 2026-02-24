//! Slack message handler implementation

use std::sync::Arc;
use tracing::{debug, error, info};

use cc_core::{ClaudeClient, Message};

use crate::api::SlackApiClient;
use crate::error::Result;
use crate::session::InMemorySessionStore;
use crate::types::SlackMessage;

/// Configuration for the message handler
#[derive(Clone, Debug)]
pub struct HandlerConfig {
    /// Allowed channels. Empty = allow all.
    pub allowed_channels: Vec<String>,
    /// Allowed users. Empty = allow all.
    pub allowed_users: Vec<String>,
    /// Bot user ID (to ignore own messages)
    pub bot_user_id: Option<String>,
    /// System prompt for Claude
    pub system_prompt: String,
    /// Maximum message length before splitting
    pub max_message_length: usize,
}

impl Default for HandlerConfig {
    fn default() -> Self {
        Self {
            allowed_channels: Vec::new(),
            allowed_users: Vec::new(),
            bot_user_id: None,
            system_prompt: "You are a helpful assistant. Respond concisely. Use Slack markdown formatting when appropriate.".to_string(),
            max_message_length: 3500, // Slack has ~4000 char limit, leave some buffer
        }
    }
}

/// Message handler for Slack
pub struct MessageHandler {
    api_client: SlackApiClient,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    config: HandlerConfig,
}

impl MessageHandler {
    /// Create a new message handler
    pub fn new(
        api_client: SlackApiClient,
        claude_client: Arc<ClaudeClient>,
        session_store: Arc<InMemorySessionStore>,
        config: HandlerConfig,
    ) -> Self {
        Self {
            api_client,
            claude_client,
            session_store,
            config,
        }
    }

    /// Process an incoming message
    pub async fn process_message(&self, msg: &SlackMessage) -> Result<()> {
        // Skip bot messages (including our own)
        if msg.bot_id.is_some() || msg.subtype.as_deref() == Some("bot_message") {
            return Ok(());
        }

        // Skip messages from ourselves
        if let Some(ref bot_id) = self.config.bot_user_id {
            if msg.user.as_deref() == Some(bot_id) {
                return Ok(());
            }
        }

        // Check channel authorization
        if !self.is_channel_allowed(&msg.channel) {
            debug!("Ignoring message from unauthorized channel: {}", msg.channel);
            return Ok(());
        }

        // Check user authorization
        if let Some(ref user_id) = msg.user {
            if !self.is_user_allowed(user_id) {
                debug!("Ignoring message from unauthorized user: {}", user_id);
                return Ok(());
            }
        }

        let content = msg.text.trim();
        if content.is_empty() {
            return Ok(());
        }

        // Handle commands
        if content.starts_with('!') || content.starts_with('/') {
            return self.handle_command(msg).await;
        }

        // Regular message processing
        self.process_with_claude(msg).await
    }

    /// Check if channel is allowed
    fn is_channel_allowed(&self, channel_id: &str) -> bool {
        if self.config.allowed_channels.is_empty() {
            return true;
        }
        self.config.allowed_channels.contains(&channel_id.to_string())
    }

    /// Check if user is allowed
    fn is_user_allowed(&self, user_id: &str) -> bool {
        if self.config.allowed_users.is_empty() {
            return true;
        }
        self.config.allowed_users.contains(&user_id.to_string())
    }

    /// Handle commands
    async fn handle_command(&self, msg: &SlackMessage) -> Result<()> {
        let content = msg.text.trim();
        let command = content.split_whitespace().next().unwrap_or("");

        match command {
            "!clear" | "!reset" | "/clear" | "/reset" => {
                let session_key = self.get_session_key(msg);
                if self.session_store.clear(&session_key) {
                    self.send_reply(msg, "Session reset.").await?;
                } else {
                    self.send_reply(msg, "No session to reset.").await?;
                }
            }
            "!help" | "/help" => {
                let help_text = concat!(
                    "Available commands:\n",
                    "!clear - Reset conversation session\n",
                    "!help - Show this help\n",
                    "!status - Show session status\n",
                    "\n",
                    "Otherwise, just chat normally!"
                );
                self.send_reply(msg, help_text).await?;
            }
            "!status" | "/status" => {
                let session_key = self.get_session_key(msg);
                let session = self.session_store.get(&session_key);
                let status = match session {
                    Some(s) => format!("Message count: {}", s.message_count()),
                    None => "New session".to_string(),
                };
                self.send_reply(msg, &status).await?;
            }
            _ => {
                // Unknown command, treat as regular message
                self.process_with_claude(msg).await?;
            }
        }

        Ok(())
    }

    /// Get session key (channel or thread)
    fn get_session_key(&self, msg: &SlackMessage) -> String {
        // Use thread_ts if available, otherwise channel
        match &msg.thread_ts {
            Some(thread_ts) => format!("{}-{}", msg.channel, thread_ts),
            None => msg.channel.clone(),
        }
    }

    /// Process message with Claude API
    async fn process_with_claude(&self, msg: &SlackMessage) -> Result<()> {
        let content = msg.text.trim();
        let session_key = self.get_session_key(msg);

        info!("Processing message from {:?} in {}: {}", msg.user, msg.channel, content);

        // Add reaction to show we're processing
        let _ = self.api_client.reactions_add(&msg.channel, &msg.ts, "thinking_face").await;

        // Get or create session
        let session = self.session_store.get_or_create(&session_key);

        // Build message history
        let mut messages: Vec<Message> = session.messages.clone();
        messages.push(Message::user(content));

        // Build request with conversation history
        let mut request_builder = self
            .claude_client
            .request_builder()
            .system(&self.config.system_prompt)
            .max_tokens(2048);

        // Add conversation history (limit to last 20 messages)
        let history_start = messages.len().saturating_sub(20);
        for message in messages.into_iter().skip(history_start) {
            request_builder = request_builder.message(message);
        }

        let request = request_builder.build();

        match self.claude_client.messages(request).await {
            Ok(response) => {
                // Extract text from response
                let text = response
                    .content
                    .iter()
                    .filter_map(|c| {
                        if let cc_core::MessageContent::Text { text } = c {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Update session
                self.session_store
                    .add_message(&session_key, Message::user(content));
                self.session_store
                    .add_message(&session_key, Message::assistant(&text));

                // Send response
                self.send_reply(msg, &text).await?;
            }
            Err(e) => {
                error!("Claude API error: {:?}", e);
                self.send_reply(msg, &format!("Error: {}", e))
                    .await?;
            }
        }

        Ok(())
    }

    /// Send a reply
    async fn send_reply(&self, msg: &SlackMessage, text: &str) -> Result<()> {
        // Split message if necessary
        if text.len() <= self.config.max_message_length {
            self.api_client
                .send_message(&msg.channel, text, Some(&msg.ts))
                .await?;
        } else {
            let chunks = self.split_message(text, self.config.max_message_length);
            for (i, chunk) in chunks.iter().enumerate() {
                let content = if i == 0 {
                    chunk.clone()
                } else {
                    format!("(continued {})\n{}", i + 1, chunk)
                };

                // Small delay between messages
                if i > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }

                if let Err(e) = self.api_client
                    .send_message(&msg.channel, &content, Some(&msg.ts))
                    .await
                {
                    error!("Failed to send message chunk {}: {:?}", i, e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Split message at sentence boundaries
    fn split_message(&self, text: &str, max_size: usize) -> Vec<String> {
        if text.len() <= max_size {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            if remaining.len() <= max_size {
                chunks.push(remaining.to_string());
                break;
            }

            let search_end = max_size.min(remaining.len());
            let chunk = &remaining[..search_end];

            // Try to break at sentence end
            let break_point = chunk
                .rfind("\n\n")
                .or_else(|| chunk.rfind("\n"))
                .or_else(|| chunk.rfind("。"))
                .or_else(|| chunk.rfind(". "))
                .or_else(|| chunk.rfind("! "))
                .or_else(|| chunk.rfind("? "))
                .or_else(|| chunk.rfind(" "))
                .map(|i| i + 1)
                .unwrap_or(max_size);

            chunks.push(remaining[..break_point].to_string());
            remaining = &remaining[break_point..];
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::{Config, LlmConfig, LlmProvider, ApiConfig, MemoryConfig, McpConfig, SchedulerConfig};

    fn mock_config() -> Config {
        Config {
            llm: LlmConfig {
                api_key: "test-key".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                provider: LlmProvider::Claude,
                base_url: None,
            },
            claude_api_key: "test-key".to_string(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            discord_token: None,
            admin_user_ids: vec![],
            api: ApiConfig::default(),
            api_key: None,
            memory: MemoryConfig::default(),
            mcp: McpConfig::default(),
            scheduler: SchedulerConfig::default(),
        }
    }

    #[test]
    fn test_handler_config_default() {
        let config = HandlerConfig::default();
        assert!(config.allowed_channels.is_empty());
        assert!(config.allowed_users.is_empty());
        assert_eq!(config.max_message_length, 3500);
    }

    #[test]
    fn test_split_message() {
        let config = HandlerConfig::default();
        let api_client = SlackApiClient::new("xoxb-test").unwrap();
        let claude_client = Arc::new(ClaudeClient::new(&mock_config()).unwrap());

        let handler = MessageHandler {
            api_client,
            claude_client,
            session_store: Arc::new(InMemorySessionStore::new()),
            config: config.clone(),
        };

        let short = "Short message";
        let result = handler.split_message(short, 100);
        assert_eq!(result.len(), 1);

        let long = "This is a long message.\n\nIt should be split.\n\nAt paragraph boundaries.";
        let result = handler.split_message(long, 30);
        assert!(result.len() > 1);
    }

    #[test]
    fn test_is_channel_allowed() {
        let config = HandlerConfig::default();
        let api_client = SlackApiClient::new("xoxb-test").unwrap();
        let claude_client = Arc::new(ClaudeClient::new(&mock_config()).unwrap());
        let store = InMemorySessionStore::new();

        let handler = MessageHandler {
            api_client,
            claude_client: claude_client.clone(),
            session_store: Arc::new(store.clone()),
            config: config.clone(),
        };

        // Empty allowed_channels = allow all
        assert!(handler.is_channel_allowed("C12345678"));

        // Specific allow list
        let config_with_allow = HandlerConfig {
            allowed_channels: vec!["C12345678".to_string()],
            ..Default::default()
        };

        let handler_with_allow = MessageHandler {
            api_client: SlackApiClient::new("xoxb-test").unwrap(),
            claude_client,
            session_store: Arc::new(store),
            config: config_with_allow,
        };
        assert!(handler_with_allow.is_channel_allowed("C12345678"));
        assert!(!handler_with_allow.is_channel_allowed("C87654321"));
    }
}
