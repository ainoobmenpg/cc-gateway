//! cc-core: Claude Code Gateway Core Library
//!
//! Claude APIとの通信、ツールシステム、セッション管理、
//! メモリシステムのコア機能を提供します。

pub mod config;
pub mod error;
pub mod llm;
pub mod memory;
pub mod session;
pub mod tool;

pub use config::{Config, LlmConfig, LlmProvider};
pub use error::{Error, Result};
pub use llm::{ClaudeClient, Message, MessageContent};
pub use memory::{Memory, MemoryStore};
pub use session::{Session, SessionManager, SessionStore};
pub use tool::{Tool, ToolManager, ToolResult};
