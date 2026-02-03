//! Pagination Rules - Keep rules, widow/orphan control, and line numbering
//!
//! This module implements advanced pagination rules for document layout:
//! - Paragraph keep rules (keep_with_next, keep_together, page_break_before)
//! - Widow/orphan control with configurable minimum lines
//! - Line numbering configuration per section

use serde::{Deserialize, Serialize};

// =============================================================================
// Paragraph Keep Rules
// =============================================================================

/// Paragraph keep rules that control page/column breaks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ParagraphKeepRules {
    /// Don't break between this paragraph and the next paragraph.
    /// When enabled, if a page/column break would occur between this paragraph
    /// and the next, both paragraphs are moved to the new page/column together.
    pub keep_with_next: bool,

    /// Don't break within this paragraph.
    /// When enabled, all lines of the paragraph must appear on the same page/column.
    /// If the paragraph doesn't fit, it's moved entirely to the next page/column.
    pub keep_together: bool,

    /// Always start this paragraph on a new page.
    /// This forces a page break before the paragraph regardless of available space.
    pub page_break_before: bool,
}

impl ParagraphKeepRules {
    /// Create new keep rules with all options disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create keep rules with keep_with_next enabled
    pub fn keep_with_next() -> Self {
        Self {
            keep_with_next: true,
            ..Default::default()
        }
    }

    /// Create keep rules with keep_together enabled
    pub fn keep_together() -> Self {
        Self {
            keep_together: true,
            ..Default::default()
        }
    }

    /// Create keep rules with page_break_before enabled
    pub fn page_break_before() -> Self {
        Self {
            page_break_before: true,
            ..Default::default()
        }
    }

    /// Check if any keep rules are active
    pub fn is_active(&self) -> bool {
        self.keep_with_next || self.keep_together || self.page_break_before
    }

    /// Builder method to set keep_with_next
    pub fn with_keep_with_next(mut self, value: bool) -> Self {
        self.keep_with_next = value;
        self
    }

    /// Builder method to set keep_together
    pub fn with_keep_together(mut self, value: bool) -> Self {
        self.keep_together = value;
        self
    }

    /// Builder method to set page_break_before
    pub fn with_page_break_before(mut self, value: bool) -> Self {
        self.page_break_before = value;
        self
    }
}

// =============================================================================
// Widow/Orphan Control
// =============================================================================

/// Widow/orphan control settings for pagination
///
/// - **Widow**: The last line of a paragraph appearing alone at the top of a page/column.
/// - **Orphan**: The first line of a paragraph appearing alone at the bottom of a page/column.
///
/// Both are considered poor typography and should be avoided. This control ensures
/// that at least `min_lines_top` lines appear at the top of a page (widow control)
/// and at least `min_lines_bottom` lines appear at the bottom (orphan control).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidowOrphanControl {
    /// Whether widow/orphan control is enabled
    pub enabled: bool,

    /// Minimum number of lines that must appear at the top of a page/column.
    /// This prevents widows - if only 1 line would appear at the top,
    /// 2 or more lines from the previous page are moved to keep them together.
    /// Typical value: 2
    pub min_lines_top: u8,

    /// Minimum number of lines that must appear at the bottom of a page/column.
    /// This prevents orphans - if only 1 line would appear at the bottom,
    /// more lines are moved to the next page to avoid leaving an orphan.
    /// Typical value: 2
    pub min_lines_bottom: u8,
}

impl WidowOrphanControl {
    /// Create new widow/orphan control with default settings (enabled, 2 lines each)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create disabled widow/orphan control
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            min_lines_top: 2,
            min_lines_bottom: 2,
        }
    }

    /// Create with custom minimum line counts
    pub fn with_min_lines(min_top: u8, min_bottom: u8) -> Self {
        Self {
            enabled: true,
            min_lines_top: min_top.max(1),
            min_lines_bottom: min_bottom.max(1),
        }
    }

    /// Builder method to enable/disable
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder method to set minimum lines at top
    pub fn with_min_lines_top(mut self, lines: u8) -> Self {
        self.min_lines_top = lines.max(1);
        self
    }

    /// Builder method to set minimum lines at bottom
    pub fn with_min_lines_bottom(mut self, lines: u8) -> Self {
        self.min_lines_bottom = lines.max(1);
        self
    }

    /// Get the effective minimum lines for the top (0 if disabled)
    pub fn effective_min_top(&self) -> usize {
        if self.enabled {
            self.min_lines_top as usize
        } else {
            0
        }
    }

    /// Get the effective minimum lines for the bottom (0 if disabled)
    pub fn effective_min_bottom(&self) -> usize {
        if self.enabled {
            self.min_lines_bottom as usize
        } else {
            0
        }
    }
}

impl Default for WidowOrphanControl {
    fn default() -> Self {
        Self {
            enabled: true,
            min_lines_top: 2,
            min_lines_bottom: 2,
        }
    }
}

// =============================================================================
// Line Number Restart
// =============================================================================

/// When to restart line numbering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineNumberRestart {
    /// Restart numbering on each new page
    #[default]
    PerPage,
    /// Restart numbering at each new section
    PerSection,
    /// Continuous numbering throughout the document
    Continuous,
}

// =============================================================================
// Line Numbering
// =============================================================================

/// Line numbering configuration for a section
///
/// Line numbers appear in the left margin and can be useful for:
/// - Legal documents
/// - Code listings
/// - Academic papers
/// - Reference documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineNumbering {
    /// Whether line numbering is enabled for this section
    pub enabled: bool,

    /// The starting line number (1 = start at 1)
    pub start_at: u32,

    /// Show line number every Nth line (1 = every line, 5 = every 5th line)
    /// A value of 0 is treated as 1 (show every line)
    pub count_by: u32,

    /// When to restart line numbering
    pub restart: LineNumberRestart,

    /// Distance from the text to the line number in points
    /// Positive values move the number away from the text margin
    pub distance_from_text: f32,
}

impl LineNumbering {
    /// Create new line numbering with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create line numbering that's disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create enabled line numbering with defaults
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Create line numbering showing every Nth line
    pub fn every_n_lines(n: u32) -> Self {
        Self {
            enabled: true,
            count_by: n.max(1),
            ..Default::default()
        }
    }

    /// Builder method to enable/disable
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder method to set start number
    pub fn with_start_at(mut self, start: u32) -> Self {
        self.start_at = start.max(1);
        self
    }

    /// Builder method to set count_by
    pub fn with_count_by(mut self, count: u32) -> Self {
        self.count_by = count.max(1);
        self
    }

    /// Builder method to set restart mode
    pub fn with_restart(mut self, restart: LineNumberRestart) -> Self {
        self.restart = restart;
        self
    }

    /// Builder method to set distance from text
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance_from_text = distance;
        self
    }

    /// Check if a given line number should be displayed
    pub fn should_display(&self, line_number: u32) -> bool {
        if !self.enabled {
            return false;
        }
        let count_by = if self.count_by == 0 { 1 } else { self.count_by };
        line_number % count_by == 0
    }

    /// Get the effective count_by value (minimum 1)
    pub fn effective_count_by(&self) -> u32 {
        if self.count_by == 0 { 1 } else { self.count_by }
    }
}

impl Default for LineNumbering {
    fn default() -> Self {
        Self {
            enabled: false,
            start_at: 1,
            count_by: 1,
            restart: LineNumberRestart::default(),
            distance_from_text: 18.0, // 0.25 inch
        }
    }
}

// =============================================================================
// Document-Level Pagination Settings
// =============================================================================

/// Document-level pagination settings
///
/// These settings apply to the entire document unless overridden at the
/// section or paragraph level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPaginationSettings {
    /// Default widow/orphan control for the document
    pub widow_orphan_control: WidowOrphanControl,
}

impl DocumentPaginationSettings {
    /// Create new document pagination settings with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create settings with widow/orphan control disabled
    pub fn without_widow_orphan_control() -> Self {
        Self {
            widow_orphan_control: WidowOrphanControl::disabled(),
        }
    }
}

impl Default for DocumentPaginationSettings {
    fn default() -> Self {
        Self {
            widow_orphan_control: WidowOrphanControl::default(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_rules_default() {
        let rules = ParagraphKeepRules::new();
        assert!(!rules.keep_with_next);
        assert!(!rules.keep_together);
        assert!(!rules.page_break_before);
        assert!(!rules.is_active());
    }

    #[test]
    fn test_keep_rules_builders() {
        let rules = ParagraphKeepRules::keep_with_next();
        assert!(rules.keep_with_next);
        assert!(rules.is_active());

        let rules = ParagraphKeepRules::keep_together();
        assert!(rules.keep_together);
        assert!(rules.is_active());

        let rules = ParagraphKeepRules::page_break_before();
        assert!(rules.page_break_before);
        assert!(rules.is_active());
    }

    #[test]
    fn test_keep_rules_chained_builder() {
        let rules = ParagraphKeepRules::new()
            .with_keep_with_next(true)
            .with_keep_together(true);
        assert!(rules.keep_with_next);
        assert!(rules.keep_together);
        assert!(!rules.page_break_before);
    }

    #[test]
    fn test_widow_orphan_default() {
        let control = WidowOrphanControl::new();
        assert!(control.enabled);
        assert_eq!(control.min_lines_top, 2);
        assert_eq!(control.min_lines_bottom, 2);
        assert_eq!(control.effective_min_top(), 2);
        assert_eq!(control.effective_min_bottom(), 2);
    }

    #[test]
    fn test_widow_orphan_disabled() {
        let control = WidowOrphanControl::disabled();
        assert!(!control.enabled);
        assert_eq!(control.effective_min_top(), 0);
        assert_eq!(control.effective_min_bottom(), 0);
    }

    #[test]
    fn test_widow_orphan_custom() {
        let control = WidowOrphanControl::with_min_lines(3, 4);
        assert!(control.enabled);
        assert_eq!(control.min_lines_top, 3);
        assert_eq!(control.min_lines_bottom, 4);
    }

    #[test]
    fn test_widow_orphan_min_clamp() {
        // Minimum should be 1
        let control = WidowOrphanControl::with_min_lines(0, 0);
        assert_eq!(control.min_lines_top, 1);
        assert_eq!(control.min_lines_bottom, 1);
    }

    #[test]
    fn test_line_numbering_default() {
        let ln = LineNumbering::new();
        assert!(!ln.enabled);
        assert_eq!(ln.start_at, 1);
        assert_eq!(ln.count_by, 1);
        assert_eq!(ln.restart, LineNumberRestart::PerPage);
    }

    #[test]
    fn test_line_numbering_enabled() {
        let ln = LineNumbering::enabled();
        assert!(ln.enabled);
    }

    #[test]
    fn test_line_numbering_every_n_lines() {
        let ln = LineNumbering::every_n_lines(5);
        assert!(ln.enabled);
        assert_eq!(ln.count_by, 5);

        // Should display at 5, 10, 15, etc.
        assert!(!ln.should_display(1));
        assert!(!ln.should_display(4));
        assert!(ln.should_display(5));
        assert!(ln.should_display(10));
    }

    #[test]
    fn test_line_numbering_should_display() {
        let ln = LineNumbering::enabled();
        assert!(ln.should_display(1));
        assert!(ln.should_display(100));

        let disabled = LineNumbering::disabled();
        assert!(!disabled.should_display(1));
    }

    #[test]
    fn test_line_numbering_count_by_zero() {
        let ln = LineNumbering::every_n_lines(0);
        // 0 should be treated as 1
        assert_eq!(ln.effective_count_by(), 1);
    }

    #[test]
    fn test_line_number_restart_variants() {
        assert_eq!(LineNumberRestart::default(), LineNumberRestart::PerPage);

        let _ = LineNumberRestart::PerSection;
        let _ = LineNumberRestart::Continuous;
    }

    #[test]
    fn test_document_pagination_settings() {
        let settings = DocumentPaginationSettings::new();
        assert!(settings.widow_orphan_control.enabled);

        let settings = DocumentPaginationSettings::without_widow_orphan_control();
        assert!(!settings.widow_orphan_control.enabled);
    }
}
