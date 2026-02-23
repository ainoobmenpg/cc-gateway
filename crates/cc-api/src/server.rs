//! HTTP API Server
//!
//! Starts and manages the axum-based HTTP server.

use crate::error::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use cc_core::{ClaudeClient, Config, SessionManager, ToolManager};

use crate::routes::routes;

/// 共有アプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub claude_client: Arc<ClaudeClient>,
    pub session_manager: Arc<SessionManager>,
    pub tool_manager: Arc<ToolManager>,
}

/// Start the HTTP API server
pub async fn start_server(
    port: u16,
    config: Config,
    claude_client: ClaudeClient,
    session_manager: SessionManager,
    tool_manager: Arc<ToolManager>,
) -> Result<()> {
    let state = AppState {
        config,
        claude_client: Arc::new(claude_client),
        session_manager: Arc::new(session_manager),
        tool_manager,
    };

    let app = Router::new()
        .merge(routes())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTP API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
