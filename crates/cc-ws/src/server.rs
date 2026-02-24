//! WebSocket server implementation
//!
//! Starts and manages the axum-based WebSocket server.

use axum::{
    routing::{get, get_service},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;

use cc_core::{ClaudeClient, Config, SessionManager, ToolManager};

use crate::handler::websocket_handler;
use crate::Result;

/// Shared WebSocket server state
#[derive(Clone)]
pub struct WsState {
    /// Claude API client
    pub claude_client: Arc<ClaudeClient>,
    /// Session manager
    pub session_manager: Arc<SessionManager>,
    /// Tool manager
    pub tool_manager: Arc<ToolManager>,
    /// Broadcast channel for server-wide events
    pub broadcast_tx: broadcast::Sender<String>,
    /// Default system prompt
    pub default_system_prompt: Option<String>,
    /// Server configuration
    pub config: Config,
}

/// Start the WebSocket server
pub async fn start_ws_server(
    port: u16,
    config: Config,
    claude_client: ClaudeClient,
    session_manager: SessionManager,
    tool_manager: Arc<ToolManager>,
    static_dir: Option<&str>,
) -> Result<()> {
    // Create broadcast channel
    let (broadcast_tx, _) = broadcast::channel(256);

    // Create shared state
    let state = Arc::new(WsState {
        claude_client: Arc::new(claude_client),
        session_manager: Arc::new(session_manager),
        tool_manager,
        broadcast_tx,
        default_system_prompt: None, // Can be set via environment or config
        config: config.clone(),
    });

    // Build CORS layer
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let mut router = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/health", get(|| async { "OK" }));

    // Add static file serving if directory provided
    if let Some(dir) = static_dir {
        info!("Serving static files from: {}", dir);
        let serve_dir = get_service(ServeDir::new(dir)).handle_error(|e| async move {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", e),
            )
        });
        router = router.fallback_service(serve_dir);
    }

    let app = router
        .layer(cors_layer)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("WebSocket server listening on {}", addr);
    info!("WebSocket endpoint: ws://localhost:{}/ws", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Builder for WebSocket server configuration
pub struct WsServerBuilder {
    port: u16,
    config: Config,
    static_dir: Option<String>,
}

impl WsServerBuilder {
    /// Create a new builder
    pub fn new(config: Config) -> Self {
        Self {
            port: 3001,
            config,
            static_dir: None,
        }
    }

    /// Set the port
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the static files directory
    pub fn static_dir(mut self, dir: impl Into<String>) -> Self {
        self.static_dir = Some(dir.into());
        self
    }

    /// Build and start the server
    pub async fn start(
        self,
        claude_client: ClaudeClient,
        session_manager: SessionManager,
        tool_manager: Arc<ToolManager>,
    ) -> Result<()> {
        start_ws_server(
            self.port,
            self.config,
            claude_client,
            session_manager,
            tool_manager,
            self.static_dir.as_deref(),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::{LlmConfig, LlmProvider, ApiConfig, MemoryConfig, McpConfig, SchedulerConfig};

    fn create_test_config() -> Config {
        Config {
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

    #[test]
    fn test_server_builder() {
        let config = create_test_config();
        let builder = WsServerBuilder::new(config)
            .port(8080)
            .static_dir("./static");

        assert_eq!(builder.port, 8080);
        assert_eq!(builder.static_dir, Some("./static".to_string()));
    }
}
