//! Discord Bot implementation using poise Framework

use std::sync::Arc;
use tracing::info;

use cc_core::{ClaudeClient, Config};
use poise::serenity_prelude as serenity;
use serenity::FullEvent as Event;

use crate::commands::{get_commands, Data};
use crate::error::{DiscordError, Result};
use crate::handler::handle_message;
use crate::session::InMemorySessionStore;

/// Discord Bot for Claude Code Gateway
pub struct DiscordBot {
    config: Config,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
}

impl DiscordBot {
    /// Create a new Discord bot
    pub fn new(
        config: Config,
        claude_client: ClaudeClient,
        session_store: Arc<InMemorySessionStore>,
    ) -> Result<Self> {
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
            if let Err(e) = store_clone.start_cleanup_task().await {
                tracing::error!("Session cleanup task failed: {}", e);
            }
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
        let token = self
            .config
            .discord_token
            .as_ref()
            .ok_or(DiscordError::TokenNotSet)?;

        // Set up gateway intents
        // - GUILD_MESSAGES: Receive messages in guild channels
        // - DIRECT_MESSAGES: Receive direct messages
        // - MESSAGE_CONTENT: Read message content (privileged intent)
        let intents = serenity::GatewayIntents::GUILD_MESSAGES
            | serenity::GatewayIntents::DIRECT_MESSAGES
            | serenity::GatewayIntents::MESSAGE_CONTENT;

        info!("Starting Discord bot with poise framework...");

        // Create user data
        let data = Data {
            claude_client: self.claude_client.clone(),
            session_store: self.session_store.clone(),
            admin_user_ids: self.config.admin_user_ids.clone(),
        };

        // Build poise framework
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: get_commands(),
                event_handler: |ctx, event, _framework, data| {
                    Box::pin(async move {
                        if let Event::Message { new_message } = event {
                            if let Err(e) = handle_message(ctx, new_message, data).await {
                                tracing::error!("Error handling message: {:?}", e);
                            }
                        }
                        Ok(())
                    })
                },
                ..Default::default()
            })
            .setup(|ctx, ready, framework| {
                Box::pin(async move {
                    info!("{} is connected!", ready.user.name);

                    // Register slash commands globally
                    poise::builtins::register_globally(ctx, &framework.options().commands)
                        .await?;

                    Ok(data)
                })
            })
            .build();

        // Build and start the client
        let mut client = serenity::ClientBuilder::new(token, intents)
            .framework(framework)
            .await?;

        client.start().await?;

        Ok(())
    }
}
