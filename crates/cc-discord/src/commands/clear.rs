//! /clear command - Clear conversation history

use serenity::all::{CommandInteraction, CreateCommand};
use std::sync::Arc;
use tracing::info;

use crate::session::InMemorySessionStore;

/// Register the /clear command
pub fn register_clear_command(command: CreateCommand) -> CreateCommand {
    command
        .name("clear")
        .description("Clear conversation history for this channel")
}

/// Run the /clear command
pub async fn run(
    interaction: &CommandInteraction,
    session_store: Arc<InMemorySessionStore>,
) -> String {
    let channel_id = interaction.channel_id.to_string();
    info!("Clearing conversation history for channel: {}", channel_id);

    // Clear the session
    let cleared = session_store.clear(&channel_id);

    if cleared {
        "会話履歴をクリアしました。".to_string()
    } else {
        "このチャンネルには会話履歴がありません。".to_string()
    }
}
