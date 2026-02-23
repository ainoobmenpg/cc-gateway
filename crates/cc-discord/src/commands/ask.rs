//! /ask command - Ask Claude a question

use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption,
    CommandDataOptionValue,
};
use std::sync::Arc;
use tracing::info;

use cc_core::{ClaudeClient, Message};

use crate::session::InMemorySessionStore;

/// Register the /ask command
pub fn register_ask_command(command: CreateCommand) -> CreateCommand {
    command
        .name("ask")
        .description("Ask Claude a question")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "question",
                "The question to ask Claude",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Boolean,
                "ephemeral",
                "Only visible to you (default: true)",
            )
            .required(false),
        )
}

/// Run the /ask command
pub async fn run(
    interaction: &CommandInteraction,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
) -> String {
    // Extract the question from options
    let question = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == "question")
        .and_then(|opt| {
            if let CommandDataOptionValue::String(s) = &opt.value {
                Some(s.as_str())
            } else {
                None
            }
        })
        .unwrap_or("No question provided");

    if question.is_empty() {
        return "質問を入力してください。".to_string();
    }

    info!("Processing /ask command: {}", question);

    // Get session key from channel
    let session_key = interaction.channel_id.to_string();

    // Get existing session or create new one
    let session = session_store.get_or_create(&session_key);

    // Build message history
    let mut messages: Vec<Message> = session.messages.clone();
    messages.push(Message::user(question));

    // Build request with conversation history
    let mut request_builder = claude_client
        .request_builder()
        .system("You are a helpful assistant. Respond in the same language as the user's question. Be concise and helpful.")
        .max_tokens(2048);

    // Add conversation history (limit to last 20 messages)
    let history_start = messages.len().saturating_sub(20);
    for message in messages.into_iter().skip(history_start) {
        request_builder = request_builder.message(message);
    }

    let request = request_builder.build();

    match claude_client.messages(request).await {
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
            session_store.add_message(&session_key, Message::user(question));
            session_store.add_message(&session_key, Message::assistant(&text));

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
    }
}
