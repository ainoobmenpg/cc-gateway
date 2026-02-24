//! LINE message handler implementation

use std::sync::Arc;
use tracing::{debug, error, info};

use cc_core::{ClaudeClient, Message};

use crate::api::LineApiClient;
use crate::error::Result;
use crate::session::InMemorySessionStore;
use crate::types::LineEvent;

/// Configuration for the message handler
#[derive(Clone, Debug)]
pub struct HandlerConfig {
    /// Allowed users. Empty = allow all.
    pub allowed_users: Vec<String>,
    /// System prompt for Claude
    pub system_prompt: String,
    /// Maximum message length before splitting
    pub max_message_length: usize,
}

impl Default for HandlerConfig {
    fn default() -> Self {
        Self {
            allowed_users: Vec::new(),
            system_prompt: "You are a helpful assistant. Respond concisely as this is a messaging app. Respond in the same language as the user's question.".to_string(),
            max_message_length: 5000, // LINE has ~5000 char limit per message
        }
    }
}

/// Message handler for LINE
pub struct MessageHandler {
    api_client: LineApiClient,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    config: HandlerConfig,
}

impl MessageHandler {
    /// Create a new message handler
    pub fn new(
        api_client: LineApiClient,
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

    /// Process an incoming event
    pub async fn process_event(&self, event: &LineEvent) -> Result<()> {
        // Only handle message events
        if event.event_type != "message" {
            return Ok(());
        }

        // Get the message text
        let text = match &event.message {
            Some(msg) => msg.text.as_deref().unwrap_or(""),
            None => return Ok(()),
        };

        // Get the sender ID
        let sender_id = match &event.source.user_id {
            Some(id) => id,
            None => return Ok(()),
        };

        // Check user authorization
        if !self.is_user_allowed(sender_id) {
            debug!("Ignoring message from unauthorized user: {}", sender_id);
            return Ok(());
        }

        // Get reply token if available
        let reply_token = event.reply_token.as_deref();

        let content = text.trim();
        if content.is_empty() {
            return Ok(());
        }

        // Handle commands
        if content.starts_with('/') {
            return self.handle_command(sender_id, content, reply_token).await;
        }

        // Regular message processing
        self.process_with_claude(sender_id, content, reply_token).await
    }

    /// Check if user is allowed
    fn is_user_allowed(&self, user_id: &str) -> bool {
        if self.config.allowed_users.is_empty() {
            return true;
        }
        self.config.allowed_users.contains(&user_id.to_string())
    }

    /// Handle commands
    async fn handle_command(&self, sender_id: &str, content: &str, reply_token: Option<&str>) -> Result<()> {
        let command = content.split_whitespace().next().unwrap_or("");

        match command {
            "/clear" | "/reset" => {
                if self.session_store.clear(sender_id) {
                    self.send_reply(sender_id, "Session reset.", reply_token).await?;
                } else {
                    self.send_reply(sender_id, "No session to reset.", reply_token).await?;
                }
            }
            "/help" => {
                let help_text = concat!(
                    "Available commands:\n",
                    "/clear - Reset session\n",
                    "/help - Show this help\n",
                    "/status - Show session status\n",
                    "\n",
                    "Otherwise, just chat normally!"
                );
                self.send_reply(sender_id, help_text, reply_token).await?;
            }
            "/status" => {
                let session = self.session_store.get(sender_id);
                let status = match session {
                    Some(s) => format!("Message count: {}", s.message_count()),
                    None => "New session".to_string(),
                };
                self.send_reply(sender_id, &status, reply_token).await?;
            }
            _ => {
                // Unknown command, treat as regular message
                let clean_content = content.trim_start_matches('/');
                self.process_with_claude(sender_id, clean_content, reply_token).await?;
            }
        }

        Ok(())
    }

    /// Process message with Claude API
    async fn process_with_claude(&self, sender_id: &str, content: &str, reply_token: Option<&str>) -> Result<()> {
        info!("Processing message from {}: {}", sender_id, content);

        // Get or create session
        let session = self.session_store.get_or_create(sender_id);

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
                    .add_message(sender_id, Message::user(content));
                self.session_store
                    .add_message(sender_id, Message::assistant(&text));

                // Send response
                self.send_reply(sender_id, &text, reply_token).await?;
            }
            Err(e) => {
                error!("Claude API error: {:?}", e);
                self.send_reply(sender_id, &format!("Error: {}", e), reply_token)
                    .await?;
            }
        }

        Ok(())
    }

    /// Send a reply
    async fn send_reply(&self, sender_id: &str, text: &str, reply_token: Option<&str>) -> Result<()> {
        // Prefer reply API if we have a token, otherwise use push
        if let Some(token) = reply_token {
            if text.len() <= self.config.max_message_length {
                self.api_client.reply_message(token, text).await?;
            } else {
                // Split and use push for multiple messages
                let chunks = self.split_message(text, self.config.max_message_length);
                // First message via reply
                if let Some(first) = chunks.first() {
                    self.api_client.reply_message(token, first).await?;
                }
                // Rest via push
                for chunk in chunks.iter().skip(1) {
                    if let Err(e) = self.api_client.push_message(sender_id, chunk).await {
                        error!("Failed to push message: {:?}", e);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        } else {
            // Split message if necessary
            if text.len() <= self.config.max_message_length {
                self.api_client.push_message(sender_id, text).await?;
            } else {
                let chunks = self.split_message(text, self.config.max_message_length);
                self.api_client.push_messages(sender_id, &chunks).await?;
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
                .rfind("。")
                .or_else(|| chunk.rfind("\n\n"))
                .or_else(|| chunk.rfind("\n"))
                .or_else(|| chunk.rfind(". "))
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
        assert!(config.allowed_users.is_empty());
        assert_eq!(config.max_message_length, 5000);
    }

    #[test]
    fn test_split_message() {
        let config = HandlerConfig::default();
        let api_client = LineApiClient::new("test-token").unwrap();
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
}
