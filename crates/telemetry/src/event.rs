//! Telemetry event types and definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// A telemetry event containing usage or diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryEvent {
    /// Unique identifier for this event
    pub event_id: String,
    /// Name/type of the event
    pub event_name: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Session ID for grouping related events
    pub session_id: String,
    /// Application version
    pub app_version: String,
    /// Platform identifier (e.g., "macos", "windows", "linux")
    pub platform: String,
    /// Custom string properties
    pub properties: HashMap<String, Value>,
    /// Numeric measurements
    pub measurements: HashMap<String, f64>,
}

impl TelemetryEvent {
    /// Create a new telemetry event with the given name.
    pub fn new(
        event_name: &str,
        session_id: &str,
        app_version: &str,
        platform: &str,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_name: event_name.to_string(),
            timestamp: Utc::now(),
            session_id: session_id.to_string(),
            app_version: app_version.to_string(),
            platform: platform.to_string(),
            properties: HashMap::new(),
            measurements: HashMap::new(),
        }
    }

    /// Add a string property to the event.
    pub fn with_property(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.properties.insert(key.to_string(), value.into());
        self
    }

    /// Add a numeric measurement to the event.
    pub fn with_measurement(mut self, key: &str, value: f64) -> Self {
        self.measurements.insert(key.to_string(), value);
        self
    }

    /// Check if this is a performance-related event.
    pub fn is_performance_event(&self) -> bool {
        self.event_name.starts_with("perf_") || !self.measurements.is_empty()
    }

    /// Check if this is an error event.
    pub fn is_error_event(&self) -> bool {
        self.event_name == "error" || self.properties.contains_key("error_type")
    }
}

/// Pre-defined core event types for common telemetry scenarios.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    /// Application started
    AppStart {
        /// Whether this was a cold start (fresh launch) or warm start
        cold_start: bool,
    },
    /// Application exiting
    AppExit {
        /// Total session duration in milliseconds
        session_duration_ms: u64,
    },
    /// Document opened
    DocOpen {
        /// File format (e.g., "docx", "rtf", "odt")
        format: String,
        /// File size in kilobytes
        size_kb: u64,
        /// Number of pages
        page_count: u32,
    },
    /// Document saved
    DocSave {
        /// File format
        format: String,
        /// File size in kilobytes
        size_kb: u64,
        /// Time taken to save in milliseconds
        duration_ms: u64,
    },
    /// Document exported
    DocExport {
        /// Export format
        format: String,
        /// Whether export succeeded
        success: bool,
    },
    /// Command executed
    CommandExecute {
        /// Command identifier
        command_id: String,
        /// How the command was invoked
        source: CommandSource,
    },
    /// Feature used
    FeatureUse {
        /// Name of the feature
        feature_name: String,
    },
    /// Error occurred
    Error {
        /// Category/type of error
        error_type: String,
        /// Error message (sanitized)
        error_message: String,
    },
    /// Performance metric
    Performance {
        /// Metric name
        metric_name: String,
        /// Value in milliseconds
        value_ms: f64,
    },
}

/// Source of a command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CommandSource {
    /// Invoked from menu
    Menu,
    /// Invoked from toolbar
    Toolbar,
    /// Invoked via keyboard shortcut
    Keyboard,
    /// Invoked from context menu
    ContextMenu,
    /// Invoked programmatically via API
    Api,
}

impl std::fmt::Display for CommandSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandSource::Menu => write!(f, "menu"),
            CommandSource::Toolbar => write!(f, "toolbar"),
            CommandSource::Keyboard => write!(f, "keyboard"),
            CommandSource::ContextMenu => write!(f, "context_menu"),
            CommandSource::Api => write!(f, "api"),
        }
    }
}

impl CoreEvent {
    /// Get the event name for this core event type.
    pub fn event_name(&self) -> &'static str {
        match self {
            CoreEvent::AppStart { .. } => "app_start",
            CoreEvent::AppExit { .. } => "app_exit",
            CoreEvent::DocOpen { .. } => "doc_open",
            CoreEvent::DocSave { .. } => "doc_save",
            CoreEvent::DocExport { .. } => "doc_export",
            CoreEvent::CommandExecute { .. } => "command_execute",
            CoreEvent::FeatureUse { .. } => "feature_use",
            CoreEvent::Error { .. } => "error",
            CoreEvent::Performance { .. } => "perf_metric",
        }
    }

    /// Convert to a TelemetryEvent with session info.
    pub fn to_event(
        &self,
        session_id: &str,
        app_version: &str,
        platform: &str,
    ) -> TelemetryEvent {
        let mut event = TelemetryEvent::new(
            self.event_name(),
            session_id,
            app_version,
            platform,
        );

        match self {
            CoreEvent::AppStart { cold_start } => {
                event.properties.insert("cold_start".to_string(), Value::Bool(*cold_start));
            }
            CoreEvent::AppExit { session_duration_ms } => {
                event.measurements.insert("session_duration_ms".to_string(), *session_duration_ms as f64);
            }
            CoreEvent::DocOpen { format, size_kb, page_count } => {
                event.properties.insert("format".to_string(), Value::String(format.clone()));
                event.measurements.insert("size_kb".to_string(), *size_kb as f64);
                event.measurements.insert("page_count".to_string(), *page_count as f64);
            }
            CoreEvent::DocSave { format, size_kb, duration_ms } => {
                event.properties.insert("format".to_string(), Value::String(format.clone()));
                event.measurements.insert("size_kb".to_string(), *size_kb as f64);
                event.measurements.insert("duration_ms".to_string(), *duration_ms as f64);
            }
            CoreEvent::DocExport { format, success } => {
                event.properties.insert("format".to_string(), Value::String(format.clone()));
                event.properties.insert("success".to_string(), Value::Bool(*success));
            }
            CoreEvent::CommandExecute { command_id, source } => {
                event.properties.insert("command_id".to_string(), Value::String(command_id.clone()));
                event.properties.insert("source".to_string(), Value::String(source.to_string()));
            }
            CoreEvent::FeatureUse { feature_name } => {
                event.properties.insert("feature_name".to_string(), Value::String(feature_name.clone()));
            }
            CoreEvent::Error { error_type, error_message } => {
                event.properties.insert("error_type".to_string(), Value::String(error_type.clone()));
                event.properties.insert("error_message".to_string(), Value::String(error_message.clone()));
            }
            CoreEvent::Performance { metric_name, value_ms } => {
                event.properties.insert("metric_name".to_string(), Value::String(metric_name.clone()));
                event.measurements.insert("value_ms".to_string(), *value_ms);
            }
        }

        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_creation() {
        let event = TelemetryEvent::new("test_event", "session-123", "1.0.0", "macos");

        assert!(!event.event_id.is_empty());
        assert_eq!(event.event_name, "test_event");
        assert_eq!(event.session_id, "session-123");
        assert_eq!(event.app_version, "1.0.0");
        assert_eq!(event.platform, "macos");
        assert!(event.properties.is_empty());
        assert!(event.measurements.is_empty());
    }

    #[test]
    fn test_telemetry_event_with_property() {
        let event = TelemetryEvent::new("test", "s", "1.0", "mac")
            .with_property("key", "value");

        assert_eq!(event.properties.get("key"), Some(&Value::String("value".to_string())));
    }

    #[test]
    fn test_telemetry_event_with_measurement() {
        let event = TelemetryEvent::new("test", "s", "1.0", "mac")
            .with_measurement("latency_ms", 42.5);

        assert_eq!(event.measurements.get("latency_ms"), Some(&42.5));
    }

    #[test]
    fn test_telemetry_event_is_performance_event() {
        let perf_event = TelemetryEvent::new("perf_layout", "s", "1.0", "mac");
        assert!(perf_event.is_performance_event());

        let measurement_event = TelemetryEvent::new("test", "s", "1.0", "mac")
            .with_measurement("time", 100.0);
        assert!(measurement_event.is_performance_event());

        let normal_event = TelemetryEvent::new("click", "s", "1.0", "mac");
        assert!(!normal_event.is_performance_event());
    }

    #[test]
    fn test_telemetry_event_is_error_event() {
        let error_event = TelemetryEvent::new("error", "s", "1.0", "mac");
        assert!(error_event.is_error_event());

        let error_type_event = TelemetryEvent::new("test", "s", "1.0", "mac")
            .with_property("error_type", "crash");
        assert!(error_type_event.is_error_event());

        let normal_event = TelemetryEvent::new("click", "s", "1.0", "mac");
        assert!(!normal_event.is_error_event());
    }

    #[test]
    fn test_telemetry_event_serialization() {
        let event = TelemetryEvent::new("test_event", "session-123", "1.0.0", "macos")
            .with_property("key", "value")
            .with_measurement("latency_ms", 42.5);

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TelemetryEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_id, deserialized.event_id);
        assert_eq!(event.event_name, deserialized.event_name);
        assert_eq!(event.properties, deserialized.properties);
        assert_eq!(event.measurements, deserialized.measurements);
    }

    #[test]
    fn test_command_source_display() {
        assert_eq!(CommandSource::Menu.to_string(), "menu");
        assert_eq!(CommandSource::Toolbar.to_string(), "toolbar");
        assert_eq!(CommandSource::Keyboard.to_string(), "keyboard");
        assert_eq!(CommandSource::ContextMenu.to_string(), "context_menu");
        assert_eq!(CommandSource::Api.to_string(), "api");
    }

    #[test]
    fn test_command_source_serialization() {
        let source = CommandSource::Keyboard;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"keyboard\"");

        let deserialized: CommandSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, CommandSource::Keyboard);
    }

    #[test]
    fn test_core_event_app_start() {
        let event = CoreEvent::AppStart { cold_start: true };
        assert_eq!(event.event_name(), "app_start");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("cold_start"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_core_event_app_exit() {
        let event = CoreEvent::AppExit { session_duration_ms: 60000 };
        assert_eq!(event.event_name(), "app_exit");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.measurements.get("session_duration_ms"), Some(&60000.0));
    }

    #[test]
    fn test_core_event_doc_open() {
        let event = CoreEvent::DocOpen {
            format: "docx".to_string(),
            size_kb: 1024,
            page_count: 10,
        };
        assert_eq!(event.event_name(), "doc_open");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("format"), Some(&Value::String("docx".to_string())));
        assert_eq!(telemetry.measurements.get("size_kb"), Some(&1024.0));
        assert_eq!(telemetry.measurements.get("page_count"), Some(&10.0));
    }

    #[test]
    fn test_core_event_doc_save() {
        let event = CoreEvent::DocSave {
            format: "rtf".to_string(),
            size_kb: 512,
            duration_ms: 150,
        };
        assert_eq!(event.event_name(), "doc_save");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("format"), Some(&Value::String("rtf".to_string())));
        assert_eq!(telemetry.measurements.get("size_kb"), Some(&512.0));
        assert_eq!(telemetry.measurements.get("duration_ms"), Some(&150.0));
    }

    #[test]
    fn test_core_event_doc_export() {
        let event = CoreEvent::DocExport {
            format: "pdf".to_string(),
            success: true,
        };
        assert_eq!(event.event_name(), "doc_export");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("format"), Some(&Value::String("pdf".to_string())));
        assert_eq!(telemetry.properties.get("success"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_core_event_command_execute() {
        let event = CoreEvent::CommandExecute {
            command_id: "bold".to_string(),
            source: CommandSource::Keyboard,
        };
        assert_eq!(event.event_name(), "command_execute");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("command_id"), Some(&Value::String("bold".to_string())));
        assert_eq!(telemetry.properties.get("source"), Some(&Value::String("keyboard".to_string())));
    }

    #[test]
    fn test_core_event_feature_use() {
        let event = CoreEvent::FeatureUse {
            feature_name: "spell_check".to_string(),
        };
        assert_eq!(event.event_name(), "feature_use");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("feature_name"), Some(&Value::String("spell_check".to_string())));
    }

    #[test]
    fn test_core_event_error() {
        let event = CoreEvent::Error {
            error_type: "io_error".to_string(),
            error_message: "File not found".to_string(),
        };
        assert_eq!(event.event_name(), "error");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("error_type"), Some(&Value::String("io_error".to_string())));
        assert_eq!(telemetry.properties.get("error_message"), Some(&Value::String("File not found".to_string())));
    }

    #[test]
    fn test_core_event_performance() {
        let event = CoreEvent::Performance {
            metric_name: "layout_time".to_string(),
            value_ms: 16.5,
        };
        assert_eq!(event.event_name(), "perf_metric");

        let telemetry = event.to_event("s", "1.0", "mac");
        assert_eq!(telemetry.properties.get("metric_name"), Some(&Value::String("layout_time".to_string())));
        assert_eq!(telemetry.measurements.get("value_ms"), Some(&16.5));
    }
}
