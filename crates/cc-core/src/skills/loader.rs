//! Skill loader for loading skills from configuration files
//!
//! Provides functionality to discover and load skill definitions
//! from YAML or TOML files.

use crate::skills::types::{Skill, SkillConfig, SkillExecution, SkillHttpConfig, SkillPromptConfig, SkillShellConfig};
use crate::{Error, Result, Tool, ToolManager, ToolResult};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;

/// Skill loader that discovers and loads skills from directories
pub struct SkillLoader {
    /// Directories to search for skill files
    search_dirs: Vec<PathBuf>,
}

impl SkillLoader {
    /// Create a new skill loader with default search directories
    pub fn new() -> Self {
        let mut search_dirs = vec![
            PathBuf::from("skills"),
            PathBuf::from(".cc-gateway/skills"),
        ];

        // Add user home directory
        if let Some(home) = std::env::var_os("HOME") {
            search_dirs.push(PathBuf::from(home).join(".cc-gateway/skills"));
        }

        Self { search_dirs }
    }

    /// Create a skill loader with custom search directories
    pub fn with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { search_dirs: dirs }
    }

    /// Add a search directory
    pub fn add_dir(&mut self, dir: PathBuf) {
        self.search_dirs.push(dir);
    }

    /// Load all skills from search directories and register them
    pub async fn load_and_register(&self, manager: &mut ToolManager) -> Result<Vec<String>> {
        let mut registered = Vec::new();

        for dir in &self.search_dirs {
            if !dir.exists() {
                continue;
            }

            let skills = self.load_from_dir(dir).await?;
            for skill in skills {
                let name = skill.name.clone();
                manager.register(Arc::new(SkillTool::new(skill)));
                registered.push(name);
            }
        }

        Ok(registered)
    }

    /// Load all skills from a directory
    pub async fn load_from_dir(&self, dir: &Path) -> Result<Vec<Skill>> {
        let mut skills = Vec::new();

        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| Error::Config(format!("Failed to read skills directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::Config(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();

            // Check if it's a skill file
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" || ext == "toml" {
                    match self.load_skill(&path).await {
                        Ok(skill) => {
                            tracing::info!(
                                skill = %skill.name,
                                path = %path.display(),
                                "Loaded skill"
                            );
                            skills.push(skill);
                        }
                        Err(e) => {
                            tracing::warn!(
                                path = %path.display(),
                                error = %e,
                                "Failed to load skill file"
                            );
                        }
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Load a single skill from a file
    pub async fn load_skill(&self, path: &Path) -> Result<Skill> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| Error::Config(format!("Failed to read skill file {}: {}", path.display(), e)))?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let config: SkillConfig = match extension {
            "yaml" | "yml" => serde_yaml::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse YAML skill file: {}", e)))?,
            "toml" => toml::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse TOML skill file: {}", e)))?,
            _ => {
                return Err(Error::Config(format!(
                    "Unsupported skill file format: {}",
                    extension
                )))
            }
        };

        Ok(config.skill)
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool that wraps a skill definition
pub struct SkillTool {
    skill: Skill,
    http_client: reqwest::Client,
}

impl SkillTool {
    /// Create a new skill tool
    pub fn new(skill: Skill) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { skill, http_client }
    }

    /// Substitute parameters in a template string
    fn substitute_template(&self, template: &str, input: &JsonValue) -> String {
        let mut result = template.to_string();

        if let Some(obj) = input.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{}}}", key);
                let replacement = match value {
                    JsonValue::String(s) => s.clone(),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        result
    }

    /// Execute an HTTP-based skill
    async fn execute_http(&self, http_config: &SkillHttpConfig, input: &JsonValue) -> Result<String> {
        let url = self.substitute_template(&http_config.url, input);
        let method = http_config.method.as_deref().unwrap_or("GET");

        let mut request = match method {
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => self.http_client.get(&url),
        };

        // Add headers
        for (key, value) in &http_config.headers {
            request = request.header(key, value);
        }

        // Add body if present
        if let Some(body) = &http_config.body {
            let body_str = self.substitute_template(&body.to_string(), input);
            request = request.body(body_str);
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::ToolExecution(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ToolExecution(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| Error::ToolExecution(format!("Failed to read response: {}", e)))?;

        // Extract if specified
        if let Some(extract_path) = &http_config.extract {
            extract_json_value(&response_text, extract_path)
        } else {
            Ok(response_text)
        }
    }

    /// Execute a prompt-based skill (returns the prompt for processing)
    async fn execute_prompt(&self, prompt_config: &SkillPromptConfig, input: &JsonValue) -> Result<String> {
        let prompt = self.substitute_template(&prompt_config.prompt, input);

        // Return the prompt as output - the caller can use this to generate a response
        let mut output = format!("Prompt: {}\n", prompt);

        if let Some(system) = &prompt_config.system {
            output.push_str(&format!("System: {}\n", system));
        }

        if let Some(model) = &prompt_config.model {
            output.push_str(&format!("Model: {}\n", model));
        }

        Ok(output)
    }

    /// Execute a shell-based skill
    async fn execute_shell(&self, shell_config: &SkillShellConfig, input: &JsonValue) -> Result<String> {
        let command_str = self.substitute_template(&shell_config.command, input);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .output()
            .await
            .map_err(|e| Error::ToolExecution(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(if stdout.is_empty() { stderr } else { stdout })
        } else {
            Err(Error::ToolExecution(format!(
                "Command failed with status {}: {}",
                output.status,
                if stderr.is_empty() { &stdout } else { &stderr }
            )))
        }
    }
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        &self.skill.name
    }

    fn description(&self) -> &str {
        &self.skill.description
    }

    fn input_schema(&self) -> JsonValue {
        self.skill.to_input_schema()
    }

    async fn execute(&self, input: JsonValue) -> Result<ToolResult> {
        tracing::info!(
            skill = %self.skill.name,
            input = ?input,
            "Executing skill"
        );

        let result = match &self.skill.execution {
            SkillExecution::Http(http_config) => self.execute_http(http_config, &input).await,
            SkillExecution::Prompt(prompt_config) => self.execute_prompt(prompt_config, &input).await,
            SkillExecution::Shell(shell_config) => self.execute_shell(shell_config, &input).await,
        };

        match result {
            Ok(output) => Ok(ToolResult::success(output)),
            Err(e) => Ok(ToolResult::error(format!("Skill execution failed: {}", e))),
        }
    }
}

/// Extract a value from JSON using a simple path syntax (e.g., "data.results.0.name")
fn extract_json_value(json_str: &str, path: &str) -> Result<String> {
    let value: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| Error::ToolExecution(format!("Failed to parse JSON response: {}", e)))?;

    let mut current = &value;

    for part in path.split('.') {
        if part.chars().all(|c| c.is_ascii_digit()) {
            // Array index
            let index: usize = part.parse().map_err(|_| {
                Error::ToolExecution(format!("Invalid array index in path: {}", part))
            })?;
            current = current.get(index).ok_or_else(|| {
                Error::ToolExecution(format!("Array index {} out of bounds", index))
            })?;
        } else {
            // Object key
            current = current.get(part).ok_or_else(|| {
                Error::ToolExecution(format!("Key '{}' not found in JSON", part))
            })?;
        }
    }

    match current {
        JsonValue::String(s) => Ok(s.clone()),
        JsonValue::Number(n) => Ok(n.to_string()),
        JsonValue::Bool(b) => Ok(b.to_string()),
        JsonValue::Null => Ok("null".to_string()),
        _ => Ok(current.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_substitute_template() {
        let skill = Skill {
            name: "test".to_string(),
            description: "Test".to_string(),
            input_schema: None,
            parameters: None,
            execution: SkillExecution::Prompt(SkillPromptConfig {
                prompt: "Hello {name}!".to_string(),
                system: None,
                model: None,
                temperature: None,
            }),
        };

        let tool = SkillTool::new(skill);
        let input = serde_json::json!({ "name": "World" });
        let result = tool.substitute_template("Hello {name}!", &input);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_extract_json_value() {
        let json = r#"{"data": {"results": [{"name": "test"}]}}"#;

        assert_eq!(extract_json_value(json, "data.results.0.name").unwrap(), "test");
        assert_eq!(extract_json_value(json, "data").unwrap(), r#"{"results":[{"name":"test"}]}"#);
    }

    #[tokio::test]
    async fn test_load_from_dir() {
        let dir = tempdir().unwrap();
        let skill_path = dir.path().join("test_skill.yaml");

        let yaml_content = r#"
skill:
  name: test
  description: Test skill
  prompt: "Test {input}"
"#;

        let mut file = std::fs::File::create(&skill_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        let loader = SkillLoader::with_dirs(vec![dir.path().to_path_buf()]);
        let skills = loader.load_from_dir(dir.path()).await.unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test");
    }
}
