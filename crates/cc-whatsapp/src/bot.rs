//! WhatsApp bot wrapper

use std::net::SocketAddr;
use std::sync::Arc;

use crate::error::Result;
use crate::twilio::TwilioClient;
use crate::webhook::WebhookServer;

/// WhatsApp bot wrapper
pub struct WhatsAppBot {
    twilio_client: Arc<TwilioClient>,
    claude_client: Arc<cc_core::ClaudeClient>,
    admin_numbers: Vec<String>,
    port: u16,
}

impl WhatsAppBot {
    /// Create a new WhatsApp bot
    pub fn new(
        account_sid: &str,
        auth_token: &str,
        phone_number: &str,
        claude_client: Arc<cc_core::ClaudeClient>,
        admin_numbers: Vec<String>,
        port: u16,
    ) -> Self {
        let twilio_client = Arc::new(TwilioClient::new(
            account_sid.to_string(),
            auth_token.to_string(),
            phone_number.to_string(),
        ));

        Self {
            twilio_client,
            claude_client,
            admin_numbers,
            port,
        }
    }

    /// Start the bot (webhook server)
    pub async fn start(self) -> Result<()> {
        let addr: SocketAddr = ([0, 0, 0, 0], self.port).into();
        let server = WebhookServer::new(
            addr,
            self.twilio_client,
            self.claude_client,
            self.admin_numbers,
        );

        server.start().await
    }

    /// Get the Twilio client for direct use
    pub fn twilio_client(&self) -> Arc<TwilioClient> {
        Arc::clone(&self.twilio_client)
    }
}
