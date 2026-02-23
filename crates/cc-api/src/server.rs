//! HTTP API Server
//!
//! Starts and manages the axum-based HTTP server.

use crate::error::Result;
use axum::{
    middleware,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use cc_core::{ClaudeClient, Config, SessionManager, ToolManager};

use crate::middleware::auth::auth_middleware;
use crate::routes::{protected_routes, public_routes};

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
        config: config.clone(),
        claude_client: Arc::new(claude_client),
        session_manager: Arc::new(session_manager),
        tool_manager,
    };

    // Check if API key is configured
    let api_key_configured = std::env::var("API_KEY").is_ok() || config.api_key.is_some();

    if api_key_configured {
        info!("API authentication enabled");
    } else {
        info!("API authentication disabled (no API_KEY configured)");
    }

    // Build CORS layer with restricted origins
    let cors_layer = build_cors_layer(&config);

    // Build the app router
    let app = if api_key_configured {
        // With authentication
        Router::new()
            .merge(public_routes())
            .merge(
                protected_routes()
                    .layer(middleware::from_fn(auth_middleware))
            )
            .layer(cors_layer)
            .with_state(state)
    } else {
        // Without authentication (development mode)
        Router::new()
            .merge(public_routes())
            .merge(protected_routes())
            .layer(cors_layer)
            .with_state(state)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTP API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Build CORS layer with configured allowed origins
fn build_cors_layer(config: &Config) -> CorsLayer {
    // Get allowed origins from config or environment
    let allowed_origins = config.api.allowed_origins.as_ref();

    match allowed_origins {
        Some(origins) if !origins.is_empty() => {
            // Parse origins and build CORS layer
            let origins: Vec<http::HeaderValue> = origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();

            if origins.is_empty() {
                info!("No valid CORS origins configured, using permissive mode");
                CorsLayer::permissive()
            } else {
                info!("CORS origins: {:?}", origins);
                CorsLayer::new()
                    .allow_origin(origins)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
        }
        _ => {
            // Default: allow localhost only (development-friendly)
            info!("CORS: allowing localhost origins");
            CorsLayer::new()
                .allow_origin([
                    "http://localhost".parse().unwrap(),
                    "http://127.0.0.1".parse().unwrap(),
                ])
                .allow_methods(Any)
                .allow_headers(Any)
        }
    }
}
