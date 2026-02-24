//! LINE Bot implementation
//!
//! Main entry point for the LINE Gateway

use std::sync::Arc;

use tracing::{error, info};

use cc_core::{ClaudeClient, Config};

use crate::api::LineApiClient;
use crate::error::{LineError, Result};
use crate::handler::{HandlerConfig, MessageHandler};
use crate::session::InMemorySessionStore;
use crate::webhook::{start_webhook_server, WebhookState};

/// LINE Bot configuration
#[derive(Clone, Debug, Default)]
pub struct LineBotConfig {
    /// Channel secret
    pub channel_secret: String,
    /// Channel access token
    pub channel_access_token: String,
    /// Webhook server port
    pub webhook_port: u16,
    /// Allowed users (empty = all)
    pub allowed_users: Vec<String>,
    /// System prompt for Claude
    pub system_prompt: Option<String>,
}

/// LINE Bot for Claude Code Gateway
pub struct LineBot {
    bot_config: LineBotConfig,
    api_client: LineApiClient,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    handler_config: HandlerConfig,
}

impl LineBot {
    /// Create a new LINE bot
    pub fn new(
        bot_config: LineBotConfig,
        _config: Config,
        claude_client: ClaudeClient,
    ) -> Result<Self> {
        if bot_config.channel_secret.is_empty() {
            return Err(LineError::Config("Channel secret not configured".to_string()));
        }
        if bot_config.channel_access_token.is_empty() {
            return Err(LineError::Config("Channel access token not configured".to_string()));
        }

        let api_client = LineApiClient::new(&bot_config.channel_access_token)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        let handler_config = HandlerConfig {
            allowed_users: bot_config.allowed_users.clone(),
            system_prompt: bot_config.system_prompt.clone().unwrap_or_else(|| {
                "You are a helpful assistant. Respond concisely as this is a messaging app. Respond in the same language as the user's question.".to_string()
            }),
            max_message_length: 5000,
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
        bot_config: LineBotConfig,
        _config: Config,
        claude_client: Arc<ClaudeClient>,
    ) -> Result<Self> {
        if bot_config.channel_secret.is_empty() {
            return Err(LineError::Config("Channel secret not configured".to_string()));
        }
        if bot_config.channel_access_token.is_empty() {
            return Err(LineError::Config("Channel access token not configured".to_string()));
        }

        let api_client = LineApiClient::new(&bot_config.channel_access_token)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        let handler_config = HandlerConfig {
            allowed_users: bot_config.allowed_users.clone(),
            system_prompt: bot_config.system_prompt.clone().unwrap_or_else(|| {
                "You are a helpful assistant. Respond concisely as this is a messaging app. Respond in the same language as the user's question.".to_string()
            }),
            max_message_length: 5000,
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

    /// Send a message to a user
    pub async fn send_message(&self, user_id: &str, text: &str) -> Result<()> {
        self.api_client.push_message(user_id, text).await
    }

    /// Get user profile
    pub async fn get_profile(&self, user_id: &str) -> Result<crate::types::LineProfile> {
        self.api_client.get_profile(user_id).await
    }

    /// Start the webhook server (blocking)
    pub async fn start(&self) -> Result<()> {
        info!("Starting LINE bot webhook server on port {}", self.bot_config.webhook_port);

        let handler = Arc::new(MessageHandler::new(
            self.api_client.clone(),
            self.claude_client.clone(),
            self.session_store.clone(),
            self.handler_config.clone(),
        ));

        let state = WebhookState {
            channel_secret: self.bot_config.channel_secret.clone(),
            handler,
        };

        start_webhook_server(state, self.bot_config.webhook_port).await
    }

    /// Run the bot with shutdown signal
    pub async fn run(&self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        info!("Starting LINE bot webhook server on port {}", self.bot_config.webhook_port);

        let handler = Arc::new(MessageHandler::new(
            self.api_client.clone(),
            self.claude_client.clone(),
            self.session_store.clone(),
            self.handler_config.clone(),
        ));

        let state = WebhookState {
            channel_secret: self.bot_config.channel_secret.clone(),
            handler,
        };

        let addr = format!("0.0.0.0:{}", self.bot_config.webhook_port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| LineError::Webhook(e.to_string()))?;

        info!("LINE webhook server listening on {}", addr);

        let app = crate::webhook::create_webhook_router(state);

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown.recv().await;
                info!("LINE bot shutting down");
            })
            .await
            .map_err(|e| LineError::Webhook(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_bot_creation_fails_without_credentials() {
        let bot_config = LineBotConfig::default();
        let core_config = create_test_config();
        let client = ClaudeClient::new(&core_config).unwrap();

        let result = LineBot::new(bot_config, core_config, client);
        assert!(result.is_err());
    }

    #[test]
    fn test_bot_config_default() {
        let config = LineBotConfig::default();
        assert!(config.channel_secret.is_empty());
        assert!(config.channel_access_token.is_empty());
        assert_eq!(config.webhook_port, 0);
    }
}
