//! MCP Configuration
//!
//! MCPサーバー設定の読み込みと管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name (used for identification)
    pub name: String,

    /// Command to start the MCP server
    /// Example: "uvx mcp-server-git" or "node /path/to/server.js"
    pub command: String,

    /// Environment variables to pass to the server
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether this server is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            env: HashMap::new(),
            enabled: true,
        }
    }
}

/// MCP configuration containing all server definitions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// List of MCP servers to connect
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

impl McpConfig {
    /// Create an empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a JSON file
    pub fn from_json_file(path: &str) -> cc_core::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| cc_core::Error::Config(format!("Failed to read MCP config: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| cc_core::Error::Config(format!("Invalid MCP config JSON: {}", e)))
    }

    /// Load configuration from a TOML file
    pub fn from_toml_file(path: &str) -> cc_core::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| cc_core::Error::Config(format!("Failed to read MCP config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| cc_core::Error::Config(format!("Invalid MCP config TOML: {}", e)))
    }

    /// Load configuration from environment variable (JSON format)
    pub fn from_env() -> cc_core::Result<Self> {
        let config_json = std::env::var("MCP_CONFIG")
            .map_err(|_| cc_core::Error::Config("MCP_CONFIG not set".to_string()))?;

        serde_json::from_str(&config_json)
            .map_err(|e| cc_core::Error::Config(format!("Invalid MCP_CONFIG JSON: {}", e)))
    }

    /// Get only enabled servers
    pub fn enabled_servers(&self) -> Vec<&McpServerConfig> {
        self.servers.iter().filter(|s| s.enabled).collect()
    }

    /// Add a server configuration
    pub fn add_server(&mut self, config: McpServerConfig) {
        self.servers.push(config);
    }

    /// Create a default configuration with common MCP servers
    pub fn with_defaults() -> Self {
        Self {
            servers: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpConfig::new();
        assert!(config.servers.is_empty());
    }

    #[test]
    fn test_from_json() {
        let json = r#"{
            "servers": [
                {
                    "name": "git",
                    "command": "uvx mcp-server-git",
                    "enabled": true
                }
            ]
        }"#;

        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].name, "git");
        assert_eq!(config.servers[0].command, "uvx mcp-server-git");
    }
}
