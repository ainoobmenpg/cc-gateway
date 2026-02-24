//! cc-line: LINE Gateway for Claude Code
//!
//! LINE Messaging API を使用して LINE 経由で Claude API へのアクセスを提供します。
//! Webhook サーバーと API クライアントを実装します。

pub mod api;
pub mod bot;
pub mod error;
pub mod handler;
pub mod session;
pub mod types;
pub mod webhook;

pub use bot::LineBot;
pub use error::{LineError, Result};
pub use session::InMemorySessionStore;
