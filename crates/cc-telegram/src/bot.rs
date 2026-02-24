//! Telegram bot implementation

use std::sync::Arc;

use teloxide::{dispatching::UpdateFilterExt, prelude::*, utils::command::BotCommands};
use tracing::info;

use cc_core::ClaudeClient;

use crate::commands::{handle_ask, handle_clear, handle_help, BotState};
use crate::error::Result;
use crate::session::InMemorySessionStore;

/// Telegram bot commands
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "cc-gateway Telegram Bot Commands"
)]
enum Command {
    #[command(description = "Ask Claude a question")]
    Ask(String),
    #[command(description = "Clear conversation history")]
    Clear,
    #[command(description = "Show help message")]
    Help,
}

/// Telegram bot wrapper
pub struct TelegramBot {
    bot: Bot,
    state: Arc<BotState>,
}

impl TelegramBot {
    /// Create a new Telegram bot
    pub fn new(token: &str, claude_client: Arc<ClaudeClient>, admin_user_ids: Vec<i64>) -> Self {
        let bot = Bot::new(token);
        let session_store = Arc::new(InMemorySessionStore::new());

        let state = Arc::new(BotState {
            claude_client,
            session_store,
            admin_user_ids,
        });

        Self { bot, state }
    }

    /// Start the bot
    pub async fn start(self) -> Result<()> {
        info!("Starting Telegram bot...");

        let command_handler = Update::filter_message()
            .filter_command::<Command>()
            .endpoint(|bot: Bot, msg: Message, cmd: Command, state: Arc<BotState>| async move {
                match cmd {
                    Command::Ask(question) => handle_ask(bot, msg, state, question).await,
                    Command::Clear => handle_clear(bot, msg, state).await,
                    Command::Help => handle_help(bot, msg).await,
                }
            });

        Dispatcher::builder(self.bot, command_handler)
            .dependencies(dptree::deps![self.state])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing() {
        // Test that commands can be parsed
        let cmd = Command::Ask("Hello".to_string());
        assert!(matches!(cmd, Command::Ask(_)));

        let cmd = Command::Clear;
        assert!(matches!(cmd, Command::Clear));

        let cmd = Command::Help;
        assert!(matches!(cmd, Command::Help));
    }
}
