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

    /// Create a user message with text and image
    pub fn user_with_image(text: impl Into<String>, image: ImageSource) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![
                MessageContent::Text { text: text.into() },
                MessageContent::Image { source: image },
            ],
        }
    }

    /// Create a user message with multiple images
    pub fn user_with_images(text: impl Into<String>, images: Vec<ImageSource>) -> Self {
        let mut content = vec![MessageContent::Text { text: text.into() }];
        for image in images {
            content.push(MessageContent::Image { source: image });
        }
        Self {
            role: "user".to_string(),
            content,
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

    /// Check if message contains images
    pub fn has_images(&self) -> bool {
        self.content.iter().any(|c| matches!(c, MessageContent::Image { .. }))
    }

    /// Get image count
    pub fn image_count(&self) -> usize {
        self.content.iter().filter(|c| matches!(c, MessageContent::Image { .. })).count()
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
    /// Extended thinking block (Claude 3.7+)
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Redacted thinking block (when thinking is sensitive)
    RedactedThinking {
        data: String,
    },
}

impl MessageContent {
    /// Check if this is a thinking block
    pub fn is_thinking(&self) -> bool {
        matches!(self, Self::Thinking { .. } | Self::RedactedThinking { .. })
    }

    /// Get thinking content if this is a thinking block
    pub fn thinking_content(&self) -> Option<&str> {
        match self {
            Self::Thinking { thinking, .. } => Some(thinking),
            _ => None,
        }
    }
}

/// Image source for multimodal input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

impl ImageSource {
    /// Supported image media types
    pub const MEDIA_TYPE_PNG: &'static str = "image/png";
    pub const MEDIA_TYPE_JPEG: &'static str = "image/jpeg";
    pub const MEDIA_TYPE_GIF: &'static str = "image/gif";
    pub const MEDIA_TYPE_WEBP: &'static str = "image/webp";

    /// Create a new image source from base64 data
    pub fn base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            source_type: "base64".to_string(),
            media_type: media_type.into(),
            data: data.into(),
        }
    }

    /// Create an image source from raw bytes (encodes to base64)
    pub fn from_bytes(media_type: impl Into<String>, bytes: &[u8]) -> Self {
        Self {
            source_type: "base64".to_string(),
            media_type: media_type.into(),
            data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes),
        }
    }

    /// Create a PNG image source from bytes
    pub fn png(bytes: &[u8]) -> Self {
        Self::from_bytes(Self::MEDIA_TYPE_PNG, bytes)
    }

    /// Create a JPEG image source from bytes
    pub fn jpeg(bytes: &[u8]) -> Self {
        Self::from_bytes(Self::MEDIA_TYPE_JPEG, bytes)
    }

    /// Create a GIF image source from bytes
    pub fn gif(bytes: &[u8]) -> Self {
        Self::from_bytes(Self::MEDIA_TYPE_GIF, bytes)
    }

    /// Create a WebP image source from bytes
    pub fn webp(bytes: &[u8]) -> Self {
        Self::from_bytes(Self::MEDIA_TYPE_WEBP, bytes)
    }

    /// Create from a data URL (e.g., "data:image/png;base64,....")
    pub fn from_data_url(data_url: &str) -> Option<Self> {
        let prefix = "data:";
        if !data_url.starts_with(prefix) {
            return None;
        }

        let rest = &data_url[prefix.len()..];
        let comma_pos = rest.find(',')?;
        let header = &rest[..comma_pos];
        let data = &rest[comma_pos + 1..];

        // Parse media type from header
        let media_type = if let Some(semicolon_pos) = header.find(';') {
            &header[..semicolon_pos]
        } else {
            header
        };

        Some(Self::base64(media_type, data))
    }

    /// Decode base64 data to bytes
    pub fn decode(&self) -> Option<Vec<u8>> {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &self.data).ok()
    }

    /// Convert to a data URL
    pub fn to_data_url(&self) -> String {
        format!("data:{};base64,{}", self.media_type, self.data)
    }

    /// Get approximate size in bytes (decoded)
    pub fn approximate_size(&self) -> usize {
        // Base64 is ~4/3 the size of original, so decoded size is ~3/4
        (self.data.len() * 3) / 4
    }
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

/// Thinking configuration for Claude 3.7+ extended thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// Thinking type: "enabled" or "disabled"
    #[serde(rename = "type")]
    pub thinking_type: String,
    /// Budget tokens for thinking (minimum 1024 when enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<u64>,
}

impl ThinkingConfig {
    /// Default thinking budget
    pub const DEFAULT_BUDGET: u64 = 1024;
    /// Maximum thinking budget
    pub const MAX_BUDGET: u64 = 128_000;
    /// Minimum thinking budget
    pub const MIN_BUDGET: u64 = 1024;

    /// Create a disabled thinking config
    pub fn disabled() -> Self {
        Self {
            thinking_type: "disabled".to_string(),
            budget_tokens: None,
        }
    }

    /// Create an enabled thinking config with default budget
    pub fn enabled() -> Self {
        Self::with_budget(Self::DEFAULT_BUDGET)
    }

    /// Create an enabled thinking config with custom budget
    pub fn with_budget(budget_tokens: u64) -> Self {
        let budget = budget_tokens.clamp(Self::MIN_BUDGET, Self::MAX_BUDGET);
        Self {
            thinking_type: "enabled".to_string(),
            budget_tokens: Some(budget),
        }
    }

    /// Check if thinking is enabled
    pub fn is_enabled(&self) -> bool {
        self.thinking_type == "enabled"
    }
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Thinking level preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingLevel {
    /// No extended thinking
    #[default]
    None,
    /// Light thinking (1024 tokens)
    Light,
    /// Medium thinking (8192 tokens)
    Medium,
    /// Heavy thinking (32768 tokens)
    Heavy,
    /// Maximum thinking (128000 tokens)
    Max,
}

impl ThinkingLevel {
    /// Convert to ThinkingConfig
    pub fn to_config(&self) -> ThinkingConfig {
        match self {
            Self::None => ThinkingConfig::disabled(),
            Self::Light => ThinkingConfig::with_budget(1024),
            Self::Medium => ThinkingConfig::with_budget(8192),
            Self::Heavy => ThinkingConfig::with_budget(32768),
            Self::Max => ThinkingConfig::with_budget(128_000),
        }
    }

    /// Get budget tokens for this level
    pub fn budget_tokens(&self) -> u64 {
        match self {
            Self::None => 0,
            Self::Light => 1024,
            Self::Medium => 8192,
            Self::Heavy => 32768,
            Self::Max => 128_000,
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
    /// Extended thinking configuration (Claude 3.7+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
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
    /// Thinking tokens used (Claude 3.7+ extended thinking)
    #[serde(default)]
    pub thinking_tokens: u64,
    /// Cache read tokens (if prompt caching is used)
    #[serde(default)]
    pub cache_read_tokens: u64,
    /// Cache write tokens (if prompt caching is used)
    #[serde(default)]
    pub cache_write_tokens: u64,
}

impl Usage {
    /// Get total tokens (input + output + thinking)
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.thinking_tokens
    }

    /// Calculate approximate cost in dollars (Claude 3.7 Sonnet pricing)
    /// Input: $3/M, Output: $15/M
    pub fn estimated_cost_dollars(&self) -> f64 {
        let input_cost = (self.input_tokens as f64) / 1_000_000.0 * 3.0;
        let output_cost = (self.output_tokens as f64) / 1_000_000.0 * 15.0;
        let thinking_cost = (self.thinking_tokens as f64) / 1_000_000.0 * 15.0; // Thinking counts as output
        input_cost + output_cost + thinking_cost
    }
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
                thinking_tokens: 0,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
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
    thinking: Option<ThinkingConfig>,
}

impl MessagesRequestBuilder {
    pub fn new(model: String) -> Self {
        Self {
            model,
            max_tokens: 4096,
            system: None,
            messages: vec![],
            tools: vec![],
            thinking: None,
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

    /// Enable extended thinking with default budget
    pub fn thinking(mut self) -> Self {
        self.thinking = Some(ThinkingConfig::enabled());
        self
    }

    /// Enable extended thinking with custom budget
    pub fn thinking_with_budget(mut self, budget_tokens: u64) -> Self {
        self.thinking = Some(ThinkingConfig::with_budget(budget_tokens));
        self
    }

    /// Set thinking level
    pub fn thinking_level(mut self, level: ThinkingLevel) -> Self {
        if level == ThinkingLevel::None {
            self.thinking = None;
        } else {
            self.thinking = Some(level.to_config());
        }
        self
    }

    /// Set thinking configuration directly
    pub fn thinking_config(mut self, config: ThinkingConfig) -> Self {
        self.thinking = Some(config);
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
            thinking: self.thinking,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_source_base64() {
        let img = ImageSource::base64("image/png", "dGVzdA==");
        assert_eq!(img.source_type, "base64");
        assert_eq!(img.media_type, "image/png");
        assert_eq!(img.data, "dGVzdA==");
    }

    #[test]
    fn test_image_source_from_bytes() {
        let bytes = b"test image data";
        let img = ImageSource::png(bytes);
        assert_eq!(img.source_type, "base64");
        assert_eq!(img.media_type, "image/png");
        // Verify it's valid base64
        assert!(img.decode().is_some());
    }

    #[test]
    fn test_image_source_decode() {
        let original = b"Hello, World!";
        let img = ImageSource::jpeg(original);
        let decoded = img.decode().unwrap();
        assert_eq!(decoded.as_slice(), original);
    }

    #[test]
    fn test_image_source_data_url() {
        let data_url = "data:image/png;base64,dGVzdA==";
        let img = ImageSource::from_data_url(data_url).unwrap();
        assert_eq!(img.media_type, "image/png");
        assert_eq!(img.data, "dGVzdA==");
    }

    #[test]
    fn test_image_source_to_data_url() {
        let img = ImageSource::base64("image/jpeg", "dGVzdA==");
        let data_url = img.to_data_url();
        assert_eq!(data_url, "data:image/jpeg;base64,dGVzdA==");
    }

    #[test]
    fn test_image_source_approximate_size() {
        // Base64 "dGVzdA==" is 8 chars, representing 6 bytes of original data
        let img = ImageSource::base64("image/png", "dGVzdA==");
        assert_eq!(img.approximate_size(), 6); // 8 * 3 / 4 = 6
    }

    #[test]
    fn test_image_source_shortcuts() {
        let bytes = b"test";

        let png = ImageSource::png(bytes);
        assert_eq!(png.media_type, ImageSource::MEDIA_TYPE_PNG);

        let jpeg = ImageSource::jpeg(bytes);
        assert_eq!(jpeg.media_type, ImageSource::MEDIA_TYPE_JPEG);

        let gif = ImageSource::gif(bytes);
        assert_eq!(gif.media_type, ImageSource::MEDIA_TYPE_GIF);

        let webp = ImageSource::webp(bytes);
        assert_eq!(webp.media_type, ImageSource::MEDIA_TYPE_WEBP);
    }

    #[test]
    fn test_message_user_with_image() {
        let img = ImageSource::png(b"test");
        let msg = Message::user_with_image("What's in this image?", img);

        assert_eq!(msg.role, "user");
        assert!(msg.has_images());
        assert_eq!(msg.image_count(), 1);
        // 1 text + 1 image = 2 content blocks
        assert_eq!(msg.content.len(), 2);
    }

    #[test]
    fn test_message_user_with_images() {
        let img1 = ImageSource::png(b"test1");
        let img2 = ImageSource::jpeg(b"test2");
        let msg = Message::user_with_images("Compare these", vec![img1, img2]);

        assert_eq!(msg.role, "user");
        // 1 text + 2 images = 3 content blocks
        assert_eq!(msg.content.len(), 3);
    }

    #[test]
    fn test_message_has_images() {
        let msg_text = Message::user("Hello");
        assert!(!msg_text.has_images());
        assert_eq!(msg_text.image_count(), 0);

        let img = ImageSource::png(b"test");
        let msg_with_img = Message::user_with_image("Look at this", img);
        assert!(msg_with_img.has_images());
        assert_eq!(msg_with_img.image_count(), 1);
    }

    #[test]
    fn test_image_serialization() {
        let img = ImageSource::base64("image/png", "dGVzdA==");
        let json = serde_json::to_string(&img).unwrap();
        assert!(json.contains(r#""type":"base64""#));
        assert!(json.contains(r#""media_type":"image/png""#));
    }

    #[test]
    fn test_message_with_image_serialization() {
        let img = ImageSource::png(b"test");
        let msg = Message::user_with_image("Describe this", img);
        let json = serde_json::to_string(&msg).unwrap();

        // Should contain both text and image content
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""type":"image""#));
    }

    // Thinking tests

    #[test]
    fn test_thinking_config_disabled() {
        let config = ThinkingConfig::disabled();
        assert!(!config.is_enabled());
        assert!(config.budget_tokens.is_none());
    }

    #[test]
    fn test_thinking_config_enabled() {
        let config = ThinkingConfig::enabled();
        assert!(config.is_enabled());
        assert_eq!(config.budget_tokens, Some(1024));
    }

    #[test]
    fn test_thinking_config_with_budget() {
        let config = ThinkingConfig::with_budget(8192);
        assert!(config.is_enabled());
        assert_eq!(config.budget_tokens, Some(8192));
    }

    #[test]
    fn test_thinking_config_budget_clamping() {
        // Below minimum
        let config = ThinkingConfig::with_budget(100);
        assert_eq!(config.budget_tokens, Some(1024));

        // Above maximum
        let config = ThinkingConfig::with_budget(1_000_000);
        assert_eq!(config.budget_tokens, Some(128_000));
    }

    #[test]
    fn test_thinking_level_to_config() {
        assert!(!ThinkingLevel::None.to_config().is_enabled());
        assert!(ThinkingLevel::Light.to_config().is_enabled());
        assert_eq!(ThinkingLevel::Medium.budget_tokens(), 8192);
        assert_eq!(ThinkingLevel::Heavy.budget_tokens(), 32768);
        assert_eq!(ThinkingLevel::Max.budget_tokens(), 128_000);
    }

    #[test]
    fn test_message_content_thinking() {
        let thinking = MessageContent::Thinking {
            thinking: "Let me think about this...".to_string(),
            signature: Some("sig123".to_string()),
        };

        assert!(thinking.is_thinking());
        assert_eq!(thinking.thinking_content(), Some("Let me think about this..."));
    }

    #[test]
    fn test_message_content_redacted_thinking() {
        let redacted = MessageContent::RedactedThinking {
            data: "redacted_data".to_string(),
        };

        assert!(redacted.is_thinking());
        assert!(redacted.thinking_content().is_none());
    }

    #[test]
    fn test_usage_with_thinking() {
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 50,
            thinking_tokens: 200,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        assert_eq!(usage.total_tokens(), 350);

        // Test cost calculation (approximately)
        let cost = usage.estimated_cost_dollars();
        assert!(cost > 0.0);
        // 100 * 3/1M + 250 * 15/1M = 0.0003 + 0.00375 = 0.00405
        assert!(cost < 0.01);
    }

    #[test]
    fn test_messages_request_with_thinking() {
        let request = MessagesRequestBuilder::new("claude-sonnet-4-20250514".to_string())
            .user("What is 2+2?")
            .thinking()
            .build();

        assert!(request.thinking.is_some());
        assert!(request.thinking.unwrap().is_enabled());
    }

    #[test]
    fn test_messages_request_with_thinking_level() {
        let request = MessagesRequestBuilder::new("claude-sonnet-4-20250514".to_string())
            .user("Solve this complex problem")
            .thinking_level(ThinkingLevel::Heavy)
            .build();

        let thinking = request.thinking.unwrap();
        assert!(thinking.is_enabled());
        assert_eq!(thinking.budget_tokens, Some(32768));
    }

    #[test]
    fn test_messages_request_without_thinking() {
        let request = MessagesRequestBuilder::new("claude-sonnet-4-20250514".to_string())
            .user("Hello")
            .build();

        assert!(request.thinking.is_none());
    }

    #[test]
    fn test_thinking_serialization() {
        let config = ThinkingConfig::with_budget(8192);
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains(r#""type":"enabled""#));
        assert!(json.contains(r#""budget_tokens":8192"#));
    }

    #[test]
    fn test_thinking_content_serialization() {
        let content = MessageContent::Thinking {
            thinking: "I need to analyze this...".to_string(),
            signature: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"thinking""#));
        assert!(json.contains("I need to analyze this"));
    }
}
