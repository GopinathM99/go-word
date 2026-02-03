//! Table model - Tables, rows, cells, and grid definitions
//!
//! This module implements the table data model for the word processor,
//! supporting tables with rows and cells, column definitions, cell properties,
//! and spanning (grid_span for column spanning, row_span for row spanning).
//!
//! Advanced features:
//! - Cell merging (horizontal and vertical)
//! - Header row repeat across pages
//! - Row breaking control
//! - Nested tables
//! - Auto-fit modes
//! - Text direction per cell

use crate::{Node, NodeId, NodeType, StyleId};
use serde::{Deserialize, Serialize};

// =============================================================================
// Width Types
// =============================================================================

/// How width is specified for columns and tables
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WidthType {
    /// Fixed width in points
    Fixed,
    /// Auto-fit to content
    Auto,
    /// Percentage of available width
    Percent,
}

impl Default for WidthType {
    fn default() -> Self {
        WidthType::Auto
    }
}

/// Width specification with value and type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TableWidth {
    /// Width value (interpretation depends on width_type)
    pub value: f32,
    /// How to interpret the value
    pub width_type: WidthType,
}

impl Default for TableWidth {
    fn default() -> Self {
        Self {
            value: 0.0,
            width_type: WidthType::Auto,
        }
    }
}

impl TableWidth {
    /// Create a fixed width in points
    pub fn fixed(points: f32) -> Self {
        Self {
            value: points,
            width_type: WidthType::Fixed,
        }
    }

    /// Create an auto width
    pub fn auto() -> Self {
        Self {
            value: 0.0,
            width_type: WidthType::Auto,
        }
    }

    /// Create a percentage width
    pub fn percent(pct: f32) -> Self {
        Self {
            value: pct,
            width_type: WidthType::Percent,
        }
    }
}

// =============================================================================
// Height Rules
// =============================================================================

/// How row height is determined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeightRule {
    /// Height is automatically determined by content
    Auto,
    /// Height is exactly as specified
    Exact,
    /// Height is at least as specified (can grow)
    AtLeast,
}

impl Default for HeightRule {
    fn default() -> Self {
        HeightRule::Auto
    }
}

// =============================================================================
// Vertical Alignment
// =============================================================================

/// Vertical alignment within a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CellVerticalAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

// =============================================================================
// Cell Text Direction
// =============================================================================

/// Text direction for cell content (distinct from paragraph text direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CellTextDirection {
    /// Left-to-right (default for most languages)
    #[default]
    Ltr,
    /// Right-to-left (Arabic, Hebrew, etc.)
    Rtl,
    /// Top-to-bottom, left-to-right (East Asian vertical)
    TbLr,
    /// Top-to-bottom, right-to-left
    TbRl,
}

// =============================================================================
// Cell Merge State
// =============================================================================

/// Horizontal merge state for a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HorizontalMerge {
    /// Cell is not part of a horizontal merge
    #[default]
    None,
    /// Cell starts a horizontal merge
    Start,
    /// Cell continues a horizontal merge (covered by the start cell)
    Continue,
}

/// Vertical merge state for a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VerticalMerge {
    /// Cell is not part of a vertical merge
    #[default]
    None,
    /// Cell starts a vertical merge
    Start,
    /// Cell continues a vertical merge (covered by the cell above)
    Continue,
}

// =============================================================================
// Table Auto-fit Mode
// =============================================================================

/// How the table width is determined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TableAutoFitMode {
    /// Auto-fit to content - columns sized to fit their content
    #[default]
    AutoFitContent,
    /// Auto-fit to window/page - table fills available width
    AutoFitWindow,
    /// Fixed width - use specified column widths
    FixedWidth,
}

// =============================================================================
// Border Styles
// =============================================================================

/// Border style type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TableBorderStyle {
    #[default]
    None,
    Single,
    Double,
    Dotted,
    Dashed,
    Thick,
}

/// A single border definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableBorder {
    /// Border style
    pub style: TableBorderStyle,
    /// Border width in points
    pub width: f32,
    /// Border color (CSS color string)
    pub color: String,
}

impl Default for TableBorder {
    fn default() -> Self {
        Self {
            style: TableBorderStyle::Single,
            width: 0.5,
            color: "#000000".to_string(),
        }
    }
}

impl TableBorder {
    /// Create a simple single-line border
    pub fn single(width: f32, color: &str) -> Self {
        Self {
            style: TableBorderStyle::Single,
            width,
            color: color.to_string(),
        }
    }

    /// Create a border with no line
    pub fn none() -> Self {
        Self {
            style: TableBorderStyle::None,
            width: 0.0,
            color: String::new(),
        }
    }
}

/// Cell borders (all four sides)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CellBorders {
    pub top: Option<TableBorder>,
    pub bottom: Option<TableBorder>,
    pub left: Option<TableBorder>,
    pub right: Option<TableBorder>,
}

impl CellBorders {
    /// Create borders with all sides the same
    pub fn all(border: TableBorder) -> Self {
        Self {
            top: Some(border.clone()),
            bottom: Some(border.clone()),
            left: Some(border.clone()),
            right: Some(border),
        }
    }

    /// Create default borders (single black line)
    pub fn default_borders() -> Self {
        Self::all(TableBorder::default())
    }
}

/// Table borders (includes inside borders)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TableBorders {
    pub top: Option<TableBorder>,
    pub bottom: Option<TableBorder>,
    pub left: Option<TableBorder>,
    pub right: Option<TableBorder>,
    pub inside_h: Option<TableBorder>,
    pub inside_v: Option<TableBorder>,
}

impl TableBorders {
    /// Create borders with all sides the same
    pub fn all(border: TableBorder) -> Self {
        Self {
            top: Some(border.clone()),
            bottom: Some(border.clone()),
            left: Some(border.clone()),
            right: Some(border.clone()),
            inside_h: Some(border.clone()),
            inside_v: Some(border),
        }
    }

    /// Create default borders
    pub fn default_borders() -> Self {
        Self::all(TableBorder::default())
    }
}

// =============================================================================
// Cell Padding
// =============================================================================

/// Padding inside a cell
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CellPadding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for CellPadding {
    fn default() -> Self {
        Self {
            top: 2.0,
            bottom: 2.0,
            left: 5.0,
            right: 5.0,
        }
    }
}

impl CellPadding {
    /// Create uniform padding
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    /// Create padding with separate horizontal and vertical values
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}

// =============================================================================
// Grid Column
// =============================================================================

/// Definition of a column in the table grid
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridColumn {
    /// Column width
    pub width: TableWidth,
}

impl Default for GridColumn {
    fn default() -> Self {
        Self {
            width: TableWidth::auto(),
        }
    }
}

impl GridColumn {
    /// Create a column with fixed width
    pub fn fixed(points: f32) -> Self {
        Self {
            width: TableWidth::fixed(points),
        }
    }

    /// Create a column with auto width
    pub fn auto() -> Self {
        Self {
            width: TableWidth::auto(),
        }
    }

    /// Create a column with percentage width
    pub fn percent(pct: f32) -> Self {
        Self {
            width: TableWidth::percent(pct),
        }
    }
}

// =============================================================================
// Table Grid
// =============================================================================

/// Grid definition for a table (column layout)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TableGrid {
    /// Column definitions
    pub columns: Vec<GridColumn>,
}

impl TableGrid {
    /// Create a new grid with the specified number of columns
    pub fn new(column_count: usize) -> Self {
        Self {
            columns: vec![GridColumn::default(); column_count],
        }
    }

    /// Create a grid with fixed-width columns
    pub fn with_fixed_columns(widths: &[f32]) -> Self {
        Self {
            columns: widths.iter().map(|&w| GridColumn::fixed(w)).collect(),
        }
    }

    /// Create a grid with equal-width columns
    pub fn with_equal_columns(count: usize, total_width: f32) -> Self {
        let col_width = total_width / count as f32;
        Self {
            columns: vec![GridColumn::fixed(col_width); count],
        }
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Add a column
    pub fn add_column(&mut self, column: GridColumn) {
        self.columns.push(column);
    }

    /// Insert a column at an index
    pub fn insert_column(&mut self, index: usize, column: GridColumn) {
        if index <= self.columns.len() {
            self.columns.insert(index, column);
        }
    }

    /// Remove a column
    pub fn remove_column(&mut self, index: usize) -> Option<GridColumn> {
        if index < self.columns.len() {
            Some(self.columns.remove(index))
        } else {
            None
        }
    }
}

// =============================================================================
// Cell Properties
// =============================================================================

/// Properties for a table cell
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CellProperties {
    /// Cell borders
    pub borders: Option<CellBorders>,
    /// Cell background/shading color (CSS color string)
    pub shading: Option<String>,
    /// Cell padding/margins
    pub padding: Option<CellPadding>,
    /// Vertical alignment within the cell
    pub vertical_align: Option<CellVerticalAlign>,
    /// Cell width override
    pub width: Option<TableWidth>,
    /// Text direction for cell content
    pub text_direction: Option<CellTextDirection>,
    /// Cell margins (separate from padding for DOCX compatibility)
    pub margins: Option<CellPadding>,
    /// No-wrap: prevent text wrapping in cell
    pub no_wrap: bool,
}

impl CellProperties {
    /// Create default cell properties
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the shading color
    pub fn with_shading(mut self, color: &str) -> Self {
        self.shading = Some(color.to_string());
        self
    }

    /// Set the borders
    pub fn with_borders(mut self, borders: CellBorders) -> Self {
        self.borders = Some(borders);
        self
    }

    /// Set the padding
    pub fn with_padding(mut self, padding: CellPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the vertical alignment
    pub fn with_vertical_align(mut self, align: CellVerticalAlign) -> Self {
        self.vertical_align = Some(align);
        self
    }

    /// Set the text direction
    pub fn with_text_direction(mut self, direction: CellTextDirection) -> Self {
        self.text_direction = Some(direction);
        self
    }

    /// Set cell margins
    pub fn with_margins(mut self, margins: CellPadding) -> Self {
        self.margins = Some(margins);
        self
    }

    /// Set no-wrap flag
    pub fn with_no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = no_wrap;
        self
    }
}

// =============================================================================
// Table Cell
// =============================================================================

/// A cell in a table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    id: NodeId,
    parent: Option<NodeId>,
    /// IDs of child nodes (paragraphs, tables, etc.)
    children: Vec<NodeId>,
    /// Cell properties
    pub properties: CellProperties,
    /// Number of grid columns this cell spans (colspan)
    pub grid_span: u32,
    /// Number of rows this cell spans (rowspan)
    pub row_span: u32,
    /// Horizontal merge state
    pub h_merge: HorizontalMerge,
    /// Vertical merge state
    pub v_merge: VerticalMerge,
    /// Whether this cell is a continuation of a vertical merge (deprecated, use v_merge)
    pub v_merge_continue: bool,
}

impl TableCell {
    /// Create a new empty cell
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties: CellProperties::default(),
            grid_span: 1,
            row_span: 1,
            h_merge: HorizontalMerge::None,
            v_merge: VerticalMerge::None,
            v_merge_continue: false,
        }
    }

    /// Create a cell with properties
    pub fn with_properties(properties: CellProperties) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties,
            grid_span: 1,
            row_span: 1,
            h_merge: HorizontalMerge::None,
            v_merge: VerticalMerge::None,
            v_merge_continue: false,
        }
    }

    /// Create a cell that spans multiple columns
    pub fn spanning_columns(grid_span: u32) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties: CellProperties::default(),
            grid_span,
            row_span: 1,
            h_merge: if grid_span > 1 { HorizontalMerge::Start } else { HorizontalMerge::None },
            v_merge: VerticalMerge::None,
            v_merge_continue: false,
        }
    }

    /// Create a cell that spans multiple rows
    pub fn spanning_rows(row_span: u32) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties: CellProperties::default(),
            grid_span: 1,
            row_span,
            h_merge: HorizontalMerge::None,
            v_merge: if row_span > 1 { VerticalMerge::Start } else { VerticalMerge::None },
            v_merge_continue: false,
        }
    }

    /// Create a cell that spans both rows and columns
    pub fn spanning(grid_span: u32, row_span: u32) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties: CellProperties::default(),
            grid_span,
            row_span,
            h_merge: if grid_span > 1 { HorizontalMerge::Start } else { HorizontalMerge::None },
            v_merge: if row_span > 1 { VerticalMerge::Start } else { VerticalMerge::None },
            v_merge_continue: false,
        }
    }

    /// Create a cell that continues a horizontal merge
    pub fn h_merge_continue() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties: CellProperties::default(),
            grid_span: 1,
            row_span: 1,
            h_merge: HorizontalMerge::Continue,
            v_merge: VerticalMerge::None,
            v_merge_continue: false,
        }
    }

    /// Create a cell that continues a vertical merge
    pub fn v_merge_continue_cell() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            properties: CellProperties::default(),
            grid_span: 1,
            row_span: 1,
            h_merge: HorizontalMerge::None,
            v_merge: VerticalMerge::Continue,
            v_merge_continue: true,
        }
    }

    /// Check if this cell is merged (either start or continue)
    pub fn is_merged(&self) -> bool {
        self.h_merge != HorizontalMerge::None || self.v_merge != VerticalMerge::None
    }

    /// Check if this cell starts a merge
    pub fn is_merge_start(&self) -> bool {
        self.h_merge == HorizontalMerge::Start || self.v_merge == VerticalMerge::Start
    }

    /// Check if this cell is covered by another cell's merge
    pub fn is_covered(&self) -> bool {
        self.h_merge == HorizontalMerge::Continue || self.v_merge == VerticalMerge::Continue
    }

    /// Set horizontal merge state
    pub fn set_h_merge(&mut self, merge: HorizontalMerge) {
        self.h_merge = merge;
    }

    /// Set vertical merge state
    pub fn set_v_merge(&mut self, merge: VerticalMerge) {
        self.v_merge = merge;
        self.v_merge_continue = merge == VerticalMerge::Continue;
    }

    /// Add a child node ID
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

    /// Get effective grid span (at least 1)
    pub fn effective_grid_span(&self) -> u32 {
        self.grid_span.max(1)
    }

    /// Get effective row span (at least 1)
    pub fn effective_row_span(&self) -> u32 {
        self.row_span.max(1)
    }
}

impl Default for TableCell {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for TableCell {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::TableCell
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
// Row Properties
// =============================================================================

/// Properties for a table row
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RowProperties {
    /// Row height
    pub height: Option<f32>,
    /// How height is interpreted
    pub height_rule: HeightRule,
    /// Whether this is a header row (repeats on page breaks)
    pub is_header: bool,
    /// Whether to allow row to break across pages
    pub can_split: bool,
    /// Prevent row from splitting across pages (opposite of can_split for clarity)
    pub cant_split: bool,
    /// Keep with next row (don't allow page break after this row)
    pub keep_with_next: bool,
}

impl RowProperties {
    /// Create default row properties
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the height
    pub fn with_height(mut self, height: f32, rule: HeightRule) -> Self {
        self.height = Some(height);
        self.height_rule = rule;
        self
    }

    /// Mark as header row
    pub fn as_header(mut self) -> Self {
        self.is_header = true;
        self
    }

    /// Allow splitting across pages
    pub fn allow_split(mut self, can_split: bool) -> Self {
        self.can_split = can_split;
        self.cant_split = !can_split;
        self
    }

    /// Prevent splitting across pages
    pub fn prevent_split(mut self) -> Self {
        self.cant_split = true;
        self.can_split = false;
        self
    }

    /// Keep with next row
    pub fn keep_with_next(mut self) -> Self {
        self.keep_with_next = true;
        self
    }
}

// =============================================================================
// Table Row
// =============================================================================

/// A row in a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    id: NodeId,
    parent: Option<NodeId>,
    /// IDs of child cells
    cells: Vec<NodeId>,
    /// Row properties
    pub properties: RowProperties,
}

impl TableRow {
    /// Create a new empty row
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            cells: Vec::new(),
            properties: RowProperties::default(),
        }
    }

    /// Create a row with properties
    pub fn with_properties(properties: RowProperties) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            cells: Vec::new(),
            properties,
        }
    }

    /// Create a header row
    pub fn header() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            cells: Vec::new(),
            properties: RowProperties::new().as_header(),
        }
    }

    /// Add a cell ID
    pub fn add_cell(&mut self, cell_id: NodeId) {
        self.cells.push(cell_id);
    }

    /// Insert a cell at a specific index
    pub fn insert_cell(&mut self, index: usize, cell_id: NodeId) {
        self.cells.insert(index, cell_id);
    }

    /// Remove a cell by ID
    pub fn remove_cell(&mut self, cell_id: NodeId) -> bool {
        if let Some(pos) = self.cells.iter().position(|&id| id == cell_id) {
            self.cells.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the number of cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

impl Default for TableRow {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for TableRow {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::TableRow
    }

    fn children(&self) -> &[NodeId] {
        &self.cells
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
// Table Properties
// =============================================================================

/// Properties for a table
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TableProperties {
    /// Table width
    pub width: Option<TableWidth>,
    /// Table alignment (within page)
    pub alignment: Option<TableAlignment>,
    /// Table borders
    pub borders: Option<TableBorders>,
    /// Default cell padding (applies to all cells unless overridden)
    pub default_cell_padding: Option<CellPadding>,
    /// Cell spacing (space between cells)
    pub cell_spacing: Option<f32>,
    /// Table style ID reference
    pub style_id: Option<StyleId>,
    /// Left indent from margin
    pub indent_left: Option<f32>,
    /// Auto-fit mode
    pub auto_fit_mode: TableAutoFitMode,
    /// Preferred table layout algorithm (auto vs fixed)
    pub table_layout: TableLayoutMode,
    /// Allow table to overlap other content
    pub allow_overlap: bool,
}

/// Table alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TableAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Table layout algorithm mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TableLayoutMode {
    /// Auto layout - columns adjust based on content
    #[default]
    Auto,
    /// Fixed layout - column widths are fixed
    Fixed,
}

impl TableProperties {
    /// Create default table properties
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the table width
    pub fn with_width(mut self, width: TableWidth) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the alignment
    pub fn with_alignment(mut self, alignment: TableAlignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Set the borders
    pub fn with_borders(mut self, borders: TableBorders) -> Self {
        self.borders = Some(borders);
        self
    }

    /// Set default cell padding
    pub fn with_cell_padding(mut self, padding: CellPadding) -> Self {
        self.default_cell_padding = Some(padding);
        self
    }

    /// Set auto-fit mode
    pub fn with_auto_fit(mut self, mode: TableAutoFitMode) -> Self {
        self.auto_fit_mode = mode;
        self
    }

    /// Set table layout mode
    pub fn with_layout_mode(mut self, mode: TableLayoutMode) -> Self {
        self.table_layout = mode;
        self
    }
}

// =============================================================================
// Table
// =============================================================================

/// Maximum nesting depth for tables
pub const MAX_TABLE_NESTING_DEPTH: usize = 10;

/// A table containing rows and cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    id: NodeId,
    parent: Option<NodeId>,
    /// IDs of rows
    rows: Vec<NodeId>,
    /// Grid definition (column widths)
    pub grid: TableGrid,
    /// Table properties
    pub properties: TableProperties,
    /// Nesting depth (0 = top-level table, 1 = nested in cell, etc.)
    pub nesting_depth: usize,
}

impl Table {
    /// Create a new empty table
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            rows: Vec::new(),
            grid: TableGrid::default(),
            properties: TableProperties::default(),
            nesting_depth: 0,
        }
    }

    /// Create a table with specified grid
    pub fn with_grid(grid: TableGrid) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            rows: Vec::new(),
            grid,
            properties: TableProperties::default(),
            nesting_depth: 0,
        }
    }

    /// Create a table with grid and properties
    pub fn with_grid_and_properties(grid: TableGrid, properties: TableProperties) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            rows: Vec::new(),
            grid,
            properties,
            nesting_depth: 0,
        }
    }

    /// Create a simple table structure
    pub fn simple(rows: usize, cols: usize, col_width: f32) -> Self {
        let grid = TableGrid::with_equal_columns(cols, col_width * cols as f32);
        let properties = TableProperties::new()
            .with_borders(TableBorders::default_borders())
            .with_width(TableWidth::fixed(col_width * cols as f32));

        Self {
            id: NodeId::new(),
            parent: None,
            rows: Vec::new(),
            grid,
            properties,
            nesting_depth: 0,
        }
    }

    /// Create a nested table with specified depth
    pub fn nested(grid: TableGrid, nesting_depth: usize) -> Option<Self> {
        if nesting_depth >= MAX_TABLE_NESTING_DEPTH {
            return None;
        }
        Some(Self {
            id: NodeId::new(),
            parent: None,
            rows: Vec::new(),
            grid,
            properties: TableProperties::default(),
            nesting_depth,
        })
    }

    /// Check if this table can contain nested tables
    pub fn can_nest_tables(&self) -> bool {
        self.nesting_depth + 1 < MAX_TABLE_NESTING_DEPTH
    }

    /// Get the nesting depth
    pub fn nesting_depth(&self) -> usize {
        self.nesting_depth
    }

    /// Set the nesting depth
    pub fn set_nesting_depth(&mut self, depth: usize) {
        self.nesting_depth = depth.min(MAX_TABLE_NESTING_DEPTH);
    }

    /// Add a row ID
    pub fn add_row(&mut self, row_id: NodeId) {
        self.rows.push(row_id);
    }

    /// Insert a row at a specific index
    pub fn insert_row(&mut self, index: usize, row_id: NodeId) {
        if index <= self.rows.len() {
            self.rows.insert(index, row_id);
        }
    }

    /// Remove a row by ID
    pub fn remove_row(&mut self, row_id: NodeId) -> bool {
        if let Some(pos) = self.rows.iter().position(|&id| id == row_id) {
            self.rows.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns (from grid)
    pub fn column_count(&self) -> usize {
        self.grid.column_count()
    }

    /// Get header rows (rows marked as headers)
    pub fn header_row_ids(&self) -> Vec<NodeId> {
        // Note: This just returns the row IDs; actual header checking requires
        // looking up the row properties in DocumentTree
        self.rows.clone()
    }

    /// Get the row at a specific index
    pub fn row_at(&self, index: usize) -> Option<NodeId> {
        self.rows.get(index).copied()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Merged Cell Info
// =============================================================================

/// Information about a merged cell region
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergedCellRegion {
    /// The ID of the starting (anchor) cell
    pub anchor_cell_id: NodeId,
    /// Starting row index
    pub start_row: usize,
    /// Starting column index
    pub start_col: usize,
    /// Ending row index (inclusive)
    pub end_row: usize,
    /// Ending column index (inclusive)
    pub end_col: usize,
}

impl MergedCellRegion {
    /// Create a new merged cell region
    pub fn new(
        anchor_cell_id: NodeId,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> Self {
        Self {
            anchor_cell_id,
            start_row: start_row.min(end_row),
            start_col: start_col.min(end_col),
            end_row: start_row.max(end_row),
            end_col: start_col.max(end_col),
        }
    }

    /// Get the number of rows spanned
    pub fn row_span(&self) -> usize {
        self.end_row - self.start_row + 1
    }

    /// Get the number of columns spanned
    pub fn col_span(&self) -> usize {
        self.end_col - self.start_col + 1
    }

    /// Check if a cell position is within this merged region
    pub fn contains(&self, row: usize, col: usize) -> bool {
        row >= self.start_row && row <= self.end_row && col >= self.start_col && col <= self.end_col
    }

    /// Check if a cell position is the anchor (starting) cell
    pub fn is_anchor(&self, row: usize, col: usize) -> bool {
        row == self.start_row && col == self.start_col
    }
}

impl Node for Table {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Table
    }

    fn children(&self) -> &[NodeId] {
        &self.rows
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
// Table Selection
// =============================================================================

/// Represents a selection within a table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableSelection {
    /// Selection is within cell content (normal text selection)
    CellContent {
        cell_id: NodeId,
    },
    /// Single cell selected
    SingleCell {
        cell_id: NodeId,
    },
    /// Multiple cells selected (rectangular region)
    CellRange {
        table_id: NodeId,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    },
    /// Entire row(s) selected
    Rows {
        table_id: NodeId,
        start_row: usize,
        end_row: usize,
    },
    /// Entire column(s) selected
    Columns {
        table_id: NodeId,
        start_col: usize,
        end_col: usize,
    },
    /// Entire table selected
    WholeTable {
        table_id: NodeId,
    },
}

impl TableSelection {
    /// Create a cell content selection
    pub fn cell_content(cell_id: NodeId) -> Self {
        Self::CellContent { cell_id }
    }

    /// Create a single cell selection
    pub fn single_cell(cell_id: NodeId) -> Self {
        Self::SingleCell { cell_id }
    }

    /// Create a cell range selection
    pub fn cell_range(
        table_id: NodeId,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> Self {
        Self::CellRange {
            table_id,
            start_row: start_row.min(end_row),
            start_col: start_col.min(end_col),
            end_row: start_row.max(end_row),
            end_col: start_col.max(end_col),
        }
    }

    /// Create a row selection
    pub fn rows(table_id: NodeId, start_row: usize, end_row: usize) -> Self {
        Self::Rows {
            table_id,
            start_row: start_row.min(end_row),
            end_row: start_row.max(end_row),
        }
    }

    /// Create a column selection
    pub fn columns(table_id: NodeId, start_col: usize, end_col: usize) -> Self {
        Self::Columns {
            table_id,
            start_col: start_col.min(end_col),
            end_col: start_col.max(end_col),
        }
    }

    /// Create a whole table selection
    pub fn whole_table(table_id: NodeId) -> Self {
        Self::WholeTable { table_id }
    }

    /// Check if this is a multi-cell selection
    pub fn is_multi_cell(&self) -> bool {
        !matches!(self, Self::CellContent { .. } | Self::SingleCell { .. })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let table = Table::simple(3, 4, 100.0);
        assert_eq!(table.column_count(), 4);
        assert_eq!(table.row_count(), 0); // Rows need to be added separately
    }

    #[test]
    fn test_grid_column_operations() {
        let mut grid = TableGrid::new(3);
        assert_eq!(grid.column_count(), 3);

        grid.add_column(GridColumn::fixed(100.0));
        assert_eq!(grid.column_count(), 4);

        grid.insert_column(1, GridColumn::fixed(50.0));
        assert_eq!(grid.column_count(), 5);

        grid.remove_column(0);
        assert_eq!(grid.column_count(), 4);
    }

    #[test]
    fn test_cell_spans() {
        let mut cell = TableCell::new();
        assert_eq!(cell.effective_grid_span(), 1);
        assert_eq!(cell.effective_row_span(), 1);

        cell.grid_span = 3;
        assert_eq!(cell.effective_grid_span(), 3);

        cell.grid_span = 0; // Invalid but handled
        assert_eq!(cell.effective_grid_span(), 1);
    }

    #[test]
    fn test_table_width_types() {
        let fixed = TableWidth::fixed(200.0);
        assert_eq!(fixed.width_type, WidthType::Fixed);
        assert_eq!(fixed.value, 200.0);

        let auto = TableWidth::auto();
        assert_eq!(auto.width_type, WidthType::Auto);

        let percent = TableWidth::percent(50.0);
        assert_eq!(percent.width_type, WidthType::Percent);
        assert_eq!(percent.value, 50.0);
    }

    #[test]
    fn test_cell_properties() {
        let props = CellProperties::new()
            .with_shading("#EEEEEE")
            .with_vertical_align(CellVerticalAlign::Center)
            .with_padding(CellPadding::uniform(5.0));

        assert_eq!(props.shading, Some("#EEEEEE".to_string()));
        assert_eq!(props.vertical_align, Some(CellVerticalAlign::Center));
        assert!(props.padding.is_some());
    }

    #[test]
    fn test_table_selection() {
        let table_id = NodeId::new();

        let range = TableSelection::cell_range(table_id, 2, 1, 0, 3);
        if let TableSelection::CellRange { start_row, end_row, start_col, end_col, .. } = range {
            // Should normalize to min/max
            assert_eq!(start_row, 0);
            assert_eq!(end_row, 2);
            assert_eq!(start_col, 1);
            assert_eq!(end_col, 3);
        } else {
            panic!("Expected CellRange");
        }
    }

    #[test]
    fn test_row_properties() {
        let props = RowProperties::new()
            .with_height(24.0, HeightRule::Exact)
            .as_header();

        assert_eq!(props.height, Some(24.0));
        assert_eq!(props.height_rule, HeightRule::Exact);
        assert!(props.is_header);
    }

    #[test]
    fn test_cell_merge_states() {
        // Test horizontal merge
        let h_start = TableCell::spanning_columns(3);
        assert_eq!(h_start.h_merge, HorizontalMerge::Start);
        assert_eq!(h_start.grid_span, 3);
        assert!(h_start.is_merge_start());
        assert!(!h_start.is_covered());

        let h_continue = TableCell::h_merge_continue();
        assert_eq!(h_continue.h_merge, HorizontalMerge::Continue);
        assert!(h_continue.is_covered());

        // Test vertical merge
        let v_start = TableCell::spanning_rows(2);
        assert_eq!(v_start.v_merge, VerticalMerge::Start);
        assert_eq!(v_start.row_span, 2);
        assert!(v_start.is_merge_start());

        let v_continue = TableCell::v_merge_continue_cell();
        assert_eq!(v_continue.v_merge, VerticalMerge::Continue);
        assert!(v_continue.is_covered());

        // Test combined merge
        let combined = TableCell::spanning(2, 3);
        assert_eq!(combined.grid_span, 2);
        assert_eq!(combined.row_span, 3);
        assert_eq!(combined.h_merge, HorizontalMerge::Start);
        assert_eq!(combined.v_merge, VerticalMerge::Start);
    }

    #[test]
    fn test_merged_cell_region() {
        let cell_id = NodeId::new();
        let region = MergedCellRegion::new(cell_id, 1, 2, 3, 4);

        assert_eq!(region.row_span(), 3);
        assert_eq!(region.col_span(), 3);
        assert!(region.contains(2, 3));
        assert!(!region.contains(0, 0));
        assert!(region.is_anchor(1, 2));
        assert!(!region.is_anchor(2, 3));
    }

    #[test]
    fn test_text_direction() {
        let props = CellProperties::new()
            .with_text_direction(CellTextDirection::Rtl);
        assert_eq!(props.text_direction, Some(CellTextDirection::Rtl));
    }

    #[test]
    fn test_row_cant_split() {
        let props = RowProperties::new().prevent_split();
        assert!(props.cant_split);
        assert!(!props.can_split);
    }

    #[test]
    fn test_table_auto_fit_modes() {
        let props = TableProperties::new()
            .with_auto_fit(TableAutoFitMode::AutoFitWindow);
        assert_eq!(props.auto_fit_mode, TableAutoFitMode::AutoFitWindow);

        let props2 = TableProperties::new()
            .with_layout_mode(TableLayoutMode::Fixed);
        assert_eq!(props2.table_layout, TableLayoutMode::Fixed);
    }

    #[test]
    fn test_nested_table_depth() {
        let table = Table::new();
        assert_eq!(table.nesting_depth(), 0);
        assert!(table.can_nest_tables());

        let nested = Table::nested(TableGrid::new(2), 5).unwrap();
        assert_eq!(nested.nesting_depth(), 5);
        assert!(nested.can_nest_tables());

        // Max depth should fail
        let too_deep = Table::nested(TableGrid::new(2), MAX_TABLE_NESTING_DEPTH);
        assert!(too_deep.is_none());
    }
}
