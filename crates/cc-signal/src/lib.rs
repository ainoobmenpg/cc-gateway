//! cc-signal: Signal Gateway for Claude Code
//!
//! signal-cli REST API を使用して Signal 経由で Claude API へのアクセスを提供します。
//! メッセージの送受信とセッション管理を行います。

pub mod api;
pub mod bot;
pub mod error;
pub mod handler;
pub mod session;
pub mod types;

pub use bot::SignalBot;
pub use error::{Result, SignalError};
pub use session::InMemorySessionStore;
