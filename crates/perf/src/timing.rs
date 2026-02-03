//! Timing utilities for performance measurement

use std::time::Instant;

/// A timer that measures elapsed time from creation to drop.
///
/// When the `telemetry` feature is enabled, the timer will log the elapsed
/// time when dropped. This is useful for RAII-based scope timing.
///
/// # Example
///
/// ```rust
/// use perf::PerfTimer;
///
/// fn process_paragraph() {
///     let _timer = PerfTimer::new("paragraph_processing");
///     // ... processing code ...
///     // Timer automatically logs on drop
/// }
/// ```
pub struct PerfTimer {
    name: &'static str,
    start: Instant,
    #[cfg(feature = "telemetry")]
    category: TimerCategory,
}

/// Category of operation being timed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCategory {
    /// Command execution timing
    Command,
    /// Layout calculation timing
    Layout,
    /// Rendering timing
    Render,
    /// Input handling timing
    Input,
    /// General/uncategorized timing
    General,
}

impl PerfTimer {
    /// Create a new timer with the given name.
    ///
    /// The timer starts immediately upon creation.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            #[cfg(feature = "telemetry")]
            category: TimerCategory::General,
        }
    }

    /// Create a new timer with a specific category.
    ///
    /// The category determines how the timing is recorded in metrics.
    #[inline]
    #[cfg(feature = "telemetry")]
    pub fn with_category(name: &'static str, category: TimerCategory) -> Self {
        Self {
            name,
            start: Instant::now(),
            category,
        }
    }

    /// Create a new timer with a specific category (no-op when telemetry is disabled).
    #[inline]
    #[cfg(not(feature = "telemetry"))]
    pub fn with_category(name: &'static str, _category: TimerCategory) -> Self {
        Self::new(name)
    }

    /// Create a timer for command execution.
    #[inline]
    pub fn command(name: &'static str) -> Self {
        Self::with_category(name, TimerCategory::Command)
    }

    /// Create a timer for layout operations.
    #[inline]
    pub fn layout(name: &'static str) -> Self {
        Self::with_category(name, TimerCategory::Layout)
    }

    /// Create a timer for render operations.
    #[inline]
    pub fn render(name: &'static str) -> Self {
        Self::with_category(name, TimerCategory::Render)
    }

    /// Create a timer for input handling.
    #[inline]
    pub fn input(name: &'static str) -> Self {
        Self::with_category(name, TimerCategory::Input)
    }

    /// Get the elapsed time in milliseconds.
    #[inline]
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Get the elapsed time in microseconds.
    #[inline]
    pub fn elapsed_us(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1_000_000.0
    }

    /// Get the elapsed duration.
    #[inline]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    /// Get the name of this timer.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get the start instant.
    #[inline]
    pub fn start(&self) -> Instant {
        self.start
    }

    /// Reset the timer to start from now.
    #[inline]
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Stop the timer and return the elapsed milliseconds without logging.
    ///
    /// This consumes the timer, preventing the drop implementation from running.
    #[inline]
    pub fn stop(self) -> f64 {
        let elapsed = self.elapsed_ms();
        std::mem::forget(self); // Prevent drop from running
        elapsed
    }

    /// Stop and record the timing to the global metrics.
    #[cfg(feature = "telemetry")]
    pub fn stop_and_record(self) {
        let elapsed = self.elapsed_ms();
        let category = self.category;
        let name = self.name;
        std::mem::forget(self); // Prevent double recording

        if let Ok(mut metrics) = crate::global_metrics().lock() {
            metrics.record_timing(name, elapsed, category);
        }
    }

    /// Stop and record (no-op when telemetry is disabled).
    #[cfg(not(feature = "telemetry"))]
    #[inline]
    pub fn stop_and_record(self) {
        let _ = self.stop();
    }
}

#[cfg(feature = "telemetry")]
impl Drop for PerfTimer {
    fn drop(&mut self) {
        let elapsed_ms = self.elapsed_ms();

        // Log the timing
        tracing::trace!(
            target: "perf",
            name = self.name,
            elapsed_ms = elapsed_ms,
            "timer completed"
        );

        // Record to global metrics
        if let Ok(mut metrics) = crate::global_metrics().lock() {
            metrics.record_timing(self.name, elapsed_ms, self.category);
        }
    }
}

#[cfg(not(feature = "telemetry"))]
impl Drop for PerfTimer {
    fn drop(&mut self) {
        // No-op when telemetry is disabled
    }
}

/// Macro for easy scope timing.
///
/// Creates a timer that automatically logs when the scope ends.
///
/// # Example
///
/// ```rust
/// use perf::time_scope;
///
/// fn do_work() {
///     time_scope!("work");
///     // ... work ...
/// } // Timer logs here
/// ```
#[macro_export]
macro_rules! time_scope {
    ($name:expr) => {
        let _timer = $crate::PerfTimer::new($name);
    };
    ($name:expr, $category:expr) => {
        let _timer = $crate::PerfTimer::with_category($name, $category);
    };
}

/// Macro for timing command execution.
#[macro_export]
macro_rules! time_command {
    ($name:expr) => {
        let _timer = $crate::PerfTimer::command($name);
    };
}

/// Macro for timing layout operations.
#[macro_export]
macro_rules! time_layout {
    ($name:expr) => {
        let _timer = $crate::PerfTimer::layout($name);
    };
}

/// Macro for timing render operations.
#[macro_export]
macro_rules! time_render {
    ($name:expr) => {
        let _timer = $crate::PerfTimer::render($name);
    };
}

/// Macro for timing input handling.
#[macro_export]
macro_rules! time_input {
    ($name:expr) => {
        let _timer = $crate::PerfTimer::input($name);
    };
}

/// A guard that measures time between creation and a checkpoint.
///
/// Unlike `PerfTimer`, this doesn't automatically record on drop,
/// making it suitable for measuring specific intervals manually.
pub struct Stopwatch {
    start: Instant,
    checkpoints: Vec<(&'static str, std::time::Duration)>,
}

impl Stopwatch {
    /// Create a new stopwatch.
    #[inline]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    /// Record a checkpoint with the given name.
    #[inline]
    pub fn checkpoint(&mut self, name: &'static str) {
        self.checkpoints.push((name, self.start.elapsed()));
    }

    /// Get the total elapsed time.
    #[inline]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    /// Get the total elapsed time in milliseconds.
    #[inline]
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Get all checkpoints with their times.
    #[inline]
    pub fn checkpoints(&self) -> &[(&'static str, std::time::Duration)] {
        &self.checkpoints
    }

    /// Get the time between two consecutive checkpoints.
    ///
    /// Returns `None` if the checkpoint index is out of bounds.
    pub fn interval(&self, index: usize) -> Option<std::time::Duration> {
        if index >= self.checkpoints.len() {
            return None;
        }

        let end = self.checkpoints[index].1;
        let start = if index == 0 {
            std::time::Duration::ZERO
        } else {
            self.checkpoints[index - 1].1
        };

        Some(end - start)
    }

    /// Reset the stopwatch.
    #[inline]
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.checkpoints.clear();
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_perf_timer_elapsed() {
        let timer = PerfTimer::new("test");
        sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 9.0, "elapsed should be at least 9ms, got {}", elapsed);
    }

    #[test]
    fn test_perf_timer_stop() {
        let timer = PerfTimer::new("test");
        sleep(Duration::from_millis(5));
        let elapsed = timer.stop();
        assert!(elapsed >= 4.0, "elapsed should be at least 4ms, got {}", elapsed);
    }

    #[test]
    fn test_stopwatch_checkpoints() {
        let mut sw = Stopwatch::new();
        sleep(Duration::from_millis(5));
        sw.checkpoint("first");
        sleep(Duration::from_millis(5));
        sw.checkpoint("second");

        assert_eq!(sw.checkpoints().len(), 2);
        assert_eq!(sw.checkpoints()[0].0, "first");
        assert_eq!(sw.checkpoints()[1].0, "second");

        // Second checkpoint should be after first
        assert!(sw.checkpoints()[1].1 > sw.checkpoints()[0].1);
    }

    #[test]
    fn test_stopwatch_interval() {
        let mut sw = Stopwatch::new();
        sleep(Duration::from_millis(5));
        sw.checkpoint("first");
        sleep(Duration::from_millis(10));
        sw.checkpoint("second");

        let first_interval = sw.interval(0).unwrap();
        let second_interval = sw.interval(1).unwrap();

        assert!(first_interval.as_millis() >= 4);
        assert!(second_interval.as_millis() >= 9);
    }

    #[test]
    fn test_timer_category() {
        let timer = PerfTimer::command("cmd");
        assert_eq!(timer.name(), "cmd");

        let timer = PerfTimer::layout("lay");
        assert_eq!(timer.name(), "lay");

        let timer = PerfTimer::render("rend");
        assert_eq!(timer.name(), "rend");

        let timer = PerfTimer::input("inp");
        assert_eq!(timer.name(), "inp");
    }
}
