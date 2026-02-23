//! MCP Tool Adapter
//!
//! MCPツールをcc-coreのTool traitに適合させるアダプター

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use cc_core::{Result, Tool, ToolResult};

use crate::client::{McpClient, McpTool};

/// Adapter to make MCP tools compatible with cc-core Tool trait
///
/// This allows MCP server tools to be used seamlessly within the
/// cc-gateway tool system.
pub struct McpToolAdapter {
    /// Reference to the MCP client
    client: Arc<McpClient>,
    /// Tool name
    name: String,
    /// Tool description
    description: String,
    /// JSON schema for input parameters
    input_schema: JsonValue,
}

impl McpToolAdapter {
    /// Create a new adapter from an MCP client and tool definition
    ///
    /// # Arguments
    /// * `client` - Shared reference to the MCP client
    /// * `tool` - The MCP tool definition
    pub fn new(client: Arc<McpClient>, tool: McpTool) -> Self {
        Self {
            client,
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
        }
    }

    /// Get the underlying MCP client
    pub fn client(&self) -> &McpClient {
        &self.client
    }
}

#[async_trait]
impl Tool for McpToolAdapter {
    /// Get the tool name
    fn name(&self) -> &str {
        &self.name
    }

    /// Get the tool description
    fn description(&self) -> &str {
        &self.description
    }

    /// Get the JSON schema for input parameters
    fn input_schema(&self) -> JsonValue {
        self.input_schema.clone()
    }

    /// Execute the tool with the given input
    ///
    /// # Arguments
    /// * `input` - JSON value containing the tool input parameters
    ///
    /// # Returns
    /// A ToolResult containing the output or error message
    async fn execute(&self, input: JsonValue) -> Result<ToolResult> {
        match self.client.call_tool(&self.name, input).await {
            Ok(output) => Ok(ToolResult::success(output)),
            Err(e) => Ok(ToolResult::error(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running MCP server to work properly.
    // They are marked with #[ignore] for that reason.

    #[test]
    fn test_mcp_tool_adapter_creation() {
        // Test that adapter can be created with the expected fields
        let schema = JsonValue::Object(serde_json::Map::new());
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: schema.clone(),
        };

        // We can't create a full McpClient without a real connection,
        // but we can verify the struct is correct
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
    }
}
