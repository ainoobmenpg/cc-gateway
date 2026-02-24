//! Audit log entry types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditLevel {
    /// Informational events (normal operations)
    Info,
    /// Warning events (potential issues)
    Warning,
    /// Error events (failures)
    Error,
    /// Critical security events (security incidents)
    Critical,
}

/// Types of audit events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    ApiKeyCreated,
    ApiKeyRevoked,
    ApiKeyUsed,
    AuthenticationSuccess,
    AuthenticationFailure,

    // Session events
    SessionCreated,
    SessionAccessed,
    SessionDeleted,
    SessionExpired,

    // Message events
    MessageSent,
    MessageReceived,
    MessageFiltered,
    ToolExecuted,

    // Configuration events
    ConfigChanged,
    GatewayStarted,
    GatewayStopped,

    // Security events
    RateLimitExceeded,
    SuspiciousActivity,
    AccessDenied,
    EncryptionEnabled,
    EncryptionDisabled,

    // Admin events
    UserCreated,
    UserDeleted,
    PermissionChanged,
}

/// Source of an audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSource {
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Gateway/platform that originated the event
    pub gateway: Option<String>,
    /// Channel ID (for messaging platforms)
    pub channel_id: Option<String>,
}

/// Target of an audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTarget {
    /// Resource type affected
    pub resource_type: String,
    /// Resource identifier
    pub resource_id: Option<String>,
    /// Action performed on the resource
    pub action: String,
}

/// A single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Severity level
    pub level: AuditLevel,
    /// Human-readable message
    pub message: String,
    /// Source of the event
    pub source: Option<AuditSource>,
    /// Target of the event
    pub target: Option<AuditTarget>,
    /// Additional metadata as JSON
    pub metadata: Option<serde_json::Value>,
    /// Request/correlation ID for tracing
    pub correlation_id: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(event_type: AuditEventType, level: AuditLevel, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            level,
            message: message.into(),
            source: None,
            target: None,
            metadata: None,
            correlation_id: None,
        }
    }

    /// Add source information
    pub fn with_source(mut self, source: AuditSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Add target information
    pub fn with_target(mut self, target: AuditTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Audit log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log file path (None = no file logging)
    pub log_file: Option<String>,
    /// Maximum log file size in bytes before rotation
    pub max_file_size: usize,
    /// Number of rotated log files to keep
    pub max_rotated_files: usize,
    /// Minimum level to log
    pub min_level: AuditLevel,
    /// Include sensitive data in logs (use with caution)
    pub include_sensitive: bool,
    /// Log to stdout/stderr
    pub log_to_console: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_file: Some("logs/audit.log".to_string()),
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_rotated_files: 5,
            min_level: AuditLevel::Info,
            include_sensitive: false,
            log_to_console: true,
        }
    }
}
