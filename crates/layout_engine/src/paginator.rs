//! Pagination Algorithm (D2)
//!
//! This module implements the pagination algorithm for the MS Word document editor.
//! It takes line-broken paragraphs from the line breaker (D1) and flows them onto pages.
//!
//! Key features:
//! - Multiple page sizes (A4, Letter, custom)
//! - Header and footer areas
//! - Block pagination with line-boundary splitting
//! - Incremental reflow for efficient editing
//! - Layout cache integration

use crate::{
    AreaBox, BlockBox, ColumnBox, LayoutCache, LayoutTree, LineBox, LineBreakConfig,
    LineBreaker, LineNumberItem, LineNumberTracker, PageBox, Rect, Result,
};
use doc_model::{Alignment, DocumentTree, LineNumbering, LineNumberRestart, Node, NodeId, WidowOrphanControl, ParagraphKeepRules};
use std::collections::HashSet;

/// Standard page sizes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageSize {
    /// US Letter (8.5" x 11")
    Letter,
    /// A4 (210mm x 297mm)
    A4,
    /// Legal (8.5" x 14")
    Legal,
    /// Custom size in points
    Custom { width: f32, height: f32 },
}

impl PageSize {
    /// Get the width and height in points
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            PageSize::Letter => (612.0, 792.0),   // 8.5" x 11" at 72 dpi
            PageSize::A4 => (595.276, 841.89),    // 210mm x 297mm at 72 dpi
            PageSize::Legal => (612.0, 1008.0),   // 8.5" x 14" at 72 dpi
            PageSize::Custom { width, height } => (*width, *height),
        }
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize::Letter
    }
}

/// Header/footer configuration
#[derive(Debug, Clone)]
pub struct HeaderFooterConfig {
    /// Height reserved for header
    pub header_height: f32,
    /// Height reserved for footer
    pub footer_height: f32,
    /// Distance from page edge to header
    pub header_margin: f32,
    /// Distance from page edge to footer
    pub footer_margin: f32,
    /// Whether to include header on first page
    pub header_on_first_page: bool,
    /// Whether to include footer on first page
    pub footer_on_first_page: bool,
}

impl Default for HeaderFooterConfig {
    fn default() -> Self {
        Self {
            header_height: 36.0,  // 0.5 inch
            footer_height: 36.0,  // 0.5 inch
            header_margin: 36.0,  // 0.5 inch from edge
            footer_margin: 36.0,  // 0.5 inch from edge
            header_on_first_page: true,
            footer_on_first_page: true,
        }
    }
}

/// Column configuration for multi-column layout
#[derive(Debug, Clone)]
pub struct ColumnLayout {
    /// Number of columns
    pub column_count: usize,
    /// Space between columns in points
    pub column_spacing: f32,
    /// Whether columns have equal width
    pub equal_width: bool,
    /// Custom column widths (when equal_width is false)
    pub column_widths: Vec<f32>,
    /// Whether to draw separator lines between columns
    pub draw_separators: bool,
}

impl Default for ColumnLayout {
    fn default() -> Self {
        Self {
            column_count: 1,
            column_spacing: 36.0, // 0.5 inch
            equal_width: true,
            column_widths: Vec::new(),
            draw_separators: false,
        }
    }
}

impl ColumnLayout {
    /// Create a single-column layout
    pub fn single() -> Self {
        Self::default()
    }

    /// Create a multi-column layout with equal widths
    pub fn equal(count: usize, spacing: f32) -> Self {
        Self {
            column_count: count.max(1),
            column_spacing: spacing,
            equal_width: true,
            column_widths: Vec::new(),
            draw_separators: false,
        }
    }

    /// Create a multi-column layout with custom widths
    pub fn custom(widths: Vec<f32>, spacing: f32) -> Self {
        Self {
            column_count: widths.len().max(1),
            column_spacing: spacing,
            equal_width: false,
            column_widths: widths,
            draw_separators: false,
        }
    }

    /// Calculate column bounds for a given content width
    pub fn calculate_bounds(&self, content_width: f32) -> Vec<(f32, f32)> {
        if self.column_count <= 1 {
            return vec![(0.0, content_width)];
        }

        if self.equal_width {
            let total_spacing = self.column_spacing * (self.column_count - 1) as f32;
            let col_width = (content_width - total_spacing) / self.column_count as f32;

            (0..self.column_count)
                .map(|i| (i as f32 * (col_width + self.column_spacing), col_width))
                .collect()
        } else {
            let mut bounds = Vec::new();
            let mut x = 0.0;
            for (i, &width) in self.column_widths.iter().enumerate() {
                bounds.push((x, width));
                if i < self.column_widths.len() - 1 {
                    x += width + self.column_spacing;
                }
            }
            bounds
        }
    }
}

/// Page layout configuration
#[derive(Debug, Clone)]
pub struct PageConfig {
    /// Page size
    pub page_size: PageSize,
    /// Page width in points (derived from page_size or custom)
    pub page_width: f32,
    /// Page height in points (derived from page_size or custom)
    pub page_height: f32,
    /// Top margin in points
    pub margin_top: f32,
    /// Bottom margin in points
    pub margin_bottom: f32,
    /// Left margin in points
    pub margin_left: f32,
    /// Right margin in points
    pub margin_right: f32,
    /// Header and footer configuration
    pub header_footer: HeaderFooterConfig,
    /// Widow/orphan control settings
    pub widow_orphan_control: WidowOrphanControl,
    /// Column layout configuration
    pub columns: ColumnLayout,
    /// Line numbering configuration
    pub line_numbering: LineNumbering,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self::letter()
    }
}

impl PageConfig {
    /// Create a Letter-sized page configuration
    pub fn letter() -> Self {
        let (width, height) = PageSize::Letter.dimensions();
        Self {
            page_size: PageSize::Letter,
            page_width: width,
            page_height: height,
            margin_top: 72.0,    // 1 inch
            margin_bottom: 72.0, // 1 inch
            margin_left: 72.0,   // 1 inch
            margin_right: 72.0,  // 1 inch
            header_footer: HeaderFooterConfig::default(),
            widow_orphan_control: WidowOrphanControl::default(),
            columns: ColumnLayout::default(),
            line_numbering: LineNumbering::default(),
        }
    }

    /// Create an A4-sized page configuration
    pub fn a4() -> Self {
        let (width, height) = PageSize::A4.dimensions();
        Self {
            page_size: PageSize::A4,
            page_width: width,
            page_height: height,
            margin_top: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            header_footer: HeaderFooterConfig::default(),
            widow_orphan_control: WidowOrphanControl::default(),
            columns: ColumnLayout::default(),
            line_numbering: LineNumbering::default(),
        }
    }

    /// Create a custom page configuration
    pub fn custom(width: f32, height: f32) -> Self {
        Self {
            page_size: PageSize::Custom { width, height },
            page_width: width,
            page_height: height,
            margin_top: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            header_footer: HeaderFooterConfig::default(),
            widow_orphan_control: WidowOrphanControl::default(),
            columns: ColumnLayout::default(),
            line_numbering: LineNumbering::default(),
        }
    }

    /// Set widow/orphan control settings
    pub fn with_widow_orphan_control(mut self, control: WidowOrphanControl) -> Self {
        self.widow_orphan_control = control;
        self
    }

    /// Get the effective orphan control value (min lines at bottom of page)
    pub fn orphan_control(&self) -> usize {
        self.widow_orphan_control.effective_min_bottom()
    }

    /// Get the effective widow control value (min lines at top of page)
    pub fn widow_control(&self) -> usize {
        self.widow_orphan_control.effective_min_top()
    }

    /// Create a configuration with multi-column layout
    pub fn with_columns(mut self, column_count: usize, spacing: f32) -> Self {
        self.columns = ColumnLayout::equal(column_count, spacing);
        self
    }

    /// Set column configuration
    pub fn set_columns(&mut self, columns: ColumnLayout) {
        self.columns = columns;
    }

    /// Get the content area width (excluding margins)
    pub fn content_width(&self) -> f32 {
        self.page_width - self.margin_left - self.margin_right
    }

    /// Get the content area height (excluding margins and header/footer)
    pub fn content_height(&self) -> f32 {
        self.page_height - self.margin_top - self.margin_bottom
    }

    /// Get the content area height for a specific page
    pub fn content_height_for_page(&self, page_index: usize) -> f32 {
        let mut height = self.content_height();

        // Subtract header space if applicable
        if page_index == 0 && self.header_footer.header_on_first_page
            || page_index > 0
        {
            height -= self.header_footer.header_height;
        }

        // Subtract footer space if applicable
        if page_index == 0 && self.header_footer.footer_on_first_page
            || page_index > 0
        {
            height -= self.header_footer.footer_height;
        }

        height
    }

    /// Get the content area top offset for a specific page
    pub fn content_top_for_page(&self, page_index: usize) -> f32 {
        let mut top = self.margin_top;

        // Add header space if applicable
        if page_index == 0 && self.header_footer.header_on_first_page
            || page_index > 0
        {
            top += self.header_footer.header_height;
        }

        top
    }

    /// Calculate column bounds for the content area
    pub fn column_bounds(&self) -> Vec<(f32, f32)> {
        self.columns.calculate_bounds(self.content_width())
    }

    /// Get the width of a specific column
    pub fn column_width(&self, column_index: usize) -> f32 {
        let bounds = self.column_bounds();
        bounds.get(column_index).map(|(_, w)| *w).unwrap_or(self.content_width())
    }

    /// Check if this is a multi-column layout
    pub fn is_multi_column(&self) -> bool {
        self.columns.column_count > 1
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.column_count
    }

    /// Set line numbering configuration
    pub fn with_line_numbering(mut self, line_numbering: LineNumbering) -> Self {
        self.line_numbering = line_numbering;
        self
    }

    /// Enable line numbering with default settings
    pub fn enable_line_numbering(&mut self) {
        self.line_numbering = LineNumbering::enabled();
    }

    /// Disable line numbering
    pub fn disable_line_numbering(&mut self) {
        self.line_numbering.enabled = false;
    }

    /// Check if line numbering is enabled
    pub fn has_line_numbering(&self) -> bool {
        self.line_numbering.enabled
    }
}

/// A block pending pagination, which may be a full or partial paragraph
#[derive(Debug, Clone)]
struct PendingBlock {
    /// The original paragraph's node ID
    node_id: NodeId,
    /// Lines in this block
    lines: Vec<LineBox>,
    /// Total height of the block
    height: f32,
    /// Whether this is a continuation from a previous page
    is_continuation: bool,
    /// Space before the paragraph (only for first fragment)
    space_before: f32,
    /// Space after the paragraph (only for last fragment)
    space_after: f32,
    /// Keep with next paragraph (don't allow page break after)
    keep_with_next: bool,
    /// Keep lines together (don't split paragraph)
    keep_together: bool,
    /// Page break before this paragraph
    page_break_before: bool,
}

impl PendingBlock {
    fn new(node_id: NodeId, lines: Vec<LineBox>, space_before: f32, space_after: f32) -> Self {
        let height = lines.iter().map(|l| l.bounds.height).sum::<f32>() + space_before + space_after;
        Self {
            node_id,
            lines,
            height,
            is_continuation: false,
            space_before,
            space_after,
            keep_with_next: false,
            keep_together: false,
            page_break_before: false,
        }
    }

    fn with_pagination_options(
        node_id: NodeId,
        lines: Vec<LineBox>,
        space_before: f32,
        space_after: f32,
        keep_with_next: bool,
        keep_together: bool,
        page_break_before: bool,
    ) -> Self {
        let height = lines.iter().map(|l| l.bounds.height).sum::<f32>() + space_before + space_after;
        Self {
            node_id,
            lines,
            height,
            is_continuation: false,
            space_before,
            space_after,
            keep_with_next,
            keep_together,
            page_break_before,
        }
    }

    /// Create a pending block with ParagraphKeepRules
    fn with_keep_rules(
        node_id: NodeId,
        lines: Vec<LineBox>,
        space_before: f32,
        space_after: f32,
        keep_rules: ParagraphKeepRules,
    ) -> Self {
        Self::with_pagination_options(
            node_id,
            lines,
            space_before,
            space_after,
            keep_rules.keep_with_next,
            keep_rules.keep_together,
            keep_rules.page_break_before,
        )
    }

    /// Get the keep rules for this block
    fn keep_rules(&self) -> ParagraphKeepRules {
        ParagraphKeepRules {
            keep_with_next: self.keep_with_next,
            keep_together: self.keep_together,
            page_break_before: self.page_break_before,
        }
    }

    /// Check if this block can be split at line boundaries
    fn can_split(&self, min_lines_before: usize, min_lines_after: usize) -> bool {
        // Don't split if keep_together is set
        if self.keep_together {
            return false;
        }
        self.lines.len() > min_lines_before + min_lines_after
    }

    /// Split this block at the given line index
    fn split_at(&self, line_index: usize) -> (PendingBlock, PendingBlock) {
        let first_lines: Vec<LineBox> = self.lines[..line_index].to_vec();
        let second_lines: Vec<LineBox> = self.lines[line_index..].to_vec();

        let first_height = first_lines.iter().map(|l| l.bounds.height).sum::<f32>()
            + self.space_before;
        let second_height = second_lines.iter().map(|l| l.bounds.height).sum::<f32>()
            + self.space_after;

        let first = PendingBlock {
            node_id: self.node_id,
            lines: first_lines,
            height: first_height,
            is_continuation: self.is_continuation,
            space_before: self.space_before,
            space_after: 0.0,
            keep_with_next: false, // First part doesn't keep with next
            keep_together: false, // Already split, so this doesn't apply
            page_break_before: self.page_break_before,
        };

        let second = PendingBlock {
            node_id: self.node_id,
            lines: second_lines,
            height: second_height,
            is_continuation: true,
            space_before: 0.0,
            space_after: self.space_after,
            keep_with_next: self.keep_with_next,
            keep_together: false, // Second part doesn't need keep_together
            page_break_before: false, // Second part doesn't need page break
        };

        (first, second)
    }

    /// Find the best split point that fits in the available height
    fn find_split_point(
        &self,
        available_height: f32,
        min_lines_before: usize,
        min_lines_after: usize,
    ) -> Option<usize> {
        if !self.can_split(min_lines_before, min_lines_after) {
            return None;
        }

        let mut cumulative_height = self.space_before;
        let max_line = self.lines.len().saturating_sub(min_lines_after);

        for (i, line) in self.lines.iter().enumerate() {
            if i < min_lines_before {
                cumulative_height += line.bounds.height;
                continue;
            }

            if i >= max_line {
                break;
            }

            cumulative_height += line.bounds.height;

            if cumulative_height > available_height {
                // Return the previous line as the split point
                if i > min_lines_before {
                    return Some(i);
                }
                return None;
            }
        }

        // All lines fit
        None
    }

    /// Calculate the split point with full widow/orphan control algorithm.
    ///
    /// Returns the number of lines to keep on the current page, or None if the
    /// entire paragraph should be moved to the next page.
    ///
    /// Algorithm:
    /// 1. Calculate how many lines fit in available height
    /// 2. Check orphan control: if lines_on_current < min_lines_bottom, move all to next page
    /// 3. Check widow control: if lines_on_next < min_lines_top, pull more lines to next page
    /// 4. If pulling more lines would cause orphan, move entire paragraph
    fn calculate_widow_orphan_split(
        &self,
        available_height: f32,
        min_lines_bottom: usize,  // orphan control - min lines at bottom of current page
        min_lines_top: usize,     // widow control - min lines at top of next page
    ) -> Option<usize> {
        let total_lines = self.lines.len();

        // Can't split if keep_together is set
        if self.keep_together {
            return None;
        }

        // Calculate how many lines fit in available height
        let mut cumulative_height = self.space_before;
        let mut lines_that_fit = 0;

        for line in &self.lines {
            let next_height = cumulative_height + line.bounds.height;
            if next_height > available_height {
                break;
            }
            cumulative_height = next_height;
            lines_that_fit += 1;
        }

        // If all lines fit, no split needed
        if lines_that_fit >= total_lines {
            return Some(total_lines);
        }

        let mut lines_on_current_page = lines_that_fit;
        let lines_on_next_page = total_lines - lines_on_current_page;

        // Check orphan control (bottom of current page)
        // If fewer than min_lines_bottom would remain on current page, move all to next
        if lines_on_current_page > 0 && lines_on_current_page < min_lines_bottom {
            // Move entire paragraph to next page
            return None;
        }

        // Check widow control (top of next page)
        // If fewer than min_lines_top would start on next page, pull more lines to next page
        if lines_on_next_page > 0 && lines_on_next_page < min_lines_top {
            // Pull more lines to next page to satisfy widow control
            lines_on_current_page = total_lines.saturating_sub(min_lines_top);

            // If pulling lines would cause orphan (too few on current page), move entire paragraph
            if lines_on_current_page > 0 && lines_on_current_page < min_lines_bottom {
                return None;
            }
        }

        // If no lines would remain on current page, signal to move entire paragraph
        if lines_on_current_page == 0 {
            return None;
        }

        // Verify the calculated split point actually fits in available height
        let height_for_split: f32 = self.space_before
            + self.lines[..lines_on_current_page]
                .iter()
                .map(|l| l.bounds.height)
                .sum::<f32>();

        if height_for_split > available_height {
            // The adjusted split point doesn't fit, recalculate
            // Find the maximum lines that fit while respecting widow/orphan control
            let mut best_split = None;
            cumulative_height = self.space_before;

            for (i, line) in self.lines.iter().enumerate() {
                cumulative_height += line.bounds.height;
                if cumulative_height > available_height {
                    break;
                }

                let current_lines = i + 1;
                let next_lines = total_lines - current_lines;

                // Check both orphan and widow constraints
                let orphan_ok = current_lines >= min_lines_bottom;
                let widow_ok = next_lines == 0 || next_lines >= min_lines_top;

                if orphan_ok && widow_ok {
                    best_split = Some(current_lines);
                }
            }

            return best_split;
        }

        Some(lines_on_current_page)
    }
}

/// Result of attempting to place a block on a page
#[derive(Debug)]
enum PlacementResult {
    /// Block fits entirely
    Fits,
    /// Block was split, second part returned
    Split(PendingBlock),
    /// Block doesn't fit at all, should go to next page
    DoesNotFit,
}

/// Tracking information for incremental reflow
#[derive(Debug, Default)]
pub struct ReflowState {
    /// Paragraphs marked as dirty (need re-layout)
    dirty_paragraphs: HashSet<NodeId>,
    /// Page index where reflow started
    reflow_start_page: Option<usize>,
    /// Whether page breaks have stabilized
    page_breaks_stable: bool,
    /// Previous page break positions for comparison
    previous_page_breaks: Vec<(NodeId, usize)>, // (paragraph_id, line_index)
}

impl ReflowState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a paragraph as dirty
    pub fn mark_dirty(&mut self, para_id: NodeId) {
        self.dirty_paragraphs.insert(para_id);
        self.page_breaks_stable = false;
    }

    /// Clear all dirty marks
    pub fn clear_dirty(&mut self) {
        self.dirty_paragraphs.clear();
    }

    /// Check if a paragraph is dirty
    pub fn is_dirty(&self, para_id: NodeId) -> bool {
        self.dirty_paragraphs.contains(&para_id)
    }

    /// Check if reflow can stop early
    pub fn can_stop_reflow(&self, current_page_breaks: &[(NodeId, usize)]) -> bool {
        if self.dirty_paragraphs.is_empty() {
            return true;
        }

        // Check if page breaks match previous layout
        if current_page_breaks == self.previous_page_breaks {
            return true;
        }

        false
    }
}

/// Paginator that converts a document into pages
pub struct Paginator {
    /// Line breaker for paragraph layout
    line_breaker: LineBreaker,
    /// Page configuration
    config: PageConfig,
    /// Layout cache for efficient incremental layout
    cache: LayoutCache,
    /// Reflow state for incremental updates
    reflow_state: ReflowState,
}

impl Paginator {
    /// Create a new paginator with the given configuration
    pub fn new(config: PageConfig) -> Self {
        Self {
            line_breaker: LineBreaker::new(),
            config,
            cache: LayoutCache::new(),
            reflow_state: ReflowState::new(),
        }
    }

    /// Create a paginator with default Letter page configuration
    pub fn letter() -> Self {
        Self::new(PageConfig::letter())
    }

    /// Create a paginator with A4 page configuration
    pub fn a4() -> Self {
        Self::new(PageConfig::a4())
    }

    /// Get a reference to the page configuration
    pub fn config(&self) -> &PageConfig {
        &self.config
    }

    /// Get a mutable reference to the page configuration
    pub fn config_mut(&mut self) -> &mut PageConfig {
        &mut self.config
    }

    /// Get a mutable reference to the line breaker
    pub fn line_breaker_mut(&mut self) -> &mut LineBreaker {
        &mut self.line_breaker
    }

    /// Get a reference to the layout cache
    pub fn cache(&self) -> &LayoutCache {
        &self.cache
    }

    /// Invalidate cache for a specific paragraph (call after editing)
    pub fn invalidate_paragraph(&mut self, para_id: NodeId) {
        self.cache.invalidate_paragraph(para_id);
        self.reflow_state.mark_dirty(para_id);
    }

    /// Invalidate the entire cache (call after major changes)
    pub fn invalidate_all(&mut self) {
        self.cache.invalidate_all();
        self.reflow_state = ReflowState::new();
    }

    /// Layout the entire document into pages
    pub fn layout(&mut self, tree: &DocumentTree) -> Result<LayoutTree> {
        let mut layout = LayoutTree::new();

        // Create line break configuration from page config
        let line_config = self.create_line_config(tree);

        // Break all paragraphs into lines and create pending blocks
        let mut pending_blocks: Vec<PendingBlock> = Vec::new();

        for para in tree.paragraphs() {
            let para_id = para.id();

            // Get paragraph spacing
            let space_before = para.style.space_before.unwrap_or(0.0);
            let space_after = para.style.space_after.unwrap_or(0.0);

            // Get list properties if paragraph is in a list
            let (list_marker_text, list_is_bullet, list_marker_font, list_level, list_num_id, list_hanging) =
                if let Some(list_props) = &para.direct_formatting.list_props {
                    if let Some(num_id) = list_props.num_id {
                        let level = list_props.effective_level();
                        let is_bullet = tree.numbering.is_bullet_list(num_id);

                        // Get the level definition
                        if let Some(level_def) = tree.numbering.get_effective_level(num_id, level) {
                            // Build counts array for multi-level formatting
                            let counts: Vec<u32> = (0..=level)
                                .map(|l| tree.numbering.get_counter(num_id, l) + 1)
                                .collect();

                            let marker_text = level_def.format_number(&counts);
                            let marker_font = level_def.font.clone();
                            let hanging = level_def.hanging;

                            (Some(marker_text), is_bullet, marker_font, Some(level), Some(num_id), hanging)
                        } else {
                            (None, false, None, None, None, 0.0)
                        }
                    } else {
                        (None, false, None, None, None, 0.0)
                    }
                } else {
                    (None, false, None, None, None, 0.0)
                };

            // Apply list indent to left indent
            let list_indent = if let (Some(num_id), Some(level)) = (list_num_id, list_level) {
                if let Some(level_def) = tree.numbering.get_effective_level(num_id, level) {
                    level_def.indent
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Create custom line config for this paragraph
            let para_line_config = LineBreakConfig {
                available_width: line_config.available_width,
                font_size: line_config.font_size,
                line_spacing: match para.style.line_spacing {
                    Some(doc_model::LineSpacing::Multiple(m)) => m,
                    _ => line_config.line_spacing,
                },
                first_line_indent: para.style.indent_first_line.unwrap_or(0.0),
                left_indent: para.style.indent_left.unwrap_or(0.0) + list_indent,
                right_indent: para.style.indent_right.unwrap_or(0.0),
                direction: line_config.direction,
                allow_hyphenation: line_config.allow_hyphenation,
                alignment: para.style.alignment.unwrap_or(Alignment::Left),
                list_num_id,
                list_level,
                list_marker_text,
                list_is_bullet,
                list_marker_font,
                list_hanging,
            };

            // Break paragraph into lines
            let broken = self.line_breaker.break_paragraph(tree, para_id, &para_line_config)?;

            // Cache the result
            self.cache.store(
                para_id,
                tree.document.version(),
                line_config.available_width,
                &broken.lines,
                broken.total_height,
            );

            // Get paragraph pagination options
            let keep_with_next = para.style.keep_with_next.unwrap_or(false)
                || para.direct_formatting.keep_with_next.unwrap_or(false);
            let keep_together = para.style.keep_together.unwrap_or(false)
                || para.direct_formatting.keep_together.unwrap_or(false);
            let page_break_before = para.style.page_break_before.unwrap_or(false)
                || para.direct_formatting.page_break_before.unwrap_or(false);

            // Create pending block with pagination options
            pending_blocks.push(PendingBlock::with_pagination_options(
                para_id,
                broken.lines,
                space_before,
                space_after,
                keep_with_next,
                keep_together,
                page_break_before,
            ));
        }

        // Paginate all blocks
        self.paginate_blocks(&mut layout, pending_blocks)?;

        // Ensure at least one page
        if layout.pages.is_empty() {
            layout.add_page(self.create_empty_page(0));
        }

        // Generate line numbers if enabled
        self.generate_line_numbers(&mut layout);

        Ok(layout)
    }

    /// Perform incremental layout after an edit
    pub fn layout_incremental(
        &mut self,
        tree: &DocumentTree,
        edited_para_id: NodeId,
    ) -> Result<LayoutTree> {
        // Mark the edited paragraph as dirty
        self.invalidate_paragraph(edited_para_id);

        // For now, perform full layout
        // A more sophisticated implementation would:
        // 1. Find the page containing the edited paragraph
        // 2. Re-layout only from that paragraph forward
        // 3. Stop when page breaks stabilize
        self.layout(tree)
    }

    /// Create line break configuration from page config and document
    fn create_line_config(&self, _tree: &DocumentTree) -> LineBreakConfig {
        LineBreakConfig {
            available_width: self.config.content_width(),
            font_size: 12.0, // Default font size
            line_spacing: 1.0,
            first_line_indent: 0.0,
            left_indent: 0.0,
            right_indent: 0.0,
            direction: crate::Direction::Ltr,
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

    /// Paginate a list of pending blocks onto pages
    ///
    /// This method implements the full pagination algorithm with support for:
    ///
    /// ## Keep Rules
    ///
    /// 1. **page_break_before**: When a paragraph has `page_break_before = true`,
    ///    start it on a new page regardless of available space.
    ///
    /// 2. **keep_with_next**: When a paragraph has `keep_with_next = true`,
    ///    ensure it stays on the same page as the following paragraph.
    ///    If they don't fit together, move both to the next page.
    ///
    /// 3. **keep_together**: When a paragraph has `keep_together = true`,
    ///    don't allow page breaks within the paragraph. If the entire
    ///    paragraph doesn't fit on the current page, move it to the next page.
    ///
    /// ## Algorithm
    ///
    /// ```text
    /// for each block:
    ///     if page_break_before:
    ///         start new page
    ///
    ///     calculate required space (including keep_with_next constraints)
    ///
    ///     if doesn't fit:
    ///         if keep_together:
    ///             move entire block to next page
    ///         else:
    ///             break within block (check widow/orphan)
    ///
    ///         if keep_with_next:
    ///             move this block + next block to next page together
    /// ```
    fn paginate_blocks(
        &mut self,
        layout: &mut LayoutTree,
        mut blocks: Vec<PendingBlock>,
    ) -> Result<()> {
        let mut current_page_blocks: Vec<BlockBox> = Vec::new();
        let mut current_y = 0.0;
        let mut page_index = 0;

        let mut block_index = 0;
        while block_index < blocks.len() {
            let block = &blocks[block_index];
            let page_content_height = self.config.content_height_for_page(page_index);
            let remaining_height = page_content_height - current_y;

            // Handle page_break_before
            if block.page_break_before && !current_page_blocks.is_empty() {
                // Finalize current page and start a new one
                layout.add_page(self.create_page(page_index, current_page_blocks));
                current_page_blocks = Vec::new();
                page_index += 1;
                current_y = 0.0;
                continue; // Re-evaluate with fresh page
            }

            // =========================================================================
            // RULE 2: Calculate required space including keep_with_next constraints
            // =========================================================================
            // When a paragraph has keep_with_next = true, ensure it stays on the same
            // page as the following paragraph. Calculate the minimum height needed
            // for the next block(s).
            let keep_with_next_height = self.calculate_keep_with_next_chain_height(&blocks, block_index);
            let effective_remaining = remaining_height - keep_with_next_height;

            // =========================================================================
            // RULE 3: Placement decision based on keep rules
            // =========================================================================
            let block_fits = block.height <= effective_remaining.max(0.0);

            if block_fits {
                // Block fits entirely on this page (with room for keep_with_next if needed)
                let block_box = self.create_block_box(block, current_y);
                current_y += block.height;
                current_page_blocks.push(block_box);
                block_index += 1;
            } else if block.keep_together {
                // =====================================================================
                // keep_together: Don't split the block - move entire block to next page
                // =====================================================================
                if !current_page_blocks.is_empty() {
                    // Finalize current page
                    layout.add_page(self.create_page(page_index, current_page_blocks));
                    current_page_blocks = Vec::new();
                    page_index += 1;
                    current_y = 0.0;
                    // Re-evaluate on new page
                    continue;
                } else {
                    // Block is at top of page but still doesn't fit
                    // This means the block is taller than a full page
                    // Force-place it to avoid infinite loop (content will overflow)
                    let block_box = self.create_block_box(block, current_y);
                    current_page_blocks.push(block_box);
                    layout.add_page(self.create_page(page_index, current_page_blocks));
                    current_page_blocks = Vec::new();
                    page_index += 1;
                    current_y = 0.0;
                    block_index += 1;
                }
            } else if block.keep_with_next && block_index + 1 < blocks.len() && effective_remaining < block.height {
                // =====================================================================
                // keep_with_next: Block doesn't fit with next block on same page
                // =====================================================================
                // Move the entire block (and by extension, the next block) to next page.
                if !current_page_blocks.is_empty() {
                    layout.add_page(self.create_page(page_index, current_page_blocks));
                    current_page_blocks = Vec::new();
                    page_index += 1;
                    current_y = 0.0;
                    // Re-evaluate on new page
                    continue;
                } else {
                    // At top of page and still doesn't fit with next block
                    // We must place the current block; keep_with_next constraint cannot be satisfied
                    // Try to split if possible, or force-place the whole block
                    let split_result = block.calculate_widow_orphan_split(
                        remaining_height,
                        self.config.orphan_control(),
                        self.config.widow_control(),
                    );

                    if let Some(lines_on_current) = split_result {
                        if lines_on_current > 0 && lines_on_current < block.lines.len() {
                            // Split the block
                            let (first_part, second_part) = block.split_at(lines_on_current);
                            let block_box = self.create_block_box(&first_part, current_y);
                            current_page_blocks.push(block_box);
                            layout.add_page(self.create_page(page_index, current_page_blocks));
                            current_page_blocks = Vec::new();
                            page_index += 1;
                            current_y = 0.0;
                            // Replace current block with continuation
                            blocks[block_index] = second_part;
                            continue;
                        }
                    }

                    // Can't split effectively - force-place the entire block
                    let block_box = self.create_block_box(block, current_y);
                    current_page_blocks.push(block_box);
                    layout.add_page(self.create_page(page_index, current_page_blocks));
                    current_page_blocks = Vec::new();
                    page_index += 1;
                    current_y = 0.0;
                    block_index += 1;
                }
            } else {
                // =====================================================================
                // Normal case: Block doesn't fit - try to split it
                // =====================================================================
                let split_result = block.calculate_widow_orphan_split(
                    remaining_height,
                    self.config.orphan_control(),
                    self.config.widow_control(),
                );

                match split_result {
                    Some(lines_on_current) if lines_on_current > 0 && lines_on_current < block.lines.len() => {
                        // Valid split point found
                        let (first_part, second_part) = block.split_at(lines_on_current);

                        // Place first part on current page
                        let block_box = self.create_block_box(&first_part, current_y);
                        current_y += first_part.height;
                        current_page_blocks.push(block_box);

                        // Finalize current page
                        layout.add_page(self.create_page(page_index, current_page_blocks));
                        current_page_blocks = Vec::new();
                        page_index += 1;
                        current_y = 0.0;

                        // Replace current block with continuation
                        blocks[block_index] = second_part;
                        // Don't increment - process continuation on next iteration
                    }
                    Some(lines_on_current) if lines_on_current >= block.lines.len() => {
                        // All lines fit (shouldn't normally happen in this branch)
                        let block_box = self.create_block_box(block, current_y);
                        current_y += block.height;
                        current_page_blocks.push(block_box);
                        block_index += 1;
                    }
                    _ => {
                        // Can't split due to widow/orphan control - move to next page
                        if !current_page_blocks.is_empty() {
                            layout.add_page(self.create_page(page_index, current_page_blocks));
                            current_page_blocks = Vec::new();
                            page_index += 1;
                            current_y = 0.0;
                            continue;
                        } else {
                            // Force-place the block (it's taller than a page or constraints prevent split)
                            let block_box = self.create_block_box(block, current_y);
                            current_page_blocks.push(block_box);
                            layout.add_page(self.create_page(page_index, current_page_blocks));
                            current_page_blocks = Vec::new();
                            page_index += 1;
                            current_y = 0.0;
                            block_index += 1;
                        }
                    }
                }
            }
        }

        // Add final page if there are remaining blocks
        if !current_page_blocks.is_empty() {
            layout.add_page(self.create_page(page_index, current_page_blocks));
        }

        Ok(())
    }

    /// Calculate the minimum height required for keep_with_next chains
    ///
    /// When a block has `keep_with_next = true`, we need to reserve space for
    /// at least part of the next block. If the next block has `keep_together = true`,
    /// we need the entire block. Otherwise, we need the first line (or widow_control
    /// number of lines).
    ///
    /// This method follows chains of `keep_with_next` blocks recursively.
    fn calculate_keep_with_next_chain_height(&self, blocks: &[PendingBlock], start_index: usize) -> f32 {
        let block = &blocks[start_index];

        if !block.keep_with_next {
            return 0.0;
        }

        // Check if there's a next block
        if start_index + 1 >= blocks.len() {
            return 0.0;
        }

        let next_block = &blocks[start_index + 1];

        // Calculate minimum height needed for next block
        let next_block_min_height = if next_block.keep_together {
            // Next block has keep_together - need entire block height
            next_block.height
        } else if !next_block.lines.is_empty() {
            // Need at least widow_control lines (minimum 1) from next block
            let min_lines = self.config.widow_control().max(1);
            let lines_to_reserve = min_lines.min(next_block.lines.len());

            let lines_height: f32 = next_block.lines[..lines_to_reserve]
                .iter()
                .map(|l| l.bounds.height)
                .sum();

            next_block.space_before + lines_height
        } else {
            // Empty next block - just the space_before
            next_block.space_before
        };

        // If the next block also has keep_with_next, recursively calculate chain height
        // (limit recursion depth to avoid stack overflow on pathological input)
        if next_block.keep_with_next && start_index + 2 < blocks.len() {
            // For chains, we need full height of next block plus its chain
            next_block.height + self.calculate_keep_with_next_chain_height(blocks, start_index + 1)
        } else {
            next_block_min_height
        }
    }

    /// Try to place a block in the available space
    fn try_place_block(&self, block: &PendingBlock, available_height: f32) -> PlacementResult {
        self.try_place_block_with_height(block, available_height)
    }

    /// Try to place a block in the available space (with explicit height)
    fn try_place_block_with_height(&self, block: &PendingBlock, available_height: f32) -> PlacementResult {
        if block.height <= available_height {
            PlacementResult::Fits
        } else {
            // Use widow/orphan control algorithm to determine if block can be split
            let split_result = block.calculate_widow_orphan_split(
                available_height,
                self.config.orphan_control(),
                self.config.widow_control(),
            );

            match split_result {
                Some(lines_on_current) if lines_on_current > 0 && lines_on_current < block.lines.len() => {
                    // Valid split point found
                    PlacementResult::Split(PendingBlock {
                        node_id: block.node_id,
                        lines: Vec::new(),
                        height: 0.0,
                        is_continuation: true,
                        space_before: 0.0,
                        space_after: block.space_after,
                        keep_with_next: block.keep_with_next,
                        keep_together: false,
                        page_break_before: false,
                    })
                }
                _ => {
                    // Cannot split due to widow/orphan control or keep_together
                    PlacementResult::DoesNotFit
                }
            }
        }
    }

    /// Create a BlockBox from a PendingBlock
    fn create_block_box(&self, block: &PendingBlock, y_offset: f32) -> BlockBox {
        let mut lines = block.lines.clone();
        let mut block_y = y_offset + block.space_before;

        // Update line positions
        for line in &mut lines {
            line.bounds.y = block_y;
            block_y += line.bounds.height;
        }

        let total_height = block.height;

        BlockBox {
            node_id: block.node_id,
            bounds: Rect::new(0.0, y_offset, self.config.content_width(), total_height),
            lines,
        }
    }

    /// Create a PageBox with proper content areas
    fn create_page(&self, index: usize, blocks: Vec<BlockBox>) -> PageBox {
        let content_top = self.config.content_top_for_page(index);
        let content_height = self.config.content_height_for_page(index);

        let content_area = Rect::new(
            self.config.margin_left,
            content_top,
            self.config.content_width(),
            content_height,
        );

        // Create the main content area with columns
        let column_bounds = self.config.column_bounds();
        let mut content_area_box = AreaBox::content(content_area);

        if column_bounds.len() <= 1 {
            // Single column - put all blocks in one column
            let mut column = ColumnBox::new(content_area, 0);
            column.blocks = blocks;
            content_area_box.columns.push(column);
        } else {
            // Multi-column - create column boxes (blocks will be distributed by caller)
            for (i, (x_offset, width)) in column_bounds.iter().enumerate() {
                let col_bounds = Rect::new(
                    self.config.margin_left + x_offset,
                    content_top,
                    *width,
                    content_height,
                );
                content_area_box.columns.push(ColumnBox::new(col_bounds, i));
            }
            // For compatibility with existing code that passes blocks directly
            if !blocks.is_empty() && !content_area_box.columns.is_empty() {
                content_area_box.columns[0].blocks = blocks;
            }
        }

        let mut areas = vec![content_area_box];

        // Add header area if applicable
        let has_header = (index == 0 && self.config.header_footer.header_on_first_page)
            || index > 0;
        if has_header {
            let header_area = Rect::new(
                self.config.margin_left,
                self.config.header_footer.header_margin,
                self.config.content_width(),
                self.config.header_footer.header_height,
            );
            areas.insert(0, AreaBox::header(header_area));
        }

        // Add footer area if applicable
        let has_footer = (index == 0 && self.config.header_footer.footer_on_first_page)
            || index > 0;
        if has_footer {
            let footer_area = Rect::new(
                self.config.margin_left,
                self.config.page_height
                    - self.config.header_footer.footer_margin
                    - self.config.header_footer.footer_height,
                self.config.content_width(),
                self.config.header_footer.footer_height,
            );
            areas.push(AreaBox::footer(footer_area));
        }

        let mut page = PageBox::new(
            index,
            Rect::new(0.0, 0.0, self.config.page_width, self.config.page_height),
            content_area,
        );
        page.areas = areas;
        page.draw_column_separators = self.config.columns.draw_separators;
        page
    }

    /// Create an empty page
    fn create_empty_page(&self, index: usize) -> PageBox {
        self.create_page(index, Vec::new())
    }

    /// Generate line numbers for all pages in the layout
    ///
    /// This method walks through all pages, blocks, and lines in the layout tree
    /// and generates line number items based on the configuration.
    fn generate_line_numbers(&self, layout: &mut LayoutTree) {
        if !self.config.line_numbering.enabled {
            return;
        }

        let mut tracker = LineNumberTracker::new(self.config.line_numbering.clone());
        let font_size = 10.0; // Default line number font size (slightly smaller than body text)

        // Collect all line numbers first to avoid borrow issues
        let mut collected_line_numbers: Vec<(usize, LineNumberItem)> = Vec::new();

        for page in &layout.pages {
            // Handle per-page restart
            if self.config.line_numbering.restart == LineNumberRestart::PerPage {
                tracker.reset();
            }

            let page_index = page.index;
            let content_area_x = page.content_area.x;
            let content_area_y = page.content_area.y;

            // Iterate through areas (content areas contain the lines)
            for area in &page.areas {
                // Only process content areas, not headers/footers
                if area.area_type != crate::AreaType::Content {
                    continue;
                }

                for column in &area.columns {
                    for block in &column.blocks {
                        for line in &block.lines {
                            // Get current line number before incrementing
                            let line_num = tracker.current_number();

                            // Increment counter
                            tracker.process_line_silent();

                            // Check if we should display this line number
                            if !self.config.line_numbering.should_display(line_num) {
                                continue;
                            }

                            // Calculate position
                            // X: right-aligned in the left margin, at distance_from_text from content
                            let x = content_area_x - self.config.line_numbering.distance_from_text;

                            // Y: baseline-aligned with the text line
                            let y = content_area_y + block.bounds.y + line.bounds.y + line.baseline;

                            // Collect the line number for later addition
                            collected_line_numbers.push((
                                page_index,
                                LineNumberItem {
                                    number: line_num,
                                    x,
                                    y,
                                    font_size,
                                },
                            ));
                        }
                    }
                }
            }
        }

        // Now add all collected line numbers to the layout
        for (page_index, item) in collected_line_numbers {
            layout.add_line_number(page_index, item);
        }
    }
}

impl Default for Paginator {
    fn default() -> Self {
        Self::new(PageConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Run};

    fn create_test_document() -> DocumentTree {
        let mut tree = DocumentTree::new();

        // Create a paragraph with some text
        let para = Paragraph::new();
        let para_id = para.id();
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        // Add a run with text
        let run = Run::new("Hello, world! This is a test paragraph with some content.");
        let run_id = run.id();
        tree.nodes.runs.insert(run_id, run);
        tree.get_paragraph_mut(para_id).unwrap().add_child(run_id);

        tree
    }

    fn create_long_document() -> DocumentTree {
        let mut tree = DocumentTree::new();

        // Create many paragraphs to force multiple pages
        for i in 0..50 {
            let para = Paragraph::new();
            let para_id = para.id();
            tree.nodes.paragraphs.insert(para_id, para);
            tree.document.add_body_child(para_id);

            let text = format!(
                "Paragraph {}. Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                 Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                 Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.",
                i + 1
            );
            let run = Run::new(&text);
            let run_id = run.id();
            tree.nodes.runs.insert(run_id, run);
            tree.get_paragraph_mut(para_id).unwrap().add_child(run_id);
        }

        tree
    }

    #[test]
    fn test_page_config_letter() {
        let config = PageConfig::letter();
        assert_eq!(config.page_width, 612.0);
        assert_eq!(config.page_height, 792.0);
        assert_eq!(config.content_width(), 468.0); // 612 - 72 - 72
        assert_eq!(config.content_height(), 648.0); // 792 - 72 - 72
    }

    #[test]
    fn test_page_config_a4() {
        let config = PageConfig::a4();
        assert!((config.page_width - 595.276).abs() < 0.01);
        assert!((config.page_height - 841.89).abs() < 0.01);
    }

    #[test]
    fn test_single_page_layout() {
        let tree = create_test_document();
        let mut paginator = Paginator::default();
        let layout = paginator.layout(&tree).unwrap();

        assert_eq!(layout.page_count(), 1);
        assert!(!layout.pages[0].areas.is_empty());
    }

    #[test]
    fn test_multi_page_layout() {
        let tree = create_long_document();
        let mut paginator = Paginator::default();
        let layout = paginator.layout(&tree).unwrap();

        // Should have multiple pages due to the amount of content
        assert!(layout.page_count() > 1, "Expected multiple pages, got {}", layout.page_count());
    }

    #[test]
    fn test_empty_document_has_one_page() {
        let tree = DocumentTree::new();
        let mut paginator = Paginator::default();
        let layout = paginator.layout(&tree).unwrap();

        assert_eq!(layout.page_count(), 1);
    }

    #[test]
    fn test_page_has_proper_bounds() {
        let tree = create_test_document();
        let mut paginator = Paginator::default();
        let layout = paginator.layout(&tree).unwrap();

        let page = &layout.pages[0];
        assert_eq!(page.bounds.x, 0.0);
        assert_eq!(page.bounds.y, 0.0);
        assert_eq!(page.bounds.width, 612.0);
        assert_eq!(page.bounds.height, 792.0);
    }

    #[test]
    fn test_content_area_respects_margins() {
        let tree = create_test_document();
        let mut paginator = Paginator::default();
        let layout = paginator.layout(&tree).unwrap();

        let page = &layout.pages[0];
        assert_eq!(page.content_area.x, 72.0);
        assert!(page.content_area.y >= 72.0); // May include header
        assert_eq!(page.content_area.width, 468.0);
    }

    #[test]
    fn test_pending_block_split() {
        // Create a pending block with 10 lines
        let lines: Vec<LineBox> = (0..10)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // Block should be splittable with default widow/orphan controls
        assert!(block.can_split(2, 2));

        // Split at line 5
        let (first, second) = block.split_at(5);
        assert_eq!(first.lines.len(), 5);
        assert_eq!(second.lines.len(), 5);
        assert!(!first.is_continuation);
        assert!(second.is_continuation);
    }

    #[test]
    fn test_find_split_point() {
        let lines: Vec<LineBox> = (0..10)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // Available height of 100 should fit 5 lines (5 * 20 = 100)
        let split = block.find_split_point(100.0, 2, 2);
        assert!(split.is_some());
        assert!(split.unwrap() <= 5);
    }

    #[test]
    fn test_cache_invalidation() {
        let tree = create_test_document();
        let mut paginator = Paginator::default();

        // Layout once
        let _ = paginator.layout(&tree).unwrap();

        // Invalidate a paragraph
        let para_id = tree.paragraphs().next().unwrap().id();
        paginator.invalidate_paragraph(para_id);

        // Check reflow state
        assert!(paginator.reflow_state.is_dirty(para_id));

        // Full invalidation
        paginator.invalidate_all();
        assert!(!paginator.reflow_state.is_dirty(para_id));
    }

    #[test]
    fn test_page_size_enum() {
        assert_eq!(PageSize::Letter.dimensions(), (612.0, 792.0));
        assert_eq!(PageSize::A4.dimensions(), (595.276, 841.89));
        assert_eq!(PageSize::Legal.dimensions(), (612.0, 1008.0));
        assert_eq!(
            PageSize::Custom { width: 500.0, height: 700.0 }.dimensions(),
            (500.0, 700.0)
        );
    }

    #[test]
    fn test_header_footer_config() {
        let config = PageConfig::letter();

        // Content height should account for header/footer on first page
        let content_height_page_0 = config.content_height_for_page(0);
        let content_height_page_1 = config.content_height_for_page(1);

        // Both pages should have header/footer by default
        assert_eq!(content_height_page_0, content_height_page_1);
        assert!(content_height_page_0 < config.content_height());
    }

    // =============================================================================
    // Advanced Pagination Rule Tests
    // =============================================================================

    #[test]
    fn test_widow_orphan_control_default() {
        let config = PageConfig::letter();

        // Default should have widow/orphan control enabled with 2 lines each
        assert_eq!(config.widow_control(), 2);
        assert_eq!(config.orphan_control(), 2);
    }

    #[test]
    fn test_widow_orphan_control_custom() {
        let control = WidowOrphanControl::with_min_lines(3, 4);
        let config = PageConfig::letter().with_widow_orphan_control(control);

        assert_eq!(config.widow_control(), 3);
        assert_eq!(config.orphan_control(), 4);
    }

    #[test]
    fn test_widow_orphan_control_disabled() {
        let control = WidowOrphanControl::disabled();
        let config = PageConfig::letter().with_widow_orphan_control(control);

        // When disabled, should return 0
        assert_eq!(config.widow_control(), 0);
        assert_eq!(config.orphan_control(), 0);
    }

    #[test]
    fn test_keep_together_prevents_split() {
        let lines: Vec<LineBox> = (0..10)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let keep_rules = ParagraphKeepRules::keep_together();
        let block = PendingBlock::with_keep_rules(
            NodeId::new(),
            lines,
            0.0,
            0.0,
            keep_rules,
        );

        // Block should NOT be splittable when keep_together is set
        assert!(!block.can_split(2, 2));
        assert!(block.keep_rules().keep_together);
    }

    #[test]
    fn test_keep_with_next_flag() {
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let keep_rules = ParagraphKeepRules::keep_with_next();
        let block = PendingBlock::with_keep_rules(
            NodeId::new(),
            lines,
            0.0,
            0.0,
            keep_rules,
        );

        assert!(block.keep_rules().keep_with_next);
        assert!(!block.keep_rules().keep_together);
    }

    #[test]
    fn test_page_break_before_flag() {
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let keep_rules = ParagraphKeepRules::page_break_before();
        let block = PendingBlock::with_keep_rules(
            NodeId::new(),
            lines,
            0.0,
            0.0,
            keep_rules,
        );

        assert!(block.keep_rules().page_break_before);
    }

    #[test]
    fn test_split_preserves_keep_with_next() {
        let lines: Vec<LineBox> = (0..10)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        // Create block with keep_with_next enabled
        let block = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines,
            0.0,
            0.0,
            true,  // keep_with_next
            false, // keep_together
            false, // page_break_before
        );

        // Split the block
        let (first, second) = block.split_at(5);

        // First part should NOT have keep_with_next (it's not the final part)
        assert!(!first.keep_with_next);
        // Second part should preserve keep_with_next
        assert!(second.keep_with_next);
    }

    #[test]
    fn test_split_marks_continuation() {
        let lines: Vec<LineBox> = (0..10)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 10.0, 10.0);

        let (first, second) = block.split_at(5);

        // First part should have original space_before
        assert_eq!(first.space_before, 10.0);
        assert_eq!(first.space_after, 0.0);

        // Second part should be marked as continuation with no space_before
        assert!(second.is_continuation);
        assert_eq!(second.space_before, 0.0);
        assert_eq!(second.space_after, 10.0);
    }

    #[test]
    fn test_no_split_with_few_lines() {
        // Create a block with only 3 lines
        let lines: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With min_lines_before=2 and min_lines_after=2,
        // a 3-line block shouldn't be splittable (would need at least 4 lines)
        assert!(!block.can_split(2, 2));
    }

    #[test]
    fn test_find_split_point_respects_min_lines() {
        // Create a block with 6 lines, each 20 points high
        let lines: Vec<LineBox> = (0..6)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 30 points available (fits 1 line), but min_lines_before=2,
        // there should be no valid split point
        let split = block.find_split_point(30.0, 2, 2);
        assert!(split.is_none());

        // With 50 points available (fits exactly 2 lines at boundary),
        // the algorithm requires MORE than min_lines_before, so this also returns None
        let split = block.find_split_point(50.0, 2, 2);
        assert!(split.is_none());

        // With 70 points available (fits 3 lines), we can split at line 3
        // leaving 3 lines on first page, 3 on second
        let split = block.find_split_point(70.0, 2, 2);
        assert_eq!(split, Some(3));
    }

    #[test]
    fn test_combined_keep_rules() {
        let rules = ParagraphKeepRules::new()
            .with_keep_with_next(true)
            .with_keep_together(true)
            .with_page_break_before(true);

        assert!(rules.keep_with_next);
        assert!(rules.keep_together);
        assert!(rules.page_break_before);
        assert!(rules.is_active());
    }

    // =============================================================================
    // Widow/Orphan Control Algorithm Tests
    // =============================================================================

    #[test]
    fn test_widow_orphan_split_all_fit() {
        // Create a block with 5 lines, each 20 points high (100 total)
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 200 points available, all lines should fit
        let split = block.calculate_widow_orphan_split(200.0, 2, 2);
        assert_eq!(split, Some(5));
    }

    #[test]
    fn test_widow_orphan_split_orphan_control() {
        // Create a block with 5 lines, each 20 points high
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With only 20 points available (1 line fits), orphan control (min 2 at bottom)
        // should move entire paragraph to next page
        let split = block.calculate_widow_orphan_split(20.0, 2, 2);
        assert!(split.is_none(), "Should move entire paragraph when only 1 line would fit (orphan control)");
    }

    #[test]
    fn test_widow_orphan_split_widow_control() {
        // Create a block with 5 lines, each 20 points high
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 80 points available (4 lines fit), widow control (min 2 at top)
        // should only allow 3 lines on current page (leaving 2 for next page)
        let split = block.calculate_widow_orphan_split(80.0, 2, 2);
        assert_eq!(split, Some(3), "Should split at 3 lines to leave 2 for next page (widow control)");
    }

    #[test]
    fn test_widow_orphan_split_combined_forces_full_move() {
        // Create a block with 4 lines, each 20 points high
        let lines: Vec<LineBox> = (0..4)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 60 points available (3 lines fit), we'd have 3 on current, 1 on next
        // Widow control requires 2 on next, so we'd need 2 on current, 2 on next
        // But 2*20=40 < 60, so 2 lines fit
        // This should result in a split at 2
        let split = block.calculate_widow_orphan_split(60.0, 2, 2);
        assert_eq!(split, Some(2), "Should split at 2 to satisfy widow control (2 on next page)");
    }

    #[test]
    fn test_widow_orphan_split_impossible() {
        // Create a block with 3 lines - cannot satisfy both orphan (2) and widow (2)
        let lines: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 40 points available (2 lines fit), we'd have 2 on current, 1 on next
        // Widow control requires 2 on next, so we need 1 on current, 2 on next
        // But orphan control requires 2 on current - contradiction!
        // Result: move entire paragraph
        let split = block.calculate_widow_orphan_split(40.0, 2, 2);
        assert!(split.is_none(), "Should move entire paragraph when both constraints can't be satisfied");
    }

    #[test]
    fn test_widow_orphan_disabled() {
        // Create a block with 5 lines, each 20 points high
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With widow/orphan control disabled (0, 0), any split should be valid
        // With 20 points available (1 line fits)
        let split = block.calculate_widow_orphan_split(20.0, 0, 0);
        assert_eq!(split, Some(1), "With disabled control, should split at 1");
    }

    #[test]
    fn test_widow_orphan_with_space_before() {
        // Create a block with 5 lines, each 20 points high, plus 10 points space_before
        let lines: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 10.0, 0.0);

        // With 50 points available (10 + 2*20 = 50), 2 lines fit
        let split = block.calculate_widow_orphan_split(50.0, 2, 2);
        assert_eq!(split, Some(2), "Should account for space_before in calculations");
    }

    #[test]
    fn test_widow_orphan_keep_together_prevents_split() {
        // Create a block with 10 lines, keep_together enabled
        let lines: Vec<LineBox> = (0..10)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines,
            0.0,
            0.0,
            false,
            true,  // keep_together
            false,
        );

        // Even with plenty of room (100 points fits 5 lines), keep_together prevents split
        let split = block.calculate_widow_orphan_split(100.0, 2, 2);
        assert!(split.is_none(), "keep_together should prevent any split");
    }

    #[test]
    fn test_widow_orphan_respects_min_lines_top() {
        // Test with min_lines_top = 3 (widow control)
        let lines: Vec<LineBox> = (0..6)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 100 points (5 lines fit), widow control of 3 means we can only put 3 on current
        let split = block.calculate_widow_orphan_split(100.0, 2, 3);
        assert_eq!(split, Some(3), "Should respect custom min_lines_top (widow control)");
    }

    #[test]
    fn test_widow_orphan_respects_min_lines_bottom() {
        // Test with min_lines_bottom = 3 (orphan control)
        let lines: Vec<LineBox> = (0..6)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With 50 points (2 lines fit), orphan control of 3 means we can't leave 2
        // Must move entire paragraph
        let split = block.calculate_widow_orphan_split(50.0, 3, 2);
        assert!(split.is_none(), "Should respect custom min_lines_bottom (orphan control)");
    }

    #[test]
    fn test_widow_orphan_exact_boundary() {
        // Create a block with 6 lines, each 20 points high
        let lines: Vec<LineBox> = (0..6)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);

        // With exactly 60 points (3 lines fit exactly), should work
        let split = block.calculate_widow_orphan_split(60.0, 2, 2);
        assert_eq!(split, Some(3), "Should handle exact boundary correctly");
    }

    // =============================================================================
    // Keep Rules Integration Tests
    // =============================================================================

    #[test]
    fn test_calculate_keep_with_next_chain_height_no_chain() {
        let paginator = Paginator::default();

        // Create a block without keep_with_next
        let lines: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::new(NodeId::new(), lines, 0.0, 0.0);
        let blocks = vec![block];

        // Should return 0 since block doesn't have keep_with_next
        let height = paginator.calculate_keep_with_next_chain_height(&blocks, 0);
        assert_eq!(height, 0.0);
    }

    #[test]
    fn test_calculate_keep_with_next_chain_height_simple() {
        let paginator = Paginator::default();

        // Create a block with keep_with_next
        let lines1: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block1 = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines1,
            0.0,
            0.0,
            true,  // keep_with_next
            false,
            false,
        );

        // Create a second block (the "next" block)
        let lines2: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block2 = PendingBlock::new(NodeId::new(), lines2, 10.0, 0.0);

        let blocks = vec![block1, block2];

        // Should reserve space for widow_control lines of next block (default 2)
        // Plus space_before of next block (10.0)
        // 2 lines * 20 = 40, + 10 space_before = 50
        let height = paginator.calculate_keep_with_next_chain_height(&blocks, 0);
        assert_eq!(height, 50.0);
    }

    #[test]
    fn test_calculate_keep_with_next_chain_height_with_keep_together() {
        let paginator = Paginator::default();

        // Create a block with keep_with_next
        let lines1: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block1 = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines1,
            0.0,
            0.0,
            true,  // keep_with_next
            false,
            false,
        );

        // Create a second block with keep_together (should require entire block)
        let lines2: Vec<LineBox> = (0..5)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block2 = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines2,
            10.0,
            5.0,
            false,
            true,  // keep_together
            false,
        );

        let blocks = vec![block1, block2];

        // Should reserve entire height of next block (5 lines * 20 = 100 + 10 space_before + 5 space_after = 115)
        let height = paginator.calculate_keep_with_next_chain_height(&blocks, 0);
        assert_eq!(height, 115.0);
    }

    #[test]
    fn test_calculate_keep_with_next_chain_height_chain() {
        let paginator = Paginator::default();

        // Create a chain: block1 -> block2 -> block3
        // where block1 and block2 have keep_with_next

        let lines1: Vec<LineBox> = (0..2)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();
        let block1 = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines1,
            0.0,
            0.0,
            true,  // keep_with_next
            false,
            false,
        );

        let lines2: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();
        let block2 = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines2,
            0.0,
            0.0,
            true,  // keep_with_next (chain continues)
            false,
            false,
        );

        let lines3: Vec<LineBox> = (0..4)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();
        let block3 = PendingBlock::new(NodeId::new(), lines3, 0.0, 0.0);

        let blocks = vec![block1, block2, block3];

        // For block1 with keep_with_next:
        // - block2 also has keep_with_next, so we need full block2 height (60)
        //   plus the chain calculation for block2
        // - block2's chain: 2 lines * 20 = 40 (widow control for block3)
        // Total: 60 + 40 = 100
        let height = paginator.calculate_keep_with_next_chain_height(&blocks, 0);
        assert_eq!(height, 100.0);
    }

    #[test]
    fn test_calculate_keep_with_next_last_block() {
        let paginator = Paginator::default();

        // Create a block with keep_with_next but no next block
        let lines: Vec<LineBox> = (0..3)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * 20.0, 400.0, 20.0),
                baseline: 15.0,
                direction: crate::Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect();

        let block = PendingBlock::with_pagination_options(
            NodeId::new(),
            lines,
            0.0,
            0.0,
            true,  // keep_with_next (but no next block)
            false,
            false,
        );
        let blocks = vec![block];

        // Should return 0 since there's no next block
        let height = paginator.calculate_keep_with_next_chain_height(&blocks, 0);
        assert_eq!(height, 0.0);
    }

    // =============================================================================
    // Line Numbering Tests
    // =============================================================================

    #[test]
    fn test_line_numbering_disabled_by_default() {
        let config = PageConfig::letter();
        assert!(!config.has_line_numbering());
    }

    #[test]
    fn test_line_numbering_enable() {
        let mut config = PageConfig::letter();
        config.enable_line_numbering();
        assert!(config.has_line_numbering());
    }

    #[test]
    fn test_line_numbering_with_config() {
        let config = PageConfig::letter()
            .with_line_numbering(LineNumbering::enabled());
        assert!(config.has_line_numbering());
    }

    #[test]
    fn test_line_numbering_every_n_lines() {
        let config = PageConfig::letter()
            .with_line_numbering(LineNumbering::every_n_lines(5));
        assert!(config.has_line_numbering());
        assert_eq!(config.line_numbering.count_by, 5);
    }

    #[test]
    fn test_line_numbering_per_page_restart() {
        let config = PageConfig::letter()
            .with_line_numbering(
                LineNumbering::enabled()
                    .with_restart(LineNumberRestart::PerPage)
            );
        assert_eq!(config.line_numbering.restart, LineNumberRestart::PerPage);
    }

    #[test]
    fn test_line_numbering_continuous() {
        let config = PageConfig::letter()
            .with_line_numbering(
                LineNumbering::enabled()
                    .with_restart(LineNumberRestart::Continuous)
            );
        assert_eq!(config.line_numbering.restart, LineNumberRestart::Continuous);
    }

    #[test]
    fn test_line_numbering_custom_distance() {
        let config = PageConfig::letter()
            .with_line_numbering(
                LineNumbering::enabled()
                    .with_distance(36.0)
            );
        assert_eq!(config.line_numbering.distance_from_text, 36.0);
    }

    #[test]
    fn test_line_numbering_start_at() {
        let config = PageConfig::letter()
            .with_line_numbering(
                LineNumbering::enabled()
                    .with_start_at(10)
            );
        assert_eq!(config.line_numbering.start_at, 10);
    }
}
