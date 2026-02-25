//! Message handler for Facebook Messenger

use std::sync::Arc;
use tracing::{debug, error, info, warn};

use cc_core::{ClaudeClient, Message as CoreMessage, MessageContent};

use crate::api::{FacebookApi, WebhookEntry, WebhookMessaging};
use crate::error::Result;
use crate::session::InMemorySessionStore;

/// Facebook message handler
pub struct FacebookHandler {
    api: FacebookApi,
    session_store: InMemorySessionStore,
    claude_client: Arc<ClaudeClient>,
}

impl FacebookHandler {
    /// Create a new Facebook handler
    pub fn new(
        page_id: &str,
        access_token: &str,
        verify_token: &str,
        claude_client: Arc<ClaudeClient>,
    ) -> Self {
        let api = FacebookApi::new(page_id, access_token, verify_token);
        let session_store = InMemorySessionStore::new();

        Self {
            api,
            session_store,
            claude_client,
        }
    }

    /// Handle incoming webhook entry
    pub async fn handle_webhook_entry(&self, entry: &WebhookEntry) -> Result<()> {
        if let Some(messages) = &entry.messaging {
            for messaging in messages {
                if let Err(e) = self.handle_messaging(messaging).await {
                    error!("Error handling messaging: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Handle a single messaging event
    async fn handle_messaging(&self, messaging: &WebhookMessaging) -> Result<()> {
        // Only handle text messages
        let message = match &messaging.message {
            Some(msg) => msg,
            None => {
                debug!("Ignoring non-message event");
                return Ok(());
            }
        };

        let sender_id = match &messaging.sender {
            Some(sender) => &sender.id,
            None => {
                warn!("No sender ID in message");
                return Ok(());
            }
        };

        // Check for quick replies
        if let Some(quick_reply) = &message.quick_reply {
            info!("Quick reply payload: {}", quick_reply.payload);
            // Handle quick reply if needed
        }

        // Get the message text
        let text = match &message.text {
            Some(text) => text.clone(),
            None => {
                debug!("Ignoring message without text");
                return Ok(());
            }
        };

        info!("Received message from {}: {}", sender_id, text);

        // Get or create session
        let mut session = self.session_store.get_or_create(sender_id).await;

        // Add user message to session
        session.messages.push(CoreMessage::user(&text));
        self.session_store
            .add_message(sender_id, CoreMessage::user(&text))
            .await;

        // Build request
        let mut request_builder = self
            .claude_client
            .request_builder()
            .system("You are a helpful assistant. Respond in the same language as the user's question. Be concise and helpful.")
            .max_tokens(2048);

        // Add conversation history (limit to last 20 messages)
        let history_start = session.messages.len().saturating_sub(20);
        for message in session.messages.iter().skip(history_start) {
            request_builder = request_builder.message(message.clone());
        }

        let request = request_builder.build();

        // Send typing indicator (optional)
        // Note: Facebook API doesn't have a simple typing indicator endpoint

        // Get response from Claude
        let response_text = match self.claude_client.messages(request).await {
            Ok(response) => {
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
                text
            }
            Err(e) => {
                error!("Claude API error: {}", e);
                let error_msg = "Sorry, I encountered an error processing your request.";
                self.api.send_message(sender_id, error_msg).await?;
                return Ok(());
            }
        };

        // Add assistant response to session
        self.session_store
            .add_message(sender_id, CoreMessage::assistant(&response_text))
            .await;

        // Send response to user
        self.api.send_message(sender_id, &response_text).await?;

        info!("Response sent to {}", sender_id);

        Ok(())
    }

    /// Process a webhook payload
    pub async fn process_webhook(&self, payload: &str) -> Result<()> {
        let entries = self.api.handle_webhook(payload)?;

        for entry in entries {
            self.handle_webhook_entry(&entry).await?;
        }

        Ok(())
    }

    /// Handle incoming message and get response
    pub async fn handle_message(&self, sender_id: &str, text: &str) -> Result<String> {
        // Get or create session
        let mut session = self.session_store.get_or_create(sender_id).await;

        // Add user message to session
        session.messages.push(CoreMessage::user(text));
        self.session_store
            .add_message(sender_id, CoreMessage::user(text))
            .await;

        // Build request
        let mut request_builder = self
            .claude_client
            .request_builder()
            .system("You are a helpful assistant. Respond in the same language as the user's question. Be concise and helpful.")
            .max_tokens(2048);

        // Add conversation history (limit to last 20 messages)
        let history_start = session.messages.len().saturating_sub(20);
        for message in session.messages.iter().skip(history_start) {
            request_builder = request_builder.message(message.clone());
        }

        let request = request_builder.build();

        // Get response from Claude
        let response_text = match self.claude_client.messages(request).await {
            Ok(response) => {
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
                text
            }
            Err(e) => {
                error!("Claude API error: {}", e);
                return Err(crate::error::FacebookError::Api(e.to_string()).into());
            }
        };

        // Add assistant response to session
        self.session_store
            .add_message(sender_id, CoreMessage::assistant(&response_text))
            .await;

        Ok(response_text)
    }

    /// Clear conversation history for a user
    pub async fn clear_conversation(&self, sender_id: &str) -> Result<()> {
        self.session_store.clear(sender_id).await;
        info!("Cleared conversation for {}", sender_id);
        Ok(())
    }

    /// Get user profile information
    pub async fn get_user_profile(&self, user_id: &str) -> Result<crate::api::UserProfile> {
        self.api.get_user_profile(user_id).await
    }

    /// Get session store for external access
    pub fn session_store(&self) -> &InMemorySessionStore {
        &self.session_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_creation() {
        // This would require a real ClaudeClient to test properly
        // For now, just verify the struct can be created
        // let handler = FacebookHandler::new("page_id", "token", client);
    }
}
