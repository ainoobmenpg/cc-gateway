//! Read tool for reading file contents

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};
use tokio::fs;

/// Read tool for reading file contents
pub struct ReadTool;

impl ReadTool {
    /// Create a new ReadTool instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read a file from the filesystem. Supports line ranges for partial reading."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-indexed)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let path = input["path"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'path' parameter".to_string())
        })?;

        let offset = input["offset"].as_u64().unwrap_or(1) as usize;
        let limit = input["limit"].as_u64();

        tracing::debug!(path = %path, offset = offset, limit = ?limit, "Reading file");

        match fs::read_to_string(path).await {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();

                // Apply offset (1-indexed to 0-indexed)
                let start = if offset > 0 { offset - 1 } else { 0 };
                if start >= lines.len() {
                    return Ok(ToolResult::success("(empty or beyond file end)".to_string()));
                }

                let end = match limit {
                    Some(l) => (start + l as usize).min(lines.len()),
                    None => lines.len(),
                };

                let selected_lines: Vec<&str> = lines[start..end].to_vec();
                let result = selected_lines.join("\n");

                Ok(ToolResult::success(result))
            }
            Err(e) => {
                Ok(ToolResult::error(format!("Failed to read file '{}': {}", path, e)))
            }
        }
    }
}

impl Default for ReadTool {
    fn default() -> Self {
        Self::new()
    }
}
