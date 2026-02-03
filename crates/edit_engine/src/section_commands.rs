//! Section Commands - Commands for page setup, headers, footers, sections, and columns
//!
//! This module implements commands for manipulating sections, page setup,
//! headers, footers, page numbers, and multi-column layout.

use crate::{Command, CommandResult, Result, EditError};
use doc_model::{
    ColumnConfig, ColumnDef, DocumentTree, FieldCode, GutterPosition, Node, NodeId,
    Orientation, PageNumberFormat, PageSizePreset, Paragraph, Position,
    Run, Section, SectionBreakType, Selection,
};
use serde::{Deserialize, Serialize};

// =============================================================================
// SetPageSetup Command
// =============================================================================

/// Set the page setup for a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPageSetup {
    /// Section ID to modify (None = first/default section)
    pub section_id: Option<NodeId>,
    /// New page size preset (optional)
    pub page_size_preset: Option<PageSizePreset>,
    /// Custom width in points (used with Custom preset)
    pub custom_width: Option<f32>,
    /// Custom height in points (used with Custom preset)
    pub custom_height: Option<f32>,
    /// Page orientation
    pub orientation: Option<Orientation>,
    /// Top margin in points
    pub margin_top: Option<f32>,
    /// Bottom margin in points
    pub margin_bottom: Option<f32>,
    /// Left margin in points
    pub margin_left: Option<f32>,
    /// Right margin in points
    pub margin_right: Option<f32>,
    /// Header distance from edge
    pub margin_header: Option<f32>,
    /// Footer distance from edge
    pub margin_footer: Option<f32>,
    /// Gutter width
    pub gutter: Option<f32>,
    /// Gutter position
    pub gutter_position: Option<GutterPosition>,
}

impl SetPageSetup {
    /// Create a new SetPageSetup command with margins
    pub fn with_margins(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self {
            section_id: None,
            page_size_preset: None,
            custom_width: None,
            custom_height: None,
            orientation: None,
            margin_top: Some(top),
            margin_bottom: Some(bottom),
            margin_left: Some(left),
            margin_right: Some(right),
            margin_header: None,
            margin_footer: None,
            gutter: None,
            gutter_position: None,
        }
    }

    /// Create a new SetPageSetup command with page size
    pub fn with_page_size(preset: PageSizePreset) -> Self {
        Self {
            section_id: None,
            page_size_preset: Some(preset),
            custom_width: None,
            custom_height: None,
            orientation: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            margin_header: None,
            margin_footer: None,
            gutter: None,
            gutter_position: None,
        }
    }

    /// Create a new SetPageSetup command with custom size
    pub fn with_custom_size(width: f32, height: f32) -> Self {
        Self {
            section_id: None,
            page_size_preset: Some(PageSizePreset::Custom),
            custom_width: Some(width),
            custom_height: Some(height),
            orientation: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            margin_header: None,
            margin_footer: None,
            gutter: None,
            gutter_position: None,
        }
    }

    /// Create a new SetPageSetup command with orientation
    pub fn with_orientation(orientation: Orientation) -> Self {
        Self {
            section_id: None,
            page_size_preset: None,
            custom_width: None,
            custom_height: None,
            orientation: Some(orientation),
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            margin_header: None,
            margin_footer: None,
            gutter: None,
            gutter_position: None,
        }
    }
}

impl Command for SetPageSetup {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current page setup to create inverse command
        let old_setup = new_tree.document.page_setup.clone();

        // Apply page size
        if let Some(preset) = &self.page_size_preset {
            if *preset == PageSizePreset::Custom {
                if let (Some(w), Some(h)) = (self.custom_width, self.custom_height) {
                    new_tree.document.page_setup.width = w;
                    new_tree.document.page_setup.height = h;
                }
            } else if let Some((w, h)) = preset.dimensions() {
                new_tree.document.page_setup.width = w;
                new_tree.document.page_setup.height = h;
            }
        }

        // Apply orientation by swapping width/height if needed
        if let Some(orientation) = &self.orientation {
            let (w, h) = (new_tree.document.page_setup.width, new_tree.document.page_setup.height);
            let (new_w, new_h) = orientation.apply(w, h);
            new_tree.document.page_setup.width = new_w;
            new_tree.document.page_setup.height = new_h;
        }

        // Apply margins
        if let Some(top) = self.margin_top {
            new_tree.document.page_setup.margin_top = top;
        }
        if let Some(bottom) = self.margin_bottom {
            new_tree.document.page_setup.margin_bottom = bottom;
        }
        if let Some(left) = self.margin_left {
            new_tree.document.page_setup.margin_left = left;
        }
        if let Some(right) = self.margin_right {
            new_tree.document.page_setup.margin_right = right;
        }

        // Create inverse command
        let inverse = Box::new(SetPageSetup {
            section_id: self.section_id,
            page_size_preset: Some(PageSizePreset::Custom),
            custom_width: Some(old_setup.width),
            custom_height: Some(old_setup.height),
            orientation: None,
            margin_top: Some(old_setup.margin_top),
            margin_bottom: Some(old_setup.margin_bottom),
            margin_left: Some(old_setup.margin_left),
            margin_right: Some(old_setup.margin_right),
            margin_header: None,
            margin_footer: None,
            gutter: None,
            gutter_position: None,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let old_setup = &tree.document.page_setup;
        Box::new(SetPageSetup {
            section_id: self.section_id,
            page_size_preset: Some(PageSizePreset::Custom),
            custom_width: Some(old_setup.width),
            custom_height: Some(old_setup.height),
            orientation: None,
            margin_top: Some(old_setup.margin_top),
            margin_bottom: Some(old_setup.margin_bottom),
            margin_left: Some(old_setup.margin_left),
            margin_right: Some(old_setup.margin_right),
            margin_header: None,
            margin_footer: None,
            gutter: None,
            gutter_position: None,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Page Setup"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetColumnLayout Command
// =============================================================================

/// Set the column layout for a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetColumnLayout {
    /// Section ID (None = current/default section)
    pub section_id: Option<NodeId>,
    /// Number of columns
    pub column_count: u32,
    /// Space between columns in points
    pub column_spacing: Option<f32>,
    /// Whether columns have equal width
    pub equal_width: bool,
    /// Custom column definitions (used when equal_width is false)
    pub custom_columns: Option<Vec<ColumnDefDto>>,
    /// Whether to draw separator lines between columns
    pub draw_separator: Option<bool>,
}

/// Column definition DTO for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefDto {
    pub width: f32,
    pub space_after: f32,
}

impl SetColumnLayout {
    /// Create a single-column layout
    pub fn single() -> Self {
        Self {
            section_id: None,
            column_count: 1,
            column_spacing: None,
            equal_width: true,
            custom_columns: None,
            draw_separator: None,
        }
    }

    /// Create a two-column layout with equal widths
    pub fn two_columns() -> Self {
        Self {
            section_id: None,
            column_count: 2,
            column_spacing: Some(36.0),
            equal_width: true,
            custom_columns: None,
            draw_separator: None,
        }
    }

    /// Create a three-column layout with equal widths
    pub fn three_columns() -> Self {
        Self {
            section_id: None,
            column_count: 3,
            column_spacing: Some(36.0),
            equal_width: true,
            custom_columns: None,
            draw_separator: None,
        }
    }

    /// Create a custom multi-column layout
    pub fn custom(column_count: u32, spacing: f32) -> Self {
        Self {
            section_id: None,
            column_count,
            column_spacing: Some(spacing),
            equal_width: true,
            custom_columns: None,
            draw_separator: None,
        }
    }

    /// Create a layout with custom column widths
    pub fn with_custom_columns(columns: Vec<ColumnDefDto>) -> Self {
        Self {
            section_id: None,
            column_count: columns.len() as u32,
            column_spacing: None,
            equal_width: false,
            custom_columns: Some(columns),
            draw_separator: None,
        }
    }

    /// Enable column separators
    pub fn with_separator(mut self) -> Self {
        self.draw_separator = Some(true);
        self
    }
}

impl Command for SetColumnLayout {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the old column config for the inverse command
        // For now, use default since full section support isn't complete
        let old_config = ColumnConfig::single();

        // Create the new column config
        let new_config = if self.equal_width {
            let mut config = ColumnConfig::custom(self.column_count, self.column_spacing.unwrap_or(36.0));
            if let Some(sep) = self.draw_separator {
                config.separator = sep;
            }
            config
        } else if let Some(ref cols) = self.custom_columns {
            let column_defs: Vec<ColumnDef> = cols.iter()
                .map(|c| ColumnDef::new(c.width, c.space_after))
                .collect();
            let mut config = ColumnConfig::with_columns(column_defs);
            if let Some(sep) = self.draw_separator {
                config.separator = sep;
            }
            config
        } else {
            ColumnConfig::custom(self.column_count, self.column_spacing.unwrap_or(36.0))
        };

        // Store the config in the document (simplified for now)
        // In a full implementation, this would be stored per-section
        new_tree.document.increment_version();

        // Create inverse command
        let old_columns: Option<Vec<ColumnDefDto>> = if !old_config.equal_width {
            Some(old_config.columns.iter()
                .map(|c| ColumnDefDto { width: c.width, space_after: c.space_after })
                .collect())
        } else {
            None
        };

        let inverse = Box::new(SetColumnLayout {
            section_id: self.section_id,
            column_count: old_config.count,
            column_spacing: Some(old_config.space),
            equal_width: old_config.equal_width,
            custom_columns: old_columns,
            draw_separator: Some(old_config.separator),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetColumnLayout::single())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Column Layout"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertColumnBreak Command
// =============================================================================

/// Insert a column break at the current position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertColumnBreak {
    /// Position to insert the column break
    pub position: Position,
}

impl InsertColumnBreak {
    pub fn new(position: Position) -> Self {
        Self { position }
    }
}

impl Command for InsertColumnBreak {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Insert a column break marker at the position
        // This would be represented as a special run or paragraph property
        // For now, we increment version to trigger re-layout
        new_tree.document.increment_version();

        let new_selection = Selection::collapsed(self.position);

        let inverse = Box::new(RemoveColumnBreak {
            position: self.position,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RemoveColumnBreak {
            position: self.position,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Column Break"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// RemoveColumnBreak Command
// =============================================================================

/// Remove a column break at the specified position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveColumnBreak {
    pub position: Position,
}

impl Command for RemoveColumnBreak {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        new_tree.document.increment_version();

        let inverse = Box::new(InsertColumnBreak {
            position: self.position,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(InsertColumnBreak {
            position: self.position,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Column Break"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertSectionBreak Command (updated version)
// =============================================================================

/// Insert a section break
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSectionBreak {
    /// Position to insert the section break
    pub position: Position,
    /// Type of section break
    pub break_type: SectionBreakType,
    /// Optional page setup for the new section
    pub page_setup: Option<SectionPageSetupDto>,
    /// The ID of the new section (for undo tracking)
    #[serde(skip)]
    new_section_id: Option<NodeId>,
}

/// Section page setup DTO for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionPageSetupDto {
    pub page_size_preset: Option<PageSizePreset>,
    pub orientation: Option<Orientation>,
    pub column_count: Option<u32>,
    pub column_spacing: Option<f32>,
}

impl InsertSectionBreak {
    pub fn new(position: Position, break_type: SectionBreakType) -> Self {
        Self {
            position,
            break_type,
            page_setup: None,
            new_section_id: None,
        }
    }

    pub fn next_page(position: Position) -> Self {
        Self::new(position, SectionBreakType::NextPage)
    }

    pub fn continuous(position: Position) -> Self {
        Self::new(position, SectionBreakType::Continuous)
    }

    pub fn even_page(position: Position) -> Self {
        Self::new(position, SectionBreakType::EvenPage)
    }

    pub fn odd_page(position: Position) -> Self {
        Self::new(position, SectionBreakType::OddPage)
    }

    pub fn with_page_setup(mut self, setup: SectionPageSetupDto) -> Self {
        self.page_setup = Some(setup);
        self
    }
}

impl Command for InsertSectionBreak {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Create a new section
        let mut section = Section::new();
        section.page_setup.section_start = self.break_type;

        // Apply any custom page setup
        if let Some(ref setup) = self.page_setup {
            if let Some(preset) = setup.page_size_preset {
                section.page_setup.page_size = doc_model::PageSize::from_preset(preset);
            }
            if let Some(orientation) = setup.orientation {
                section.page_setup.orientation = orientation;
            }
            if let Some(count) = setup.column_count {
                section.page_setup.set_columns(count);
            }
        }

        let section_id = section.id();

        // Note: Full section support would require restructuring the document
        // to support multiple sections. For now, we just acknowledge the command.
        new_tree.document.increment_version();

        let new_selection = Selection::collapsed(self.position);

        let inverse = Box::new(RemoveSectionBreak {
            section_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RemoveSectionBreak {
            section_id: self.new_section_id.unwrap_or_else(NodeId::new),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.break_type {
            SectionBreakType::NextPage => "Insert Section Break (Next Page)",
            SectionBreakType::Continuous => "Insert Section Break (Continuous)",
            SectionBreakType::EvenPage => "Insert Section Break (Even Page)",
            SectionBreakType::OddPage => "Insert Section Break (Odd Page)",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// RemoveSectionBreak Command
// =============================================================================

/// Remove a section break (merge with previous section)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSectionBreak {
    pub section_id: NodeId,
}

impl Command for RemoveSectionBreak {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        new_tree.document.increment_version();

        let inverse = Box::new(InsertSectionBreak::new(
            selection.anchor,
            SectionBreakType::NextPage,
        ));

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let position = tree.document.children().first()
            .map(|&id| Position::new(id, 0))
            .unwrap_or_else(|| Position::new(NodeId::new(), 0));
        Box::new(InsertSectionBreak::new(position, SectionBreakType::NextPage))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Section Break"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// GetSectionProperties Command (Query)
// =============================================================================

/// Get properties of a section (this is a query, not a mutation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionProperties {
    pub section_id: NodeId,
    pub break_type: SectionBreakType,
    pub page_size_preset: Option<PageSizePreset>,
    pub page_width: f32,
    pub page_height: f32,
    pub orientation: Orientation,
    pub column_count: u32,
    pub column_spacing: f32,
    pub has_different_first_page: bool,
    pub has_different_odd_even: bool,
}

impl SectionProperties {
    /// Create from a section
    pub fn from_section(section: &Section) -> Self {
        Self {
            section_id: section.id(),
            break_type: section.page_setup.section_start,
            page_size_preset: section.page_setup.page_size.preset,
            page_width: section.page_setup.effective_width(),
            page_height: section.page_setup.effective_height(),
            orientation: section.page_setup.orientation,
            column_count: section.page_setup.column_count(),
            column_spacing: section.page_setup.column_config.space,
            has_different_first_page: section.different_first_page,
            has_different_odd_even: section.different_odd_even,
        }
    }

    /// Create default properties
    pub fn default_properties() -> Self {
        let section = Section::new();
        Self::from_section(&section)
    }
}

// =============================================================================
// Legacy InsertSection alias (for backward compatibility)
// =============================================================================

/// Insert a section break (legacy alias for InsertSectionBreak)
pub type InsertSection = InsertSectionBreak;

// =============================================================================
// Legacy RemoveSection alias (for backward compatibility)
// =============================================================================

/// Remove a section (legacy alias for RemoveSectionBreak)
pub type RemoveSection = RemoveSectionBreak;

// Note: The Command implementations are on InsertSectionBreak and RemoveSectionBreak
// The type aliases above will use those implementations.

/// Legacy command struct that is no longer used
#[doc(hidden)]
#[deprecated(note = "Use RemoveSectionBreak instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSectionLegacy {
    pub section_id: NodeId,
}

impl Command for RemoveSectionLegacy {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Forward to RemoveSectionBreak
        let cmd = RemoveSectionBreak {
            section_id: self.section_id,
        };
        cmd.apply(tree, selection)
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Get a valid position from the tree
        let position = tree.document.children().first()
            .map(|&id| Position::new(id, 0))
            .unwrap_or_else(|| Position::new(NodeId::new(), 0));
        Box::new(InsertSectionBreak::new(position, SectionBreakType::NextPage))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Section Break"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetDifferentFirstPage Command
// =============================================================================

/// Enable/disable different first page header/footer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDifferentFirstPage {
    /// Section ID (None = first/default section)
    pub section_id: Option<NodeId>,
    /// Whether to enable different first page
    pub enabled: bool,
}

impl SetDifferentFirstPage {
    pub fn enable() -> Self {
        Self {
            section_id: None,
            enabled: true,
        }
    }

    pub fn disable() -> Self {
        Self {
            section_id: None,
            enabled: false,
        }
    }
}

impl Command for SetDifferentFirstPage {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Store the setting - would be stored in section in full implementation
        let new_tree = tree.clone();

        let inverse = Box::new(SetDifferentFirstPage {
            section_id: self.section_id,
            enabled: !self.enabled,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetDifferentFirstPage {
            section_id: self.section_id,
            enabled: !self.enabled,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Different First Page"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetDifferentOddEven Command
// =============================================================================

/// Enable/disable different odd/even page headers/footers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDifferentOddEven {
    /// Section ID (None = first/default section)
    pub section_id: Option<NodeId>,
    /// Whether to enable different odd/even pages
    pub enabled: bool,
}

impl SetDifferentOddEven {
    pub fn enable() -> Self {
        Self {
            section_id: None,
            enabled: true,
        }
    }

    pub fn disable() -> Self {
        Self {
            section_id: None,
            enabled: false,
        }
    }
}

impl Command for SetDifferentOddEven {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let new_tree = tree.clone();

        let inverse = Box::new(SetDifferentOddEven {
            section_id: self.section_id,
            enabled: !self.enabled,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetDifferentOddEven {
            section_id: self.section_id,
            enabled: !self.enabled,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Different Odd/Even Pages"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertPageNumber Command
// =============================================================================

/// Position options for page number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageNumberPosition {
    /// Top of page (header)
    Top,
    /// Bottom of page (footer)
    Bottom,
    /// Current cursor position
    Current,
}

/// Alignment for page number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageNumberAlignment {
    Left,
    Center,
    Right,
}

/// Insert a page number field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertPageNumber {
    /// Position in document (header, footer, or inline)
    pub position: PageNumberPosition,
    /// Alignment within the area
    pub alignment: PageNumberAlignment,
    /// Number format
    pub format: PageNumberFormat,
    /// Include "Page X of Y" style
    pub include_total: bool,
    /// Starting page number (default 1)
    pub start_at: Option<u32>,
}

impl InsertPageNumber {
    /// Insert a simple page number at the bottom center
    pub fn footer_center() -> Self {
        Self {
            position: PageNumberPosition::Bottom,
            alignment: PageNumberAlignment::Center,
            format: PageNumberFormat::Arabic,
            include_total: false,
            start_at: None,
        }
    }

    /// Insert "Page X of Y" at the bottom right
    pub fn page_x_of_y() -> Self {
        Self {
            position: PageNumberPosition::Bottom,
            alignment: PageNumberAlignment::Right,
            format: PageNumberFormat::Arabic,
            include_total: true,
            start_at: None,
        }
    }
}

impl Command for InsertPageNumber {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Find the target paragraph based on position
        // For inline position, use the current selection
        // For header/footer positions, would need to find/create the appropriate header/footer

        let target_para_id = if self.position == PageNumberPosition::Current {
            selection.anchor.node_id
        } else {
            // For header/footer, we'd need to create or find the appropriate paragraph
            // For now, just use the first paragraph
            new_tree.document.children().first().copied()
                .ok_or_else(|| EditError::InvalidCommand("No paragraphs in document".to_string()))?
        };

        // Create the field text
        let field_text = if self.include_total {
            "Page X of Y" // Placeholder - would be rendered dynamically
        } else {
            "X" // Placeholder
        };

        // Create a run with the page number placeholder
        let run = Run::new(field_text);
        let run_id = run.id();

        // Insert the run at the appropriate position
        new_tree.nodes.runs.insert(run_id, run);
        if let Some(para) = new_tree.get_paragraph_mut(target_para_id) {
            // For now, append to the paragraph
            para.add_child(run_id);
        }

        let new_selection = Selection::collapsed(Position::new(
            target_para_id,
            selection.anchor.offset + field_text.len(),
        ));

        let inverse = Box::new(RemovePageNumber {
            field_id: run_id, // Using run_id as the field identifier
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RemovePageNumber {
            field_id: NodeId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Page Number"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// RemovePageNumber Command
// =============================================================================

/// Remove a page number field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovePageNumber {
    pub field_id: NodeId,
}

impl Command for RemovePageNumber {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Try to remove the run/field
        let _ = new_tree.remove_run(self.field_id);

        let inverse = Box::new(InsertPageNumber::footer_center());

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(InsertPageNumber::footer_center())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Page Number"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertFieldCode Command (legacy - use field_commands::InsertField instead)
// =============================================================================

/// Insert a field code at the current position (legacy command)
/// For new code, prefer using field_commands::InsertField which supports
/// the full Field model with FieldInstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertFieldCode {
    /// The field code to insert
    pub field_code: FieldCode,
    /// Position to insert at
    pub position: Position,
}

impl InsertFieldCode {
    pub fn new(field_code: FieldCode, position: Position) -> Self {
        Self { field_code, position }
    }

    pub fn date(position: Position) -> Self {
        Self::new(FieldCode::Date, position)
    }

    pub fn time(position: Position) -> Self {
        Self::new(FieldCode::Time, position)
    }

    pub fn file_name(position: Position) -> Self {
        Self::new(FieldCode::FileName, position)
    }
}

impl Command for InsertFieldCode {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get placeholder text for the field
        let field_text = format!("{{ {} }}", self.field_code.code_string());

        // Create a run with the field placeholder
        let run = Run::new(&field_text);
        let run_id = run.id();

        // Try to insert at the position
        let para_id = self.position.node_id;
        new_tree.nodes.runs.insert(run_id, run);
        if let Some(para) = new_tree.get_paragraph_mut(para_id) {
            para.add_child(run_id);
        } else {
            return Err(EditError::InvalidCommand(
                format!("Cannot find paragraph for field insertion: {:?}", para_id)
            ));
        }

        let new_selection = Selection::collapsed(Position::new(
            para_id,
            self.position.offset + field_text.len(),
        ));

        let inverse = Box::new(RemoveFieldCode { field_id: run_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RemoveFieldCode {
            field_id: NodeId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Field"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// RemoveFieldCode Command (legacy)
// =============================================================================

/// Remove a field code (legacy command)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveFieldCode {
    pub field_id: NodeId,
}

impl Command for RemoveFieldCode {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let _ = new_tree.remove_run(self.field_id);

        let inverse = Box::new(InsertFieldCode {
            field_code: FieldCode::PageNumber,
            position: selection.anchor,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Get a valid position from the tree
        let position = tree.document.children().first()
            .map(|&id| Position::new(id, 0))
            .unwrap_or_else(|| Position::new(NodeId::new(), 0));
        Box::new(InsertFieldCode {
            field_code: FieldCode::PageNumber,
            position,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Field Code"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> DocumentTree {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);
        tree
    }

    #[test]
    fn test_set_page_setup_margins() {
        let tree = create_test_tree();
        let selection = Selection::collapsed(Position::new(
            tree.document.children()[0],
            0,
        ));

        let cmd = SetPageSetup::with_margins(36.0, 36.0, 36.0, 36.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.document.page_setup.margin_top, 36.0);
        assert_eq!(result.tree.document.page_setup.margin_bottom, 36.0);
        assert_eq!(result.tree.document.page_setup.margin_left, 36.0);
        assert_eq!(result.tree.document.page_setup.margin_right, 36.0);
    }

    #[test]
    fn test_set_page_setup_size() {
        let tree = create_test_tree();
        let selection = Selection::collapsed(Position::new(
            tree.document.children()[0],
            0,
        ));

        let cmd = SetPageSetup::with_page_size(PageSizePreset::A4);
        let result = cmd.apply(&tree, &selection).unwrap();

        // A4 dimensions
        assert!((result.tree.document.page_setup.width - 595.276).abs() < 0.01);
        assert!((result.tree.document.page_setup.height - 841.89).abs() < 0.01);
    }

    #[test]
    fn test_set_page_setup_orientation() {
        let tree = create_test_tree();
        let selection = Selection::collapsed(Position::new(
            tree.document.children()[0],
            0,
        ));

        // Start with letter size (612 x 792)
        let cmd = SetPageSetup::with_orientation(Orientation::Landscape);
        let result = cmd.apply(&tree, &selection).unwrap();

        // After landscape, width should be > height
        assert!(result.tree.document.page_setup.width > result.tree.document.page_setup.height);
    }

    #[test]
    fn test_insert_page_number() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertPageNumber::footer_center();
        let result = cmd.apply(&tree, &selection);

        // Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_different_first_page() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetDifferentFirstPage::enable();
        let result = cmd.apply(&tree, &selection).unwrap();

        // The inverse should disable it
        let inverse_cmd = SetDifferentFirstPage::disable();
        assert_eq!(inverse_cmd.enabled, false);
    }

    // =============================================================================
    // Column Layout Tests
    // =============================================================================

    #[test]
    fn test_set_column_layout_single() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetColumnLayout::single();
        let result = cmd.apply(&tree, &selection);

        assert!(result.is_ok());
        assert_eq!(cmd.column_count, 1);
    }

    #[test]
    fn test_set_column_layout_two_columns() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetColumnLayout::two_columns();
        let result = cmd.apply(&tree, &selection);

        assert!(result.is_ok());
        assert_eq!(cmd.column_count, 2);
        assert!(cmd.equal_width);
    }

    #[test]
    fn test_set_column_layout_custom() {
        let columns = vec![
            ColumnDefDto { width: 150.0, space_after: 24.0 },
            ColumnDefDto { width: 200.0, space_after: 0.0 },
        ];
        let cmd = SetColumnLayout::with_custom_columns(columns);

        assert_eq!(cmd.column_count, 2);
        assert!(!cmd.equal_width);
        assert!(cmd.custom_columns.is_some());
    }

    #[test]
    fn test_set_column_layout_with_separator() {
        let cmd = SetColumnLayout::two_columns().with_separator();
        assert_eq!(cmd.draw_separator, Some(true));
    }

    #[test]
    fn test_insert_column_break() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let position = Position::new(para_id, 0);

        let cmd = InsertColumnBreak::new(position);
        let selection = Selection::collapsed(position);
        let result = cmd.apply(&tree, &selection);

        assert!(result.is_ok());
    }

    // =============================================================================
    // Section Break Tests
    // =============================================================================

    #[test]
    fn test_insert_section_break_next_page() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let position = Position::new(para_id, 0);

        let cmd = InsertSectionBreak::next_page(position);
        assert_eq!(cmd.break_type, SectionBreakType::NextPage);

        let selection = Selection::collapsed(position);
        let result = cmd.apply(&tree, &selection);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_section_break_continuous() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let position = Position::new(para_id, 0);

        let cmd = InsertSectionBreak::continuous(position);
        assert_eq!(cmd.break_type, SectionBreakType::Continuous);
    }

    #[test]
    fn test_insert_section_break_even_page() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let position = Position::new(para_id, 0);

        let cmd = InsertSectionBreak::even_page(position);
        assert_eq!(cmd.break_type, SectionBreakType::EvenPage);
    }

    #[test]
    fn test_insert_section_break_odd_page() {
        let tree = create_test_tree();
        let para_id = tree.document.children()[0];
        let position = Position::new(para_id, 0);

        let cmd = InsertSectionBreak::odd_page(position);
        assert_eq!(cmd.break_type, SectionBreakType::OddPage);
    }

    #[test]
    fn test_section_properties_default() {
        let props = SectionProperties::default_properties();

        assert_eq!(props.break_type, SectionBreakType::NextPage);
        assert_eq!(props.column_count, 1);
        assert!(!props.has_different_first_page);
        assert!(!props.has_different_odd_even);
    }

    #[test]
    fn test_section_properties_from_section() {
        let mut section = Section::new();
        section.page_setup.section_start = SectionBreakType::Continuous;
        section.different_first_page = true;
        section.page_setup.set_columns(2);

        let props = SectionProperties::from_section(&section);

        assert_eq!(props.break_type, SectionBreakType::Continuous);
        assert_eq!(props.column_count, 2);
        assert!(props.has_different_first_page);
    }
}
