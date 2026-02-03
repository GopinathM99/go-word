//! Performance Telemetry and Profiling
//!
//! This crate provides performance measurement infrastructure for the word processor:
//! - Timing utilities with RAII-based scope timing
//! - Metrics collection for commands, layout, rendering, and input latency
//! - Performance budgets with violation detection
//!
//! # Feature Flags
//!
//! - `telemetry` (default): Enables performance data collection
//! - `profiling`: Enables detailed profiling with additional overhead
//!
//! # Example
//!
//! ```rust
//! use perf::{time_scope, PerfTimer, global_metrics};
//!
//! fn do_layout() {
//!     time_scope!("layout");
//!     // ... layout code ...
//! }
//!
//! // Check performance
//! let metrics_guard = global_metrics().lock().unwrap();
//! let summary = metrics_guard.summary();
//! ```

mod timing;
mod metrics;
mod budget;

pub use timing::*;
pub use metrics::*;
pub use budget::*;

/// Re-export for convenience
pub use std::time::{Duration, Instant};
