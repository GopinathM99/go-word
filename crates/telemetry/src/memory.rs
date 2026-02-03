//! Memory Profiler Module
//!
//! Provides memory profiling capabilities for tracking allocations,
//! detecting memory leaks, and monitoring memory growth.
//!
//! # Example
//!
//! ```rust
//! use telemetry::memory::{MemoryProfiler, AllocationInfo};
//!
//! let mut profiler = MemoryProfiler::new();
//!
//! // Take periodic snapshots
//! profiler.take_snapshot();
//! // ... perform operations ...
//! profiler.take_snapshot();
//!
//! // Compare snapshots to detect leaks
//! if let Some(leaks) = profiler.detect_leaks() {
//!     for leak in leaks {
//!         println!("Potential leak in {}: {} bytes", leak.component, leak.bytes_leaked);
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// Allocation Info
// =============================================================================

/// Information about allocations in a specific category/component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllocationInfo {
    /// Number of bytes currently allocated
    pub bytes: usize,
    /// Number of individual allocations
    pub count: usize,
    /// Peak bytes allocated
    pub peak_bytes: usize,
    /// Total bytes allocated over time (cumulative)
    pub total_allocated: usize,
    /// Total bytes freed over time (cumulative)
    pub total_freed: usize,
}

impl Default for AllocationInfo {
    fn default() -> Self {
        Self {
            bytes: 0,
            count: 0,
            peak_bytes: 0,
            total_allocated: 0,
            total_freed: 0,
        }
    }
}

impl AllocationInfo {
    /// Create new allocation info with initial values.
    pub fn new(bytes: usize, count: usize) -> Self {
        Self {
            bytes,
            count,
            peak_bytes: bytes,
            total_allocated: bytes,
            total_freed: 0,
        }
    }

    /// Record an allocation.
    pub fn allocate(&mut self, bytes: usize) {
        self.bytes += bytes;
        self.count += 1;
        self.total_allocated += bytes;
        if self.bytes > self.peak_bytes {
            self.peak_bytes = self.bytes;
        }
    }

    /// Record a deallocation.
    pub fn deallocate(&mut self, bytes: usize) {
        self.bytes = self.bytes.saturating_sub(bytes);
        self.count = self.count.saturating_sub(1);
        self.total_freed += bytes;
    }

    /// Calculate net allocation (allocated - freed).
    pub fn net_allocation(&self) -> isize {
        self.total_allocated as isize - self.total_freed as isize
    }
}

// =============================================================================
// Memory Snapshot
// =============================================================================

/// A snapshot of memory state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Unique identifier for this snapshot
    pub id: u64,
    /// When the snapshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Time since profiling started
    #[serde(with = "duration_serde")]
    pub elapsed: Duration,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Allocations by component/category
    pub allocations: HashMap<String, AllocationInfo>,
    /// System memory information (if available)
    pub system_memory: Option<SystemMemoryInfo>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl MemorySnapshot {
    /// Create a new memory snapshot.
    pub fn new(id: u64, elapsed: Duration) -> Self {
        Self {
            id,
            timestamp: chrono::Utc::now(),
            elapsed,
            total_bytes: 0,
            allocations: HashMap::new(),
            system_memory: None,
            metadata: HashMap::new(),
        }
    }

    /// Add allocation info for a component.
    pub fn add_allocation(&mut self, component: impl Into<String>, info: AllocationInfo) {
        let component = component.into();
        self.total_bytes += info.bytes;
        self.allocations.insert(component, info);
    }

    /// Get allocation info for a component.
    pub fn get_allocation(&self, component: &str) -> Option<&AllocationInfo> {
        self.allocations.get(component)
    }

    /// Get number of tracked components.
    pub fn component_count(&self) -> usize {
        self.allocations.len()
    }

    /// Add metadata to the snapshot.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// System-level memory information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemoryInfo {
    /// Total system memory in bytes
    pub total_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Memory used by this process
    pub process_bytes: u64,
}

impl SystemMemoryInfo {
    /// Create mock system memory info (for testing).
    pub fn mock(total: u64, available: u64, process: u64) -> Self {
        Self {
            total_bytes: total,
            available_bytes: available,
            process_bytes: process,
        }
    }

    /// Get memory usage as a percentage of total.
    pub fn usage_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        ((self.total_bytes - self.available_bytes) as f64 / self.total_bytes as f64) * 100.0
    }

    /// Get process memory as percentage of total.
    pub fn process_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.process_bytes as f64 / self.total_bytes as f64) * 100.0
    }
}

// =============================================================================
// Snapshot Comparison
// =============================================================================

/// Result of comparing two memory snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotComparison {
    /// ID of the first (earlier) snapshot
    pub from_id: u64,
    /// ID of the second (later) snapshot
    pub to_id: u64,
    /// Time between snapshots
    #[serde(with = "duration_serde")]
    pub time_delta: Duration,
    /// Change in total bytes
    pub total_bytes_delta: isize,
    /// Per-component deltas
    pub component_deltas: HashMap<String, AllocationDelta>,
    /// Components that appeared in the second snapshot
    pub new_components: Vec<String>,
    /// Components that disappeared from the second snapshot
    pub removed_components: Vec<String>,
}

/// Change in allocation for a single component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDelta {
    /// Change in bytes
    pub bytes_delta: isize,
    /// Change in allocation count
    pub count_delta: isize,
    /// Bytes in first snapshot
    pub from_bytes: usize,
    /// Bytes in second snapshot
    pub to_bytes: usize,
}

impl SnapshotComparison {
    /// Check if memory grew between snapshots.
    pub fn memory_grew(&self) -> bool {
        self.total_bytes_delta > 0
    }

    /// Get components with memory growth.
    pub fn growing_components(&self) -> Vec<(&String, &AllocationDelta)> {
        self.component_deltas
            .iter()
            .filter(|(_, d)| d.bytes_delta > 0)
            .collect()
    }

    /// Get memory growth rate in bytes per second.
    pub fn growth_rate(&self) -> f64 {
        let secs = self.time_delta.as_secs_f64();
        if secs == 0.0 {
            return 0.0;
        }
        self.total_bytes_delta as f64 / secs
    }
}

// =============================================================================
// Memory Leak Detection
// =============================================================================

/// Information about a potential memory leak.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakInfo {
    /// Component/category where leak was detected
    pub component: String,
    /// Estimated bytes leaked
    pub bytes_leaked: usize,
    /// Number of snapshots showing continuous growth
    pub growth_count: usize,
    /// Average growth rate (bytes per second)
    pub growth_rate: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

// =============================================================================
// Memory Profiler
// =============================================================================

/// Memory profiler for tracking allocations and detecting leaks.
#[derive(Debug)]
pub struct MemoryProfiler {
    /// Collected memory snapshots
    snapshots: Vec<MemorySnapshot>,
    /// Maximum number of snapshots to retain
    max_snapshots: usize,
    /// When profiling started
    start_time: Instant,
    /// Current allocation state (for incremental updates)
    current_allocations: HashMap<String, AllocationInfo>,
    /// Next snapshot ID
    next_id: u64,
    /// Minimum growth count for leak detection
    leak_detection_threshold: usize,
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryProfiler {
    /// Create a new memory profiler.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 100,
            start_time: Instant::now(),
            current_allocations: HashMap::new(),
            next_id: 0,
            leak_detection_threshold: 3,
        }
    }

    /// Create a profiler with custom settings.
    pub fn with_settings(max_snapshots: usize, leak_threshold: usize) -> Self {
        Self {
            max_snapshots,
            leak_detection_threshold: leak_threshold,
            ..Self::new()
        }
    }

    /// Record an allocation in a component.
    pub fn record_allocation(&mut self, component: impl Into<String>, bytes: usize) {
        let component = component.into();
        self.current_allocations
            .entry(component)
            .or_default()
            .allocate(bytes);
    }

    /// Record a deallocation in a component.
    pub fn record_deallocation(&mut self, component: impl Into<String>, bytes: usize) {
        let component = component.into();
        if let Some(info) = self.current_allocations.get_mut(&component) {
            info.deallocate(bytes);
        }
    }

    /// Set the current allocation state for a component.
    pub fn set_allocation(&mut self, component: impl Into<String>, info: AllocationInfo) {
        self.current_allocations.insert(component.into(), info);
    }

    /// Take a memory snapshot of the current state.
    pub fn take_snapshot(&mut self) -> &MemorySnapshot {
        let id = self.next_id;
        self.next_id += 1;

        let mut snapshot = MemorySnapshot::new(id, self.start_time.elapsed());

        // Copy current allocations to snapshot
        for (component, info) in &self.current_allocations {
            snapshot.add_allocation(component.clone(), info.clone());
        }

        // Enforce max snapshots
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.remove(0);
        }

        self.snapshots.push(snapshot);
        self.snapshots.last().unwrap()
    }

    /// Take a snapshot with system memory information.
    pub fn take_snapshot_with_system(&mut self, system_info: SystemMemoryInfo) -> &MemorySnapshot {
        self.take_snapshot();
        if let Some(snapshot) = self.snapshots.last_mut() {
            snapshot.system_memory = Some(system_info);
        }
        self.snapshots.last().unwrap()
    }

    /// Get all snapshots.
    pub fn get_snapshots(&self) -> &[MemorySnapshot] {
        &self.snapshots
    }

    /// Get the latest snapshot.
    pub fn get_latest(&self) -> Option<&MemorySnapshot> {
        self.snapshots.last()
    }

    /// Get snapshot by ID.
    pub fn get_snapshot(&self, id: u64) -> Option<&MemorySnapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    /// Get snapshot count.
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Compare two snapshots.
    pub fn compare_snapshots(&self, from_id: u64, to_id: u64) -> Option<SnapshotComparison> {
        let from = self.get_snapshot(from_id)?;
        let to = self.get_snapshot(to_id)?;

        let time_delta = to.elapsed.saturating_sub(from.elapsed);
        let total_bytes_delta = to.total_bytes as isize - from.total_bytes as isize;

        let mut component_deltas = HashMap::new();
        let mut all_components: std::collections::HashSet<&String> =
            from.allocations.keys().chain(to.allocations.keys()).collect();

        let mut new_components = Vec::new();
        let mut removed_components = Vec::new();

        for component in all_components.drain() {
            let from_info = from.allocations.get(component);
            let to_info = to.allocations.get(component);

            match (from_info, to_info) {
                (Some(f), Some(t)) => {
                    component_deltas.insert(
                        component.clone(),
                        AllocationDelta {
                            bytes_delta: t.bytes as isize - f.bytes as isize,
                            count_delta: t.count as isize - f.count as isize,
                            from_bytes: f.bytes,
                            to_bytes: t.bytes,
                        },
                    );
                }
                (None, Some(t)) => {
                    new_components.push(component.clone());
                    component_deltas.insert(
                        component.clone(),
                        AllocationDelta {
                            bytes_delta: t.bytes as isize,
                            count_delta: t.count as isize,
                            from_bytes: 0,
                            to_bytes: t.bytes,
                        },
                    );
                }
                (Some(f), None) => {
                    removed_components.push(component.clone());
                    component_deltas.insert(
                        component.clone(),
                        AllocationDelta {
                            bytes_delta: -(f.bytes as isize),
                            count_delta: -(f.count as isize),
                            from_bytes: f.bytes,
                            to_bytes: 0,
                        },
                    );
                }
                (None, None) => unreachable!(),
            }
        }

        Some(SnapshotComparison {
            from_id,
            to_id,
            time_delta,
            total_bytes_delta,
            component_deltas,
            new_components,
            removed_components,
        })
    }

    /// Compare latest two snapshots.
    pub fn compare_latest(&self) -> Option<SnapshotComparison> {
        if self.snapshots.len() < 2 {
            return None;
        }
        let len = self.snapshots.len();
        self.compare_snapshots(
            self.snapshots[len - 2].id,
            self.snapshots[len - 1].id,
        )
    }

    /// Detect potential memory leaks based on snapshot history.
    pub fn detect_leaks(&self) -> Option<Vec<LeakInfo>> {
        if self.snapshots.len() < self.leak_detection_threshold {
            return None;
        }

        let mut leaks = Vec::new();

        // Get all components that appear in any snapshot
        let mut all_components: std::collections::HashSet<String> = std::collections::HashSet::new();
        for snapshot in &self.snapshots {
            for component in snapshot.allocations.keys() {
                all_components.insert(component.clone());
            }
        }

        // Check each component for continuous growth
        for component in all_components {
            let values: Vec<Option<usize>> = self
                .snapshots
                .iter()
                .map(|s| s.allocations.get(&component).map(|a| a.bytes))
                .collect();

            // Count consecutive growth
            let mut growth_count = 0;
            let mut prev_value: Option<usize> = None;
            let mut total_growth = 0isize;

            for value in &values {
                match (prev_value, value) {
                    (Some(prev), Some(curr)) if *curr > prev => {
                        growth_count += 1;
                        total_growth += (*curr - prev) as isize;
                    }
                    (Some(_), Some(_)) => {
                        // Reset if not growing
                        if growth_count < self.leak_detection_threshold {
                            growth_count = 0;
                            total_growth = 0;
                        }
                    }
                    _ => {}
                }
                prev_value = *value;
            }

            // If we have enough consecutive growth, it might be a leak
            if growth_count >= self.leak_detection_threshold {
                let time_span = self.snapshots.last().unwrap().elapsed
                    - self.snapshots[self.snapshots.len() - growth_count - 1].elapsed;
                let growth_rate = total_growth as f64 / time_span.as_secs_f64();

                // Calculate confidence based on consistency
                let confidence = (growth_count as f64 / self.snapshots.len() as f64)
                    .min(1.0);

                leaks.push(LeakInfo {
                    component,
                    bytes_leaked: total_growth.max(0) as usize,
                    growth_count,
                    growth_rate,
                    confidence,
                });
            }
        }

        if leaks.is_empty() {
            None
        } else {
            // Sort by bytes leaked (descending)
            leaks.sort_by(|a, b| b.bytes_leaked.cmp(&a.bytes_leaked));
            Some(leaks)
        }
    }

    /// Get current total memory usage.
    pub fn current_total_bytes(&self) -> usize {
        self.current_allocations.values().map(|a| a.bytes).sum()
    }

    /// Get current allocations.
    pub fn current_allocations(&self) -> &HashMap<String, AllocationInfo> {
        &self.current_allocations
    }

    /// Clear all snapshots.
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    /// Reset the profiler.
    pub fn reset(&mut self) {
        self.snapshots.clear();
        self.current_allocations.clear();
        self.start_time = Instant::now();
        self.next_id = 0;
    }

    /// Generate a memory report.
    pub fn generate_report(&self) -> MemoryReport {
        let latest = self.get_latest();
        let comparison = self.compare_latest();
        let leaks = self.detect_leaks();

        MemoryReport {
            snapshot_count: self.snapshot_count(),
            current_total_bytes: self.current_total_bytes(),
            peak_bytes: self.snapshots.iter().map(|s| s.total_bytes).max().unwrap_or(0),
            component_count: self.current_allocations.len(),
            latest_snapshot: latest.cloned(),
            latest_comparison: comparison,
            potential_leaks: leaks.unwrap_or_default(),
        }
    }
}

/// Summary memory report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReport {
    /// Number of snapshots taken
    pub snapshot_count: usize,
    /// Current total bytes allocated
    pub current_total_bytes: usize,
    /// Peak bytes allocated
    pub peak_bytes: usize,
    /// Number of tracked components
    pub component_count: usize,
    /// Latest snapshot
    pub latest_snapshot: Option<MemorySnapshot>,
    /// Comparison with previous snapshot
    pub latest_comparison: Option<SnapshotComparison>,
    /// Potential memory leaks
    pub potential_leaks: Vec<LeakInfo>,
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

    #[test]
    fn test_allocation_info_default() {
        let info = AllocationInfo::default();
        assert_eq!(info.bytes, 0);
        assert_eq!(info.count, 0);
        assert_eq!(info.peak_bytes, 0);
    }

    #[test]
    fn test_allocation_info_new() {
        let info = AllocationInfo::new(1024, 10);
        assert_eq!(info.bytes, 1024);
        assert_eq!(info.count, 10);
        assert_eq!(info.peak_bytes, 1024);
    }

    #[test]
    fn test_allocation_info_allocate() {
        let mut info = AllocationInfo::default();
        info.allocate(100);
        info.allocate(200);

        assert_eq!(info.bytes, 300);
        assert_eq!(info.count, 2);
        assert_eq!(info.peak_bytes, 300);
        assert_eq!(info.total_allocated, 300);
    }

    #[test]
    fn test_allocation_info_deallocate() {
        let mut info = AllocationInfo::new(300, 3);
        info.deallocate(100);

        assert_eq!(info.bytes, 200);
        assert_eq!(info.count, 2);
        assert_eq!(info.total_freed, 100);
    }

    #[test]
    fn test_allocation_info_peak() {
        let mut info = AllocationInfo::default();
        info.allocate(500);
        info.deallocate(300);
        info.allocate(100);

        assert_eq!(info.bytes, 300);
        assert_eq!(info.peak_bytes, 500);
    }

    #[test]
    fn test_allocation_info_net_allocation() {
        let mut info = AllocationInfo::default();
        info.allocate(1000);
        info.deallocate(300);

        assert_eq!(info.net_allocation(), 700);
    }

    #[test]
    fn test_memory_snapshot_new() {
        let snapshot = MemorySnapshot::new(1, Duration::from_secs(10));
        assert_eq!(snapshot.id, 1);
        assert_eq!(snapshot.total_bytes, 0);
        assert!(snapshot.allocations.is_empty());
    }

    #[test]
    fn test_memory_snapshot_add_allocation() {
        let mut snapshot = MemorySnapshot::new(1, Duration::ZERO);
        snapshot.add_allocation("component1", AllocationInfo::new(100, 1));
        snapshot.add_allocation("component2", AllocationInfo::new(200, 2));

        assert_eq!(snapshot.total_bytes, 300);
        assert_eq!(snapshot.component_count(), 2);
    }

    #[test]
    fn test_memory_snapshot_with_metadata() {
        let snapshot = MemorySnapshot::new(1, Duration::ZERO)
            .with_metadata("test_key", "test_value");

        assert_eq!(snapshot.metadata.get("test_key"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_system_memory_info() {
        let info = SystemMemoryInfo::mock(16_000_000_000, 8_000_000_000, 500_000_000);
        assert_eq!(info.usage_percent(), 50.0);
        assert!(info.process_percent() < 5.0);
    }

    #[test]
    fn test_profiler_new() {
        let profiler = MemoryProfiler::new();
        assert_eq!(profiler.snapshot_count(), 0);
        assert_eq!(profiler.current_total_bytes(), 0);
    }

    #[test]
    fn test_profiler_record_allocation() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_allocation("test_component", 1024);

        assert_eq!(profiler.current_total_bytes(), 1024);
        let alloc = profiler.current_allocations().get("test_component").unwrap();
        assert_eq!(alloc.bytes, 1024);
        assert_eq!(alloc.count, 1);
    }

    #[test]
    fn test_profiler_record_deallocation() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_allocation("component", 1000);
        profiler.record_deallocation("component", 400);

        assert_eq!(profiler.current_total_bytes(), 600);
    }

    #[test]
    fn test_profiler_take_snapshot() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_allocation("component", 500);

        let snapshot = profiler.take_snapshot();
        assert_eq!(snapshot.id, 0);
        assert_eq!(snapshot.total_bytes, 500);
    }

    #[test]
    fn test_profiler_multiple_snapshots() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation("component", 100);
        profiler.take_snapshot();

        profiler.record_allocation("component", 100);
        profiler.take_snapshot();

        assert_eq!(profiler.snapshot_count(), 2);
        assert_eq!(profiler.get_snapshot(0).unwrap().total_bytes, 100);
        assert_eq!(profiler.get_snapshot(1).unwrap().total_bytes, 200);
    }

    #[test]
    fn test_profiler_max_snapshots() {
        let mut profiler = MemoryProfiler::with_settings(3, 2);

        for _ in 0..5 {
            profiler.record_allocation("component", 100);
            profiler.take_snapshot();
        }

        assert_eq!(profiler.snapshot_count(), 3);
        // First two snapshots should have been removed
        assert!(profiler.get_snapshot(0).is_none());
        assert!(profiler.get_snapshot(1).is_none());
        assert!(profiler.get_snapshot(2).is_some());
    }

    #[test]
    fn test_profiler_compare_snapshots() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation("component", 100);
        profiler.take_snapshot();

        profiler.record_allocation("component", 50);
        profiler.take_snapshot();

        let comparison = profiler.compare_snapshots(0, 1).unwrap();
        assert_eq!(comparison.from_id, 0);
        assert_eq!(comparison.to_id, 1);
        assert_eq!(comparison.total_bytes_delta, 50);
        assert!(comparison.memory_grew());
    }

    #[test]
    fn test_profiler_compare_latest() {
        let mut profiler = MemoryProfiler::new();

        profiler.set_allocation("comp", AllocationInfo::new(1000, 1));
        profiler.take_snapshot();

        profiler.set_allocation("comp", AllocationInfo::new(2000, 2));
        profiler.take_snapshot();

        let comparison = profiler.compare_latest().unwrap();
        assert_eq!(comparison.total_bytes_delta, 1000);
    }

    #[test]
    fn test_profiler_detect_leaks() {
        let mut profiler = MemoryProfiler::with_settings(100, 3);

        // Simulate a memory leak - continuous growth
        for i in 0..5 {
            profiler.set_allocation("leaky_component", AllocationInfo::new(1000 * (i + 1), 1));
            profiler.take_snapshot();
        }

        let leaks = profiler.detect_leaks();
        assert!(leaks.is_some());
        let leaks = leaks.unwrap();
        assert!(!leaks.is_empty());
        assert_eq!(leaks[0].component, "leaky_component");
    }

    #[test]
    fn test_profiler_no_leaks() {
        let mut profiler = MemoryProfiler::with_settings(100, 3);

        // Stable memory usage
        for _ in 0..5 {
            profiler.set_allocation("stable_component", AllocationInfo::new(1000, 1));
            profiler.take_snapshot();
        }

        let leaks = profiler.detect_leaks();
        assert!(leaks.is_none());
    }

    #[test]
    fn test_profiler_generate_report() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_allocation("component", 500);
        profiler.take_snapshot();

        let report = profiler.generate_report();
        assert_eq!(report.snapshot_count, 1);
        assert_eq!(report.current_total_bytes, 500);
        assert_eq!(report.component_count, 1);
    }

    #[test]
    fn test_profiler_reset() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_allocation("component", 500);
        profiler.take_snapshot();

        profiler.reset();

        assert_eq!(profiler.snapshot_count(), 0);
        assert_eq!(profiler.current_total_bytes(), 0);
    }

    #[test]
    fn test_snapshot_comparison_growing_components() {
        let mut profiler = MemoryProfiler::new();

        profiler.set_allocation("growing", AllocationInfo::new(100, 1));
        profiler.set_allocation("shrinking", AllocationInfo::new(200, 1));
        profiler.take_snapshot();

        profiler.set_allocation("growing", AllocationInfo::new(200, 2));
        profiler.set_allocation("shrinking", AllocationInfo::new(100, 1));
        profiler.take_snapshot();

        let comparison = profiler.compare_latest().unwrap();
        let growing = comparison.growing_components();

        assert_eq!(growing.len(), 1);
        assert_eq!(growing[0].0, "growing");
    }

    #[test]
    fn test_snapshot_serialization() {
        let mut snapshot = MemorySnapshot::new(1, Duration::from_millis(500));
        snapshot.add_allocation("test", AllocationInfo::new(100, 1));

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: MemorySnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.total_bytes, 100);
    }

    #[test]
    fn test_memory_report_serialization() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_allocation("test", 100);
        profiler.take_snapshot();

        let report = profiler.generate_report();
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: MemoryReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.current_total_bytes, 100);
    }
}
