//! Section Model - Page setup, headers, footers, and multi-column layout
//!
//! This module implements sections which divide the document for different page setups.
//! Each section can have its own page configuration, headers, footers, and column layout.
//!
//! ## Column Layout
//!
//! Sections support multi-column layout with configurable:
//! - Column count (1-10 columns)
//! - Column spacing (gutter between columns)
//! - Equal or unequal column widths
//! - Column separator lines
//! - RTL-aware column ordering

use crate::{Node, NodeId, NodeType, LineNumbering};
use serde::{Deserialize, Serialize};

// =============================================================================
// Page Size Presets
// =============================================================================

/// Standard page size presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageSizePreset {
    /// US Letter (8.5" x 11")
    Letter,
    /// A4 (210mm x 297mm)
    A4,
    /// Legal (8.5" x 14")
    Legal,
    /// A3 (297mm x 420mm)
    A3,
    /// A5 (148mm x 210mm)
    A5,
    /// B5 (182mm x 257mm)
    B5,
    /// Executive (7.25" x 10.5")
    Executive,
    /// Tabloid (11" x 17")
    Tabloid,
    /// Custom size
    Custom,
}

impl PageSizePreset {
    /// Get the dimensions in points (width, height) for portrait orientation
    pub fn dimensions(&self) -> Option<(f32, f32)> {
        match self {
            PageSizePreset::Letter => Some((612.0, 792.0)),   // 8.5" x 11" at 72 dpi
            PageSizePreset::A4 => Some((595.276, 841.89)),    // 210mm x 297mm
            PageSizePreset::Legal => Some((612.0, 1008.0)),   // 8.5" x 14"
            PageSizePreset::A3 => Some((841.89, 1190.55)),    // 297mm x 420mm
            PageSizePreset::A5 => Some((419.53, 595.28)),     // 148mm x 210mm
            PageSizePreset::B5 => Some((515.91, 728.50)),     // 182mm x 257mm
            PageSizePreset::Executive => Some((522.0, 756.0)), // 7.25" x 10.5"
            PageSizePreset::Tabloid => Some((792.0, 1224.0)),  // 11" x 17"
            PageSizePreset::Custom => None,
        }
    }

    /// Get the preset name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            PageSizePreset::Letter => "Letter",
            PageSizePreset::A4 => "A4",
            PageSizePreset::Legal => "Legal",
            PageSizePreset::A3 => "A3",
            PageSizePreset::A5 => "A5",
            PageSizePreset::B5 => "B5",
            PageSizePreset::Executive => "Executive",
            PageSizePreset::Tabloid => "Tabloid",
            PageSizePreset::Custom => "Custom",
        }
    }
}

impl Default for PageSizePreset {
    fn default() -> Self {
        PageSizePreset::Letter
    }
}

// =============================================================================
// Page Size
// =============================================================================

/// Page size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize {
    /// Width in points
    pub width: f32,
    /// Height in points
    pub height: f32,
    /// Preset that was used (None if custom size was entered directly)
    pub preset: Option<PageSizePreset>,
}

impl PageSize {
    /// Create a new page size from a preset
    pub fn from_preset(preset: PageSizePreset) -> Self {
        if let Some((width, height)) = preset.dimensions() {
            Self {
                width,
                height,
                preset: Some(preset),
            }
        } else {
            // Custom preset - use Letter dimensions as default
            Self {
                width: 612.0,
                height: 792.0,
                preset: Some(PageSizePreset::Custom),
            }
        }
    }

    /// Create a custom page size
    pub fn custom(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            preset: Some(PageSizePreset::Custom),
        }
    }

    /// Create a Letter-sized page
    pub fn letter() -> Self {
        Self::from_preset(PageSizePreset::Letter)
    }

    /// Create an A4-sized page
    pub fn a4() -> Self {
        Self::from_preset(PageSizePreset::A4)
    }

    /// Create a Legal-sized page
    pub fn legal() -> Self {
        Self::from_preset(PageSizePreset::Legal)
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self::letter()
    }
}

// =============================================================================
// Page Orientation
// =============================================================================

/// Page orientation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

impl Orientation {
    /// Apply orientation to dimensions
    pub fn apply(&self, width: f32, height: f32) -> (f32, f32) {
        match self {
            Orientation::Portrait => {
                if width > height {
                    (height, width)
                } else {
                    (width, height)
                }
            }
            Orientation::Landscape => {
                if width < height {
                    (height, width)
                } else {
                    (width, height)
                }
            }
        }
    }
}

// =============================================================================
// Page Margins
// =============================================================================

/// Page margin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMargins {
    /// Top margin in points
    pub top: f32,
    /// Bottom margin in points
    pub bottom: f32,
    /// Left margin in points
    pub left: f32,
    /// Right margin in points
    pub right: f32,
    /// Distance from page edge to header content
    pub header: f32,
    /// Distance from page edge to footer content
    pub footer: f32,
}

impl PageMargins {
    /// Create normal margins (1 inch all around)
    pub fn normal() -> Self {
        Self {
            top: 72.0,
            bottom: 72.0,
            left: 72.0,
            right: 72.0,
            header: 36.0,  // 0.5 inch from edge
            footer: 36.0,  // 0.5 inch from edge
        }
    }

    /// Create narrow margins (0.5 inch all around)
    pub fn narrow() -> Self {
        Self {
            top: 36.0,
            bottom: 36.0,
            left: 36.0,
            right: 36.0,
            header: 36.0,
            footer: 36.0,
        }
    }

    /// Create moderate margins
    pub fn moderate() -> Self {
        Self {
            top: 72.0,
            bottom: 72.0,
            left: 54.0,  // 0.75 inch
            right: 54.0,
            header: 36.0,
            footer: 36.0,
        }
    }

    /// Create wide margins (1 inch top/bottom, 2 inches left/right)
    pub fn wide() -> Self {
        Self {
            top: 72.0,
            bottom: 72.0,
            left: 144.0,  // 2 inches
            right: 144.0,
            header: 36.0,
            footer: 36.0,
        }
    }

    /// Create mirrored margins for book binding
    pub fn mirrored() -> Self {
        Self {
            top: 72.0,
            bottom: 72.0,
            left: 90.0,   // 1.25 inches (inside)
            right: 72.0,  // 1 inch (outside)
            header: 36.0,
            footer: 36.0,
        }
    }
}

impl Default for PageMargins {
    fn default() -> Self {
        Self::normal()
    }
}

// =============================================================================
// Gutter Position
// =============================================================================

/// Position of the gutter (extra margin for binding)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GutterPosition {
    #[default]
    Left,
    Top,
}

// =============================================================================
// Section Break Type
// =============================================================================

/// How a new section starts (section break type)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionBreakType {
    /// Start on a new page (default)
    #[default]
    NextPage,
    /// Continue on the same page (continuous)
    Continuous,
    /// Start on the next even page
    EvenPage,
    /// Start on the next odd page
    OddPage,
}

/// Alias for backward compatibility
pub type SectionStart = SectionBreakType;

// =============================================================================
// Column Definition
// =============================================================================

/// Definition of a single column in a multi-column layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// Width of this column in points
    pub width: f32,
    /// Space after this column (before the next column)
    pub space_after: f32,
}

impl ColumnDef {
    /// Create a new column definition
    pub fn new(width: f32, space_after: f32) -> Self {
        Self { width, space_after }
    }

    /// Create a column with default spacing
    pub fn with_width(width: f32) -> Self {
        Self {
            width,
            space_after: 36.0, // 0.5 inch default spacing
        }
    }
}

impl Default for ColumnDef {
    fn default() -> Self {
        Self {
            width: 468.0, // Default to full content width
            space_after: 36.0,
        }
    }
}

// =============================================================================
// Column Configuration
// =============================================================================

/// Configuration for multi-column layout in a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    /// Number of columns (1-10)
    pub count: u32,
    /// Default space between columns in points (used when equal_width is true)
    pub space: f32,
    /// Whether all columns have equal width
    pub equal_width: bool,
    /// Individual column definitions (used when equal_width is false)
    pub columns: Vec<ColumnDef>,
    /// Whether to draw a separator line between columns
    pub separator: bool,
}

impl ColumnConfig {
    /// Create a single-column configuration
    pub fn single() -> Self {
        Self {
            count: 1,
            space: 36.0,
            equal_width: true,
            columns: Vec::new(),
            separator: false,
        }
    }

    /// Create a two-column configuration with equal widths
    pub fn two_columns() -> Self {
        Self {
            count: 2,
            space: 36.0,
            equal_width: true,
            columns: Vec::new(),
            separator: false,
        }
    }

    /// Create a three-column configuration with equal widths
    pub fn three_columns() -> Self {
        Self {
            count: 3,
            space: 36.0,
            equal_width: true,
            columns: Vec::new(),
            separator: false,
        }
    }

    /// Create a custom multi-column configuration
    pub fn custom(count: u32, space: f32) -> Self {
        Self {
            count: count.clamp(1, 10),
            space,
            equal_width: true,
            columns: Vec::new(),
            separator: false,
        }
    }

    /// Create a configuration with custom column definitions
    pub fn with_columns(columns: Vec<ColumnDef>) -> Self {
        Self {
            count: columns.len() as u32,
            space: 36.0,
            equal_width: false,
            columns,
            separator: false,
        }
    }

    /// Enable column separator lines
    pub fn with_separator(mut self) -> Self {
        self.separator = true;
        self
    }

    /// Calculate column widths for a given content area width
    ///
    /// Returns a vector of (x_offset, width) tuples for each column.
    pub fn calculate_column_bounds(&self, content_width: f32) -> Vec<(f32, f32)> {
        if self.count <= 1 {
            return vec![(0.0, content_width)];
        }

        if self.equal_width {
            let total_spacing = self.space * (self.count - 1) as f32;
            let column_width = (content_width - total_spacing) / self.count as f32;

            (0..self.count)
                .map(|i| {
                    let x = i as f32 * (column_width + self.space);
                    (x, column_width)
                })
                .collect()
        } else {
            let mut bounds = Vec::with_capacity(self.columns.len());
            let mut x = 0.0;

            for (i, col) in self.columns.iter().enumerate() {
                bounds.push((x, col.width));
                if i < self.columns.len() - 1 {
                    x += col.width + col.space_after;
                }
            }

            bounds
        }
    }

    /// Calculate column bounds for RTL text direction
    ///
    /// Returns columns in right-to-left order.
    pub fn calculate_column_bounds_rtl(&self, content_width: f32) -> Vec<(f32, f32)> {
        let bounds = self.calculate_column_bounds(content_width);
        bounds
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, (_, width))| {
                if self.equal_width {
                    let total_spacing = self.space * (self.count - 1) as f32;
                    let column_width = (content_width - total_spacing) / self.count as f32;
                    let x = i as f32 * (column_width + self.space);
                    (x, width)
                } else {
                    // For custom columns, we need to recalculate x positions
                    let mut x = 0.0;
                    for j in 0..i {
                        x += self.columns[self.columns.len() - 1 - j].width
                            + self.columns[self.columns.len() - 1 - j].space_after;
                    }
                    (x, width)
                }
            })
            .collect()
    }

    /// Get the width of a specific column
    pub fn column_width(&self, content_width: f32, column_index: usize) -> f32 {
        if self.equal_width {
            let total_spacing = self.space * (self.count - 1) as f32;
            (content_width - total_spacing) / self.count as f32
        } else {
            self.columns
                .get(column_index)
                .map(|c| c.width)
                .unwrap_or(content_width)
        }
    }

    /// Check if configuration is valid
    pub fn is_valid(&self) -> bool {
        if self.count == 0 || self.count > 10 {
            return false;
        }
        if !self.equal_width && self.columns.len() != self.count as usize {
            return false;
        }
        true
    }
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self::single()
    }
}

// =============================================================================
// Column Break
// =============================================================================

/// A column break marker that forces content to the next column
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnBreak;

impl ColumnBreak {
    /// Create a new column break
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColumnBreak {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Page Setup
// =============================================================================

/// Complete page setup configuration for a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionPageSetup {
    /// Page size
    pub page_size: PageSize,
    /// Page orientation
    pub orientation: Orientation,
    /// Page margins
    pub margins: PageMargins,
    /// Gutter width in points (extra space for binding)
    pub gutter: f32,
    /// Gutter position
    pub gutter_position: GutterPosition,
    /// How this section starts (section break type)
    pub section_start: SectionBreakType,
    /// Column configuration for multi-column layout
    pub column_config: ColumnConfig,
    /// Vertical alignment of content on page
    pub vertical_alignment: VerticalAlignment,
    /// Line numbering configuration for this section
    pub line_numbering: LineNumbering,
    /// Text direction for this section
    pub text_direction: SectionTextDirection,
}

/// Text direction for the section (section-level)
/// This determines the primary text direction for the section (for column ordering).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionTextDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

impl SectionPageSetup {
    /// Get effective page width after applying orientation
    pub fn effective_width(&self) -> f32 {
        let (w, _h) = self.orientation.apply(self.page_size.width, self.page_size.height);
        w
    }

    /// Get effective page height after applying orientation
    pub fn effective_height(&self) -> f32 {
        let (_w, h) = self.orientation.apply(self.page_size.width, self.page_size.height);
        h
    }

    /// Get content area width (excluding margins and gutter)
    pub fn content_width(&self) -> f32 {
        let gutter_width = if self.gutter_position == GutterPosition::Left {
            self.gutter
        } else {
            0.0
        };
        self.effective_width() - self.margins.left - self.margins.right - gutter_width
    }

    /// Get content area height (excluding margins)
    pub fn content_height(&self) -> f32 {
        let gutter_height = if self.gutter_position == GutterPosition::Top {
            self.gutter
        } else {
            0.0
        };
        self.effective_height() - self.margins.top - self.margins.bottom - gutter_height
    }

    /// Get the left margin including gutter if applicable
    pub fn effective_left_margin(&self) -> f32 {
        if self.gutter_position == GutterPosition::Left {
            self.margins.left + self.gutter
        } else {
            self.margins.left
        }
    }

    /// Get the top margin including gutter if applicable
    pub fn effective_top_margin(&self) -> f32 {
        if self.gutter_position == GutterPosition::Top {
            self.margins.top + self.gutter
        } else {
            self.margins.top
        }
    }

    /// Get the number of columns in this section
    pub fn column_count(&self) -> u32 {
        self.column_config.count
    }

    /// Get column bounds for layout
    ///
    /// Returns (x_offset, width) pairs for each column, respecting text direction.
    pub fn column_bounds(&self) -> Vec<(f32, f32)> {
        let content_width = self.content_width();
        if self.text_direction == SectionTextDirection::RightToLeft {
            self.column_config.calculate_column_bounds_rtl(content_width)
        } else {
            self.column_config.calculate_column_bounds(content_width)
        }
    }

    /// Get the width of a specific column
    pub fn column_width(&self, column_index: usize) -> f32 {
        self.column_config.column_width(self.content_width(), column_index)
    }

    /// Set the number of columns (with equal widths)
    pub fn set_columns(&mut self, count: u32) {
        self.column_config = ColumnConfig::custom(count, self.column_config.space);
    }

    /// Set custom column configuration
    pub fn set_column_config(&mut self, config: ColumnConfig) {
        self.column_config = config;
    }

    /// Check if this is a multi-column section
    pub fn is_multi_column(&self) -> bool {
        self.column_config.count > 1
    }

    /// Check if line numbering is enabled
    pub fn has_line_numbers(&self) -> bool {
        self.line_numbering.enabled
    }

    /// Set line numbering configuration
    pub fn set_line_numbering(&mut self, config: LineNumbering) {
        self.line_numbering = config;
    }

    /// Enable line numbering with default settings
    pub fn enable_line_numbering(&mut self) {
        self.line_numbering = LineNumbering::enabled();
    }

    /// Disable line numbering
    pub fn disable_line_numbering(&mut self) {
        self.line_numbering.enabled = false;
    }
}

impl Default for SectionPageSetup {
    fn default() -> Self {
        Self {
            page_size: PageSize::default(),
            orientation: Orientation::default(),
            margins: PageMargins::default(),
            gutter: 0.0,
            gutter_position: GutterPosition::default(),
            section_start: SectionBreakType::default(),
            column_config: ColumnConfig::default(),
            vertical_alignment: VerticalAlignment::default(),
            line_numbering: LineNumbering::default(),
            text_direction: SectionTextDirection::default(),
        }
    }
}

/// Vertical alignment of page content
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlignment {
    #[default]
    Top,
    Center,
    Justified,
    Bottom,
}

// =============================================================================
// Header/Footer
// =============================================================================

/// A header or footer container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooter {
    /// Unique identifier
    id: NodeId,
    /// Parent section ID
    parent: Option<NodeId>,
    /// Content - paragraph node IDs
    children: Vec<NodeId>,
}

impl HeaderFooter {
    /// Create a new empty header/footer
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Add a content paragraph
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Insert a content paragraph at a specific index
    pub fn insert_child(&mut self, index: usize, child_id: NodeId) {
        self.children.insert(index, child_id);
    }

    /// Remove a content paragraph
    pub fn remove_child(&mut self, child_id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if this header/footer has any content
    pub fn has_content(&self) -> bool {
        !self.children.is_empty()
    }
}

impl Default for HeaderFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for HeaderFooter {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Field // Using Field type for now, could add HeaderFooter type
    }

    fn children(&self) -> &[NodeId] {
        &self.children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        true
    }
}

// =============================================================================
// Header/Footer Set
// =============================================================================

/// Complete set of headers or footers for a section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeaderFooterSet {
    /// Default header/footer (used when others don't apply)
    pub default: Option<HeaderFooter>,
    /// Header/footer for the first page only
    pub first_page: Option<HeaderFooter>,
    /// Header/footer for odd pages (when different odd/even is enabled)
    pub odd: Option<HeaderFooter>,
    /// Header/footer for even pages
    pub even: Option<HeaderFooter>,
}

impl HeaderFooterSet {
    /// Create a new empty set
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the appropriate header/footer for a given page
    pub fn get_for_page(
        &self,
        page_index: usize,
        is_first_page: bool,
        different_first_page: bool,
        different_odd_even: bool,
    ) -> Option<&HeaderFooter> {
        // First page different?
        if is_first_page && different_first_page {
            if let Some(ref hf) = self.first_page {
                return Some(hf);
            }
        }

        // Odd/even different?
        if different_odd_even {
            let is_odd = (page_index % 2) == 0; // 0-indexed, so first page is odd
            if is_odd {
                if let Some(ref hf) = self.odd {
                    return Some(hf);
                }
            } else {
                if let Some(ref hf) = self.even {
                    return Some(hf);
                }
            }
        }

        // Default
        self.default.as_ref()
    }

    /// Check if any header/footer is defined
    pub fn has_any(&self) -> bool {
        self.default.is_some()
            || self.first_page.is_some()
            || self.odd.is_some()
            || self.even.is_some()
    }
}

// =============================================================================
// Section
// =============================================================================

/// A section in the document
///
/// Sections divide the document to allow different page setups, headers, and footers.
/// The first section uses the default page setup; subsequent sections can override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Unique identifier
    id: NodeId,
    /// Parent document ID
    parent: Option<NodeId>,
    /// Page setup for this section
    pub page_setup: SectionPageSetup,
    /// Headers for this section
    pub headers: HeaderFooterSet,
    /// Footers for this section
    pub footers: HeaderFooterSet,
    /// Whether this section has a different first page header/footer
    pub different_first_page: bool,
    /// Whether this section has different odd/even headers/footers
    pub different_odd_even: bool,
    /// Link to previous section (use previous section's headers/footers)
    pub link_to_previous: bool,
    /// Child paragraph and table IDs (content of this section)
    children: Vec<NodeId>,
}

impl Section {
    /// Create a new section with default page setup
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            page_setup: SectionPageSetup::default(),
            headers: HeaderFooterSet::default(),
            footers: HeaderFooterSet::default(),
            different_first_page: false,
            different_odd_even: false,
            link_to_previous: false,
            children: Vec::new(),
        }
    }

    /// Create a section with specific page setup
    pub fn with_page_setup(page_setup: SectionPageSetup) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            page_setup,
            headers: HeaderFooterSet::default(),
            footers: HeaderFooterSet::default(),
            different_first_page: false,
            different_odd_even: false,
            link_to_previous: false,
            children: Vec::new(),
        }
    }

    /// Add a child element (paragraph or table)
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Insert a child at a specific index
    pub fn insert_child(&mut self, index: usize, child_id: NodeId) {
        self.children.insert(index, child_id);
    }

    /// Remove a child by ID
    pub fn remove_child(&mut self, child_id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the header for a specific page
    pub fn get_header_for_page(&self, page_index: usize, is_first_page_of_section: bool) -> Option<&HeaderFooter> {
        self.headers.get_for_page(
            page_index,
            is_first_page_of_section,
            self.different_first_page,
            self.different_odd_even,
        )
    }

    /// Get the footer for a specific page
    pub fn get_footer_for_page(&self, page_index: usize, is_first_page_of_section: bool) -> Option<&HeaderFooter> {
        self.footers.get_for_page(
            page_index,
            is_first_page_of_section,
            self.different_first_page,
            self.different_odd_even,
        )
    }

    /// Set the default header
    pub fn set_default_header(&mut self, header: HeaderFooter) {
        self.headers.default = Some(header);
    }

    /// Set the default footer
    pub fn set_default_footer(&mut self, footer: HeaderFooter) {
        self.footers.default = Some(footer);
    }

    /// Set the first page header
    pub fn set_first_page_header(&mut self, header: HeaderFooter) {
        self.headers.first_page = Some(header);
    }

    /// Set the first page footer
    pub fn set_first_page_footer(&mut self, footer: HeaderFooter) {
        self.footers.first_page = Some(footer);
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Section {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Section
    }

    fn children(&self) -> &[NodeId] {
        &self.children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        true
    }
}

// =============================================================================
// Page Number Format (for header/footer fields)
// =============================================================================

/// Format for page numbers in headers/footers
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageNumberFormat {
    /// Arabic numerals (1, 2, 3...)
    #[default]
    Arabic,
    /// Lowercase letters (a, b, c...)
    LowercaseLetter,
    /// Uppercase letters (A, B, C...)
    UppercaseLetter,
    /// Lowercase Roman numerals (i, ii, iii...)
    LowercaseRoman,
    /// Uppercase Roman numerals (I, II, III...)
    UppercaseRoman,
}

// =============================================================================
// Field Code (for header/footer dynamic content)
// =============================================================================

/// Field codes for dynamic content in headers/footers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldCode {
    /// Current page number
    PageNumber,
    /// Total number of pages
    NumPages,
    /// Current date
    Date,
    /// Current time
    Time,
    /// File name
    FileName,
    /// File path
    FilePath,
    /// Document title
    Title,
    /// Document author
    Author,
}

impl Default for FieldCode {
    fn default() -> Self {
        FieldCode::PageNumber
    }
}

impl FieldCode {
    /// Get the field code string representation
    pub fn code_string(&self) -> &'static str {
        match self {
            FieldCode::PageNumber => "PAGE",
            FieldCode::NumPages => "NUMPAGES",
            FieldCode::Date => "DATE",
            FieldCode::Time => "TIME",
            FieldCode::FileName => "FILENAME",
            FieldCode::FilePath => "FILEPATH",
            FieldCode::Title => "TITLE",
            FieldCode::Author => "AUTHOR",
        }
    }

    /// Parse a field code from a string
    pub fn from_code_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PAGE" => Some(FieldCode::PageNumber),
            "NUMPAGES" => Some(FieldCode::NumPages),
            "DATE" => Some(FieldCode::Date),
            "TIME" => Some(FieldCode::Time),
            "FILENAME" => Some(FieldCode::FileName),
            "FILEPATH" => Some(FieldCode::FilePath),
            "TITLE" => Some(FieldCode::Title),
            "AUTHOR" => Some(FieldCode::Author),
            _ => None,
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
    fn test_page_size_presets() {
        let letter = PageSize::letter();
        assert_eq!(letter.width, 612.0);
        assert_eq!(letter.height, 792.0);
        assert_eq!(letter.preset, Some(PageSizePreset::Letter));

        let a4 = PageSize::a4();
        assert!((a4.width - 595.276).abs() < 0.01);
        assert!((a4.height - 841.89).abs() < 0.01);
    }

    #[test]
    fn test_orientation() {
        let portrait = Orientation::Portrait;
        let landscape = Orientation::Landscape;

        // Portrait should ensure height > width
        let (w, h) = portrait.apply(792.0, 612.0);
        assert!(h > w);

        // Landscape should ensure width > height
        let (w, h) = landscape.apply(612.0, 792.0);
        assert!(w > h);
    }

    #[test]
    fn test_page_setup_content_dimensions() {
        let setup = SectionPageSetup::default();

        // Letter size: 612 x 792
        // Normal margins: 72 all around
        // Content: 612 - 72 - 72 = 468 wide
        // Content: 792 - 72 - 72 = 648 high
        assert_eq!(setup.content_width(), 468.0);
        assert_eq!(setup.content_height(), 648.0);
    }

    #[test]
    fn test_page_setup_with_gutter() {
        let mut setup = SectionPageSetup::default();
        setup.gutter = 36.0; // 0.5 inch gutter
        setup.gutter_position = GutterPosition::Left;

        // Content width should be reduced by gutter
        assert_eq!(setup.content_width(), 432.0); // 468 - 36

        // Effective left margin should include gutter
        assert_eq!(setup.effective_left_margin(), 108.0); // 72 + 36
    }

    #[test]
    fn test_header_footer_set_selection() {
        let mut set = HeaderFooterSet::new();

        // Set up different headers
        set.default = Some(HeaderFooter::new());
        set.first_page = Some(HeaderFooter::new());
        set.odd = Some(HeaderFooter::new());
        set.even = Some(HeaderFooter::new());

        // First page with different_first_page enabled
        let hf = set.get_for_page(0, true, true, false);
        assert!(hf.is_some());

        // Odd page with different_odd_even enabled
        let hf = set.get_for_page(2, false, false, true);
        assert!(hf.is_some());
    }

    #[test]
    fn test_section_creation() {
        let section = Section::new();

        assert!(section.children().is_empty());
        assert!(!section.different_first_page);
        assert!(!section.different_odd_even);
        assert!(!section.link_to_previous);
    }

    #[test]
    fn test_section_children() {
        let mut section = Section::new();
        let child1 = NodeId::new();
        let child2 = NodeId::new();

        section.add_child(child1);
        section.add_child(child2);

        assert_eq!(section.children().len(), 2);
        assert_eq!(section.children()[0], child1);
        assert_eq!(section.children()[1], child2);

        section.remove_child(child1);
        assert_eq!(section.children().len(), 1);
        assert_eq!(section.children()[0], child2);
    }

    #[test]
    fn test_margins_presets() {
        let normal = PageMargins::normal();
        assert_eq!(normal.top, 72.0);
        assert_eq!(normal.left, 72.0);

        let narrow = PageMargins::narrow();
        assert_eq!(narrow.top, 36.0);
        assert_eq!(narrow.left, 36.0);

        let wide = PageMargins::wide();
        assert_eq!(wide.left, 144.0);
    }

    // =============================================================================
    // Column Configuration Tests
    // =============================================================================

    #[test]
    fn test_column_config_single() {
        let config = ColumnConfig::single();
        assert_eq!(config.count, 1);
        assert!(config.equal_width);

        let bounds = config.calculate_column_bounds(468.0);
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0], (0.0, 468.0));
    }

    #[test]
    fn test_column_config_two_columns() {
        let config = ColumnConfig::two_columns();
        assert_eq!(config.count, 2);
        assert!(config.equal_width);

        // Content width: 468, spacing: 36
        // Column width: (468 - 36) / 2 = 216
        let bounds = config.calculate_column_bounds(468.0);
        assert_eq!(bounds.len(), 2);
        assert!((bounds[0].1 - 216.0).abs() < 0.01);
        assert!((bounds[1].1 - 216.0).abs() < 0.01);
        assert!((bounds[1].0 - 252.0).abs() < 0.01); // 216 + 36 = 252
    }

    #[test]
    fn test_column_config_three_columns() {
        let config = ColumnConfig::three_columns();
        assert_eq!(config.count, 3);

        // Content width: 468, spacing: 36 * 2 = 72
        // Column width: (468 - 72) / 3 = 132
        let bounds = config.calculate_column_bounds(468.0);
        assert_eq!(bounds.len(), 3);
        assert!((bounds[0].1 - 132.0).abs() < 0.01);
    }

    #[test]
    fn test_column_config_custom() {
        let columns = vec![
            ColumnDef::new(150.0, 24.0),
            ColumnDef::new(200.0, 24.0),
            ColumnDef::new(70.0, 0.0),
        ];
        let config = ColumnConfig::with_columns(columns);
        assert_eq!(config.count, 3);
        assert!(!config.equal_width);

        let bounds = config.calculate_column_bounds(468.0);
        assert_eq!(bounds.len(), 3);
        assert_eq!(bounds[0], (0.0, 150.0));
        assert_eq!(bounds[1], (174.0, 200.0)); // 150 + 24 = 174
        assert_eq!(bounds[2], (398.0, 70.0));  // 174 + 200 + 24 = 398
    }

    #[test]
    fn test_column_config_rtl() {
        let config = ColumnConfig::two_columns();

        // LTR: col0 at 0, col1 at 252
        // RTL: col0 at 252, col1 at 0 (reversed visual order)
        let bounds_ltr = config.calculate_column_bounds(468.0);
        let bounds_rtl = config.calculate_column_bounds_rtl(468.0);

        assert_eq!(bounds_ltr.len(), 2);
        assert_eq!(bounds_rtl.len(), 2);

        // RTL should have the same column widths but reversed positions
        assert!((bounds_rtl[0].0 - bounds_ltr[0].0).abs() < 0.01);
        assert!((bounds_rtl[1].0 - bounds_ltr[1].0).abs() < 0.01);
    }

    #[test]
    fn test_column_config_with_separator() {
        let config = ColumnConfig::two_columns().with_separator();
        assert!(config.separator);
    }

    #[test]
    fn test_column_config_validation() {
        let valid = ColumnConfig::two_columns();
        assert!(valid.is_valid());

        let invalid = ColumnConfig {
            count: 3,
            equal_width: false,
            columns: vec![ColumnDef::default()], // Only 1 column but count is 3
            ..Default::default()
        };
        assert!(!invalid.is_valid());

        let invalid_count = ColumnConfig {
            count: 0,
            ..Default::default()
        };
        assert!(!invalid_count.is_valid());
    }

    #[test]
    fn test_section_page_setup_columns() {
        let mut setup = SectionPageSetup::default();
        assert_eq!(setup.column_count(), 1);
        assert!(!setup.is_multi_column());

        setup.set_columns(2);
        assert_eq!(setup.column_count(), 2);
        assert!(setup.is_multi_column());

        let bounds = setup.column_bounds();
        assert_eq!(bounds.len(), 2);
    }

    #[test]
    fn test_section_break_types() {
        assert_eq!(SectionBreakType::default(), SectionBreakType::NextPage);

        let section = Section::new();
        assert_eq!(section.page_setup.section_start, SectionBreakType::NextPage);
    }

    #[test]
    fn test_column_break() {
        let _break = ColumnBreak::new();
        let _break2 = ColumnBreak::default();
    }

    #[test]
    fn test_text_direction() {
        let mut setup = SectionPageSetup::default();
        assert_eq!(setup.text_direction, SectionTextDirection::LeftToRight);

        setup.text_direction = SectionTextDirection::RightToLeft;
        assert_eq!(setup.text_direction, SectionTextDirection::RightToLeft);
    }
}
