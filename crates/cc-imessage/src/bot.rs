//! iMessage Bot implementation
//!
//! Main entry point for the iMessage Gateway

use std::sync::Arc;

use tracing::{error, info};

use cc_core::{ClaudeClient, Config};

use crate::error::{IMessageError, Result};
use crate::handler::{HandlerConfig, MessageHandler};
use crate::script::AppleScript;
use crate::session::InMemorySessionStore;
use crate::watcher::{WatcherBuilder, WatcherConfig};

/// iMessage Bot for Claude Code Gateway
pub struct IMessageBot {
    #[allow(dead_code)]
    config: Config,
    claude_client: Arc<ClaudeClient>,
    session_store: Arc<InMemorySessionStore>,
    handler_config: HandlerConfig,
    watcher_config: WatcherConfig,
}

impl IMessageBot {
    /// Create a new iMessage bot
    pub fn new(config: Config, claude_client: ClaudeClient) -> Result<Self> {
        // Check if we're running on macOS
        if !cfg!(target_os = "macos") {
            return Err(IMessageError::NotAvailable);
        }

        // Check if Messages.app is available
        if AppleScript::is_messages_running().is_err() {
            return Err(IMessageError::NotAvailable);
        }

        let session_store = Arc::new(InMemorySessionStore::new());

        // Start session cleanup task
        let store_clone = session_store.clone();
        tokio::spawn(async move {
            if let Err(e) = store_clone.start_cleanup_task().await {
                error!("Session cleanup task failed: {}", e);
            }
        });

        Ok(Self {
            config,
            claude_client: Arc::new(claude_client),
            session_store,
            handler_config: HandlerConfig::default(),
            watcher_config: WatcherConfig::default(),
        })
    }

    /// Create with shared Claude client
    pub fn with_client(config: Config, claude_client: Arc<ClaudeClient>) -> Result<Self> {
        // Check if we're running on macOS
        if !cfg!(target_os = "macos") {
            return Err(IMessageError::NotAvailable);
        }

        let session_store = Arc::new(InMemorySessionStore::new());

        // Start session cleanup task
        let store_clone = session_store.clone();
        tokio::spawn(async move {
            if let Err(e) = store_clone.start_cleanup_task().await {
                error!("Session cleanup task failed: {}", e);
            }
        });

        Ok(Self {
            config,
            claude_client,
            session_store,
            handler_config: HandlerConfig::default(),
            watcher_config: WatcherConfig::default(),
        })
    }

    /// Get the session store
    pub fn session_store(&self) -> Arc<InMemorySessionStore> {
        self.session_store.clone()
    }

    /// Set handler configuration
    pub fn with_handler_config(mut self, config: HandlerConfig) -> Self {
        self.handler_config = config;
        self
    }

    /// Set watcher configuration
    pub fn with_watcher_config(mut self, config: WatcherConfig) -> Self {
        self.watcher_config = config;
        self
    }

    /// Set allowed senders
    pub fn allowed_senders(mut self, senders: Vec<String>) -> Self {
        self.handler_config.allowed_senders = senders;
        self
    }

    /// Set system prompt
    pub fn system_prompt(mut self, prompt: String) -> Self {
        self.handler_config.system_prompt = prompt;
        self
    }

    /// Set poll interval for message watching
    pub fn poll_interval(mut self, secs: u64) -> Self {
        self.watcher_config.poll_interval_secs = secs;
        self
    }

    /// Set chats to watch
    pub fn watch_chats(mut self, chats: Vec<String>) -> Self {
        self.watcher_config.watch_chats = chats;
        self
    }

    /// Send a message to a recipient (one-way, no session)
    pub async fn send_message(&self, recipient: &str, message: &str) -> Result<()> {
        AppleScript::send_message(recipient, message)
    }

    /// Send a message to a chat by name
    pub async fn send_to_chat(&self, chat_name: &str, message: &str) -> Result<()> {
        AppleScript::send_to_chat(chat_name, message)
    }

    /// Get unread message count
    pub fn get_unread_count(&self) -> Result<i32> {
        AppleScript::get_unread_count()
    }

    /// Get list of chats
    pub fn get_chats(&self) -> Result<Vec<crate::script::ChatInfo>> {
        AppleScript::get_chats()
    }

    /// Start the bot (blocking)
    pub async fn start(&self) -> Result<()> {
        info!("Starting iMessage bot...");

        // Create message handler
        let handler = Arc::new(MessageHandler::new(
            self.claude_client.clone(),
            self.session_store.clone(),
            self.handler_config.clone(),
        ));

        // Build and start watcher
        let watcher = WatcherBuilder::new()
            .poll_interval(self.watcher_config.poll_interval_secs)
            .watch_chats(self.watcher_config.watch_chats.clone())
            .handler(handler)
            .build()?;

        info!("iMessage bot started. Watching for messages...");

        // This will block until stopped
        watcher.start().await
    }

    /// Run the bot with shutdown signal
    pub async fn run(&self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        let handler = Arc::new(MessageHandler::new(
            self.claude_client.clone(),
            self.session_store.clone(),
            self.handler_config.clone(),
        ));

        let watcher = WatcherBuilder::new()
            .poll_interval(self.watcher_config.poll_interval_secs)
            .watch_chats(self.watcher_config.watch_chats.clone())
            .handler(handler)
            .build()?;

        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let running_clone = running.clone();

        // Spawn watcher task
        let watcher_task = tokio::spawn(async move {
            if running_clone.load(std::sync::atomic::Ordering::SeqCst) {
                if let Err(e) = watcher.start().await {
                    error!("Watcher error: {:?}", e);
                }
            }
        });

        // Wait for shutdown signal
        let _ = shutdown.recv().await;
        running.store(false, std::sync::atomic::Ordering::SeqCst);

        // Abort watcher
        watcher_task.abort();

        info!("iMessage bot stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::{Config, LlmConfig, LlmProvider, ApiConfig, MemoryConfig, McpConfig, SchedulerConfig};

    /// Helper function to create a mock config for testing
    fn mock_config() -> Config {
        Config {
            llm: LlmConfig {
                api_key: "test-key".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                provider: LlmProvider::Claude,
                base_url: None,
            },
            claude_api_key: "test-key".to_string(),
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

    // Note: These tests only run on macOS
    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_bot_creation() {
        let config = mock_config();
        let client = ClaudeClient::new(&config).unwrap();

        // This test may fail if Messages.app is not running
        // Just check that it doesn't panic
        let _ = IMessageBot::new(config, client);
    }

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_bot_with_client() {
        let config = mock_config();
        let client = Arc::new(ClaudeClient::new(&config).unwrap());

        // This test may fail if Messages.app is not running
        let _ = IMessageBot::with_client(config, client);
    }
}
