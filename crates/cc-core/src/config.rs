//! Configuration management

use serde::{Deserialize, Serialize};

/// LLM Provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    /// Anthropic Claude API
    Claude,
    /// OpenAI-compatible API (GLM, etc.)
    OpenAi,
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::Claude
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API key
    pub api_key: String,

    /// Model to use
    #[serde(default = "default_model")]
    pub model: String,

    /// API provider
    #[serde(default)]
    pub provider: LlmProvider,

    /// Base URL (optional, for custom endpoints)
    pub base_url: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: default_model(),
            provider: LlmProvider::Claude,
            base_url: None,
        }
    }
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

/// Main configuration for cc-gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM configuration (new unified config)
    #[serde(default)]
    pub llm: LlmConfig,

    /// Claude API key (legacy, maps to llm.api_key)
    #[serde(skip_serializing)]
    pub claude_api_key: String,

    /// Claude model to use (legacy, maps to llm.model)
    #[serde(skip_serializing)]
    #[serde(default = "default_model")]
    pub claude_model: String,

    /// Discord bot token (optional)
    pub discord_token: Option<String>,

    /// Admin user IDs (comma-separated or array)
    #[serde(default)]
    pub admin_user_ids: Vec<String>,

    /// HTTP API configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// Memory configuration
    #[serde(default)]
    pub memory: MemoryConfig,

    /// MCP configuration
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    /// API key for HTTP API authentication
    pub key: Option<String>,

    /// Port for HTTP API server
    #[serde(default = "default_api_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Path to MCP configuration file (JSON format)
    pub config_path: Option<String>,

    /// Whether MCP integration is enabled
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            config_path: None,
            enabled: default_mcp_enabled(),
        }
    }
}

fn default_mcp_enabled() -> bool {
    true
}

fn default_api_port() -> u16 {
    3000
}

fn default_db_path() -> String {
    "data/cc-gateway.db".to_string()
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> crate::Result<Self> {
        // Get API key from either LLM_API_KEY or CLAUDE_API_KEY
        let api_key = std::env::var("LLM_API_KEY")
            .or_else(|_| std::env::var("CLAUDE_API_KEY"))
            .map_err(|_| Error::Config("LLM_API_KEY or CLAUDE_API_KEY not set".to_string()))?;

        // Get model from either LLM_MODEL or CLAUDE_MODEL
        let model = std::env::var("LLM_MODEL")
            .or_else(|_| std::env::var("CLAUDE_MODEL"))
            .unwrap_or_else(|_| default_model());

        // Determine provider
        let provider = match std::env::var("LLM_PROVIDER").unwrap_or_default().to_lowercase().as_str() {
            "openai" | "glm" | "zai" => LlmProvider::OpenAi,
            _ => LlmProvider::Claude,
        };

        // Get base URL (for custom endpoints like GLM Coding Plan)
        let base_url = std::env::var("LLM_BASE_URL").ok();

        let llm_config = LlmConfig {
            api_key: api_key.clone(),
            model: model.clone(),
            provider,
            base_url,
        };

        Ok(Config {
            llm: llm_config,
            claude_api_key: api_key,
            claude_model: model,
            discord_token: std::env::var("DISCORD_BOT_TOKEN").ok(),
            admin_user_ids: std::env::var("ADMIN_USER_IDS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            api: ApiConfig {
                key: std::env::var("API_KEY").ok(),
                port: std::env::var("API_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(default_api_port()),
            },
            memory: MemoryConfig {
                db_path: std::env::var("DB_PATH")
                    .unwrap_or_else(|_| default_db_path()),
            },
            mcp: McpConfig {
                config_path: std::env::var("MCP_CONFIG_PATH").ok(),
                enabled: std::env::var("MCP_ENABLED")
                    .map(|v| v.to_lowercase() != "false")
                    .unwrap_or(default_mcp_enabled()),
            },
        })
    }

    /// Get the effective LLM configuration
    pub fn llm_config(&self) -> &LlmConfig {
        &self.llm
    }
}

use crate::Error;
