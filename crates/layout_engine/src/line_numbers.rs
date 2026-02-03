//! Line Number Layout - Track and render line numbers in the left margin
//!
//! This module implements line numbering for document layout:
//! - Tracks line numbers during layout based on restart mode
//! - Generates line number render items positioned in the left margin
//! - Supports per-page, per-section, and continuous numbering

use crate::LineBox;
use doc_model::{LineNumbering, LineNumberRestart, NodeId};

/// Information about a rendered line number
#[derive(Debug, Clone)]
pub struct LineNumberInfo {
    /// The line number value to display
    pub number: u32,
    /// X position (distance from left margin)
    pub x: f32,
    /// Y position (baseline-aligned with the text line)
    pub y: f32,
    /// Font size for the line number
    pub font_size: f32,
    /// The page index this line number is on
    pub page_index: usize,
}

/// Tracker for line numbers during layout
///
/// This struct maintains the current line number state and generates
/// line number information for each line based on the configuration.
#[derive(Debug, Clone)]
pub struct LineNumberTracker {
    /// Current line number counter
    current_number: u32,
    /// The section's line numbering configuration
    config: LineNumbering,
    /// The current section ID (for per-section restart)
    current_section_id: Option<NodeId>,
    /// The current page index (for per-page restart)
    current_page_index: usize,
    /// Collected line numbers for rendering
    line_numbers: Vec<LineNumberInfo>,
}

impl LineNumberTracker {
    /// Create a new line number tracker with the given configuration
    pub fn new(config: LineNumbering) -> Self {
        Self {
            current_number: config.start_at,
            config,
            current_section_id: None,
            current_page_index: 0,
            line_numbers: Vec::new(),
        }
    }

    /// Create a disabled tracker (no line numbers)
    pub fn disabled() -> Self {
        Self::new(LineNumbering::disabled())
    }

    /// Check if line numbering is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the current configuration
    pub fn config(&self) -> &LineNumbering {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: LineNumbering) {
        self.config = config;
        self.reset();
    }

    /// Reset the counter to the start value
    pub fn reset(&mut self) {
        self.current_number = self.config.start_at;
    }

    /// Notify the tracker that we've moved to a new page
    pub fn on_new_page(&mut self, page_index: usize) {
        if self.config.restart == LineNumberRestart::PerPage && page_index != self.current_page_index {
            self.reset();
        }
        self.current_page_index = page_index;
    }

    /// Notify the tracker that we've moved to a new section
    pub fn on_new_section(&mut self, section_id: NodeId) {
        if self.config.restart == LineNumberRestart::PerSection {
            if self.current_section_id != Some(section_id) {
                self.reset();
            }
        }
        self.current_section_id = Some(section_id);
    }

    /// Process a line and optionally generate a line number info
    ///
    /// Returns the line number info if this line should display a number.
    ///
    /// # Arguments
    /// * `line` - The line box being laid out
    /// * `content_area_x` - The x position of the content area (left margin)
    /// * `content_area_y` - The y position of the content area top
    /// * `page_index` - The current page index
    /// * `font_size` - The font size to use for line numbers
    pub fn process_line(
        &mut self,
        line: &LineBox,
        content_area_x: f32,
        content_area_y: f32,
        page_index: usize,
        font_size: f32,
    ) -> Option<LineNumberInfo> {
        if !self.config.enabled {
            return None;
        }

        // Get the current number before incrementing
        let line_num = self.current_number;

        // Increment the counter for the next line
        self.current_number += 1;

        // Check if we should display this line number based on count_by
        if !self.config.should_display(line_num) {
            return None;
        }

        // Calculate position
        // Line number is positioned to the left of the content area
        // Right-aligned at distance_from_text from the left margin
        let x = content_area_x - self.config.distance_from_text;

        // Y position is aligned with the line's baseline
        let y = content_area_y + line.bounds.y + line.baseline;

        let info = LineNumberInfo {
            number: line_num,
            x,
            y,
            font_size,
            page_index,
        };

        self.line_numbers.push(info.clone());
        Some(info)
    }

    /// Get all collected line numbers
    pub fn line_numbers(&self) -> &[LineNumberInfo] {
        &self.line_numbers
    }

    /// Clear collected line numbers (call before re-layout)
    pub fn clear_line_numbers(&mut self) {
        self.line_numbers.clear();
    }

    /// Get line numbers for a specific page
    pub fn line_numbers_on_page(&self, page_index: usize) -> impl Iterator<Item = &LineNumberInfo> {
        self.line_numbers.iter().filter(move |ln| ln.page_index == page_index)
    }

    /// Get the current line number value (next number to be assigned)
    pub fn current_number(&self) -> u32 {
        self.current_number
    }

    /// Increment the line counter without generating any info
    ///
    /// Use this when you want to track line numbers but handle
    /// the display logic externally.
    pub fn process_line_silent(&mut self) {
        if self.config.enabled {
            self.current_number += 1;
        }
    }
}

impl Default for LineNumberTracker {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Line number layout information stored in the layout tree
#[derive(Debug, Clone, Default)]
pub struct LineNumberLayout {
    /// Line numbers per page (indexed by page number)
    pub line_numbers: Vec<Vec<LineNumberInfo>>,
}

impl LineNumberLayout {
    /// Create a new empty line number layout
    pub fn new() -> Self {
        Self {
            line_numbers: Vec::new(),
        }
    }

    /// Add a line number for a specific page
    pub fn add_line_number(&mut self, page_index: usize, info: LineNumberInfo) {
        // Ensure we have enough pages
        while self.line_numbers.len() <= page_index {
            self.line_numbers.push(Vec::new());
        }
        self.line_numbers[page_index].push(info);
    }

    /// Get line numbers for a specific page
    pub fn get_page_line_numbers(&self, page_index: usize) -> &[LineNumberInfo] {
        self.line_numbers.get(page_index).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if there are any line numbers
    pub fn has_line_numbers(&self) -> bool {
        self.line_numbers.iter().any(|page| !page.is_empty())
    }

    /// Clear all line numbers
    pub fn clear(&mut self) {
        self.line_numbers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    #[test]
    fn test_line_number_tracker_disabled() {
        let tracker = LineNumberTracker::disabled();
        assert!(!tracker.is_enabled());
    }

    #[test]
    fn test_line_number_tracker_enabled() {
        let config = LineNumbering::enabled();
        let tracker = LineNumberTracker::new(config);
        assert!(tracker.is_enabled());
        assert_eq!(tracker.current_number(), 1);
    }

    #[test]
    fn test_line_number_tracker_count_by() {
        let config = LineNumbering::every_n_lines(5);
        let mut tracker = LineNumberTracker::new(config);

        // Create a dummy line
        let line = LineBox {
            bounds: Rect::new(0.0, 0.0, 400.0, 20.0),
            baseline: 15.0,
            direction: crate::Direction::Ltr,
            inlines: Vec::new(),
        };

        // Lines 1-4 should not display
        for _ in 0..4 {
            let result = tracker.process_line(&line, 72.0, 100.0, 0, 12.0);
            assert!(result.is_none());
        }

        // Line 5 should display
        let result = tracker.process_line(&line, 72.0, 100.0, 0, 12.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().number, 5);
    }

    #[test]
    fn test_line_number_tracker_per_page_restart() {
        let config = LineNumbering::enabled()
            .with_restart(LineNumberRestart::PerPage);
        let mut tracker = LineNumberTracker::new(config);

        // Process some lines on page 0
        let line = LineBox {
            bounds: Rect::new(0.0, 0.0, 400.0, 20.0),
            baseline: 15.0,
            direction: crate::Direction::Ltr,
            inlines: Vec::new(),
        };

        for _ in 0..3 {
            tracker.process_line(&line, 72.0, 100.0, 0, 12.0);
        }
        assert_eq!(tracker.current_number(), 4);

        // Move to page 1 - should reset
        tracker.on_new_page(1);
        assert_eq!(tracker.current_number(), 1);
    }

    #[test]
    fn test_line_number_tracker_continuous() {
        let config = LineNumbering::enabled()
            .with_restart(LineNumberRestart::Continuous);
        let mut tracker = LineNumberTracker::new(config);

        let line = LineBox {
            bounds: Rect::new(0.0, 0.0, 400.0, 20.0),
            baseline: 15.0,
            direction: crate::Direction::Ltr,
            inlines: Vec::new(),
        };

        // Process some lines on page 0
        for _ in 0..3 {
            tracker.process_line(&line, 72.0, 100.0, 0, 12.0);
        }
        assert_eq!(tracker.current_number(), 4);

        // Move to page 1 - should NOT reset
        tracker.on_new_page(1);
        assert_eq!(tracker.current_number(), 4);
    }

    #[test]
    fn test_line_number_position() {
        let config = LineNumbering::enabled()
            .with_distance(24.0); // 24 points from text
        let mut tracker = LineNumberTracker::new(config);

        let line = LineBox {
            bounds: Rect::new(0.0, 50.0, 400.0, 20.0),
            baseline: 15.0,
            direction: crate::Direction::Ltr,
            inlines: Vec::new(),
        };

        let result = tracker.process_line(&line, 72.0, 100.0, 0, 12.0);
        assert!(result.is_some());

        let info = result.unwrap();
        // X should be content_area_x - distance_from_text
        assert_eq!(info.x, 72.0 - 24.0);
        // Y should be content_area_y + line.bounds.y + line.baseline
        assert_eq!(info.y, 100.0 + 50.0 + 15.0);
    }

    #[test]
    fn test_line_number_layout() {
        let mut layout = LineNumberLayout::new();

        let info1 = LineNumberInfo {
            number: 1,
            x: 48.0,
            y: 115.0,
            font_size: 10.0,
            page_index: 0,
        };
        let info2 = LineNumberInfo {
            number: 2,
            x: 48.0,
            y: 135.0,
            font_size: 10.0,
            page_index: 0,
        };
        let info3 = LineNumberInfo {
            number: 1,
            x: 48.0,
            y: 115.0,
            font_size: 10.0,
            page_index: 1,
        };

        layout.add_line_number(0, info1);
        layout.add_line_number(0, info2);
        layout.add_line_number(1, info3);

        assert!(layout.has_line_numbers());
        assert_eq!(layout.get_page_line_numbers(0).len(), 2);
        assert_eq!(layout.get_page_line_numbers(1).len(), 1);
        assert_eq!(layout.get_page_line_numbers(2).len(), 0);
    }
}
