//! Tool manager for registering and executing tools

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value as JsonValue;

use crate::tool::{Tool, ToolResult};
use crate::llm::ToolDefinition;
use crate::Result;

/// Manager for registered tools
///
/// Handles tool registration, retrieval, and execution.
pub struct ToolManager {
    /// Registered tools indexed by name
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolManager {
    /// Create a new empty tool manager
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    ///
    /// If a tool with the same name already exists, it will be replaced.
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Get all registered tool definitions for Claude API
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition::new(t.name(), t.description(), t.input_schema()))
            .collect()
    }

    /// Execute a tool by name
    ///
    /// # Arguments
    /// * `name` - The name of the tool to execute
    /// * `input` - The input parameters for the tool
    ///
    /// # Errors
    /// Returns an error if the tool is not found or execution fails
    pub async fn execute(&self, name: &str, input: JsonValue) -> Result<ToolResult> {
        let tool = self.get(name).ok_or_else(|| {
            crate::Error::ToolExecution(format!("Unknown tool: {}", name))
        })?;
        tool.execute(input).await
    }

    /// Check if a tool is registered
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if no tools are registered
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Get all registered tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}
