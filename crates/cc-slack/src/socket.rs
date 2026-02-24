//! Socket Mode implementation for Slack
//!
//! Connects to Slack via WebSocket for real-time events

use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info};

use crate::api::SlackApiClient;
use crate::error::{Result, SlackError};
use crate::types::SlackEvent;

/// Socket Mode client
pub struct SocketModeClient {
    app_token: String,
    #[allow(dead_code)]
    api_client: SlackApiClient,
}

impl SocketModeClient {
    /// Create a new Socket Mode client
    pub fn new(app_token: &str, api_client: SlackApiClient) -> Self {
        Self {
            app_token: app_token.to_string(),
            api_client,
        }
    }

    /// Connect and get WebSocket URL
    async fn get_websocket_url(&self) -> Result<String> {
        let client = reqwest::Client::new();

        let response = client
            .post("https://slack.com/api/apps.connections.open")
            .bearer_auth(&self.app_token)
            .send()
            .await
            .map_err(SlackError::HttpError)?;

        #[derive(Debug, serde::Deserialize)]
        struct ConnectionsOpenResponse {
            ok: bool,
            url: Option<String>,
            error: Option<String>,
        }

        let result: ConnectionsOpenResponse = response
            .json()
            .await
            .map_err(|e| SlackError::ParseError(e.to_string()))?;

        if !result.ok {
            return Err(SlackError::ApiError(result.error.unwrap_or("Unknown error".to_string())));
        }

        result.url.ok_or_else(|| SlackError::ApiError("No URL returned".to_string()))
    }

    /// Start the Socket Mode connection
    pub async fn start<F>(&self, mut event_handler: F) -> Result<()>
    where
        F: FnMut(SlackEvent) -> Result<()> + Send,
    {
        let ws_url = self.get_websocket_url().await?;
        info!("Connecting to Slack Socket Mode: {}", ws_url.split('?').next().unwrap_or(&ws_url));

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| SlackError::WebSocketError(e.to_string()))?;

        info!("Connected to Slack Socket Mode");

        let (mut write, mut read) = ws_stream.split();

        while let Some(message) = read.next().await {
            match message {
                Ok(WsMessage::Text(text)) => {
                    // Parse the envelope
                    #[derive(Debug, serde::Deserialize)]
                    struct SocketEnvelope {
                        #[serde(rename = "type")]
                        envelope_type: Option<String>,
                        payload: Option<serde_json::Value>,
                        #[serde(default)]
                        ack: Option<String>,
                    }

                    let envelope: SocketEnvelope = serde_json::from_str(&text)
                        .map_err(|e| SlackError::ParseError(e.to_string()))?;

                    debug!("Received envelope type: {:?}", envelope.envelope_type);

                    // Send ACK if required
                    if let Some(ref ack_id) = envelope.ack {
                        let ack_msg = serde_json::json!({
                            "envelope_id": ack_id
                        });
                        let ack_str = serde_json::to_string(&ack_msg).unwrap();
                        write.send(WsMessage::Text(ack_str.into())).await
                            .map_err(|e| SlackError::WebSocketError(e.to_string()))?;
                    }

                    // Handle different message types
                    match envelope.envelope_type.as_deref() {
                        Some("hello") => {
                            info!("Received hello from Slack Socket Mode");
                        }
                        Some("events_api") => {
                            if let Some(payload) = envelope.payload {
                                if let Some(event) = payload.get("event") {
                                    let slack_event: SlackEvent = serde_json::from_value(event.clone())
                                        .map_err(|e| SlackError::ParseError(e.to_string()))?;

                                    if let Err(e) = event_handler(slack_event) {
                                        error!("Error handling event: {:?}", e);
                                    }
                                }
                            }
                        }
                        Some("interactive") => {
                            // Handle interactive components (buttons, etc.)
                            debug!("Received interactive event");
                        }
                        Some("slash_commands") => {
                            // Handle slash commands
                            debug!("Received slash command");
                        }
                        _ => {
                            debug!("Unknown envelope type: {:?}", envelope.envelope_type);
                        }
                    }
                }
                Ok(WsMessage::Ping(data)) => {
                    write.send(WsMessage::Pong(data)).await
                        .map_err(|e| SlackError::WebSocketError(e.to_string()))?;
                }
                Ok(WsMessage::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(WsMessage::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
