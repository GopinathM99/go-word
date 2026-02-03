//! Privacy management for telemetry collection.

use serde::{Deserialize, Serialize};

use crate::event::TelemetryEvent;

/// Privacy settings controlling what telemetry data is collected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacySettings {
    /// Master switch for all telemetry
    pub telemetry_enabled: bool,
    /// Allow sending crash reports
    pub crash_reports_enabled: bool,
    /// Allow collecting performance metrics
    pub performance_metrics_enabled: bool,
    /// Allow collecting usage analytics
    pub usage_analytics_enabled: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        // Opt-in by default - all telemetry disabled until user consents
        Self {
            telemetry_enabled: false,
            crash_reports_enabled: false,
            performance_metrics_enabled: false,
            usage_analytics_enabled: false,
        }
    }
}

impl PrivacySettings {
    /// Create settings with all telemetry enabled.
    pub fn all_enabled() -> Self {
        Self {
            telemetry_enabled: true,
            crash_reports_enabled: true,
            performance_metrics_enabled: true,
            usage_analytics_enabled: true,
        }
    }

    /// Create settings with minimal telemetry (only crash reports).
    pub fn minimal() -> Self {
        Self {
            telemetry_enabled: true,
            crash_reports_enabled: true,
            performance_metrics_enabled: false,
            usage_analytics_enabled: false,
        }
    }

    /// Check if any telemetry is allowed.
    pub fn any_enabled(&self) -> bool {
        self.telemetry_enabled
            && (self.crash_reports_enabled
                || self.performance_metrics_enabled
                || self.usage_analytics_enabled)
    }
}

/// Event categories for privacy filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    /// Error and crash events
    CrashReport,
    /// Performance measurements
    PerformanceMetric,
    /// Usage and feature analytics
    UsageAnalytics,
    /// Unknown/other events
    Other,
}

impl EventCategory {
    /// Categorize a telemetry event.
    pub fn from_event(event: &TelemetryEvent) -> Self {
        let name = event.event_name.as_str();

        // Crash/error events
        if name == "error" || name == "crash" || event.is_error_event() {
            return EventCategory::CrashReport;
        }

        // Performance events
        if event.is_performance_event() || name.starts_with("perf_") {
            return EventCategory::PerformanceMetric;
        }

        // Common usage events
        if matches!(
            name,
            "app_start"
                | "app_exit"
                | "doc_open"
                | "doc_save"
                | "doc_export"
                | "command_execute"
                | "feature_use"
        ) {
            return EventCategory::UsageAnalytics;
        }

        EventCategory::Other
    }
}

/// Manager for enforcing privacy settings on telemetry.
#[derive(Debug)]
pub struct PrivacyManager {
    settings: PrivacySettings,
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self {
            settings: PrivacySettings::default(),
        }
    }
}

impl PrivacyManager {
    /// Create a new privacy manager with the given settings.
    pub fn new(settings: PrivacySettings) -> Self {
        Self { settings }
    }

    /// Check if a telemetry event is allowed by current privacy settings.
    pub fn is_allowed(&self, event: &TelemetryEvent) -> bool {
        // Master switch
        if !self.settings.telemetry_enabled {
            return false;
        }

        // Check category-specific settings
        match EventCategory::from_event(event) {
            EventCategory::CrashReport => self.settings.crash_reports_enabled,
            EventCategory::PerformanceMetric => self.settings.performance_metrics_enabled,
            EventCategory::UsageAnalytics => self.settings.usage_analytics_enabled,
            EventCategory::Other => self.settings.usage_analytics_enabled,
        }
    }

    /// Filter a list of events based on privacy settings.
    pub fn filter_events(&self, events: Vec<TelemetryEvent>) -> Vec<TelemetryEvent> {
        events.into_iter().filter(|e| self.is_allowed(e)).collect()
    }

    /// Update privacy settings.
    pub fn set_settings(&mut self, settings: PrivacySettings) {
        self.settings = settings;
    }

    /// Get current privacy settings.
    pub fn get_settings(&self) -> &PrivacySettings {
        &self.settings
    }

    /// Scrub potentially sensitive data from an event.
    ///
    /// This removes or redacts any fields that might contain PII.
    pub fn scrub_event(&self, mut event: TelemetryEvent) -> TelemetryEvent {
        // Remove potentially sensitive properties
        let sensitive_keys = ["file_path", "user_name", "email", "file_name"];
        for key in sensitive_keys {
            event.properties.remove(key);
        }

        // Redact error messages that might contain paths
        if let Some(msg) = event.properties.get_mut("error_message") {
            if let Some(s) = msg.as_str() {
                // Redact file paths
                let redacted = redact_paths(s);
                *msg = serde_json::Value::String(redacted);
            }
        }

        event
    }
}

/// Redact file paths from a string.
fn redact_paths(s: &str) -> String {
    // Simple path redaction - replace anything that looks like a path
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' || c == '\\' {
            // Start of a path - add placeholder
            result.push_str("<path>");
            // Skip until whitespace or end
            while let Some(&next) = chars.peek() {
                if next.is_whitespace() || next == '"' || next == '\'' || next == ')' {
                    break;
                }
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn make_event(name: &str) -> TelemetryEvent {
        TelemetryEvent::new(name, "session", "1.0", "test")
    }

    #[test]
    fn test_privacy_settings_default() {
        let settings = PrivacySettings::default();
        assert!(!settings.telemetry_enabled);
        assert!(!settings.crash_reports_enabled);
        assert!(!settings.performance_metrics_enabled);
        assert!(!settings.usage_analytics_enabled);
    }

    #[test]
    fn test_privacy_settings_all_enabled() {
        let settings = PrivacySettings::all_enabled();
        assert!(settings.telemetry_enabled);
        assert!(settings.crash_reports_enabled);
        assert!(settings.performance_metrics_enabled);
        assert!(settings.usage_analytics_enabled);
    }

    #[test]
    fn test_privacy_settings_minimal() {
        let settings = PrivacySettings::minimal();
        assert!(settings.telemetry_enabled);
        assert!(settings.crash_reports_enabled);
        assert!(!settings.performance_metrics_enabled);
        assert!(!settings.usage_analytics_enabled);
    }

    #[test]
    fn test_privacy_settings_any_enabled() {
        let mut settings = PrivacySettings::default();
        assert!(!settings.any_enabled());

        settings.telemetry_enabled = true;
        assert!(!settings.any_enabled()); // Still false - no specific types enabled

        settings.crash_reports_enabled = true;
        assert!(settings.any_enabled());
    }

    #[test]
    fn test_privacy_settings_serialization() {
        let settings = PrivacySettings::all_enabled();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: PrivacySettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_event_category_crash_report() {
        let error_event = make_event("error");
        assert_eq!(
            EventCategory::from_event(&error_event),
            EventCategory::CrashReport
        );

        let crash_event = make_event("crash");
        assert_eq!(
            EventCategory::from_event(&crash_event),
            EventCategory::CrashReport
        );

        let event_with_error_type = make_event("something")
            .with_property("error_type", "test");
        assert_eq!(
            EventCategory::from_event(&event_with_error_type),
            EventCategory::CrashReport
        );
    }

    #[test]
    fn test_event_category_performance() {
        let perf_event = make_event("perf_layout");
        assert_eq!(
            EventCategory::from_event(&perf_event),
            EventCategory::PerformanceMetric
        );

        let measurement_event = make_event("custom").with_measurement("time", 100.0);
        assert_eq!(
            EventCategory::from_event(&measurement_event),
            EventCategory::PerformanceMetric
        );
    }

    #[test]
    fn test_event_category_usage() {
        let events = [
            "app_start",
            "app_exit",
            "doc_open",
            "doc_save",
            "doc_export",
            "command_execute",
            "feature_use",
        ];

        for name in events {
            let event = make_event(name);
            assert_eq!(
                EventCategory::from_event(&event),
                EventCategory::UsageAnalytics,
                "Failed for event: {}",
                name
            );
        }
    }

    #[test]
    fn test_event_category_other() {
        let event = make_event("custom_event");
        assert_eq!(EventCategory::from_event(&event), EventCategory::Other);
    }

    #[test]
    fn test_privacy_manager_default() {
        let manager = PrivacyManager::default();
        let settings = manager.get_settings();
        assert!(!settings.telemetry_enabled);
    }

    #[test]
    fn test_privacy_manager_is_allowed_master_disabled() {
        let manager = PrivacyManager::new(PrivacySettings::default());
        let event = make_event("error");
        assert!(!manager.is_allowed(&event));
    }

    #[test]
    fn test_privacy_manager_is_allowed_crash_reports() {
        let manager = PrivacyManager::new(PrivacySettings::minimal());

        let error_event = make_event("error");
        assert!(manager.is_allowed(&error_event));

        let usage_event = make_event("app_start");
        assert!(!manager.is_allowed(&usage_event));
    }

    #[test]
    fn test_privacy_manager_is_allowed_all_types() {
        let manager = PrivacyManager::new(PrivacySettings::all_enabled());

        assert!(manager.is_allowed(&make_event("error")));
        assert!(manager.is_allowed(&make_event("perf_layout")));
        assert!(manager.is_allowed(&make_event("app_start")));
        assert!(manager.is_allowed(&make_event("custom")));
    }

    #[test]
    fn test_privacy_manager_filter_events() {
        let manager = PrivacyManager::new(PrivacySettings::minimal());

        let events = vec![
            make_event("error"),
            make_event("app_start"),
            make_event("perf_render"),
        ];

        let filtered = manager.filter_events(events);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].event_name, "error");
    }

    #[test]
    fn test_privacy_manager_set_settings() {
        let mut manager = PrivacyManager::default();
        assert!(!manager.get_settings().telemetry_enabled);

        manager.set_settings(PrivacySettings::all_enabled());
        assert!(manager.get_settings().telemetry_enabled);
    }

    #[test]
    fn test_privacy_manager_scrub_event_removes_sensitive() {
        let manager = PrivacyManager::default();

        let event = make_event("error")
            .with_property("file_path", "/users/john/documents/secret.docx")
            .with_property("user_name", "John Doe")
            .with_property("safe_property", "keep this");

        let scrubbed = manager.scrub_event(event);

        assert!(scrubbed.properties.get("file_path").is_none());
        assert!(scrubbed.properties.get("user_name").is_none());
        assert_eq!(
            scrubbed.properties.get("safe_property"),
            Some(&Value::String("keep this".to_string()))
        );
    }

    #[test]
    fn test_privacy_manager_scrub_event_redacts_paths() {
        let manager = PrivacyManager::default();

        let event = make_event("error")
            .with_property("error_message", "Failed to open /users/test/file.docx");

        let scrubbed = manager.scrub_event(event);

        let msg = scrubbed.properties.get("error_message").unwrap();
        let msg_str = msg.as_str().unwrap();
        assert!(!msg_str.contains("/users"));
        assert!(msg_str.contains("<path>"));
    }

    #[test]
    fn test_redact_paths() {
        assert_eq!(
            redact_paths("Error at /home/user/file.txt"),
            "Error at <path>"
        );

        assert_eq!(
            redact_paths("Path C:\\Users\\Test\\file.txt is invalid"),
            "Path C:<path> is invalid"
        );

        assert_eq!(redact_paths("No paths here"), "No paths here");

        assert_eq!(
            redact_paths("Multiple /path/one and /path/two"),
            "Multiple <path> and <path>"
        );
    }

    #[test]
    fn test_privacy_selective_categories() {
        // Only performance metrics enabled
        let settings = PrivacySettings {
            telemetry_enabled: true,
            crash_reports_enabled: false,
            performance_metrics_enabled: true,
            usage_analytics_enabled: false,
        };
        let manager = PrivacyManager::new(settings);

        assert!(!manager.is_allowed(&make_event("error")));
        assert!(manager.is_allowed(&make_event("perf_layout")));
        assert!(!manager.is_allowed(&make_event("app_start")));
    }
}
