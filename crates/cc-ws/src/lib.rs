//! cc-ws: WebSocket Gateway for Claude Code Gateway
//!
//! Provides real-time bidirectional communication via WebSocket.
//! Built with axum for HTTP handling and tokio-tungstenite for WebSocket.

pub mod error;
pub mod handler;
pub mod message;
pub mod server;
pub mod session;

pub use error::{Result, WsError};
pub use handler::websocket_handler;
pub use message::{ClientMessage, ServerMessage};
pub use server::{start_ws_server, WsState};
pub use session::WsSession;
