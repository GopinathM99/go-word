//! Performance Profiler Module
//!
//! Provides hierarchical performance profiling for tracking operation timings.
//! Supports nested spans for detailed performance analysis.
//!
//! # Example
//!
//! ```rust
//! use telemetry::profiler::PerformanceProfiler;
//!
//! let mut profiler = PerformanceProfiler::new();
//!
//! profiler.start_span("document_save");
//! profiler.start_span("serialize");
//! // ... serialize document ...
//! profiler.end_span();
//! profiler.start_span("write_file");
//! // ... write to disk ...
//! profiler.end_span();
//! profiler.end_span();
//!
//! let trace = profiler.get_trace();
//! println!("Total duration: {:?}", trace.total_duration);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// =============================================================================
// Profile Span
// =============================================================================

/// A single profiled operation span with timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSpan {
    /// Name of the operation being profiled
    pub name: String,
    /// Duration of this span (calculated when span ends)
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Child spans within this span
    pub children: Vec<ProfileSpan>,
    /// Depth level in the span hierarchy
    pub depth: usize,
    /// Metadata/tags for this span
    pub tags: Vec<(String, String)>,
}

impl ProfileSpan {
    /// Create a new profile span.
    pub fn new(name: impl Into<String>, depth: usize) -> Self {
        Self {
            name: name.into(),
            duration: Duration::ZERO,
            children: Vec::new(),
            depth,
            tags: Vec::new(),
        }
    }

    /// Add a tag to this span.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }

    /// Get the self time (excluding children).
    pub fn self_time(&self) -> Duration {
        let children_time: Duration = self.children.iter().map(|c| c.duration).sum();
        self.duration.saturating_sub(children_time)
    }

    /// Get total span count including children.
    pub fn total_span_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_span_count()).sum::<usize>()
    }
}

/// Internal representation of an active span.
#[derive(Debug)]
struct ActiveSpan {
    name: String,
    start_time: Instant,
    children: Vec<ProfileSpan>,
    depth: usize,
    tags: Vec<(String, String)>,
}

impl ActiveSpan {
    fn new(name: impl Into<String>, depth: usize) -> Self {
        Self {
            name: name.into(),
            start_time: Instant::now(),
            children: Vec::new(),
            depth,
            tags: Vec::new(),
        }
    }

    fn into_span(self) -> ProfileSpan {
        ProfileSpan {
            name: self.name,
            duration: self.start_time.elapsed(),
            children: self.children,
            depth: self.depth,
            tags: self.tags,
        }
    }
}

// =============================================================================
// Profile Trace
// =============================================================================

/// A complete profile trace containing all spans from a profiling session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileTrace {
    /// Root spans in this trace
    pub spans: Vec<ProfileSpan>,
    /// Total duration of the entire trace
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
    /// When the trace was started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Name/description of what was being traced
    pub name: String,
}

impl ProfileTrace {
    /// Create a new profile trace.
    pub fn new(name: impl Into<String>, spans: Vec<ProfileSpan>, total_duration: Duration) -> Self {
        Self {
            name: name.into(),
            spans,
            total_duration,
            started_at: chrono::Utc::now(),
        }
    }

    /// Get total number of spans in this trace.
    pub fn span_count(&self) -> usize {
        self.spans.iter().map(|s| s.total_span_count()).sum()
    }

    /// Check if the trace is empty.
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Find spans by name.
    pub fn find_spans(&self, name: &str) -> Vec<&ProfileSpan> {
        fn find_recursive<'a>(span: &'a ProfileSpan, name: &str, results: &mut Vec<&'a ProfileSpan>) {
            if span.name == name {
                results.push(span);
            }
            for child in &span.children {
                find_recursive(child, name, results);
            }
        }

        let mut results = Vec::new();
        for span in &self.spans {
            find_recursive(span, name, &mut results);
        }
        results
    }

    /// Generate timeline visualization data for UI display.
    pub fn to_timeline(&self) -> TimelineData {
        let mut entries = Vec::new();
        let _base_time = Instant::now(); // Reference point for relative times

        fn flatten_spans(
            span: &ProfileSpan,
            entries: &mut Vec<TimelineEntry>,
            offset: Duration,
        ) -> Duration {
            entries.push(TimelineEntry {
                name: span.name.clone(),
                start_offset: offset,
                duration: span.duration,
                depth: span.depth,
                tags: span.tags.clone(),
            });

            let mut child_offset = offset;
            for child in &span.children {
                child_offset = flatten_spans(child, entries, child_offset);
            }

            offset + span.duration
        }

        let mut current_offset = Duration::ZERO;
        for span in &self.spans {
            current_offset = flatten_spans(span, &mut entries, current_offset);
        }

        TimelineData {
            entries,
            total_duration: self.total_duration,
        }
    }
}

/// Timeline visualization data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    /// Flattened timeline entries
    pub entries: Vec<TimelineEntry>,
    /// Total timeline duration
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
}

/// A single entry in the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Span name
    pub name: String,
    /// Start offset from trace start
    #[serde(with = "duration_serde")]
    pub start_offset: Duration,
    /// Duration of this entry
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Nesting depth
    pub depth: usize,
    /// Tags/metadata
    pub tags: Vec<(String, String)>,
}

// =============================================================================
// Performance Profiler
// =============================================================================

/// Performance profiler for tracking hierarchical operation timings.
#[derive(Debug)]
pub struct PerformanceProfiler {
    /// Stack of active spans
    active_spans: Vec<ActiveSpan>,
    /// Completed root spans
    completed_spans: Vec<ProfileSpan>,
    /// When profiling started
    start_time: Option<Instant>,
    /// Name of the current profiling session
    session_name: String,
    /// Maximum span depth allowed
    max_depth: usize,
    /// History of completed traces
    trace_history: VecDeque<ProfileTrace>,
    /// Maximum traces to keep in history
    max_history: usize,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceProfiler {
    /// Create a new performance profiler.
    pub fn new() -> Self {
        Self {
            active_spans: Vec::new(),
            completed_spans: Vec::new(),
            start_time: None,
            session_name: String::new(),
            max_depth: 100,
            trace_history: VecDeque::new(),
            max_history: 50,
        }
    }

    /// Create a profiler with custom settings.
    pub fn with_settings(max_depth: usize, max_history: usize) -> Self {
        Self {
            max_depth,
            max_history,
            ..Self::new()
        }
    }

    /// Start a new profiling session.
    pub fn start_session(&mut self, name: impl Into<String>) {
        self.session_name = name.into();
        self.start_time = Some(Instant::now());
        self.active_spans.clear();
        self.completed_spans.clear();
    }

    /// Start a new span.
    ///
    /// Returns `true` if the span was started, `false` if max depth was exceeded.
    pub fn start_span(&mut self, name: impl Into<String>) -> bool {
        let depth = self.active_spans.len();

        if depth >= self.max_depth {
            return false;
        }

        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        self.active_spans.push(ActiveSpan::new(name, depth));
        true
    }

    /// Start a span with tags.
    pub fn start_span_with_tags(
        &mut self,
        name: impl Into<String>,
        tags: Vec<(String, String)>,
    ) -> bool {
        let depth = self.active_spans.len();

        if depth >= self.max_depth {
            return false;
        }

        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        let mut span = ActiveSpan::new(name, depth);
        span.tags = tags;
        self.active_spans.push(span);
        true
    }

    /// End the current span.
    ///
    /// Returns the completed span, or `None` if no span was active.
    pub fn end_span(&mut self) -> Option<ProfileSpan> {
        let active = self.active_spans.pop()?;
        let completed = active.into_span();

        // Add to parent or root
        if let Some(parent) = self.active_spans.last_mut() {
            parent.children.push(completed.clone());
        } else {
            self.completed_spans.push(completed.clone());
        }

        Some(completed)
    }

    /// End span and return its duration.
    pub fn end_span_duration(&mut self) -> Option<Duration> {
        self.end_span().map(|s| s.duration)
    }

    /// Get the current profiling trace without ending the session.
    pub fn get_trace(&self) -> ProfileTrace {
        let total_duration = self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        ProfileTrace::new(
            self.session_name.clone(),
            self.completed_spans.clone(),
            total_duration,
        )
    }

    /// Finish the current session and get the final trace.
    pub fn finish_session(&mut self) -> ProfileTrace {
        // End any remaining active spans
        while !self.active_spans.is_empty() {
            self.end_span();
        }

        let trace = self.get_trace();

        // Store in history
        self.trace_history.push_back(trace.clone());
        if self.trace_history.len() > self.max_history {
            self.trace_history.pop_front();
        }

        // Reset for next session
        self.start_time = None;
        self.session_name.clear();
        self.completed_spans.clear();

        trace
    }

    /// Check if profiling is currently active.
    pub fn is_active(&self) -> bool {
        self.start_time.is_some()
    }

    /// Get the current depth (number of active spans).
    pub fn current_depth(&self) -> usize {
        self.active_spans.len()
    }

    /// Get trace history.
    pub fn get_history(&self) -> &VecDeque<ProfileTrace> {
        &self.trace_history
    }

    /// Clear trace history.
    pub fn clear_history(&mut self) {
        self.trace_history.clear();
    }

    /// Export trace to JSON format.
    pub fn export_trace(&self) -> Result<String, serde_json::Error> {
        let trace = self.get_trace();
        serde_json::to_string_pretty(&trace)
    }

    /// Export trace to a compact format suitable for logging.
    pub fn export_trace_compact(&self) -> String {
        let trace = self.get_trace();
        let mut output = format!(
            "Profile: {} (total: {:?})\n",
            trace.name, trace.total_duration
        );

        fn format_span(span: &ProfileSpan, output: &mut String) {
            let indent = "  ".repeat(span.depth);
            output.push_str(&format!(
                "{}{}: {:?} (self: {:?})\n",
                indent,
                span.name,
                span.duration,
                span.self_time()
            ));
            for child in &span.children {
                format_span(child, output);
            }
        }

        for span in &trace.spans {
            format_span(span, &mut output);
        }

        output
    }
}

// =============================================================================
// Scoped Span Guard
// =============================================================================

/// RAII guard that automatically ends a span when dropped.
pub struct SpanGuard<'a> {
    profiler: &'a mut PerformanceProfiler,
}

impl<'a> SpanGuard<'a> {
    /// Create a new span guard.
    pub fn new(profiler: &'a mut PerformanceProfiler, name: impl Into<String>) -> Option<Self> {
        if profiler.start_span(name) {
            Some(Self { profiler })
        } else {
            None
        }
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        self.profiler.end_span();
    }
}

// =============================================================================
// Serde helpers for Duration
// =============================================================================

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_nanos().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u128::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos as u64))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_profile_span_new() {
        let span = ProfileSpan::new("test_op", 0);
        assert_eq!(span.name, "test_op");
        assert_eq!(span.depth, 0);
        assert!(span.children.is_empty());
        assert_eq!(span.duration, Duration::ZERO);
    }

    #[test]
    fn test_profile_span_with_tag() {
        let span = ProfileSpan::new("test", 0)
            .with_tag("key", "value");
        assert_eq!(span.tags.len(), 1);
        assert_eq!(span.tags[0], ("key".to_string(), "value".to_string()));
    }

    #[test]
    fn test_profile_span_self_time() {
        let mut span = ProfileSpan::new("parent", 0);
        span.duration = Duration::from_millis(100);

        let mut child = ProfileSpan::new("child", 1);
        child.duration = Duration::from_millis(30);
        span.children.push(child);

        assert_eq!(span.self_time(), Duration::from_millis(70));
    }

    #[test]
    fn test_profile_span_total_span_count() {
        let mut span = ProfileSpan::new("parent", 0);
        span.children.push(ProfileSpan::new("child1", 1));
        span.children.push(ProfileSpan::new("child2", 1));
        span.children[0].children.push(ProfileSpan::new("grandchild", 2));

        assert_eq!(span.total_span_count(), 4);
    }

    #[test]
    fn test_profile_trace_new() {
        let spans = vec![ProfileSpan::new("root", 0)];
        let trace = ProfileTrace::new("test_trace", spans, Duration::from_millis(100));

        assert_eq!(trace.name, "test_trace");
        assert_eq!(trace.total_duration, Duration::from_millis(100));
        assert_eq!(trace.span_count(), 1);
    }

    #[test]
    fn test_profile_trace_find_spans() {
        let mut root = ProfileSpan::new("root", 0);
        root.children.push(ProfileSpan::new("child", 1));
        root.children.push(ProfileSpan::new("child", 1));

        let trace = ProfileTrace::new("test", vec![root], Duration::ZERO);
        let found = trace.find_spans("child");

        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_profile_trace_to_timeline() {
        let mut root = ProfileSpan::new("root", 0);
        root.duration = Duration::from_millis(100);
        root.children.push({
            let mut child = ProfileSpan::new("child", 1);
            child.duration = Duration::from_millis(50);
            child
        });

        let trace = ProfileTrace::new("test", vec![root], Duration::from_millis(100));
        let timeline = trace.to_timeline();

        assert_eq!(timeline.entries.len(), 2);
        assert_eq!(timeline.entries[0].name, "root");
        assert_eq!(timeline.entries[1].name, "child");
    }

    #[test]
    fn test_profiler_new() {
        let profiler = PerformanceProfiler::new();
        assert!(!profiler.is_active());
        assert_eq!(profiler.current_depth(), 0);
    }

    #[test]
    fn test_profiler_start_span() {
        let mut profiler = PerformanceProfiler::new();

        assert!(profiler.start_span("test"));
        assert!(profiler.is_active());
        assert_eq!(profiler.current_depth(), 1);
    }

    #[test]
    fn test_profiler_nested_spans() {
        let mut profiler = PerformanceProfiler::new();

        profiler.start_span("outer");
        assert_eq!(profiler.current_depth(), 1);

        profiler.start_span("inner");
        assert_eq!(profiler.current_depth(), 2);

        profiler.end_span();
        assert_eq!(profiler.current_depth(), 1);

        profiler.end_span();
        assert_eq!(profiler.current_depth(), 0);
    }

    #[test]
    fn test_profiler_end_span() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_span("test");

        sleep(Duration::from_millis(10));

        let span = profiler.end_span().unwrap();
        assert_eq!(span.name, "test");
        assert!(span.duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_profiler_end_span_no_active() {
        let mut profiler = PerformanceProfiler::new();
        assert!(profiler.end_span().is_none());
    }

    #[test]
    fn test_profiler_get_trace() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_session("test_session");

        profiler.start_span("op1");
        profiler.end_span();

        profiler.start_span("op2");
        profiler.end_span();

        let trace = profiler.get_trace();
        assert_eq!(trace.name, "test_session");
        assert_eq!(trace.spans.len(), 2);
    }

    #[test]
    fn test_profiler_finish_session() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_session("test");

        profiler.start_span("outer");
        profiler.start_span("inner"); // Not ended manually

        let trace = profiler.finish_session();

        // Should have ended all spans
        assert_eq!(profiler.current_depth(), 0);
        assert!(!profiler.is_active());
        assert_eq!(trace.spans.len(), 1); // outer
        assert_eq!(trace.spans[0].children.len(), 1); // inner
    }

    #[test]
    fn test_profiler_max_depth() {
        let mut profiler = PerformanceProfiler::with_settings(3, 10);

        assert!(profiler.start_span("depth0"));
        assert!(profiler.start_span("depth1"));
        assert!(profiler.start_span("depth2"));
        assert!(!profiler.start_span("depth3")); // Should fail

        assert_eq!(profiler.current_depth(), 3);
    }

    #[test]
    fn test_profiler_history() {
        let mut profiler = PerformanceProfiler::with_settings(100, 2);

        profiler.start_session("session1");
        profiler.start_span("op");
        profiler.finish_session();

        profiler.start_session("session2");
        profiler.start_span("op");
        profiler.finish_session();

        profiler.start_session("session3");
        profiler.start_span("op");
        profiler.finish_session();

        assert_eq!(profiler.get_history().len(), 2);
        assert_eq!(profiler.get_history()[0].name, "session2");
        assert_eq!(profiler.get_history()[1].name, "session3");
    }

    #[test]
    fn test_profiler_export_trace() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_session("export_test");
        profiler.start_span("op");
        profiler.end_span();

        let json = profiler.export_trace().unwrap();
        assert!(json.contains("export_test"));
        assert!(json.contains("op"));
    }

    #[test]
    fn test_profiler_export_trace_compact() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_session("compact_test");
        profiler.start_span("parent");
        profiler.start_span("child");
        profiler.end_span();
        profiler.end_span();

        let output = profiler.export_trace_compact();
        assert!(output.contains("compact_test"));
        assert!(output.contains("parent"));
        assert!(output.contains("  child")); // Indented
    }

    #[test]
    fn test_profiler_span_with_tags() {
        let mut profiler = PerformanceProfiler::new();

        profiler.start_span_with_tags(
            "tagged_op",
            vec![("key".to_string(), "value".to_string())],
        );

        let span = profiler.end_span().unwrap();
        assert_eq!(span.tags.len(), 1);
    }

    #[test]
    fn test_profiler_clear_history() {
        let mut profiler = PerformanceProfiler::new();

        profiler.start_session("test");
        profiler.start_span("op");
        profiler.finish_session();

        assert!(!profiler.get_history().is_empty());
        profiler.clear_history();
        assert!(profiler.get_history().is_empty());
    }

    #[test]
    fn test_profile_trace_serialization() {
        let span = ProfileSpan::new("test", 0);
        let trace = ProfileTrace::new("test_trace", vec![span], Duration::from_millis(100));

        let json = serde_json::to_string(&trace).unwrap();
        let deserialized: ProfileTrace = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_trace");
        assert_eq!(deserialized.spans.len(), 1);
    }
}
