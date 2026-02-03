//! Footnote and Endnote Layout
//!
//! This module handles the layout of footnotes and endnotes:
//! - Reserving space at the bottom of pages for footnotes
//! - Drawing separator lines between content and footnotes
//! - Flowing footnotes to the next page when needed
//! - Handling footnote continuation with notices
//! - Collecting endnotes at section or document end

use crate::{LineBox, Rect};
use doc_model::{EndnoteProperties, FootnoteProperties, NodeId, Note, NoteId};
use serde::{Deserialize, Serialize};

// =============================================================================
// Footnote Area Layout
// =============================================================================

/// Layout information for a footnote area on a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteAreaLayout {
    /// Bounds of the footnote area
    pub bounds: Rect,
    /// Whether to show the separator line
    pub show_separator: bool,
    /// Separator line layout (if shown)
    pub separator: Option<FootnoteSeparator>,
    /// Footnote blocks in this area
    pub footnotes: Vec<FootnoteBlockLayout>,
    /// Whether there's a continuation from the previous page
    pub has_continuation: bool,
    /// Whether this page's footnotes continue to the next page
    pub continues_to_next: bool,
}

impl FootnoteAreaLayout {
    /// Create a new empty footnote area layout
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            show_separator: true,
            separator: None,
            footnotes: Vec::new(),
            has_continuation: false,
            continues_to_next: false,
        }
    }

    /// Get the total height of the footnote area
    pub fn total_height(&self) -> f32 {
        let separator_height = self.separator.as_ref().map(|s| s.height()).unwrap_or(0.0);
        let footnote_height: f32 = self.footnotes.iter().map(|f| f.height).sum();
        separator_height + footnote_height
    }

    /// Check if the footnote area is empty
    pub fn is_empty(&self) -> bool {
        self.footnotes.is_empty()
    }

    /// Add a footnote to this area
    pub fn add_footnote(&mut self, footnote: FootnoteBlockLayout) {
        self.footnotes.push(footnote);
    }
}

// =============================================================================
// Footnote Separator
// =============================================================================

/// Layout for the footnote separator line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteSeparator {
    /// X position of the separator line start
    pub x: f32,
    /// Y position of the separator line
    pub y: f32,
    /// Width of the separator line
    pub width: f32,
    /// Weight (thickness) of the separator line
    pub weight: f32,
    /// Space above the separator
    pub space_above: f32,
    /// Space below the separator
    pub space_below: f32,
}

impl FootnoteSeparator {
    /// Create a new footnote separator
    pub fn new(x: f32, y: f32, width: f32, weight: f32) -> Self {
        Self {
            x,
            y,
            width,
            weight,
            space_above: 6.0,  // 6pt above
            space_below: 6.0,  // 6pt below
        }
    }

    /// Get the total height including spacing
    pub fn height(&self) -> f32 {
        self.space_above + self.weight + self.space_below
    }
}

// =============================================================================
// Footnote Block Layout
// =============================================================================

/// Layout for a single footnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteBlockLayout {
    /// The note ID
    pub note_id: NoteId,
    /// Bounds of this footnote
    pub bounds: Rect,
    /// Height of this footnote
    pub height: f32,
    /// The formatted mark
    pub mark: String,
    /// Line layouts for the footnote content
    pub lines: Vec<LineBox>,
    /// Whether this is a continuation from the previous page
    pub is_continuation: bool,
    /// Whether this footnote continues to the next page
    pub continues_to_next: bool,
}

impl FootnoteBlockLayout {
    /// Create a new footnote block layout
    pub fn new(note_id: NoteId, mark: String, bounds: Rect) -> Self {
        Self {
            note_id,
            bounds,
            height: bounds.height,
            mark,
            lines: Vec::new(),
            is_continuation: false,
            continues_to_next: false,
        }
    }

    /// Add a line to this footnote
    pub fn add_line(&mut self, line: LineBox) {
        self.lines.push(line);
    }
}

// =============================================================================
// Endnote Section Layout
// =============================================================================

/// Layout for an endnote section (appears at end of section or document)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndnoteSectionLayout {
    /// The section or document this belongs to
    pub source_id: Option<NodeId>,
    /// Whether this is at end of section (vs end of document)
    pub is_section_end: bool,
    /// Bounds of the endnote section
    pub bounds: Rect,
    /// Endnote blocks
    pub endnotes: Vec<EndnoteBlockLayout>,
    /// Optional heading for the endnote section
    pub heading: Option<String>,
}

impl EndnoteSectionLayout {
    /// Create a new endnote section layout
    pub fn new(bounds: Rect, is_section_end: bool) -> Self {
        Self {
            source_id: None,
            is_section_end,
            bounds,
            endnotes: Vec::new(),
            heading: Some(if is_section_end {
                "Section Notes".to_string()
            } else {
                "Notes".to_string()
            }),
        }
    }

    /// Add an endnote to this section
    pub fn add_endnote(&mut self, endnote: EndnoteBlockLayout) {
        self.endnotes.push(endnote);
    }

    /// Get the total height of the endnote section
    pub fn total_height(&self) -> f32 {
        self.endnotes.iter().map(|e| e.height).sum()
    }

    /// Check if the section is empty
    pub fn is_empty(&self) -> bool {
        self.endnotes.is_empty()
    }
}

// =============================================================================
// Endnote Block Layout
// =============================================================================

/// Layout for a single endnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndnoteBlockLayout {
    /// The note ID
    pub note_id: NoteId,
    /// Bounds of this endnote
    pub bounds: Rect,
    /// Height of this endnote
    pub height: f32,
    /// The formatted mark
    pub mark: String,
    /// Line layouts for the endnote content
    pub lines: Vec<LineBox>,
}

impl EndnoteBlockLayout {
    /// Create a new endnote block layout
    pub fn new(note_id: NoteId, mark: String, bounds: Rect) -> Self {
        Self {
            note_id,
            bounds,
            height: bounds.height,
            mark,
            lines: Vec::new(),
        }
    }

    /// Add a line to this endnote
    pub fn add_line(&mut self, line: LineBox) {
        self.lines.push(line);
    }
}

// =============================================================================
// Footnote Layouter
// =============================================================================

/// Handles footnote layout calculations
pub struct FootnoteLayouter {
    /// Default font size for footnotes
    pub footnote_font_size: f32,
    /// Default line height multiplier
    pub line_height_multiplier: f32,
    /// Indent for continuation lines
    pub hanging_indent: f32,
    /// Space between footnotes
    pub footnote_spacing: f32,
}

impl Default for FootnoteLayouter {
    fn default() -> Self {
        Self {
            footnote_font_size: 10.0,  // Typically smaller than body text
            line_height_multiplier: 1.2,
            hanging_indent: 18.0,       // For the superscript mark
            footnote_spacing: 4.0,      // 4pt between footnotes
        }
    }
}

impl FootnoteLayouter {
    /// Create a new footnote layouter
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the space needed for footnotes on a page
    pub fn calculate_footnote_space(
        &self,
        notes: &[&Note],
        props: &FootnoteProperties,
        available_width: f32,
    ) -> f32 {
        if notes.is_empty() {
            return 0.0;
        }

        // Calculate approximate height needed
        let separator_height = if props.show_separator {
            props.separator_weight + 12.0 // weight + spacing
        } else {
            0.0
        };

        let line_height = self.footnote_font_size * self.line_height_multiplier;

        // Estimate lines per footnote (simplified)
        let total_footnote_height: f32 = notes
            .iter()
            .map(|note| {
                // Estimate based on content paragraphs
                let para_count = note.content().len().max(1);
                (para_count as f32 * line_height * 2.0) + self.footnote_spacing
            })
            .sum();

        props.space_before + separator_height + total_footnote_height
    }

    /// Create footnote area layout for a page
    pub fn layout_footnotes(
        &self,
        notes: &[&Note],
        props: &FootnoteProperties,
        page_bounds: Rect,
        available_height: f32,
    ) -> FootnoteAreaLayout {
        let content_width = page_bounds.width;

        // Calculate starting Y position (from bottom of available area)
        let footnote_area_y = page_bounds.y + page_bounds.height - available_height;

        let bounds = Rect::new(
            page_bounds.x,
            footnote_area_y,
            content_width,
            available_height,
        );

        let mut layout = FootnoteAreaLayout::new(bounds);
        layout.show_separator = props.show_separator;

        if notes.is_empty() {
            return layout;
        }

        // Create separator
        if props.show_separator {
            let separator_width = content_width * props.separator_length;
            let separator_y = footnote_area_y;

            layout.separator = Some(FootnoteSeparator::new(
                page_bounds.x,
                separator_y,
                separator_width,
                props.separator_weight,
            ));
        }

        // Layout footnotes
        let separator_height = layout.separator.as_ref().map(|s| s.height()).unwrap_or(0.0);
        let mut current_y = footnote_area_y + separator_height;

        for note in notes {
            let line_height = self.footnote_font_size * self.line_height_multiplier;
            let para_count = note.content().len().max(1);
            let footnote_height = (para_count as f32 * line_height * 2.0).max(line_height);

            let footnote_bounds = Rect::new(
                page_bounds.x,
                current_y,
                content_width,
                footnote_height,
            );

            let footnote_layout = FootnoteBlockLayout::new(
                note.id(),
                note.mark().to_string(),
                footnote_bounds,
            );

            layout.add_footnote(footnote_layout);
            current_y += footnote_height + self.footnote_spacing;
        }

        layout
    }

    /// Check if a footnote needs to be split across pages
    pub fn should_split_footnote(&self, footnote_height: f32, available_height: f32) -> bool {
        // Don't split if the footnote is small enough
        if footnote_height <= available_height {
            return false;
        }

        // Also don't split if available height is too small (move entire footnote)
        let min_height = self.footnote_font_size * self.line_height_multiplier * 2.0;
        available_height >= min_height
    }
}

// =============================================================================
// Endnote Layouter
// =============================================================================

/// Handles endnote layout calculations
pub struct EndnoteLayouter {
    /// Default font size for endnotes
    pub endnote_font_size: f32,
    /// Default line height multiplier
    pub line_height_multiplier: f32,
    /// Indent for continuation lines
    pub hanging_indent: f32,
    /// Space between endnotes
    pub endnote_spacing: f32,
    /// Heading font size
    pub heading_font_size: f32,
    /// Space after heading
    pub heading_space_after: f32,
}

impl Default for EndnoteLayouter {
    fn default() -> Self {
        Self {
            endnote_font_size: 10.0,
            line_height_multiplier: 1.2,
            hanging_indent: 18.0,
            endnote_spacing: 4.0,
            heading_font_size: 14.0,
            heading_space_after: 12.0,
        }
    }
}

impl EndnoteLayouter {
    /// Create a new endnote layouter
    pub fn new() -> Self {
        Self::default()
    }

    /// Create endnote section layout
    pub fn layout_endnotes(
        &self,
        notes: &[&Note],
        props: &EndnoteProperties,
        bounds: Rect,
        is_section_end: bool,
    ) -> EndnoteSectionLayout {
        let mut layout = EndnoteSectionLayout::new(bounds, is_section_end);

        if notes.is_empty() {
            return layout;
        }

        let line_height = self.endnote_font_size * self.line_height_multiplier;
        let mut current_y = bounds.y + self.heading_font_size + self.heading_space_after;

        for note in notes {
            let para_count = note.content().len().max(1);
            let endnote_height = (para_count as f32 * line_height * 2.0).max(line_height);

            let endnote_bounds = Rect::new(
                bounds.x,
                current_y,
                bounds.width,
                endnote_height,
            );

            let endnote_layout = EndnoteBlockLayout::new(
                note.id(),
                note.mark().to_string(),
                endnote_bounds,
            );

            layout.add_endnote(endnote_layout);
            current_y += endnote_height + self.endnote_spacing;
        }

        layout
    }

    /// Calculate total height needed for endnotes
    pub fn calculate_endnote_height(&self, notes: &[&Note]) -> f32 {
        if notes.is_empty() {
            return 0.0;
        }

        let line_height = self.endnote_font_size * self.line_height_multiplier;
        let heading_height = self.heading_font_size + self.heading_space_after;

        let total_endnote_height: f32 = notes
            .iter()
            .map(|note| {
                let para_count = note.content().len().max(1);
                (para_count as f32 * line_height * 2.0).max(line_height) + self.endnote_spacing
            })
            .sum();

        heading_height + total_endnote_height
    }
}

// =============================================================================
// Page Layout Integration
// =============================================================================

/// Extended page layout information including footnotes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageFootnoteInfo {
    /// Footnotes on this page
    pub footnote_area: Option<FootnoteAreaLayout>,
    /// Whether this page has footnote continuation
    pub has_continuation: bool,
    /// Height reserved for footnotes
    pub reserved_height: f32,
}

impl PageFootnoteInfo {
    /// Create a new page footnote info
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the footnote area
    pub fn set_footnote_area(&mut self, area: FootnoteAreaLayout) {
        self.reserved_height = area.total_height();
        self.has_continuation = area.has_continuation;
        self.footnote_area = Some(area);
    }
}

// =============================================================================
// Continuation Notice
// =============================================================================

/// Layout for a continuation notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationNotice {
    /// The notice text
    pub text: String,
    /// Bounds of the notice
    pub bounds: Rect,
    /// Whether this is "continued from" or "continued to"
    pub is_continuation_from: bool,
}

impl ContinuationNotice {
    /// Create a "continued on next page" notice
    pub fn continued_to(bounds: Rect) -> Self {
        Self {
            text: "continued on next page".to_string(),
            bounds,
            is_continuation_from: false,
        }
    }

    /// Create a "continued from previous page" notice
    pub fn continued_from(bounds: Rect) -> Self {
        Self {
            text: "continued from previous page".to_string(),
            bounds,
            is_continuation_from: true,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_note() -> Note {
        let mut note = Note::footnote();
        note.set_mark("1");
        note
    }

    #[test]
    fn test_footnote_area_layout_creation() {
        let bounds = Rect::new(72.0, 600.0, 468.0, 100.0);
        let layout = FootnoteAreaLayout::new(bounds);

        assert!(layout.is_empty());
        assert_eq!(layout.total_height(), 0.0);
    }

    #[test]
    fn test_footnote_separator() {
        let separator = FootnoteSeparator::new(72.0, 600.0, 156.0, 0.5);

        assert_eq!(separator.x, 72.0);
        assert_eq!(separator.width, 156.0);
        assert_eq!(separator.weight, 0.5);
        assert!(separator.height() > 0.0);
    }

    #[test]
    fn test_footnote_block_layout() {
        let note_id = NoteId::new();
        let bounds = Rect::new(72.0, 620.0, 468.0, 24.0);
        let block = FootnoteBlockLayout::new(note_id, "1".to_string(), bounds);

        assert_eq!(block.mark, "1");
        assert_eq!(block.height, 24.0);
        assert!(!block.is_continuation);
    }

    #[test]
    fn test_footnote_layouter_calculate_space() {
        let layouter = FootnoteLayouter::new();
        let note = create_test_note();
        let notes: Vec<&Note> = vec![&note];
        let props = FootnoteProperties::default();

        let space = layouter.calculate_footnote_space(&notes, &props, 468.0);
        assert!(space > 0.0);
    }

    #[test]
    fn test_footnote_layouter_empty() {
        let layouter = FootnoteLayouter::new();
        let notes: Vec<&Note> = vec![];
        let props = FootnoteProperties::default();

        let space = layouter.calculate_footnote_space(&notes, &props, 468.0);
        assert_eq!(space, 0.0);
    }

    #[test]
    fn test_endnote_section_layout() {
        let bounds = Rect::new(72.0, 100.0, 468.0, 500.0);
        let layout = EndnoteSectionLayout::new(bounds, false);

        assert!(!layout.is_section_end);
        assert!(layout.is_empty());
        assert_eq!(layout.heading, Some("Notes".to_string()));
    }

    #[test]
    fn test_endnote_layouter() {
        let layouter = EndnoteLayouter::new();
        let mut note = Note::endnote();
        note.set_mark("i");
        let notes: Vec<&Note> = vec![&note];
        let props = EndnoteProperties::default();
        let bounds = Rect::new(72.0, 100.0, 468.0, 500.0);

        let layout = layouter.layout_endnotes(&notes, &props, bounds, false);

        assert!(!layout.is_empty());
        assert_eq!(layout.endnotes.len(), 1);
        assert_eq!(layout.endnotes[0].mark, "i");
    }

    #[test]
    fn test_page_footnote_info() {
        let mut info = PageFootnoteInfo::new();
        let bounds = Rect::new(72.0, 600.0, 468.0, 100.0);
        let mut area = FootnoteAreaLayout::new(bounds);

        let note_id = NoteId::new();
        let fn_bounds = Rect::new(72.0, 620.0, 468.0, 24.0);
        area.add_footnote(FootnoteBlockLayout::new(note_id, "1".to_string(), fn_bounds));

        info.set_footnote_area(area);

        assert!(info.footnote_area.is_some());
        assert!(info.reserved_height > 0.0);
    }

    #[test]
    fn test_continuation_notice() {
        let bounds = Rect::new(72.0, 700.0, 468.0, 12.0);

        let to_notice = ContinuationNotice::continued_to(bounds);
        assert!(!to_notice.is_continuation_from);

        let from_notice = ContinuationNotice::continued_from(bounds);
        assert!(from_notice.is_continuation_from);
    }

    #[test]
    fn test_footnote_split_decision() {
        let layouter = FootnoteLayouter::new();

        // Small footnote should not split
        assert!(!layouter.should_split_footnote(20.0, 100.0));

        // Large footnote that exceeds available space should split
        assert!(layouter.should_split_footnote(200.0, 100.0));

        // Large footnote but available space too small
        assert!(!layouter.should_split_footnote(200.0, 10.0));
    }
}
