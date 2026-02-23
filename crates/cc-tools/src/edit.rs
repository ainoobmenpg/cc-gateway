//! Edit tool for making string replacements in files

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};
use tokio::fs;

/// Edit tool for making string replacements in files
pub struct EditTool;

impl EditTool {
    /// Create a new EditTool instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Perform exact string replacement in a file. The old_string must be unique."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "The text to search for (must be unique in the file)"
                },
                "new_string": {
                    "type": "string",
                    "description": "The text to replace it with"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: false)",
                    "default": false
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let path = input["path"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'path' parameter".to_string())
        })?;

        let old_string = input["old_string"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'old_string' parameter".to_string())
        })?;

        let new_string = input["new_string"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'new_string' parameter".to_string())
        })?;

        let replace_all = input["replace_all"].as_bool().unwrap_or(false);

        tracing::debug!(path = %path, replace_all = replace_all, "Editing file");

        // Read the file
        let content = match fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to read file '{}': {}", path, e)));
            }
        };

        // Count occurrences
        let count = content.matches(old_string).count();

        if count == 0 {
            return Ok(ToolResult::error(format!(
                "String not found in file: '{}'",
                old_string
            )));
        }

        if !replace_all && count > 1 {
            return Ok(ToolResult::error(format!(
                "Found {} occurrences of the string. Use 'replace_all: true' to replace all, or make the string more specific.",
                count
            )));
        }

        // Perform replacement
        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };

        // Write back
        match fs::write(path, new_content).await {
            Ok(()) => {
                let replaced_count = if replace_all { count } else { 1 };
                Ok(ToolResult::success(format!(
                    "Successfully replaced {} occurrence(s) in '{}'",
                    replaced_count, path
                )))
            }
            Err(e) => {
                Ok(ToolResult::error(format!("Failed to write file '{}': {}", path, e)))
            }
        }
    }
}

impl Default for EditTool {
    fn default() -> Self {
        Self::new()
    }
}
