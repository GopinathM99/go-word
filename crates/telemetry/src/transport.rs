//! Transport layer for sending telemetry data.

use std::time::Duration;

use crate::error::{TelemetryError, TelemetryResult};
use crate::event::TelemetryEvent;

/// Configuration for telemetry transport.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Endpoint URL for sending telemetry
    pub endpoint: String,
    /// Maximum events to batch before sending
    pub batch_size: usize,
    /// How often to flush batched events
    pub flush_interval: Duration,
    /// Maximum queue size before dropping events
    pub max_queue_size: usize,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            batch_size: 100,
            flush_interval: Duration::from_secs(60),
            max_queue_size: 10000,
            timeout: Duration::from_secs(30),
        }
    }
}

impl TransportConfig {
    /// Create a new transport config with the specified endpoint.
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            ..Default::default()
        }
    }

    /// Set the batch size.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the flush interval.
    pub fn with_flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    /// Set the maximum queue size.
    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Transport layer for sending telemetry events to a remote endpoint.
#[derive(Debug)]
pub struct TelemetryTransport {
    config: TransportConfig,
    batch: Vec<TelemetryEvent>,
    offline: bool,
    failed_send_count: u32,
}

impl TelemetryTransport {
    /// Create a new transport with the given endpoint.
    pub fn new(endpoint: &str) -> Self {
        Self::with_config(TransportConfig::new(endpoint))
    }

    /// Create a new transport with full configuration.
    pub fn with_config(config: TransportConfig) -> Self {
        Self {
            config,
            batch: Vec::new(),
            offline: false,
            failed_send_count: 0,
        }
    }

    /// Queue an event for sending.
    pub fn queue(&mut self, event: TelemetryEvent) -> TelemetryResult<()> {
        if self.batch.len() >= self.config.max_queue_size {
            return Err(TelemetryError::QueueFull);
        }
        self.batch.push(event);
        Ok(())
    }

    /// Check if the batch is ready to be flushed.
    pub fn should_flush(&self) -> bool {
        self.batch.len() >= self.config.batch_size
    }

    /// Get the number of queued events.
    pub fn queued_count(&self) -> usize {
        self.batch.len()
    }

    /// Check if there are any queued events.
    pub fn has_queued(&self) -> bool {
        !self.batch.is_empty()
    }

    /// Flush all queued events.
    ///
    /// In a real implementation, this would send events to the endpoint.
    /// For now, this simulates the send and clears the batch.
    pub async fn flush(&mut self) -> TelemetryResult<()> {
        if self.offline {
            return Err(TelemetryError::Offline);
        }

        if self.batch.is_empty() {
            return Ok(());
        }

        // In a real implementation, we would serialize and send the events
        // For now, we just simulate success
        let result = self.send_batch().await;

        match result {
            Ok(()) => {
                self.failed_send_count = 0;
                self.batch.clear();
                Ok(())
            }
            Err(e) => {
                self.failed_send_count += 1;
                Err(e)
            }
        }
    }

    /// Internal method to send the batch.
    async fn send_batch(&self) -> TelemetryResult<()> {
        if self.config.endpoint.is_empty() {
            // No endpoint configured - silently succeed (useful for testing/dev)
            return Ok(());
        }

        // Serialize the batch
        let _payload = serde_json::to_string(&self.batch)?;

        // In a real implementation, this would use an HTTP client to POST
        // to the endpoint. For now, we just simulate success.
        //
        // Example with reqwest (not included in dependencies):
        // let client = reqwest::Client::new();
        // client.post(&self.config.endpoint)
        //     .timeout(self.config.timeout)
        //     .header("Content-Type", "application/json")
        //     .body(payload)
        //     .send()
        //     .await
        //     .map_err(|e| TelemetryError::Network(e.to_string()))?;

        Ok(())
    }

    /// Set offline mode.
    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    /// Check if transport is in offline mode.
    pub fn is_offline(&self) -> bool {
        self.offline
    }

    /// Get the number of consecutive failed sends.
    pub fn failed_send_count(&self) -> u32 {
        self.failed_send_count
    }

    /// Get a reference to the currently queued events.
    pub fn queued_events(&self) -> &[TelemetryEvent] {
        &self.batch
    }

    /// Get the transport configuration.
    pub fn config(&self) -> &TransportConfig {
        &self.config
    }

    /// Clear all queued events without sending.
    pub fn clear(&mut self) {
        self.batch.clear();
    }

    /// Take ownership of queued events (for persistence/retry).
    pub fn take_queued(&mut self) -> Vec<TelemetryEvent> {
        std::mem::take(&mut self.batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(name: &str) -> TelemetryEvent {
        TelemetryEvent::new(name, "session", "1.0", "test")
    }

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert!(config.endpoint.is_empty());
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.flush_interval, Duration::from_secs(60));
        assert_eq!(config.max_queue_size, 10000);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_transport_config_new() {
        let config = TransportConfig::new("https://telemetry.example.com");
        assert_eq!(config.endpoint, "https://telemetry.example.com");
    }

    #[test]
    fn test_transport_config_builder() {
        let config = TransportConfig::new("https://example.com")
            .with_batch_size(50)
            .with_flush_interval(Duration::from_secs(30))
            .with_max_queue_size(5000)
            .with_timeout(Duration::from_secs(10));

        assert_eq!(config.batch_size, 50);
        assert_eq!(config.flush_interval, Duration::from_secs(30));
        assert_eq!(config.max_queue_size, 5000);
        assert_eq!(config.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_transport_new() {
        let transport = TelemetryTransport::new("https://example.com");
        assert_eq!(transport.config().endpoint, "https://example.com");
        assert!(!transport.is_offline());
        assert_eq!(transport.queued_count(), 0);
    }

    #[test]
    fn test_transport_queue() {
        let mut transport = TelemetryTransport::new("");
        assert!(!transport.has_queued());

        transport.queue(make_event("test")).unwrap();
        assert!(transport.has_queued());
        assert_eq!(transport.queued_count(), 1);
    }

    #[test]
    fn test_transport_queue_full() {
        let config = TransportConfig::new("").with_max_queue_size(2);
        let mut transport = TelemetryTransport::with_config(config);

        transport.queue(make_event("test1")).unwrap();
        transport.queue(make_event("test2")).unwrap();

        let result = transport.queue(make_event("test3"));
        assert!(matches!(result, Err(TelemetryError::QueueFull)));
    }

    #[test]
    fn test_transport_should_flush() {
        let config = TransportConfig::new("").with_batch_size(2);
        let mut transport = TelemetryTransport::with_config(config);

        assert!(!transport.should_flush());

        transport.queue(make_event("test1")).unwrap();
        assert!(!transport.should_flush());

        transport.queue(make_event("test2")).unwrap();
        assert!(transport.should_flush());
    }

    #[tokio::test]
    async fn test_transport_flush_empty() {
        let mut transport = TelemetryTransport::new("");
        let result = transport.flush().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transport_flush_success() {
        let mut transport = TelemetryTransport::new("");
        transport.queue(make_event("test")).unwrap();

        let result = transport.flush().await;
        assert!(result.is_ok());
        assert_eq!(transport.queued_count(), 0);
    }

    #[tokio::test]
    async fn test_transport_flush_offline() {
        let mut transport = TelemetryTransport::new("");
        transport.queue(make_event("test")).unwrap();
        transport.set_offline(true);

        let result = transport.flush().await;
        assert!(matches!(result, Err(TelemetryError::Offline)));
        // Events should still be in queue
        assert_eq!(transport.queued_count(), 1);
    }

    #[test]
    fn test_transport_set_offline() {
        let mut transport = TelemetryTransport::new("");
        assert!(!transport.is_offline());

        transport.set_offline(true);
        assert!(transport.is_offline());

        transport.set_offline(false);
        assert!(!transport.is_offline());
    }

    #[test]
    fn test_transport_queued_events() {
        let mut transport = TelemetryTransport::new("");
        transport.queue(make_event("test1")).unwrap();
        transport.queue(make_event("test2")).unwrap();

        let events = transport.queued_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_name, "test1");
        assert_eq!(events[1].event_name, "test2");
    }

    #[test]
    fn test_transport_clear() {
        let mut transport = TelemetryTransport::new("");
        transport.queue(make_event("test")).unwrap();
        assert!(transport.has_queued());

        transport.clear();
        assert!(!transport.has_queued());
    }

    #[test]
    fn test_transport_take_queued() {
        let mut transport = TelemetryTransport::new("");
        transport.queue(make_event("test1")).unwrap();
        transport.queue(make_event("test2")).unwrap();

        let events = transport.take_queued();
        assert_eq!(events.len(), 2);
        assert!(!transport.has_queued());
    }

    #[tokio::test]
    async fn test_transport_failed_send_count() {
        let mut transport = TelemetryTransport::new("");
        assert_eq!(transport.failed_send_count(), 0);

        // Successful flush resets counter
        transport.queue(make_event("test")).unwrap();
        transport.flush().await.unwrap();
        assert_eq!(transport.failed_send_count(), 0);
    }

    #[test]
    fn test_transport_config_access() {
        let transport = TelemetryTransport::new("https://example.com");
        let config = transport.config();
        assert_eq!(config.endpoint, "https://example.com");
    }
}
