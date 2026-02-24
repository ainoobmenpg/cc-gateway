//! Email tools for cc-gateway

use async_trait::async_trait;
use serde_json::{json, Value};

use cc_core::{Tool, ToolResult};

use super::error::Result;
use super::receive::{EmailReceiver, ImapConfig};
use super::send::{EmailConfig, EmailSender};

/// Email send tool
pub struct EmailSendTool {
    sender: EmailSender,
}

impl EmailSendTool {
    pub fn new(config: EmailConfig) -> Result<Self> {
        let sender = EmailSender::new(config)?;
        Ok(Self { sender })
    }
}

#[async_trait]
impl Tool for EmailSendTool {
    fn name(&self) -> &str {
        "email_send"
    }

    fn description(&self) -> &str {
        "Send an email via SMTP"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Recipient email address"
                },
                "subject": {
                    "type": "string",
                    "description": "Email subject"
                },
                "body": {
                    "type": "string",
                    "description": "Email body (text or HTML)"
                },
                "html": {
                    "type": "boolean",
                    "description": "Whether body is HTML (default: false)",
                    "default": false
                }
            },
            "required": ["to", "subject", "body"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let to = input["to"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing 'to' parameter".to_string()))?;
        let subject = input["subject"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing 'subject' parameter".to_string()))?;
        let body = input["body"]
            .as_str()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing 'body' parameter".to_string()))?;
        let html = input["html"].as_bool().unwrap_or(false);

        let result = self
            .sender
            .send(to, subject, body, html)
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(serde_json::to_string(&json!({
            "status": "sent",
            "to": to,
            "subject": subject,
            "message": result
        })).unwrap_or_default()))
    }
}

/// Email list tool
pub struct EmailListTool {
    receiver: EmailReceiver,
}

impl EmailListTool {
    pub fn new(config: ImapConfig) -> Self {
        Self {
            receiver: EmailReceiver::new(config),
        }
    }
}

#[async_trait]
impl Tool for EmailListTool {
    fn name(&self) -> &str {
        "email_list"
    }

    fn description(&self) -> &str {
        "List emails from an IMAP folder"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {
                    "type": "string",
                    "description": "IMAP folder name (default: INBOX)",
                    "default": "INBOX"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of emails to return (default: 10)",
                    "default": 10
                }
            }
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let folder = input["folder"].as_str().unwrap_or("INBOX");
        let limit = input["limit"].as_u64().unwrap_or(10) as usize;

        let emails = self
            .receiver
            .list_emails(folder, limit)
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        let summaries: Vec<Value> = emails
            .iter()
            .map(|e| {
                json!({
                    "uid": e.uid,
                    "subject": e.subject,
                    "from": e.from,
                    "date": e.date,
                    "flags": e.flags
                })
            })
            .collect();

        Ok(ToolResult::success(serde_json::to_string(&json!({
            "folder": folder,
            "count": summaries.len(),
            "emails": summaries
        })).unwrap_or_default()))
    }
}

/// Email read tool
pub struct EmailReadTool {
    receiver: EmailReceiver,
}

impl EmailReadTool {
    pub fn new(config: ImapConfig) -> Self {
        Self {
            receiver: EmailReceiver::new(config),
        }
    }
}

#[async_trait]
impl Tool for EmailReadTool {
    fn name(&self) -> &str {
        "email_read"
    }

    fn description(&self) -> &str {
        "Read a specific email by UID"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {
                    "type": "string",
                    "description": "IMAP folder name (default: INBOX)",
                    "default": "INBOX"
                },
                "uid": {
                    "type": "integer",
                    "description": "Email UID to read"
                }
            },
            "required": ["uid"]
        })
    }

    async fn execute(&self, input: Value) -> cc_core::Result<ToolResult> {
        let folder = input["folder"].as_str().unwrap_or("INBOX");
        let uid = input["uid"]
            .as_u64()
            .ok_or_else(|| cc_core::Error::ToolExecution("Missing 'uid' parameter".to_string()))?
            as u32;

        let body = self
            .receiver
            .get_email(folder, uid)
            .await
            .map_err(|e| cc_core::Error::ToolExecution(e.to_string()))?;

        Ok(ToolResult::success(serde_json::to_string(&json!({
            "folder": folder,
            "uid": uid,
            "body": body,
            "length": body.len()
        })).unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config() {
        let config = EmailConfig {
            smtp_host: "localhost".to_string(),
            smtp_port: 25,
            smtp_user: "test".to_string(),
            smtp_pass: "test".to_string(),
            from_address: "test@test.com".to_string(),
            from_name: None,
        };
        assert_eq!(config.smtp_host, "localhost");
    }

    #[test]
    fn test_imap_config() {
        let config = ImapConfig {
            imap_host: "localhost".to_string(),
            imap_port: 993,
            imap_user: "test".to_string(),
            imap_pass: "test".to_string(),
        };
        assert_eq!(config.imap_host, "localhost");
    }
}
