//! Rate limiting middleware
//!
//! Provides request rate limiting to prevent API abuse.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tokio::sync::RwLock;
use tracing::warn;

/// Rate limiter configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window for rate limiting
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,        // 60 requests
            window: Duration::from_secs(60), // per minute
        }
    }
}

/// Client rate limit state
#[derive(Clone)]
struct ClientState {
    request_count: u32,
    window_start: Instant,
}

/// In-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<String, ClientState>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create a rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a client is allowed to make a request
    pub async fn check(&self, client_id: &str) -> bool {
        let mut clients = self.clients.write().await;
        let now = Instant::now();

        let state = clients.entry(client_id.to_string()).or_insert(ClientState {
            request_count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(state.window_start) > self.config.window {
            state.request_count = 0;
            state.window_start = now;
        }

        // Check limit
        if state.request_count >= self.config.max_requests {
            warn!("Rate limit exceeded for client: {}", client_id);
            return false;
        }

        state.request_count += 1;
        true
    }

    /// Cleanup expired entries (should be called periodically)
    pub async fn cleanup(&self) {
        let mut clients = self.clients.write().await;
        let now = Instant::now();

        clients.retain(|_, state| {
            now.duration_since(state.window_start) <= self.config.window
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    limiter: Arc<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Use client IP or API key as identifier
    let client_id = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if !limiter.check(&client_id).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);

        // Should allow 3 requests
        assert!(limiter.check("client1").await);
        assert!(limiter.check("client1").await);
        assert!(limiter.check("client1").await);

        // 4th should be denied
        assert!(!limiter.check("client1").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_clients() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);

        // Each client has separate limit
        assert!(limiter.check("client1").await);
        assert!(limiter.check("client1").await);
        assert!(!limiter.check("client1").await);

        assert!(limiter.check("client2").await);
        assert!(limiter.check("client2").await);
    }
}
