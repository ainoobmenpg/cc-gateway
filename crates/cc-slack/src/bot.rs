//! Slack Bot implementation
//!
//! Main entry point for the Slack Gateway

use std::sync::Arc;

use tracing::{error, info};

use cc_core::{ClaudeClient, Config};

use crate::api::SlackApiClient;
use crate::error::{Result, SlackError};
use crate::handler::{HandlerConfig, MessageHandler};
use crate::session::InMemorySessionStore;
use crate::socket::SocketModeClient;
use crate::types::{SlackEvent, SlackMessage};

/// Slack Bot configuration
#[derive(Clone, Debug, Default)]
pub struct SlackBotConfig {
    /// Bot token (xoxb-...)
    pub bot_token: String,
    /// App-level token (xapp-...) for Socket Mode
    pub app_token: Option<String>,
    /// Allowed channels (empty = all)
    pub allowed_channels: Vec<String>,
    /// Allowed users (empty = all)
    pub allowed_users: Vec<String>,
    /// System prompt for Claude
    pub system_prompt: Option<String>,
}

/// Slack Bot for Claude Code Gateway
pub struct SlackBot {
    bot_config: SlackBotConfig,
    api_client: SlackApiClient,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    handler_config: HandlerConfig,
}

impl SlackBot {
    /// Create a new Slack bot
    pub fn new(
        bot_config: SlackBotConfig,
        _config: Config,
        claude_client: ClaudeClient,
    ) -> Result<Self> {
        if bot_config.bot_token.is_empty() {
            return Err(SlackError::TokenNotConfigured);
        }

        let api_client = SlackApiClient::new(&bot_config.bot_token)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        let handler_config = HandlerConfig {
            allowed_channels: bot_config.allowed_channels.clone(),
            allowed_users: bot_config.allowed_users.clone(),
            bot_user_id: None,
            system_prompt: bot_config.system_prompt.clone().unwrap_or_else(|| {
                "You are a helpful assistant. Respond concisely. Use Slack markdown formatting when appropriate.".to_string()
            }),
            max_message_length: 3500,
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
        })
    }

    /// Create with shared Claude client
    pub fn with_client(
        bot_config: SlackBotConfig,
        _config: Config,
        claude_client: Arc<ClaudeClient>,
    ) -> Result<Self> {
        if bot_config.bot_token.is_empty() {
            return Err(SlackError::TokenNotConfigured);
        }

        let api_client = SlackApiClient::new(&bot_config.bot_token)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        let handler_config = HandlerConfig {
            allowed_channels: bot_config.allowed_channels.clone(),
            allowed_users: bot_config.allowed_users.clone(),
            bot_user_id: None,
            system_prompt: bot_config.system_prompt.clone().unwrap_or_else(|| {
                "You are a helpful assistant. Respond concisely. Use Slack markdown formatting when appropriate.".to_string()
            }),
            max_message_length: 3500,
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
        })
    }

    /// Get the session store
    pub fn session_store(&self) -> Arc<InMemorySessionStore> {
        self.session_store.clone()
    }

    /// Test the Slack API connection
    pub async fn test_connection(&self) -> Result<String> {
        let auth = self.api_client.auth_test().await?;
        info!("Connected to Slack as {} in team {}", auth.user, auth.team);
        Ok(auth.team)
    }

    /// Send a message to a channel
    pub async fn send_message(&self, channel: &str, text: &str) -> Result<()> {
        self.api_client.send_message(channel, text, None).await?;
        Ok(())
    }

    /// Get list of channels
    pub async fn get_channels(&self) -> Result<Vec<crate::types::SlackChannel>> {
        self.api_client.conversations_list(Some("public_channel,private_channel,mpim,im")).await
    }

    /// Start the bot using Socket Mode
    pub async fn start(&self) -> Result<()> {
        let app_token = self.bot_config.app_token.as_ref()
            .ok_or_else(|| SlackError::Config("App token required for Socket Mode".to_string()))?;

        info!("Starting Slack bot with Socket Mode");

        // Test connection first
        let auth = self.api_client.auth_test().await?;
        info!("Connected to Slack as {} in team {}", auth.user, auth.team);

        // Create handler
        let mut handler_config = self.handler_config.clone();
        handler_config.bot_user_id = auth.bot_id;

        let handler = Arc::new(MessageHandler::new(
            self.api_client.clone(),
            self.claude_client.clone(),
            self.session_store.clone(),
            handler_config,
        ));

        // Start Socket Mode
        let socket_client = SocketModeClient::new(app_token, self.api_client.clone());

        socket_client.start(move |event: SlackEvent| {
            // Only handle message events
            if event.event_type == "message" {
                let msg = SlackMessage {
                    channel: event.channel.unwrap_or_default(),
                    user: event.user,
                    text: event.text.unwrap_or_default(),
                    ts: event.ts.unwrap_or_default(),
                    thread_ts: event.thread_ts,
                    bot_id: event.bot_id,
                    subtype: event.subtype,
                };

                // We need to handle this in an async context
                // For simplicity, we'll spawn a task
                let handler_clone = handler.clone();
                tokio::spawn(async move {
                    if let Err(e) = handler_clone.process_message(&msg).await {
                        error!("Error processing message: {:?}", e);
                    }
                });
            }
            Ok(())
        }).await
    }

    /// Run the bot with shutdown signal
    pub async fn run(&self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        let app_token = self.bot_config.app_token.as_ref()
            .ok_or_else(|| SlackError::Config("App token required for Socket Mode".to_string()))?;

        info!("Starting Slack bot with Socket Mode");

        // Test connection first
        let auth = self.api_client.auth_test().await?;
        info!("Connected to Slack as {} in team {}", auth.user, auth.team);

        // Create handler
        let mut handler_config = self.handler_config.clone();
        handler_config.bot_user_id = auth.bot_id.clone();

        let handler = Arc::new(MessageHandler::new(
            self.api_client.clone(),
            self.claude_client.clone(),
            self.session_store.clone(),
            handler_config,
        ));

        // Start Socket Mode in a task
        let socket_client = SocketModeClient::new(app_token, self.api_client.clone());
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let running_clone = running.clone();

        let socket_task = tokio::spawn(async move {
            if running_clone.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = socket_client.start(move |event: SlackEvent| {
                    if event.event_type == "message" {
                        let msg = SlackMessage {
                            channel: event.channel.unwrap_or_default(),
                            user: event.user,
                            text: event.text.unwrap_or_default(),
                            ts: event.ts.unwrap_or_default(),
                            thread_ts: event.thread_ts,
                            bot_id: event.bot_id,
                            subtype: event.subtype,
                        };

                        let handler_clone = handler.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handler_clone.process_message(&msg).await {
                                error!("Error processing message: {:?}", e);
                            }
                        });
                    }
                    Ok(())
                }).await;
            }
        });

        // Wait for shutdown signal
        let _ = shutdown.recv().await;
        running.store(false, std::sync::atomic::Ordering::SeqCst);
        socket_task.abort();

        info!("Slack bot stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_config_default() {
        let config = SlackBotConfig::default();
        assert!(config.bot_token.is_empty());
        assert!(config.app_token.is_none());
        assert!(config.allowed_channels.is_empty());
    }

    #[test]
    fn test_bot_creation_fails_without_token() {
        let config = SlackBotConfig::default();
        let core_config = create_test_config();
        let client = ClaudeClient::new(&core_config).unwrap();

        let result = SlackBot::new(config, core_config, client);
        assert!(result.is_err());
    }

    fn create_test_config() -> Config {
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
}
