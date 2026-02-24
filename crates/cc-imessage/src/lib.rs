//! cc-imessage: iMessage Gateway for Claude Code
//!
//! macOS の Apple Script を使用して iMessage 経由で Claude API へのアクセスを提供します。
//! メッセージの送受信とセッション管理を行います。

pub mod bot;
pub mod error;
pub mod handler;
pub mod script;
pub mod session;
pub mod watcher;

pub use bot::IMessageBot;
pub use error::{IMessageError, Result};
pub use session::InMemorySessionStore;
