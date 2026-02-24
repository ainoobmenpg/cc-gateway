//! Dashboard server configuration and startup

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tracing::info;

use crate::api::{create_router, DashboardState, SessionProvider, UsageProvider};
use crate::error::{DashboardError, Result};

/// Dashboard server configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

impl DashboardConfig {
    /// Create a new configuration
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Get the socket address
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.host, self.port);
        addr.parse()
            .map_err(|e| DashboardError::ConfigError(format!("Invalid address: {}", e)))
    }
}

/// Dashboard server
pub struct DashboardServer {
    config: DashboardConfig,
    state: DashboardState,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(
        config: DashboardConfig,
        sessions: Arc<dyn SessionProvider + Send + Sync>,
        usage: Arc<dyn UsageProvider + Send + Sync>,
    ) -> Self {
        Self {
            config,
            state: DashboardState::new(sessions, usage),
        }
    }

    /// Get the router
    pub fn router(&self) -> Router {
        create_router(self.state.clone())
    }

    /// Start the server
    pub async fn run(self) -> Result<()> {
        let addr = self.config.socket_addr()?;
        let app = self.router();

        info!("Dashboard server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| DashboardError::ServerError(format!("Failed to bind: {}", e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| DashboardError::ServerError(format!("Server error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_dashboard_config_socket_addr() {
        let config = DashboardConfig::new("0.0.0.0", 8080);
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.port(), 8080);
    }
}
