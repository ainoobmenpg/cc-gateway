//! Signal Bot implementation
//!
//! Main entry point for the Signal Gateway

use std::sync::Arc;
use std::time::Duration;

use tokio::time::interval;
use tracing::{error, info, warn};

use cc_core::{ClaudeClient, Config};

use crate::api::SignalApiClient;
use crate::error::{Result, SignalError};
use crate::handler::{HandlerConfig, MessageHandler};
use crate::session::InMemorySessionStore;
use crate::types::GroupInfo;

/// Signal Bot configuration
#[derive(Clone, Debug)]
pub struct SignalBotConfig {
    /// Signal CLI REST API base URL
    pub api_url: String,
    /// Bot phone number
    pub phone_number: String,
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
    /// Allowed senders
    pub allowed_senders: Vec<String>,
    /// System prompt for Claude
    pub system_prompt: Option<String>,
}

impl Default for SignalBotConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8080".to_string(),
            phone_number: String::new(),
            poll_interval_secs: 5,
            allowed_senders: Vec::new(),
            system_prompt: None,
        }
    }
}

/// Signal Bot for Claude Code Gateway
pub struct SignalBot {
    bot_config: SignalBotConfig,
    api_client: SignalApiClient,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    handler_config: HandlerConfig,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl SignalBot {
    /// Create a new Signal bot
    pub fn new(
        bot_config: SignalBotConfig,
        _config: Config,
        claude_client: ClaudeClient,
    ) -> Result<Self> {
        let api_client = SignalApiClient::new(&bot_config.api_url, &bot_config.phone_number)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        let handler_config = HandlerConfig {
            allowed_senders: bot_config.allowed_senders.clone(),
            max_message_length: 2000,
            system_prompt: bot_config.system_prompt.clone().unwrap_or_else(|| {
                "You are a helpful assistant. Respond concisely as this is a messaging app. Respond in the same language as the user's question.".to_string()
            }),
        };

        // Start session cleanup task
        let store_clone = session_store.clone();
        tokio::spawn(async move {
            if let Err(e) = store_clone.start_cleanup_task().await {
                error!("Session cleanup task failed: {}", e);
            }
        });

        Ok(Self {
            bot_config,
            api_client,
            claude_client: Arc::new(claude_client),
            session_store,
            handler_config,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Create with shared Claude client
    pub fn with_client(
        bot_config: SignalBotConfig,
        _config: Config,
        claude_client: Arc<ClaudeClient>,
    ) -> Result<Self> {
        let api_client = SignalApiClient::new(&bot_config.api_url, &bot_config.phone_number)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        let handler_config = HandlerConfig {
            allowed_senders: bot_config.allowed_senders.clone(),
            max_message_length: 2000,
            system_prompt: bot_config.system_prompt.clone().unwrap_or_else(|| {
                "You are a helpful assistant. Respond concisely as this is a messaging app. Respond in the same language as the user's question.".to_string()
            }),
        };

        // Start session cleanup task
        let store_clone = session_store.clone();
        tokio::spawn(async move {
            if let Err(e) = store_clone.start_cleanup_task().await {
                error!("Session cleanup task failed: {}", e);
            }
        });

        Ok(Self {
            bot_config,
            api_client,
            claude_client,
            session_store,
            handler_config,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Get the session store
    pub fn session_store(&self) -> Arc<InMemorySessionStore> {
        self.session_store.clone()
    }

    /// Check if the bot is connected to Signal CLI
    pub async fn health_check(&self) -> Result<bool> {
        self.api_client.health_check().await
    }

    /// Send a message to a recipient (one-way, no session)
    pub async fn send_message(&self, recipient: &str, message: &str) -> Result<()> {
        self.api_client.send_message(recipient, message).await?;
        Ok(())
    }

    /// Get list of groups
    pub async fn get_groups(&self) -> Result<Vec<GroupInfo>> {
        self.api_client.get_groups().await
    }

    /// Start the bot (blocking)
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            warn!("Bot is already running");
            return Ok(());
        }

        info!(
            "Starting Signal bot for {} (poll interval: {}s)",
            self.bot_config.phone_number, self.bot_config.poll_interval_secs
        );

        // Check connection
        if !self.health_check().await? {
            return Err(SignalError::NotAvailable);
        }

        let handler = Arc::new(MessageHandler::new(
            self.api_client.clone(),
            self.claude_client.clone(),
            self.session_store.clone(),
            self.handler_config.clone(),
        ));

        let mut poll_interval = interval(Duration::from_secs(self.bot_config.poll_interval_secs));
        let mut last_timestamp: u64 = 0;

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            poll_interval.tick().await;

            match self.poll_messages(&handler, &mut last_timestamp).await {
                Ok(count) => {
                    if count > 0 {
                        info!("Processed {} new messages", count);
                    }
                }
                Err(e) => {
                    error!("Error polling messages: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Stop the bot
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Signal bot stopped");
    }

    /// Poll for new messages
    async fn poll_messages(
        &self,
        handler: &Arc<MessageHandler>,
        last_timestamp: &mut u64,
    ) -> Result<usize> {
        let messages = self.api_client.receive_messages().await?;

        let mut processed = 0;
        for msg in messages {
            // Only process messages newer than the last seen
            if msg.timestamp > *last_timestamp {
                *last_timestamp = msg.timestamp;

                if let Err(e) = handler.process_message(&msg).await {
                    error!("Error processing message: {:?}", e);
                }
                processed += 1;
            }
        }

        Ok(processed)
    }

    /// Run the bot with shutdown signal
    pub async fn run(&self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        if !self.health_check().await? {
            return Err(SignalError::NotAvailable);
        }

        let handler = Arc::new(MessageHandler::new(
            self.api_client.clone(),
            self.claude_client.clone(),
            self.session_store.clone(),
            self.handler_config.clone(),
        ));

        let mut poll_interval = interval(Duration::from_secs(self.bot_config.poll_interval_secs));
        let mut last_timestamp: u64 = 0;

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Received shutdown signal");
                    break;
                }
                _ = poll_interval.tick() => {
                    if let Err(e) = self.poll_messages(&handler, &mut last_timestamp).await {
                        error!("Error polling messages: {:?}", e);
                    }
                }
            }
        }

        info!("Signal bot stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_config() -> Config {
        Config {
            llm: cc_core::LlmConfig {
                api_key: "test-key".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                provider: cc_core::LlmProvider::Claude,
                base_url: None,
            },
            claude_api_key: "test-key".to_string(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            discord_token: None,
            admin_user_ids: vec![],
            api: cc_core::ApiConfig::default(),
            api_key: None,
            memory: cc_core::MemoryConfig::default(),
            mcp: cc_core::McpConfig::default(),
            scheduler: cc_core::SchedulerConfig::default(),
        }
    }

    #[test]
    fn test_bot_config_default() {
        let config = SignalBotConfig::default();
        assert_eq!(config.api_url, "http://localhost:8080");
        assert!(config.phone_number.is_empty());
        assert_eq!(config.poll_interval_secs, 5);
    }

    #[tokio::test]
    async fn test_bot_creation() {
        let bot_config = SignalBotConfig {
            phone_number: "+1234567890".to_string(),
            ..Default::default()
        };
        let config = mock_config();
        let client = ClaudeClient::new(&config).unwrap();

        let bot = SignalBot::new(bot_config, config, client);
        assert!(bot.is_ok());
    }
}
