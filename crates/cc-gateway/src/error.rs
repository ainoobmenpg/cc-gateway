//! エラー型定義 (cc-gateway)

use std::fmt;

/// cc-gateway の統合エラー型
///
/// 各サブクレートのエラー型を統合して扱います
#[derive(Debug)]
pub enum GatewayError {
    /// Core error
    Core(cc_core::Error),
    /// Schedule error
    Schedule(String),
    /// Discord error
    Discord(String),
    /// API error
    Api(String),
    /// MCP error
    Mcp(String),
    /// Other error
    Other(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(e) => write!(f, "Core error: {}", e),
            Self::Schedule(e) => write!(f, "Schedule error: {}", e),
            Self::Discord(e) => write!(f, "Discord error: {}", e),
            Self::Api(e) => write!(f, "API error: {}", e),
            Self::Mcp(e) => write!(f, "MCP error: {}", e),
            Self::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for GatewayError {}

impl From<cc_core::Error> for GatewayError {
    fn from(e: cc_core::Error) -> Self {
        Self::Core(e)
    }
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, GatewayError>;
