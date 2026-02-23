//! Write tool for creating/overwriting files

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};
use tokio::fs;

/// Write tool for creating/overwriting files
pub struct WriteTool;

impl WriteTool {
    /// Create a new WriteTool instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it doesn't exist, overwrites if it does."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let path = input["path"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'path' parameter".to_string())
        })?;

        let content = input["content"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'content' parameter".to_string())
        })?;

        tracing::debug!(path = %path, content_len = content.len(), "Writing file");

        // Create parent directories if needed
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    return Ok(ToolResult::error(format!(
                        "Failed to create parent directories: {}",
                        e
                    )));
                }
            }
        }

        match fs::write(path, content).await {
            Ok(()) => {
                Ok(ToolResult::success(format!(
                    "Successfully wrote {} bytes to '{}'",
                    content.len(),
                    path
                )))
            }
            Err(e) => {
                Ok(ToolResult::error(format!("Failed to write file '{}': {}", path, e)))
            }
        }
    }
}

impl Default for WriteTool {
    fn default() -> Self {
        Self::new()
    }
}
