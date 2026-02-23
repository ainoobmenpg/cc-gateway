//! /ask command - Ask Claude a question (poise implementation)

use tracing::info;

use cc_core::Message;

use crate::commands::Data;
use crate::error::Result;

/// Ask Claude a question
#[poise::command(slash_command, rename = "ask")]
pub async fn ask(
    ctx: poise::Context<'_, Data, crate::error::DiscordError>,
    #[description = "The question to ask Claude"] question: String,
    #[description = "Only visible to you (default: true)"]
    #[flag]
    ephemeral: bool,
) -> Result<()> {
    // Defer the response since Claude API may take time
    if ephemeral {
        ctx.defer_ephemeral().await?;
    } else {
        ctx.defer().await?;
    }

    if question.is_empty() {
        ctx.say("質問を入力してください。").await?;
        return Ok(());
    }

    info!("Processing /ask command: {}", question);

    // Get session key from channel
    let session_key = ctx.channel_id().to_string();

    // Get shared data
    let data = ctx.data();

    // Get existing session or create new one
    let session = data.session_store.get_or_create(&session_key);

    // Build message history
    let mut messages: Vec<Message> = session.messages.clone();
    messages.push(Message::user(&question));

    // Build request with conversation history
    let mut request_builder = data
        .claude_client
        .request_builder()
        .system("You are a helpful assistant. Respond in the same language as the user's question. Be concise and helpful.")
        .max_tokens(2048);

    // Add conversation history (limit to last 20 messages)
    let history_start = messages.len().saturating_sub(20);
    for message in messages.into_iter().skip(history_start) {
        request_builder = request_builder.message(message);
    }

    let request = request_builder.build();

    let response_text = match data.claude_client.messages(request).await {
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
                .add_message(&session_key, Message::user(&question));
            data.session_store
                .add_message(&session_key, Message::assistant(&text));

            // Truncate if too long for Discord
            if text.len() > 1900 {
                format!("{}\n\n...(truncated)", &text[..1900])
            } else {
                text
            }
        }
        Err(e) => {
            format!("エラーが発生しました: {}", e)
        }
    };

    ctx.say(response_text).await?;

    Ok(())
}
