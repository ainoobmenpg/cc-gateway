//! Bash command execution tool
//!
//! Executes shell commands with optional timeout.

use async_trait::async_trait;
use cc_core::{Result, Tool, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Bash tool for executing shell commands
pub struct BashTool;

/// Input parameters for the bash tool
#[derive(Debug, Deserialize)]
struct BashInput {
    /// The command to execute
    command: String,
    /// Timeout in milliseconds (default: 120000)
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

fn default_timeout() -> u64 {
    120_000
}

/// Output from bash execution
#[derive(Debug, Serialize)]
struct BashOutput {
    /// Standard output from the command
    stdout: String,
    /// Standard error from the command
    stderr: String,
    /// Exit code of the command
    exit_code: Option<i32>,
    /// Whether the command timed out
    timed_out: bool,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command with optional timeout. Use this for terminal operations like git, npm, docker, etc."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds (default: 120000, max: 600000)",
                    "default": 120000,
                    "maximum": 600000
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolResult> {
        let bash_input: BashInput = serde_json::from_value(input)
            .map_err(|e| cc_core::Error::ToolExecution(format!("Invalid input: {}", e)))?;

        // Limit timeout to 10 minutes max
        let timeout_ms = bash_input.timeout_ms.min(600_000);
        let duration = Duration::from_millis(timeout_ms);

        tracing::debug!(
            command = %bash_input.command,
            timeout_ms = timeout_ms,
            "Executing bash command"
        );

        // Execute the command with timeout
        let result = timeout(
            duration,
            Command::new("bash")
                .arg("-c")
                .arg(&bash_input.command)
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let bash_output = BashOutput {
                    stdout,
                    stderr,
                    exit_code: output.status.code(),
                    timed_out: false,
                };

                let output_str = serde_json::to_string_pretty(&bash_output)
                    .unwrap_or_else(|_| format!("{:?}", bash_output));

                // Return as error if exit code is non-zero
                if output.status.success() {
                    Ok(ToolResult::success(output_str))
                } else {
                    Ok(ToolResult::error(output_str))
                }
            }
            Ok(Err(e)) => {
                Ok(ToolResult::error(format!("Failed to execute command: {}", e)))
            }
            Err(_) => {
                Ok(ToolResult::error(format!(
                    "Command timed out after {}ms",
                    timeout_ms
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool;
        let input = json!({"command": "echo hello"});
        let result = tool.execute(input).await.unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_failure() {
        let tool = BashTool;
        let input = json!({"command": "exit 1"});
        let result = tool.execute(input).await.unwrap();

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_bash_timeout() {
        let tool = BashTool;
        let input = json!({
            "command": "sleep 10",
            "timeout_ms": 100
        });
        let result = tool.execute(input).await.unwrap();

        assert!(result.is_error);
        assert!(result.output.contains("timed out"));
    }
}
