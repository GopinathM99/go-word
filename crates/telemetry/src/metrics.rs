//! Performance metrics collection and analysis.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Performance metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMetrics {
    /// Input event processing latency in milliseconds
    pub input_latency_ms: f64,
    /// Document layout computation time in milliseconds
    pub layout_time_ms: f64,
    /// Screen render time in milliseconds
    pub render_time_ms: f64,
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            input_latency_ms: 0.0,
            layout_time_ms: 0.0,
            render_time_ms: 0.0,
            memory_usage_mb: 0.0,
        }
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics with all values.
    pub fn new(
        input_latency_ms: f64,
        layout_time_ms: f64,
        render_time_ms: f64,
        memory_usage_mb: f64,
    ) -> Self {
        Self {
            input_latency_ms,
            layout_time_ms,
            render_time_ms,
            memory_usage_mb,
        }
    }

    /// Calculate total frame time (layout + render).
    pub fn total_frame_time_ms(&self) -> f64 {
        self.layout_time_ms + self.render_time_ms
    }

    /// Check if metrics indicate good performance (60 FPS target).
    pub fn is_within_budget(&self) -> bool {
        // 60 FPS = 16.67ms per frame
        self.total_frame_time_ms() <= 16.67 && self.input_latency_ms <= 100.0
    }
}

/// Collector for performance metrics samples.
#[derive(Debug)]
pub struct MetricsCollector {
    /// Circular buffer of metric samples
    samples: VecDeque<PerformanceMetrics>,
    /// Maximum number of samples to retain
    max_samples: usize,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl MetricsCollector {
    /// Create a new metrics collector with specified capacity.
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Record a new metrics sample.
    pub fn record(&mut self, metrics: PerformanceMetrics) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(metrics);
    }

    /// Get the number of recorded samples.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Clear all recorded samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Check if there are any samples.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get the most recent metrics sample.
    pub fn get_latest(&self) -> Option<&PerformanceMetrics> {
        self.samples.back()
    }

    /// Calculate average metrics across all samples.
    pub fn get_average(&self) -> PerformanceMetrics {
        if self.samples.is_empty() {
            return PerformanceMetrics::default();
        }

        let count = self.samples.len() as f64;
        let mut sum = PerformanceMetrics::default();

        for sample in &self.samples {
            sum.input_latency_ms += sample.input_latency_ms;
            sum.layout_time_ms += sample.layout_time_ms;
            sum.render_time_ms += sample.render_time_ms;
            sum.memory_usage_mb += sample.memory_usage_mb;
        }

        PerformanceMetrics {
            input_latency_ms: sum.input_latency_ms / count,
            layout_time_ms: sum.layout_time_ms / count,
            render_time_ms: sum.render_time_ms / count,
            memory_usage_mb: sum.memory_usage_mb / count,
        }
    }

    /// Get a specific percentile of metrics.
    ///
    /// Percentile should be between 0.0 and 100.0.
    pub fn get_percentile(&self, percentile: f64) -> PerformanceMetrics {
        if self.samples.is_empty() {
            return PerformanceMetrics::default();
        }

        let percentile = percentile.clamp(0.0, 100.0);
        let index = ((percentile / 100.0) * (self.samples.len() - 1) as f64).round() as usize;

        // Sort each metric independently
        let mut input_latencies: Vec<f64> = self.samples.iter().map(|s| s.input_latency_ms).collect();
        let mut layout_times: Vec<f64> = self.samples.iter().map(|s| s.layout_time_ms).collect();
        let mut render_times: Vec<f64> = self.samples.iter().map(|s| s.render_time_ms).collect();
        let mut memory_usages: Vec<f64> = self.samples.iter().map(|s| s.memory_usage_mb).collect();

        input_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        layout_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        render_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        memory_usages.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        PerformanceMetrics {
            input_latency_ms: input_latencies[index],
            layout_time_ms: layout_times[index],
            render_time_ms: render_times[index],
            memory_usage_mb: memory_usages[index],
        }
    }

    /// Get minimum values across all samples.
    pub fn get_min(&self) -> PerformanceMetrics {
        self.get_percentile(0.0)
    }

    /// Get maximum values across all samples.
    pub fn get_max(&self) -> PerformanceMetrics {
        self.get_percentile(100.0)
    }

    /// Get the 50th percentile (median).
    pub fn get_median(&self) -> PerformanceMetrics {
        self.get_percentile(50.0)
    }

    /// Get the 95th percentile (for SLA monitoring).
    pub fn get_p95(&self) -> PerformanceMetrics {
        self.get_percentile(95.0)
    }

    /// Get the 99th percentile (for outlier detection).
    pub fn get_p99(&self) -> PerformanceMetrics {
        self.get_percentile(99.0)
    }

    /// Generate a summary report.
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            sample_count: self.sample_count(),
            average: self.get_average(),
            median: self.get_median(),
            p95: self.get_p95(),
            p99: self.get_p99(),
            min: self.get_min(),
            max: self.get_max(),
        }
    }
}

/// Summary of collected metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Number of samples collected
    pub sample_count: usize,
    /// Average metrics
    pub average: PerformanceMetrics,
    /// Median (50th percentile) metrics
    pub median: PerformanceMetrics,
    /// 95th percentile metrics
    pub p95: PerformanceMetrics,
    /// 99th percentile metrics
    pub p99: PerformanceMetrics,
    /// Minimum values
    pub min: PerformanceMetrics,
    /// Maximum values
    pub max: PerformanceMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.input_latency_ms, 0.0);
        assert_eq!(metrics.layout_time_ms, 0.0);
        assert_eq!(metrics.render_time_ms, 0.0);
        assert_eq!(metrics.memory_usage_mb, 0.0);
    }

    #[test]
    fn test_performance_metrics_new() {
        let metrics = PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0);
        assert_eq!(metrics.input_latency_ms, 5.0);
        assert_eq!(metrics.layout_time_ms, 10.0);
        assert_eq!(metrics.render_time_ms, 6.0);
        assert_eq!(metrics.memory_usage_mb, 128.0);
    }

    #[test]
    fn test_total_frame_time() {
        let metrics = PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0);
        assert_eq!(metrics.total_frame_time_ms(), 16.0);
    }

    #[test]
    fn test_is_within_budget_good() {
        let metrics = PerformanceMetrics::new(50.0, 8.0, 6.0, 128.0);
        assert!(metrics.is_within_budget());
    }

    #[test]
    fn test_is_within_budget_slow_frame() {
        let metrics = PerformanceMetrics::new(50.0, 15.0, 10.0, 128.0);
        assert!(!metrics.is_within_budget());
    }

    #[test]
    fn test_is_within_budget_high_latency() {
        let metrics = PerformanceMetrics::new(150.0, 5.0, 5.0, 128.0);
        assert!(!metrics.is_within_budget());
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0);
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: PerformanceMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, deserialized);
    }

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::new(100);
        assert_eq!(collector.sample_count(), 0);
        assert!(collector.is_empty());
    }

    #[test]
    fn test_metrics_collector_record() {
        let mut collector = MetricsCollector::new(100);
        collector.record(PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0));
        assert_eq!(collector.sample_count(), 1);
        assert!(!collector.is_empty());
    }

    #[test]
    fn test_metrics_collector_max_samples() {
        let mut collector = MetricsCollector::new(3);
        for i in 0..5 {
            collector.record(PerformanceMetrics::new(i as f64, 0.0, 0.0, 0.0));
        }
        assert_eq!(collector.sample_count(), 3);
        // Should have samples 2, 3, 4
        assert_eq!(collector.get_min().input_latency_ms, 2.0);
    }

    #[test]
    fn test_metrics_collector_clear() {
        let mut collector = MetricsCollector::new(100);
        collector.record(PerformanceMetrics::default());
        collector.clear();
        assert!(collector.is_empty());
    }

    #[test]
    fn test_metrics_collector_get_latest() {
        let mut collector = MetricsCollector::new(100);
        assert!(collector.get_latest().is_none());

        collector.record(PerformanceMetrics::new(1.0, 0.0, 0.0, 0.0));
        collector.record(PerformanceMetrics::new(2.0, 0.0, 0.0, 0.0));

        let latest = collector.get_latest().unwrap();
        assert_eq!(latest.input_latency_ms, 2.0);
    }

    #[test]
    fn test_metrics_collector_get_average() {
        let mut collector = MetricsCollector::new(100);

        // Empty collector returns default
        let avg = collector.get_average();
        assert_eq!(avg.input_latency_ms, 0.0);

        // Add samples
        collector.record(PerformanceMetrics::new(10.0, 20.0, 30.0, 40.0));
        collector.record(PerformanceMetrics::new(20.0, 40.0, 60.0, 80.0));

        let avg = collector.get_average();
        assert_eq!(avg.input_latency_ms, 15.0);
        assert_eq!(avg.layout_time_ms, 30.0);
        assert_eq!(avg.render_time_ms, 45.0);
        assert_eq!(avg.memory_usage_mb, 60.0);
    }

    #[test]
    fn test_metrics_collector_get_percentile_empty() {
        let collector = MetricsCollector::new(100);
        let p50 = collector.get_percentile(50.0);
        assert_eq!(p50.input_latency_ms, 0.0);
    }

    #[test]
    fn test_metrics_collector_get_percentile() {
        let mut collector = MetricsCollector::new(100);

        // Add samples with known values
        for i in 1..=10 {
            collector.record(PerformanceMetrics::new(i as f64 * 10.0, 0.0, 0.0, 0.0));
        }

        // Minimum (0th percentile)
        let min = collector.get_percentile(0.0);
        assert_eq!(min.input_latency_ms, 10.0);

        // Maximum (100th percentile)
        let max = collector.get_percentile(100.0);
        assert_eq!(max.input_latency_ms, 100.0);

        // Clamping out of range
        let clamped = collector.get_percentile(150.0);
        assert_eq!(clamped.input_latency_ms, 100.0);
    }

    #[test]
    fn test_metrics_collector_percentile_methods() {
        let mut collector = MetricsCollector::new(100);

        for i in 1..=100 {
            collector.record(PerformanceMetrics::new(i as f64, 0.0, 0.0, 0.0));
        }

        let min = collector.get_min();
        assert_eq!(min.input_latency_ms, 1.0);

        let max = collector.get_max();
        assert_eq!(max.input_latency_ms, 100.0);

        // Median should be around 50
        let median = collector.get_median();
        assert!(median.input_latency_ms >= 49.0 && median.input_latency_ms <= 51.0);

        // P95 should be around 95
        let p95 = collector.get_p95();
        assert!(p95.input_latency_ms >= 94.0 && p95.input_latency_ms <= 96.0);

        // P99 should be around 99
        let p99 = collector.get_p99();
        assert!(p99.input_latency_ms >= 98.0 && p99.input_latency_ms <= 100.0);
    }

    #[test]
    fn test_metrics_collector_summary() {
        let mut collector = MetricsCollector::new(100);

        for i in 1..=10 {
            collector.record(PerformanceMetrics::new(i as f64 * 10.0, 0.0, 0.0, 0.0));
        }

        let summary = collector.summary();
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.average.input_latency_ms, 55.0);
        assert_eq!(summary.min.input_latency_ms, 10.0);
        assert_eq!(summary.max.input_latency_ms, 100.0);
    }

    #[test]
    fn test_metrics_summary_serialization() {
        let mut collector = MetricsCollector::new(100);
        collector.record(PerformanceMetrics::new(5.0, 10.0, 6.0, 128.0));

        let summary = collector.summary();
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: MetricsSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(summary.sample_count, deserialized.sample_count);
        assert_eq!(summary.average, deserialized.average);
    }
}
