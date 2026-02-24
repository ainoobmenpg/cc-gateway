//! Apple Script utilities for iMessage
//!
//! macOS の Apple Script を使用して Messages.app と対話します。

use std::process::Command;
use tracing::{debug, error};

use crate::error::{IMessageError, Result};

/// Apple Script executor for iMessage operations
pub struct AppleScript;

impl AppleScript {
    /// Execute an Apple Script and return the output
    fn execute(script: &str) -> Result<String> {
        debug!("Executing Apple Script: {}", script.lines().next().unwrap_or(""));

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(IMessageError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Apple Script failed: {}", stderr);
            return Err(IMessageError::ScriptError(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    /// Check if Messages.app is running
    pub fn is_messages_running() -> Result<bool> {
        let script = r#"
            tell application "System Events"
                return (name of processes) contains "Messages"
            end tell
        "#;

        let result = Self::execute(script)?;
        Ok(result == "true")
    }

    /// Activate Messages.app (bring to front)
    pub fn activate_messages() -> Result<()> {
        let script = r#"
            tell application "Messages"
                activate
            end tell
        "#;

        Self::execute(script)?;
        Ok(())
    }

    /// Send a message to a specific contact (by phone number or email)
    ///
    /// # Arguments
    /// * `recipient` - Phone number (e.g., "+819012345678") or email
    /// * `message` - Message text to send
    pub fn send_message(recipient: &str, message: &str) -> Result<()> {
        // Escape special characters for Apple Script
        let escaped_message = Self::escape_applescript(message);
        let escaped_recipient = Self::escape_applescript(recipient);

        let script = format!(
            r#"
            tell application "Messages"
                set targetService to 1st service whose service type = iMessage
                set targetBuddy to buddy "{}" of targetService
                send "{}" to targetBuddy
            end tell
            "#,
            escaped_recipient, escaped_message
        );

        Self::execute(&script)?;
        debug!("Message sent to {}", recipient);
        Ok(())
    }

    /// Send a message using an existing chat (by chat name)
    ///
    /// # Arguments
    /// * `chat_name` - Name of the chat (contact name or group name)
    /// * `message` - Message text to send
    pub fn send_to_chat(chat_name: &str, message: &str) -> Result<()> {
        let escaped_message = Self::escape_applescript(message);
        let escaped_chat = Self::escape_applescript(chat_name);

        let script = format!(
            r#"
            tell application "Messages"
                set targetChat to 1st chat whose name = "{}"
                send "{}" to targetChat
            end tell
            "#,
            escaped_chat, escaped_message
        );

        Self::execute(&script)?;
        debug!("Message sent to chat: {}", chat_name);
        Ok(())
    }

    /// Get unread messages count
    pub fn get_unread_count() -> Result<i32> {
        let script = r#"
            tell application "Messages"
                set unreadCount to 0
                repeat with eachChat in chats
                    set unreadCount to unreadCount + (unread count of eachChat)
                end repeat
                return unreadCount
            end tell
        "#;

        let result = Self::execute(script)?;
        result.parse().map_err(|e| IMessageError::ParseError(format!("Failed to parse unread count: {}", e)))
    }

    /// Get recent messages from a specific chat
    ///
    /// Returns a list of messages with sender and content
    pub fn get_recent_messages(chat_name: &str, limit: usize) -> Result<Vec<ReceivedMessage>> {
        let escaped_chat = Self::escape_applescript(chat_name);

        let script = format!(
            r#"
            tell application "Messages"
                set targetChat to 1st chat whose name = "{}"
                set messageList to {{}}
                set chatMessages to messages of targetChat
                set msgCount to count of chatMessages
                set startIndex to 1
                if msgCount > {} then
                    set startIndex to msgCount - {} + 1
                end if
                repeat with i from startIndex to msgCount
                    set msg to item i of chatMessages
                    set msgSender to sender of msg as string
                    set msgContent to content of msg as string
                    set msgDate to date sent of msg as string
                    set end of messageList to msgSender & "|||" & msgContent & "|||" & msgDate
                end repeat
                return messageList as string
            end tell
            "#,
            escaped_chat, limit, limit
        );

        let result = Self::execute(&script)?;

        if result.is_empty() {
            return Ok(Vec::new());
        }

        let messages: Vec<ReceivedMessage> = result
            .split(", ")
            .filter_map(|entry| {
                let parts: Vec<&str> = entry.split("|||").collect();
                if parts.len() >= 3 {
                    Some(ReceivedMessage {
                        sender: Self::clean_sender(parts[0]),
                        content: parts[1].to_string(),
                        timestamp: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(messages)
    }

    /// Get list of all chats
    pub fn get_chats() -> Result<Vec<ChatInfo>> {
        let script = r#"
            tell application "Messages"
                set chatList to {}
                repeat with eachChat in chats
                    set chatName to name of eachChat
                    set chatID to id of eachChat
                    set end of chatList to chatName & "|||" & chatID
                end repeat
                return chatList as string
            end tell
        "#;

        let result = Self::execute(script)?;

        if result.is_empty() {
            return Ok(Vec::new());
        }

        let chats: Vec<ChatInfo> = result
            .split(", ")
            .filter_map(|entry| {
                let parts: Vec<&str> = entry.split("|||").collect();
                if parts.len() >= 2 {
                    Some(ChatInfo {
                        name: parts[0].to_string(),
                        id: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(chats)
    }

    /// Escape special characters for Apple Script strings
    fn escape_applescript(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Clean sender string (remove formatting)
    fn clean_sender(sender: &str) -> String {
        // Format is usually: "name <email@icloud.com>" or just the phone number
        let re = regex::Regex::new(r"<([^>]+)>").unwrap();
        if let Some(caps) = re.captures(sender) {
            caps[1].to_string()
        } else {
            sender.trim().to_string()
        }
    }
}

/// Received message information
#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
}

/// Chat information
#[derive(Debug, Clone)]
pub struct ChatInfo {
    pub name: String,
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_applescript() {
        assert_eq!(
            AppleScript::escape_applescript(r#"Hello "world""#),
            r#"Hello \"world\""#
        );
        assert_eq!(
            AppleScript::escape_applescript("Line1\nLine2"),
            "Line1\\nLine2"
        );
    }

    #[test]
    fn test_clean_sender() {
        assert_eq!(
            AppleScript::clean_sender("John Doe <john@example.com>"),
            "john@example.com"
        );
        assert_eq!(
            AppleScript::clean_sender("+819012345678"),
            "+819012345678"
        );
    }
}
