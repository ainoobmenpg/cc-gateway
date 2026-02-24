//! cc-whatsapp: WhatsApp Bot for cc-gateway via Twilio API
//!
//! This crate provides WhatsApp bot integration for cc-gateway,
//! using Twilio's WhatsApp Business API.

pub mod bot;
pub mod error;
pub mod session;
pub mod twilio;
pub mod webhook;

pub use bot::WhatsAppBot;
pub use error::{Result, WhatsAppError};
pub use session::InMemorySessionStore;
pub use twilio::TwilioClient;
pub use webhook::WebhookServer;
