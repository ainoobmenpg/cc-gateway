//! iMessage message handler implementation

use std::sync::Arc;
use tracing::{debug, error, info};

use cc_core::{ClaudeClient, Message};

use crate::error::Result;
use crate::script::{AppleScript, ReceivedMessage};
use crate::session::InMemorySessionStore;

/// Configuration for the message handler
#[derive(Clone, Debug)]
pub struct HandlerConfig {
    /// Allowed senders (phone numbers or emails). Empty = allow all.
    pub allowed_senders: Vec<String>,
    /// Maximum message length before splitting
    pub max_message_length: usize,
    /// System prompt for Claude
    pub system_prompt: String,
}

impl Default for HandlerConfig {
    fn default() -> Self {
        Self {
            allowed_senders: Vec::new(),
            max_message_length: 1000, // iMessage has no strict limit, but we split for readability
            system_prompt: "You are a helpful assistant. Respond concisely as this is a messaging app. Respond in the same language as the user's question.".to_string(),
        }
    }
}

/// Message handler for iMessage
pub struct MessageHandler {
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    config: HandlerConfig,
}

impl MessageHandler {
    /// Create a new message handler
    pub fn new(
        claude_client: Arc<ClaudeClient>,
        session_store: Arc<InMemorySessionStore>,
        config: HandlerConfig,
    ) -> Self {
        Self {
            claude_client,
            session_store,
            config,
        }
    }

    /// Process an incoming message
    pub async fn process_message(&self, msg: &ReceivedMessage) -> Result<()> {
        // Check sender authorization
        if !self.is_sender_allowed(&msg.sender) {
            debug!("Ignoring message from unauthorized sender: {}", msg.sender);
            return Ok(());
        }

        let content = msg.content.trim();
        if content.is_empty() {
            return Ok(());
        }

        // Handle commands
        if content.starts_with('/') {
            return self.handle_command(&msg.sender, content).await;
        }

        // Regular message processing
        self.process_with_claude(&msg.sender, content).await
    }

    /// Check if sender is allowed
    fn is_sender_allowed(&self, sender: &str) -> bool {
        if self.config.allowed_senders.is_empty() {
            return true;
        }
        self.config.allowed_senders.iter().any(|allowed| {
            sender == allowed || sender.contains(allowed) || allowed.contains(sender)
        })
    }

    /// Handle slash commands
    async fn handle_command(&self, sender: &str, content: &str) -> Result<()> {
        let parts: Vec<&str> = content.splitn(2, ' ').collect();
        let command = parts[0];
        let _args = parts.get(1).copied().unwrap_or("");

        match command {
            "/clear" | "/reset" => {
                if self.session_store.clear(sender) {
                    self.send_reply(sender, "セッションをリセットしました。").await?;
                } else {
                    self.send_reply(sender, "リセットするセッションがありません。").await?;
                }
            }
            "/help" => {
                let help_text = concat!(
                    "利用可能なコマンド:\n",
                    "/clear - セッションをリセット\n",
                    "/help - このヘルプを表示\n",
                    "/status - セッション状態を表示\n",
                    "\n",
                    "それ以外は通常のチャットとして動作します。"
                );
                self.send_reply(sender, help_text).await?;
            }
            "/status" => {
                let session = self.session_store.get(sender);
                let status = match session {
                    Some(s) => format!("メッセージ数: {}", s.message_count()),
                    None => "新しいセッション".to_string(),
                };
                self.send_reply(sender, &status).await?;
            }
            _ => {
                // Unknown command, treat as regular message
                let clean_content = content.trim_start_matches('/');
                self.process_with_claude(sender, clean_content).await?;
            }
        }

        Ok(())
    }

    /// Process message with Claude API
    async fn process_with_claude(&self, sender: &str, content: &str) -> Result<()> {
        info!("Processing message from {}: {}", sender, content);

        // Get or create session
        let session = self.session_store.get_or_create(sender);

        // Build message history
        let mut messages: Vec<Message> = session.messages.clone();
        messages.push(Message::user(content));

        // Build request with conversation history
        let mut request_builder = self
            .claude_client
            .request_builder()
            .system(&self.config.system_prompt)
            .max_tokens(1024);

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
                    .add_message(sender, Message::user(content));
                self.session_store
                    .add_message(sender, Message::assistant(&text));

                // Send response
                self.send_reply(sender, &text).await?;
            }
            Err(e) => {
                error!("Claude API error: {:?}", e);
                self.send_reply(sender, &format!("エラーが発生しました: {}", e))
                    .await?;
            }
        }

        Ok(())
    }

    /// Send a reply via iMessage
    async fn send_reply(&self, sender: &str, text: &str) -> Result<()> {
        // Split message if necessary
        if text.len() <= self.config.max_message_length {
            AppleScript::send_message(sender, text)?;
        } else {
            let chunks = self.split_message(text, self.config.max_message_length);
            for (i, chunk) in chunks.iter().enumerate() {
                let content = if i == 0 {
                    chunk.clone()
                } else {
                    format!("(続き {})\n{}", i + 1, chunk)
                };

                // Small delay between messages
                if i > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }

                if let Err(e) = AppleScript::send_message(sender, &content) {
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
                .rfind("。")
                .or_else(|| chunk.rfind("!"))
                .or_else(|| chunk.rfind("?"))
                .or_else(|| chunk.rfind("\n\n"))
                .or_else(|| chunk.rfind("\n"))
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

    /// Helper function to create a mock config for testing
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
    fn test_split_message() {
        let config = mock_config();
        let claude_client = Arc::new(ClaudeClient::new(&config).unwrap());
        let handler = MessageHandler {
            claude_client,
            session_store: Arc::new(InMemorySessionStore::new()),
            config: HandlerConfig::default(),
        };

        let short = "Short message";
        let result = handler.split_message(short, 100);
        assert_eq!(result.len(), 1);

        let long = "This is a long message. It should be split. At sentence boundaries.";
        let result = handler.split_message(long, 30);
        assert!(result.len() > 1);
    }

    #[test]
    fn test_is_sender_allowed() {
        let config = mock_config();
        let claude_client = Arc::new(ClaudeClient::new(&config).unwrap());
        let mut handler_config = HandlerConfig::default();
        let store = InMemorySessionStore::new();
        let handler = MessageHandler {
            claude_client: claude_client.clone(),
            session_store: Arc::new(store.clone()),
            config: handler_config.clone(),
        };

        // Empty allowed_senders = allow all
        assert!(handler.is_sender_allowed("+819012345678"));

        // Specific allow list
        handler_config.allowed_senders = vec!["+819012345678".to_string()];
        let handler = MessageHandler {
            claude_client,
            session_store: Arc::new(store),
            config: handler_config,
        };
        assert!(handler.is_sender_allowed("+819012345678"));
        assert!(!handler.is_sender_allowed("+81998765432"));
    }

    #[test]
    fn test_handler_config_default() {
        let config = HandlerConfig::default();
        assert!(config.allowed_senders.is_empty());
        assert_eq!(config.max_message_length, 1000);
        assert!(config.system_prompt.contains("helpful assistant"));
    }
}
