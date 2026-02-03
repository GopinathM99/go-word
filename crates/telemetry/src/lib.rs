//! Telemetry System
//!
//! This crate provides a comprehensive telemetry system for analytics and diagnostics
//! in the word processor application. It supports:
//!
//! - Event tracking for user actions, feature usage, and errors
//! - Performance metrics collection with statistical analysis
//! - Privacy-first design with configurable data collection
//! - Batched transport with offline support
//! - Crash reporting and recovery management
//! - Performance profiling with hierarchical spans
//! - Memory profiling and leak detection
//! - Document inspection for debugging
//! - Support report generation
//!
//! # Privacy
//!
//! All telemetry is opt-in by default. Users must explicitly enable telemetry
//! collection through privacy settings. The system supports:
//!
//! - Master telemetry switch
//! - Per-category toggles (crash reports, performance, usage analytics)
//! - Automatic scrubbing of potentially sensitive data
//!
//! # Example
//!
//! ```rust
//! use telemetry::{TelemetryClient, TelemetryConfig, PrivacySettings, CoreEvent};
//!
//! // Create a telemetry client
//! let config = TelemetryConfig::new("1.0.0")
//!     .with_endpoint("https://telemetry.example.com")
//!     .with_privacy(PrivacySettings::all_enabled());
//!
//! let mut client = TelemetryClient::new(config);
//!
//! // Track application start
//! client.track_app_start(true);
//!
//! // Track a custom event
//! client.track(CoreEvent::FeatureUse {
//!     feature_name: "spell_check".to_string(),
//! });
//!
//! // Track performance metrics
//! use telemetry::PerformanceMetrics;
//! client.record_metrics(PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0));
//! ```
//!
//! # Modules
//!
//! - [`event`] - Telemetry event types and core event definitions
//! - [`metrics`] - Performance metrics collection and analysis
//! - [`privacy`] - Privacy settings and filtering
//! - [`session`] - Session management
//! - [`transport`] - Event batching and transport
//! - [`client`] - High-level telemetry client
//! - [`crash`] - Crash reporting and recovery
//! - [`error`] - Error types
//! - [`profiler`] - Performance profiling with hierarchical spans
//! - [`memory`] - Memory profiling and leak detection
//! - [`inspector`] - Document inspection for debugging
//! - [`report`] - Support report generation

mod client;
pub mod crash;
mod error;
mod event;
pub mod inspector;
pub mod memory;
mod metrics;
mod privacy;
pub mod profiler;
pub mod report;
mod session;
mod transport;

pub use client::{TelemetryClient, TelemetryConfig};
pub use crash::{CrashReport, CrashReporter, CrashType, DocumentMetrics, ErrorBoundary, SystemInfo as CrashSystemInfo};
pub use error::{TelemetryError, TelemetryResult};
pub use event::{CommandSource, CoreEvent, TelemetryEvent};
pub use inspector::{CrdtState, DocumentInspector, InspectorNode, InspectorFilter};
pub use memory::{AllocationInfo, LeakInfo, MemoryProfiler, MemorySnapshot, SnapshotComparison};
pub use metrics::{MetricsCollector, MetricsSummary, PerformanceMetrics};
pub use privacy::{EventCategory, PrivacyManager, PrivacySettings};
pub use profiler::{PerformanceProfiler, ProfileSpan, ProfileTrace, TimelineData};
pub use report::{AppState, LogEntry, LogLevel, PerformanceSummary, ReportConfig, SupportReport, SupportReportGenerator, SystemInfo};
pub use session::{get_platform, TelemetrySession};
pub use transport::{TelemetryTransport, TransportConfig};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_full_telemetry_flow() {
        // Create client with all telemetry enabled
        let config = TelemetryConfig::new("1.0.0")
            .with_privacy(PrivacySettings::all_enabled());
        let mut client = TelemetryClient::new(config);

        // Track app start
        client.track_app_start(true);

        // Track document open
        client.track(CoreEvent::DocOpen {
            format: "docx".to_string(),
            size_kb: 256,
            page_count: 10,
        });

        // Track some commands
        client.track(CoreEvent::CommandExecute {
            command_id: "bold".to_string(),
            source: CommandSource::Keyboard,
        });

        client.track(CoreEvent::CommandExecute {
            command_id: "save".to_string(),
            source: CommandSource::Menu,
        });

        // Record performance metrics (within budget - layout + render < 16.67ms)
        client.record_metrics(PerformanceMetrics::new(5.0, 8.0, 6.0, 128.0));
        client.record_metrics(PerformanceMetrics::new(6.0, 7.0, 5.0, 130.0));

        // Check stats
        assert_eq!(client.events_tracked(), 4);
        assert_eq!(client.metrics_summary().sample_count, 2);
    }

    #[test]
    fn test_privacy_filtering_flow() {
        // Start with minimal telemetry (only crash reports)
        let config = TelemetryConfig::new("1.0.0")
            .with_privacy(PrivacySettings::minimal());
        let mut client = TelemetryClient::new(config);

        // Usage events should be filtered
        client.track(CoreEvent::AppStart { cold_start: true });
        assert_eq!(client.events_tracked(), 0);
        assert_eq!(client.events_filtered(), 1);

        // Error events should pass
        client.track(CoreEvent::Error {
            error_type: "io".to_string(),
            error_message: "File not found".to_string(),
        });
        assert_eq!(client.events_tracked(), 1);
    }

    #[test]
    fn test_custom_event_with_properties() {
        let config = TelemetryConfig::new("1.0.0")
            .with_privacy(PrivacySettings::all_enabled());
        let mut client = TelemetryClient::new(config);

        let mut props = HashMap::new();
        props.insert("action".to_string(), serde_json::json!("click"));
        props.insert("element".to_string(), serde_json::json!("button"));

        client.track_custom("ui_interaction", props);

        assert_eq!(client.events_tracked(), 1);
    }

    #[test]
    fn test_metrics_percentiles() {
        let config = TelemetryConfig::new("1.0.0");
        let mut client = TelemetryClient::new(config);

        // Record varying metrics
        for i in 1..=100 {
            client.record_metrics(PerformanceMetrics::new(
                i as f64,
                i as f64 * 2.0,
                i as f64 * 0.5,
                128.0,
            ));
        }

        let summary = client.metrics_summary();
        assert_eq!(summary.sample_count, 100);

        // Check percentiles are reasonable
        assert!(summary.min.input_latency_ms < summary.median.input_latency_ms);
        assert!(summary.median.input_latency_ms < summary.p95.input_latency_ms);
        assert!(summary.p95.input_latency_ms <= summary.max.input_latency_ms);
    }

    #[test]
    fn test_session_info_propagation() {
        let config = TelemetryConfig::new("2.5.0")
            .with_privacy(PrivacySettings::all_enabled());
        let client = TelemetryClient::new(config);

        let session = client.session();
        assert_eq!(session.app_version, "2.5.0");
        assert!(!session.session_id.is_empty());
        assert!(!session.platform.is_empty());
    }

    #[tokio::test]
    async fn test_flush_and_clear() {
        let config = TelemetryConfig::new("1.0.0")
            .with_privacy(PrivacySettings::all_enabled());
        let mut client = TelemetryClient::new(config);

        client.track_app_start(true);
        assert_eq!(client.events_queued(), 1);

        client.flush().await.unwrap();
        assert_eq!(client.events_queued(), 0);
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let event = TelemetryEvent::new("test", "session", "1.0", "macos")
            .with_property("key", "value")
            .with_measurement("time_ms", 42.0);

        let json = serde_json::to_string(&event).unwrap();
        let parsed: TelemetryEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_id, parsed.event_id);
        assert_eq!(event.event_name, parsed.event_name);
        assert_eq!(event.properties, parsed.properties);
        assert_eq!(event.measurements, parsed.measurements);
    }

    #[test]
    fn test_privacy_settings_serialization() {
        let settings = PrivacySettings {
            telemetry_enabled: true,
            crash_reports_enabled: true,
            performance_metrics_enabled: false,
            usage_analytics_enabled: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: PrivacySettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, parsed);
    }

    #[test]
    fn test_dynamic_privacy_changes() {
        let config = TelemetryConfig::new("1.0.0")
            .with_privacy(PrivacySettings::default());
        let mut client = TelemetryClient::new(config);

        // Initially disabled
        client.track_app_start(true);
        assert_eq!(client.events_tracked(), 0);

        // Enable telemetry
        client.set_privacy(PrivacySettings::all_enabled());

        // Now events should track
        client.track_app_start(false);
        assert_eq!(client.events_tracked(), 1);

        // Disable again
        client.set_privacy(PrivacySettings::default());

        client.track_app_start(false);
        assert_eq!(client.events_tracked(), 1); // Still 1
    }
}
