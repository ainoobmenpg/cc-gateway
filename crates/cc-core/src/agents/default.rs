//! Default Sub-Agent Implementation
//!
//! Provides a standard sub-agent implementation that uses ClaudeClient
//! for task execution.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

use super::types::{
    AgentCapability, SubAgent, SubAgentId, SubAgentResult, SubAgentTask, TaskStatus,
    ToolCallRecord,
};
use crate::config::Config;
use crate::llm::{ClaudeClient, Message, MessagesRequest};
use crate::tool::ToolManager;
use crate::Result;

/// Default sub-agent implementation using ClaudeClient
pub struct DefaultSubAgent {
    id: SubAgentId,
    name: String,
    description: String,
    capabilities: Vec<AgentCapability>,
    system_prompt: Option<String>,
    model_override: Option<String>,
    client: ClaudeClient,
    tool_manager: Arc<ToolManager>,
}

impl DefaultSubAgent {
    /// Create a new default sub-agent
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        config: &Config,
        tool_manager: Arc<ToolManager>,
    ) -> Result<Self> {
        let client = ClaudeClient::new(config)?;

        Ok(Self {
            id: SubAgentId::default(),
            name: name.into(),
            description: description.into(),
            capabilities: vec![],
            system_prompt: None,
            model_override: None,
            client,
            tool_manager,
        })
    }

    /// Create a builder for more configuration
    pub fn builder(
        name: impl Into<String>,
        config: &Config,
        tool_manager: Arc<ToolManager>,
    ) -> DefaultSubAgentBuilder {
        DefaultSubAgentBuilder::new(name, config, tool_manager)
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: AgentCapability) {
        self.capabilities.push(capability);
    }

    /// Set system prompt
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    /// Set model override
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model_override = Some(model.into());
    }

    /// Get the model to use
    fn get_model(&self) -> String {
        self.model_override
            .clone()
            .unwrap_or_else(|| self.client.model().to_string())
    }

    /// Execute with ClaudeClient agent loop
    async fn execute_agent_loop(
        &self,
        task: SubAgentTask,
    ) -> std::result::Result<(String, usize, u64, u64, Vec<ToolCallRecord>), String> {
        let model = self.get_model();
        let system = self.system_prompt.clone();
        let tools = if task.available_tools.is_empty() {
            self.tool_manager.definitions()
        } else {
            task.available_tools.clone()
        };

        let mut messages = task.context.clone();
        messages.push(Message::user(&task.instruction));

        let mut iterations = 0;
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut tool_calls = Vec::new();
        let tool_manager = self.tool_manager.clone();

        loop {
            iterations += 1;
            if iterations > task.max_iterations {
                return Err("Max iterations reached".to_string());
            }

            let request = MessagesRequest {
                model: model.clone(),
                max_tokens: task.max_tokens,
                system: system.clone(),
                messages: messages.clone(),
                tools: if tools.is_empty() {
                    None
                } else {
                    Some(tools.clone())
                },
                thinking: None,
            };

            let response = self
                .client
                .messages(request)
                .await
                .map_err(|e| e.to_string())?;

            // Track token usage
            if let Some(usage) = &response.usage {
                total_input += usage.input_tokens;
                total_output += usage.output_tokens;
            }

            match response.stop_reason.as_str() {
                "end_turn" | "stop_sequence" | "stop" => {
                    let text = response
                        .content
                        .iter()
                        .filter_map(|c| {
                            if let crate::llm::MessageContent::Text { text } = c {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    return Ok((text, iterations, total_input, total_output, tool_calls));
                }
                "tool_use" | "tool_calls" => {
                    let uses: Vec<_> = response
                        .content
                        .iter()
                        .filter_map(|c| {
                            if let crate::llm::MessageContent::ToolUse { id, name, input } = c {
                                Some((id.clone(), name.clone(), input.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if uses.is_empty() {
                        warn!("tool_use stop_reason but no tool_uses found");
                        continue;
                    }

                    let mut tool_results = Vec::new();
                    for (id, name, input) in &uses {
                        debug!("SubAgent executing tool: {} with input: {:?}", name, input);

                        let result = tool_manager
                            .execute(name, input.clone())
                            .await
                            .unwrap_or_else(|e| crate::tool::ToolResult::error(e.to_string()));

                        tool_calls.push(ToolCallRecord {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                            output: result.output.clone(),
                            is_error: result.is_error,
                        });

                        tool_results.push(crate::llm::MessageContent::ToolResult {
                            tool_use_id: id.clone(),
                            content: result.output,
                            is_error: result.is_error,
                        });
                    }

                    // Add assistant message
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: response.content,
                    });

                    // Add tool results
                    messages.push(Message {
                        role: "user".to_string(),
                        content: tool_results,
                    });
                }
                other => {
                    return Err(format!("Unknown stop_reason: {}", other));
                }
            }
        }
    }
}

#[async_trait]
impl SubAgent for DefaultSubAgent {
    fn id(&self) -> &SubAgentId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        self.capabilities.clone()
    }

    fn can_handle(&self, task: &SubAgentTask) -> bool {
        // Default agent can handle any task
        if self.capabilities.is_empty() {
            return true;
        }

        self.capabilities
            .iter()
            .any(|c| c.matches(&task.instruction))
    }

    fn system_prompt(&self) -> Option<String> {
        self.system_prompt.clone()
    }

    fn model(&self) -> Option<&str> {
        self.model_override.as_deref()
    }

    async fn execute(&self, task: SubAgentTask) -> Result<SubAgentResult> {
        let start_time = Instant::now();
        let task_id = task.id.clone();

        info!(
            "SubAgent '{}' executing task: {}",
            self.name,
            task_id.as_str()
        );

        match self.execute_agent_loop(task).await {
            Ok((output, iterations, input_tokens, output_tokens, tool_calls)) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;

                info!(
                    "SubAgent '{}' completed task in {}ms, {} iterations",
                    self.name, execution_time_ms, iterations
                );

                Ok(SubAgentResult {
                    task_id,
                    agent_id: self.id.clone(),
                    output,
                    success: true,
                    error: None,
                    iterations,
                    input_tokens,
                    output_tokens,
                    execution_time_ms,
                    status: TaskStatus::Completed,
                    tool_calls,
                })
            }
            Err(e) => {
                warn!("SubAgent '{}' failed: {}", self.name, e);

                Ok(SubAgentResult::failure(
                    task_id,
                    self.id.clone(),
                    e,
                    TaskStatus::Failed,
                ))
            }
        }
    }
}

/// Builder for DefaultSubAgent
pub struct DefaultSubAgentBuilder {
    name: String,
    description: String,
    capabilities: Vec<AgentCapability>,
    system_prompt: Option<String>,
    model_override: Option<String>,
    config: Config,
    tool_manager: Arc<ToolManager>,
}

impl DefaultSubAgentBuilder {
    pub fn new(
        name: impl Into<String>,
        config: &Config,
        tool_manager: Arc<ToolManager>,
    ) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            capabilities: vec![],
            system_prompt: None,
            model_override: None,
            config: config.clone(),
            tool_manager,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn capabilities(mut self, capabilities: Vec<AgentCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model_override = Some(model.into());
        self
    }

    pub fn build(self) -> Result<DefaultSubAgent> {
        let client = ClaudeClient::new(&self.config)?;

        Ok(DefaultSubAgent {
            id: SubAgentId::default(),
            name: self.name,
            description: self.description,
            capabilities: self.capabilities,
            system_prompt: self.system_prompt,
            model_override: self.model_override,
            client,
            tool_manager: self.tool_manager,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            api: crate::config::ApiConfig::default(),
            llm: crate::config::LlmConfig {
                provider: crate::config::LlmProvider::Claude,
                api_key: "test-key".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                base_url: None,
            },
            claude_api_key: "test-key".to_string(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            discord_token: None,
            admin_user_ids: vec![],
            api_key: None,
            mcp: crate::config::McpConfig::default(),
            memory: crate::config::MemoryConfig::default(),
            scheduler: crate::config::SchedulerConfig::default(),
        }
    }

    #[test]
    fn test_default_sub_agent_builder() {
        let config = create_test_config();
        let tool_manager = Arc::new(ToolManager::new());

        let result = DefaultSubAgent::builder("test_agent", &config, tool_manager)
            .description("Test agent for unit tests")
            .system_prompt("You are a test agent")
            .capability(AgentCapability::new("test", "Test capability"))
            .build();

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name(), "test_agent");
        assert_eq!(agent.description(), "Test agent for unit tests");
        assert_eq!(agent.system_prompt(), Some("You are a test agent".to_string()));
        assert_eq!(agent.capabilities().len(), 1);
    }

    #[test]
    fn test_default_sub_agent_can_handle() {
        let config = create_test_config();
        let tool_manager = Arc::new(ToolManager::new());

        let agent = DefaultSubAgent::builder("code_agent", &config, tool_manager)
            .capability(
                AgentCapability::new("code", "Code analysis")
                    .with_keywords(vec!["code".into(), "analyze".into()]),
            )
            .build()
            .unwrap();

        let code_task = SubAgentTask::new("Analyze this code for bugs");
        let weather_task = SubAgentTask::new("What's the weather today?");

        assert!(agent.can_handle(&code_task));
        // Agent with capabilities only handles matching tasks
        assert!(!agent.can_handle(&weather_task));
    }

    #[test]
    fn test_default_sub_agent_no_capabilities_handles_all() {
        let config = create_test_config();
        let tool_manager = Arc::new(ToolManager::new());

        let agent = DefaultSubAgent::builder("general", &config, tool_manager)
            .build()
            .unwrap();

        // Agent without capabilities handles all tasks
        let task1 = SubAgentTask::new("Do anything");
        let task2 = SubAgentTask::new("Something else");

        assert!(agent.can_handle(&task1));
        assert!(agent.can_handle(&task2));
    }
}
