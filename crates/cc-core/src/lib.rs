//! cc-core: Claude Code Gateway Core Library
//!
//! Claude APIとの通信、ツールシステム、セッション管理、
//! メモリシステム、サブエージェント、監査ログのコア機能を提供します。

pub mod agents;
pub mod audit;
pub mod config;
pub mod error;
pub mod llm;
pub mod memory;
pub mod session;
pub mod skills;
pub mod tool;

pub use agents::{
    AggregatedResult, AggregationStrategy, AgentCapability, DefaultSubAgent, DelegationConfig,
    ParallelExecutor, ResultAggregator, SubAgent, SubAgentId, SubAgentManager, SubAgentResult,
    SubAgentTask, SubAgentTaskBuilder, TaskDelegator, TaskId, TaskPriority, TaskStatus,
    ToolCallRecord,
};
pub use audit::{
    AuditConfig, AuditEntry, AuditEntryBuilder, AuditError, AuditEventType, AuditLevel,
    AuditLogger, AuditResult, AuditSource, AuditTarget, CryptoError, CryptoResult, EncryptedData,
    EncryptionAlgorithm, EncryptionConfig, SimpleEncryptor,
};
pub use config::{ApiConfig, Config, LlmConfig, LlmProvider, McpConfig, MemoryConfig, SchedulerConfig};
pub use error::{Error, Result};
pub use llm::{
    ClaudeClient, ImageSource, Message, MessageContent, MessagesRequest, MessagesRequestBuilder,
    MessagesResponse, ThinkingConfig, ThinkingLevel, ToolDefinition, Usage,
};
pub use memory::{Memory, MemoryStore};
pub use session::{Session, SessionManager, SessionStore};
pub use skills::{Skill, SkillConfig, SkillLoader};
pub use tool::{Tool, ToolManager, ToolResult};
