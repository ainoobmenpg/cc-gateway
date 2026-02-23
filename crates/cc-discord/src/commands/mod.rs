//! Slash commands for Discord bot (poise implementation)

mod ask;
mod clear;
mod help;

use std::sync::Arc;

use cc_core::ClaudeClient;

use crate::session::InMemorySessionStore;

/// User data stored and accessible in all command invocations
pub struct Data {
    pub claude_client: Arc<ClaudeClient>,
    pub session_store: Arc<InMemorySessionStore>,
    pub admin_user_ids: Vec<String>,
}

/// Error type for commands
pub type Error = crate::error::DiscordError;

/// Export commands for registration
pub use ask::ask;
pub use clear::clear;
pub use help::help;

/// Get all commands for registration
pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![ask(), clear(), help()]
}
