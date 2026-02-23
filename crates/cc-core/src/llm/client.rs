//! LLM API HTTP Client
//!
//! Supports both Claude API and OpenAI-compatible APIs (GLM, etc.)

use reqwest::Client;
use tracing::{debug, info, warn};

use crate::config::{Config, LlmProvider};
use crate::error::{Error, Result};

use super::types::*;

/// LLM API client (supports Claude and OpenAI-compatible APIs)
#[derive(Clone)]
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    provider: LlmProvider,
}

impl ClaudeClient {
    /// Create a new LLM client
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(Error::Http)?;

        let llm_config = config.llm_config();

        // Determine base URL based on provider
        let base_url = match &llm_config.base_url {
            Some(url) => url.clone(),
            None => match llm_config.provider {
                LlmProvider::Claude => "https://api.anthropic.com/v1".to_string(),
                LlmProvider::OpenAi => "https://api.openai.com/v1".to_string(),
            },
        };

        Ok(Self {
            client,
            api_key: llm_config.api_key.clone(),
            model: llm_config.model.clone(),
            base_url,
            provider: llm_config.provider.clone(),
        })
    }

    /// Create with custom base URL (for testing or custom endpoints)
    pub fn with_base_url(config: &Config, base_url: String) -> Result<Self> {
        let mut client = Self::new(config)?;
        client.base_url = base_url;
        Ok(client)
    }

    /// Send a message to the LLM API
    pub async fn messages(
        &self,
        request: MessagesRequest,
    ) -> Result<MessagesResponse> {
        match self.provider {
            LlmProvider::Claude => self.send_claude_request(request).await,
            LlmProvider::OpenAi => self.send_openai_request(request).await,
        }
    }

    /// Send request to Claude API
    async fn send_claude_request(
        &self,
        request: MessagesRequest,
    ) -> Result<MessagesResponse> {
        let url = format!("{}/messages", self.base_url);

        debug!("Sending request to Claude API: {}", url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(Error::Http)?;

        let status = response.status();
        let body = response.text().await.map_err(Error::Http)?;

        if !status.is_success() {
            warn!("Claude API error: {} - {}", status, body);
            return Err(Error::ClaudeApi(format!("{}: {}", status, body)));
        }

        let parsed: MessagesResponse =
            serde_json::from_str(&body).map_err(|e| {
                Error::ClaudeApi(format!("Failed to parse response: {} - {}", e, body))
            })?;

        info!(
            "Claude API response: stop_reason={:?}, tokens={}",
            parsed.stop_reason,
            parsed.usage.as_ref().map(|u| u.output_tokens).unwrap_or(0)
        );

        Ok(parsed)
    }

    /// Send request to OpenAI-compatible API (GLM, etc.)
    async fn send_openai_request(
        &self,
        request: MessagesRequest,
    ) -> Result<MessagesResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        debug!("Sending request to OpenAI-compatible API: {}", url);

        // Convert to OpenAI format
        let openai_request = ChatCompletionRequest::from_claude_request(&request);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(Error::Http)?;

        let status = response.status();
        let body = response.text().await.map_err(Error::Http)?;

        if !status.is_success() {
            warn!("OpenAI API error: {} - {}", status, body);
            return Err(Error::ClaudeApi(format!("{}: {}", status, body)));
        }

        // Parse OpenAI response
        let openai_response: ChatCompletionResponse =
            serde_json::from_str(&body).map_err(|e| {
                Error::ClaudeApi(format!("Failed to parse response: {} - {}", e, body))
            })?;

        // Convert to Claude format
        let parsed = openai_response.to_claude_response();

        info!(
            "OpenAI API response: stop_reason={:?}, tokens={}",
            parsed.stop_reason,
            parsed.usage.as_ref().map(|u| u.output_tokens).unwrap_or(0)
        );

        Ok(parsed)
    }

    /// Create a messages request builder
    pub fn request_builder(&self) -> MessagesRequestBuilder {
        MessagesRequestBuilder::new(self.model.clone())
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the provider type
    pub fn provider(&self) -> &LlmProvider {
        &self.provider
    }

    /// Run the agent loop with tools
    pub async fn run_agent_loop(
        &self,
        messages: Vec<Message>,
        system: Option<String>,
        tools: Vec<ToolDefinition>,
        max_iterations: usize,
        tool_executor: impl Fn(&str, &serde_json::Value) -> Result<ToolResult>,
    ) -> Result<AgentLoopResult> {
        let mut current_messages = messages;
        let mut iterations = 0;
        let mut total_tokens = TokenUsage::default();

        loop {
            iterations += 1;
            if iterations > max_iterations {
                return Ok(AgentLoopResult {
                    final_response: "Max iterations reached".to_string(),
                    iterations,
                    total_tokens,
                    tool_calls: vec![],
                });
            }

            let request = MessagesRequest {
                model: self.model.clone(),
                max_tokens: 4096,
                system: system.clone(),
                messages: current_messages.clone(),
                tools: Some(tools.clone()),
            };

            let response = self.messages(request).await?;

            // Accumulate token usage
            if let Some(usage) = &response.usage {
                total_tokens.input_tokens += usage.input_tokens;
                total_tokens.output_tokens += usage.output_tokens;
            }

            match response.stop_reason.as_str() {
                "end_turn" | "stop_sequence" | "stop" => {
                    // Extract text response
                    let text = response
                        .content
                        .iter()
                        .filter_map(|c| {
                            if let MessageContent::Text { text } = c {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    return Ok(AgentLoopResult {
                        final_response: text,
                        iterations,
                        total_tokens,
                        tool_calls: vec![],
                    });
                }
                "tool_use" | "tool_calls" => {
                    // Process tool uses
                    let tool_uses: Vec<_> = response
                        .content
                        .iter()
                        .filter_map(|c| {
                            if let MessageContent::ToolUse { id, name, input } = c {
                                Some((id.clone(), name.clone(), input.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if tool_uses.is_empty() {
                        warn!("tool_use stop_reason but no tool_uses found");
                        continue;
                    }

                    // Execute tools and collect results
                    let mut tool_results = Vec::new();
                    for (id, name, input) in &tool_uses {
                        debug!("Executing tool: {} with input: {:?}", name, input);
                        let result = tool_executor(name, input)?;
                        tool_results.push(MessageContent::ToolResult {
                            tool_use_id: id.clone(),
                            content: result.output,
                            is_error: result.is_error,
                        });
                    }

                    // Add assistant message with tool_use
                    current_messages.push(Message {
                        role: "assistant".to_string(),
                        content: response.content.clone(),
                    });

                    // Add user message with tool_results
                    current_messages.push(Message {
                        role: "user".to_string(),
                        content: tool_results,
                    });
                }
                other => {
                    warn!("Unknown stop_reason: {}", other);
                    return Err(Error::ClaudeApi(format!(
                        "Unknown stop_reason: {}",
                        other
                    )));
                }
            }
        }
    }
}

/// Result of agent loop execution
#[derive(Debug)]
pub struct AgentLoopResult {
    pub final_response: String,
    pub iterations: usize,
    pub total_tokens: TokenUsage,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            is_error: false,
        }
    }

    pub fn error(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            is_error: true,
        }
    }
}
