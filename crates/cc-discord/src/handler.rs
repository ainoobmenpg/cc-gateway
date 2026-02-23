//! Discord event handler implementation using poise Framework

use tracing::{debug, error, info, warn};

use cc_core::Message;

use crate::commands::Data;
use crate::error::Result;

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

/// Handle message events (for mentions, DMs, and replies)
pub async fn handle_message(
    ctx: &poise::serenity_prelude::Context,
    msg: &poise::serenity_prelude::Message,
    data: &Data,
) -> Result<()> {
    // Ignore messages from bots
    if msg.author.bot {
        return Ok(());
    }

    let bot_id = ctx.cache.current_user().id;

    // Check if message is a reply to the bot
    let is_reply_to_bot = msg
        .message_reference
        .as_ref()
        .and_then(|r| r.message_id)
        .is_some();

    let is_mention = msg.mentions.iter().any(|user| user.id == bot_id);
    let is_dm = msg.guild_id.is_none();

    // Only respond if mentioned, replied to, or in DM
    if !is_mention && !is_reply_to_bot && !is_dm {
        return Ok(());
    }

    // Check admin permissions
    let user_id_str = msg.author.id.to_string();
    let is_admin =
        data.admin_user_ids.is_empty() || data.admin_user_ids.contains(&user_id_str);
    if !is_admin {
        debug!("Ignoring message from non-admin user: {}", msg.author.id);
        return Ok(());
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
        return Ok(());
    }

    // Show typing indicator
    let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

    // Get session key (thread or channel)
    let session_key = msg.channel_id.to_string();

    // Get existing session or create new one
    let session = data.session_store.get_or_create(&session_key);

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
    let mut request_builder = data
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

    match data.claude_client.messages(request).await {
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
            data.session_store
                .add_message(&session_key, Message::user(&clean_content));
            data.session_store
                .add_message(&session_key, Message::assistant(&text));

            // Send response (Discord has 2000 char limit)
            if text.len() <= 2000 {
                if let Err(e) = msg.reply(&ctx.http, &text).await {
                    error!("Failed to send reply: {:?}", e);
                }
            } else {
                // Split into chunks
                let chunks = split_message(&text, 1900);
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

    Ok(())
}
