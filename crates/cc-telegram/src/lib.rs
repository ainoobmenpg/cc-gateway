//! cc-telegram: Telegram Bot for cc-gateway
//!
//! This crate provides Telegram bot integration for cc-gateway,
//! allowing users to interact with Claude through Telegram.

pub mod bot;
pub mod commands;
pub mod error;
pub mod session;

pub use bot::TelegramBot;
pub use commands::BotState;
pub use error::{Result, TelegramError};
pub use session::InMemorySessionStore;
