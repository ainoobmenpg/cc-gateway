//! WebSocket connection handler
//!
//! Handles WebSocket connections and message routing.

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use cc_core::llm::{Message, MessageContent, MessagesRequest, ToolDefinition};

use crate::message::{ClientMessage, ImageData, ServerMessage, TokenUsage};
use crate::session::WsSession;
use crate::server::WsState;
use crate::Result;

/// Handle WebSocket upgrade request
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle established WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    let session_id = uuid::Uuid::new_v4().to_string();
    info!("New WebSocket connection: {}", session_id);

    // Split socket into sender and receiver
    let (ws_tx, mut ws_rx) = socket.split();
    let ws_tx = Arc::new(tokio::sync::Mutex::new(ws_tx));

    // Create channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Create WebSocket session
    let session = Arc::new(tokio::sync::Mutex::new(WsSession::new(
        session_id.clone(),
        tx.clone(),
        state.broadcast_tx.clone(),
        state.claude_client.clone(),
        state.session_manager.clone(),
        state.tool_manager.clone(),
    )));

    // Send initial session info
    let init_msg = ServerMessage::SessionInfo {
        session_id: session_id.clone(),
        message_count: 0,
    };
    if let Err(e) = tx.send(serde_json::to_string(&init_msg).unwrap()) {
        error!("Failed to send initial message: {}", e);
        return;
    }

    // Clone for tasks
    let session_id_send = session_id.clone();
    let session_id_recv = session_id.clone();
    let ws_tx_send = ws_tx.clone();
    let ws_tx_recv = ws_tx.clone();

    // Task to send messages to client
    let send_task = async move {
        while let Some(msg) = rx.recv().await {
            let mut tx = ws_tx_send.lock().await;
            if tx.send(WsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
        debug!("Send task ended for session: {}", session_id_send);
    };

    // Task to receive messages from client
    let session_clone = session.clone();
    let state_clone = state.clone();
    let recv_task = async move {
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    if let Err(e) = handle_client_message(&text, &session_clone, &state_clone).await
                    {
                        error!("Error handling message: {}", e);
                        let error_msg = ServerMessage::Error {
                            message: e.to_string(),
                        };
                        let _ = session_clone.lock().await.tx.send(
                            serde_json::to_string(&error_msg).unwrap(),
                        );
                    }
                }
                Ok(WsMessage::Ping(data)) => {
                    debug!("Received ping from session: {}", session_id_recv);
                    let mut tx = ws_tx_recv.lock().await;
                    let _ = tx.send(WsMessage::Pong(data)).await;
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Client closed connection: {}", session_id_recv);
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        debug!("Receive task ended for session: {}", session_id_recv);
    };

    // Run both tasks
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    info!("WebSocket connection closed: {}", session_id);
}

/// Handle incoming client message
async fn handle_client_message(
    text: &str,
    session: &Arc<tokio::sync::Mutex<WsSession>>,
    state: &Arc<WsState>,
) -> Result<()> {
    let msg: ClientMessage = serde_json::from_str(text)?;

    debug!("Received message: {:?}", msg);

    match msg {
        ClientMessage::Chat { message, image } => {
            handle_chat(session, state, message, image).await?;
        }
        ClientMessage::Clear => {
            handle_clear(session).await?;
        }
        ClientMessage::SessionInfo => {
            handle_session_info(session).await?;
        }
        ClientMessage::Ping => {
            let session = session.lock().await;
            let pong = ServerMessage::Pong;
            session.tx.send(serde_json::to_string(&pong).unwrap()).ok();
        }
    }

    Ok(())
}

/// Handle chat message
async fn handle_chat(
    session: &Arc<tokio::sync::Mutex<WsSession>>,
    state: &Arc<WsState>,
    text: String,
    image: Option<ImageData>,
) -> Result<()> {
    let (_session_id, system_prompt, tx) = {
        let s = session.lock().await;
        (s.session_id.clone(), s.system_prompt.clone(), s.tx.clone())
    };

    // Build message content
    let content = if let Some(img) = image {
        vec![
            MessageContent::Text { text },
            MessageContent::Image {
                source: cc_core::llm::ImageSource {
                    source_type: "base64".to_string(),
                    media_type: img.media_type,
                    data: img.data,
                },
            },
        ]
    } else {
        vec![MessageContent::Text { text: text.clone() }]
    };

    // Add user message to session
    {
        let s = session.lock().await;
        s.add_message(Message {
            role: "user".to_string(),
            content: content.clone(),
        })
        .await?;
    }

    // Get conversation history
    let messages = {
        let s = session.lock().await;
        s.get_messages().await?
    };

    // Get tool definitions
    let tools: Vec<ToolDefinition> = state
        .tool_manager
        .definitions()
        .into_iter()
        .map(|d| ToolDefinition::new(d.name, d.description, d.input_schema))
        .collect();

    // Build request
    let model = state.claude_client.model().to_string();
    let request = MessagesRequest {
        model,
        max_tokens: 4096,
        system: system_prompt.or_else(|| state.default_system_prompt.clone()),
        messages,
        tools: if tools.is_empty() { None } else { Some(tools) },
    };

    // Send to Claude API
    match state.claude_client.messages(request).await {
        Ok(response) => {
            // Extract text response
            let response_text = response
                .content
                .iter()
                .filter_map(|c| {
                    if let MessageContent::Text { text } = c {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            let tokens_used = response.usage.as_ref().map(|u| TokenUsage::from(u.clone()));

            // Add assistant message to session
            {
                let s = session.lock().await;
                s.add_message(Message::assistant(&response_text)).await?;
            }

            // Send response
            let server_msg = ServerMessage::ChatResponse {
                response: response_text,
                tokens_used,
            };
            tx.send(serde_json::to_string(&server_msg)?).ok();
        }
        Err(e) => {
            error!("Claude API error: {}", e);
            let error_msg = ServerMessage::Error {
                message: format!("Claude API error: {}", e),
            };
            tx.send(serde_json::to_string(&error_msg)?).ok();
        }
    }

    Ok(())
}

/// Handle clear session
async fn handle_clear(session: &Arc<tokio::sync::Mutex<WsSession>>) -> Result<()> {
    let (tx, session_id) = {
        let s = session.lock().await;
        s.clear_session().await?;
        (s.tx.clone(), s.session_id.clone())
    };

    info!("Session cleared: {}", session_id);

    let msg = ServerMessage::SessionCleared;
    tx.send(serde_json::to_string(&msg)?).ok();

    Ok(())
}

/// Handle session info request
async fn handle_session_info(session: &Arc<tokio::sync::Mutex<WsSession>>) -> Result<()> {
    let (tx, session_id) = {
        let s = session.lock().await;
        (s.tx.clone(), s.session_id.clone())
    };

    let messages = {
        let s = session.lock().await;
        s.get_messages().await?
    };

    let msg = ServerMessage::SessionInfo {
        session_id,
        message_count: messages.len(),
    };
    tx.send(serde_json::to_string(&msg)?).ok();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::ChatResponse {
            response: "Hello!".to_string(),
            tokens_used: Some(TokenUsage {
                input_tokens: 10,
                output_tokens: 5,
            }),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("chat_response"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"chat","message":"Hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Chat { message, image } => {
                assert_eq!(message, "Hello");
                assert!(image.is_none());
            }
            _ => panic!("Wrong message type"),
        }
    }
}
