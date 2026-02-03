//! BiDi (Bidirectional text) support
//!
//! This module implements the Unicode Bidirectional Algorithm (UAX #9) for
//! proper display of mixed right-to-left (RTL) and left-to-right (LTR) text.
//!
//! The implementation uses the `unicode-bidi` crate for the heavy lifting and
//! provides a higher-level API suitable for integration with the layout engine.
//!
//! # Overview
//!
//! The BiDi algorithm works in several phases:
//! 1. Determine the base (paragraph) direction
//! 2. Analyze the text to assign embedding levels to each character
//! 3. Split the text into runs of the same level
//! 4. Reorder the runs for visual display
//!
//! # Example
//!
//! ```rust,ignore
//! use layout_engine::{BidiAnalyzer, Direction};
//!
//! let analyzer = BidiAnalyzer::new();
//! let text = "Hello \u{05D0}\u{05D1}\u{05D2} World"; // "Hello אבג World"
//!
//! // Analyze with auto-detected direction
//! let runs = analyzer.analyze(text, None);
//!
//! // Get visual order for display
//! let visual_order = analyzer.visual_order(&runs);
//! ```

use crate::Direction;
use std::ops::Range;
use unicode_bidi::{BidiInfo, Level};

/// BiDi run information
///
/// Represents a contiguous run of text with the same embedding level and direction.
/// Runs are the basic unit for visual reordering in the BiDi algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiRun {
    /// Start byte offset in text
    pub start: usize,
    /// End byte offset in text
    pub end: usize,
    /// Direction of this run
    pub direction: Direction,
    /// Embedding level (even = LTR, odd = RTL)
    pub level: u8,
}

impl BidiRun {
    /// Create a new BiDi run
    pub fn new(start: usize, end: usize, level: u8) -> Self {
        Self {
            start,
            end,
            direction: if level % 2 == 0 { Direction::Ltr } else { Direction::Rtl },
            level,
        }
    }

    /// Get the byte range of this run
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Check if this run is RTL
    pub fn is_rtl(&self) -> bool {
        self.direction == Direction::Rtl
    }

    /// Check if this run is LTR
    pub fn is_ltr(&self) -> bool {
        self.direction == Direction::Ltr
    }

    /// Get the length of this run in bytes
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this run is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Result of BiDi analysis for a paragraph
#[derive(Debug, Clone)]
pub struct BidiParagraph {
    /// The analyzed text
    pub text: String,
    /// Base (paragraph) direction
    pub base_direction: Direction,
    /// Base embedding level
    pub base_level: u8,
    /// BiDi runs in logical order
    pub runs: Vec<BidiRun>,
    /// Embedding levels per byte
    pub levels: Vec<u8>,
    /// Whether the paragraph contains any RTL text
    pub has_rtl: bool,
}

impl BidiParagraph {
    /// Check if this paragraph is purely LTR (no RTL content)
    pub fn is_pure_ltr(&self) -> bool {
        !self.has_rtl
    }

    /// Check if this paragraph is purely RTL (no LTR content)
    pub fn is_pure_rtl(&self) -> bool {
        self.has_rtl && self.runs.iter().all(|r| r.is_rtl())
    }

    /// Get the runs in visual (display) order
    pub fn visual_runs(&self) -> Vec<&BidiRun> {
        if self.runs.is_empty() {
            return Vec::new();
        }

        // Get the visual order indices
        let levels: Vec<Level> = self.levels
            .iter()
            .filter_map(|&l| Level::new(l).ok())
            .collect();

        if levels.is_empty() {
            return self.runs.iter().collect();
        }

        // Use the run levels for reordering
        let run_levels: Vec<Level> = self.runs
            .iter()
            .filter_map(|r| Level::new(r.level).ok())
            .collect();

        if run_levels.len() != self.runs.len() {
            return self.runs.iter().collect();
        }

        let visual_indices = BidiInfo::reorder_visual(&run_levels);

        visual_indices
            .iter()
            .filter_map(|&idx| self.runs.get(idx))
            .collect()
    }

    /// Get text slice for a run
    pub fn run_text(&self, run: &BidiRun) -> &str {
        &self.text[run.start..run.end]
    }
}

/// BiDi analyzer for determining text direction and creating runs
///
/// The analyzer uses the Unicode Bidirectional Algorithm (UAX #9) to:
/// - Detect the base paragraph direction
/// - Compute embedding levels for each character
/// - Split text into directional runs
/// - Provide visual reordering information
#[derive(Debug, Clone, Default)]
pub struct BidiAnalyzer {
    /// Cache the last analyzed text for potential reuse
    _cache_enabled: bool,
}

impl BidiAnalyzer {
    /// Create a new BiDi analyzer
    pub fn new() -> Self {
        Self {
            _cache_enabled: false,
        }
    }

    /// Detect the base direction of text from the first strong character
    ///
    /// This follows the Unicode BiDi Algorithm rule P2/P3:
    /// - Scan for the first character with a strong directional type (L, R, or AL)
    /// - Return LTR if L is found first, RTL if R or AL is found first
    /// - Return the default if no strong character is found
    pub fn detect_base_direction(&self, text: &str) -> Direction {
        match unicode_bidi::get_base_direction(text) {
            unicode_bidi::Direction::Ltr => Direction::Ltr,
            unicode_bidi::Direction::Rtl => Direction::Rtl,
            unicode_bidi::Direction::Mixed => {
                // For mixed, check the first strong character
                match unicode_bidi::get_base_direction_full(text) {
                    unicode_bidi::Direction::Rtl => Direction::Rtl,
                    _ => Direction::Ltr,
                }
            }
        }
    }

    /// Detect base direction with a fallback default
    pub fn detect_base_direction_or(&self, text: &str, default: Direction) -> Direction {
        match unicode_bidi::get_base_direction(text) {
            unicode_bidi::Direction::Ltr => Direction::Ltr,
            unicode_bidi::Direction::Rtl => Direction::Rtl,
            unicode_bidi::Direction::Mixed => default,
        }
    }

    /// Analyze a paragraph and return detailed BiDi information
    ///
    /// This performs full BiDi analysis including:
    /// - Computing embedding levels for each byte
    /// - Splitting into directional runs
    /// - Determining if RTL content is present
    ///
    /// # Arguments
    /// * `text` - The text to analyze
    /// * `base_direction` - Optional explicit base direction. If None, auto-detect.
    pub fn analyze_paragraph(
        &self,
        text: &str,
        base_direction: Option<Direction>,
    ) -> BidiParagraph {
        if text.is_empty() {
            let base_dir = base_direction.unwrap_or(Direction::Ltr);
            return BidiParagraph {
                text: String::new(),
                base_direction: base_dir,
                base_level: if base_dir == Direction::Rtl { 1 } else { 0 },
                runs: Vec::new(),
                levels: Vec::new(),
                has_rtl: false,
            };
        }

        // Convert our Direction to unicode_bidi Level
        let default_level = base_direction.map(|d| {
            if d == Direction::Rtl {
                Level::rtl()
            } else {
                Level::ltr()
            }
        });

        // Perform BiDi analysis
        let bidi_info = BidiInfo::new(text, default_level);

        // Get the first (and usually only) paragraph
        let para = &bidi_info.paragraphs[0];
        let base_level = para.level.number();
        let base_dir = if para.level.is_rtl() {
            Direction::Rtl
        } else {
            Direction::Ltr
        };

        // Extract levels for this paragraph range
        let para_range = para.range.clone();
        let levels: Vec<u8> = bidi_info.levels[para_range.clone()]
            .iter()
            .map(|l| l.number())
            .collect();

        // Check if there's any RTL content
        let has_rtl = bidi_info.has_rtl();

        // Get visual runs for the paragraph
        let line_range = para_range.clone();
        let (_, visual_runs) = bidi_info.visual_runs(para, line_range);

        // Convert to our BidiRun format
        let runs: Vec<BidiRun> = visual_runs
            .iter()
            .map(|range| {
                let level = bidi_info.levels[range.start].number();
                BidiRun::new(range.start, range.end, level)
            })
            .collect();

        BidiParagraph {
            text: text.to_string(),
            base_direction: base_dir,
            base_level,
            runs,
            levels,
            has_rtl,
        }
    }

    /// Analyze a paragraph and return BiDi runs (simplified API)
    ///
    /// This is a convenience method that returns just the runs.
    /// For full analysis information, use `analyze_paragraph`.
    pub fn analyze(&self, text: &str, base_direction: Direction) -> Vec<BidiRun> {
        if text.is_empty() {
            return vec![BidiRun::new(0, 0, if base_direction == Direction::Rtl { 1 } else { 0 })];
        }

        let default_level = if base_direction == Direction::Rtl {
            Some(Level::rtl())
        } else {
            Some(Level::ltr())
        };

        let bidi_info = BidiInfo::new(text, default_level);

        if bidi_info.paragraphs.is_empty() {
            return vec![BidiRun::new(
                0,
                text.len(),
                if base_direction == Direction::Rtl { 1 } else { 0 },
            )];
        }

        let para = &bidi_info.paragraphs[0];
        let line_range = para.range.clone();
        let (_, visual_runs) = bidi_info.visual_runs(para, line_range);

        visual_runs
            .iter()
            .map(|range| {
                let level = bidi_info.levels[range.start].number();
                BidiRun::new(range.start, range.end, level)
            })
            .collect()
    }

    /// Analyze text for a specific line range
    ///
    /// This is useful when you've already broken text into lines and need
    /// BiDi information for each line independently.
    pub fn analyze_line(
        &self,
        text: &str,
        line_range: Range<usize>,
        base_direction: Direction,
    ) -> Vec<BidiRun> {
        if line_range.is_empty() || text.is_empty() {
            return Vec::new();
        }

        let line_text = &text[line_range.clone()];
        if line_text.is_empty() {
            return Vec::new();
        }

        let default_level = if base_direction == Direction::Rtl {
            Some(Level::rtl())
        } else {
            Some(Level::ltr())
        };

        let bidi_info = BidiInfo::new(text, default_level);

        if bidi_info.paragraphs.is_empty() {
            return vec![BidiRun::new(
                line_range.start,
                line_range.end,
                if base_direction == Direction::Rtl { 1 } else { 0 },
            )];
        }

        // Find the paragraph containing this line
        let para = bidi_info.paragraphs.iter()
            .find(|p| p.range.start <= line_range.start && p.range.end >= line_range.end)
            .unwrap_or(&bidi_info.paragraphs[0]);

        let (_, visual_runs) = bidi_info.visual_runs(para, line_range);

        visual_runs
            .iter()
            .map(|range| {
                let level = bidi_info.levels[range.start].number();
                BidiRun::new(range.start, range.end, level)
            })
            .collect()
    }

    /// Get the visual order of runs for display
    ///
    /// Returns indices into the runs slice in visual (display) order.
    /// For RTL base direction, rightmost content comes first.
    pub fn visual_order(&self, runs: &[BidiRun]) -> Vec<usize> {
        if runs.is_empty() {
            return Vec::new();
        }

        if runs.len() == 1 {
            return vec![0];
        }

        // Convert run levels to unicode_bidi Levels
        let levels: Vec<Level> = runs
            .iter()
            .filter_map(|r| Level::new(r.level).ok())
            .collect();

        if levels.len() != runs.len() {
            // Fallback if level conversion fails
            return (0..runs.len()).collect();
        }

        BidiInfo::reorder_visual(&levels)
    }

    /// Reorder runs in place for visual display
    ///
    /// This modifies the input vector to be in visual order.
    pub fn reorder_runs_visual(&self, runs: &mut Vec<BidiRun>) {
        let order = self.visual_order(runs);
        let original: Vec<BidiRun> = runs.clone();

        runs.clear();
        for idx in order {
            if let Some(run) = original.get(idx) {
                runs.push(run.clone());
            }
        }
    }

    /// Check if text contains any RTL characters
    pub fn has_rtl(&self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        let bidi_info = BidiInfo::new(text, None);
        bidi_info.has_rtl()
    }

    /// Check if text is purely LTR (no RTL characters)
    pub fn is_pure_ltr(&self, text: &str) -> bool {
        !self.has_rtl(text)
    }

    /// Get embedding levels for each byte in the text
    pub fn get_levels(&self, text: &str, base_direction: Direction) -> Vec<u8> {
        if text.is_empty() {
            return Vec::new();
        }

        let default_level = if base_direction == Direction::Rtl {
            Some(Level::rtl())
        } else {
            Some(Level::ltr())
        };

        let bidi_info = BidiInfo::new(text, default_level);
        bidi_info.levels.iter().map(|l| l.number()).collect()
    }

    /// Reorder a string for visual display
    ///
    /// Returns the text with characters reordered for left-to-right rendering.
    /// This is a convenience method for simple use cases.
    pub fn reorder_text(&self, text: &str, base_direction: Direction) -> String {
        if text.is_empty() {
            return String::new();
        }

        let default_level = if base_direction == Direction::Rtl {
            Some(Level::rtl())
        } else {
            Some(Level::ltr())
        };

        let bidi_info = BidiInfo::new(text, default_level);

        if bidi_info.paragraphs.is_empty() {
            return text.to_string();
        }

        let para = &bidi_info.paragraphs[0];
        let line = para.range.clone();

        bidi_info.reorder_line(para, line).into_owned()
    }
}

/// Helper function to convert between Direction types
pub fn direction_from_level(level: u8) -> Direction {
    if level % 2 == 0 {
        Direction::Ltr
    } else {
        Direction::Rtl
    }
}

/// Helper function to get the embedding level for a direction
pub fn level_for_direction(direction: Direction) -> u8 {
    match direction {
        Direction::Ltr => 0,
        Direction::Rtl => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidi_run_creation() {
        let run = BidiRun::new(0, 10, 0);
        assert_eq!(run.direction, Direction::Ltr);
        assert!(!run.is_rtl());
        assert!(run.is_ltr());
        assert_eq!(run.len(), 10);

        let rtl_run = BidiRun::new(0, 10, 1);
        assert_eq!(rtl_run.direction, Direction::Rtl);
        assert!(rtl_run.is_rtl());
        assert!(!rtl_run.is_ltr());
    }

    #[test]
    fn test_pure_ltr_text() {
        let analyzer = BidiAnalyzer::new();
        let text = "Hello World";

        let runs = analyzer.analyze(text, Direction::Ltr);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, Direction::Ltr);
        assert_eq!(runs[0].level, 0);
        assert_eq!(runs[0].start, 0);
        assert_eq!(runs[0].end, text.len());
    }

    #[test]
    fn test_pure_rtl_text() {
        let analyzer = BidiAnalyzer::new();
        // Hebrew text: "שלום"
        let text = "\u{05E9}\u{05DC}\u{05D5}\u{05DD}";

        let runs = analyzer.analyze(text, Direction::Rtl);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, Direction::Rtl);
        assert!(runs[0].level % 2 == 1); // Odd level = RTL
    }

    #[test]
    fn test_mixed_ltr_rtl() {
        let analyzer = BidiAnalyzer::new();
        // "Hello אבג World" - Hebrew letters in the middle
        let text = "Hello \u{05D0}\u{05D1}\u{05D2} World";

        let para = analyzer.analyze_paragraph(text, Some(Direction::Ltr));

        // Should have RTL content
        assert!(para.has_rtl);

        // Base direction should be LTR
        assert_eq!(para.base_direction, Direction::Ltr);

        // Should have multiple runs (LTR, RTL, LTR)
        assert!(para.runs.len() >= 2);
    }

    #[test]
    fn test_direction_detection() {
        let analyzer = BidiAnalyzer::new();

        // Pure LTR
        assert_eq!(analyzer.detect_base_direction("Hello"), Direction::Ltr);

        // Pure RTL (Hebrew)
        assert_eq!(
            analyzer.detect_base_direction("\u{05E9}\u{05DC}\u{05D5}\u{05DD}"),
            Direction::Rtl
        );

        // Starts with LTR
        assert_eq!(
            analyzer.detect_base_direction("Hello \u{05D0}\u{05D1}\u{05D2}"),
            Direction::Ltr
        );

        // Starts with RTL
        assert_eq!(
            analyzer.detect_base_direction("\u{05D0}\u{05D1}\u{05D2} Hello"),
            Direction::Rtl
        );
    }

    #[test]
    fn test_visual_order() {
        let analyzer = BidiAnalyzer::new();

        // Simple LTR - visual order same as logical
        let ltr_runs = vec![
            BidiRun::new(0, 5, 0),
            BidiRun::new(5, 10, 0),
        ];
        let order = analyzer.visual_order(&ltr_runs);
        assert_eq!(order, vec![0, 1]);

        // RTL runs should be reversed
        let rtl_runs = vec![
            BidiRun::new(0, 5, 1),
            BidiRun::new(5, 10, 1),
        ];
        let order = analyzer.visual_order(&rtl_runs);
        assert_eq!(order, vec![1, 0]);
    }

    #[test]
    fn test_empty_text() {
        let analyzer = BidiAnalyzer::new();

        let runs = analyzer.analyze("", Direction::Ltr);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].start, 0);
        assert_eq!(runs[0].end, 0);

        let para = analyzer.analyze_paragraph("", None);
        assert!(!para.has_rtl);
        assert!(para.runs.is_empty());
    }

    #[test]
    fn test_has_rtl() {
        let analyzer = BidiAnalyzer::new();

        assert!(!analyzer.has_rtl("Hello World"));
        assert!(analyzer.has_rtl("\u{05D0}\u{05D1}\u{05D2}"));
        assert!(analyzer.has_rtl("Hello \u{05D0} World"));
        assert!(!analyzer.has_rtl(""));
    }

    #[test]
    fn test_get_levels() {
        let analyzer = BidiAnalyzer::new();

        // Pure LTR - all levels should be 0
        let levels = analyzer.get_levels("Hello", Direction::Ltr);
        assert!(levels.iter().all(|&l| l == 0));

        // Pure RTL with RTL base
        let levels = analyzer.get_levels("\u{05D0}\u{05D1}", Direction::Rtl);
        assert!(levels.iter().all(|&l| l % 2 == 1)); // All odd (RTL)
    }

    #[test]
    fn test_reorder_text() {
        let analyzer = BidiAnalyzer::new();

        // Pure LTR should remain unchanged
        let text = "Hello World";
        let reordered = analyzer.reorder_text(text, Direction::Ltr);
        assert_eq!(reordered, text);

        // Empty text
        assert_eq!(analyzer.reorder_text("", Direction::Ltr), "");
    }

    #[test]
    fn test_bidi_paragraph_visual_runs() {
        let analyzer = BidiAnalyzer::new();
        let text = "Hello World";

        let para = analyzer.analyze_paragraph(text, Some(Direction::Ltr));
        let visual = para.visual_runs();

        assert!(!visual.is_empty());
    }

    #[test]
    fn test_direction_from_level() {
        assert_eq!(direction_from_level(0), Direction::Ltr);
        assert_eq!(direction_from_level(1), Direction::Rtl);
        assert_eq!(direction_from_level(2), Direction::Ltr);
        assert_eq!(direction_from_level(3), Direction::Rtl);
    }

    #[test]
    fn test_level_for_direction() {
        assert_eq!(level_for_direction(Direction::Ltr), 0);
        assert_eq!(level_for_direction(Direction::Rtl), 1);
    }

    #[test]
    fn test_analyze_line() {
        let analyzer = BidiAnalyzer::new();
        let text = "Hello World Test";

        // Analyze a portion of the text
        let runs = analyzer.analyze_line(text, 0..5, Direction::Ltr);
        assert!(!runs.is_empty());

        // Empty range
        let runs = analyzer.analyze_line(text, 0..0, Direction::Ltr);
        assert!(runs.is_empty());
    }

    #[test]
    fn test_reorder_runs_visual() {
        let analyzer = BidiAnalyzer::new();

        let mut runs = vec![
            BidiRun::new(0, 5, 1),  // RTL
            BidiRun::new(5, 10, 1), // RTL
        ];

        analyzer.reorder_runs_visual(&mut runs);

        // RTL runs should be reversed
        assert_eq!(runs[0].start, 5);
        assert_eq!(runs[1].start, 0);
    }
}
