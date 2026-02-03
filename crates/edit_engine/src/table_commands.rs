//! Table editing commands
//!
//! This module implements commands for creating and modifying tables:
//! - InsertTable: Create a new table with specified rows and columns
//! - InsertRowAbove/InsertRowBelow: Add rows
//! - InsertColumnLeft/InsertColumnRight: Add columns
//! - DeleteRow/DeleteColumn/DeleteTable: Remove table elements
//! - SetCellBorders/SetCellShading: Format cells
//! - MergeCells/SplitCell: Cell merging (horizontal and vertical)
//! - SetHeaderRow: Mark rows as headers
//! - SetRowProperties: Configure row breaking behavior
//! - SetCellProperties: Configure cell properties including vertical alignment and text direction
//! - InsertNestedTable: Insert table within a cell
//! - SetTableAutoFit: Configure auto-fit mode

use crate::{Command, CommandResult, Result};
use doc_model::{
    CellBorders, CellPadding, CellProperties, CellVerticalAlign, CellTextDirection,
    DocumentTree, GridColumn, HorizontalMerge, Node, NodeId, Paragraph, Position, Selection,
    Table, TableAutoFitMode, TableBorders, TableCell, TableGrid, TableProperties, TableRow,
    TableWidth, VerticalMerge, MAX_TABLE_NESTING_DEPTH,
};
use serde::{Deserialize, Serialize};

// =============================================================================
// InsertTable Command
// =============================================================================

/// Insert a new table at the current position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTable {
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
    /// Total table width in points (None = auto)
    pub width: Option<f32>,
    /// Insert position (body child index)
    pub insert_index: Option<usize>,
    /// ID of created table (set after apply, used for undo)
    #[serde(skip)]
    created_table_id: Option<NodeId>,
}

impl InsertTable {
    /// Create a new InsertTable command
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows: rows.max(1),
            cols: cols.max(1),
            width: None,
            insert_index: None,
            created_table_id: None,
        }
    }

    /// Set the table width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the insert index
    pub fn at_index(mut self, index: usize) -> Self {
        self.insert_index = Some(index);
        self
    }
}

impl Command for InsertTable {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Calculate column width
        let total_width = self.width.unwrap_or(468.0); // Default 6.5 inches
        let col_width = total_width / self.cols as f32;

        // Create the table grid
        let grid = TableGrid::with_equal_columns(self.cols, total_width);

        // Create table properties
        let properties = TableProperties::new()
            .with_width(TableWidth::fixed(total_width))
            .with_borders(TableBorders::default_borders())
            .with_cell_padding(CellPadding::default());

        // Create the table
        let mut table = Table::with_grid_and_properties(grid, properties);
        let table_id = table.id();

        // Insert the table
        new_tree.insert_table(table, self.insert_index)?;

        // Create rows and cells
        for _row_idx in 0..self.rows {
            let row = TableRow::new();
            let row_id = new_tree.insert_table_row(row, table_id, None)?;

            for _col_idx in 0..self.cols {
                let cell = TableCell::with_properties(
                    CellProperties::new().with_borders(CellBorders::default_borders()),
                );
                let cell_id = new_tree.insert_table_cell(cell, row_id, None)?;

                // Add an empty paragraph to each cell
                let para = Paragraph::new();
                new_tree.insert_paragraph_into_cell(para, cell_id, None)?;
            }
        }

        // Set selection to the first cell's paragraph
        let new_selection = if let Some(first_row_id) = new_tree.get_table(table_id)
            .and_then(|t| t.children().first().copied())
        {
            if let Some(first_cell_id) = new_tree.get_table_row(first_row_id)
                .and_then(|r| r.children().first().copied())
            {
                if let Some(first_para_id) = new_tree.get_table_cell(first_cell_id)
                    .and_then(|c| c.children().first().copied())
                {
                    Selection::collapsed(Position::new(first_para_id, 0))
                } else {
                    *selection
                }
            } else {
                *selection
            }
        } else {
            *selection
        };

        // Create inverse command (delete the table)
        let inverse = Box::new(DeleteTable { table_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteTable {
            table_id: self.created_table_id.unwrap_or_else(NodeId::new),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Table"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// DeleteTable Command
// =============================================================================

/// Delete an entire table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTable {
    pub table_id: NodeId,
}

impl DeleteTable {
    pub fn new(table_id: NodeId) -> Self {
        Self { table_id }
    }
}

impl Command for DeleteTable {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store table info for undo
        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?
            .clone();

        // Find the insert index for undo
        let insert_index = new_tree.document.children()
            .iter()
            .position(|&id| id == self.table_id);

        // Remove the table
        new_tree.remove_table(self.table_id)?;

        // Update selection to be after the deleted table position
        // (For simplicity, just keep current selection or move to first paragraph)
        let new_selection = if let Some(first_para) = new_tree.paragraphs().next() {
            Selection::collapsed(Position::new(first_para.id(), 0))
        } else {
            *selection
        };

        // Create inverse - would need to recreate the entire table structure
        // For now, use a placeholder
        let inverse = Box::new(InsertTable {
            rows: table.row_count(),
            cols: table.column_count(),
            width: table.properties.width.map(|w| w.value),
            insert_index,
            created_table_id: Some(self.table_id),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(table) = tree.get_table(self.table_id) {
            let insert_index = tree.document.children()
                .iter()
                .position(|&id| id == self.table_id);

            Box::new(InsertTable {
                rows: table.row_count(),
                cols: table.column_count(),
                width: table.properties.width.map(|w| w.value),
                insert_index,
                created_table_id: Some(self.table_id),
            })
        } else {
            Box::new(InsertTable::new(1, 1))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Table"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertRow Command
// =============================================================================

/// Insert a row above or below the current row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRow {
    /// The table to insert into
    pub table_id: NodeId,
    /// Insert after this row index (None = at end)
    pub after_row: Option<usize>,
    /// Whether to insert above (false) or below (true) the reference row
    pub below: bool,
    /// ID of created row (for undo)
    #[serde(skip)]
    created_row_id: Option<NodeId>,
}

impl InsertRow {
    pub fn above(table_id: NodeId, row_index: usize) -> Self {
        Self {
            table_id,
            after_row: Some(row_index),
            below: false,
            created_row_id: None,
        }
    }

    pub fn below(table_id: NodeId, row_index: usize) -> Self {
        Self {
            table_id,
            after_row: Some(row_index),
            below: true,
            created_row_id: None,
        }
    }

    pub fn at_end(table_id: NodeId) -> Self {
        Self {
            table_id,
            after_row: None,
            below: true,
            created_row_id: None,
        }
    }
}

impl Command for InsertRow {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let col_count = table.column_count();
        let insert_index = match self.after_row {
            Some(idx) if self.below => Some(idx + 1),
            Some(idx) => Some(idx),
            None => None,
        };

        // Create the new row
        let row = TableRow::new();
        let row_id = new_tree.insert_table_row(row, self.table_id, insert_index)?;

        // Add cells to the row
        for _ in 0..col_count {
            let cell = TableCell::with_properties(
                CellProperties::new().with_borders(CellBorders::default_borders()),
            );
            let cell_id = new_tree.insert_table_cell(cell, row_id, None)?;

            // Add an empty paragraph to each cell
            let para = Paragraph::new();
            new_tree.insert_paragraph_into_cell(para, cell_id, None)?;
        }

        // Set selection to first cell of new row
        let new_selection = if let Some(row) = new_tree.get_table_row(row_id) {
            if let Some(&first_cell_id) = row.children().first() {
                if let Some(cell) = new_tree.get_table_cell(first_cell_id) {
                    if let Some(&first_para_id) = cell.children().first() {
                        Selection::collapsed(Position::new(first_para_id, 0))
                    } else {
                        *selection
                    }
                } else {
                    *selection
                }
            } else {
                *selection
            }
        } else {
            *selection
        };

        let inverse = Box::new(DeleteRow {
            table_id: self.table_id,
            row_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteRow {
            table_id: self.table_id,
            row_id: self.created_row_id.unwrap_or_else(NodeId::new),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        if self.below { "Insert Row Below" } else { "Insert Row Above" }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// DeleteRow Command
// =============================================================================

/// Delete a row from a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRow {
    pub table_id: NodeId,
    pub row_id: NodeId,
}

impl DeleteRow {
    pub fn new(table_id: NodeId, row_id: NodeId) -> Self {
        Self { table_id, row_id }
    }
}

impl Command for DeleteRow {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store row index for undo
        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let row_index = table.children().iter().position(|&id| id == self.row_id);

        // Remove the row
        new_tree.remove_table_row(self.row_id)?;

        // Keep selection or move to another row
        let new_selection = *selection;

        let inverse = Box::new(InsertRow {
            table_id: self.table_id,
            after_row: row_index.map(|i| if i > 0 { i - 1 } else { 0 }),
            below: row_index.map(|i| i > 0).unwrap_or(false),
            created_row_id: Some(self.row_id),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let row_index = tree.get_table(self.table_id)
            .and_then(|t| t.children().iter().position(|&id| id == self.row_id));

        Box::new(InsertRow {
            table_id: self.table_id,
            after_row: row_index.map(|i| if i > 0 { i - 1 } else { 0 }),
            below: row_index.map(|i| i > 0).unwrap_or(false),
            created_row_id: Some(self.row_id),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Row"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertColumn Command
// =============================================================================

/// Insert a column at a specified position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertColumn {
    pub table_id: NodeId,
    /// Column index to insert at/after
    pub column_index: usize,
    /// Insert to the right (true) or left (false)
    pub to_right: bool,
    /// Column width (None = auto)
    pub width: Option<f32>,
}

impl InsertColumn {
    pub fn left(table_id: NodeId, column_index: usize) -> Self {
        Self {
            table_id,
            column_index,
            to_right: false,
            width: None,
        }
    }

    pub fn right(table_id: NodeId, column_index: usize) -> Self {
        Self {
            table_id,
            column_index,
            to_right: true,
            width: None,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Command for InsertColumn {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let insert_col = if self.to_right {
            self.column_index + 1
        } else {
            self.column_index
        };

        // Add column to grid
        {
            let table = new_tree.get_table_mut(self.table_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Table not found: {:?}", self.table_id)
                ))?;

            let col = match self.width {
                Some(w) => GridColumn::fixed(w),
                None => GridColumn::auto(),
            };
            table.grid.insert_column(insert_col, col);
        }

        // Add cell to each row
        let table = new_tree.get_table(self.table_id).unwrap();
        let row_ids: Vec<NodeId> = table.children().to_vec();

        for row_id in row_ids {
            let cell = TableCell::with_properties(
                CellProperties::new().with_borders(CellBorders::default_borders()),
            );
            let cell_id = new_tree.insert_table_cell(cell, row_id, Some(insert_col))?;

            let para = Paragraph::new();
            new_tree.insert_paragraph_into_cell(para, cell_id, None)?;
        }

        let inverse = Box::new(DeleteColumn {
            table_id: self.table_id,
            column_index: insert_col,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        let col = if self.to_right {
            self.column_index + 1
        } else {
            self.column_index
        };
        Box::new(DeleteColumn {
            table_id: self.table_id,
            column_index: col,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        if self.to_right { "Insert Column Right" } else { "Insert Column Left" }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// DeleteColumn Command
// =============================================================================

/// Delete a column from a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteColumn {
    pub table_id: NodeId,
    pub column_index: usize,
}

impl DeleteColumn {
    pub fn new(table_id: NodeId, column_index: usize) -> Self {
        Self { table_id, column_index }
    }
}

impl Command for DeleteColumn {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Remove column from grid
        {
            let table = new_tree.get_table_mut(self.table_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Table not found: {:?}", self.table_id)
                ))?;

            table.grid.remove_column(self.column_index);
        }

        // Remove cell from each row
        let table = new_tree.get_table(self.table_id).unwrap();
        let row_ids: Vec<NodeId> = table.children().to_vec();

        for row_id in row_ids {
            if let Some(row) = new_tree.get_table_row(row_id) {
                if let Some(&cell_id) = row.children().get(self.column_index) {
                    new_tree.remove_table_cell(cell_id)?;
                }
            }
        }

        let inverse = Box::new(InsertColumn {
            table_id: self.table_id,
            column_index: self.column_index,
            to_right: false,
            width: None,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(InsertColumn {
            table_id: self.table_id,
            column_index: self.column_index,
            to_right: false,
            width: None,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Column"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetCellBorders Command
// =============================================================================

/// Set borders on one or more cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCellBorders {
    pub cell_ids: Vec<NodeId>,
    pub borders: CellBorders,
    /// Previous borders (for undo)
    #[serde(skip)]
    previous_borders: Vec<Option<CellBorders>>,
}

impl SetCellBorders {
    pub fn new(cell_ids: Vec<NodeId>, borders: CellBorders) -> Self {
        Self {
            cell_ids,
            borders,
            previous_borders: Vec::new(),
        }
    }

    pub fn single(cell_id: NodeId, borders: CellBorders) -> Self {
        Self::new(vec![cell_id], borders)
    }
}

impl Command for SetCellBorders {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let mut previous = Vec::new();

        for &cell_id in &self.cell_ids {
            let cell = new_tree.get_table_cell_mut(cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Cell not found: {:?}", cell_id)
                ))?;

            previous.push(cell.properties.borders.clone());
            cell.properties.borders = Some(self.borders.clone());
        }

        let inverse = Box::new(SetCellBorders {
            cell_ids: self.cell_ids.clone(),
            borders: CellBorders::default(),
            previous_borders: previous,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let mut previous = Vec::new();
        for &cell_id in &self.cell_ids {
            if let Some(cell) = tree.get_table_cell(cell_id) {
                previous.push(cell.properties.borders.clone());
            } else {
                previous.push(None);
            }
        }

        Box::new(SetCellBorders {
            cell_ids: self.cell_ids.clone(),
            borders: CellBorders::default(),
            previous_borders: previous,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Cell Borders"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetCellShading Command
// =============================================================================

/// Set background shading on one or more cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCellShading {
    pub cell_ids: Vec<NodeId>,
    /// Shading color (CSS color string, None to remove)
    pub color: Option<String>,
    /// Previous colors (for undo)
    #[serde(skip)]
    previous_colors: Vec<Option<String>>,
}

impl SetCellShading {
    pub fn new(cell_ids: Vec<NodeId>, color: Option<String>) -> Self {
        Self {
            cell_ids,
            color,
            previous_colors: Vec::new(),
        }
    }

    pub fn single(cell_id: NodeId, color: &str) -> Self {
        Self::new(vec![cell_id], Some(color.to_string()))
    }

    pub fn remove(cell_ids: Vec<NodeId>) -> Self {
        Self::new(cell_ids, None)
    }
}

impl Command for SetCellShading {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let mut previous = Vec::new();

        for &cell_id in &self.cell_ids {
            let cell = new_tree.get_table_cell_mut(cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Cell not found: {:?}", cell_id)
                ))?;

            previous.push(cell.properties.shading.clone());
            cell.properties.shading = self.color.clone();
        }

        let inverse = Box::new(SetCellShading {
            cell_ids: self.cell_ids.clone(),
            color: None,
            previous_colors: previous,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let mut previous = Vec::new();
        for &cell_id in &self.cell_ids {
            if let Some(cell) = tree.get_table_cell(cell_id) {
                previous.push(cell.properties.shading.clone());
            } else {
                previous.push(None);
            }
        }

        Box::new(SetCellShading {
            cell_ids: self.cell_ids.clone(),
            color: None,
            previous_colors: previous,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Cell Shading"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// MergeCells Command (Basic)
// =============================================================================

/// Merge multiple cells into one (basic horizontal merge only for Phase 1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCells {
    pub table_id: NodeId,
    pub row_index: usize,
    pub start_col: usize,
    pub end_col: usize,
}

impl MergeCells {
    pub fn new(table_id: NodeId, row_index: usize, start_col: usize, end_col: usize) -> Self {
        Self {
            table_id,
            row_index,
            start_col: start_col.min(end_col),
            end_col: start_col.max(end_col),
        }
    }
}

impl Command for MergeCells {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the row
        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let row_id = *table.children().get(self.row_index)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Row {} not found", self.row_index)
            ))?;

        let row = new_tree.get_table_row(row_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Row not found: {:?}", row_id)
            ))?;

        // Get cells to merge
        let cell_ids: Vec<NodeId> = row.children()[self.start_col..=self.end_col].to_vec();
        if cell_ids.len() < 2 {
            return Err(crate::EditError::InvalidCommand(
                "Need at least 2 cells to merge".to_string()
            ));
        }

        // Set grid_span on first cell
        let first_cell_id = cell_ids[0];
        {
            let first_cell = new_tree.get_table_cell_mut(first_cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand("First cell not found".into()))?;
            first_cell.grid_span = (self.end_col - self.start_col + 1) as u32;
        }

        // Remove other cells (keep first one)
        for &cell_id in &cell_ids[1..] {
            new_tree.remove_table_cell(cell_id)?;
        }

        let inverse = Box::new(SplitCell {
            table_id: self.table_id,
            row_index: self.row_index,
            col_index: self.start_col,
            split_count: cell_ids.len(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SplitCell {
            table_id: self.table_id,
            row_index: self.row_index,
            col_index: self.start_col,
            split_count: self.end_col - self.start_col + 1,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Merge Cells"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SplitCell Command (Basic)
// =============================================================================

/// Split a merged cell back into individual cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitCell {
    pub table_id: NodeId,
    pub row_index: usize,
    pub col_index: usize,
    pub split_count: usize,
}

impl SplitCell {
    pub fn new(table_id: NodeId, row_index: usize, col_index: usize, split_count: usize) -> Self {
        Self {
            table_id,
            row_index,
            col_index,
            split_count: split_count.max(2),
        }
    }
}

impl Command for SplitCell {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the row
        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let row_id = *table.children().get(self.row_index)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Row {} not found", self.row_index)
            ))?;

        // Reset grid_span on the cell
        let row = new_tree.get_table_row(row_id).unwrap();
        let cell_id = *row.children().get(self.col_index)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Cell at column {} not found", self.col_index)
            ))?;

        {
            let cell = new_tree.get_table_cell_mut(cell_id).unwrap();
            cell.grid_span = 1;
        }

        // Add new cells
        for i in 1..self.split_count {
            let cell = TableCell::with_properties(
                CellProperties::new().with_borders(CellBorders::default_borders()),
            );
            let new_cell_id = new_tree.insert_table_cell(cell, row_id, Some(self.col_index + i))?;

            let para = Paragraph::new();
            new_tree.insert_paragraph_into_cell(para, new_cell_id, None)?;
        }

        let inverse = Box::new(MergeCells {
            table_id: self.table_id,
            row_index: self.row_index,
            start_col: self.col_index,
            end_col: self.col_index + self.split_count - 1,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(MergeCells {
            table_id: self.table_id,
            row_index: self.row_index,
            start_col: self.col_index,
            end_col: self.col_index + self.split_count - 1,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Split Cell"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// MergeCellsVertical Command
// =============================================================================

/// Merge cells vertically (across rows)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCellsVertical {
    pub table_id: NodeId,
    pub start_row: usize,
    pub end_row: usize,
    pub col_index: usize,
}

impl MergeCellsVertical {
    pub fn new(table_id: NodeId, start_row: usize, end_row: usize, col_index: usize) -> Self {
        Self {
            table_id,
            start_row: start_row.min(end_row),
            end_row: start_row.max(end_row),
            col_index,
        }
    }
}

impl Command for MergeCellsVertical {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        // Get all rows in the merge range
        let row_ids: Vec<NodeId> = table.children()[self.start_row..=self.end_row].to_vec();
        if row_ids.len() < 2 {
            return Err(crate::EditError::InvalidCommand(
                "Need at least 2 rows to merge vertically".to_string()
            ));
        }

        // Set the first cell as merge start
        let first_row_id = row_ids[0];
        let first_row = new_tree.get_table_row(first_row_id)
            .ok_or_else(|| crate::EditError::InvalidCommand("First row not found".into()))?;

        let first_cell_id = *first_row.children().get(self.col_index)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Cell at column {} not found", self.col_index)
            ))?;

        {
            let first_cell = new_tree.get_table_cell_mut(first_cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand("First cell not found".into()))?;
            first_cell.row_span = (self.end_row - self.start_row + 1) as u32;
            first_cell.set_v_merge(VerticalMerge::Start);
        }

        // Set subsequent cells as merge continue
        for &row_id in &row_ids[1..] {
            let row = new_tree.get_table_row(row_id)
                .ok_or_else(|| crate::EditError::InvalidCommand("Row not found".into()))?;

            if let Some(&cell_id) = row.children().get(self.col_index) {
                let cell = new_tree.get_table_cell_mut(cell_id)
                    .ok_or_else(|| crate::EditError::InvalidCommand("Cell not found".into()))?;
                cell.set_v_merge(VerticalMerge::Continue);
            }
        }

        let inverse = Box::new(SplitCellVertical {
            table_id: self.table_id,
            start_row: self.start_row,
            col_index: self.col_index,
            split_count: row_ids.len(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SplitCellVertical {
            table_id: self.table_id,
            start_row: self.start_row,
            col_index: self.col_index,
            split_count: self.end_row - self.start_row + 1,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Merge Cells Vertically"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SplitCellVertical Command
// =============================================================================

/// Split a vertically merged cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitCellVertical {
    pub table_id: NodeId,
    pub start_row: usize,
    pub col_index: usize,
    pub split_count: usize,
}

impl SplitCellVertical {
    pub fn new(table_id: NodeId, start_row: usize, col_index: usize, split_count: usize) -> Self {
        Self {
            table_id,
            start_row,
            col_index,
            split_count: split_count.max(2),
        }
    }
}

impl Command for SplitCellVertical {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get row IDs from the table first
        let row_ids: Vec<NodeId> = {
            let table = new_tree.get_table(self.table_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Table not found: {:?}", self.table_id)
                ))?;
            table.children().to_vec()
        };

        // Get the first row and cell ID
        let first_row_id = *row_ids.get(self.start_row)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Row {} not found", self.start_row)
            ))?;

        let first_cell_id = {
            let first_row = new_tree.get_table_row(first_row_id)
                .ok_or_else(|| crate::EditError::InvalidCommand("Row not found".into()))?;
            *first_row.children().get(self.col_index)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Cell at column {} not found", self.col_index)
                ))?
        };

        // Reset the first cell
        {
            let first_cell = new_tree.get_table_cell_mut(first_cell_id).unwrap();
            first_cell.row_span = 1;
            first_cell.set_v_merge(VerticalMerge::None);
        }

        // Reset subsequent cells - collect cell IDs first
        let mut cell_ids_to_reset = Vec::new();
        for row_idx in (self.start_row + 1)..(self.start_row + self.split_count) {
            if let Some(&row_id) = row_ids.get(row_idx) {
                if let Some(row) = new_tree.get_table_row(row_id) {
                    if let Some(&cell_id) = row.children().get(self.col_index) {
                        cell_ids_to_reset.push(cell_id);
                    }
                }
            }
        }

        // Now reset the cells
        for cell_id in cell_ids_to_reset {
            if let Some(cell) = new_tree.get_table_cell_mut(cell_id) {
                cell.set_v_merge(VerticalMerge::None);
            }
        }

        let inverse = Box::new(MergeCellsVertical {
            table_id: self.table_id,
            start_row: self.start_row,
            end_row: self.start_row + self.split_count - 1,
            col_index: self.col_index,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(MergeCellsVertical {
            table_id: self.table_id,
            start_row: self.start_row,
            end_row: self.start_row + self.split_count - 1,
            col_index: self.col_index,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Split Cell Vertically"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetHeaderRow Command
// =============================================================================

/// Mark or unmark a row as a header row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetHeaderRow {
    pub table_id: NodeId,
    pub row_index: usize,
    pub is_header: bool,
    #[serde(skip)]
    previous_is_header: Option<bool>,
}

impl SetHeaderRow {
    pub fn new(table_id: NodeId, row_index: usize, is_header: bool) -> Self {
        Self {
            table_id,
            row_index,
            is_header,
            previous_is_header: None,
        }
    }
}

impl Command for SetHeaderRow {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let row_id = *table.children().get(self.row_index)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Row {} not found", self.row_index)
            ))?;

        let row = new_tree.get_table_row_mut(row_id)
            .ok_or_else(|| crate::EditError::InvalidCommand("Row not found".into()))?;

        let previous = row.properties.is_header;
        row.properties.is_header = self.is_header;

        let inverse = Box::new(SetHeaderRow {
            table_id: self.table_id,
            row_index: self.row_index,
            is_header: previous,
            previous_is_header: Some(self.is_header),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let previous = if let Some(table) = tree.get_table(self.table_id) {
            if let Some(&row_id) = table.children().get(self.row_index) {
                tree.get_table_row(row_id).map(|r| r.properties.is_header).unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        Box::new(SetHeaderRow {
            table_id: self.table_id,
            row_index: self.row_index,
            is_header: previous,
            previous_is_header: Some(self.is_header),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        if self.is_header { "Set Header Row" } else { "Unset Header Row" }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetRowCanSplit Command
// =============================================================================

/// Set whether a row can split across pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRowCanSplit {
    pub table_id: NodeId,
    pub row_index: usize,
    pub can_split: bool,
}

impl SetRowCanSplit {
    pub fn new(table_id: NodeId, row_index: usize, can_split: bool) -> Self {
        Self {
            table_id,
            row_index,
            can_split,
        }
    }
}

impl Command for SetRowCanSplit {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let table = new_tree.get_table(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let row_id = *table.children().get(self.row_index)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Row {} not found", self.row_index)
            ))?;

        let row = new_tree.get_table_row_mut(row_id)
            .ok_or_else(|| crate::EditError::InvalidCommand("Row not found".into()))?;

        let previous = row.properties.can_split;
        row.properties.can_split = self.can_split;
        row.properties.cant_split = !self.can_split;

        let inverse = Box::new(SetRowCanSplit {
            table_id: self.table_id,
            row_index: self.row_index,
            can_split: previous,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let previous = if let Some(table) = tree.get_table(self.table_id) {
            if let Some(&row_id) = table.children().get(self.row_index) {
                tree.get_table_row(row_id).map(|r| r.properties.can_split).unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        Box::new(SetRowCanSplit {
            table_id: self.table_id,
            row_index: self.row_index,
            can_split: previous,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        if self.can_split { "Allow Row Break" } else { "Prevent Row Break" }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetCellVerticalAlign Command
// =============================================================================

/// Set vertical alignment for one or more cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCellVerticalAlign {
    pub cell_ids: Vec<NodeId>,
    pub alignment: CellVerticalAlign,
}

impl SetCellVerticalAlign {
    pub fn new(cell_ids: Vec<NodeId>, alignment: CellVerticalAlign) -> Self {
        Self { cell_ids, alignment }
    }

    pub fn single(cell_id: NodeId, alignment: CellVerticalAlign) -> Self {
        Self::new(vec![cell_id], alignment)
    }
}

impl Command for SetCellVerticalAlign {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let mut previous_alignments = Vec::new();

        for &cell_id in &self.cell_ids {
            let cell = new_tree.get_table_cell_mut(cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Cell not found: {:?}", cell_id)
                ))?;

            previous_alignments.push(cell.properties.vertical_align);
            cell.properties.vertical_align = Some(self.alignment);
        }

        let inverse = Box::new(SetCellVerticalAlign {
            cell_ids: self.cell_ids.clone(),
            alignment: CellVerticalAlign::Top, // Default for inverse
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetCellVerticalAlign {
            cell_ids: self.cell_ids.clone(),
            alignment: CellVerticalAlign::Top,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Cell Vertical Alignment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetCellTextDirection Command
// =============================================================================

/// Set text direction for one or more cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCellTextDirection {
    pub cell_ids: Vec<NodeId>,
    pub direction: CellTextDirection,
}

impl SetCellTextDirection {
    pub fn new(cell_ids: Vec<NodeId>, direction: CellTextDirection) -> Self {
        Self { cell_ids, direction }
    }

    pub fn single(cell_id: NodeId, direction: CellTextDirection) -> Self {
        Self::new(vec![cell_id], direction)
    }
}

impl Command for SetCellTextDirection {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        for &cell_id in &self.cell_ids {
            let cell = new_tree.get_table_cell_mut(cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Cell not found: {:?}", cell_id)
                ))?;

            cell.properties.text_direction = Some(self.direction);
        }

        let inverse = Box::new(SetCellTextDirection {
            cell_ids: self.cell_ids.clone(),
            direction: CellTextDirection::Ltr, // Default for inverse
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetCellTextDirection {
            cell_ids: self.cell_ids.clone(),
            direction: CellTextDirection::Ltr,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Cell Text Direction"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetCellPadding Command
// =============================================================================

/// Set padding for one or more cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCellPadding {
    pub cell_ids: Vec<NodeId>,
    pub padding: CellPadding,
}

impl SetCellPadding {
    pub fn new(cell_ids: Vec<NodeId>, padding: CellPadding) -> Self {
        Self { cell_ids, padding }
    }

    pub fn single(cell_id: NodeId, padding: CellPadding) -> Self {
        Self::new(vec![cell_id], padding)
    }
}

impl Command for SetCellPadding {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        for &cell_id in &self.cell_ids {
            let cell = new_tree.get_table_cell_mut(cell_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Cell not found: {:?}", cell_id)
                ))?;

            cell.properties.padding = Some(self.padding);
        }

        let inverse = Box::new(SetCellPadding {
            cell_ids: self.cell_ids.clone(),
            padding: CellPadding::default(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetCellPadding {
            cell_ids: self.cell_ids.clone(),
            padding: CellPadding::default(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Cell Padding"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// SetTableAutoFit Command
// =============================================================================

/// Set auto-fit mode for a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTableAutoFit {
    pub table_id: NodeId,
    pub mode: TableAutoFitMode,
}

impl SetTableAutoFit {
    pub fn new(table_id: NodeId, mode: TableAutoFitMode) -> Self {
        Self { table_id, mode }
    }
}

impl Command for SetTableAutoFit {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let table = new_tree.get_table_mut(self.table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Table not found: {:?}", self.table_id)
            ))?;

        let previous = table.properties.auto_fit_mode;
        table.properties.auto_fit_mode = self.mode;

        let inverse = Box::new(SetTableAutoFit {
            table_id: self.table_id,
            mode: previous,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let previous = tree.get_table(self.table_id)
            .map(|t| t.properties.auto_fit_mode)
            .unwrap_or_default();

        Box::new(SetTableAutoFit {
            table_id: self.table_id,
            mode: previous,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.mode {
            TableAutoFitMode::AutoFitContent => "Auto-fit Content",
            TableAutoFitMode::AutoFitWindow => "Auto-fit Window",
            TableAutoFitMode::FixedWidth => "Fixed Width",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// InsertNestedTable Command
// =============================================================================

/// Insert a nested table inside a cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertNestedTable {
    pub cell_id: NodeId,
    pub rows: usize,
    pub cols: usize,
    pub width: Option<f32>,
    #[serde(skip)]
    created_table_id: Option<NodeId>,
}

impl InsertNestedTable {
    pub fn new(cell_id: NodeId, rows: usize, cols: usize) -> Self {
        Self {
            cell_id,
            rows: rows.max(1),
            cols: cols.max(1),
            width: None,
            created_table_id: None,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Command for InsertNestedTable {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Verify the cell exists
        let cell = new_tree.get_table_cell(self.cell_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Cell not found: {:?}", self.cell_id)
            ))?;

        // Check nesting depth
        let parent_table_id = new_tree.find_table_for_node(self.cell_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                "Could not find parent table".to_string()
            ))?;

        let parent_table = new_tree.get_table(parent_table_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                "Parent table not found".to_string()
            ))?;

        let new_depth = parent_table.nesting_depth + 1;
        if new_depth >= MAX_TABLE_NESTING_DEPTH {
            return Err(crate::EditError::InvalidCommand(
                format!("Maximum nesting depth ({}) exceeded", MAX_TABLE_NESTING_DEPTH)
            ));
        }

        // Calculate width
        let total_width = self.width.unwrap_or(200.0);
        let col_width = total_width / self.cols as f32;

        // Create the nested table
        let grid = TableGrid::with_equal_columns(self.cols, total_width);
        let properties = TableProperties::new()
            .with_width(TableWidth::fixed(total_width))
            .with_borders(TableBorders::default_borders())
            .with_cell_padding(CellPadding::default());

        let mut table = Table::with_grid_and_properties(grid, properties);
        table.set_nesting_depth(new_depth);
        let table_id = table.id();

        // Insert the table into the cell (cells can contain tables like paragraphs)
        // Note: This requires the DocumentTree to support tables in cells
        // For now, we store the table in the nodes and add its ID to cell children
        new_tree.nodes.tables.insert(table_id, table);

        // Add table ID to cell's children
        if let Some(cell) = new_tree.get_table_cell_mut(self.cell_id) {
            cell.add_child(table_id);
        }

        // Create rows and cells for the nested table
        for _row_idx in 0..self.rows {
            let row = TableRow::new();
            let row_id = new_tree.insert_table_row(row, table_id, None)?;

            for _col_idx in 0..self.cols {
                let cell = TableCell::with_properties(
                    CellProperties::new().with_borders(CellBorders::default_borders()),
                );
                let cell_id = new_tree.insert_table_cell(cell, row_id, None)?;

                // Add an empty paragraph to each cell
                let para = Paragraph::new();
                new_tree.insert_paragraph_into_cell(para, cell_id, None)?;
            }
        }

        let inverse = Box::new(DeleteNestedTable {
            cell_id: self.cell_id,
            table_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteNestedTable {
            cell_id: self.cell_id,
            table_id: self.created_table_id.unwrap_or_else(NodeId::new),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Nested Table"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// DeleteNestedTable Command
// =============================================================================

/// Delete a nested table from a cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNestedTable {
    pub cell_id: NodeId,
    pub table_id: NodeId,
}

impl DeleteNestedTable {
    pub fn new(cell_id: NodeId, table_id: NodeId) -> Self {
        Self { cell_id, table_id }
    }
}

impl Command for DeleteNestedTable {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Remove table from cell's children
        if let Some(cell) = new_tree.get_table_cell_mut(self.cell_id) {
            cell.remove_child(self.table_id);
        }

        // Remove the table and all its contents
        new_tree.remove_table(self.table_id)?;

        let inverse = Box::new(InsertNestedTable {
            cell_id: self.cell_id,
            rows: 1,
            cols: 1,
            width: None,
            created_table_id: Some(self.table_id),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(InsertNestedTable {
            cell_id: self.cell_id,
            rows: 1,
            cols: 1,
            width: None,
            created_table_id: Some(self.table_id),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Nested Table"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_table() {
        let tree = DocumentTree::new();
        let selection = Selection::default();

        let cmd = InsertTable::new(3, 4).with_width(400.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Should have one table in body
        let tables: Vec<_> = result.tree.tables().collect();
        assert_eq!(tables.len(), 1);

        // Table should have 3 rows
        assert_eq!(tables[0].row_count(), 3);
        assert_eq!(tables[0].column_count(), 4);
    }

    #[test]
    fn test_insert_row() {
        let mut tree = DocumentTree::new();

        // Create a simple 2x2 table
        let grid = doc_model::TableGrid::with_equal_columns(2, 200.0);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        for _ in 0..2 {
            let row = TableRow::new();
            let row_id = tree.insert_table_row(row, table_id, None).unwrap();

            for _ in 0..2 {
                let cell = TableCell::new();
                let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
                let para = Paragraph::new();
                tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
            }
        }

        let selection = Selection::default();
        let cmd = InsertRow::below(table_id, 0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let table = result.tree.get_table(table_id).unwrap();
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn test_insert_column() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::with_equal_columns(2, 200.0);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        for _ in 0..2 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        let selection = Selection::default();
        let cmd = InsertColumn::right(table_id, 0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let table = result.tree.get_table(table_id).unwrap();
        assert_eq!(table.column_count(), 3);

        let row = result.tree.get_table_row(row_id).unwrap();
        assert_eq!(row.cell_count(), 3);
    }

    #[test]
    fn test_delete_table() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::with_equal_columns(2, 200.0);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let selection = Selection::default();
        let cmd = DeleteTable::new(table_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        let tables: Vec<_> = result.tree.tables().collect();
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_set_cell_shading() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::new(1);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        let cell = TableCell::new();
        let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();

        let selection = Selection::default();
        let cmd = SetCellShading::single(cell_id, "#FFFF00");
        let result = cmd.apply(&tree, &selection).unwrap();

        let cell = result.tree.get_table_cell(cell_id).unwrap();
        assert_eq!(cell.properties.shading, Some("#FFFF00".to_string()));
    }

    #[test]
    fn test_set_header_row() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::with_equal_columns(2, 200.0);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        for _ in 0..2 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        let selection = Selection::default();
        let cmd = SetHeaderRow::new(table_id, 0, true);
        let result = cmd.apply(&tree, &selection).unwrap();

        let row = result.tree.get_table_row(row_id).unwrap();
        assert!(row.properties.is_header);
    }

    #[test]
    fn test_set_row_can_split() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::with_equal_columns(2, 200.0);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        for _ in 0..2 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        let selection = Selection::default();
        let cmd = SetRowCanSplit::new(table_id, 0, false);
        let result = cmd.apply(&tree, &selection).unwrap();

        let row = result.tree.get_table_row(row_id).unwrap();
        assert!(!row.properties.can_split);
        assert!(row.properties.cant_split);
    }

    #[test]
    fn test_set_cell_vertical_align() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::new(1);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        let cell = TableCell::new();
        let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();

        let selection = Selection::default();
        let cmd = SetCellVerticalAlign::single(cell_id, CellVerticalAlign::Center);
        let result = cmd.apply(&tree, &selection).unwrap();

        let cell = result.tree.get_table_cell(cell_id).unwrap();
        assert_eq!(cell.properties.vertical_align, Some(CellVerticalAlign::Center));
    }

    #[test]
    fn test_set_cell_text_direction() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::new(1);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        let cell = TableCell::new();
        let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();

        let selection = Selection::default();
        let cmd = SetCellTextDirection::single(cell_id, CellTextDirection::Rtl);
        let result = cmd.apply(&tree, &selection).unwrap();

        let cell = result.tree.get_table_cell(cell_id).unwrap();
        assert_eq!(cell.properties.text_direction, Some(CellTextDirection::Rtl));
    }

    #[test]
    fn test_set_table_auto_fit() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::new(2);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let selection = Selection::default();
        let cmd = SetTableAutoFit::new(table_id, TableAutoFitMode::AutoFitWindow);
        let result = cmd.apply(&tree, &selection).unwrap();

        let table = result.tree.get_table(table_id).unwrap();
        assert_eq!(table.properties.auto_fit_mode, TableAutoFitMode::AutoFitWindow);
    }

    #[test]
    fn test_set_cell_padding() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::new(1);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        let cell = TableCell::new();
        let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();

        let selection = Selection::default();
        let padding = CellPadding::uniform(10.0);
        let cmd = SetCellPadding::single(cell_id, padding);
        let result = cmd.apply(&tree, &selection).unwrap();

        let cell = result.tree.get_table_cell(cell_id).unwrap();
        assert_eq!(cell.properties.padding, Some(padding));
    }

    #[test]
    fn test_vertical_merge() {
        let mut tree = DocumentTree::new();

        let grid = doc_model::TableGrid::with_equal_columns(2, 200.0);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        // Create 3 rows
        let mut row_ids = Vec::new();
        for _ in 0..3 {
            let row = TableRow::new();
            let row_id = tree.insert_table_row(row, table_id, None).unwrap();
            row_ids.push(row_id);

            for _ in 0..2 {
                let cell = TableCell::new();
                let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
                let para = Paragraph::new();
                tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
            }
        }

        let selection = Selection::default();
        let cmd = MergeCellsVertical::new(table_id, 0, 2, 0);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Check first cell has merge start
        let first_row = result.tree.get_table_row(row_ids[0]).unwrap();
        let first_cell_id = first_row.children()[0];
        let first_cell = result.tree.get_table_cell(first_cell_id).unwrap();
        assert_eq!(first_cell.v_merge, VerticalMerge::Start);
        assert_eq!(first_cell.row_span, 3);

        // Check second row's first cell has merge continue
        let second_row = result.tree.get_table_row(row_ids[1]).unwrap();
        let second_cell_id = second_row.children()[0];
        let second_cell = result.tree.get_table_cell(second_cell_id).unwrap();
        assert_eq!(second_cell.v_merge, VerticalMerge::Continue);
    }
}
