//! cc-mcp: MCP (Model Context Protocol) Integration
//!
//! MCPサーバーと通信し、MCPツールをcc-coreのTool traitに適合させる機能を提供します。

pub mod adapter;
pub mod client;
pub mod config;
pub mod registry;

pub use adapter::McpToolAdapter;
pub use client::{McpClient, McpTool};
pub use config::{McpConfig, McpServerConfig};
pub use registry::{McpRegistry, initialize_mcp_tools};
