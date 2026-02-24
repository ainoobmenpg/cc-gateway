//! Sub-Agent Architecture
//!
//! This module provides a sub-agent system for task delegation and parallel execution.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     SubAgentManager                          │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
//! │  │ Agent 1 │  │ Agent 2 │  │ Agent 3 │  │ Agent N │        │
//! │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
//! └───────┼────────────┼────────────┼────────────┼──────────────┘
//!         │            │            │            │
//!         ▼            ▼            ▼            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     TaskDelegator                            │
//! │  - Task splitting                                           │
//! │  - Capability-based routing                                  │
//! │  - Parallel execution                                        │
//! └─────────────────────────────────────────────────────────────┘
//!         │
//!         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   ResultAggregator                           │
//! │  - Combine outputs                                           │
//! │  - Track statistics                                          │
//! │  - Handle failures                                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cc_core::agents::{SubAgentManager, TaskDelegator, DefaultSubAgent};
//! use cc_core::tool::ToolManager;
//! use std::sync::Arc;
//!
//! // Create tool manager
//! let mut tool_manager = ToolManager::new();
//! // Register tools...
//!
//! // Create sub-agent manager
//! let mut manager = SubAgentManager::new();
//!
//! // Register agents
//! let agent = DefaultSubAgent::builder("code_reviewer", &config, Arc::new(tool_manager))
//!     .description("Reviews code for issues")
//!     .capability(AgentCapability::new("code_review", "Code review")
//!         .with_keywords(vec!["review".into(), "code".into()]))
//!     .system_prompt("You are a code reviewer...")
//!     .build()?;
//!
//! manager.register(Arc::new(agent));
//!
//! // Delegate tasks
//! let delegator = TaskDelegator::with_defaults(Arc::new(Mutex::new(manager)));
//! let task = SubAgentTask::new("Review this code for security issues");
//!
//! let result = delegator.delegate(task).await?;
//! println!("Result: {}", result.output);
//! ```

pub mod default;
pub mod delegation;
pub mod manager;
pub mod types;

// Re-exports
pub use default::DefaultSubAgent;
pub use delegation::{
    AggregatedResult, AggregationStrategy, DelegationConfig, ParallelExecutor, ResultAggregator,
    TaskDelegator,
};
pub use manager::SubAgentManager;
pub use types::{
    AgentCapability, SubAgent, SubAgentId, SubAgentResult, SubAgentTask, SubAgentTaskBuilder,
    TaskId, TaskPriority, TaskStatus, ToolCallRecord,
};
