//! Tool trait definition
//!
//! Defines the core trait for implementing tools that can be
//! executed by Claude API tool_use.

use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::Result;

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Output string from tool execution
    pub output: String,
    /// Whether the execution resulted in an error
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful tool result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            is_error: false,
        }
    }

    /// Create an error tool result
    pub fn error(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            is_error: true,
        }
    }
}

/// Tool trait for Claude API tool_use
///
/// Implement this trait to create custom tools that can be
/// executed when Claude requests them via tool_use.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool name (used in Claude API tool definitions)
    fn name(&self) -> &str;

    /// Get the tool description (shown to Claude when selecting tools)
    fn description(&self) -> &str;

    /// Get the JSON schema for the tool's input parameters
    fn input_schema(&self) -> JsonValue;

    /// Execute the tool with the given input
    ///
    /// # Arguments
    /// * `input` - JSON value containing the tool input parameters
    ///
    /// # Returns
    /// A `ToolResult` containing the output or error message
    async fn execute(&self, input: JsonValue) -> Result<ToolResult>;
}
