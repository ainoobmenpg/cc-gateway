//! cc-email: Email tools for cc-gateway
//!
//! This crate provides email sending and receiving capabilities.

pub mod error;
pub mod receive;
pub mod send;
pub mod tools;

pub use error::{EmailError, Result};
pub use receive::EmailReceiver;
pub use send::EmailSender;
pub use tools::{EmailListTool, EmailReadTool, EmailSendTool};
