//! Claude API types

use serde::{Deserialize, Serialize};

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<MessageContent>,
}

impl Message {
    /// Create a user message with text
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![MessageContent::Text { text: text.into() }],
        }
    }

    /// Create an assistant message with text
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![MessageContent::Text { text: text.into() }],
        }
    }

    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: vec![MessageContent::Text { text: text.into() }],
        }
    }

    /// Get text content from message
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| {
                if let MessageContent::Text { text } = c {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Content block in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { text: String },
    Image { source: ImageSource },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

/// Image source for multimodal input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Tool definition for Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Messages API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    pub model: String,
    #[serde(rename = "max_tokens")]
    pub max_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

/// Messages API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<MessageContent>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub stop_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

// ============================================================================
// OpenAI-compatible types (for GLM, etc.)
// ============================================================================

/// OpenAI-compatible chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

impl OpenAiMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: text.into(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: text.into(),
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: text.into(),
        }
    }
}

impl From<&Message> for OpenAiMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: msg.role.clone(),
            content: msg.text_content(),
        }
    }
}

/// OpenAI-compatible tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAiFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl From<&ToolDefinition> for OpenAiTool {
    fn from(tool: &ToolDefinition) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: OpenAiFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            },
        }
    }
}

/// OpenAI-compatible chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAiTool>>,
}

impl ChatCompletionRequest {
    /// Convert from Claude-style request
    pub fn from_claude_request(req: &MessagesRequest) -> Self {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &req.system {
            messages.push(OpenAiMessage::system(system));
        }

        // Convert messages
        for msg in &req.messages {
            messages.push(OpenAiMessage::from(msg));
        }

        // Convert tools
        let tools = req.tools.as_ref().map(|t| {
            t.iter().map(OpenAiTool::from).collect()
        });

        Self {
            model: req.model.clone(),
            messages,
            max_tokens: Some(req.max_tokens),
            tools,
        }
    }
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAiUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessageResponse,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCallResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallResponse {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

impl ChatCompletionResponse {
    /// Convert to Claude-style response
    pub fn to_claude_response(&self) -> MessagesResponse {
        let choice = self.choices.first();

        let content = match choice {
            Some(c) => {
                let mut content = Vec::new();

                // Add text content
                if let Some(text) = &c.message.content {
                    if !text.is_empty() {
                        content.push(MessageContent::Text { text: text.clone() });
                    }
                }

                // Add tool calls
                if let Some(tool_calls) = &c.message.tool_calls {
                    for tc in tool_calls {
                        let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                            .unwrap_or(serde_json::Value::Null);
                        content.push(MessageContent::ToolUse {
                            id: tc.id.clone(),
                            name: tc.function.name.clone(),
                            input: args,
                        });
                    }
                }

                content
            }
            None => vec![MessageContent::Text { text: String::new() }],
        };

        let stop_reason = choice
            .map(|c| match c.finish_reason.as_str() {
                "stop" => "end_turn".to_string(),
                "tool_calls" => "tool_use".to_string(),
                other => other.to_string(),
            })
            .unwrap_or_else(|| "end_turn".to_string());

        MessagesResponse {
            id: self.id.clone(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            content,
            model: self.model.clone(),
            stop_sequence: None,
            stop_reason,
            usage: self.usage.as_ref().map(|u| Usage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
            }),
        }
    }
}

/// Builder for creating messages requests
pub struct MessagesRequestBuilder {
    model: String,
    max_tokens: u64,
    system: Option<String>,
    messages: Vec<Message>,
    tools: Vec<ToolDefinition>,
}

impl MessagesRequestBuilder {
    pub fn new(model: String) -> Self {
        Self {
            model,
            max_tokens: 4096,
            system: None,
            messages: vec![],
            tools: vec![],
        }
    }

    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn max_tokens(mut self, max_tokens: u64) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    pub fn user(mut self, text: impl Into<String>) -> Self {
        self.messages.push(Message::user(text));
        self
    }

    pub fn assistant(mut self, text: impl Into<String>) -> Self {
        self.messages.push(Message::assistant(text));
        self
    }

    pub fn tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn build(self) -> MessagesRequest {
        MessagesRequest {
            model: self.model,
            max_tokens: self.max_tokens,
            system: self.system,
            messages: self.messages,
            tools: if self.tools.is_empty() {
                None
            } else {
                Some(self.tools)
            },
        }
    }
}
