//! Slash commands for Discord bot

mod ask;
mod clear;
mod help;

use anyhow::Result;
use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tracing::{error, info};

use cc_core::ClaudeClient;

use crate::session::InMemorySessionStore;

pub use ask::register_ask_command;
pub use clear::register_clear_command;
pub use help::register_help_command;

/// Register all slash commands
pub async fn register_commands(ctx: &Context) -> Result<()> {
    // Register global commands
    let commands = vec![
        register_ask_command(CreateCommand::new("ask")),
        register_clear_command(CreateCommand::new("clear")),
        register_help_command(CreateCommand::new("help")),
    ];

    // Register commands globally
    let registered = Command::set_global_commands(&ctx.http, commands).await?;

    info!("Registered {} slash commands", registered.len());

    Ok(())
}

/// Handle slash command interactions
pub async fn handle_command(
    ctx: &Context,
    command: CommandInteraction,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
) {
    info!(
        "Received command: {} from {}",
        command.data.name, command.user.name
    );

    let response = match command.data.name.as_str() {
        "ask" => ask::run(&command, claude_client, session_store).await,
        "clear" => clear::run(&command, session_store).await,
        "help" => help::run(&command).await,
        _ => format!("Unknown command: {}", command.data.name),
    };

    // Send the response
    if let Err(e) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(&response)
                    .ephemeral(true),
            ),
        )
        .await
    {
        error!("Failed to send command response: {:?}", e);
    }
}
