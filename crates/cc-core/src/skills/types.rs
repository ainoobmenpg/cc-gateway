//! Skill types and configurations
//!
//! Defines the structure for skill configuration files.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A skill definition from a configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    /// Skill metadata
    pub skill: Skill,
}

/// Skill metadata and definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier for the skill (e.g., "weather", "joke")
    pub name: String,

    /// Human-readable description shown to Claude
    pub description: String,

    /// Input schema (JSON Schema format)
    #[serde(default)]
    pub input_schema: Option<JsonValue>,

    /// Parameters definition (alternative to input_schema)
    #[serde(default)]
    pub parameters: Option<HashMap<String, SkillParameter>>,

    /// Execution configuration
    #[serde(flatten)]
    pub execution: SkillExecution,
}

/// Parameter definition for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParameter {
    /// Parameter type (string, number, boolean, array, object)
    #[serde(rename = "type")]
    pub param_type: String,

    /// Parameter description
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the parameter is required
    #[serde(default)]
    pub required: Option<bool>,

    /// Default value
    #[serde(default)]
    pub default: Option<JsonValue>,

    /// Enum values (for string type)
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,

    #[serde(rename = "enum")]
    pub enum_alias: Option<Vec<String>>,
}

/// Execution configuration for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillExecution {
    /// HTTP-based skill (calls an API)
    Http(SkillHttpConfig),
    /// Prompt-based skill (generates a prompt)
    Prompt(SkillPromptConfig),
    /// Shell-based skill (runs a command)
    Shell(SkillShellConfig),
}

/// HTTP-based skill configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHttpConfig {
    /// HTTP method (GET, POST, PUT, DELETE)
    pub method: Option<String>,

    /// URL template (supports {param} substitution)
    pub url: String,

    /// Headers to include
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Request body template (for POST/PUT)
    #[serde(default)]
    pub body: Option<JsonValue>,

    /// Path to extract from response (JSONPath-like)
    #[serde(default)]
    pub extract: Option<String>,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

/// Prompt-based skill configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPromptConfig {
    /// Prompt template (supports {param} substitution)
    pub prompt: String,

    /// System prompt to include
    #[serde(default)]
    pub system: Option<String>,

    /// Model to use (defaults to configured model)
    #[serde(default)]
    pub model: Option<String>,

    /// Temperature for generation
    #[serde(default)]
    pub temperature: Option<f32>,
}

/// Shell-based skill configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillShellConfig {
    /// Command template (supports {param} substitution)
    pub command: String,

    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    30
}

impl Skill {
    /// Convert parameters to JSON Schema format
    pub fn to_input_schema(&self) -> JsonValue {
        if let Some(schema) = &self.input_schema {
            return schema.clone();
        }

        if let Some(params) = &self.parameters {
            let mut properties = serde_json::Map::new();
            let mut required = Vec::new();

            for (name, param) in params {
                let mut prop = serde_json::Map::new();
                prop.insert("type".to_string(), JsonValue::String(param.param_type.clone()));

                if let Some(desc) = &param.description {
                    prop.insert("description".to_string(), JsonValue::String(desc.clone()));
                }

                if let Some(enum_vals) = &param.enum_values {
                    prop.insert(
                        "enum".to_string(),
                        JsonValue::Array(enum_vals.iter().map(|s| JsonValue::String(s.clone())).collect()),
                    );
                } else if let Some(enum_vals) = &param.enum_alias {
                    prop.insert(
                        "enum".to_string(),
                        JsonValue::Array(enum_vals.iter().map(|s| JsonValue::String(s.clone())).collect()),
                    );
                }

                properties.insert(name.clone(), JsonValue::Object(prop));

                if param.required.unwrap_or(false) {
                    required.push(JsonValue::String(name.clone()));
                }
            }

            serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": required
            })
        } else {
            serde_json::json!({
                "type": "object",
                "properties": {}
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_http_config_parsing() {
        let yaml = r#"
skill:
  name: weather
  description: Get current weather for a location
  parameters:
    location:
      type: string
      description: City name or coordinates
      required: true
  url: "https://api.weather.com/v1/current?location={location}"
  method: GET
  headers:
    Accept: application/json
"#;
        let config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.skill.name, "weather");
        assert!(config.skill.parameters.is_some());
    }

    #[test]
    fn test_skill_to_input_schema() {
        let mut params = HashMap::new();
        params.insert(
            "query".to_string(),
            SkillParameter {
                param_type: "string".to_string(),
                description: Some("Search query".to_string()),
                required: Some(true),
                default: None,
                enum_values: None,
                enum_alias: None,
            },
        );

        let skill = Skill {
            name: "test".to_string(),
            description: "Test skill".to_string(),
            input_schema: None,
            parameters: Some(params),
            execution: SkillExecution::Prompt(SkillPromptConfig {
                prompt: "Test {query}".to_string(),
                system: None,
                model: None,
                temperature: None,
            }),
        };

        let schema = skill.to_input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
        assert_eq!(schema["required"][0], "query");
    }

    #[test]
    fn test_skill_prompt_config_parsing() {
        let yaml = r#"
skill:
  name: summarize
  description: Summarize the given text
  parameters:
    text:
      type: string
      description: Text to summarize
      required: true
  prompt: "Please summarize the following text concisely:\n\n{text}"
  system: "You are a helpful assistant that creates concise summaries."
"#;
        let config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.skill.name, "summarize");

        if let SkillExecution::Prompt(prompt_config) = &config.skill.execution {
            assert!(prompt_config.prompt.contains("{text}"));
            assert!(prompt_config.system.is_some());
        } else {
            panic!("Expected Prompt execution");
        }
    }

    #[test]
    fn test_skill_shell_config_parsing() {
        let yaml = r#"
skill:
  name: run_script
  description: Run a shell script
  parameters:
    script:
      type: string
      description: Script path
      required: true
  command: "bash {script}"
  timeout: 60
"#;
        let config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.skill.name, "run_script");

        if let SkillExecution::Shell(shell_config) = &config.skill.execution {
            assert_eq!(shell_config.command, "bash {script}");
            assert_eq!(shell_config.timeout, 60);
        } else {
            panic!("Expected Shell execution");
        }
    }
}
