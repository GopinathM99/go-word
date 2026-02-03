//! Telemetry session management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// A telemetry session representing a single application run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySession {
    /// Unique identifier for this session
    pub session_id: String,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// Application version
    pub app_version: String,
    /// Platform identifier
    pub platform: String,
}

impl TelemetrySession {
    /// Create a new telemetry session.
    pub fn new(app_version: &str) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            app_version: app_version.to_string(),
            platform: detect_platform(),
        }
    }

    /// Create a session with a specific platform override.
    pub fn with_platform(app_version: &str, platform: &str) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            app_version: app_version.to_string(),
            platform: platform.to_string(),
        }
    }

    /// Get the session duration since start.
    pub fn duration(&self) -> Duration {
        let now = Utc::now();
        let diff = now.signed_duration_since(self.started_at);
        Duration::from_millis(diff.num_milliseconds().max(0) as u64)
    }

    /// Get session duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.duration().as_millis() as u64
    }

    /// Check if the session has been running for longer than the given duration.
    pub fn is_older_than(&self, duration: Duration) -> bool {
        self.duration() > duration
    }
}

/// Detect the current platform.
fn detect_platform() -> String {
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

/// Get the current platform string.
pub fn get_platform() -> String {
    detect_platform()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_session_new() {
        let session = TelemetrySession::new("1.0.0");

        assert!(!session.session_id.is_empty());
        assert_eq!(session.app_version, "1.0.0");
        assert!(!session.platform.is_empty());
    }

    #[test]
    fn test_session_with_platform() {
        let session = TelemetrySession::with_platform("1.0.0", "test-platform");

        assert_eq!(session.app_version, "1.0.0");
        assert_eq!(session.platform, "test-platform");
    }

    #[test]
    fn test_session_unique_ids() {
        let session1 = TelemetrySession::new("1.0.0");
        let session2 = TelemetrySession::new("1.0.0");

        assert_ne!(session1.session_id, session2.session_id);
    }

    #[test]
    fn test_session_duration() {
        let session = TelemetrySession::new("1.0.0");

        // Duration should be very small (just created)
        let duration = session.duration();
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_session_duration_after_wait() {
        let session = TelemetrySession::new("1.0.0");

        // Wait a bit
        sleep(Duration::from_millis(50));

        let duration = session.duration();
        assert!(duration.as_millis() >= 50);
    }

    #[test]
    fn test_session_duration_ms() {
        let session = TelemetrySession::new("1.0.0");
        sleep(Duration::from_millis(10));

        let ms = session.duration_ms();
        assert!(ms >= 10);
    }

    #[test]
    fn test_session_is_older_than() {
        let session = TelemetrySession::new("1.0.0");

        // Freshly created session should not be older than 1 hour
        assert!(!session.is_older_than(Duration::from_secs(3600)));

        // Wait briefly and check it's older than 0
        sleep(Duration::from_millis(5));
        assert!(session.is_older_than(Duration::from_secs(0)));
    }

    #[test]
    fn test_session_serialization() {
        let session = TelemetrySession::new("1.0.0");
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: TelemetrySession = serde_json::from_str(&json).unwrap();

        assert_eq!(session.session_id, deserialized.session_id);
        assert_eq!(session.app_version, deserialized.app_version);
        assert_eq!(session.platform, deserialized.platform);
    }

    #[test]
    fn test_get_platform() {
        let platform = get_platform();
        assert!(!platform.is_empty());

        #[cfg(target_os = "macos")]
        assert_eq!(platform, "macos");

        #[cfg(target_os = "windows")]
        assert_eq!(platform, "windows");

        #[cfg(target_os = "linux")]
        assert_eq!(platform, "linux");
    }

    #[test]
    fn test_detect_platform_not_empty() {
        let platform = detect_platform();
        assert!(!platform.is_empty());
    }
}
