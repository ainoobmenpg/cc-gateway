//! cc-dashboard: Web Dashboard for cc-gateway
//!
//! This crate provides a web-based dashboard for monitoring
//! sessions, usage, and costs.
//!
//! ## Features
//!
//! - Real-time session monitoring
//! - Token usage tracking
//! - Cost estimation
//! - Channel-based statistics
//! - RESTful API
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cc_dashboard::{DashboardServer, DashboardConfig, SessionProvider, UsageProvider};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = DashboardConfig::default();
//!     let sessions = Arc::new(MySessionProvider);
//!     let usage = Arc::new(MyUsageProvider);
//!
//!     let server = DashboardServer::new(config, sessions, usage);
//!     server.run().await.unwrap();
//! }
//! ```

pub mod api;
pub mod error;
pub mod server;

pub use api::{ChannelStats, DashboardState, DailyStats, SessionInfo, SessionProvider, TokenUsage, UsageProvider, UsageStats};
pub use error::{DashboardError, Result};
pub use server::{DashboardConfig, DashboardServer};
