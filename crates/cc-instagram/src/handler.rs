//! Message handler for Instagram bot

use std::sync::Arc;
use tracing::{debug, error, info, warn};

use cc_core::ClaudeClient;
use cc_core::Message;

use crate::api::{InstagramApi, WebhookMessagingEvent};
use crate::error::Result;
use crate::session::InMemorySessionStore;

/// Split a message into chunks at sentence boundaries
fn split_message(text: &str, max_size: usize) -> Vec<String> {
    if text.len() <= max_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= max_size {
            chunks.push(remaining.to_string());
            break;
        }

        // Find a good break point
        let search_end = max_size.min(remaining.len());
        let chunk = &remaining[..search_end];

        // Try to break at sentence end
        let break_point = chunk
            .rfind("。")
            .or_else(|| chunk.rfind("!"))
            .or_else(|| chunk.rfind("?"))
            .or_else(|| chunk.rfind("\n\n"))
            .or_else(|| chunk.rfind("\n"))
            .or_else(|| chunk.rfind(" "))
            .map(|i| i + 1)
            .unwrap_or(max_size);

        chunks.push(remaining[..break_point].to_string());
        remaining = &remaining[break_point..];
    }

    chunks
}

/// Handler state shared across requests
#[derive(Clone)]
pub struct HandlerState {
    pub claude_client: Arc<ClaudeClient>,
    pub session_store: InMemorySessionStore,
    pub admin_psids: Vec<String>,
}

/// Instagram message handler
#[derive(Clone)]
pub struct InstagramHandler {
    api: InstagramApi,
    state: Arc<HandlerState>,
}

impl InstagramHandler {
    /// Create a new Instagram handler
    pub fn new(
        api: InstagramApi,
        claude_client: Arc<ClaudeClient>,
        admin_psids: Vec<String>,
    ) -> Self {
        let state = Arc::new(HandlerState {
            claude_client,
            session_store: InMemorySessionStore::new(),
            admin_psids,
        });

        Self {
            api,
            state,
        }
    }

    /// Handle incoming webhook event
    pub async fn handle_event(&self, event: WebhookMessagingEvent) -> Result<Option<String>> {
        let sender_psid = event.sender.id;

        // Check admin permissions
        if !self.state.admin_psids.is_empty()
            && !self.state.admin_psids.contains(&sender_psid)
        {
            debug!("Ignoring message from non-admin PSID: {}", sender_psid);
            return Ok(None);
        }

        // Get message text
        let message_text = match &event.message {
            Some(msg) => msg.text.clone().unwrap_or_default(),
            None => return Ok(None),
        };

        if message_text.is_empty() {
            return Ok(None);
        }

        info!(
            "Processing message from PSID {}: {}",
            sender_psid, message_text
        );

        // Get or create session
        let session = self.state.session_store.get_or_create(&sender_psid).await;

        // Build message history
        let mut messages: Vec<Message> = session.messages.clone();
        messages.push(Message::user(&message_text));

        // Build request with conversation history
        let mut request_builder = self
            .state
            .claude_client
            .request_builder()
            .system("You are a helpful assistant. Respond in the same language as the user's question. Keep track of the conversation context.")
            .max_tokens(4096);

        // Add conversation history (limit to last 20 messages)
        let history_start = messages.len().saturating_sub(20);
        for message in messages.into_iter().skip(history_start) {
            request_builder = request_builder.message(message);
        }

        let request = request_builder.build();

        match self.state.claude_client.messages(request).await {
            Ok(response) => {
                // Extract text from response
                let text = response
                    .content
                    .iter()
                    .filter_map(|c| {
                        if let cc_core::MessageContent::Text { text } = c {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Update session
                self.state
                    .session_store
                    .add_message(&sender_psid, Message::user(&message_text))
                    .await;
                self.state
                    .session_store
                    .add_message(&sender_psid, Message::assistant(&text))
                    .await;

                // Send response via Instagram API
                // Instagram messages have a limit, typically 2000 characters
                let max_size = 1900;
                if text.len() <= max_size {
                    match self.api.send_message(&sender_psid, &text).await {
                        Ok(response) => {
                            info!("Sent message to {}: {:?}", sender_psid, response.message_id);
                        }
                        Err(e) => {
                            error!("Failed to send message: {:?}", e);
                        }
                    }
                } else {
                    // Split into chunks
                    let chunks = split_message(&text, max_size);
                    for (i, chunk) in chunks.iter().enumerate() {
                        let content = if i == 0 {
                            chunk.clone()
                        } else {
                            format!("(continued {})\n{}", i + 1, chunk)
                        };

                        if let Err(e) = self.api.send_message(&sender_psid, &content).await {
                            error!("Failed to send message chunk {}: {:?}", i, e);
                            break;
                        }

                        // Small delay between chunks
                        if i < chunks.len() - 1 {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    }
                }

                Ok(Some(text))
            }
            Err(e) => {
                error!("Claude API error: {:?}", e);
                let error_message = format!("Error occurred: {}", e);
                if let Err(e) = self.api.send_message(&sender_psid, &error_message).await {
                    warn!("Failed to send error message: {:?}", e);
                }
                Err(e.into())
            }
        }
    }

    /// Handle a text message directly (for testing or direct API calls)
    pub async fn handle_message(&self, psid: &str, text: &str) -> Result<Option<String>> {
        let event = WebhookMessagingEvent {
            sender: crate::api::WebhookSender {
                id: psid.to_string(),
            },
            recipient: crate::api::WebhookRecipient {
                id: self.api.page_id().to_string(),
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
            message: Some(crate::api::WebhookMessage {
                mid: Some(uuid::Uuid::new_v4().to_string()),
                text: Some(text.to_string()),
                attachments: None,
            }),
        };

        self.handle_event(event).await
    }

    /// Clear conversation history for a user
    pub async fn clear_session(&self, psid: &str) {
        self.state.session_store.clear(psid).await;
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        self.state.session_store.session_count().await
    }

    /// Get the Instagram API client
    pub fn api(&self) -> &InstagramApi {
        &self.api
    }

    /// Get the handler state
    pub fn state(&self) -> &Arc<HandlerState> {
        &self.state
    }
}
