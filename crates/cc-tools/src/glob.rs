//! Glob tool for file pattern matching

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};

/// Glob tool for file pattern matching
pub struct GlobTool;

impl GlobTool {
    /// Create a new GlobTool instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern (e.g., '**/*.rs')"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern to match files against (e.g., '**/*.rs')"
                },
                "path": {
                    "type": "string",
                    "description": "The base directory to search from (default: current directory)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let pattern = input["pattern"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'pattern' parameter".to_string())
        })?;

        let base_path = input["path"].as_str().unwrap_or(".");

        tracing::debug!(pattern = %pattern, base_path = %base_path, "Globbing files");

        let full_pattern = if base_path == "." {
            pattern.to_string()
        } else {
            format!("{}/{}", base_path.trim_end_matches('/'), pattern)
        };

        match glob::glob(&full_pattern) {
            Ok(paths) => {
                let mut results = Vec::new();
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            results.push(path.display().to_string());
                        }
                        Err(e) => {
                            tracing::warn!("Glob error: {}", e);
                        }
                    }
                }

                if results.is_empty() {
                    Ok(ToolResult::success("No files found matching the pattern".to_string()))
                } else {
                    Ok(ToolResult::success(results.join("\n")))
                }
            }
            Err(e) => {
                Ok(ToolResult::error(format!("Invalid glob pattern: {}", e)))
            }
        }
    }
}

impl Default for GlobTool {
    fn default() -> Self {
        Self::new()
    }
}
