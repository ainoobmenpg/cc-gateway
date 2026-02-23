//! cc-tools: Built-in tools for cc-gateway
//!
//! This crate provides built-in tools for Claude Code Gateway.

use cc_core::ToolManager;

pub mod bash;
pub mod read;
pub mod write;
pub mod edit;
pub mod glob;
pub mod grep;

pub use bash::BashTool;
pub use read::ReadTool;
pub use write::WriteTool;
pub use edit::EditTool;
pub use glob::GlobTool;
pub use grep::GrepTool;

use std::sync::Arc;

/// Register all default built-in tools with the tool manager
pub fn register_default_tools(manager: &mut ToolManager) {
    manager.register(Arc::new(BashTool));
    manager.register(Arc::new(ReadTool));
    manager.register(Arc::new(WriteTool));
    manager.register(Arc::new(EditTool));
    manager.register(Arc::new(GlobTool));
    manager.register(Arc::new(GrepTool));
}
