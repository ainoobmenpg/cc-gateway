//! MCP Client implementation
//!
//! rmcpを使用してMCPサーバーと通信するクライアント

use rmcp::{
    model::{CallToolRequestParams, Tool},
    service::{RoleClient, RunningService, ServiceExt},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::Value as JsonValue;
use tokio::process::Command;

use cc_core::Result;

/// MCP Tool information
#[derive(Debug, Clone)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for input parameters
    pub input_schema: JsonValue,
}

impl From<Tool> for McpTool {
    fn from(tool: Tool) -> Self {
        Self {
            name: tool.name.to_string(),
            description: tool
                .description
                .clone()
                .unwrap_or_default()
                .to_string(),
            input_schema: serde_json::to_value(&tool.input_schema).unwrap_or(JsonValue::Null),
        }
    }
}

/// MCP Client for communicating with MCP servers
pub struct McpClient {
    /// Inner rmcp running service
    service: RunningService<RoleClient, ()>,
    /// Server name for identification
    server_name: String,
}

impl McpClient {
    /// Connect to an MCP server via child process
    ///
    /// # Arguments
    /// * `command` - The command to start the MCP server (e.g., "uvx mcp-server-git")
    ///
    /// # Example
    /// ```ignore
    /// let client = McpClient::connect("uvx mcp-server-git").await?;
    /// ```
    pub async fn connect(command: &str) -> Result<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(cc_core::Error::Config("Empty command".into()));
        }

        let cmd = parts[0];
        let args = &parts[1..];

        // Start the child process and create transport
        let transport = TokioChildProcess::new(
            Command::new(cmd).configure(|c| {
                for arg in args {
                    c.arg(*arg);
                }
            }),
        )
        .map_err(|e| cc_core::Error::Mcp(format!("Failed to create transport: {}", e)))?;

        // Serve with unit type handler (client-only mode)
        let service = ()
            .serve(transport)
            .await
            .map_err(|e| cc_core::Error::Mcp(format!("Failed to connect: {}", e)))?;

        // Get server information
        let server_name = service
            .peer_info()
            .map(|info| info.server_info.name.clone().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            service,
            server_name,
        })
    }

    /// Get the server name
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// List available tools from the MCP server
    ///
    /// # Returns
    /// A vector of available tools with their schemas
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let result = self
            .service
            .list_tools(Default::default())
            .await
            .map_err(|e| cc_core::Error::Mcp(format!("Failed to list tools: {}", e)))?;

        let tools = result.tools.into_iter().map(McpTool::from).collect();
        Ok(tools)
    }

    /// Call a tool on the MCP server
    ///
    /// # Arguments
    /// * `name` - The name of the tool to call
    /// * `args` - The arguments to pass to the tool
    ///
    /// # Returns
    /// The tool execution result as a string
    pub async fn call_tool(&self, name: &str, args: JsonValue) -> Result<String> {
        let arguments = args.as_object().cloned();
        let name_str = name.to_string();

        let result = self
            .service
            .call_tool(CallToolRequestParams {
                meta: None,
                name: name_str.into(),
                arguments,
                task: None,
            })
            .await
            .map_err(|e| cc_core::Error::Mcp(format!("Tool call failed: {}", e)))?;

        // Extract text content from the result
        let output = result
            .content
            .into_iter()
            .filter_map(|c| {
                if let rmcp::model::RawContent::Text(text) = c.raw {
                    Some(text.text)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output)
    }

    /// Gracefully close the connection
    pub async fn shutdown(self) -> Result<()> {
        self.service
            .cancel()
            .await
            .map_err(|e| cc_core::Error::Mcp(format!("Shutdown failed: {}", e)))?;
        Ok(())
    }
}
