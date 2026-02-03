//! Support Report Generator Module
//!
//! Generates comprehensive support reports for diagnostics and troubleshooting.
//! Collects system information, app state, logs, crash reports, and performance data.
//! Includes privacy-aware anonymization of sensitive information.
//!
//! # Example
//!
//! ```rust
//! use telemetry::report::{SupportReportGenerator, ReportConfig};
//!
//! let generator = SupportReportGenerator::new(ReportConfig::default());
//!
//! // Generate a full support report
//! let report = generator.generate_report();
//!
//! // Anonymize before sharing
//! let anonymized = generator.anonymize(report);
//!
//! // Export to file
//! generator.export_to_file(&anonymized, "/tmp/support_report.json").unwrap();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::crash::{CrashReport, CrashType, SystemInfo as CrashSystemInfo};
use crate::metrics::MetricsSummary;

// =============================================================================
// System Information
// =============================================================================

/// Detailed system information for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system name
    pub os_name: String,
    /// Operating system version
    pub os_version: String,
    /// CPU architecture
    pub architecture: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Total system memory in MB
    pub total_memory_mb: u64,
    /// Available memory in MB
    pub available_memory_mb: u64,
    /// Disk space available in MB
    pub disk_available_mb: u64,
    /// Whether running in debug mode
    pub debug_mode: bool,
    /// Display information
    pub display_info: Option<DisplayInfo>,
    /// Locale/language
    pub locale: String,
    /// Timezone
    pub timezone: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::collect()
    }
}

impl SystemInfo {
    /// Collect current system information.
    pub fn collect() -> Self {
        Self {
            os_name: std::env::consts::OS.to_string(),
            os_version: get_os_version(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_cores: std::thread::available_parallelism()
                .map(|p| p.get() as u32)
                .unwrap_or(1),
            total_memory_mb: 0, // Would require platform-specific APIs
            available_memory_mb: 0,
            disk_available_mb: 0,
            debug_mode: cfg!(debug_assertions),
            display_info: None,
            locale: std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string()),
            timezone: std::env::var("TZ").unwrap_or_else(|_| "UTC".to_string()),
        }
    }

    /// Create from crash system info.
    pub fn from_crash_info(info: &CrashSystemInfo) -> Self {
        Self {
            os_name: info.os.clone(),
            os_version: String::new(),
            architecture: info.arch.clone(),
            cpu_cores: info.cpu_count,
            total_memory_mb: info.memory_mb,
            available_memory_mb: 0,
            disk_available_mb: 0,
            debug_mode: info.debug_mode,
            display_info: None,
            locale: String::new(),
            timezone: String::new(),
        }
    }

    /// Set memory information.
    pub fn with_memory(mut self, total_mb: u64, available_mb: u64) -> Self {
        self.total_memory_mb = total_mb;
        self.available_memory_mb = available_mb;
        self
    }

    /// Set display information.
    pub fn with_display(mut self, info: DisplayInfo) -> Self {
        self.display_info = Some(info);
        self
    }
}

fn get_os_version() -> String {
    // Platform-specific version detection would go here
    "unknown".to_string()
}

/// Display/screen information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Display scale factor
    pub scale_factor: f64,
    /// Number of displays
    pub display_count: u32,
}

impl DisplayInfo {
    /// Create new display info.
    pub fn new(width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
            display_count: 1,
        }
    }
}

// =============================================================================
// Application State
// =============================================================================

/// Current application state for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Application version
    pub app_version: String,
    /// Build number
    pub build_number: String,
    /// Session ID
    pub session_id: String,
    /// Session duration in seconds
    pub session_duration_secs: u64,
    /// Number of documents currently open
    pub open_documents: u32,
    /// Current document information
    pub current_document: Option<DocumentState>,
    /// Active features/modes
    pub active_features: Vec<String>,
    /// Enabled plugins/extensions
    pub enabled_plugins: Vec<String>,
    /// Recent commands executed
    pub recent_commands: Vec<String>,
    /// Current memory usage in MB
    pub memory_usage_mb: f64,
    /// Whether there are unsaved changes
    pub has_unsaved_changes: bool,
    /// Whether collaborative editing is active
    pub is_collaborative: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            build_number: String::new(),
            session_id: String::new(),
            session_duration_secs: 0,
            open_documents: 0,
            current_document: None,
            active_features: Vec::new(),
            enabled_plugins: Vec::new(),
            recent_commands: Vec::new(),
            memory_usage_mb: 0.0,
            has_unsaved_changes: false,
            is_collaborative: false,
        }
    }
}

impl AppState {
    /// Create a new app state.
    pub fn new(app_version: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            app_version: app_version.into(),
            session_id: session_id.into(),
            ..Default::default()
        }
    }

    /// Set document state.
    pub fn with_document(mut self, doc: DocumentState) -> Self {
        self.current_document = Some(doc);
        self.open_documents = 1;
        self
    }

    /// Add an active feature.
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.active_features.push(feature.into());
        self
    }

    /// Add a recent command.
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.recent_commands.push(command.into());
        if self.recent_commands.len() > 20 {
            self.recent_commands.remove(0);
        }
        self
    }
}

/// State of a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    /// Document format
    pub format: String,
    /// File size in KB
    pub size_kb: u64,
    /// Number of pages
    pub page_count: u32,
    /// Number of words
    pub word_count: u32,
    /// Whether document has been modified
    pub is_modified: bool,
    /// Open duration in seconds
    pub open_duration_secs: u64,
    /// Content features present
    pub features: DocumentFeatures,
}

impl DocumentState {
    /// Create new document state.
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            size_kb: 0,
            page_count: 0,
            word_count: 0,
            is_modified: false,
            open_duration_secs: 0,
            features: DocumentFeatures::default(),
        }
    }

    /// Set document metrics.
    pub fn with_metrics(mut self, size_kb: u64, pages: u32, words: u32) -> Self {
        self.size_kb = size_kb;
        self.page_count = pages;
        self.word_count = words;
        self
    }
}

/// Features present in a document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentFeatures {
    pub has_images: bool,
    pub has_tables: bool,
    pub has_charts: bool,
    pub has_equations: bool,
    pub has_comments: bool,
    pub has_tracked_changes: bool,
    pub has_hyperlinks: bool,
    pub has_bookmarks: bool,
}

// =============================================================================
// Log Entry
// =============================================================================

/// A log entry for the support report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Source/component that generated the log
    pub source: String,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: LogLevel, message: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message: message.into(),
            source: source.into(),
            context: HashMap::new(),
        }
    }

    /// Add context to the log entry.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Get numeric severity (higher = more severe).
    pub fn severity(&self) -> u8 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        }
    }
}

// =============================================================================
// Performance Summary
// =============================================================================

/// Performance summary for the support report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Number of performance samples
    pub sample_count: usize,
    /// Average input latency in ms
    pub avg_input_latency_ms: f64,
    /// Average layout time in ms
    pub avg_layout_time_ms: f64,
    /// Average render time in ms
    pub avg_render_time_ms: f64,
    /// Average memory usage in MB
    pub avg_memory_mb: f64,
    /// P95 frame time in ms
    pub p95_frame_time_ms: f64,
    /// Number of frame budget violations
    pub budget_violations: usize,
    /// Percentage of time within budget
    pub within_budget_percent: f64,
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            sample_count: 0,
            avg_input_latency_ms: 0.0,
            avg_layout_time_ms: 0.0,
            avg_render_time_ms: 0.0,
            avg_memory_mb: 0.0,
            p95_frame_time_ms: 0.0,
            budget_violations: 0,
            within_budget_percent: 100.0,
        }
    }
}

impl PerformanceSummary {
    /// Create from metrics summary.
    pub fn from_metrics(summary: &MetricsSummary) -> Self {
        Self {
            sample_count: summary.sample_count,
            avg_input_latency_ms: summary.average.input_latency_ms,
            avg_layout_time_ms: summary.average.layout_time_ms,
            avg_render_time_ms: summary.average.render_time_ms,
            avg_memory_mb: summary.average.memory_usage_mb,
            p95_frame_time_ms: summary.p95.total_frame_time_ms(),
            budget_violations: 0, // Would need to track this
            within_budget_percent: 100.0,
        }
    }

    /// Check if performance is good.
    pub fn is_healthy(&self) -> bool {
        self.p95_frame_time_ms <= 16.67 && self.within_budget_percent >= 95.0
    }
}

// =============================================================================
// Support Report
// =============================================================================

/// A comprehensive support report for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportReport {
    /// Unique identifier for this report
    pub report_id: String,
    /// When the report was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Report version/format
    pub report_version: String,
    /// System information
    pub system_info: SystemInfo,
    /// Application state
    pub app_state: AppState,
    /// Recent log entries
    pub recent_logs: Vec<LogEntry>,
    /// Crash reports from this session
    pub crash_reports: Vec<CrashReport>,
    /// Performance summary
    pub performance_summary: PerformanceSummary,
    /// User description of the issue (if provided)
    pub user_description: Option<String>,
    /// Steps to reproduce (if provided)
    pub steps_to_reproduce: Option<String>,
    /// Additional attachments/metadata
    pub attachments: HashMap<String, String>,
    /// Whether the report has been anonymized
    pub is_anonymized: bool,
}

impl SupportReport {
    /// Create a new support report.
    pub fn new() -> Self {
        Self {
            report_id: uuid::Uuid::new_v4().to_string(),
            generated_at: chrono::Utc::now(),
            report_version: "1.0".to_string(),
            system_info: SystemInfo::default(),
            app_state: AppState::default(),
            recent_logs: Vec::new(),
            crash_reports: Vec::new(),
            performance_summary: PerformanceSummary::default(),
            user_description: None,
            steps_to_reproduce: None,
            attachments: HashMap::new(),
            is_anonymized: false,
        }
    }

    /// Set system info.
    pub fn with_system_info(mut self, info: SystemInfo) -> Self {
        self.system_info = info;
        self
    }

    /// Set app state.
    pub fn with_app_state(mut self, state: AppState) -> Self {
        self.app_state = state;
        self
    }

    /// Add log entries.
    pub fn with_logs(mut self, logs: Vec<LogEntry>) -> Self {
        self.recent_logs = logs;
        self
    }

    /// Add crash reports.
    pub fn with_crashes(mut self, crashes: Vec<CrashReport>) -> Self {
        self.crash_reports = crashes;
        self
    }

    /// Set performance summary.
    pub fn with_performance(mut self, summary: PerformanceSummary) -> Self {
        self.performance_summary = summary;
        self
    }

    /// Set user description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.user_description = Some(description.into());
        self
    }

    /// Add an attachment.
    pub fn with_attachment(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.attachments.insert(name.into(), content.into());
        self
    }

    /// Get severity level of the report based on crashes and errors.
    pub fn severity(&self) -> ReportSeverity {
        // Check for severe crashes
        let has_severe_crash = self.crash_reports.iter().any(|c| {
            matches!(
                c.crash_type,
                CrashType::Panic
                    | CrashType::SegmentationFault
                    | CrashType::OutOfMemory
                    | CrashType::StackOverflow
            )
        });

        if has_severe_crash {
            return ReportSeverity::Critical;
        }

        // Check for any crashes
        if !self.crash_reports.is_empty() {
            return ReportSeverity::High;
        }

        // Check for error logs
        let error_count = self
            .recent_logs
            .iter()
            .filter(|l| l.level == LogLevel::Error)
            .count();

        if error_count > 5 {
            return ReportSeverity::High;
        } else if error_count > 0 {
            return ReportSeverity::Medium;
        }

        // Check performance
        if !self.performance_summary.is_healthy() {
            return ReportSeverity::Medium;
        }

        ReportSeverity::Low
    }
}

impl Default for SupportReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Severity level of a support report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// =============================================================================
// Report Configuration
// =============================================================================

/// Configuration for report generation.
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Maximum number of log entries to include
    pub max_logs: usize,
    /// Maximum number of crash reports to include
    pub max_crashes: usize,
    /// Log level threshold (include this level and above)
    pub log_level_threshold: LogLevel,
    /// Whether to include system info
    pub include_system_info: bool,
    /// Whether to include performance data
    pub include_performance: bool,
    /// Whether to auto-anonymize
    pub auto_anonymize: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            max_logs: 100,
            max_crashes: 10,
            log_level_threshold: LogLevel::Info,
            include_system_info: true,
            include_performance: true,
            auto_anonymize: false,
        }
    }
}

impl ReportConfig {
    /// Create a minimal report configuration.
    pub fn minimal() -> Self {
        Self {
            max_logs: 20,
            max_crashes: 3,
            log_level_threshold: LogLevel::Warn,
            include_system_info: false,
            include_performance: false,
            auto_anonymize: true,
        }
    }

    /// Create a full report configuration.
    pub fn full() -> Self {
        Self {
            max_logs: 500,
            max_crashes: 50,
            log_level_threshold: LogLevel::Debug,
            include_system_info: true,
            include_performance: true,
            auto_anonymize: false,
        }
    }
}

// =============================================================================
// Support Report Generator
// =============================================================================

/// Generator for support reports.
#[derive(Debug, Clone)]
pub struct SupportReportGenerator {
    /// Configuration
    config: ReportConfig,
    /// Collected log entries
    logs: Vec<LogEntry>,
    /// Collected crash reports
    crashes: Vec<CrashReport>,
    /// Current app state
    app_state: Option<AppState>,
    /// Performance metrics
    performance: Option<PerformanceSummary>,
}

impl Default for SupportReportGenerator {
    fn default() -> Self {
        Self::new(ReportConfig::default())
    }
}

impl SupportReportGenerator {
    /// Create a new report generator.
    pub fn new(config: ReportConfig) -> Self {
        Self {
            config,
            logs: Vec::new(),
            crashes: Vec::new(),
            app_state: None,
            performance: None,
        }
    }

    /// Add a log entry.
    pub fn add_log(&mut self, entry: LogEntry) {
        if entry.level.severity() >= self.config.log_level_threshold.severity() {
            self.logs.push(entry);
            // Keep under limit
            while self.logs.len() > self.config.max_logs * 2 {
                self.logs.remove(0);
            }
        }
    }

    /// Add a crash report.
    pub fn add_crash(&mut self, crash: CrashReport) {
        self.crashes.push(crash);
        // Keep under limit
        while self.crashes.len() > self.config.max_crashes * 2 {
            self.crashes.remove(0);
        }
    }

    /// Set current app state.
    pub fn set_app_state(&mut self, state: AppState) {
        self.app_state = Some(state);
    }

    /// Set performance summary.
    pub fn set_performance(&mut self, perf: PerformanceSummary) {
        self.performance = Some(perf);
    }

    /// Set performance from metrics summary.
    pub fn set_performance_from_metrics(&mut self, summary: &MetricsSummary) {
        self.performance = Some(PerformanceSummary::from_metrics(summary));
    }

    /// Generate a support report.
    pub fn generate_report(&self) -> SupportReport {
        let mut report = SupportReport::new();

        // Add system info
        if self.config.include_system_info {
            report.system_info = SystemInfo::collect();
        }

        // Add app state
        if let Some(ref state) = self.app_state {
            report.app_state = state.clone();
        }

        // Add filtered logs
        let mut logs: Vec<LogEntry> = self
            .logs
            .iter()
            .filter(|l| l.level.severity() >= self.config.log_level_threshold.severity())
            .cloned()
            .collect();
        logs.truncate(self.config.max_logs);
        report.recent_logs = logs;

        // Add crash reports
        let mut crashes = self.crashes.clone();
        crashes.truncate(self.config.max_crashes);
        report.crash_reports = crashes;

        // Add performance
        if self.config.include_performance {
            if let Some(ref perf) = self.performance {
                report.performance_summary = perf.clone();
            }
        }

        // Auto-anonymize if configured
        if self.config.auto_anonymize {
            report = self.anonymize(report);
        }

        report
    }

    /// Anonymize a support report.
    pub fn anonymize(&self, mut report: SupportReport) -> SupportReport {
        // Anonymize system info
        report.system_info.locale = "redacted".to_string();
        report.system_info.timezone = "redacted".to_string();

        // Anonymize app state
        report.app_state.session_id = anonymize_id(&report.app_state.session_id);

        // Anonymize logs
        for log in &mut report.recent_logs {
            log.message = anonymize_text(&log.message);
            for value in log.context.values_mut() {
                *value = anonymize_text(value);
            }
        }

        // Anonymize crash reports
        for crash in &mut report.crash_reports {
            crash.session_id = anonymize_id(&crash.session_id);
            crash.message = anonymize_text(&crash.message);
            if let Some(ref mut trace) = crash.stack_trace {
                *trace = anonymize_text(trace);
            }
            for value in crash.context.values_mut() {
                *value = anonymize_text(value);
            }
        }

        // Anonymize user description
        if let Some(ref mut desc) = report.user_description {
            *desc = anonymize_text(desc);
        }

        // Anonymize attachments
        for value in report.attachments.values_mut() {
            *value = anonymize_text(value);
        }

        report.is_anonymized = true;
        report
    }

    /// Export report to JSON string.
    pub fn export_to_json(&self, report: &SupportReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// Export report to file.
    pub fn export_to_file(
        &self,
        report: &SupportReport,
        path: impl AsRef<Path>,
    ) -> std::io::Result<PathBuf> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let path = path.as_ref().to_path_buf();
        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Get configuration.
    pub fn config(&self) -> &ReportConfig {
        &self.config
    }

    /// Clear collected data.
    pub fn clear(&mut self) {
        self.logs.clear();
        self.crashes.clear();
        self.app_state = None;
        self.performance = None;
    }
}

// =============================================================================
// Anonymization Helpers
// =============================================================================

fn anonymize_id(id: &str) -> String {
    if id.len() <= 8 {
        "********".to_string()
    } else {
        format!("{}****{}", &id[..4], &id[id.len() - 4..])
    }
}

fn anonymize_text(text: &str) -> String {
    let mut result = text.to_string();

    // Redact file paths
    result = redact_paths(&result);

    // Redact email addresses
    result = redact_emails(&result);

    // Redact usernames in common patterns
    result = redact_usernames(&result);

    result
}

fn redact_paths(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' || c == '\\' {
            // Check if this looks like the start of a path
            if let Some(&next) = chars.peek() {
                if next.is_alphanumeric() || next == '~' {
                    result.push_str("<path>");
                    // Skip until whitespace or end
                    while let Some(&next) = chars.peek() {
                        if next.is_whitespace() || next == '"' || next == '\'' || next == ')' || next == ']' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

fn redact_emails(s: &str) -> String {
    // Simple email pattern replacement
    let mut result = s.to_string();
    let email_pattern = regex_lite::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").ok();

    if let Some(re) = email_pattern {
        result = re.replace_all(&result, "<email>").to_string();
    }

    result
}

fn redact_usernames(s: &str) -> String {
    let mut result = s.to_string();

    // Common patterns like /Users/username or C:\Users\username
    let patterns = [
        (r"/Users/([^/\s]+)", "/Users/<user>"),
        (r"/home/([^/\s]+)", "/home/<user>"),
        (r"C:\\Users\\([^\\\s]+)", "C:\\Users\\<user>"),
    ];

    for (pattern, replacement) in patterns {
        if let Ok(re) = regex_lite::Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_collect() {
        let info = SystemInfo::collect();
        assert!(!info.os_name.is_empty());
        assert!(!info.architecture.is_empty());
        assert!(info.cpu_cores >= 1);
    }

    #[test]
    fn test_system_info_with_memory() {
        let info = SystemInfo::collect().with_memory(16000, 8000);
        assert_eq!(info.total_memory_mb, 16000);
        assert_eq!(info.available_memory_mb, 8000);
    }

    #[test]
    fn test_display_info() {
        let display = DisplayInfo::new(1920, 1080, 2.0);
        assert_eq!(display.width, 1920);
        assert_eq!(display.height, 1080);
        assert_eq!(display.scale_factor, 2.0);
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new("1.0.0", "session-123");
        assert_eq!(state.app_version, "1.0.0");
        assert_eq!(state.session_id, "session-123");
    }

    #[test]
    fn test_app_state_with_document() {
        let doc = DocumentState::new("docx").with_metrics(100, 5, 1000);
        let state = AppState::new("1.0", "s").with_document(doc);

        assert!(state.current_document.is_some());
        assert_eq!(state.open_documents, 1);
    }

    #[test]
    fn test_app_state_with_commands() {
        let state = AppState::new("1.0", "s")
            .with_command("bold")
            .with_command("italic");

        assert_eq!(state.recent_commands.len(), 2);
    }

    #[test]
    fn test_document_state() {
        let doc = DocumentState::new("docx").with_metrics(500, 10, 5000);
        assert_eq!(doc.format, "docx");
        assert_eq!(doc.size_kb, 500);
        assert_eq!(doc.page_count, 10);
    }

    #[test]
    fn test_log_entry() {
        let entry = LogEntry::new(LogLevel::Error, "Test error", "test_component")
            .with_context("key", "value");

        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.message, "Test error");
        assert_eq!(entry.context.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_log_level_severity() {
        assert!(LogLevel::Error.severity() > LogLevel::Warn.severity());
        assert!(LogLevel::Warn.severity() > LogLevel::Info.severity());
        assert!(LogLevel::Info.severity() > LogLevel::Debug.severity());
    }

    #[test]
    fn test_performance_summary_default() {
        let summary = PerformanceSummary::default();
        assert_eq!(summary.sample_count, 0);
        assert!(summary.is_healthy());
    }

    #[test]
    fn test_performance_summary_unhealthy() {
        let mut summary = PerformanceSummary::default();
        summary.p95_frame_time_ms = 30.0;
        assert!(!summary.is_healthy());
    }

    #[test]
    fn test_support_report_new() {
        let report = SupportReport::new();
        assert!(!report.report_id.is_empty());
        assert!(!report.is_anonymized);
    }

    #[test]
    fn test_support_report_builder() {
        let report = SupportReport::new()
            .with_description("Test issue")
            .with_attachment("config", "some config data");

        assert_eq!(report.user_description, Some("Test issue".to_string()));
        assert!(report.attachments.contains_key("config"));
    }

    #[test]
    fn test_support_report_severity_low() {
        let report = SupportReport::new();
        assert_eq!(report.severity(), ReportSeverity::Low);
    }

    #[test]
    fn test_support_report_severity_with_crash() {
        let crash = CrashReport::new("1.0", "test", "session", CrashType::Panic, "test error");
        let report = SupportReport::new().with_crashes(vec![crash]);
        assert_eq!(report.severity(), ReportSeverity::Critical);
    }

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert_eq!(config.max_logs, 100);
        assert!(config.include_system_info);
    }

    #[test]
    fn test_report_config_minimal() {
        let config = ReportConfig::minimal();
        assert_eq!(config.max_logs, 20);
        assert!(!config.include_system_info);
        assert!(config.auto_anonymize);
    }

    #[test]
    fn test_generator_new() {
        let generator = SupportReportGenerator::new(ReportConfig::default());
        assert_eq!(generator.config().max_logs, 100);
    }

    #[test]
    fn test_generator_add_log() {
        let mut generator = SupportReportGenerator::new(ReportConfig::default());
        generator.add_log(LogEntry::new(LogLevel::Error, "Error message", "test"));

        let report = generator.generate_report();
        assert_eq!(report.recent_logs.len(), 1);
    }

    #[test]
    fn test_generator_add_crash() {
        let mut generator = SupportReportGenerator::new(ReportConfig::default());
        let crash = CrashReport::new("1.0", "test", "session", CrashType::Panic, "test");
        generator.add_crash(crash);

        let report = generator.generate_report();
        assert_eq!(report.crash_reports.len(), 1);
    }

    #[test]
    fn test_generator_generate_report() {
        let mut generator = SupportReportGenerator::new(ReportConfig::default());
        generator.set_app_state(AppState::new("1.0.0", "session"));
        generator.set_performance(PerformanceSummary::default());

        let report = generator.generate_report();
        assert_eq!(report.app_state.app_version, "1.0.0");
    }

    #[test]
    fn test_generator_anonymize() {
        let generator = SupportReportGenerator::new(ReportConfig::default());
        let report = SupportReport::new()
            .with_description("Error at /Users/john/documents/file.docx");

        let anonymized = generator.anonymize(report);
        assert!(anonymized.is_anonymized);
        assert!(!anonymized.user_description.unwrap().contains("john"));
    }

    #[test]
    fn test_generator_clear() {
        let mut generator = SupportReportGenerator::new(ReportConfig::default());
        generator.add_log(LogEntry::new(LogLevel::Info, "test", "test"));
        generator.clear();

        let report = generator.generate_report();
        assert!(report.recent_logs.is_empty());
    }

    #[test]
    fn test_anonymize_id() {
        assert_eq!(anonymize_id("short"), "********");
        assert_eq!(anonymize_id("1234567890abcdef"), "1234****cdef");
    }

    #[test]
    fn test_redact_paths() {
        let text = "Error at /Users/test/file.txt happened";
        let redacted = redact_paths(text);
        assert!(!redacted.contains("/Users/test"));
        assert!(redacted.contains("<path>"));
    }

    #[test]
    fn test_redact_emails() {
        let text = "Contact support at user@example.com for help";
        let redacted = redact_emails(text);
        assert!(!redacted.contains("user@example.com"));
        assert!(redacted.contains("<email>"));
    }

    #[test]
    fn test_support_report_serialization() {
        let report = SupportReport::new()
            .with_description("Test issue");

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: SupportReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.report_id, report.report_id);
        assert_eq!(deserialized.user_description, Some("Test issue".to_string()));
    }

    #[test]
    fn test_generator_export_json() {
        let generator = SupportReportGenerator::new(ReportConfig::default());
        let report = generator.generate_report();

        let json = generator.export_to_json(&report).unwrap();
        assert!(json.contains("report_id"));
        assert!(json.contains("system_info"));
    }
}
