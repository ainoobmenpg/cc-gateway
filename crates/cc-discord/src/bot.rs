//! Discord Bot implementation using Serenity

use anyhow::Result;
use serenity::model::gateway::GatewayIntents;
use serenity::prelude::*;
use std::sync::Arc;
use tracing::info;

use cc_core::{ClaudeClient, Config};

use crate::handler::Handler;
use crate::session::InMemorySessionStore;

/// Discord Bot for Claude Code Gateway
pub struct DiscordBot {
    config: Config,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
}

impl DiscordBot {
    /// Create a new Discord Bot instance
    pub fn new(config: Config) -> Result<Self> {
        let claude_client = ClaudeClient::new(&config)?;
        let session_store = Arc::new(InMemorySessionStore::new());

        // Start session cleanup task
        let store_clone = session_store.clone();
        tokio::spawn(async move {
            store_clone.start_cleanup_task().await.unwrap();
        });

        Ok(Self {
            config,
            claude_client: Arc::new(claude_client),
            session_store,
        })
    }

    /// Create with shared Claude client
    pub fn with_client(config: Config, claude_client: Arc<ClaudeClient>) -> Self {
        let session_store = Arc::new(InMemorySessionStore::new());

        // Start session cleanup task
        let store_clone = session_store.clone();
        tokio::spawn(async move {
            store_clone.start_cleanup_task().await.unwrap();
        });

        Self {
            config,
            claude_client,
            session_store,
        }
    }

    /// Get the session store
    pub fn session_store(&self) -> Arc<InMemorySessionStore> {
        self.session_store.clone()
    }

    /// Start the Discord bot
    pub async fn start(&self) -> Result<()> {
        let token = self.config.discord_token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("DISCORD_BOT_TOKEN not set in configuration")
        })?;

        // Set up gateway intents
        // - GUILD_MESSAGES: Receive messages in guild channels
        // - DIRECT_MESSAGES: Receive direct messages
        // - MESSAGE_CONTENT: Read message content (privileged intent)
        let intents =
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT;

        info!("Starting Discord bot...");

        // Create the handler with shared components
        let handler = Handler::new(
            self.claude_client.clone(),
            self.session_store.clone(),
            self.config.admin_user_ids.clone(),
        );

        // Build and start the client
        let mut client = Client::builder(token, intents)
            .event_handler(handler)
            .await?;

        // Start listening for events
        client.start().await?;

        Ok(())
    }
}
