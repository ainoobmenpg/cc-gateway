//! MCP Registry
//!
//! MCP クライアントとツールの一元管理

use std::sync::Arc;
use tracing::{error, info, warn};

use cc_core::{ToolManager, Tool};
use crate::{McpClient, McpConfig, McpToolAdapter};

/// Registry for managing all MCP clients
pub struct McpRegistry {
    /// Connected MCP clients
    clients: Vec<Arc<McpClient>>,
}

impl McpRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    /// Connect to a single MCP server
    ///
    /// # Arguments
    /// * `name` - Server name for logging
    /// * `command` - Command to start the MCP server
    ///
    /// # Returns
    /// The connected client on success
    async fn connect_server(&self, name: &str, command: &str) -> cc_core::Result<McpClient> {
        info!(server_name = name, command = command, "Connecting to MCP server");

        McpClient::connect(command).await.map_err(|e| {
            error!(server_name = name, error = %e, "Failed to connect to MCP server");
            e
        })
    }

    /// Initialize MCP clients and register tools
    ///
    /// This will:
    /// 1. Load the MCP configuration
    /// 2. Connect to all enabled servers
    /// 3. Register all tools from each server with the ToolManager
    ///
    /// # Arguments
    /// * `config` - MCP configuration
    /// * `tool_manager` - Tool manager to register tools with
    ///
    /// # Returns
    /// The registry on success, or None if no servers are configured
    pub async fn initialize(
        config: &McpConfig,
        tool_manager: &mut ToolManager,
    ) -> cc_core::Result<Option<Self>> {
        let enabled_servers = config.enabled_servers();

        if enabled_servers.is_empty() {
            info!("No MCP servers configured");
            return Ok(None);
        }

        let mut registry = Self::new();
        let mut total_tools = 0;

        for server_config in enabled_servers {
            match registry.connect_server(&server_config.name, &server_config.command).await {
                Ok(client) => {
                    let client = Arc::new(client);

                    // List and register tools
                    match client.list_tools().await {
                        Ok(tools) => {
                            let tool_count = tools.len();
                            info!(
                                server_name = server_config.name,
                                tool_count = tool_count,
                                "Discovered MCP tools"
                            );

                            for tool in tools {
                                let adapter = McpToolAdapter::new(Arc::clone(&client), tool);
                                let tool_name = adapter.name().to_string();
                                tool_manager.register(Arc::new(adapter));
                                info!(
                                    server_name = server_config.name,
                                    tool_name = tool_name,
                                    "Registered MCP tool"
                                );
                            }

                            total_tools += tool_count;
                            registry.clients.push(client);
                        }
                        Err(e) => {
                            warn!(
                                server_name = server_config.name,
                                error = %e,
                                "Failed to list tools from MCP server"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        server_name = server_config.name,
                        error = %e,
                        "Skipping MCP server due to connection error"
                    );
                    // Continue with other servers instead of failing
                }
            }
        }

        if registry.clients.is_empty() {
            warn!("No MCP servers connected successfully");
            Ok(None)
        } else {
            info!(
                server_count = registry.clients.len(),
                total_tools = total_tools,
                "MCP registry initialized"
            );
            Ok(Some(registry))
        }
    }

    /// Gracefully shutdown all MCP clients
    pub async fn shutdown(self) -> cc_core::Result<()> {
        info!("Shutting down MCP registry");

        for client in self.clients {
            match Arc::try_unwrap(client) {
                Ok(client) => {
                    if let Err(e) = client.shutdown().await {
                        warn!(error = %e, "Failed to shutdown MCP client");
                    }
                }
                Err(arc_client) => {
                    // If there are still references, we can't shutdown gracefully
                    warn!(
                        server_name = arc_client.server_name(),
                        "MCP client still has references, skipping graceful shutdown"
                    );
                }
            }
        }

        info!("MCP registry shutdown complete");
        Ok(())
    }

    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to initialize MCP tools
///
/// This loads the MCP configuration from the specified path,
/// connects to all enabled servers, and registers their tools.
///
/// # Arguments
/// * `config_path` - Optional path to MCP config file
/// * `tool_manager` - Tool manager to register tools with
///
/// # Returns
/// The registry on success (for shutdown purposes), or None
pub async fn initialize_mcp_tools(
    config_path: Option<&str>,
    tool_manager: &mut ToolManager,
) -> cc_core::Result<Option<McpRegistry>> {
    let mcp_config = match config_path {
        Some(path) => {
            info!(path = path, "Loading MCP configuration from file");
            McpConfig::from_json_file(path)?
        }
        None => {
            info!("No MCP configuration file specified, skipping MCP initialization");
            return Ok(None);
        }
    };

    McpRegistry::initialize(&mcp_config, tool_manager).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = McpRegistry::new();
        assert_eq!(registry.client_count(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = McpRegistry::default();
        assert_eq!(registry.client_count(), 0);
    }
}
