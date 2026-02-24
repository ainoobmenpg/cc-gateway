//! iMessage watcher for polling new messages
//!
//! Periodically checks for new messages via Apple Script

use std::sync::Arc;
use std::time::Duration;

use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::error::Result;
use crate::handler::MessageHandler;
use crate::script::{AppleScript, ChatInfo};

/// Configuration for the message watcher
#[derive(Clone, Debug)]
pub struct WatcherConfig {
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
    /// Chats to watch (empty = watch all)
    pub watch_chats: Vec<String>,
    /// Maximum messages to process per poll
    pub max_messages_per_poll: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
            watch_chats: Vec::new(),
            max_messages_per_poll: 10,
        }
    }
}

/// Message watcher that polls for new iMessages
pub struct MessageWatcher {
    config: WatcherConfig,
    handler: Arc<MessageHandler>,
    running: Arc<std::sync::atomic::AtomicBool>,
    last_seen_ids: Arc<dashmap::DashMap<String, String>>,
}

impl MessageWatcher {
    /// Create a new message watcher
    pub fn new(config: WatcherConfig, handler: Arc<MessageHandler>) -> Self {
        Self {
            config,
            handler,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_seen_ids: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Start watching for new messages
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            warn!("Watcher is already running");
            return Ok(());
        }

        info!(
            "Starting iMessage watcher (poll interval: {}s)",
            self.config.poll_interval_secs
        );

        // Check Messages.app is available
        if !AppleScript::is_messages_running()? {
            info!("Activating Messages.app...");
            AppleScript::activate_messages()?;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // Initialize with current messages
        self.initialize_seen_messages().await?;

        let mut poll_interval = interval(Duration::from_secs(self.config.poll_interval_secs));

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            poll_interval.tick().await;

            if let Err(e) = self.poll_messages().await {
                error!("Error polling messages: {:?}", e);
            }
        }

        Ok(())
    }

    /// Stop the watcher
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        info!("iMessage watcher stopped");
    }

    /// Initialize with current messages to avoid processing old ones
    async fn initialize_seen_messages(&self) -> Result<()> {
        let chats = self.get_watched_chats()?;

        for chat in chats {
            if let Ok(messages) = AppleScript::get_recent_messages(&chat.name, 5) {
                for msg in messages {
                    let msg_id = format!("{}:{}", msg.sender, msg.timestamp);
                    self.last_seen_ids.insert(chat.name.clone(), msg_id);
                }
            }
        }

        info!("Initialized message tracking for {} chats", self.last_seen_ids.len());
        Ok(())
    }

    /// Get list of chats to watch
    fn get_watched_chats(&self) -> Result<Vec<ChatInfo>> {
        let all_chats = AppleScript::get_chats()?;

        if self.config.watch_chats.is_empty() {
            return Ok(all_chats);
        }

        let filtered: Vec<ChatInfo> = all_chats
            .into_iter()
            .filter(|chat| {
                self.config
                    .watch_chats
                    .iter()
                    .any(|watch| chat.name.contains(watch) || watch.contains(&chat.name))
            })
            .collect();

        Ok(filtered)
    }

    /// Poll for new messages
    async fn poll_messages(&self) -> Result<()> {
        let chats = self.get_watched_chats()?;
        let mut processed = 0;

        for chat in chats {
            if processed >= self.config.max_messages_per_poll {
                break;
            }

            match AppleScript::get_recent_messages(&chat.name, 5) {
                Ok(messages) => {
                    for msg in messages {
                        let msg_id = format!("{}:{}", msg.sender, msg.timestamp);

                        // Check if this is a new message
                        let is_new = self
                            .last_seen_ids
                            .get(&chat.name)
                            .map(|seen| seen.value() != &msg_id)
                            .unwrap_or(true);

                        if is_new {
                            debug!("New message from {} in chat {}", msg.sender, chat.name);

                            // Update seen ID
                            self.last_seen_ids.insert(chat.name.clone(), msg_id);

                            // Process the message
                            if let Err(e) = self.handler.process_message(&msg).await {
                                error!("Error processing message: {:?}", e);
                            }

                            processed += 1;
                            if processed >= self.config.max_messages_per_poll {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get messages for chat {}: {:?}", chat.name, e);
                }
            }
        }

        if processed > 0 {
            debug!("Processed {} new messages", processed);
        }

        Ok(())
    }
}

/// Builder for MessageWatcher
pub struct WatcherBuilder {
    config: WatcherConfig,
    handler: Option<Arc<MessageHandler>>,
}

impl WatcherBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: WatcherConfig::default(),
            handler: None,
        }
    }

    /// Set polling interval
    pub fn poll_interval(mut self, secs: u64) -> Self {
        self.config.poll_interval_secs = secs;
        self
    }

    /// Set chats to watch
    pub fn watch_chats(mut self, chats: Vec<String>) -> Self {
        self.config.watch_chats = chats;
        self
    }

    /// Set message handler
    pub fn handler(mut self, handler: Arc<MessageHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Build the watcher
    pub fn build(self) -> Result<MessageWatcher> {
        let handler = self.handler.ok_or_else(|| {
            crate::error::IMessageError::Config("Handler is required".to_string())
        })?;

        Ok(MessageWatcher::new(self.config, handler))
    }
}

impl Default for WatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}
