//! Discord event handler implementation

use serenity::all::{
    async_trait, Context, EventHandler, Interaction, Ready,
};
use serenity::model::prelude::*;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use cc_core::{ClaudeClient, Message};

use crate::commands;
use crate::session::InMemorySessionStore;

/// Discord event handler
pub struct Handler {
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    admin_user_ids: Vec<String>,
}

impl Handler {
    /// Create a new handler
    pub fn new(
        claude_client: Arc<ClaudeClient>,
        session_store: Arc<InMemorySessionStore>,
        admin_user_ids: Vec<String>,
    ) -> Self {
        Self {
            claude_client,
            session_store,
            admin_user_ids,
        }
    }

    /// Check if user is an admin
    fn is_admin(&self, user_id: &UserId) -> bool {
        let user_id_str = user_id.to_string();
        self.admin_user_ids.is_empty() || self.admin_user_ids.contains(&user_id_str)
    }

    /// Check if bot is mentioned in the message
    fn is_mentioned(&self, msg: &serenity::model::prelude::Message, bot_id: UserId) -> bool {
        msg.mentions.iter().any(|user| user.id == bot_id)
    }

    /// Get the effective channel ID for session management
    fn get_session_key(&self, msg: &serenity::model::prelude::Message) -> String {
        // Use channel_id for session key (works for both regular channels and threads)
        msg.channel_id.to_string()
    }
}

#[async_trait]
impl EventHandler for Handler {
    /// Called when the bot is ready
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Register slash commands globally
        if let Err(e) = commands::register_commands(&ctx).await {
            error!("Failed to register commands: {:?}", e);
        }
    }

    /// Called when a message is received
    async fn message(&self, ctx: Context, msg: serenity::model::prelude::Message) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        let bot_id = ctx.cache.current_user().id;

        // Check if message is a reply to the bot
        let is_reply_to_bot = msg
            .message_reference
            .as_ref()
            .and_then(|r| r.message_id)
            .is_some();

        let is_mention = self.is_mentioned(&msg, bot_id);
        let is_dm = msg.guild_id.is_none();

        // Only respond if mentioned, replied to, or in DM
        if !is_mention && !is_reply_to_bot && !is_dm {
            return;
        }

        // Check admin permissions
        if !self.is_admin(&msg.author.id) {
            debug!("Ignoring message from non-admin user: {}", msg.author.id);
            return;
        }

        // Clean the message (remove mentions)
        let content = msg.content.clone();
        let clean_content = content
            .replace(&format!("<@{}>", bot_id), "")
            .replace(&format!("<@!{}>", bot_id), "")
            .trim()
            .to_string();

        if clean_content.is_empty() {
            if let Err(e) = msg.reply(&ctx.http, "はい、何かお手伝いしましょうか？").await {
                warn!("Failed to send reply: {:?}", e);
            }
            return;
        }

        // Show typing indicator
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

        // Get session key (thread or channel)
        let session_key = self.get_session_key(&msg);

        // Get existing session or create new one
        let session = self.session_store.get_or_create(&session_key);

        // Build message history
        let mut messages: Vec<Message> = session.messages.clone();
        messages.push(Message::user(&clean_content));

        info!(
            "Processing message from {} in {}: {} (history: {} messages)",
            msg.author.name,
            session_key,
            clean_content,
            messages.len() - 1
        );

        // Build request with conversation history
        let mut request_builder = self
            .claude_client
            .request_builder()
            .system("You are a helpful assistant. Respond in the same language as the user's question. Keep track of the conversation context.")
            .max_tokens(4096);

        // Add conversation history (limit to last 20 messages to avoid token limits)
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

                // Update session with user message and assistant response
                self.session_store.add_message(&session_key, Message::user(&clean_content));
                self.session_store.add_message(&session_key, Message::assistant(&text));

                // Send response (Discord has 2000 char limit)
                if text.len() <= 2000 {
                    if let Err(e) = msg.reply(&ctx.http, &text).await {
                        error!("Failed to send reply: {:?}", e);
                    }
                } else {
                    // Split into chunks
                    self.send_long_message(&ctx, &msg, &text).await;
                }
            }
            Err(e) => {
                error!("Claude API error: {:?}", e);
                if let Err(e) = msg
                    .reply(&ctx.http, &format!("エラーが発生しました: {}", e))
                    .await
                {
                    error!("Failed to send error message: {:?}", e);
                }
            }
        }
    }

    /// Called when an interaction is received (slash commands, etc.)
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            commands::handle_command(&ctx, command, self.claude_client.clone(), self.session_store.clone()).await;
        }
    }
}

impl Handler {
    /// Send a long message split into chunks
    async fn send_long_message(
        &self,
        ctx: &Context,
        msg: &serenity::model::prelude::Message,
        text: &str,
    ) {
        // Try to split on sentence boundaries
        let chunks = split_message(text, 1900);

        for (i, chunk) in chunks.iter().enumerate() {
            let content = if i == 0 {
                chunk.clone()
            } else {
                format!("(続き {})\n{}", i + 1, chunk)
            };

            if let Err(e) = msg.reply(&ctx.http, &content).await {
                error!("Failed to send reply chunk {}: {:?}", i, e);
                break;
            }

            // Small delay between chunks to avoid rate limiting
            if i < chunks.len() - 1 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}

/// Split a message into chunks at sentence boundaries
fn split_message(text: &str, max_size: usize) -> Vec<String> {
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

        // Find a good break point
        let search_end = max_size.min(remaining.len());
        let chunk = &remaining[..search_end];

        // Try to break at sentence end (。 ! ? \n)
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
