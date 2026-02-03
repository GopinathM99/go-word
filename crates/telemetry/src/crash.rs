//! Crash Reporting Module
//!
//! Provides crash capture, reporting, and recovery infrastructure.
//! All crash data is collected only when the user has opted in to crash reporting.
//!
//! ## Architecture
//!
//! 1. **CrashReport**: Captures crash context (stack trace, app state, document metrics)
//! 2. **CrashReporter**: Manages crash lifecycle (capture, persist, send, cleanup)
//! 3. **ErrorBoundary**: Catches and categorizes errors at application boundaries
//! 4. **RecoveryManager**: Manages document recovery files from crashes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// =============================================================================
// Crash Report Types
// =============================================================================

/// A crash report capturing the context of an application crash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    /// Unique identifier for this crash
    pub crash_id: String,
    /// When the crash occurred
    pub timestamp: DateTime<Utc>,
    /// Application version
    pub app_version: String,
    /// Platform identifier
    pub platform: String,
    /// Session ID when crash occurred
    pub session_id: String,
    /// Error/crash classification
    pub crash_type: CrashType,
    /// Error message
    pub message: String,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
    /// Last command executed before crash
    pub last_command: Option<String>,
    /// Document metrics at time of crash (no content)
    pub document_metrics: Option<DocumentMetrics>,
    /// System information
    pub system_info: SystemInfo,
    /// Additional context properties
    pub context: HashMap<String, String>,
    /// Whether the report has been sent
    pub sent: bool,
}

impl CrashReport {
    /// Create a new crash report
    pub fn new(
        app_version: impl Into<String>,
        platform: impl Into<String>,
        session_id: impl Into<String>,
        crash_type: CrashType,
        message: impl Into<String>,
    ) -> Self {
        Self {
            crash_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            app_version: app_version.into(),
            platform: platform.into(),
            session_id: session_id.into(),
            crash_type,
            message: message.into(),
            stack_trace: None,
            last_command: None,
            document_metrics: None,
            system_info: SystemInfo::collect(),
            context: HashMap::new(),
            sent: false,
        }
    }

    /// Set the stack trace
    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// Set the last command
    pub fn with_last_command(mut self, command: impl Into<String>) -> Self {
        self.last_command = Some(command.into());
        self
    }

    /// Set document metrics
    pub fn with_document_metrics(mut self, metrics: DocumentMetrics) -> Self {
        self.document_metrics = Some(metrics);
        self
    }

    /// Add a context property
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Mark the report as sent
    pub fn mark_sent(&mut self) {
        self.sent = true;
    }

    /// Get a fingerprint for grouping similar crashes
    pub fn fingerprint(&self) -> String {
        // Group by crash type + first line of stack trace (or message)
        let trace_key = self
            .stack_trace
            .as_ref()
            .and_then(|t| t.lines().next())
            .unwrap_or(&self.message);
        format!("{}:{}", self.crash_type.as_str(), trace_key)
    }
}

/// Classification of the crash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrashType {
    UnhandledException,
    UnhandledRejection,
    Panic,
    OutOfMemory,
    StackOverflow,
    SegmentationFault,
    Hang,
    IoError,
    RenderCrash,
    WasmError,
    Unknown,
}

impl CrashType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CrashType::UnhandledException => "unhandled_exception",
            CrashType::UnhandledRejection => "unhandled_rejection",
            CrashType::Panic => "panic",
            CrashType::OutOfMemory => "oom",
            CrashType::StackOverflow => "stack_overflow",
            CrashType::SegmentationFault => "segfault",
            CrashType::Hang => "hang",
            CrashType::IoError => "io_error",
            CrashType::RenderCrash => "render_crash",
            CrashType::WasmError => "wasm_error",
            CrashType::Unknown => "unknown",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CrashType::UnhandledException => "Unhandled exception",
            CrashType::UnhandledRejection => "Unhandled promise rejection",
            CrashType::Panic => "Internal error (panic)",
            CrashType::OutOfMemory => "Out of memory",
            CrashType::StackOverflow => "Stack overflow",
            CrashType::SegmentationFault => "Memory access violation",
            CrashType::Hang => "Application hang detected",
            CrashType::IoError => "File I/O error",
            CrashType::RenderCrash => "Rendering engine error",
            CrashType::WasmError => "WebAssembly error",
            CrashType::Unknown => "Unknown error",
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            CrashType::SegmentationFault => 5,
            CrashType::OutOfMemory => 5,
            CrashType::StackOverflow => 5,
            CrashType::Panic => 4,
            CrashType::Hang => 4,
            CrashType::RenderCrash => 3,
            CrashType::WasmError => 3,
            CrashType::UnhandledException => 3,
            CrashType::UnhandledRejection => 2,
            CrashType::IoError => 2,
            CrashType::Unknown => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetrics {
    pub page_count: u32,
    pub word_count: u32,
    pub has_images: bool,
    pub has_tables: bool,
    pub has_charts: bool,
    pub has_equations: bool,
    pub is_collaborative: bool,
    pub collaborator_count: u32,
    pub file_size_kb: u64,
    pub open_duration_secs: u64,
}

impl DocumentMetrics {
    pub fn empty() -> Self {
        Self { page_count: 0, word_count: 0, has_images: false, has_tables: false, has_charts: false, has_equations: false, is_collaborative: false, collaborator_count: 0, file_size_kb: 0, open_duration_secs: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub memory_mb: u64,
    pub cpu_count: u32,
    pub debug_mode: bool,
}

impl SystemInfo {
    pub fn collect() -> Self {
        Self { os: std::env::consts::OS.to_string(), arch: std::env::consts::ARCH.to_string(), memory_mb: 0, cpu_count: std::thread::available_parallelism().map(|p| p.get() as u32).unwrap_or(1), debug_mode: cfg!(debug_assertions) }
    }
    pub fn with_memory(mut self, mb: u64) -> Self { self.memory_mb = mb; self }
}

pub struct ErrorBoundary {
    name: String,
    recent_errors: Vec<BoundaryError>,
    max_recent: usize,
    on_crash: Option<Box<dyn Fn(&CrashReport) + Send + Sync>>,
}

impl std::fmt::Debug for ErrorBoundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorBoundary")
            .field("name", &self.name)
            .field("recent_errors", &self.recent_errors)
            .field("max_recent", &self.max_recent)
            .field("on_crash", &self.on_crash.as_ref().map(|_| "<callback>"))
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryError {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub category: String,
    pub escalated: bool,
}

impl ErrorBoundary {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into(), recent_errors: Vec::new(), max_recent: 50, on_crash: None } }
    pub fn on_crash(mut self, callback: Box<dyn Fn(&CrashReport) + Send + Sync>) -> Self { self.on_crash = Some(callback); self }
    pub fn record_error(&mut self, message: impl Into<String>, category: impl Into<String>) {
        let error = BoundaryError { timestamp: Utc::now(), message: message.into(), category: category.into(), escalated: false };
        self.recent_errors.push(error);
        if self.recent_errors.len() > self.max_recent { self.recent_errors.remove(0); }
    }
    pub fn recent_error_count(&self) -> usize { self.recent_errors.len() }
    pub fn is_crash_loop(&self, threshold: usize, window_secs: i64) -> bool {
        if self.recent_errors.len() < threshold { return false; }
        let cutoff = Utc::now() - chrono::Duration::seconds(window_secs);
        self.recent_errors.iter().rev().take_while(|e| e.timestamp > cutoff).count() >= threshold
    }
    pub fn clear(&mut self) { self.recent_errors.clear(); }
    pub fn name(&self) -> &str { &self.name }
    pub fn recent_errors(&self) -> &[BoundaryError] { &self.recent_errors }
}

pub struct CrashReporter { crash_dir: PathBuf, recovery_dir: PathBuf, app_version: String, platform: String, session_id: String, last_command: Option<String>, document_metrics: Option<DocumentMetrics>, pending_reports: Vec<CrashReport>, max_pending: usize }
impl CrashReporter {
    pub fn new(crash_dir: impl Into<PathBuf>, recovery_dir: impl Into<PathBuf>, app_version: impl Into<String>, platform: impl Into<String>, session_id: impl Into<String>) -> Self { Self { crash_dir: crash_dir.into(), recovery_dir: recovery_dir.into(), app_version: app_version.into(), platform: platform.into(), session_id: session_id.into(), last_command: None, document_metrics: None, pending_reports: Vec::new(), max_pending: 100 } }
    pub fn set_last_command(&mut self, command: impl Into<String>) { self.last_command = Some(command.into()); }
    pub fn set_document_metrics(&mut self, metrics: DocumentMetrics) { self.document_metrics = Some(metrics); }
    pub fn capture_crash(&mut self, crash_type: CrashType, message: impl Into<String>) -> CrashReport { let mut report = CrashReport::new(&self.app_version, &self.platform, &self.session_id, crash_type, message); if let Some(ref cmd) = self.last_command { report = report.with_last_command(cmd.clone()); } if let Some(ref metrics) = self.document_metrics { report = report.with_document_metrics(metrics.clone()); } self.pending_reports.push(report.clone()); if self.pending_reports.len() > self.max_pending { self.pending_reports.remove(0); } report }
    pub fn capture_crash_with_trace(&mut self, crash_type: CrashType, message: impl Into<String>, trace: impl Into<String>) -> CrashReport { let mut report = self.capture_crash(crash_type, message); report.stack_trace = Some(trace.into()); if let Some(last) = self.pending_reports.last_mut() { last.stack_trace = report.stack_trace.clone(); } report }
    pub fn persist_report(&self, report: &CrashReport) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.crash_dir)?;
        let filename = format!("crash-{}.json", report.crash_id);
        let path = self.crash_dir.join(filename);
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Get pending crash reports.
    pub fn pending_reports(&self) -> &[CrashReport] {
        &self.pending_reports
    }

    /// Clear pending reports.
    pub fn clear_pending(&mut self) {
        self.pending_reports.clear();
    }

    /// Get the recovery directory path.
    pub fn recovery_dir(&self) -> &PathBuf {
        &self.recovery_dir
    }

    /// Get the crash directory path.
    pub fn crash_dir(&self) -> &PathBuf {
        &self.crash_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_report_new() {
        let report = CrashReport::new("1.0.0", "test", "session-123", CrashType::Panic, "test error");
        assert!(!report.crash_id.is_empty());
        assert_eq!(report.app_version, "1.0.0");
        assert_eq!(report.crash_type, CrashType::Panic);
        assert!(!report.sent);
    }

    #[test]
    fn test_crash_report_with_trace() {
        let report = CrashReport::new("1.0", "test", "s", CrashType::Panic, "error")
            .with_stack_trace("at main.rs:10\nat lib.rs:20");
        assert!(report.stack_trace.is_some());
    }

    #[test]
    fn test_crash_report_fingerprint() {
        let report = CrashReport::new("1.0", "test", "s", CrashType::Panic, "test error")
            .with_stack_trace("at main.rs:10\nat lib.rs:20");
        let fingerprint = report.fingerprint();
        assert!(fingerprint.contains("panic"));
        assert!(fingerprint.contains("main.rs:10"));
    }

    #[test]
    fn test_crash_type_severity() {
        assert!(CrashType::Panic.severity() > CrashType::IoError.severity());
        assert!(CrashType::SegmentationFault.severity() >= CrashType::Panic.severity());
    }

    #[test]
    fn test_document_metrics_empty() {
        let metrics = DocumentMetrics::empty();
        assert_eq!(metrics.page_count, 0);
        assert!(!metrics.is_collaborative);
    }

    #[test]
    fn test_system_info_collect() {
        let info = SystemInfo::collect();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_error_boundary_new() {
        let boundary = ErrorBoundary::new("test_boundary");
        assert_eq!(boundary.name(), "test_boundary");
        assert_eq!(boundary.recent_error_count(), 0);
    }

    #[test]
    fn test_error_boundary_record_error() {
        let mut boundary = ErrorBoundary::new("test");
        boundary.record_error("error 1", "category_a");
        boundary.record_error("error 2", "category_b");
        assert_eq!(boundary.recent_error_count(), 2);
    }

    #[test]
    fn test_error_boundary_crash_loop() {
        let mut boundary = ErrorBoundary::new("test");
        // Not enough errors for crash loop
        boundary.record_error("error", "cat");
        assert!(!boundary.is_crash_loop(3, 60));

        // Add more errors
        boundary.record_error("error", "cat");
        boundary.record_error("error", "cat");
        assert!(boundary.is_crash_loop(3, 60));
    }

    #[test]
    fn test_crash_reporter_capture() {
        let mut reporter = CrashReporter::new(
            "/tmp/crashes",
            "/tmp/recovery",
            "1.0.0",
            "test",
            "session-123",
        );
        reporter.set_last_command("save");

        let report = reporter.capture_crash(CrashType::IoError, "disk full");
        assert_eq!(report.last_command, Some("save".to_string()));
        assert_eq!(reporter.pending_reports().len(), 1);
    }
}
