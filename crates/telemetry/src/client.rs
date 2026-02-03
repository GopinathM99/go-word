//! High-level telemetry client for easy integration.

use serde_json::Value;
use std::collections::HashMap;

use crate::error::TelemetryResult;
use crate::event::{CoreEvent, TelemetryEvent};
use crate::metrics::{MetricsCollector, MetricsSummary, PerformanceMetrics};
use crate::privacy::{PrivacyManager, PrivacySettings};
use crate::session::TelemetrySession;
use crate::transport::{TelemetryTransport, TransportConfig};

/// Configuration for the telemetry client.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Application version
    pub app_version: String,
    /// Telemetry endpoint URL
    pub endpoint: String,
    /// Initial privacy settings
    pub privacy: PrivacySettings,
    /// Maximum metrics samples to retain
    pub max_metrics_samples: usize,
    /// Transport configuration
    pub transport: TransportConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            app_version: "0.0.0".to_string(),
            endpoint: String::new(),
            privacy: PrivacySettings::default(),
            max_metrics_samples: 1000,
            transport: TransportConfig::default(),
        }
    }
}

impl TelemetryConfig {
    /// Create a new telemetry config with the specified version.
    pub fn new(app_version: &str) -> Self {
        Self {
            app_version: app_version.to_string(),
            ..Default::default()
        }
    }

    /// Set the telemetry endpoint.
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self.transport.endpoint = endpoint.to_string();
        self
    }

    /// Set initial privacy settings.
    pub fn with_privacy(mut self, privacy: PrivacySettings) -> Self {
        self.privacy = privacy;
        self
    }

    /// Set maximum metrics samples.
    pub fn with_max_metrics_samples(mut self, count: usize) -> Self {
        self.max_metrics_samples = count;
        self
    }
}

/// High-level telemetry client integrating all telemetry components.
#[derive(Debug)]
pub struct TelemetryClient {
    session: TelemetrySession,
    transport: TelemetryTransport,
    privacy: PrivacyManager,
    metrics: MetricsCollector,
    events_tracked: u64,
    events_filtered: u64,
}

impl TelemetryClient {
    /// Create a new telemetry client with the given configuration.
    pub fn new(config: TelemetryConfig) -> Self {
        let session = TelemetrySession::new(&config.app_version);
        let transport = TelemetryTransport::with_config(config.transport);
        let privacy = PrivacyManager::new(config.privacy);
        let metrics = MetricsCollector::new(config.max_metrics_samples);

        Self {
            session,
            transport,
            privacy,
            metrics,
            events_tracked: 0,
            events_filtered: 0,
        }
    }

    /// Track a core event.
    pub fn track(&mut self, event: CoreEvent) {
        let telemetry_event = event.to_event(
            &self.session.session_id,
            &self.session.app_version,
            &self.session.platform,
        );

        self.track_event(telemetry_event);
    }

    /// Track a custom event with the given name and properties.
    pub fn track_custom(&mut self, name: &str, properties: HashMap<String, Value>) {
        let mut event = TelemetryEvent::new(
            name,
            &self.session.session_id,
            &self.session.app_version,
            &self.session.platform,
        );
        event.properties = properties;

        self.track_event(event);
    }

    /// Track a custom event with both properties and measurements.
    pub fn track_custom_with_measurements(
        &mut self,
        name: &str,
        properties: HashMap<String, Value>,
        measurements: HashMap<String, f64>,
    ) {
        let mut event = TelemetryEvent::new(
            name,
            &self.session.session_id,
            &self.session.app_version,
            &self.session.platform,
        );
        event.properties = properties;
        event.measurements = measurements;

        self.track_event(event);
    }

    /// Internal method to process and queue an event.
    fn track_event(&mut self, event: TelemetryEvent) {
        // Check privacy settings
        if !self.privacy.is_allowed(&event) {
            self.events_filtered += 1;
            return;
        }

        // Scrub sensitive data
        let scrubbed = self.privacy.scrub_event(event);

        // Queue for sending
        if self.transport.queue(scrubbed).is_ok() {
            self.events_tracked += 1;
        }
    }

    /// Record performance metrics.
    pub fn record_metrics(&mut self, metrics: PerformanceMetrics) {
        self.metrics.record(metrics.clone());

        // Optionally track as an event if within budget constraints are violated
        if !metrics.is_within_budget() {
            self.track(CoreEvent::Performance {
                metric_name: "budget_violation".to_string(),
                value_ms: metrics.total_frame_time_ms(),
            });
        }
    }

    /// Flush all queued telemetry events.
    pub async fn flush(&mut self) -> TelemetryResult<()> {
        self.transport.flush().await
    }

    /// Update privacy settings.
    pub fn set_privacy(&mut self, settings: PrivacySettings) {
        self.privacy.set_settings(settings);
    }

    /// Get current privacy settings.
    pub fn get_privacy(&self) -> &PrivacySettings {
        self.privacy.get_settings()
    }

    /// Get the current session.
    pub fn session(&self) -> &TelemetrySession {
        &self.session
    }

    /// Get performance metrics summary.
    pub fn metrics_summary(&self) -> MetricsSummary {
        self.metrics.summary()
    }

    /// Get the latest performance metrics.
    pub fn latest_metrics(&self) -> Option<&PerformanceMetrics> {
        self.metrics.get_latest()
    }

    /// Get total number of events tracked.
    pub fn events_tracked(&self) -> u64 {
        self.events_tracked
    }

    /// Get total number of events filtered by privacy.
    pub fn events_filtered(&self) -> u64 {
        self.events_filtered
    }

    /// Get number of events waiting to be sent.
    pub fn events_queued(&self) -> usize {
        self.transport.queued_count()
    }

    /// Check if transport should flush (batch is full).
    pub fn should_flush(&self) -> bool {
        self.transport.should_flush()
    }

    /// Set offline mode.
    pub fn set_offline(&mut self, offline: bool) {
        self.transport.set_offline(offline);
    }

    /// Check if client is in offline mode.
    pub fn is_offline(&self) -> bool {
        self.transport.is_offline()
    }

    /// Track application start event.
    pub fn track_app_start(&mut self, cold_start: bool) {
        self.track(CoreEvent::AppStart { cold_start });
    }

    /// Track application exit event.
    pub fn track_app_exit(&mut self) {
        self.track(CoreEvent::AppExit {
            session_duration_ms: self.session.duration_ms(),
        });
    }

    /// Clear all metrics.
    pub fn clear_metrics(&mut self) {
        self.metrics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::CommandSource;

    fn make_client() -> TelemetryClient {
        TelemetryClient::new(
            TelemetryConfig::new("1.0.0").with_privacy(PrivacySettings::all_enabled()),
        )
    }

    fn make_disabled_client() -> TelemetryClient {
        TelemetryClient::new(TelemetryConfig::new("1.0.0"))
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert_eq!(config.app_version, "0.0.0");
        assert!(config.endpoint.is_empty());
        assert!(!config.privacy.telemetry_enabled);
    }

    #[test]
    fn test_telemetry_config_builder() {
        let config = TelemetryConfig::new("2.0.0")
            .with_endpoint("https://example.com")
            .with_privacy(PrivacySettings::all_enabled())
            .with_max_metrics_samples(500);

        assert_eq!(config.app_version, "2.0.0");
        assert_eq!(config.endpoint, "https://example.com");
        assert!(config.privacy.telemetry_enabled);
        assert_eq!(config.max_metrics_samples, 500);
    }

    #[test]
    fn test_client_new() {
        let client = make_client();
        assert_eq!(client.session().app_version, "1.0.0");
        assert_eq!(client.events_tracked(), 0);
        assert_eq!(client.events_filtered(), 0);
    }

    #[test]
    fn test_client_track_core_event() {
        let mut client = make_client();
        client.track(CoreEvent::AppStart { cold_start: true });

        assert_eq!(client.events_tracked(), 1);
        assert_eq!(client.events_queued(), 1);
    }

    #[test]
    fn test_client_track_filtered_by_privacy() {
        let mut client = make_disabled_client();
        client.track(CoreEvent::AppStart { cold_start: true });

        assert_eq!(client.events_tracked(), 0);
        assert_eq!(client.events_filtered(), 1);
        assert_eq!(client.events_queued(), 0);
    }

    #[test]
    fn test_client_track_custom() {
        let mut client = make_client();
        let mut props = HashMap::new();
        props.insert("key".to_string(), Value::String("value".to_string()));

        client.track_custom("custom_event", props);

        assert_eq!(client.events_tracked(), 1);
    }

    #[test]
    fn test_client_track_custom_with_measurements() {
        let mut client = make_client();
        let props = HashMap::new();
        let mut measurements = HashMap::new();
        measurements.insert("time_ms".to_string(), 42.0);

        client.track_custom_with_measurements("perf_event", props, measurements);

        assert_eq!(client.events_tracked(), 1);
    }

    #[test]
    fn test_client_record_metrics() {
        let mut client = make_client();
        let metrics = PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0);

        client.record_metrics(metrics);

        let summary = client.metrics_summary();
        assert_eq!(summary.sample_count, 1);
    }

    #[test]
    fn test_client_record_metrics_budget_violation() {
        let mut client = make_client();
        // Metrics that violate the budget
        let metrics = PerformanceMetrics::new(150.0, 20.0, 20.0, 128.0);

        client.record_metrics(metrics);

        // Should have tracked a performance violation event
        assert!(client.events_tracked() >= 1);
    }

    #[test]
    fn test_client_set_privacy() {
        let mut client = make_disabled_client();
        assert!(!client.get_privacy().telemetry_enabled);

        client.set_privacy(PrivacySettings::all_enabled());
        assert!(client.get_privacy().telemetry_enabled);
    }

    #[test]
    fn test_client_session() {
        let client = make_client();
        let session = client.session();

        assert_eq!(session.app_version, "1.0.0");
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_client_latest_metrics() {
        let mut client = make_client();
        assert!(client.latest_metrics().is_none());

        let metrics = PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0);
        client.record_metrics(metrics);

        let latest = client.latest_metrics().unwrap();
        assert_eq!(latest.input_latency_ms, 5.0);
    }

    #[test]
    fn test_client_offline_mode() {
        let mut client = make_client();
        assert!(!client.is_offline());

        client.set_offline(true);
        assert!(client.is_offline());
    }

    #[test]
    fn test_client_track_app_start() {
        let mut client = make_client();
        client.track_app_start(true);
        assert_eq!(client.events_tracked(), 1);
    }

    #[test]
    fn test_client_track_app_exit() {
        let mut client = make_client();
        client.track_app_exit();
        assert_eq!(client.events_tracked(), 1);
    }

    #[test]
    fn test_client_clear_metrics() {
        let mut client = make_client();
        client.record_metrics(PerformanceMetrics::default());
        assert_eq!(client.metrics_summary().sample_count, 1);

        client.clear_metrics();
        assert_eq!(client.metrics_summary().sample_count, 0);
    }

    #[tokio::test]
    async fn test_client_flush() {
        let mut client = make_client();
        client.track(CoreEvent::AppStart { cold_start: true });

        let result = client.flush().await;
        assert!(result.is_ok());
        assert_eq!(client.events_queued(), 0);
    }

    #[tokio::test]
    async fn test_client_flush_offline() {
        let mut client = make_client();
        client.track(CoreEvent::AppStart { cold_start: true });
        client.set_offline(true);

        let result = client.flush().await;
        assert!(result.is_err());
        // Events still queued
        assert_eq!(client.events_queued(), 1);
    }

    #[test]
    fn test_client_track_multiple_events() {
        let mut client = make_client();

        client.track(CoreEvent::AppStart { cold_start: true });
        client.track(CoreEvent::DocOpen {
            format: "docx".to_string(),
            size_kb: 100,
            page_count: 5,
        });
        client.track(CoreEvent::CommandExecute {
            command_id: "bold".to_string(),
            source: CommandSource::Keyboard,
        });

        assert_eq!(client.events_tracked(), 3);
        assert_eq!(client.events_queued(), 3);
    }

    #[test]
    fn test_client_should_flush() {
        let config = TelemetryConfig::new("1.0.0")
            .with_privacy(PrivacySettings::all_enabled());
        let mut config_with_small_batch = config.clone();
        config_with_small_batch.transport.batch_size = 2;

        let mut client = TelemetryClient::new(config_with_small_batch);

        assert!(!client.should_flush());

        client.track(CoreEvent::AppStart { cold_start: true });
        assert!(!client.should_flush());

        client.track(CoreEvent::AppStart { cold_start: false });
        assert!(client.should_flush());
    }
}
