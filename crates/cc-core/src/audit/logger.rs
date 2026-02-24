//! Audit logger implementation

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing::{debug, error, info, warn};

use super::error::{AuditError, AuditResult};
use super::types::{AuditConfig, AuditEntry, AuditLevel};

/// Audit logger that writes entries to file and/or console
pub struct AuditLogger {
    config: AuditConfig,
    log_file: Arc<Mutex<Option<File>>>,
    current_file_size: Arc<Mutex<usize>>,
}

impl AuditLogger {
    /// Create a new audit logger with the given configuration
    pub fn new(config: AuditConfig) -> AuditResult<Self> {
        let (log_file, file_size) = if let Some(ref path) = config.log_file {
            // Ensure log directory exists
            let path = PathBuf::from(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    AuditError::ConfigurationError(format!("Failed to create log directory: {}", e))
                })?;
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| {
                    AuditError::ConfigurationError(format!("Failed to open log file: {}", e))
                })?;
            let metadata = file.metadata().map_err(|e| {
                AuditError::ConfigurationError(format!("Failed to get file metadata: {}", e))
            })?;
            (Some(file), metadata.len() as usize)
        } else {
            (None, 0)
        };

        Ok(Self {
            config,
            log_file: Arc::new(Mutex::new(log_file)),
            current_file_size: Arc::new(Mutex::new(file_size)),
        })
    }

    /// Log an audit entry
    pub fn log(&self, entry: &AuditEntry) -> AuditResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if we should log this level
        if entry.level < self.config.min_level {
            return Ok(());
        }

        let json = serde_json::to_string(entry)?;

        // Log to console if enabled
        if self.config.log_to_console {
            match entry.level {
                AuditLevel::Info => info!("[AUDIT] {}", json),
                AuditLevel::Warning => warn!("[AUDIT] {}", json),
                AuditLevel::Error | AuditLevel::Critical => error!("[AUDIT] {}", json),
            }
        }

        // Log to file if enabled
        if let Some(ref _path) = self.config.log_file {
            self.write_to_file(&json)?;
        }

        Ok(())
    }

    /// Write a line to the log file
    fn write_to_file(&self, line: &str) -> AuditResult<()> {
        let mut file_guard = self.log_file.lock().unwrap();
        let mut size_guard = self.current_file_size.lock().unwrap();

        if let Some(ref mut file) = *file_guard {
            let line_with_newline = format!("{}\n", line);
            file.write_all(line_with_newline.as_bytes())?;

            *size_guard += line_with_newline.len();

            // Check if rotation is needed
            if *size_guard >= self.config.max_file_size {
                drop(file_guard);
                drop(size_guard);
                self.rotate_log()?;
            }
        }

        Ok(())
    }

    /// Rotate the log file
    fn rotate_log(&self) -> AuditResult<()> {
        let path = self.config.log_file.as_ref().ok_or_else(|| {
            AuditError::RotationError("No log file configured".to_string())
        })?;
        let path = PathBuf::from(path);

        debug!("Rotating audit log: {:?}", path);

        // Remove oldest rotated file if it exists
        let oldest = format!("{}.{}", path.display(), self.config.max_rotated_files);
        if PathBuf::from(&oldest).exists() {
            fs::remove_file(&oldest)
                .map_err(|e| AuditError::RotationError(format!("Failed to remove old log: {}", e)))?;
        }

        // Rotate existing files
        for i in (1..=self.config.max_rotated_files).rev() {
            let old_path = format!("{}.{}", path.display(), i);
            let new_path = format!("{}.{}", path.display(), i + 1);
            if PathBuf::from(&old_path).exists() {
                fs::rename(&old_path, &new_path)
                    .map_err(|e| AuditError::RotationError(format!("Failed to rotate log: {}", e)))?;
            }
        }

        // Move current file to .1
        if path.exists() {
            let rotated = format!("{}.1", path.display());
            fs::rename(&path, &rotated)
                .map_err(|e| AuditError::RotationError(format!("Failed to rename current log: {}", e)))?;
        }

        // Open new file
        let mut file_guard = self.log_file.lock().unwrap();
        let mut size_guard = self.current_file_size.lock().unwrap();

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| AuditError::RotationError(format!("Failed to create new log file: {}", e)))?;

        *file_guard = Some(new_file);
        *size_guard = 0;

        info!("Audit log rotated successfully");
        Ok(())
    }

    /// Create a builder for audit entries
    pub fn builder(&self) -> AuditEntryBuilder {
        AuditEntryBuilder::new()
    }
}

/// Builder for creating audit entries
pub struct AuditEntryBuilder {
    entry: AuditEntry,
}

impl AuditEntryBuilder {
    fn new() -> Self {
        // Default entry - will be overwritten
        Self {
            entry: AuditEntry::new(
                super::types::AuditEventType::MessageSent,
                AuditLevel::Info,
                "",
            ),
        }
    }

    /// Set the event type
    pub fn event_type(mut self, event_type: super::types::AuditEventType) -> Self {
        self.entry.event_type = event_type;
        self
    }

    /// Set the level
    pub fn level(mut self, level: AuditLevel) -> Self {
        self.entry.level = level;
        self
    }

    /// Set the message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.entry.message = message.into();
        self
    }

    /// Set the source IP address
    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        let source = self.entry.source.get_or_insert(super::types::AuditSource {
            ip_address: None,
            user_agent: None,
            gateway: None,
            channel_id: None,
        });
        source.ip_address = Some(ip.into());
        self
    }

    /// Set the gateway
    pub fn gateway(mut self, gateway: impl Into<String>) -> Self {
        let source = self.entry.source.get_or_insert(super::types::AuditSource {
            ip_address: None,
            user_agent: None,
            gateway: None,
            channel_id: None,
        });
        source.gateway = Some(gateway.into());
        self
    }

    /// Set the channel ID
    pub fn channel(mut self, channel_id: impl Into<String>) -> Self {
        let source = self.entry.source.get_or_insert(super::types::AuditSource {
            ip_address: None,
            user_agent: None,
            gateway: None,
            channel_id: None,
        });
        source.channel_id = Some(channel_id.into());
        self
    }

    /// Set the target
    pub fn target(mut self, resource_type: impl Into<String>, action: impl Into<String>) -> Self {
        self.entry.target = Some(super::types::AuditTarget {
            resource_type: resource_type.into(),
            resource_id: None,
            action: action.into(),
        });
        self
    }

    /// Set the target with resource ID
    pub fn target_with_id(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        self.entry.target = Some(super::types::AuditTarget {
            resource_type: resource_type.into(),
            resource_id: Some(resource_id.into()),
            action: action.into(),
        });
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.entry.metadata = Some(metadata);
        self
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.entry.correlation_id = Some(id.into());
        self
    }

    /// Build the entry
    pub fn build(self) -> AuditEntry {
        self.entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::AuditEventType;
    use tempfile::TempDir;

    #[test]
    fn test_audit_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = AuditConfig {
            log_file: Some(log_path.to_str().unwrap().to_string()),
            ..Default::default()
        };
        let logger = AuditLogger::new(config).unwrap();
        assert!(logger.config.enabled);
    }

    #[test]
    fn test_log_entry() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = AuditConfig {
            log_file: Some(log_path.to_str().unwrap().to_string()),
            log_to_console: false,
            ..Default::default()
        };
        let logger = AuditLogger::new(config).unwrap();

        let entry = AuditEntry::new(
            AuditEventType::MessageSent,
            AuditLevel::Info,
            "Test message",
        );
        logger.log(&entry).unwrap();

        let contents = fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("Test message"));
    }
}
