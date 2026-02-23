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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_read_file() {
        let tool = ReadTool::new();

        // 一時ファイルを作成
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();

        let input = json!({"path": temp_file.path().to_str().unwrap()});
        let result = tool.execute(input).await.unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("Hello, World!"));
        assert!(result.output.contains("Line 2"));
        assert!(result.output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_read_with_offset() {
        let tool = ReadTool::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();

        // 2行目から読み取り (offset=2 は2行目から)
        let input = json!({
            "path": temp_file.path().to_str().unwrap(),
            "offset": 2
        });
        let result = tool.execute(input).await.unwrap();

        assert!(!result.is_error);
        assert!(!result.output.contains("Line 1"));
        assert!(result.output.contains("Line 2"));
        assert!(result.output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_read_with_limit() {
        let tool = ReadTool::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();

        // 最初の2行のみ読み取り
        let input = json!({
            "path": temp_file.path().to_str().unwrap(),
            "limit": 2
        });
        let result = tool.execute(input).await.unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("Line 1"));
        assert!(result.output.contains("Line 2"));
        assert!(!result.output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let tool = ReadTool::new();

        let input = json!({"path": "/nonexistent/file/path.txt"});
        let result = tool.execute(input).await.unwrap();

        assert!(result.is_error);
        assert!(result.output.contains("Failed to read file"));
    }

    #[tokio::test]
    async fn test_read_missing_path() {
        let tool = ReadTool::new();

        let input = json!({});
        let result = tool.execute(input).await;

        assert!(result.is_err());
    }
}
