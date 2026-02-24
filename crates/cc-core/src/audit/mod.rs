//! Audit and Security Module
//!
//! Provides audit logging, encryption utilities, and security features
//! for the cc-gateway application.

pub mod crypto;
pub mod error;
pub mod logger;
pub mod types;

pub use crypto::{CryptoError, CryptoResult, EncryptedData, EncryptionAlgorithm, EncryptionConfig, SimpleEncryptor};
pub use error::{AuditError, AuditResult};
pub use logger::{AuditEntryBuilder, AuditLogger};
pub use types::{AuditConfig, AuditEntry, AuditEventType, AuditLevel, AuditSource, AuditTarget};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            AuditEventType::MessageSent,
            AuditLevel::Info,
            "Test message",
        );
        assert_eq!(entry.level, AuditLevel::Info);
        assert!(!entry.id.is_empty());
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::new(
            AuditEventType::ApiKeyUsed,
            AuditLevel::Info,
            "API key used",
        )
        .with_correlation_id("req-123");

        assert_eq!(entry.correlation_id, Some("req-123".to_string()));
    }
}
