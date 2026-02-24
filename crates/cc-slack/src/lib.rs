//! cc-slack: Slack Gateway for Claude Code
//!
//! Slack API を使用して Slack 経由で Claude API へのアクセスを提供します。
//! Socket Mode と Events API の両方をサポートします。

pub mod api;
pub mod bot;
pub mod error;
pub mod handler;
pub mod session;
pub mod socket;
pub mod types;

pub use bot::SlackBot;
pub use error::{Result, SlackError};
pub use session::InMemorySessionStore;
