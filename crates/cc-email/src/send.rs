//! Email sending via SMTP
//!
//! Note: This is a simplified implementation stub.
//! Full SMTP implementation requires careful configuration.

use tracing::info;

use crate::error::Result;

/// Email sender configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_address: String,
    pub from_name: Option<String>,
}

/// Email sender
#[derive(Debug)]
pub struct EmailSender {
    config: EmailConfig,
}

impl EmailSender {
    /// Create a new email sender
    pub fn new(config: EmailConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Send an email (stub implementation)
    pub async fn send(&self, to: &str, subject: &str, body: &str, _html: bool) -> Result<String> {
        info!(
            "Sending email to {} via {}:{}",
            to, self.config.smtp_host, self.config.smtp_port
        );

        // Stub implementation
        // Full implementation would use lettre to send
        Ok(format!(
            "Email queued: to={}, subject={}, length={} (SMTP: {}:{})",
            to,
            subject,
            body.len(),
            self.config.smtp_host,
            self.config.smtp_port
        ))
    }

    /// Send a multipart email (stub implementation)
    pub async fn send_multipart(
        &self,
        to: &str,
        subject: &str,
        _text_body: &str,
        _html_body: &str,
    ) -> Result<String> {
        info!(
            "Sending multipart email to {} via {}:{}",
            to, self.config.smtp_host, self.config.smtp_port
        );

        Ok(format!(
            "Multipart email queued: to={}, subject={} (SMTP: {}:{})",
            to, subject, self.config.smtp_host, self.config.smtp_port
        ))
    }
}

impl Clone for EmailSender {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config() {
        let config = EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_user: "user@example.com".to_string(),
            smtp_pass: "password".to_string(),
            from_address: "noreply@example.com".to_string(),
            from_name: Some("Test".to_string()),
        };
        assert_eq!(config.smtp_host, "smtp.example.com");
    }

    #[test]
    fn test_sender_creation() {
        let config = EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_user: "user@example.com".to_string(),
            smtp_pass: "password".to_string(),
            from_address: "noreply@example.com".to_string(),
            from_name: None,
        };
        let sender = EmailSender::new(config);
        assert!(sender.is_ok());
    }
}
