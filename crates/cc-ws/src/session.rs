//! WebSocket session management
//!
//! Manages individual WebSocket connections and their state.

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info};

use cc_core::{ClaudeClient, SessionManager, ToolManager};

/// WebSocket session state
pub struct WsSession {
    /// Unique session ID (maps to channel_id in SessionManager)
    pub session_id: String,
    /// Channel to send messages to this WebSocket client
    pub tx: mpsc::UnboundedSender<String>,
    /// Broadcast channel for server-wide events
    pub broadcast_tx: broadcast::Sender<String>,
    /// Reference to Claude client
    pub claude_client: Arc<ClaudeClient>,
    /// Reference to session manager
    pub session_manager: Arc<SessionManager>,
    /// Reference to tool manager
    pub tool_manager: Arc<ToolManager>,
    /// System prompt for this session
    pub system_prompt: Option<String>,
}

impl WsSession {
    /// Create a new WebSocket session
    pub fn new(
        session_id: String,
        tx: mpsc::UnboundedSender<String>,
        broadcast_tx: broadcast::Sender<String>,
        claude_client: Arc<ClaudeClient>,
        session_manager: Arc<SessionManager>,
        tool_manager: Arc<ToolManager>,
    ) -> Self {
        Self {
            session_id,
            tx,
            broadcast_tx,
            claude_client,
            session_manager,
            tool_manager,
            system_prompt: None,
        }
    }

    /// Send a message to this client
    pub fn send(&self, message: &str) {
        if let Err(e) = self.tx.send(message.to_string()) {
            debug!("Failed to send message to client: {}", e);
        }
    }

    /// Set system prompt for this session
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    /// Get or create the underlying session
    pub async fn get_or_create_session(&self) -> cc_core::Result<cc_core::Session> {
        self.session_manager.get_or_create(&self.session_id).await
    }

    /// Add a message to session history
    pub async fn add_message(&self, message: cc_core::Message) -> cc_core::Result<()> {
        self.session_manager.add_message(&self.session_id, message).await
    }

    /// Get session messages
    pub async fn get_messages(&self) -> cc_core::Result<Vec<cc_core::Message>> {
        self.session_manager.get_messages(&self.session_id).await
    }

    /// Clear session history
    pub async fn clear_session(&self) -> cc_core::Result<()> {
        self.session_manager.clear_messages(&self.session_id).await
    }

    /// Log session event
    pub fn log_event(&self, event: &str) {
        info!("[Session {}] {}", self.session_id, event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use cc_core::{LlmConfig, LlmProvider, ApiConfig, MemoryConfig, McpConfig, SchedulerConfig};

    fn create_test_config() -> cc_core::Config {
        cc_core::Config {
            llm: LlmConfig {
                api_key: "test_key".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                provider: LlmProvider::Claude,
                base_url: None,
            },
            claude_api_key: "test_key".to_string(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            discord_token: None,
            admin_user_ids: vec![],
            api: ApiConfig::default(),
            api_key: None,
            memory: MemoryConfig::default(),
            mcp: McpConfig::default(),
            scheduler: SchedulerConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_session_send() {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let (broadcast_tx, _) = broadcast::channel(16);

        // Create minimal mock dependencies
        let config = create_test_config();
        let claude_client = Arc::new(ClaudeClient::new(&config).unwrap());
        let session_manager = Arc::new(SessionManager::in_memory().unwrap());
        let tool_manager = Arc::new(ToolManager::new());

        let session = WsSession::new(
            "test-session".to_string(),
            tx,
            broadcast_tx,
            claude_client,
            session_manager,
            tool_manager,
        );

        session.send("test message");

        let received = rx.recv().await.unwrap();
        assert_eq!(received, "test message");
    }
}
