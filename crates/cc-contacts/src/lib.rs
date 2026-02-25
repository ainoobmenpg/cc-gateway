//! cc-contacts: Contacts integration for cc-gateway
//!
//! This crate provides CardDAV contact management capabilities.
//!
//! ## Features
//!
//! - CardDAV client for contact access
//! - Contact creation, retrieval, and deletion
//! - Support for multiple contact providers
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cc_contacts::{ContactsClient, ContactsConfig, Contact};
//!
//! let config = ContactsConfig {
//!     server_url: "https://carddav.example.com".to_string(),
//!     username: "user".to_string(),
//!     password: "password".to_string(),
//!     addressbook_id: Some("contacts".to_string()),
//! };
//! let client = ContactsClient::new(config).await?;
//!
//! // Get all contacts
//! let contacts = client.get_contacts().await?;
//!
//! // Add contact
//! let contact = Contact {
//!     full_name: "John Doe".to_string(),
//!     email: Some("john@example.com".to_string()),
//!     phone: Some("+1234567890".to_string()),
//!     ..Default::default()
//! };
//! client.add_contact(contact).await?;
//! ```

pub mod client;
pub mod error;
pub mod models;

pub use client::ContactsClient;
pub use error::{ContactsError, Result};
pub use models::{Contact, ContactsConfig};

/// Re-export models for easy use
pub mod prelude {
    pub use super::{Contact, ContactsClient, ContactsConfig};
}
