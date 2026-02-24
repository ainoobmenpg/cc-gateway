//! Dashboard API types and handlers
//!
//! Provides REST API endpoints for the web dashboard.

use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Dashboard state shared across handlers
pub struct DashboardState {
    /// Session data provider
    pub sessions: Arc<dyn SessionProvider + Send + Sync>,
    /// Usage data provider
    pub usage: Arc<dyn UsageProvider + Send + Sync>,
}

impl Clone for DashboardState {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            usage: self.usage.clone(),
        }
    }
}

impl DashboardState {
    /// Create a new dashboard state
    pub fn new(
        sessions: Arc<dyn SessionProvider + Send + Sync>,
        usage: Arc<dyn UsageProvider + Send + Sync>,
    ) -> Self {
        Self { sessions, usage }
    }
}

/// Session provider trait for dashboard data
#[async_trait]
pub trait SessionProvider: Send + Sync {
    /// Get all sessions
    async fn get_sessions(&self) -> Vec<SessionInfo>;

    /// Get a specific session by ID
    async fn get_session(&self, id: &str) -> Option<SessionInfo>;
}

/// Usage provider trait for cost/usage data
#[async_trait]
pub trait UsageProvider: Send + Sync {
    /// Get usage statistics
    async fn get_usage(&self) -> UsageStats;

    /// Get usage for a specific time range
    async fn get_usage_range(&self, start: i64, end: i64) -> UsageStats;
}

/// Session information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: String,
    /// Channel/Platform
    pub channel: String,
    /// Session title or summary
    pub title: Option<String>,
    /// Message count
    pub message_count: usize,
    /// Token usage
    pub tokens: TokenUsage,
    /// Created timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub updated_at: i64,
    /// Session status
    pub status: String,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input tokens
    pub input: u64,
    /// Output tokens
    pub output: u64,
    /// Thinking tokens (extended thinking)
    pub thinking: u64,
}

impl TokenUsage {
    /// Get total tokens
    pub fn total(&self) -> u64 {
        self.input + self.output + self.thinking
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    /// Total sessions
    pub total_sessions: usize,
    /// Total messages
    pub total_messages: usize,
    /// Token usage
    pub tokens: TokenUsage,
    /// Estimated cost in dollars
    pub estimated_cost: f64,
    /// Usage by channel
    pub by_channel: std::collections::HashMap<String, ChannelStats>,
    /// Usage by day
    pub daily: Vec<DailyStats>,
}

/// Statistics for a specific channel
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelStats {
    /// Channel name
    pub name: String,
    /// Number of sessions
    pub sessions: usize,
    /// Number of messages
    pub messages: usize,
    /// Token usage
    pub tokens: TokenUsage,
}

/// Daily usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    /// Date (YYYY-MM-DD)
    pub date: String,
    /// Number of sessions
    pub sessions: usize,
    /// Token usage
    pub tokens: TokenUsage,
    /// Estimated cost
    pub cost: f64,
}

/// Query parameters for session list
#[derive(Debug, Clone, Deserialize)]
pub struct SessionQuery {
    /// Filter by channel
    pub channel: Option<String>,
    /// Filter by status
    pub status: Option<String>,
    /// Limit results
    pub limit: Option<usize>,
}

/// Create the dashboard router
pub fn create_router(state: DashboardState) -> Router {
    Router::new()
        .route("/", get(dashboard_index))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/{id}", get(get_session))
        .route("/api/usage", get(get_usage))
        .route("/api/health", get(health_check))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
        .with_state(Arc::new(state))
}

/// Dashboard index page
async fn dashboard_index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

/// List sessions API endpoint
async fn list_sessions(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<SessionQuery>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.get_sessions().await;

    // Apply filters
    if let Some(channel) = query.channel {
        sessions.retain(|s| s.channel == channel);
    }
    if let Some(status) = query.status {
        sessions.retain(|s| s.status == status);
    }
    if let Some(limit) = query.limit {
        sessions.truncate(limit);
    }

    Json(sessions)
}

/// Get a specific session
async fn get_session(
    State(state): State<Arc<DashboardState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.sessions.get_session(&id).await {
        Some(session) => Json(session).into_response(),
        None => (StatusCode::NOT_FOUND, "Session not found").into_response(),
    }
}

/// Get usage statistics
async fn get_usage(State(state): State<Arc<DashboardState>>) -> impl IntoResponse {
    let stats = state.usage.get_usage().await;
    Json(stats)
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "cc-dashboard"
    }))
}

/// Index HTML template
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CC-Gateway Dashboard</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        header {
            background: #2c3e50;
            color: white;
            padding: 20px;
            margin-bottom: 20px;
        }
        header h1 { font-size: 24px; }
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }
        .stat-card {
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .stat-card h3 { color: #666; font-size: 14px; margin-bottom: 10px; }
        .stat-card .value { font-size: 32px; font-weight: bold; color: #2c3e50; }
        .sessions-table {
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #eee; }
        th { background: #f8f9fa; font-weight: 600; }
        .badge {
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: 600;
        }
        .badge-active { background: #d4edda; color: #155724; }
        .badge-inactive { background: #f8d7da; color: #721c24; }
        .refresh-btn {
            background: #3498db;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin-bottom: 20px;
        }
        .refresh-btn:hover { background: #2980b9; }
        .loading { opacity: 0.5; }
    </style>
</head>
<body>
    <header>
        <h1>CC-Gateway Dashboard</h1>
    </header>
    <div class="container">
        <button class="refresh-btn" onclick="loadData()">Refresh</button>

        <div class="stats-grid" id="stats">
            <div class="stat-card">
                <h3>Total Sessions</h3>
                <div class="value" id="total-sessions">-</div>
            </div>
            <div class="stat-card">
                <h3>Total Messages</h3>
                <div class="value" id="total-messages">-</div>
            </div>
            <div class="stat-card">
                <h3>Total Tokens</h3>
                <div class="value" id="total-tokens">-</div>
            </div>
            <div class="stat-card">
                <h3>Estimated Cost</h3>
                <div class="value" id="estimated-cost">-</div>
            </div>
        </div>

        <div class="sessions-table">
            <h2>Recent Sessions</h2>
            <table>
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>Channel</th>
                        <th>Messages</th>
                        <th>Tokens</th>
                        <th>Status</th>
                        <th>Updated</th>
                    </tr>
                </thead>
                <tbody id="sessions-body">
                </tbody>
            </table>
        </div>
    </div>
    <script>
        async function loadData() {
            try {
                const [usageRes, sessionsRes] = await Promise.all([
                    fetch('/api/usage'),
                    fetch('/api/sessions?limit=20')
                ]);

                if (usageRes.ok) {
                    const usage = await usageRes.json();
                    document.getElementById('total-sessions').textContent = usage.total_sessions;
                    document.getElementById('total-messages').textContent = usage.total_messages;
                    document.getElementById('total-tokens').textContent =
                        (usage.tokens.input + usage.tokens.output).toLocaleString();
                    document.getElementById('estimated-cost').textContent =
                        '$' + usage.estimated_cost.toFixed(4);
                }

                if (sessionsRes.ok) {
                    const sessions = await sessionsRes.json();
                    const tbody = document.getElementById('sessions-body');
                    tbody.innerHTML = sessions.map(s => `
                        <tr>
                            <td>${s.id.substring(0, 8)}...</td>
                            <td>${s.channel}</td>
                            <td>${s.message_count}</td>
                            <td>${(s.tokens.input + s.tokens.output).toLocaleString()}</td>
                            <td><span class="badge badge-${s.status}">${s.status}</span></td>
                            <td>${new Date(s.updated_at * 1000).toLocaleString()}</td>
                        </tr>
                    `).join('');
                }
            } catch (e) {
                console.error('Failed to load data:', e);
            }
        }

        loadData();
        setInterval(loadData, 30000);
    </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSessionProvider;
    struct MockUsageProvider;

    #[async_trait]
    impl SessionProvider for MockSessionProvider {
        async fn get_sessions(&self) -> Vec<SessionInfo> {
            vec![SessionInfo {
                id: "test-1".to_string(),
                channel: "discord".to_string(),
                title: Some("Test Session".to_string()),
                message_count: 10,
                tokens: TokenUsage { input: 100, output: 50, thinking: 0 },
                created_at: 0,
                updated_at: 0,
                status: "active".to_string(),
            }]
        }

        async fn get_session(&self, id: &str) -> Option<SessionInfo> {
            if id == "test-1" {
                self.get_sessions().await.into_iter().next()
            } else {
                None
            }
        }
    }

    #[async_trait]
    impl UsageProvider for MockUsageProvider {
        async fn get_usage(&self) -> UsageStats {
            UsageStats {
                total_sessions: 1,
                total_messages: 10,
                tokens: TokenUsage { input: 100, output: 50, thinking: 0 },
                estimated_cost: 0.001,
                by_channel: std::collections::HashMap::new(),
                daily: vec![],
            }
        }

        async fn get_usage_range(&self, _start: i64, _end: i64) -> UsageStats {
            self.get_usage().await
        }
    }

    #[test]
    fn test_token_usage_total() {
        let usage = TokenUsage {
            input: 100,
            output: 50,
            thinking: 25,
        };
        assert_eq!(usage.total(), 175);
    }

    #[test]
    fn test_create_router() {
        let state = DashboardState::new(
            Arc::new(MockSessionProvider),
            Arc::new(MockUsageProvider),
        );
        let _router = create_router(state);
    }
}
