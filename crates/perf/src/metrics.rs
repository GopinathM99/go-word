//! Metrics collection for performance measurement

use crate::budget::{BudgetViolation, PerfBudget};
use crate::timing::TimerCategory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Global metrics instance
static GLOBAL_METRICS: OnceLock<Mutex<PerfMetrics>> = OnceLock::new();

/// Get the global metrics instance.
///
/// This is a thread-safe singleton that collects performance metrics
/// across the entire application.
pub fn global_metrics() -> &'static Mutex<PerfMetrics> {
    GLOBAL_METRICS.get_or_init(|| Mutex::new(PerfMetrics::new()))
}

/// Reset the global metrics.
pub fn reset_global_metrics() {
    if let Ok(mut metrics) = global_metrics().lock() {
        metrics.reset();
    }
}

/// Performance metrics collection.
///
/// Thread-safe container for recording and analyzing performance data.
#[derive(Debug, Clone)]
pub struct PerfMetrics {
    /// Command execution times, keyed by command name
    command_times: HashMap<String, Vec<f64>>,
    /// Layout calculation times
    layout_times: Vec<f64>,
    /// Render times
    render_times: Vec<f64>,
    /// Input latencies
    input_latencies: Vec<f64>,
    /// General timing data, keyed by name
    general_times: HashMap<String, Vec<f64>>,
    /// Performance budget for violation checking
    budget: PerfBudget,
    /// Maximum samples to keep per category (to prevent unbounded growth)
    max_samples: usize,
    /// Whether metrics collection is enabled
    enabled: bool,
}

impl PerfMetrics {
    /// Create a new metrics collector with default settings.
    pub fn new() -> Self {
        Self {
            command_times: HashMap::new(),
            layout_times: Vec::new(),
            render_times: Vec::new(),
            input_latencies: Vec::new(),
            general_times: HashMap::new(),
            budget: PerfBudget::default(),
            max_samples: 1000,
            enabled: true,
        }
    }

    /// Create a metrics collector with a custom budget.
    pub fn with_budget(budget: PerfBudget) -> Self {
        Self {
            budget,
            ..Self::new()
        }
    }

    /// Set the maximum number of samples to keep per category.
    pub fn with_max_samples(mut self, max: usize) -> Self {
        self.max_samples = max;
        self
    }

    /// Enable or disable metrics collection.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if metrics collection is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a timing based on category.
    #[cfg(feature = "telemetry")]
    pub fn record_timing(&mut self, name: &str, duration_ms: f64, category: TimerCategory) {
        if !self.enabled {
            return;
        }

        match category {
            TimerCategory::Command => self.record_command(name, duration_ms),
            TimerCategory::Layout => self.record_layout(duration_ms),
            TimerCategory::Render => self.record_render(duration_ms),
            TimerCategory::Input => self.record_input_latency(duration_ms),
            TimerCategory::General => self.record_general(name, duration_ms),
        }
    }

    /// Record a timing (no-op when telemetry is disabled).
    #[cfg(not(feature = "telemetry"))]
    #[inline]
    pub fn record_timing(&mut self, _name: &str, _duration_ms: f64, _category: TimerCategory) {}

    /// Record a command execution time.
    pub fn record_command(&mut self, name: &str, duration_ms: f64) {
        if !self.enabled {
            return;
        }

        let times = self
            .command_times
            .entry(name.to_string())
            .or_insert_with(Vec::new);

        if times.len() >= self.max_samples {
            times.remove(0);
        }
        times.push(duration_ms);

        tracing::trace!(
            target: "perf::command",
            command = name,
            duration_ms = duration_ms,
            "command recorded"
        );
    }

    /// Record a layout calculation time.
    pub fn record_layout(&mut self, duration_ms: f64) {
        if !self.enabled {
            return;
        }

        if self.layout_times.len() >= self.max_samples {
            self.layout_times.remove(0);
        }
        self.layout_times.push(duration_ms);

        tracing::trace!(
            target: "perf::layout",
            duration_ms = duration_ms,
            "layout recorded"
        );
    }

    /// Record a render time.
    pub fn record_render(&mut self, duration_ms: f64) {
        if !self.enabled {
            return;
        }

        if self.render_times.len() >= self.max_samples {
            self.render_times.remove(0);
        }
        self.render_times.push(duration_ms);

        tracing::trace!(
            target: "perf::render",
            duration_ms = duration_ms,
            "render recorded"
        );
    }

    /// Record an input latency measurement.
    pub fn record_input_latency(&mut self, duration_ms: f64) {
        if !self.enabled {
            return;
        }

        if self.input_latencies.len() >= self.max_samples {
            self.input_latencies.remove(0);
        }
        self.input_latencies.push(duration_ms);

        tracing::trace!(
            target: "perf::input",
            duration_ms = duration_ms,
            "input latency recorded"
        );
    }

    /// Record a general timing measurement.
    pub fn record_general(&mut self, name: &str, duration_ms: f64) {
        if !self.enabled {
            return;
        }

        let times = self
            .general_times
            .entry(name.to_string())
            .or_insert_with(Vec::new);

        if times.len() >= self.max_samples {
            times.remove(0);
        }
        times.push(duration_ms);
    }

    /// Get a summary of all collected metrics.
    pub fn summary(&self) -> PerfSummary {
        PerfSummary {
            command_stats: self
                .command_times
                .iter()
                .map(|(name, times)| (name.clone(), TimingStats::from_samples(times)))
                .collect(),
            layout_stats: TimingStats::from_samples(&self.layout_times),
            render_stats: TimingStats::from_samples(&self.render_times),
            input_latency_stats: TimingStats::from_samples(&self.input_latencies),
            general_stats: self
                .general_times
                .iter()
                .map(|(name, times)| (name.clone(), TimingStats::from_samples(times)))
                .collect(),
            total_commands: self.command_times.values().map(|v| v.len()).sum(),
            total_layouts: self.layout_times.len(),
            total_renders: self.render_times.len(),
            total_inputs: self.input_latencies.len(),
        }
    }

    /// Check the performance budget and return any violations.
    pub fn check_budget(&self) -> Vec<BudgetViolation> {
        let mut violations = Vec::new();

        // Check input latency
        if let Some(&last_latency) = self.input_latencies.last() {
            if last_latency > self.budget.max_input_latency_ms {
                violations.push(BudgetViolation {
                    category: "input_latency".to_string(),
                    actual_ms: last_latency,
                    budget_ms: self.budget.max_input_latency_ms,
                    severity: violation_severity(last_latency, self.budget.max_input_latency_ms),
                });
            }
        }

        // Check layout time
        if let Some(&last_layout) = self.layout_times.last() {
            if last_layout > self.budget.max_layout_time_ms {
                violations.push(BudgetViolation {
                    category: "layout".to_string(),
                    actual_ms: last_layout,
                    budget_ms: self.budget.max_layout_time_ms,
                    severity: violation_severity(last_layout, self.budget.max_layout_time_ms),
                });
            }
        }

        // Check render time
        if let Some(&last_render) = self.render_times.last() {
            if last_render > self.budget.max_render_time_ms {
                violations.push(BudgetViolation {
                    category: "render".to_string(),
                    actual_ms: last_render,
                    budget_ms: self.budget.max_render_time_ms,
                    severity: violation_severity(last_render, self.budget.max_render_time_ms),
                });
            }
        }

        // Check command times
        for (name, times) in &self.command_times {
            if let Some(&last_time) = times.last() {
                if last_time > self.budget.max_command_time_ms {
                    violations.push(BudgetViolation {
                        category: format!("command:{}", name),
                        actual_ms: last_time,
                        budget_ms: self.budget.max_command_time_ms,
                        severity: violation_severity(last_time, self.budget.max_command_time_ms),
                    });
                }
            }
        }

        violations
    }

    /// Get the current performance budget.
    pub fn budget(&self) -> &PerfBudget {
        &self.budget
    }

    /// Set the performance budget.
    pub fn set_budget(&mut self, budget: PerfBudget) {
        self.budget = budget;
    }

    /// Reset all collected metrics.
    pub fn reset(&mut self) {
        self.command_times.clear();
        self.layout_times.clear();
        self.render_times.clear();
        self.input_latencies.clear();
        self.general_times.clear();
    }

    /// Get raw command times.
    pub fn command_times(&self) -> &HashMap<String, Vec<f64>> {
        &self.command_times
    }

    /// Get raw layout times.
    pub fn layout_times(&self) -> &[f64] {
        &self.layout_times
    }

    /// Get raw render times.
    pub fn render_times(&self) -> &[f64] {
        &self.render_times
    }

    /// Get raw input latencies.
    pub fn input_latencies(&self) -> &[f64] {
        &self.input_latencies
    }
}

impl Default for PerfMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate violation severity based on how much the actual exceeds budget.
fn violation_severity(actual: f64, budget: f64) -> ViolationSeverity {
    let ratio = actual / budget;
    if ratio > 3.0 {
        ViolationSeverity::Critical
    } else if ratio > 2.0 {
        ViolationSeverity::High
    } else if ratio > 1.5 {
        ViolationSeverity::Medium
    } else {
        ViolationSeverity::Low
    }
}

/// Severity level of a budget violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViolationSeverity {
    /// Minor violation (1-1.5x budget)
    Low,
    /// Moderate violation (1.5-2x budget)
    Medium,
    /// Significant violation (2-3x budget)
    High,
    /// Severe violation (>3x budget)
    Critical,
}

/// Summary of performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfSummary {
    /// Statistics for each command type
    pub command_stats: HashMap<String, TimingStats>,
    /// Layout timing statistics
    pub layout_stats: TimingStats,
    /// Render timing statistics
    pub render_stats: TimingStats,
    /// Input latency statistics
    pub input_latency_stats: TimingStats,
    /// General timing statistics
    pub general_stats: HashMap<String, TimingStats>,
    /// Total number of commands recorded
    pub total_commands: usize,
    /// Total number of layout operations recorded
    pub total_layouts: usize,
    /// Total number of render operations recorded
    pub total_renders: usize,
    /// Total number of input events recorded
    pub total_inputs: usize,
}

/// Statistical summary of timing data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingStats {
    /// Number of samples
    pub count: usize,
    /// Minimum time in milliseconds
    pub min_ms: f64,
    /// Maximum time in milliseconds
    pub max_ms: f64,
    /// Mean time in milliseconds
    pub mean_ms: f64,
    /// Median time in milliseconds
    pub median_ms: f64,
    /// 95th percentile in milliseconds
    pub p95_ms: f64,
    /// 99th percentile in milliseconds
    pub p99_ms: f64,
    /// Standard deviation in milliseconds
    pub std_dev_ms: f64,
    /// Total time in milliseconds
    pub total_ms: f64,
}

impl TimingStats {
    /// Calculate statistics from a slice of samples.
    pub fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let count = samples.len();
        let mut sorted: Vec<f64> = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min_ms = sorted[0];
        let max_ms = sorted[count - 1];
        let total_ms: f64 = samples.iter().sum();
        let mean_ms = total_ms / count as f64;

        let median_ms = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };

        let p95_ms = percentile(&sorted, 95.0);
        let p99_ms = percentile(&sorted, 99.0);

        let variance: f64 = samples.iter().map(|x| (x - mean_ms).powi(2)).sum::<f64>() / count as f64;
        let std_dev_ms = variance.sqrt();

        Self {
            count,
            min_ms,
            max_ms,
            mean_ms,
            median_ms,
            p95_ms,
            p99_ms,
            std_dev_ms,
            total_ms,
        }
    }
}

/// Calculate a percentile from sorted samples.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }

    let rank = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let fraction = rank - lower as f64;

    if upper >= sorted.len() {
        sorted[sorted.len() - 1]
    } else {
        sorted[lower] + fraction * (sorted[upper] - sorted[lower])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_stats_from_samples() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = TimingStats::from_samples(&samples);

        assert_eq!(stats.count, 5);
        assert_eq!(stats.min_ms, 1.0);
        assert_eq!(stats.max_ms, 5.0);
        assert_eq!(stats.mean_ms, 3.0);
        assert_eq!(stats.median_ms, 3.0);
        assert_eq!(stats.total_ms, 15.0);
    }

    #[test]
    fn test_timing_stats_empty() {
        let samples: Vec<f64> = vec![];
        let stats = TimingStats::from_samples(&samples);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.min_ms, 0.0);
        assert_eq!(stats.max_ms, 0.0);
    }

    #[test]
    fn test_metrics_record_command() {
        let mut metrics = PerfMetrics::new();
        metrics.record_command("insert_text", 5.0);
        metrics.record_command("insert_text", 10.0);
        metrics.record_command("delete", 3.0);

        assert_eq!(metrics.command_times.get("insert_text").unwrap().len(), 2);
        assert_eq!(metrics.command_times.get("delete").unwrap().len(), 1);
    }

    #[test]
    fn test_metrics_max_samples() {
        let mut metrics = PerfMetrics::new().with_max_samples(3);

        for i in 0..5 {
            metrics.record_layout(i as f64);
        }

        assert_eq!(metrics.layout_times.len(), 3);
        assert_eq!(metrics.layout_times, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_metrics_disabled() {
        let mut metrics = PerfMetrics::new();
        metrics.set_enabled(false);

        metrics.record_command("test", 5.0);
        metrics.record_layout(10.0);

        assert!(metrics.command_times.is_empty());
        assert!(metrics.layout_times.is_empty());
    }

    #[test]
    fn test_budget_violations() {
        let budget = PerfBudget {
            max_input_latency_ms: 50.0,
            max_layout_time_ms: 10.0,
            max_render_time_ms: 16.0,
            max_command_time_ms: 100.0,
        };

        let mut metrics = PerfMetrics::with_budget(budget);
        metrics.record_input_latency(100.0); // Violates 50ms budget
        metrics.record_layout(5.0); // OK
        metrics.record_render(20.0); // Violates 16ms budget

        let violations = metrics.check_budget();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_violation_severity() {
        assert_eq!(violation_severity(55.0, 50.0), ViolationSeverity::Low);
        assert_eq!(violation_severity(80.0, 50.0), ViolationSeverity::Medium);
        assert_eq!(violation_severity(110.0, 50.0), ViolationSeverity::High);
        assert_eq!(violation_severity(200.0, 50.0), ViolationSeverity::Critical);
    }

    #[test]
    fn test_percentile() {
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        assert!((percentile(&sorted, 50.0) - 5.5).abs() < 0.01);
        assert!((percentile(&sorted, 90.0) - 9.1).abs() < 0.01);
    }
}
