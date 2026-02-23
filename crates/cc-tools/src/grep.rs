//! Grep tool for content search in files

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde_json::{json, Value};

/// Grep tool for content search
pub struct GrepTool;

impl GrepTool {
    /// Create a new GrepTool instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for patterns in file contents using regular expressions"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "The file or directory to search in"
                },
                "glob": {
                    "type": "string",
                    "description": "File pattern to limit search (e.g., '*.rs')"
                },
                "ignore_case": {
                    "type": "boolean",
                    "description": "Case insensitive search (default: false)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let pattern = input["pattern"].as_str().ok_or_else(|| {
            cc_core::Error::ToolExecution("Missing 'pattern' parameter".to_string())
        })?;

        let path = input["path"].as_str().unwrap_or(".");
        let glob_pattern = input["glob"].as_str();
        let ignore_case = input["ignore_case"].as_bool().unwrap_or(false);

        tracing::debug!(pattern = %pattern, path = %path, glob = ?glob_pattern, ignore_case = ignore_case, "Grepping files");

        // Build ripgrep command
        let mut args = vec!["--no-heading", "--with-filename", "--line-number"];

        if ignore_case {
            args.push("--ignore-case");
        }

        if let Some(g) = glob_pattern {
            args.push("--glob");
            args.push(g);
        }

        args.push("--");
        args.push(pattern);
        args.push(path);

        let output = match tokio::process::Command::new("rg")
            .args(&args)
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                // rg not found, try fallback
                return Ok(ToolResult::error(format!(
                    "ripgrep (rg) not found. Please install ripgrep: {}",
                    e
                )));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if output.status.success() && !stdout.is_empty() {
            Ok(ToolResult::success(stdout))
        } else if stdout.is_empty() {
            Ok(ToolResult::success("No matches found".to_string()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(ToolResult::error(if !stderr.is_empty() {
                stderr
            } else {
                "No matches found".to_string()
            }))
        }
    }
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::new()
    }
}
