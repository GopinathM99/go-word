//! Performance budgets and violation tracking

use crate::metrics::ViolationSeverity;
use serde::{Deserialize, Serialize};

/// Performance budget configuration.
///
/// Defines acceptable performance thresholds for various operations.
/// When a measurement exceeds its budget, a violation is generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfBudget {
    /// Maximum acceptable input latency in milliseconds.
    ///
    /// This is the time from user input (keystroke, click) to visual feedback.
    /// Target: 50ms for perceived responsiveness.
    pub max_input_latency_ms: f64,

    /// Maximum acceptable layout time per paragraph in milliseconds.
    ///
    /// Time to calculate the layout of a single paragraph.
    pub max_layout_time_ms: f64,

    /// Maximum acceptable render time per frame in milliseconds.
    ///
    /// Target: 16ms for 60fps, 8ms for 120fps.
    pub max_render_time_ms: f64,

    /// Maximum acceptable command execution time in milliseconds.
    ///
    /// Time to execute a single editing command.
    pub max_command_time_ms: f64,
}

impl PerfBudget {
    /// Create a new budget with custom thresholds.
    pub fn new(
        max_input_latency_ms: f64,
        max_layout_time_ms: f64,
        max_render_time_ms: f64,
        max_command_time_ms: f64,
    ) -> Self {
        Self {
            max_input_latency_ms,
            max_layout_time_ms,
            max_render_time_ms,
            max_command_time_ms,
        }
    }

    /// Create a budget optimized for 60fps rendering.
    pub fn for_60fps() -> Self {
        Self {
            max_input_latency_ms: 50.0,
            max_layout_time_ms: 5.0,
            max_render_time_ms: 16.0,
            max_command_time_ms: 100.0,
        }
    }

    /// Create a budget optimized for 120fps rendering.
    pub fn for_120fps() -> Self {
        Self {
            max_input_latency_ms: 30.0,
            max_layout_time_ms: 2.5,
            max_render_time_ms: 8.0,
            max_command_time_ms: 50.0,
        }
    }

    /// Create a relaxed budget for complex documents.
    pub fn relaxed() -> Self {
        Self {
            max_input_latency_ms: 100.0,
            max_layout_time_ms: 20.0,
            max_render_time_ms: 33.0, // 30fps
            max_command_time_ms: 200.0,
        }
    }

    /// Create a strict budget for performance testing.
    pub fn strict() -> Self {
        Self {
            max_input_latency_ms: 16.0,
            max_layout_time_ms: 2.0,
            max_render_time_ms: 8.0,
            max_command_time_ms: 50.0,
        }
    }

    /// Builder method to set input latency budget.
    pub fn with_input_latency(mut self, ms: f64) -> Self {
        self.max_input_latency_ms = ms;
        self
    }

    /// Builder method to set layout time budget.
    pub fn with_layout_time(mut self, ms: f64) -> Self {
        self.max_layout_time_ms = ms;
        self
    }

    /// Builder method to set render time budget.
    pub fn with_render_time(mut self, ms: f64) -> Self {
        self.max_render_time_ms = ms;
        self
    }

    /// Builder method to set command time budget.
    pub fn with_command_time(mut self, ms: f64) -> Self {
        self.max_command_time_ms = ms;
        self
    }

    /// Check if an input latency measurement is within budget.
    pub fn check_input_latency(&self, ms: f64) -> bool {
        ms <= self.max_input_latency_ms
    }

    /// Check if a layout time measurement is within budget.
    pub fn check_layout_time(&self, ms: f64) -> bool {
        ms <= self.max_layout_time_ms
    }

    /// Check if a render time measurement is within budget.
    pub fn check_render_time(&self, ms: f64) -> bool {
        ms <= self.max_render_time_ms
    }

    /// Check if a command time measurement is within budget.
    pub fn check_command_time(&self, ms: f64) -> bool {
        ms <= self.max_command_time_ms
    }

    /// Get the headroom (remaining time) for input latency.
    pub fn input_headroom(&self, ms: f64) -> f64 {
        self.max_input_latency_ms - ms
    }

    /// Get the headroom (remaining time) for layout.
    pub fn layout_headroom(&self, ms: f64) -> f64 {
        self.max_layout_time_ms - ms
    }

    /// Get the headroom (remaining time) for render.
    pub fn render_headroom(&self, ms: f64) -> f64 {
        self.max_render_time_ms - ms
    }

    /// Get the headroom (remaining time) for command execution.
    pub fn command_headroom(&self, ms: f64) -> f64 {
        self.max_command_time_ms - ms
    }
}

impl Default for PerfBudget {
    /// Default budget targets 60fps with 50ms input latency.
    fn default() -> Self {
        Self::for_60fps()
    }
}

/// A performance budget violation.
///
/// Generated when a measurement exceeds its budget threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetViolation {
    /// Category of the violation (e.g., "input_latency", "layout", "render", "command:name")
    pub category: String,
    /// Actual measured time in milliseconds
    pub actual_ms: f64,
    /// Budget threshold in milliseconds
    pub budget_ms: f64,
    /// Severity of the violation
    pub severity: ViolationSeverity,
}

impl BudgetViolation {
    /// Create a new budget violation.
    pub fn new(
        category: impl Into<String>,
        actual_ms: f64,
        budget_ms: f64,
        severity: ViolationSeverity,
    ) -> Self {
        Self {
            category: category.into(),
            actual_ms,
            budget_ms,
            severity,
        }
    }

    /// Calculate how much the actual time exceeds the budget.
    pub fn excess_ms(&self) -> f64 {
        self.actual_ms - self.budget_ms
    }

    /// Calculate the ratio of actual to budget (e.g., 2.0 means 2x over budget).
    pub fn ratio(&self) -> f64 {
        self.actual_ms / self.budget_ms
    }

    /// Check if this is a critical violation.
    pub fn is_critical(&self) -> bool {
        matches!(self.severity, ViolationSeverity::Critical)
    }

    /// Check if this is at least a high severity violation.
    pub fn is_high_or_worse(&self) -> bool {
        matches!(
            self.severity,
            ViolationSeverity::High | ViolationSeverity::Critical
        )
    }
}

impl std::fmt::Display for BudgetViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.2}ms (budget: {:.2}ms, {:.1}x over)",
            self.category,
            self.actual_ms,
            self.budget_ms,
            self.ratio()
        )
    }
}

/// A report of budget violations over a time period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetReport {
    /// All violations in this report
    pub violations: Vec<BudgetViolation>,
    /// Number of critical violations
    pub critical_count: usize,
    /// Number of high severity violations
    pub high_count: usize,
    /// Number of medium severity violations
    pub medium_count: usize,
    /// Number of low severity violations
    pub low_count: usize,
    /// Total time covered by this report in milliseconds
    pub duration_ms: f64,
}

impl BudgetReport {
    /// Create a new budget report from violations.
    pub fn from_violations(violations: Vec<BudgetViolation>, duration_ms: f64) -> Self {
        let critical_count = violations
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical))
            .count();
        let high_count = violations
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::High))
            .count();
        let medium_count = violations
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Medium))
            .count();
        let low_count = violations
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Low))
            .count();

        Self {
            violations,
            critical_count,
            high_count,
            medium_count,
            low_count,
            duration_ms,
        }
    }

    /// Check if there are any violations.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Check if there are any critical violations.
    pub fn has_critical(&self) -> bool {
        self.critical_count > 0
    }

    /// Check if the report passes (no high or critical violations).
    pub fn passes(&self) -> bool {
        self.critical_count == 0 && self.high_count == 0
    }

    /// Get the total number of violations.
    pub fn total_violations(&self) -> usize {
        self.violations.len()
    }

    /// Get violations by category.
    pub fn violations_by_category(&self, category: &str) -> Vec<&BudgetViolation> {
        self.violations
            .iter()
            .filter(|v| v.category == category || v.category.starts_with(&format!("{}:", category)))
            .collect()
    }
}

impl std::fmt::Display for BudgetReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.violations.is_empty() {
            write!(f, "No budget violations")
        } else {
            writeln!(f, "Budget Report: {} violations", self.total_violations())?;
            writeln!(f, "  Critical: {}", self.critical_count)?;
            writeln!(f, "  High: {}", self.high_count)?;
            writeln!(f, "  Medium: {}", self.medium_count)?;
            writeln!(f, "  Low: {}", self.low_count)?;
            for v in &self.violations {
                writeln!(f, "  - {}", v)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_defaults() {
        let budget = PerfBudget::default();
        assert_eq!(budget.max_input_latency_ms, 50.0);
        assert_eq!(budget.max_render_time_ms, 16.0);
    }

    #[test]
    fn test_budget_for_120fps() {
        let budget = PerfBudget::for_120fps();
        assert_eq!(budget.max_render_time_ms, 8.0);
        assert_eq!(budget.max_input_latency_ms, 30.0);
    }

    #[test]
    fn test_budget_checks() {
        let budget = PerfBudget::default();

        assert!(budget.check_input_latency(40.0));
        assert!(!budget.check_input_latency(60.0));

        assert!(budget.check_render_time(15.0));
        assert!(!budget.check_render_time(20.0));
    }

    #[test]
    fn test_budget_headroom() {
        let budget = PerfBudget::default();

        assert_eq!(budget.input_headroom(30.0), 20.0);
        assert_eq!(budget.input_headroom(60.0), -10.0);
    }

    #[test]
    fn test_budget_builder() {
        let budget = PerfBudget::default()
            .with_input_latency(30.0)
            .with_render_time(8.0);

        assert_eq!(budget.max_input_latency_ms, 30.0);
        assert_eq!(budget.max_render_time_ms, 8.0);
    }

    #[test]
    fn test_violation_display() {
        let violation = BudgetViolation::new("input_latency", 100.0, 50.0, ViolationSeverity::High);

        let display = format!("{}", violation);
        assert!(display.contains("input_latency"));
        assert!(display.contains("100.00ms"));
        assert!(display.contains("50.00ms"));
    }

    #[test]
    fn test_violation_ratio() {
        let violation = BudgetViolation::new("test", 150.0, 50.0, ViolationSeverity::Critical);
        assert_eq!(violation.ratio(), 3.0);
        assert_eq!(violation.excess_ms(), 100.0);
    }

    #[test]
    fn test_budget_report() {
        let violations = vec![
            BudgetViolation::new("test1", 100.0, 50.0, ViolationSeverity::High),
            BudgetViolation::new("test2", 200.0, 50.0, ViolationSeverity::Critical),
            BudgetViolation::new("test3", 60.0, 50.0, ViolationSeverity::Low),
        ];

        let report = BudgetReport::from_violations(violations, 1000.0);

        assert_eq!(report.total_violations(), 3);
        assert_eq!(report.critical_count, 1);
        assert_eq!(report.high_count, 1);
        assert_eq!(report.low_count, 1);
        assert!(report.has_critical());
        assert!(!report.passes());
    }
}
