//! Sub-Agent types and trait definitions
//!
//! Defines the core types for the sub-agent architecture:
//! - SubAgent trait: Interface for specialized agents
//! - SubAgentTask: Task representation for delegation
//! - SubAgentResult: Result from sub-agent execution

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::llm::{Message, ToolDefinition};
use crate::Result;

/// Unique identifier for a sub-agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubAgentId(pub String);

impl SubAgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SubAgentId {
    fn default() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }
}

/// Unique identifier for a delegated task
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }
}

/// Priority level for task execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl TaskPriority {
    pub fn weight(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Normal => 5,
            Self::High => 10,
            Self::Critical => 20,
        }
    }
}

/// Status of a delegated task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Task to be delegated to a sub-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentTask {
    /// Unique task identifier
    pub id: TaskId,
    /// Task description/instruction
    pub instruction: String,
    /// Conversation history context
    pub context: Vec<Message>,
    /// Tools available to this task
    pub available_tools: Vec<ToolDefinition>,
    /// Priority level
    pub priority: TaskPriority,
    /// Maximum iterations for the agent loop
    pub max_iterations: usize,
    /// Maximum tokens for the response
    pub max_tokens: u64,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Metadata for task tracking
    pub metadata: HashMap<String, String>,
}

impl SubAgentTask {
    /// Create a new task with default settings
    pub fn new(instruction: impl Into<String>) -> Self {
        Self {
            id: TaskId::default(),
            instruction: instruction.into(),
            context: vec![],
            available_tools: vec![],
            priority: TaskPriority::Normal,
            max_iterations: 10,
            max_tokens: 4096,
            timeout_secs: 120,
            metadata: HashMap::new(),
        }
    }

    /// Create a task builder for more configuration
    pub fn builder(instruction: impl Into<String>) -> SubAgentTaskBuilder {
        SubAgentTaskBuilder::new(instruction)
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set available tools
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.available_tools = tools;
        self
    }

    /// Add context messages
    pub fn with_context(mut self, messages: Vec<Message>) -> Self {
        self.context = messages;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }
}

/// Builder for SubAgentTask
pub struct SubAgentTaskBuilder {
    instruction: String,
    context: Vec<Message>,
    tools: Vec<ToolDefinition>,
    priority: TaskPriority,
    max_iterations: usize,
    max_tokens: u64,
    timeout_secs: u64,
    metadata: HashMap<String, String>,
}

impl SubAgentTaskBuilder {
    pub fn new(instruction: impl Into<String>) -> Self {
        Self {
            instruction: instruction.into(),
            context: vec![],
            tools: vec![],
            priority: TaskPriority::Normal,
            max_iterations: 10,
            max_tokens: 4096,
            timeout_secs: 120,
            metadata: HashMap::new(),
        }
    }

    pub fn context(mut self, messages: Vec<Message>) -> Self {
        self.context = messages;
        self
    }

    pub fn add_context(mut self, message: Message) -> Self {
        self.context.push(message);
        self
    }

    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    pub fn add_tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }

    pub fn max_tokens(mut self, tokens: u64) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> SubAgentTask {
        SubAgentTask {
            id: TaskId::default(),
            instruction: self.instruction,
            context: self.context,
            available_tools: self.tools,
            priority: self.priority,
            max_iterations: self.max_iterations,
            max_tokens: self.max_tokens,
            timeout_secs: self.timeout_secs,
            metadata: self.metadata,
        }
    }
}

/// Result from sub-agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResult {
    /// Task that was executed
    pub task_id: TaskId,
    /// Sub-agent that executed the task
    pub agent_id: SubAgentId,
    /// Final response/output
    pub output: String,
    /// Whether execution was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Number of iterations used
    pub iterations: usize,
    /// Token usage
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Status of the result
    pub status: TaskStatus,
    /// Tool calls made during execution
    pub tool_calls: Vec<ToolCallRecord>,
}

impl SubAgentResult {
    /// Create a successful result
    pub fn success(
        task_id: TaskId,
        agent_id: SubAgentId,
        output: impl Into<String>,
        iterations: usize,
        input_tokens: u64,
        output_tokens: u64,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            task_id,
            agent_id,
            output: output.into(),
            success: true,
            error: None,
            iterations,
            input_tokens,
            output_tokens,
            execution_time_ms,
            status: TaskStatus::Completed,
            tool_calls: vec![],
        }
    }

    /// Create a failed result
    pub fn failure(
        task_id: TaskId,
        agent_id: SubAgentId,
        error: impl Into<String>,
        status: TaskStatus,
    ) -> Self {
        Self {
            task_id,
            agent_id,
            output: String::new(),
            success: false,
            error: Some(error.into()),
            iterations: 0,
            input_tokens: 0,
            output_tokens: 0,
            execution_time_ms: 0,
            status,
            tool_calls: vec![],
        }
    }

    /// Create a timeout result
    pub fn timeout(task_id: TaskId, agent_id: SubAgentId) -> Self {
        Self::failure(
            task_id,
            agent_id,
            "Task execution timed out",
            TaskStatus::Timeout,
        )
    }
}

/// Record of a tool call during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub output: String,
    pub is_error: bool,
}

/// Capability of a sub-agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Unique capability name
    pub name: String,
    /// Description of the capability
    pub description: String,
    /// Keywords for capability matching
    pub keywords: Vec<String>,
}

impl AgentCapability {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            keywords: vec![],
        }
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Check if a task matches this capability
    pub fn matches(&self, instruction: &str) -> bool {
        let lower = instruction.to_lowercase();
        self.keywords.iter().any(|k| lower.contains(&k.to_lowercase()))
            || lower.contains(&self.name.to_lowercase())
    }
}

/// Sub-Agent trait for specialized task execution
#[async_trait]
pub trait SubAgent: Send + Sync + 'static {
    /// Get the agent's unique identifier
    fn id(&self) -> &SubAgentId;

    /// Get the agent's name
    fn name(&self) -> &str;

    /// Get the agent's description
    fn description(&self) -> &str;

    /// Get the agent's capabilities
    fn capabilities(&self) -> Vec<AgentCapability>;

    /// Check if the agent can handle a given task
    fn can_handle(&self, task: &SubAgentTask) -> bool {
        self.capabilities()
            .iter()
            .any(|c| c.matches(&task.instruction))
    }

    /// Get the system prompt for this agent
    fn system_prompt(&self) -> Option<String> {
        None
    }

    /// Get the model to use for this agent
    fn model(&self) -> Option<&str> {
        None
    }

    /// Execute a delegated task
    async fn execute(&self, task: SubAgentTask) -> Result<SubAgentResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_default() {
        let id1 = TaskId::default();
        let id2 = TaskId::default();
        assert_ne!(id1, id2); // UUIDs should be unique
    }

    #[test]
    fn test_sub_agent_id_default() {
        let id1 = SubAgentId::default();
        let id2 = SubAgentId::default();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_priority_weight() {
        assert!(TaskPriority::Critical.weight() > TaskPriority::High.weight());
        assert!(TaskPriority::High.weight() > TaskPriority::Normal.weight());
        assert!(TaskPriority::Normal.weight() > TaskPriority::Low.weight());
    }

    #[test]
    fn test_sub_agent_task_builder() {
        let task = SubAgentTask::builder("Test task")
            .priority(TaskPriority::High)
            .max_iterations(5)
            .timeout(60)
            .metadata("key", "value")
            .build();

        assert_eq!(task.instruction, "Test task");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.max_iterations, 5);
        assert_eq!(task.timeout_secs, 60);
        assert_eq!(task.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_agent_capability_matches() {
        let cap = AgentCapability::new("code_review", "Reviews code for issues")
            .with_keywords(vec!["review".to_string(), "code".to_string()]);

        assert!(cap.matches("Please review this code"));
        assert!(cap.matches("I need a code review"));
        assert!(!cap.matches("What's the weather?"));
    }

    #[test]
    fn test_sub_agent_result_success() {
        let result = SubAgentResult::success(
            TaskId::new("task-1"),
            SubAgentId::new("agent-1"),
            "Task completed",
            3,
            100,
            50,
            1500,
        );

        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.iterations, 3);
        assert_eq!(result.status, TaskStatus::Completed);
    }

    #[test]
    fn test_sub_agent_result_failure() {
        let result = SubAgentResult::failure(
            TaskId::new("task-1"),
            SubAgentId::new("agent-1"),
            "Something went wrong",
            TaskStatus::Failed,
        );

        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(result.status, TaskStatus::Failed);
    }
}
