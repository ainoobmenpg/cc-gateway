//! Configuration management
//!
//! 設定は以下の優先順位で読み込まれます:
//! 1. 環境変数
//! 2. cc-gateway.toml 設定ファイル
//! 3. デフォルト値
//!
//! 設定ファイル内では `${VAR_NAME}` 形式で環境変数を展開できます。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// LLM Provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LlmProvider {
    /// Anthropic Claude API
    #[default]
    Claude,
    /// OpenAI-compatible API (GLM, etc.)
    OpenAi,
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

    /// Scheduler configuration
    #[serde(default)]
    pub scheduler: SchedulerConfig,

    /// API key for HTTP API (shorthand for api.key)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API key for HTTP API authentication
    pub key: Option<String>,

    /// Port for HTTP API server
    #[serde(default = "default_api_port")]
    pub port: u16,

    /// Allowed CORS origins (e.g., ["http://localhost:3000", "https://example.com"])
    /// If empty, defaults to localhost only
    #[serde(default)]
    pub allowed_origins: Option<Vec<String>>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            key: None,
            port: default_api_port(),
            allowed_origins: None,
        }
    }
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
    /// 設定ファイルから環境変数を展開する
    ///
    /// `${VAR_NAME}` 形式の文字列を環境変数の値に置換します。
    /// 環境変数が存在しない場合は文字列をそのまま返します。
    fn expand_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'{') {
                chars.next(); // '{' を消費

                let mut var_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next(); // '}' を消費
                        break;
                    }
                    var_name.push(chars.next().unwrap());
                }

                // 環境変数を展開（存在しない場合は空文字列）
                if let Ok(env_value) = std::env::var(&var_name) {
                    result.push_str(&env_value);
                } else if !var_name.is_empty() {
                    // 環境変数が存在しない場合は空文字列にする
                    // または: result.push_str(&format!("${{{}}}", var_name));
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// 文字列値または文字列配列内の環境変数を展開
    #[allow(dead_code)]
    fn expand_string_value(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => {
                serde_json::Value::String(Self::expand_env_vars(s))
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(
                    arr.iter().map(Self::expand_string_value).collect()
                )
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), Self::expand_string_value(v));
                }
                serde_json::Value::Object(new_obj)
            }
            _ => value.clone(),
        }
    }

    /// TOML 設定ファイルから設定を読み込む
    ///
    /// # 引数
    /// * `path` - TOML ファイルのパス
    ///
    /// # 環境変数展開
    /// 設定ファイル内の `${VAR_NAME}` は環境変数の値に置換されます。
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref();

        // TOML ファイルを読み込む
        let toml_content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        // 環境変数を展開
        let expanded_content = Self::expand_env_vars(&toml_content);

        // TOML をパース
        let config: TomlConfig = toml::from_str(&expanded_content)
            .map_err(|e| Error::Config(format!("Failed to parse TOML: {}", e)))?;

        // TOML 構造から Config に変換
        let mut cfg = Self::from_toml_config(config)?;

        // 既存の環境変数で上書き（環境変数が優先）
        cfg.apply_env_overrides();

        Ok(cfg)
    }

    /// デフォルトパスから設定を読み込む
    ///
    /// 以下の順序で設定ファイルを探します:
    /// 1. `./cc-gateway.toml`
    /// 2. `./cc-gateway.toml.example` (開発用)
    /// 3. 見つからない場合は環境変数のみ
    pub fn load() -> crate::Result<Self> {
        // カレントディレクトリの cc-gateway.toml を試す
        if Path::new("cc-gateway.toml").exists() {
            return Self::from_toml_file("cc-gateway.toml");
        }

        // ファイルがない場合は環境変数から読み込み
        Self::from_env()
    }

    /// TOML 構造から Config を構築
    fn from_toml_config(toml: TomlConfig) -> crate::Result<Self> {
        // LLM 設定の変換
        let llm = toml.llm.unwrap_or_default();

        // provider 文字列から LlmProvider への変換
        let provider = match llm.provider.unwrap_or_default().to_lowercase().as_str() {
            "openai" | "glm" | "zai" | "minimax" => LlmProvider::OpenAi,
            _ => LlmProvider::Claude,
        };

        let llm_config = LlmConfig {
            api_key: llm.api_key.unwrap_or_default(),
            model: llm.model.unwrap_or_else(default_model),
            provider,
            base_url: llm.base_url,
        };

        // Discord 設定
        let discord = toml.discord.unwrap_or_default();
        let admin_ids: Vec<String> = discord.admin_user_ids
            .unwrap_or_default()
            .into_iter()
            .map(|id| id.to_string())
            .collect();

        // API 設定
        let api = toml.api.unwrap_or_default();
        let api_config = ApiConfig {
            key: api.key.clone(),
            port: api.port.unwrap_or_else(default_api_port),
            allowed_origins: api.allowed_origins,
        };

        // Memory 設定
        let memory = toml.memory.unwrap_or_default();
        let memory_config = MemoryConfig {
            db_path: memory.db_path.unwrap_or_else(default_db_path),
        };

        // MCP 設定
        let mcp = toml.mcp.unwrap_or_default();
        let mcp_config = McpConfig {
            config_path: mcp.config_path,
            enabled: mcp.enabled.unwrap_or_else(default_mcp_enabled),
        };

        // Scheduler 設定
        let scheduler = toml.scheduler.unwrap_or_default();
        let scheduler_config = SchedulerConfig {
            enabled: scheduler.enabled.unwrap_or(true),
            config_path: scheduler.config_path,
        };

        Ok(Config {
            llm: llm_config.clone(),
            claude_api_key: llm_config.api_key.clone(),
            claude_model: llm_config.model.clone(),
            discord_token: discord.token,
            admin_user_ids: admin_ids,
            api: api_config.clone(),
            api_key: api_config.key,
            memory: memory_config,
            mcp: mcp_config,
            scheduler: scheduler_config,
        })
    }

    /// 環境変数で設定を上書きする
    fn apply_env_overrides(&mut self) {
        // LLM 設定の上書き
        if let Ok(api_key) = std::env::var("LLM_API_KEY") {
            self.llm.api_key = api_key.clone();
            self.claude_api_key = api_key;
        }
        if let Ok(api_key) = std::env::var("CLAUDE_API_KEY") {
            self.llm.api_key = api_key.clone();
            self.claude_api_key = api_key;
        }

        // Only use LLM_MODEL/CLAUDE_MODEL if they are explicitly set and non-empty
        // (don't use ANTHROPIC_* to avoid shell environment conflicts)
        if let Ok(model) = std::env::var("LLM_MODEL") {
            if !model.is_empty() {
                self.llm.model = model.clone();
                self.claude_model = model;
            }
        } else if let Ok(model) = std::env::var("CLAUDE_MODEL") {
            if !model.is_empty() {
                self.llm.model = model.clone();
                self.claude_model = model;
            }
        }

        if let Ok(provider) = std::env::var("LLM_PROVIDER") {
            if !provider.is_empty() {
                self.llm.provider = match provider.to_lowercase().as_str() {
                    "openai" | "glm" | "zai" | "minimax" => LlmProvider::OpenAi,
                    _ => LlmProvider::Claude,
                };
            }
        }

        // Only use LLM_BASE_URL if explicitly set and non-empty (respect TOML config)
        if let Ok(base_url) = std::env::var("LLM_BASE_URL") {
            if !base_url.is_empty() {
                self.llm.base_url = Some(base_url);
            }
        }

        // Discord 設定の上書き
        if let Ok(token) = std::env::var("DISCORD_BOT_TOKEN") {
            self.discord_token = Some(token);
        }

        if let Ok(ids) = std::env::var("ADMIN_USER_IDS") {
            self.admin_user_ids = ids.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        // API 設定の上書き
        if let Ok(key) = std::env::var("API_KEY") {
            self.api.key = Some(key.clone());
            self.api_key = Some(key);
        }
        if let Ok(port) = std::env::var("API_PORT") {
            if let Ok(p) = port.parse() {
                self.api.port = p;
            }
        }
        if let Ok(origins) = std::env::var("API_ALLOWED_ORIGINS") {
            self.api.allowed_origins = Some(
                origins.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            );
        }

        // Memory 設定の上書き
        if let Ok(path) = std::env::var("DB_PATH") {
            self.memory.db_path = path;
        }

        // MCP 設定の上書き
        if let Ok(path) = std::env::var("MCP_CONFIG_PATH") {
            self.mcp.config_path = Some(path);
        }
        if let Ok(enabled) = std::env::var("MCP_ENABLED") {
            self.mcp.enabled = enabled.to_lowercase() != "false";
        }

        // Scheduler 設定の上書き
        if let Ok(enabled) = std::env::var("SCHEDULE_ENABLED") {
            self.scheduler.enabled = enabled.to_lowercase() != "false";
        }
        if let Ok(path) = std::env::var("SCHEDULE_CONFIG_PATH") {
            self.scheduler.config_path = Some(path);
        }
    }

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
            "openai" | "glm" | "zai" | "minimax" => LlmProvider::OpenAi,
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
            claude_api_key: api_key.clone(),
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
                allowed_origins: std::env::var("API_ALLOWED_ORIGINS")
                    .ok()
                    .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
            },
            api_key: std::env::var("API_KEY").ok(),
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
            scheduler: SchedulerConfig {
                enabled: std::env::var("SCHEDULE_ENABLED")
                    .map(|v| v.to_lowercase() != "false")
                    .unwrap_or(true),
                config_path: std::env::var("SCHEDULE_CONFIG_PATH").ok(),
            },
        })
    }

    /// Get the effective LLM configuration
    pub fn llm_config(&self) -> &LlmConfig {
        &self.llm
    }
}

use crate::Error;

// ============================================================================
// TOML 構造体定義（ファイル解析用）
// ============================================================================

/// TOML ファイル用のトップレベル構造
#[derive(Debug, Deserialize)]
struct TomlConfig {
    /// LLM 設定
    llm: Option<TomlLlmConfig>,
    /// Discord 設定
    discord: Option<TomlDiscordConfig>,
    /// HTTP API 設定
    api: Option<TomlApiConfig>,
    /// メモリ設定
    memory: Option<TomlMemoryConfig>,
    /// MCP 設定
    mcp: Option<TomlMcpConfig>,
    /// スケジューラー設定
    scheduler: Option<TomlSchedulerConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlLlmConfig {
    /// API プロバイダー ("claude" または "openai")
    #[serde(default)]
    provider: Option<String>,
    /// モデル名
    #[serde(default)]
    model: Option<String>,
    /// API キー
    #[serde(default)]
    api_key: Option<String>,
    /// ベース URL (オプション)
    #[serde(default)]
    base_url: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlDiscordConfig {
    /// Discord Bot トークン
    token: Option<String>,
    /// 管理者ユーザー ID リスト
    admin_user_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlApiConfig {
    /// API キー (オプション)
    #[serde(default)]
    key: Option<String>,
    /// ポート番号
    #[serde(default)]
    port: Option<u16>,
    /// 許可する CORS オリジン
    #[serde(default)]
    allowed_origins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlMemoryConfig {
    /// データベースパス
    #[serde(default)]
    db_path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlMcpConfig {
    /// MCP 設定ファイルパス
    config_path: Option<String>,
    /// 有効/無効
    enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlSchedulerConfig {
    /// 有効/無効
    enabled: Option<bool>,
    /// スケジュール設定ファイルパス
    config_path: Option<String>,
}

// ============================================================================
// SchedulerConfig（cc-schedule から独立）
// ============================================================================

/// スケジューラー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// スケジューラーが有効かどうか
    pub enabled: bool,

    /// スケジュール設定ファイルパス
    pub config_path: Option<String>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_provider_default() {
        let provider = LlmProvider::default();
        assert_eq!(provider, LlmProvider::Claude);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.provider, LlmProvider::Claude);
        assert!(config.api_key.is_empty());
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.port, 3000);
        assert!(config.key.is_none());
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert_eq!(config.db_path, "data/cc-gateway.db");
    }

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        assert!(config.enabled);
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_expand_env_vars() {
        // テスト用環境変数を設定
        unsafe {
            std::env::set_var("CC_GATEWAY_TEST_VAR", "test_value");
        }

        let result = Config::expand_env_vars("prefix_${CC_GATEWAY_TEST_VAR}_suffix");
        assert_eq!(result, "prefix_test_value_suffix");

        // 存在しない環境変数
        let result = Config::expand_env_vars("prefix_${NONEXISTENT_VAR}_suffix");
        assert_eq!(result, "prefix__suffix");

        unsafe {
            std::env::remove_var("CC_GATEWAY_TEST_VAR");
        }
    }

    #[test]
    fn test_expand_env_vars_no_braces() {
        let result = Config::expand_env_vars("no_vars_here");
        assert_eq!(result, "no_vars_here");
    }

    #[test]
    fn test_expand_env_vars_empty_name() {
        let result = Config::expand_env_vars("${}_content");
        assert_eq!(result, "_content");
    }

    #[test]
    fn test_llm_provider_from_string() {
        // Claude provider tests
        assert_eq!(LlmProvider::Claude, LlmProvider::Claude);
        assert_eq!(LlmProvider::OpenAi, LlmProvider::OpenAi);

        // Check PartialEq works
        assert!(LlmProvider::Claude == LlmProvider::Claude);
        assert!(LlmProvider::Claude != LlmProvider::OpenAi);
    }

    #[test]
    fn test_config_llm_config() {
        let config = Config {
            llm: LlmConfig {
                api_key: "test_key".to_string(),
                model: "test_model".to_string(),
                provider: LlmProvider::Claude,
                base_url: Some("https://example.com".to_string()),
            },
            claude_api_key: "test_key".to_string(),
            claude_model: "test_model".to_string(),
            discord_token: None,
            admin_user_ids: vec![],
            api: ApiConfig::default(),
            api_key: None,
            memory: MemoryConfig::default(),
            mcp: McpConfig::default(),
            scheduler: SchedulerConfig::default(),
        };

        let llm_config = config.llm_config();
        assert_eq!(llm_config.api_key, "test_key");
        assert_eq!(llm_config.model, "test_model");
        assert_eq!(llm_config.provider, LlmProvider::Claude);
        assert_eq!(llm_config.base_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_toml_config_parsing() {
        let toml_content = r#"
[llm]
provider = "openai"
model = "glm-4.7"
api_key = "test_key"
base_url = "https://api.example.com"

[discord]
token = "discord_token"
admin_user_ids = [123456, 789012]

[api]
port = 8080
key = "api_key"

[memory]
db_path = "/path/to/db"

[mcp]
enabled = false
config_path = "/path/to/mcp.json"

[scheduler]
enabled = true
config_path = "/path/to/schedule.toml"
"#;

        let toml_config: TomlConfig = toml::from_str(toml_content).unwrap();

        // LLM 設定の検証
        let llm = toml_config.llm.unwrap();
        assert_eq!(llm.provider, Some("openai".to_string()));
        assert_eq!(llm.model, Some("glm-4.7".to_string()));
        assert_eq!(llm.api_key, Some("test_key".to_string()));
        assert_eq!(llm.base_url, Some("https://api.example.com".to_string()));

        // Discord 設定の検証
        let discord = toml_config.discord.unwrap();
        assert_eq!(discord.token, Some("discord_token".to_string()));
        assert_eq!(discord.admin_user_ids, Some(vec![123456, 789012]));

        // API 設定の検証
        let api = toml_config.api.unwrap();
        assert_eq!(api.port, Some(8080));
        assert_eq!(api.key, Some("api_key".to_string()));

        // Memory 設定の検証
        let memory = toml_config.memory.unwrap();
        assert_eq!(memory.db_path, Some("/path/to/db".to_string()));

        // MCP 設定の検証
        let mcp = toml_config.mcp.unwrap();
        assert_eq!(mcp.enabled, Some(false));
        assert_eq!(mcp.config_path, Some("/path/to/mcp.json".to_string()));

        // Scheduler 設定の検証
        let scheduler = toml_config.scheduler.unwrap();
        assert_eq!(scheduler.enabled, Some(true));
        assert_eq!(scheduler.config_path, Some("/path/to/schedule.toml".to_string()));
    }
}
