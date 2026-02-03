//! Line Breaking Algorithm (D1)
//!
//! This module implements the line breaking algorithm for the MS Word document editor.
//! It uses Unicode line breaking rules (UAX #14) to find break opportunities and
//! integrates with the text shaping system for accurate glyph measurements.
//!
//! The algorithm follows a greedy approach:
//! 1. Collect text from all runs in a paragraph
//! 2. Shape the text to get accurate glyph widths
//! 3. Find Unicode break opportunities
//! 4. Fill lines greedily, breaking at allowed positions
//! 5. Calculate proper line metrics for mixed content

use crate::{BidiAnalyzer, BidiRun, Direction, InlineBox, LineBox, ListMarkerInfo, Rect, Result};
use doc_model::{Alignment, DocumentTree, LineSpacing, Node, NodeId, NumId};
use text_engine::{FontManager, ShapedRun, TextShaper};

/// Unicode line break opportunity types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakOpportunity {
    /// No break allowed at this position
    NoBreak,
    /// Break allowed (after space, punctuation, etc.)
    Allowed,
    /// Break required (after hard line break)
    Mandatory,
}

/// A segment of shaped text that can be placed on a line
#[derive(Debug, Clone)]
pub struct ShapedSegment {
    /// The run ID this segment belongs to
    pub run_id: NodeId,
    /// Start byte offset within the run's text
    pub start_offset: usize,
    /// End byte offset within the run's text
    pub end_offset: usize,
    /// Width of this segment in points
    pub width: f32,
    /// Ascender height for this segment
    pub ascender: f32,
    /// Descender depth for this segment
    pub descender: f32,
    /// Whether this segment is whitespace-only
    pub is_whitespace: bool,
    /// Break opportunity after this segment
    pub break_after: BreakOpportunity,
    /// BiDi embedding level (even = LTR, odd = RTL)
    pub bidi_level: u8,
    /// Direction of this segment
    pub direction: Direction,
}

/// An inline image segment for line layout
#[derive(Debug, Clone)]
pub struct ImageSegment {
    /// The image node ID
    pub node_id: NodeId,
    /// Width in points
    pub width: f32,
    /// Height in points
    pub height: f32,
    /// Break opportunity after this image
    pub break_after: BreakOpportunity,
}

/// A list marker layout item
#[derive(Debug, Clone)]
pub struct ListMarkerSegment {
    /// The paragraph node ID this marker belongs to
    pub para_id: NodeId,
    /// The formatted marker text
    pub text: String,
    /// Width of the marker
    pub width: f32,
    /// Height of the marker
    pub height: f32,
    /// Ascender
    pub ascender: f32,
    /// Descender
    pub descender: f32,
    /// Font family for the marker
    pub font: Option<String>,
    /// Whether this is a bullet marker
    pub is_bullet: bool,
    /// The list level
    pub level: u8,
}

/// A layout item that can be placed on a line (text or image)
#[derive(Debug, Clone)]
pub enum LayoutItem {
    /// A text segment
    Text(ShapedSegment),
    /// An inline image
    Image(ImageSegment),
    /// A list marker (bullet or number)
    ListMarker(ListMarkerSegment),
}

impl LayoutItem {
    /// Get the width of this item
    pub fn width(&self) -> f32 {
        match self {
            Self::Text(seg) => seg.width,
            Self::Image(img) => img.width,
            Self::ListMarker(marker) => marker.width,
        }
    }

    /// Get the height/ascender of this item
    pub fn ascender(&self) -> f32 {
        match self {
            Self::Text(seg) => seg.ascender,
            Self::Image(img) => img.height, // Image sits on baseline
            Self::ListMarker(marker) => marker.ascender,
        }
    }

    /// Get the descender of this item
    pub fn descender(&self) -> f32 {
        match self {
            Self::Text(seg) => seg.descender,
            Self::Image(_) => 0.0, // Images sit on the baseline
            Self::ListMarker(marker) => marker.descender,
        }
    }

    /// Check if this is whitespace
    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::Text(seg) => seg.is_whitespace,
            Self::Image(_) => false,
            Self::ListMarker(_) => false,
        }
    }

    /// Get break opportunity after this item
    pub fn break_after(&self) -> BreakOpportunity {
        match self {
            Self::Text(seg) => seg.break_after,
            Self::Image(img) => img.break_after,
            Self::ListMarker(_) => BreakOpportunity::NoBreak, // No break after marker
        }
    }

    /// Get the node ID
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Text(seg) => seg.run_id,
            Self::Image(img) => img.node_id,
            Self::ListMarker(marker) => marker.para_id,
        }
    }

    /// Get BiDi level (images and markers default to 0 = LTR)
    pub fn bidi_level(&self) -> u8 {
        match self {
            Self::Text(seg) => seg.bidi_level,
            Self::Image(_) => 0,
            Self::ListMarker(_) => 0,
        }
    }

    /// Get direction
    pub fn direction(&self) -> Direction {
        match self {
            Self::Text(seg) => seg.direction,
            Self::Image(_) => Direction::Ltr,
            Self::ListMarker(_) => Direction::Ltr,
        }
    }

    /// Check if this is a list marker
    pub fn is_list_marker(&self) -> bool {
        matches!(self, Self::ListMarker(_))
    }
}

/// Result of breaking a paragraph into lines
#[derive(Debug)]
pub struct BrokenParagraph {
    /// The laid out lines
    pub lines: Vec<LineBox>,
    /// Total height of all lines
    pub total_height: f32,
}

/// Configuration for line breaking
#[derive(Debug, Clone)]
pub struct LineBreakConfig {
    /// Available width for text
    pub available_width: f32,
    /// Default font size in points
    pub font_size: f32,
    /// Line spacing multiplier or configuration
    pub line_spacing: f32,
    /// First line indent in points
    pub first_line_indent: f32,
    /// Left indent in points
    pub left_indent: f32,
    /// Right indent in points
    pub right_indent: f32,
    /// Paragraph direction
    pub direction: Direction,
    /// Whether to allow hyphenation
    pub allow_hyphenation: bool,
    /// Paragraph alignment
    pub alignment: Alignment,
    /// List numbering instance ID (if paragraph is in a list)
    pub list_num_id: Option<NumId>,
    /// List indent level (0-8)
    pub list_level: Option<u8>,
    /// List marker text (pre-computed)
    pub list_marker_text: Option<String>,
    /// Whether the list marker is a bullet
    pub list_is_bullet: bool,
    /// List marker font
    pub list_marker_font: Option<String>,
    /// Hanging indent for list (space for marker)
    pub list_hanging: f32,
}

impl Default for LineBreakConfig {
    fn default() -> Self {
        Self {
            available_width: 468.0, // 6.5 inches at 72 dpi
            font_size: 12.0,
            line_spacing: 1.0,
            first_line_indent: 0.0,
            left_indent: 0.0,
            right_indent: 0.0,
            direction: Direction::Ltr,
            allow_hyphenation: false,
            alignment: Alignment::Left,
            list_num_id: None,
            list_level: None,
            list_marker_text: None,
            list_is_bullet: false,
            list_marker_font: None,
            list_hanging: 0.0,
        }
    }
}

/// Information about a run for line breaking
#[derive(Debug, Clone)]
struct RunInfo {
    /// The run's node ID
    run_id: NodeId,
    /// Start byte offset in the full paragraph text
    text_start: usize,
    /// End byte offset in the full paragraph text
    text_end: usize,
    /// Font size for this run
    font_size: f32,
    /// Font family for this run
    font_family: Option<String>,
    /// Whether bold
    bold: bool,
    /// Whether italic
    italic: bool,
}

/// A pending line being built
struct PendingLine {
    /// Segments on this line
    segments: Vec<ShapedSegment>,
    /// Current width (excluding trailing whitespace)
    content_width: f32,
    /// Width including trailing whitespace
    total_width: f32,
    /// Maximum ascender on this line
    max_ascender: f32,
    /// Maximum descender on this line
    max_descender: f32,
    /// Is this the first line of the paragraph
    is_first_line: bool,
}

impl PendingLine {
    fn new(is_first_line: bool) -> Self {
        Self {
            segments: Vec::new(),
            content_width: 0.0,
            total_width: 0.0,
            max_ascender: 0.0,
            max_descender: 0.0,
            is_first_line,
        }
    }

    fn add_segment(&mut self, segment: ShapedSegment) {
        self.max_ascender = self.max_ascender.max(segment.ascender);
        self.max_descender = self.max_descender.max(segment.descender);

        if segment.is_whitespace {
            self.total_width += segment.width;
        } else {
            self.content_width = self.total_width + segment.width;
            self.total_width = self.content_width;
        }

        self.segments.push(segment);
    }

    fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Calculate line height based on the tallest content
    fn line_height(&self, line_spacing: f32, default_font_size: f32) -> f32 {
        if self.max_ascender == 0.0 && self.max_descender == 0.0 {
            // Empty line - use default font size
            default_font_size * 1.2 * line_spacing
        } else {
            (self.max_ascender + self.max_descender) * line_spacing
        }
    }

    /// Calculate baseline offset from top of line
    fn baseline(&self, default_font_size: f32) -> f32 {
        if self.max_ascender == 0.0 {
            default_font_size * 0.8
        } else {
            self.max_ascender
        }
    }
}

/// Line breaker that converts paragraphs into lines
///
/// The line breaker integrates text shaping with Unicode line breaking rules
/// to produce properly laid out lines of text.
pub struct LineBreaker {
    /// Text shaper for measuring glyphs
    shaper: TextShaper,
    /// Font manager for font metrics
    font_manager: FontManager,
}

impl LineBreaker {
    /// Create a new line breaker
    pub fn new() -> Self {
        Self {
            shaper: TextShaper::new(),
            font_manager: FontManager::new(),
        }
    }

    /// Create a line breaker with custom shaper and font manager
    pub fn with_engines(shaper: TextShaper, font_manager: FontManager) -> Self {
        Self { shaper, font_manager }
    }

    /// Get a mutable reference to the text shaper
    pub fn shaper_mut(&mut self) -> &mut TextShaper {
        &mut self.shaper
    }

    /// Get a mutable reference to the font manager
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// Break a paragraph into lines
    pub fn break_paragraph(
        &mut self,
        tree: &DocumentTree,
        para_id: NodeId,
        config: &LineBreakConfig,
    ) -> Result<BrokenParagraph> {
        let para = tree.get_paragraph(para_id)
            .ok_or_else(|| crate::LayoutError::LayoutFailed("Paragraph not found".into()))?;

        // Collect layout items (text runs and inline images)
        let mut layout_items: Vec<LayoutItem> = Vec::new();
        let mut full_text = String::new();
        let mut run_infos: Vec<RunInfo> = Vec::new();

        // Add list marker if this paragraph is in a list
        if let Some(marker_text) = &config.list_marker_text {
            if !marker_text.is_empty() {
                // Measure the marker text
                let marker_font_size = config.font_size;
                let marker_shaped = self.shaper.shape(marker_text, marker_font_size)
                    .unwrap_or_else(|_| ShapedRun {
                        glyphs: Vec::new(),
                        width: marker_text.len() as f32 * marker_font_size * 0.6,
                        font_size: marker_font_size,
                        units_per_em: 1000,
                        ascender: marker_font_size * 0.8,
                        descender: marker_font_size * 0.2,
                        line_gap: 0.0,
                    });

                layout_items.push(LayoutItem::ListMarker(ListMarkerSegment {
                    para_id,
                    text: marker_text.clone(),
                    width: marker_shaped.width,
                    height: marker_shaped.ascender + marker_shaped.descender,
                    ascender: marker_shaped.ascender,
                    descender: marker_shaped.descender,
                    font: config.list_marker_font.clone(),
                    is_bullet: config.list_is_bullet,
                    level: config.list_level.unwrap_or(0),
                }));
            }
        }

        for &child_id in para.children() {
            // Check if it's a text run
            if let Some(run) = tree.get_run(child_id) {
                let start = full_text.len();
                full_text.push_str(&run.text);
                let end = full_text.len();

                run_infos.push(RunInfo {
                    run_id: child_id,
                    text_start: start,
                    text_end: end,
                    font_size: run.style.font_size.unwrap_or(config.font_size),
                    font_family: run.style.font_family.clone(),
                    bold: run.style.bold.unwrap_or(false),
                    italic: run.style.italic.unwrap_or(false),
                });
            }
            // Check if it's an inline image
            else if let Some(image) = tree.get_image(child_id) {
                // Only process inline images here
                if image.is_inline() {
                    // First, process any pending text runs
                    if !run_infos.is_empty() {
                        let break_opportunities = self.find_break_opportunities(&full_text);
                        let segments = self.create_segments(&full_text, &run_infos, &break_opportunities, config)?;
                        for seg in segments {
                            layout_items.push(LayoutItem::Text(seg));
                        }
                        run_infos.clear();
                        full_text.clear();
                    }

                    // Add the image as a layout item
                    let img_width = image.effective_width(config.available_width);
                    let img_height = image.effective_height(config.available_width);
                    layout_items.push(LayoutItem::Image(ImageSegment {
                        node_id: child_id,
                        width: img_width,
                        height: img_height,
                        break_after: BreakOpportunity::Allowed,
                    }));
                }
            }
        }

        // Process any remaining text runs
        if !run_infos.is_empty() {
            let break_opportunities = self.find_break_opportunities(&full_text);
            let segments = self.create_segments(&full_text, &run_infos, &break_opportunities, config)?;
            for seg in segments {
                layout_items.push(LayoutItem::Text(seg));
            }
        }

        // Handle empty paragraph
        if layout_items.is_empty() {
            return self.create_empty_paragraph(config);
        }

        // Get line spacing multiplier from paragraph style
        let line_spacing = match para.style.line_spacing {
            Some(LineSpacing::Multiple(m)) => m,
            Some(LineSpacing::Exact(_)) => 1.0, // Will be handled differently
            Some(LineSpacing::AtLeast(_)) => config.line_spacing,
            None => config.line_spacing,
        };

        // Break into lines using greedy algorithm (now handles both text and images)
        let lines = self.greedy_line_break_items(layout_items, config, line_spacing)?;

        // Calculate total height
        let total_height = lines.iter().map(|l| l.bounds.height).sum();

        Ok(BrokenParagraph { lines, total_height })
    }

    /// Create an empty paragraph with a single empty line
    fn create_empty_paragraph(&self, config: &LineBreakConfig) -> Result<BrokenParagraph> {
        let line_height = config.font_size * config.line_spacing * 1.2;
        let baseline = config.font_size * 0.8;

        let line = LineBox {
            bounds: Rect::new(
                config.left_indent,
                0.0,
                config.available_width - config.left_indent - config.right_indent,
                line_height,
            ),
            baseline,
            direction: config.direction,
            inlines: Vec::new(),
        };

        Ok(BrokenParagraph {
            lines: vec![line],
            total_height: line_height,
        })
    }

    /// Find Unicode line break opportunities using UAX #14
    fn find_break_opportunities(&self, text: &str) -> Vec<BreakOpportunity> {
        use unicode_linebreak::{linebreaks, BreakOpportunity as UnicodeBreak};

        let mut opportunities = vec![BreakOpportunity::NoBreak; text.len()];

        for (offset, break_type) in linebreaks(text) {
            if offset > 0 && offset <= opportunities.len() {
                opportunities[offset - 1] = match break_type {
                    UnicodeBreak::Mandatory => BreakOpportunity::Mandatory,
                    UnicodeBreak::Allowed => BreakOpportunity::Allowed,
                };
            }
        }

        opportunities
    }

    /// Create shaped segments from the text
    fn create_segments(
        &self,
        full_text: &str,
        run_infos: &[RunInfo],
        break_opportunities: &[BreakOpportunity],
        config: &LineBreakConfig,
    ) -> Result<Vec<ShapedSegment>> {
        let mut segments = Vec::new();

        // Get BiDi levels for the full text
        let bidi_analyzer = BidiAnalyzer::new();
        let bidi_levels = bidi_analyzer.get_levels(full_text, config.direction);

        for run_info in run_infos {
            let run_text = &full_text[run_info.text_start..run_info.text_end];
            if run_text.is_empty() {
                continue;
            }

            // Shape the run text
            let shaped = self.shaper.shape_run(
                run_text,
                run_info.font_family.as_deref(),
                run_info.font_size,
                run_info.bold,
                run_info.italic,
            ).unwrap_or_else(|_| {
                // Fallback to basic shaping
                self.shaper.shape(run_text, run_info.font_size).unwrap_or_else(|_| {
                    // Ultimate fallback
                    ShapedRun {
                        glyphs: Vec::new(),
                        width: run_text.len() as f32 * run_info.font_size * 0.6,
                        font_size: run_info.font_size,
                        units_per_em: 1000,
                        ascender: run_info.font_size * 0.8,
                        descender: run_info.font_size * 0.2,
                        line_gap: 0.0,
                    }
                })
            });

            // Split run into segments at break opportunities
            let run_segments = self.split_into_segments(
                run_info,
                run_text,
                &shaped,
                break_opportunities,
                &bidi_levels,
            );

            segments.extend(run_segments);
        }

        Ok(segments)
    }

    /// Split a shaped run into segments at break opportunities
    fn split_into_segments(
        &self,
        run_info: &RunInfo,
        run_text: &str,
        shaped: &ShapedRun,
        break_opportunities: &[BreakOpportunity],
        bidi_levels: &[u8],
    ) -> Vec<ShapedSegment> {
        let mut segments = Vec::new();
        let mut segment_start = 0;

        // Iterate through characters and their byte positions
        let char_indices: Vec<(usize, char)> = run_text.char_indices().collect();

        for (idx, (byte_pos, _ch)) in char_indices.iter().enumerate() {
            let global_pos = run_info.text_start + byte_pos;
            let break_after = if global_pos < break_opportunities.len() {
                break_opportunities[global_pos]
            } else {
                BreakOpportunity::NoBreak
            };

            // Check if this is a break opportunity or mandatory break
            let is_end = idx == char_indices.len() - 1;
            let should_segment = break_after != BreakOpportunity::NoBreak || is_end;

            if should_segment {
                let segment_end = if is_end {
                    run_text.len()
                } else {
                    // Find the next character's byte position
                    char_indices.get(idx + 1).map(|(pos, _)| *pos).unwrap_or(run_text.len())
                };

                // Calculate segment width from shaped glyphs
                let segment_text = &run_text[segment_start..segment_end];
                let width = self.calculate_segment_width(
                    shaped,
                    segment_start,
                    segment_end,
                    segment_text,
                );

                let is_whitespace = segment_text.chars().all(|c| c.is_whitespace());

                // Get the BiDi level for this segment (use the level at the start of the segment)
                let segment_global_start = run_info.text_start + segment_start;
                let bidi_level = bidi_levels
                    .get(segment_global_start)
                    .copied()
                    .unwrap_or(0);
                let direction = if bidi_level % 2 == 0 {
                    Direction::Ltr
                } else {
                    Direction::Rtl
                };

                segments.push(ShapedSegment {
                    run_id: run_info.run_id,
                    start_offset: segment_start,
                    end_offset: segment_end,
                    width,
                    ascender: shaped.ascender,
                    descender: shaped.descender,
                    is_whitespace,
                    break_after: if is_end && break_after == BreakOpportunity::NoBreak {
                        BreakOpportunity::Allowed // Allow break at end of run
                    } else {
                        break_after
                    },
                    bidi_level,
                    direction,
                });

                segment_start = segment_end;
            }
        }

        // Handle case where no segments were created (shouldn't happen normally)
        if segments.is_empty() && !run_text.is_empty() {
            // Get the BiDi level for the entire segment
            let bidi_level = bidi_levels
                .get(run_info.text_start)
                .copied()
                .unwrap_or(0);
            let direction = if bidi_level % 2 == 0 {
                Direction::Ltr
            } else {
                Direction::Rtl
            };

            segments.push(ShapedSegment {
                run_id: run_info.run_id,
                start_offset: 0,
                end_offset: run_text.len(),
                width: shaped.width,
                ascender: shaped.ascender,
                descender: shaped.descender,
                is_whitespace: run_text.chars().all(|c| c.is_whitespace()),
                break_after: BreakOpportunity::Allowed,
                bidi_level,
                direction,
            });
        }

        segments
    }

    /// Calculate width for a segment from shaped glyphs
    fn calculate_segment_width(
        &self,
        shaped: &ShapedRun,
        start_byte: usize,
        end_byte: usize,
        segment_text: &str,
    ) -> f32 {
        // If we have glyphs, use them to calculate width
        if !shaped.glyphs.is_empty() {
            let mut width = 0.0;
            for glyph in &shaped.glyphs {
                let cluster = glyph.cluster as usize;
                if cluster >= start_byte && cluster < end_byte {
                    width += glyph.advance_width(shaped.font_size, shaped.units_per_em);
                }
            }
            if width > 0.0 {
                return width;
            }
        }

        // Fallback: estimate based on character count
        segment_text.chars().count() as f32 * shaped.font_size * 0.6
    }

    /// Perform greedy line breaking
    fn greedy_line_break(
        &self,
        segments: Vec<ShapedSegment>,
        config: &LineBreakConfig,
        line_spacing: f32,
    ) -> Result<Vec<LineBox>> {
        let mut lines = Vec::new();
        let mut current_line = PendingLine::new(true);
        let mut current_y = 0.0;

        let base_available = config.available_width - config.left_indent - config.right_indent;
        let total_segments = segments.len();

        for (seg_index, segment) in segments.into_iter().enumerate() {
            let available_width = if current_line.is_first_line {
                base_available - config.first_line_indent
            } else {
                base_available
            };

            // Check if we need to start a new line
            let would_overflow = current_line.total_width + segment.width > available_width
                && !current_line.is_empty()
                && !segment.is_whitespace;

            let is_mandatory_break = !current_line.is_empty()
                && current_line.segments.last()
                    .map(|s| s.break_after == BreakOpportunity::Mandatory)
                    .unwrap_or(false);

            if would_overflow || is_mandatory_break {
                // Finalize current line (not the last line for justify purposes)
                let is_last_line = false;
                let line = self.finalize_line(
                    &current_line,
                    current_y,
                    config,
                    line_spacing,
                    is_last_line,
                );
                current_y += line.bounds.height;
                lines.push(line);

                // Start new line
                current_line = PendingLine::new(false);
            }

            // Handle mandatory breaks that occur mid-segment (e.g., newline characters)
            if segment.break_after == BreakOpportunity::Mandatory {
                current_line.add_segment(segment);
                // Lines ending with mandatory break are not justified
                let is_last_line = true;
                let line = self.finalize_line(
                    &current_line,
                    current_y,
                    config,
                    line_spacing,
                    is_last_line,
                );
                current_y += line.bounds.height;
                lines.push(line);
                current_line = PendingLine::new(false);
            } else {
                current_line.add_segment(segment);
            }
        }

        // Finalize last line if it has content
        if !current_line.is_empty() {
            // Last line is never justified
            let is_last_line = true;
            let line = self.finalize_line(
                &current_line,
                current_y,
                config,
                line_spacing,
                is_last_line,
            );
            lines.push(line);
        }

        // Ensure at least one line exists
        if lines.is_empty() {
            let empty_line = LineBox {
                bounds: Rect::new(
                    config.left_indent,
                    0.0,
                    base_available,
                    config.font_size * line_spacing * 1.2,
                ),
                baseline: config.font_size * 0.8,
                direction: config.direction,
                inlines: Vec::new(),
            };
            lines.push(empty_line);
        }

        Ok(lines)
    }

    /// Finalize a pending line into a LineBox
    fn finalize_line(
        &self,
        pending: &PendingLine,
        y_offset: f32,
        config: &LineBreakConfig,
        line_spacing: f32,
        is_last_line: bool,
    ) -> LineBox {
        let line_height = pending.line_height(line_spacing, config.font_size);
        let baseline = pending.baseline(config.font_size);

        let available_width = config.available_width - config.left_indent - config.right_indent
            - if pending.is_first_line { config.first_line_indent } else { 0.0 };

        // Strip trailing whitespace segments for layout purposes
        let mut segments_to_render: Vec<&ShapedSegment> = pending.segments.iter().collect();
        while let Some(last) = segments_to_render.last() {
            if last.is_whitespace {
                segments_to_render.pop();
            } else {
                break;
            }
        }

        // Calculate total content width
        let total_content_width: f32 = segments_to_render.iter().map(|s| s.width).sum();

        // Reorder segments for visual display using BiDi algorithm
        let bidi_analyzer = BidiAnalyzer::new();
        let bidi_runs: Vec<BidiRun> = segments_to_render
            .iter()
            .enumerate()
            .map(|(i, seg)| BidiRun::new(i, i + 1, seg.bidi_level))
            .collect();
        let visual_order = bidi_analyzer.visual_order(&bidi_runs);

        // Reorder segments according to visual order
        let reordered_segments: Vec<&ShapedSegment> = visual_order
            .iter()
            .filter_map(|&idx| segments_to_render.get(idx).copied())
            .collect();

        // Create inline boxes from segments
        let mut inlines = Vec::new();

        // Calculate extra space for alignment
        let extra_space = (available_width - total_content_width).max(0.0);

        // Determine starting X position based on alignment and direction
        let first_line_indent = if pending.is_first_line { config.first_line_indent } else { 0.0 };
        let base_x = config.left_indent + first_line_indent;

        let (x_start, word_spacing_extra) = match config.alignment {
            Alignment::Left => {
                if config.direction == Direction::Rtl {
                    // RTL with left alignment starts from right
                    (base_x + extra_space, 0.0)
                } else {
                    (base_x, 0.0)
                }
            }
            Alignment::Center => {
                (base_x + extra_space / 2.0, 0.0)
            }
            Alignment::Right => {
                if config.direction == Direction::Rtl {
                    (base_x, 0.0)
                } else {
                    (base_x + extra_space, 0.0)
                }
            }
            Alignment::Justify => {
                // Don't justify last line or lines with mandatory breaks
                if is_last_line || reordered_segments.len() <= 1 {
                    (base_x, 0.0)
                } else {
                    // Count word gaps (between segments that are not whitespace)
                    let word_count = reordered_segments.len();
                    let gap_count = if word_count > 1 { word_count - 1 } else { 0 };
                    let spacing = if gap_count > 0 {
                        extra_space / gap_count as f32
                    } else {
                        0.0
                    };
                    (base_x, spacing)
                }
            }
        };

        let mut x = x_start;

        for (i, segment) in reordered_segments.iter().enumerate() {
            // Calculate the vertical offset to align baselines
            let y_offset_inline = baseline - segment.ascender;

            inlines.push(InlineBox::text(
                segment.run_id,
                Rect::new(
                    x,
                    y_offset_inline,
                    segment.width,
                    segment.ascender + segment.descender,
                ),
                segment.direction,
                segment.start_offset,
                segment.end_offset,
            ));

            x += segment.width;

            // Add extra spacing for justify alignment between words
            if config.alignment == Alignment::Justify && i < reordered_segments.len() - 1 {
                x += word_spacing_extra;
            }
        }

        LineBox {
            bounds: Rect::new(
                config.left_indent,
                y_offset,
                available_width,
                line_height,
            ),
            baseline,
            direction: config.direction,
            inlines,
        }
    }

    /// Perform greedy line breaking with layout items (text + images)
    fn greedy_line_break_items(
        &self,
        items: Vec<LayoutItem>,
        config: &LineBreakConfig,
        line_spacing: f32,
    ) -> Result<Vec<LineBox>> {
        use crate::InlineType;

        let mut lines = Vec::new();
        let mut current_items: Vec<LayoutItem> = Vec::new();
        let mut current_width = 0.0;
        let mut current_max_ascender = 0.0f32;
        let mut current_max_descender = 0.0f32;
        let mut current_y = 0.0;
        let mut is_first_line = true;

        let base_available = config.available_width - config.left_indent - config.right_indent;

        for item in items {
            let available_width = if is_first_line {
                base_available - config.first_line_indent
            } else {
                base_available
            };

            // Check if we need to start a new line
            let would_overflow = current_width + item.width() > available_width
                && !current_items.is_empty()
                && !item.is_whitespace();

            let is_mandatory_break = !current_items.is_empty()
                && current_items.last()
                    .map(|i| i.break_after() == BreakOpportunity::Mandatory)
                    .unwrap_or(false);

            if would_overflow || is_mandatory_break {
                // Finalize current line
                let line = self.finalize_line_items(
                    &current_items,
                    current_y,
                    config,
                    line_spacing,
                    current_max_ascender,
                    current_max_descender,
                    is_first_line,
                    false, // not last line
                );
                current_y += line.bounds.height;
                lines.push(line);

                // Start new line
                current_items.clear();
                current_width = 0.0;
                current_max_ascender = 0.0;
                current_max_descender = 0.0;
                is_first_line = false;
            }

            // Update metrics
            current_max_ascender = current_max_ascender.max(item.ascender());
            current_max_descender = current_max_descender.max(item.descender());

            if item.is_whitespace() {
                current_width += item.width();
            } else {
                current_width += item.width();
            }

            // Handle mandatory breaks
            if item.break_after() == BreakOpportunity::Mandatory {
                current_items.push(item);
                let line = self.finalize_line_items(
                    &current_items,
                    current_y,
                    config,
                    line_spacing,
                    current_max_ascender,
                    current_max_descender,
                    is_first_line,
                    true, // treat as last for justify
                );
                current_y += line.bounds.height;
                lines.push(line);

                current_items.clear();
                current_width = 0.0;
                current_max_ascender = 0.0;
                current_max_descender = 0.0;
                is_first_line = false;
            } else {
                current_items.push(item);
            }
        }

        // Finalize last line if it has content
        if !current_items.is_empty() {
            let line = self.finalize_line_items(
                &current_items,
                current_y,
                config,
                line_spacing,
                current_max_ascender,
                current_max_descender,
                is_first_line,
                true, // last line
            );
            lines.push(line);
        }

        // Ensure at least one line exists
        if lines.is_empty() {
            let empty_line = LineBox {
                bounds: Rect::new(
                    config.left_indent,
                    0.0,
                    base_available,
                    config.font_size * line_spacing * 1.2,
                ),
                baseline: config.font_size * 0.8,
                direction: config.direction,
                inlines: Vec::new(),
            };
            lines.push(empty_line);
        }

        Ok(lines)
    }

    /// Finalize a line from layout items into a LineBox
    fn finalize_line_items(
        &self,
        items: &[LayoutItem],
        y_offset: f32,
        config: &LineBreakConfig,
        line_spacing: f32,
        max_ascender: f32,
        max_descender: f32,
        is_first_line: bool,
        is_last_line: bool,
    ) -> LineBox {
        use crate::InlineType;

        let line_height = if max_ascender == 0.0 && max_descender == 0.0 {
            config.font_size * 1.2 * line_spacing
        } else {
            (max_ascender + max_descender) * line_spacing
        };

        let baseline = if max_ascender == 0.0 {
            config.font_size * 0.8
        } else {
            max_ascender
        };

        let available_width = config.available_width - config.left_indent - config.right_indent
            - if is_first_line { config.first_line_indent } else { 0.0 };

        // Strip trailing whitespace items
        let items_to_render: Vec<&LayoutItem> = {
            let mut v: Vec<_> = items.iter().collect();
            while let Some(last) = v.last() {
                if last.is_whitespace() {
                    v.pop();
                } else {
                    break;
                }
            }
            v
        };

        // Calculate total content width
        let total_content_width: f32 = items_to_render.iter().map(|i| i.width()).sum();

        // Reorder for BiDi using item levels
        let bidi_analyzer = BidiAnalyzer::new();
        let bidi_runs: Vec<BidiRun> = items_to_render
            .iter()
            .enumerate()
            .map(|(i, item)| BidiRun::new(i, i + 1, item.bidi_level()))
            .collect();
        let visual_order = bidi_analyzer.visual_order(&bidi_runs);

        let reordered: Vec<&LayoutItem> = visual_order
            .iter()
            .filter_map(|&idx| items_to_render.get(idx).copied())
            .collect();

        // Calculate extra space for alignment
        let extra_space = (available_width - total_content_width).max(0.0);

        let first_line_indent = if is_first_line { config.first_line_indent } else { 0.0 };
        let base_x = config.left_indent + first_line_indent;

        let (x_start, word_spacing_extra) = match config.alignment {
            Alignment::Left => {
                if config.direction == Direction::Rtl {
                    (base_x + extra_space, 0.0)
                } else {
                    (base_x, 0.0)
                }
            }
            Alignment::Center => (base_x + extra_space / 2.0, 0.0),
            Alignment::Right => {
                if config.direction == Direction::Rtl {
                    (base_x, 0.0)
                } else {
                    (base_x + extra_space, 0.0)
                }
            }
            Alignment::Justify => {
                if is_last_line || reordered.len() <= 1 {
                    (base_x, 0.0)
                } else {
                    let gap_count = reordered.len() - 1;
                    let spacing = if gap_count > 0 {
                        extra_space / gap_count as f32
                    } else {
                        0.0
                    };
                    (base_x, spacing)
                }
            }
        };

        let mut inlines = Vec::new();
        let mut x = x_start;

        for (i, item) in reordered.iter().enumerate() {
            match item {
                LayoutItem::Text(seg) => {
                    let y_offset_inline = baseline - seg.ascender;
                    inlines.push(InlineBox {
                        node_id: seg.run_id,
                        bounds: Rect::new(x, y_offset_inline, seg.width, seg.ascender + seg.descender),
                        direction: seg.direction,
                        start_offset: seg.start_offset,
                        end_offset: seg.end_offset,
                        inline_type: InlineType::Text,
                        list_marker: None,
                    });
                    x += seg.width;
                }
                LayoutItem::Image(img) => {
                    // Image sits on baseline
                    let y_offset_inline = baseline - img.height;
                    inlines.push(InlineBox::image(
                        img.node_id,
                        Rect::new(x, y_offset_inline, img.width, img.height),
                    ));
                    x += img.width;
                }
                LayoutItem::ListMarker(marker) => {
                    // List marker is positioned at the start of the line
                    // with hanging indent
                    let y_offset_inline = baseline - marker.ascender;
                    let marker_info = ListMarkerInfo {
                        text: marker.text.clone(),
                        font: marker.font.clone(),
                        is_bullet: marker.is_bullet,
                        level: marker.level,
                    };
                    inlines.push(InlineBox::list_marker(
                        marker.para_id,
                        Rect::new(x, y_offset_inline, marker.width, marker.height),
                        marker_info,
                    ));
                    // Add a tab space after the marker
                    x += marker.width + config.list_hanging.max(8.0);
                }
            }

            // Add extra spacing for justify alignment (but not after list markers)
            if config.alignment == Alignment::Justify && i < reordered.len() - 1 && !item.is_list_marker() {
                x += word_spacing_extra;
            }
        }

        LineBox {
            bounds: Rect::new(config.left_indent, y_offset, available_width, line_height),
            baseline,
            direction: config.direction,
            inlines,
        }
    }

    /// Break text into lines (simpler API for plain text)
    pub fn break_text(
        &self,
        text: &str,
        font_size: f32,
        available_width: f32,
    ) -> Vec<(usize, usize)> {
        // Find break opportunities
        let break_ops = self.find_break_opportunities(text);

        // Shape the text
        let shaped = self.shaper.shape(text, font_size)
            .unwrap_or_else(|_| ShapedRun {
                glyphs: Vec::new(),
                width: text.len() as f32 * font_size * 0.6,
                font_size,
                units_per_em: 1000,
                ascender: font_size * 0.8,
                descender: font_size * 0.2,
                line_gap: 0.0,
            });

        let mut lines = Vec::new();
        let mut line_start = 0;
        let mut line_width = 0.0;
        let mut last_break = 0;

        for (idx, ch) in text.char_indices() {
            let char_width = if !shaped.glyphs.is_empty() {
                shaped.glyphs.iter()
                    .find(|g| g.cluster as usize == idx)
                    .map(|g| g.advance_width(font_size, shaped.units_per_em))
                    .unwrap_or(font_size * 0.6)
            } else {
                font_size * 0.6
            };

            // Check for break opportunity
            if idx < break_ops.len() && break_ops[idx] != BreakOpportunity::NoBreak {
                last_break = idx + ch.len_utf8();
            }

            // Check if mandatory break
            if idx < break_ops.len() && break_ops[idx] == BreakOpportunity::Mandatory {
                lines.push((line_start, idx + ch.len_utf8()));
                line_start = idx + ch.len_utf8();
                line_width = 0.0;
                last_break = line_start;
                continue;
            }

            line_width += char_width;

            // Check if line is too wide
            if line_width > available_width && last_break > line_start {
                lines.push((line_start, last_break));
                line_start = last_break;
                // Recalculate width for remaining text on current line
                line_width = text[last_break..=idx].chars()
                    .map(|_| font_size * 0.6)
                    .sum();
            }
        }

        // Add remaining text as last line
        if line_start < text.len() {
            lines.push((line_start, text.len()));
        }

        lines
    }
}

impl Default for LineBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_break_opportunities() {
        let breaker = LineBreaker::new();
        let text = "Hello world!";
        let ops = breaker.find_break_opportunities(text);

        // Should have break opportunity after space
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_empty_text() {
        let breaker = LineBreaker::new();
        let lines = breaker.break_text("", 12.0, 100.0);
        assert!(lines.is_empty() || lines == vec![(0, 0)]);
    }

    #[test]
    fn test_single_word() {
        let breaker = LineBreaker::new();
        let lines = breaker.break_text("Hello", 12.0, 100.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], (0, 5));
    }

    #[test]
    fn test_line_breaking() {
        let breaker = LineBreaker::new();
        // Very narrow width should cause multiple lines
        let lines = breaker.break_text("Hello world test", 12.0, 50.0);
        assert!(lines.len() >= 1);
    }

    #[test]
    fn test_mandatory_break() {
        let breaker = LineBreaker::new();
        let lines = breaker.break_text("Hello\nworld", 12.0, 500.0);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_bidi_levels_ltr() {
        let bidi_analyzer = BidiAnalyzer::new();
        let levels = bidi_analyzer.get_levels("Hello World", Direction::Ltr);
        // All characters should have level 0 (LTR)
        assert!(levels.iter().all(|&l| l == 0));
    }

    #[test]
    fn test_bidi_levels_rtl_base() {
        let bidi_analyzer = BidiAnalyzer::new();
        // Hebrew text "שלום" with RTL base direction
        let text = "\u{05E9}\u{05DC}\u{05D5}\u{05DD}";
        let levels = bidi_analyzer.get_levels(text, Direction::Rtl);
        // All characters should have odd level (RTL)
        assert!(levels.iter().all(|&l| l % 2 == 1));
    }

    #[test]
    fn test_shaped_segment_direction() {
        // Create a segment with LTR direction
        let segment = ShapedSegment {
            run_id: doc_model::NodeId::new(),
            start_offset: 0,
            end_offset: 5,
            width: 50.0,
            ascender: 10.0,
            descender: 2.0,
            is_whitespace: false,
            break_after: BreakOpportunity::Allowed,
            bidi_level: 0,
            direction: Direction::Ltr,
        };
        assert_eq!(segment.direction, Direction::Ltr);
        assert_eq!(segment.bidi_level, 0);

        // Create a segment with RTL direction
        let rtl_segment = ShapedSegment {
            run_id: doc_model::NodeId::new(),
            start_offset: 0,
            end_offset: 5,
            width: 50.0,
            ascender: 10.0,
            descender: 2.0,
            is_whitespace: false,
            break_after: BreakOpportunity::Allowed,
            bidi_level: 1,
            direction: Direction::Rtl,
        };
        assert_eq!(rtl_segment.direction, Direction::Rtl);
        assert_eq!(rtl_segment.bidi_level, 1);
    }

    #[test]
    fn test_line_break_config_direction() {
        let ltr_config = LineBreakConfig {
            direction: Direction::Ltr,
            ..Default::default()
        };
        assert_eq!(ltr_config.direction, Direction::Ltr);

        let rtl_config = LineBreakConfig {
            direction: Direction::Rtl,
            ..Default::default()
        };
        assert_eq!(rtl_config.direction, Direction::Rtl);
    }
}
