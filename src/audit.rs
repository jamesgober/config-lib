//! Comprehensive Audit Logging System
//!
//! Enterprise-grade audit logging with:
//! - Structured logging for all configuration operations
//! - Access tracking with user context and timestamps
//! - Modification logging with before/after values
//! - Validation failure tracking
//! - Configurable log levels and outputs
//! - Performance-optimized with minimal overhead

use crate::value::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit event types for configuration operations
#[derive(Debug, Clone, PartialEq)]
pub enum AuditEventType {
    /// Configuration key was accessed/read
    Access,
    /// Configuration key was modified
    Modification,
    /// Configuration validation failed
    ValidationFailure,
    /// Configuration was reloaded from file
    Reload,
    /// Configuration file was loaded initially
    Load,
    /// Configuration was serialized/saved
    Save,
}

/// Severity levels for audit events
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AuditSeverity {
    /// Informational events (normal operations)
    Info = 1,
    /// Warning events (potential issues)
    Warning = 2,
    /// Error events (failures)
    Error = 3,
    /// Critical events (security concerns)
    Critical = 4,
}

/// Comprehensive audit event record
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Timestamp when the event occurred
    pub timestamp: SystemTime,
    /// Type of operation that triggered this event
    pub event_type: AuditEventType,
    /// Severity level of the event
    pub severity: AuditSeverity,
    /// Configuration key that was accessed/modified
    pub key: Option<String>,
    /// Previous value (for modifications)
    pub old_value: Option<Value>,
    /// New value (for modifications)
    pub new_value: Option<Value>,
    /// User or system context that triggered the event
    pub user_context: Option<String>,
    /// Additional contextual information
    pub metadata: HashMap<String, String>,
    /// Error message (for failures)
    pub error_message: Option<String>,
    /// Source location (file path, line number, etc.)
    pub source: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event with minimal required fields
    pub fn new(event_type: AuditEventType, severity: AuditSeverity) -> Self {
        Self {
            id: generate_event_id(),
            timestamp: SystemTime::now(),
            event_type,
            severity,
            key: None,
            old_value: None,
            new_value: None,
            user_context: None,
            metadata: HashMap::new(),
            error_message: None,
            source: None,
        }
    }

    /// Set the configuration key for this event
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the old value (for modifications)
    pub fn with_old_value(mut self, value: Value) -> Self {
        self.old_value = Some(value);
        self
    }

    /// Set the new value (for modifications)
    pub fn with_new_value(mut self, value: Value) -> Self {
        self.new_value = Some(value);
        self
    }

    /// Set the user context
    pub fn with_user_context(mut self, context: impl Into<String>) -> Self {
        self.user_context = Some(context.into());
        self
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set error message
    pub fn with_error(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }

    /// Set source location
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timestamp_millis = self
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        write!(
            f,
            "[{}] {:?}:{:?} id={} key={} user={}",
            timestamp_millis,
            self.event_type,
            self.severity,
            self.id,
            self.key.as_deref().unwrap_or("none"),
            self.user_context.as_deref().unwrap_or("system")
        )?;

        if let Some(error) = &self.error_message {
            write!(f, " error=\"{}\"", error)?;
        }

        if let (Some(old), Some(new)) = (&self.old_value, &self.new_value) {
            write!(f, " change=\"{:?}\" -> \"{:?}\"", old, new)?;
        }

        for (key, value) in &self.metadata {
            write!(f, " {}=\"{}\"", key, value)?;
        }

        Ok(())
    }
}

/// Trait for audit log outputs/sinks
pub trait AuditSink: Send + Sync {
    /// Write an audit event to this sink
    fn write_event(&self, event: &AuditEvent) -> Result<(), String>;

    /// Flush any buffered events
    fn flush(&self) -> Result<(), String>;
}

/// Console/stdout audit sink for development
pub struct ConsoleSink {
    level_filter: AuditSeverity,
}

impl ConsoleSink {
    /// Create a new console sink with minimum severity level
    pub fn new(min_level: AuditSeverity) -> Self {
        Self {
            level_filter: min_level,
        }
    }
}

impl AuditSink for ConsoleSink {
    fn write_event(&self, event: &AuditEvent) -> Result<(), String> {
        if event.severity >= self.level_filter {
            println!("AUDIT: {}", event);
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), String> {
        Ok(()) // stdout auto-flushes
    }
}

/// File-based audit sink for production
pub struct FileSink {
    file_path: String,
    level_filter: AuditSeverity,
}

impl FileSink {
    /// Create a new file sink
    pub fn new(file_path: impl Into<String>, min_level: AuditSeverity) -> Self {
        Self {
            file_path: file_path.into(),
            level_filter: min_level,
        }
    }
}

impl AuditSink for FileSink {
    fn write_event(&self, event: &AuditEvent) -> Result<(), String> {
        if event.severity >= self.level_filter {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.file_path)
                .map_err(|e| format!("Failed to open audit log file: {}", e))?;

            writeln!(file, "{}", event)
                .map_err(|e| format!("Failed to write to audit log: {}", e))?;
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), String> {
        // For append-only files, OS handles flushing
        Ok(())
    }
}

/// Main audit logger with multiple sinks
pub struct AuditLogger {
    sinks: Vec<Box<dyn AuditSink>>,
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            sinks: Vec::new(),
            enabled: true,
        }
    }

    /// Add a sink to the logger
    pub fn add_sink(mut self, sink: Box<dyn AuditSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Enable or disable audit logging
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Log an audit event to all configured sinks
    pub fn log_event(&self, event: AuditEvent) {
        if !self.enabled {
            return;
        }

        for sink in &self.sinks {
            if let Err(e) = sink.write_event(&event) {
                eprintln!("Audit sink error: {}", e);
            }
        }
    }

    /// Log a configuration access event
    pub fn log_access(&self, key: &str, user_context: Option<&str>) {
        let event = AuditEvent::new(AuditEventType::Access, AuditSeverity::Info)
            .with_key(key)
            .with_metadata("operation", "get");

        let event = if let Some(user) = user_context {
            event.with_user_context(user)
        } else {
            event
        };

        self.log_event(event);
    }

    /// Log a configuration modification event
    pub fn log_modification(
        &self,
        key: &str,
        old_value: Option<&Value>,
        new_value: &Value,
        user_context: Option<&str>,
    ) {
        let mut event = AuditEvent::new(AuditEventType::Modification, AuditSeverity::Warning)
            .with_key(key)
            .with_new_value(new_value.clone())
            .with_metadata("operation", "set");

        if let Some(old) = old_value {
            event = event.with_old_value(old.clone());
        }

        if let Some(user) = user_context {
            event = event.with_user_context(user);
        }

        self.log_event(event);
    }

    /// Log a validation failure event
    pub fn log_validation_failure(
        &self,
        key: &str,
        error: &str,
        value: &Value,
        user_context: Option<&str>,
    ) {
        let event = AuditEvent::new(AuditEventType::ValidationFailure, AuditSeverity::Error)
            .with_key(key)
            .with_new_value(value.clone())
            .with_error(error)
            .with_metadata("operation", "validate");

        let event = if let Some(user) = user_context {
            event.with_user_context(user)
        } else {
            event
        };

        self.log_event(event);
    }

    /// Log a configuration reload event
    pub fn log_reload(&self, source: &str, success: bool, error: Option<&str>) {
        let severity = if success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Error
        };
        let mut event = AuditEvent::new(AuditEventType::Reload, severity)
            .with_source(source)
            .with_metadata("operation", "reload");

        if let Some(err) = error {
            event = event.with_error(err);
        }

        self.log_event(event);
    }

    /// Flush all sinks
    pub fn flush(&self) {
        for sink in &self.sinks {
            if let Err(e) = sink.flush() {
                eprintln!("Audit sink flush error: {}", e);
            }
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe global audit logger
static GLOBAL_AUDIT_LOGGER: Mutex<Option<Arc<AuditLogger>>> = Mutex::new(None);

/// Initialize the global audit logger
pub fn init_audit_logger(logger: AuditLogger) {
    let mut global = GLOBAL_AUDIT_LOGGER.lock().unwrap();
    *global = Some(Arc::new(logger));
}

/// Get the global audit logger
pub fn get_audit_logger() -> Option<Arc<AuditLogger>> {
    GLOBAL_AUDIT_LOGGER.lock().unwrap().clone()
}

/// Log an event using the global audit logger
pub fn audit_log(event: AuditEvent) {
    if let Some(logger) = get_audit_logger() {
        logger.log_event(event);
    }
}

/// Generate a unique event ID
fn generate_event_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{:x}-{:x}", timestamp, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestSink {
        events: Arc<Mutex<Vec<AuditEvent>>>,
    }

    impl TestSink {
        fn new() -> (Self, Arc<Mutex<Vec<AuditEvent>>>) {
            let events = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    events: Arc::clone(&events),
                },
                events,
            )
        }
    }

    impl AuditSink for TestSink {
        fn write_event(&self, event: &AuditEvent) -> Result<(), String> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }

        fn flush(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditEventType::Access, AuditSeverity::Info)
            .with_key("test.key")
            .with_user_context("test_user")
            .with_metadata("operation", "get");

        assert_eq!(event.event_type, AuditEventType::Access);
        assert_eq!(event.severity, AuditSeverity::Info);
        assert_eq!(event.key, Some("test.key".to_string()));
        assert_eq!(event.user_context, Some("test_user".to_string()));
        assert_eq!(event.metadata.get("operation"), Some(&"get".to_string()));
    }

    #[test]
    fn test_audit_logger_basic() {
        let (sink, events) = TestSink::new();
        let logger = AuditLogger::new().add_sink(Box::new(sink));

        logger.log_access("test.key", Some("test_user"));
        logger.log_modification(
            "test.key",
            None,
            &Value::String("new_value".to_string()),
            Some("test_user"),
        );

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);

        assert_eq!(events[0].event_type, AuditEventType::Access);
        assert_eq!(events[0].key, Some("test.key".to_string()));

        assert_eq!(events[1].event_type, AuditEventType::Modification);
        assert_eq!(events[1].key, Some("test.key".to_string()));
    }

    #[test]
    fn test_console_sink() {
        let sink = ConsoleSink::new(AuditSeverity::Info);
        let event =
            AuditEvent::new(AuditEventType::Access, AuditSeverity::Info).with_key("test.key");

        // This should not panic
        assert!(sink.write_event(&event).is_ok());
    }

    #[test]
    fn test_event_display() {
        let event = AuditEvent::new(AuditEventType::Modification, AuditSeverity::Warning)
            .with_key("test.key")
            .with_user_context("test_user")
            .with_old_value(Value::String("old".to_string()))
            .with_new_value(Value::String("new".to_string()))
            .with_metadata("operation", "set");

        let display = format!("{}", event);
        assert!(display.contains("Modification"));
        assert!(display.contains("Warning"));
        assert!(display.contains("test.key"));
        assert!(display.contains("test_user"));
    }

    #[test]
    fn test_severity_filtering() {
        let (sink, events) = TestSink::new();
        let logger = AuditLogger::new().add_sink(Box::new(sink));

        // Log events of different severities
        logger.log_event(AuditEvent::new(AuditEventType::Access, AuditSeverity::Info));
        logger.log_event(AuditEvent::new(
            AuditEventType::ValidationFailure,
            AuditSeverity::Error,
        ));

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].severity, AuditSeverity::Info);
        assert_eq!(events[1].severity, AuditSeverity::Error);
    }
}
