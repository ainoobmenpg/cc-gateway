//! Webhook server for LINE Bot
//!
//! Handles incoming webhooks from LINE Messaging API

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, error, info, warn};

use crate::handler::MessageHandler;
use crate::types::WebhookBody;

type HmacSha256 = Hmac<Sha256>;

/// Webhook server state
#[derive(Clone)]
pub struct WebhookState {
    pub channel_secret: String,
    pub handler: Arc<MessageHandler>,
}

/// Create webhook router
pub fn create_webhook_router(state: WebhookState) -> Router {
    Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(Arc::new(state))
}

/// Handle incoming webhook
async fn handle_webhook(
    State(state): State<Arc<WebhookState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    // Convert body to string
    let body = String::from_utf8(body.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify signature
    let signature = headers
        .get("x-line-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing x-line-signature header");
            StatusCode::BAD_REQUEST
        })?;

    if !verify_signature(&state.channel_secret, &body, signature) {
        warn!("Invalid signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Parse webhook body
    let webhook: WebhookBody = serde_json::from_str(&body).map_err(|e| {
        error!("Failed to parse webhook body: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    debug!("Received webhook for destination: {}", webhook.destination);

    // Process events
    for event in webhook.events {
        if let Err(e) = state.handler.process_event(&event).await {
            error!("Error processing event: {:?}", e);
            // Continue processing other events
        }
    }

    Ok(StatusCode::OK)
}

/// Verify LINE signature
fn verify_signature(channel_secret: &str, body: &str, signature: &str) -> bool {
    let mut mac = match HmacSha256::new_from_slice(channel_secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body.as_bytes());
    let result = mac.finalize();
    let computed = STANDARD.encode(result.into_bytes());

    computed == signature
}

/// Start webhook server
pub async fn start_webhook_server(
    state: WebhookState,
    port: u16,
) -> crate::error::Result<()> {
    let app = create_webhook_router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| crate::error::LineError::Webhook(e.to_string()))?;

    info!("LINE webhook server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::LineError::Webhook(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature() {
        let secret = "test_secret";
        let body = r#"{"destination":"U123","events":[]}"#;

        // Create a valid signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let result = mac.finalize();
        let valid_signature = STANDARD.encode(result.into_bytes());

        assert!(verify_signature(secret, body, &valid_signature));
        assert!(!verify_signature(secret, body, "invalid_signature"));
    }
}
