//! Error types for the telemetry system.

use thiserror::Error;

/// Errors that can occur in the telemetry system.
#[derive(Debug, Error)]
pub enum TelemetryError {
    /// Failed to serialize telemetry event
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Failed to send telemetry data
    #[error("Transport error: {0}")]
    Transport(String),

    /// Telemetry is disabled by privacy settings
    #[error("Telemetry is disabled by user privacy settings")]
    TelemetryDisabled,

    /// Invalid event data
    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Batch queue is full
    #[error("Event queue is full")]
    QueueFull,

    /// Transport is in offline mode
    #[error("Transport is offline")]
    Offline,
}

/// Result type for telemetry operations.
pub type TelemetryResult<T> = Result<T, TelemetryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TelemetryError::TelemetryDisabled;
        assert_eq!(
            err.to_string(),
            "Telemetry is disabled by user privacy settings"
        );

        let err = TelemetryError::Transport("connection failed".to_string());
        assert_eq!(err.to_string(), "Transport error: connection failed");
    }

    #[test]
    fn test_serialization_error_conversion() {
        let json_err: Result<(), serde_json::Error> =
            serde_json::from_str::<()>("invalid json");
        let telemetry_err: TelemetryError = json_err.unwrap_err().into();
        assert!(matches!(telemetry_err, TelemetryError::Serialization(_)));
    }
}
