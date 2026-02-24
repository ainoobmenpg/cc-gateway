//! Webhook server for receiving WhatsApp messages from Twilio

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use tracing::{error, info};

use crate::error::{Result, WhatsAppError};
use crate::session::InMemorySessionStore;
use crate::twilio::{IncomingMessage, TwilioClient};

/// Webhook server state
#[derive(Clone)]
pub struct WebhookState {
    pub twilio_client: Arc<TwilioClient>,
    pub session_store: Arc<InMemorySessionStore>,
    pub claude_client: Arc<cc_core::ClaudeClient>,
    pub admin_numbers: Vec<String>,
}

/// Webhook server
pub struct WebhookServer {
    addr: SocketAddr,
    state: WebhookState,
}

impl WebhookServer {
    /// Create a new webhook server
    pub fn new(
        addr: SocketAddr,
        twilio_client: Arc<TwilioClient>,
        claude_client: Arc<cc_core::ClaudeClient>,
        admin_numbers: Vec<String>,
    ) -> Self {
        let session_store = Arc::new(InMemorySessionStore::new());

        let state = WebhookState {
            twilio_client,
            session_store,
            claude_client,
            admin_numbers,
        };

        Self { addr, state }
    }

    /// Start the webhook server
    pub async fn start(self) -> Result<()> {
        info!("Starting WhatsApp webhook server on {}", self.addr);

        let app = Router::new()
            .route("/webhook/whatsapp", post(handle_webhook))
            .with_state(Arc::new(self.state));

        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .map_err(|e| WhatsAppError::Config(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| WhatsAppError::Http(e.to_string()))?;

        Ok(())
    }
}

/// Handle incoming WhatsApp webhook
async fn handle_webhook(
    State(state): State<Arc<WebhookState>>,
    Form(msg): Form<IncomingMessage>,
) -> impl IntoResponse {
    info!("Received WhatsApp message from {}: {}", msg.from, msg.body);

    // Check admin permission
    if !state.admin_numbers.is_empty() && !state.admin_numbers.contains(&msg.from) {
        return (StatusCode::FORBIDDEN, "Unauthorized");
    }

    let body = msg.body.trim();
    if body.is_empty() {
        return (StatusCode::OK, "");
    }

    // Handle commands
    if body.starts_with('/') {
        match handle_command(&state, &msg.from, body).await {
            Ok(response) => {
                if let Err(e) = state.twilio_client.send_message(&msg.from, &response).await {
                    error!("Failed to send response: {}", e);
                }
            }
            Err(e) => {
                error!("Error handling command: {}", e);
            }
        }
        return (StatusCode::OK, "");
    }

    // Regular message - process with Claude
    match process_with_claude(&state, &msg.from, body).await {
        Ok(response) => {
            if let Err(e) = state.twilio_client.send_message(&msg.from, &response).await {
                error!("Failed to send response: {}", e);
            }
        }
        Err(e) => {
            error!("Error processing message: {}", e);
            let _ = state
                .twilio_client
                .send_message(&msg.from, &format!("Error: {}", e))
                .await;
        }
    }

    (StatusCode::OK, "")
}

/// Handle slash commands
async fn handle_command(state: &WebhookState, from: &str, body: &str) -> Result<String> {
    let parts: Vec<&str> = body.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();

    match command.as_str() {
        "/clear" => {
            state.session_store.clear(from).await;
            Ok("✅ Conversation history cleared.".to_string())
        }
        "/help" => {
            Ok(r#"🤖 cc-gateway WhatsApp Bot

Usage:
- Send any message to chat with Claude
- /clear - Clear conversation history
- /help - Show this help

powered by cc-gateway"#
                .to_string())
        }
        _ => Ok(format!("Unknown command: {}. Use /help for available commands.", command)),
    }
}

/// Process message with Claude
async fn process_with_claude(state: &WebhookState, from: &str, body: &str) -> Result<String> {
    let session = state.session_store.get_or_create(from).await;

    // Build message history
    let mut messages = session.messages.clone();
    messages.push(cc_core::Message::user(body));

    // Build request
    let mut request_builder = state
        .claude_client
        .request_builder()
        .system("You are a helpful assistant. Respond in the same language as the user's question. Be concise and helpful.")
        .max_tokens(2048);

    // Add conversation history (limit to last 20 messages)
    let history_start = messages.len().saturating_sub(20);
    for message in messages.into_iter().skip(history_start) {
        request_builder = request_builder.message(message);
    }

    let request = request_builder.build();

    // Call Claude API
    let response = state.claude_client.messages(request).await.map_err(|e| {
        WhatsAppError::Api(format!("Claude API error: {}", e))
    })?;

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
    state
        .session_store
        .add_message(from, cc_core::Message::user(body))
        .await;
    state
        .session_store
        .add_message(from, cc_core::Message::assistant(&text))
        .await;

    // Truncate for WhatsApp (character limit)
    if text.len() > 4000 {
        Ok(format!("{}...(truncated)", &text[..4000]))
    } else {
        Ok(text)
    }
}
