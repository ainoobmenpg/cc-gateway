//! Tool system for Claude API tool_use
//!
//! This module provides the tool system for executing tools
//! requested by Claude API.

pub mod definition;
pub mod manager;
pub mod traits;

pub use definition::ToolDefinition;
pub use manager::ToolManager;
pub use traits::{Tool, ToolResult};
