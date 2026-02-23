//! cc-discord: Discord Gateway for Claude Code
//!
//! Discord Botを通じてClaude APIへのアクセスを提供します。
//! Serenity 0.12を使用してDiscord Gatewayに接続します。

pub mod bot;
pub mod commands;
pub mod error;
pub mod handler;
pub mod session;

pub use bot::DiscordBot;
pub use error::{DiscordError, Result};
pub use session::InMemorySessionStore;
