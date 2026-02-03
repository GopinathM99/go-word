//! Table Layout Algorithm
//!
//! This module implements the layout algorithm for tables, handling:
//! - Column width calculation (fixed, auto, percentage)
//! - Cell content layout using the line breaker
//! - Row height calculation (auto, exact, at-least)
//! - Cell positioning in the grid
//! - Border and padding handling
//! - Cell merging (horizontal and vertical spans)
//! - Header row repeat across pages
//! - Row breaking control
//! - Nested tables
//! - Auto-fit modes

use crate::{BlockBox, Direction, LineBox, LineBreakConfig, LineBreaker, Rect, Result};
use doc_model::{
    CellPadding, CellVerticalAlign, DocumentTree, HeightRule, HorizontalMerge, Node, NodeId,
    Table, TableAutoFitMode, TableCell, TableLayoutMode, TableRow, VerticalMerge,
    WidthType, MAX_TABLE_NESTING_DEPTH,
    table::CellTextDirection,
};

/// A laid-out table
#[derive(Debug, Clone)]
pub struct TableLayout {
    /// The table node ID
    pub table_id: NodeId,
    /// Total bounds of the table
    pub bounds: Rect,
    /// Row layouts
    pub rows: Vec<RowLayout>,
    /// Computed column widths
    pub column_widths: Vec<f32>,
    /// Header row indices (rows that should repeat on page breaks)
    pub header_rows: Vec<usize>,
    /// Merged cell regions for layout reference
    pub merged_regions: Vec<MergedRegionLayout>,
    /// Nested table layouts
    pub nested_tables: Vec<TableLayout>,
    /// Nesting depth of this table
    pub nesting_depth: usize,
}

/// Layout information for a merged cell region
#[derive(Debug, Clone)]
pub struct MergedRegionLayout {
    /// The anchor cell ID
    pub anchor_cell_id: NodeId,
    /// Starting row index
    pub start_row: usize,
    /// Starting column index
    pub start_col: usize,
    /// Row span
    pub row_span: usize,
    /// Column span
    pub col_span: usize,
    /// Total bounds of the merged region
    pub bounds: Rect,
}

/// A laid-out row
#[derive(Debug, Clone)]
pub struct RowLayout {
    /// The row node ID
    pub row_id: NodeId,
    /// Bounds relative to the table
    pub bounds: Rect,
    /// Cell layouts
    pub cells: Vec<CellLayout>,
    /// Whether this is a header row
    pub is_header: bool,
    /// Whether this row can be split across pages
    pub can_split: bool,
    /// Whether this row should keep with the next row
    pub keep_with_next: bool,
    /// Row index in the table
    pub row_index: usize,
}

/// Result of attempting to split a row across a page break
#[derive(Debug, Clone)]
pub struct RowSplitResult {
    /// The part of the row that fits on the current page
    pub first_part: Option<RowLayout>,
    /// The part of the row that goes to the next page
    pub second_part: Option<RowLayout>,
    /// Height of content that fits on current page
    pub split_height: f32,
}

/// A laid-out cell
#[derive(Debug, Clone)]
pub struct CellLayout {
    /// The cell node ID
    pub cell_id: NodeId,
    /// Bounds relative to the table
    pub bounds: Rect,
    /// Content bounds (after padding)
    pub content_bounds: Rect,
    /// Layout of content (paragraphs)
    pub content: Vec<BlockBox>,
    /// Column span
    pub grid_span: u32,
    /// Row span
    pub row_span: u32,
    /// Horizontal merge state
    pub h_merge: HorizontalMerge,
    /// Vertical merge state
    pub v_merge: VerticalMerge,
    /// Column index in the row (accounting for spans)
    pub col_index: usize,
    /// Text direction for this cell
    pub text_direction: CellTextDirection,
    /// Vertical alignment
    pub vertical_align: CellVerticalAlign,
    /// Whether this cell is covered by a merged cell
    pub is_covered: bool,
    /// Nested table layouts within this cell
    pub nested_tables: Vec<TableLayout>,
}

/// Configuration for table layout
#[derive(Debug, Clone)]
pub struct TableLayoutConfig {
    /// Available width for the table
    pub available_width: f32,
    /// Available height for the current page
    pub available_height: f32,
    /// Default column width when auto-sizing
    pub default_column_width: f32,
    /// Default font size for content
    pub font_size: f32,
    /// Line spacing for cell content
    pub line_spacing: f32,
    /// Minimum cell width
    pub min_cell_width: f32,
    /// Minimum cell height
    pub min_cell_height: f32,
    /// Maximum nesting depth for nested tables
    pub max_nesting_depth: usize,
    /// Current nesting depth (for recursive calls)
    pub current_nesting_depth: usize,
    /// Whether to repeat header rows on page breaks
    pub repeat_header_rows: bool,
    /// Auto-fit mode override (None uses table's setting)
    pub auto_fit_mode: Option<TableAutoFitMode>,
}

impl Default for TableLayoutConfig {
    fn default() -> Self {
        Self {
            available_width: 468.0, // 6.5 inches at 72 dpi
            available_height: 648.0, // Default page content height
            default_column_width: 100.0,
            font_size: 11.0,
            line_spacing: 1.08,
            min_cell_width: 20.0,
            min_cell_height: 14.0,
            max_nesting_depth: MAX_TABLE_NESTING_DEPTH,
            current_nesting_depth: 0,
            repeat_header_rows: true,
            auto_fit_mode: None,
        }
    }
}

impl TableLayoutConfig {
    /// Create a config for nested table layout
    pub fn for_nested(&self, available_width: f32) -> Self {
        Self {
            available_width,
            available_height: self.available_height,
            default_column_width: self.default_column_width.min(available_width / 2.0),
            font_size: self.font_size,
            line_spacing: self.line_spacing,
            min_cell_width: self.min_cell_width,
            min_cell_height: self.min_cell_height,
            max_nesting_depth: self.max_nesting_depth,
            current_nesting_depth: self.current_nesting_depth + 1,
            repeat_header_rows: self.repeat_header_rows,
            auto_fit_mode: self.auto_fit_mode,
        }
    }

    /// Check if we can nest another table level
    pub fn can_nest(&self) -> bool {
        self.current_nesting_depth < self.max_nesting_depth
    }
}

/// Internal structure for tracking merged regions during layout
#[derive(Debug, Clone)]
struct MergedRegionInternal {
    anchor_cell_id: NodeId,
    start_row: usize,
    start_col: usize,
    row_span: usize,
    col_span: usize,
    bounds: Rect,
}

/// Table layout engine
pub struct TableLayoutEngine {
    /// Line breaker for cell content
    line_breaker: LineBreaker,
}

impl TableLayoutEngine {
    /// Create a new table layout engine
    pub fn new() -> Self {
        Self {
            line_breaker: LineBreaker::new(),
        }
    }

    /// Create with an existing line breaker
    pub fn with_line_breaker(line_breaker: LineBreaker) -> Self {
        Self { line_breaker }
    }

    /// Get a mutable reference to the line breaker
    pub fn line_breaker_mut(&mut self) -> &mut LineBreaker {
        &mut self.line_breaker
    }

    /// Layout a table
    pub fn layout_table(
        &mut self,
        tree: &DocumentTree,
        table_id: NodeId,
        config: &TableLayoutConfig,
    ) -> Result<TableLayout> {
        let table = tree.get_table(table_id)
            .ok_or_else(|| crate::LayoutError::LayoutFailed("Table not found".into()))?;

        // Check nesting depth
        if config.current_nesting_depth >= config.max_nesting_depth {
            return Err(crate::LayoutError::LayoutFailed(
                format!("Maximum table nesting depth ({}) exceeded", config.max_nesting_depth)
            ));
        }

        // Calculate column widths based on auto-fit mode
        let column_widths = self.calculate_column_widths_with_mode(tree, table, config)?;

        // Build merged region map first
        let merged_regions = self.build_merged_regions(tree, table, &column_widths)?;

        // Track header rows
        let mut header_rows = Vec::new();

        // Layout each row
        let mut rows = Vec::new();
        let mut current_y = 0.0;
        let mut nested_tables = Vec::new();

        for (row_index, &row_id) in table.children().iter().enumerate() {
            if let Some(row) = tree.get_table_row(row_id) {
                let row_layout = self.layout_row_with_merges(
                    tree,
                    row,
                    row_index,
                    &column_widths,
                    current_y,
                    config,
                    &merged_regions,
                )?;

                // Track header rows
                if row_layout.is_header {
                    header_rows.push(row_index);
                }

                // Collect nested table layouts from cells
                for cell in &row_layout.cells {
                    nested_tables.extend(cell.nested_tables.clone());
                }

                current_y += row_layout.bounds.height;
                rows.push(row_layout);
            }
        }

        // Apply vertical merge heights (cells spanning multiple rows need height adjustments)
        self.apply_vertical_merge_heights(&mut rows, &merged_regions);

        // Calculate total table width
        let total_width: f32 = column_widths.iter().sum();

        // Convert merged regions to layout format
        let merged_region_layouts = merged_regions
            .iter()
            .map(|r| MergedRegionLayout {
                anchor_cell_id: r.anchor_cell_id,
                start_row: r.start_row,
                start_col: r.start_col,
                row_span: r.row_span,
                col_span: r.col_span,
                bounds: r.bounds,
            })
            .collect();

        Ok(TableLayout {
            table_id,
            bounds: Rect::new(0.0, 0.0, total_width, current_y),
            rows,
            column_widths,
            header_rows,
            merged_regions: merged_region_layouts,
            nested_tables,
            nesting_depth: config.current_nesting_depth,
        })
    }

    /// Calculate column widths based on auto-fit mode
    fn calculate_column_widths_with_mode(
        &self,
        tree: &DocumentTree,
        table: &Table,
        config: &TableLayoutConfig,
    ) -> Result<Vec<f32>> {
        let auto_fit_mode = config.auto_fit_mode.unwrap_or(table.properties.auto_fit_mode);

        match auto_fit_mode {
            TableAutoFitMode::AutoFitContent => {
                self.calculate_column_widths_auto_content(tree, table, config)
            }
            TableAutoFitMode::AutoFitWindow => {
                self.calculate_column_widths_auto_window(tree, table, config)
            }
            TableAutoFitMode::FixedWidth => {
                self.calculate_column_widths(tree, table, config)
            }
        }
    }

    /// Calculate column widths to fit content
    fn calculate_column_widths_auto_content(
        &self,
        tree: &DocumentTree,
        table: &Table,
        config: &TableLayoutConfig,
    ) -> Result<Vec<f32>> {
        let col_count = table.column_count();
        if col_count == 0 {
            return Ok(Vec::new());
        }

        let mut widths = vec![config.min_cell_width; col_count];

        // Measure content in each column
        for &row_id in table.children() {
            if let Some(row) = tree.get_table_row(row_id) {
                let mut col_idx = 0;
                for &cell_id in row.children() {
                    if let Some(cell) = tree.get_table_cell(cell_id) {
                        // Skip covered cells
                        if cell.h_merge == HorizontalMerge::Continue {
                            col_idx += 1;
                            continue;
                        }

                        let span = cell.effective_grid_span() as usize;
                        let cell_width = self.measure_cell_content_width(tree, cell, config);

                        // Distribute width across spanned columns
                        if span == 1 {
                            widths[col_idx] = widths[col_idx].max(cell_width);
                        } else {
                            // For spanning cells, distribute evenly
                            let per_col = cell_width / span as f32;
                            for i in 0..span.min(col_count - col_idx) {
                                widths[col_idx + i] = widths[col_idx + i].max(per_col);
                            }
                        }

                        col_idx += span;
                    }
                }
            }
        }

        // Ensure total doesn't exceed available width
        let total: f32 = widths.iter().sum();
        if total > config.available_width {
            let scale = config.available_width / total;
            for w in &mut widths {
                *w = (*w * scale).max(config.min_cell_width);
            }
        }

        Ok(widths)
    }

    /// Calculate column widths to fit window/page width
    fn calculate_column_widths_auto_window(
        &self,
        tree: &DocumentTree,
        table: &Table,
        config: &TableLayoutConfig,
    ) -> Result<Vec<f32>> {
        let col_count = table.column_count();
        if col_count == 0 {
            return Ok(Vec::new());
        }

        // Start with equal distribution
        let base_width = config.available_width / col_count as f32;
        let mut widths = vec![base_width.max(config.min_cell_width); col_count];

        // Adjust based on column definitions if present
        let mut fixed_width = 0.0f32;
        let mut flex_columns = Vec::new();

        for (i, col) in table.grid.columns.iter().enumerate() {
            match col.width.width_type {
                WidthType::Fixed => {
                    widths[i] = col.width.value.max(config.min_cell_width);
                    fixed_width += widths[i];
                }
                WidthType::Percent => {
                    let width = config.available_width * (col.width.value / 100.0);
                    widths[i] = width.max(config.min_cell_width);
                    fixed_width += widths[i];
                }
                WidthType::Auto => {
                    flex_columns.push(i);
                }
            }
        }

        // Distribute remaining space to flex columns
        if !flex_columns.is_empty() {
            let remaining = (config.available_width - fixed_width).max(0.0);
            let per_col = remaining / flex_columns.len() as f32;
            for &col_idx in &flex_columns {
                widths[col_idx] = per_col.max(config.min_cell_width);
            }
        }

        Ok(widths)
    }

    /// Build merged regions from table structure
    fn build_merged_regions(
        &self,
        tree: &DocumentTree,
        table: &Table,
        column_widths: &[f32],
    ) -> Result<Vec<MergedRegionInternal>> {
        let mut regions = Vec::new();
        let mut current_y = 0.0;

        // Track row heights for vertical merges
        let mut row_heights: Vec<f32> = Vec::new();

        // First pass: calculate row heights and identify merge starts
        for (row_index, &row_id) in table.children().iter().enumerate() {
            if let Some(row) = tree.get_table_row(row_id) {
                let mut max_height = 14.0f32; // minimum row height

                let mut col_idx = 0;
                for &cell_id in row.children() {
                    if let Some(cell) = tree.get_table_cell(cell_id) {
                        // Skip covered cells for height calculation
                        if !cell.is_covered() {
                            let padding = cell.properties.padding.unwrap_or_default();
                            let content_height = 14.0; // Estimated, should measure content
                            let cell_height = content_height + padding.top + padding.bottom;
                            max_height = max_height.max(cell_height);
                        }

                        // Track horizontal merge regions
                        if cell.h_merge == HorizontalMerge::Start || cell.grid_span > 1 {
                            let span = cell.effective_grid_span() as usize;
                            let width: f32 = column_widths[col_idx..col_idx + span.min(column_widths.len() - col_idx)]
                                .iter()
                                .sum();

                            // Check if also a vertical merge
                            let row_span = if cell.v_merge == VerticalMerge::Start {
                                cell.effective_row_span() as usize
                            } else {
                                1
                            };

                            let x: f32 = column_widths[..col_idx].iter().sum();

                            regions.push(MergedRegionInternal {
                                anchor_cell_id: cell_id,
                                start_row: row_index,
                                start_col: col_idx,
                                row_span,
                                col_span: span,
                                bounds: Rect::new(x, current_y, width, 0.0), // Height TBD
                            });
                        } else if cell.v_merge == VerticalMerge::Start && cell.grid_span <= 1 {
                            // Pure vertical merge (no horizontal)
                            let row_span = cell.effective_row_span() as usize;
                            let width = column_widths.get(col_idx).copied().unwrap_or(0.0);
                            let x: f32 = column_widths[..col_idx].iter().sum();

                            regions.push(MergedRegionInternal {
                                anchor_cell_id: cell_id,
                                start_row: row_index,
                                start_col: col_idx,
                                row_span,
                                col_span: 1,
                                bounds: Rect::new(x, current_y, width, 0.0),
                            });
                        }

                        col_idx += cell.effective_grid_span() as usize;
                    }
                }

                row_heights.push(max_height);
                current_y += max_height;
            }
        }

        // Second pass: update region heights based on row heights
        for region in &mut regions {
            let start_y: f32 = row_heights[..region.start_row].iter().sum();
            let height: f32 = row_heights[region.start_row..region.start_row + region.row_span]
                .iter()
                .sum();
            region.bounds.y = start_y;
            region.bounds.height = height;
        }

        Ok(regions)
    }

    /// Apply vertical merge heights to rows
    fn apply_vertical_merge_heights(
        &self,
        rows: &mut [RowLayout],
        merged_regions: &[MergedRegionInternal],
    ) {
        for region in merged_regions {
            if region.row_span > 1 {
                // Calculate total height of merged rows
                let total_height: f32 = rows[region.start_row..region.start_row + region.row_span]
                    .iter()
                    .map(|r| r.bounds.height)
                    .sum();

                // Update the anchor cell's bounds
                if let Some(row) = rows.get_mut(region.start_row) {
                    for cell in &mut row.cells {
                        if cell.cell_id == region.anchor_cell_id {
                            cell.bounds.height = total_height;
                            cell.row_span = region.row_span as u32;
                        }
                    }
                }
            }
        }
    }

    /// Calculate column widths based on the grid definition and table properties
    fn calculate_column_widths(
        &self,
        tree: &DocumentTree,
        table: &Table,
        config: &TableLayoutConfig,
    ) -> Result<Vec<f32>> {
        let col_count = table.column_count();
        if col_count == 0 {
            return Ok(Vec::new());
        }

        // Determine available width
        let available = match &table.properties.width {
            Some(w) => match w.width_type {
                WidthType::Fixed => w.value,
                WidthType::Percent => config.available_width * (w.value / 100.0),
                WidthType::Auto => config.available_width,
            },
            None => config.available_width,
        };

        let mut widths = vec![0.0_f32; col_count];
        let mut fixed_width = 0.0_f32;
        let mut auto_columns = Vec::new();
        let mut percent_sum = 0.0_f32;

        // First pass: handle fixed and percent widths
        for (i, col) in table.grid.columns.iter().enumerate() {
            match col.width.width_type {
                WidthType::Fixed => {
                    widths[i] = col.width.value.max(config.min_cell_width);
                    fixed_width += widths[i];
                }
                WidthType::Percent => {
                    let width = available * (col.width.value / 100.0);
                    widths[i] = width.max(config.min_cell_width);
                    percent_sum += col.width.value;
                }
                WidthType::Auto => {
                    auto_columns.push(i);
                }
            }
        }

        // Calculate remaining width for auto columns
        let percent_width: f32 = widths.iter().sum::<f32>() - fixed_width;
        let remaining = (available - fixed_width - percent_width).max(0.0);

        // Distribute remaining width to auto columns
        if !auto_columns.is_empty() {
            // First, try to measure content to get optimal widths
            let auto_widths = self.measure_auto_columns(tree, table, &auto_columns, config);

            let total_auto_desired: f32 = auto_widths.iter().sum();

            if total_auto_desired <= remaining {
                // Use desired widths
                for (i, &col_idx) in auto_columns.iter().enumerate() {
                    widths[col_idx] = auto_widths[i].max(config.min_cell_width);
                }
            } else {
                // Proportionally distribute remaining space
                let scale = remaining / total_auto_desired.max(1.0);
                for (i, &col_idx) in auto_columns.iter().enumerate() {
                    widths[col_idx] = (auto_widths[i] * scale).max(config.min_cell_width);
                }
            }
        }

        Ok(widths)
    }

    /// Measure optimal widths for auto columns based on content
    fn measure_auto_columns(
        &self,
        tree: &DocumentTree,
        table: &Table,
        auto_columns: &[usize],
        config: &TableLayoutConfig,
    ) -> Vec<f32> {
        let mut widths = vec![config.default_column_width; auto_columns.len()];

        // For each row, measure content in auto columns
        for &row_id in table.children() {
            if let Some(row) = tree.get_table_row(row_id) {
                for (auto_idx, &col_idx) in auto_columns.iter().enumerate() {
                    if let Some(&cell_id) = row.children().get(col_idx) {
                        if let Some(cell) = tree.get_table_cell(cell_id) {
                            // Skip cells that span multiple columns for now
                            if cell.grid_span <= 1 {
                                let cell_width = self.measure_cell_content_width(tree, cell, config);
                                widths[auto_idx] = widths[auto_idx].max(cell_width);
                            }
                        }
                    }
                }
            }
        }

        widths
    }

    /// Measure the natural width of cell content
    fn measure_cell_content_width(
        &self,
        tree: &DocumentTree,
        cell: &TableCell,
        config: &TableLayoutConfig,
    ) -> f32 {
        let padding = cell.properties.padding
            .unwrap_or_default();

        let mut max_width = 0.0_f32;

        // Measure each paragraph's natural width (single line)
        for &child_id in cell.children() {
            if let Some(para) = tree.get_paragraph(child_id) {
                let mut para_width = 0.0_f32;
                for &run_id in para.children() {
                    if let Some(run) = tree.get_run(run_id) {
                        // Estimate width based on character count and font size
                        let font_size = run.style.font_size.unwrap_or(config.font_size);
                        para_width += run.text.chars().count() as f32 * font_size * 0.5;
                    }
                }
                max_width = max_width.max(para_width);
            }
        }

        max_width + padding.left + padding.right
    }

    /// Layout a single row (original method for backward compatibility)
    fn layout_row(
        &mut self,
        tree: &DocumentTree,
        row: &TableRow,
        column_widths: &[f32],
        y_offset: f32,
        config: &TableLayoutConfig,
    ) -> Result<RowLayout> {
        self.layout_row_with_merges(tree, row, 0, column_widths, y_offset, config, &[])
    }

    /// Layout a single row with merge region awareness
    fn layout_row_with_merges(
        &mut self,
        tree: &DocumentTree,
        row: &TableRow,
        row_index: usize,
        column_widths: &[f32],
        y_offset: f32,
        config: &TableLayoutConfig,
        merged_regions: &[MergedRegionInternal],
    ) -> Result<RowLayout> {
        let row_id = row.id();

        // Layout each cell
        let mut cells = Vec::new();
        let mut current_x = 0.0;
        let mut col_idx = 0;
        let mut max_height = config.min_cell_height;

        for &cell_id in row.children() {
            if let Some(cell) = tree.get_table_cell(cell_id) {
                // Check if this cell is covered by a vertical merge from above
                let is_covered = self.is_cell_covered_by_merge(row_index, col_idx, merged_regions);

                // Calculate cell width (considering grid_span)
                let span = cell.effective_grid_span() as usize;
                let cell_width: f32 = column_widths[col_idx..col_idx + span.min(column_widths.len() - col_idx)]
                    .iter()
                    .sum();

                let cell_layout = self.layout_cell_with_options(
                    tree,
                    cell,
                    current_x,
                    0.0,
                    cell_width,
                    config,
                    col_idx,
                    is_covered,
                )?;

                // Don't count covered cells for row height
                if !is_covered {
                    max_height = max_height.max(cell_layout.bounds.height);
                }
                cells.push(cell_layout);

                current_x += cell_width;
                col_idx += span;
            }
        }

        // Apply row height rule
        let final_height = match row.properties.height_rule {
            HeightRule::Auto => max_height,
            HeightRule::Exact => row.properties.height.unwrap_or(max_height),
            HeightRule::AtLeast => row.properties.height.unwrap_or(0.0).max(max_height),
        };

        // Update cell heights to match row height and apply vertical alignment
        for cell in &mut cells {
            // Skip covered cells
            if cell.is_covered {
                cell.bounds.height = final_height;
                continue;
            }

            cell.bounds.height = final_height;

            // Apply vertical alignment
            let v_align = cell.vertical_align;
            let content_height: f32 = cell.content.iter().map(|b| b.bounds.height).sum();

            if let Some(table_cell) = tree.get_table_cell(cell.cell_id) {
                let padding = table_cell.properties.padding.unwrap_or_default();

                let content_y_offset = match v_align {
                    CellVerticalAlign::Top => padding.top,
                    CellVerticalAlign::Center => {
                        (final_height - content_height) / 2.0
                    }
                    CellVerticalAlign::Bottom => {
                        final_height - content_height - padding.bottom
                    }
                };

                // Adjust content y positions
                let mut current_y = content_y_offset;
                for block in &mut cell.content {
                    block.bounds.y = current_y;
                    current_y += block.bounds.height;
                }
            }
        }

        let total_width: f32 = column_widths.iter().sum();

        Ok(RowLayout {
            row_id,
            bounds: Rect::new(0.0, y_offset, total_width, final_height),
            cells,
            is_header: row.properties.is_header,
            can_split: row.properties.can_split && !row.properties.cant_split,
            keep_with_next: row.properties.keep_with_next,
            row_index,
        })
    }

    /// Check if a cell position is covered by a vertical merge from a previous row
    fn is_cell_covered_by_merge(
        &self,
        row_index: usize,
        col_index: usize,
        merged_regions: &[MergedRegionInternal],
    ) -> bool {
        for region in merged_regions {
            if region.row_span > 1
                && row_index > region.start_row
                && row_index < region.start_row + region.row_span
                && col_index >= region.start_col
                && col_index < region.start_col + region.col_span
            {
                return true;
            }
        }
        false
    }

    /// Try to split a row at a given height for page breaking
    pub fn try_split_row(
        &mut self,
        tree: &DocumentTree,
        row: &TableRow,
        row_layout: &RowLayout,
        available_height: f32,
        config: &TableLayoutConfig,
    ) -> Result<RowSplitResult> {
        // Check if row can be split
        if row.properties.cant_split || !row.properties.can_split {
            return Ok(RowSplitResult {
                first_part: None,
                second_part: None,
                split_height: 0.0,
            });
        }

        // Check if available height is enough for minimum content
        if available_height < config.min_cell_height {
            return Ok(RowSplitResult {
                first_part: None,
                second_part: Some(row_layout.clone()),
                split_height: 0.0,
            });
        }

        // For now, simple implementation: if row doesn't fit, don't split
        // A more sophisticated implementation would split individual cell content
        if row_layout.bounds.height <= available_height {
            return Ok(RowSplitResult {
                first_part: Some(row_layout.clone()),
                second_part: None,
                split_height: row_layout.bounds.height,
            });
        }

        // Row is taller than available space - check if we can split cell content
        // This is a complex operation that requires splitting paragraphs within cells
        // For this implementation, we'll allow the row to overflow if cant_split is false
        Ok(RowSplitResult {
            first_part: None,
            second_part: Some(row_layout.clone()),
            split_height: 0.0,
        })
    }

    /// Get header rows that should be repeated on a new page
    pub fn get_header_rows_for_repeat<'a>(
        &self,
        layout: &'a TableLayout,
    ) -> Vec<&'a RowLayout> {
        layout.header_rows
            .iter()
            .filter_map(|&idx| layout.rows.get(idx))
            .collect()
    }

    /// Layout a single cell (original method for backward compatibility)
    fn layout_cell(
        &mut self,
        tree: &DocumentTree,
        cell: &TableCell,
        x: f32,
        y: f32,
        width: f32,
        config: &TableLayoutConfig,
    ) -> Result<CellLayout> {
        self.layout_cell_with_options(tree, cell, x, y, width, config, 0, false)
    }

    /// Layout a single cell with additional options
    fn layout_cell_with_options(
        &mut self,
        tree: &DocumentTree,
        cell: &TableCell,
        x: f32,
        y: f32,
        width: f32,
        config: &TableLayoutConfig,
        col_index: usize,
        is_covered: bool,
    ) -> Result<CellLayout> {
        let cell_id = cell.id();
        let padding = cell.properties.padding.unwrap_or_default();
        let text_direction = cell.properties.text_direction.unwrap_or_default();
        let vertical_align = cell.properties.vertical_align.unwrap_or_default();

        // If cell is covered by a merge, return minimal layout
        if is_covered || cell.is_covered() {
            return Ok(CellLayout {
                cell_id,
                bounds: Rect::new(x, y, width, config.min_cell_height),
                content_bounds: Rect::new(x + padding.left, padding.top, 0.0, 0.0),
                content: Vec::new(),
                grid_span: cell.effective_grid_span(),
                row_span: cell.effective_row_span(),
                h_merge: cell.h_merge,
                v_merge: cell.v_merge,
                col_index,
                text_direction,
                vertical_align,
                is_covered: true,
                nested_tables: Vec::new(),
            });
        }

        // Content area after padding
        let content_x = x + padding.left;
        let content_y = padding.top;
        let content_width = (width - padding.left - padding.right).max(config.min_cell_width);

        // Determine text direction for line breaking
        let direction = match text_direction {
            CellTextDirection::Ltr => Direction::Ltr,
            CellTextDirection::Rtl => Direction::Rtl,
            CellTextDirection::TbLr | CellTextDirection::TbRl => Direction::Ltr, // Vertical text not yet supported
        };

        // Layout cell content (paragraphs and nested tables)
        let mut content = Vec::new();
        let mut nested_tables = Vec::new();
        let mut content_y_offset = content_y;

        for &child_id in cell.children() {
            // Check for nested table
            if let Some(nested_table) = tree.get_table(child_id) {
                if config.can_nest() {
                    // Create nested config
                    let nested_config = config.for_nested(content_width);

                    // Layout nested table recursively
                    if let Ok(nested_layout) = self.layout_table(tree, child_id, &nested_config) {
                        content_y_offset += nested_layout.bounds.height;
                        nested_tables.push(nested_layout);
                    }
                }
                continue;
            }

            // Regular paragraph content
            if let Some(_para) = tree.get_paragraph(child_id) {
                // Create line break config for this cell
                let line_config = LineBreakConfig {
                    available_width: content_width,
                    font_size: config.font_size,
                    line_spacing: config.line_spacing,
                    first_line_indent: 0.0,
                    left_indent: 0.0,
                    right_indent: 0.0,
                    direction,
                    allow_hyphenation: false,
                    alignment: doc_model::Alignment::Left,
                    list_num_id: None,
                    list_level: None,
                    list_marker_text: None,
                    list_is_bullet: false,
                    list_marker_font: None,
                    list_hanging: 0.0,
                };

                // Break paragraph into lines
                let broken = self.line_breaker.break_paragraph(tree, child_id, &line_config)?;

                // Create block box for the paragraph
                let block = BlockBox {
                    node_id: child_id,
                    bounds: Rect::new(content_x, content_y_offset, content_width, broken.total_height),
                    lines: broken.lines,
                };

                content_y_offset += broken.total_height;
                content.push(block);
            }
        }

        // Calculate total height
        let content_height = content_y_offset - content_y;
        let total_height = content_height + padding.top + padding.bottom;
        let final_height = total_height.max(config.min_cell_height);

        Ok(CellLayout {
            cell_id,
            bounds: Rect::new(x, y, width, final_height),
            content_bounds: Rect::new(content_x, content_y, content_width, content_height),
            content,
            grid_span: cell.effective_grid_span(),
            row_span: cell.effective_row_span(),
            h_merge: cell.h_merge,
            v_merge: cell.v_merge,
            col_index,
            text_direction,
            vertical_align,
            is_covered: false,
            nested_tables,
        })
    }
}

impl Default for TableLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a table layout to block boxes for rendering
pub fn table_layout_to_blocks(
    layout: &TableLayout,
    x_offset: f32,
    y_offset: f32,
) -> Vec<BlockBox> {
    let mut blocks = Vec::new();

    // Each cell's content becomes block boxes
    for row in &layout.rows {
        for cell in &row.cells {
            for block in &cell.content {
                let mut adjusted_block = block.clone();
                adjusted_block.bounds.x += x_offset;
                adjusted_block.bounds.y += y_offset + row.bounds.y;

                // Also adjust line positions
                for line in &mut adjusted_block.lines {
                    line.bounds.y += row.bounds.y;
                    for inline in &mut line.inlines {
                        inline.bounds.y += row.bounds.y;
                    }
                }

                blocks.push(adjusted_block);
            }
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{GridColumn, Paragraph, RowProperties, Run, TableGrid, TableProperties, TableWidth};

    fn create_test_tree_with_table() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();

        // Create a 2x2 table
        let grid = TableGrid::with_fixed_columns(&[100.0, 150.0]);
        let props = TableProperties::new()
            .with_width(TableWidth::fixed(250.0));
        let table = Table::with_grid_and_properties(grid, props);
        let table_id = tree.insert_table(table, None).unwrap();

        // Add two rows with cells
        for _row_idx in 0..2 {
            let row = TableRow::new();
            let row_id = tree.insert_table_row(row, table_id, None).unwrap();

            for _col_idx in 0..2 {
                let cell = TableCell::new();
                let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();

                // Add a paragraph to each cell
                let para = Paragraph::new();
                let para_id = tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();

                // Add a run with text
                let run = Run::new("Test");
                tree.insert_run(run, para_id, None).unwrap();
            }
        }

        (tree, table_id)
    }

    #[test]
    fn test_table_layout_basic() {
        let (tree, table_id) = create_test_tree_with_table();
        let mut engine = TableLayoutEngine::new();
        let config = TableLayoutConfig::default();

        let layout = engine.layout_table(&tree, table_id, &config).unwrap();

        assert_eq!(layout.column_widths.len(), 2);
        assert_eq!(layout.rows.len(), 2);
        assert_eq!(layout.rows[0].cells.len(), 2);
    }

    #[test]
    fn test_column_width_calculation() {
        let (tree, table_id) = create_test_tree_with_table();
        let engine = TableLayoutEngine::new();
        let config = TableLayoutConfig::default();

        let table = tree.get_table(table_id).unwrap();
        let widths = engine.calculate_column_widths(&tree, table, &config).unwrap();

        // Should use fixed widths from grid
        assert_eq!(widths[0], 100.0);
        assert_eq!(widths[1], 150.0);
    }

    #[test]
    fn test_auto_column_widths() {
        let mut tree = DocumentTree::new();

        // Create a table with auto columns
        let mut grid = TableGrid::new(2);
        grid.columns[0] = GridColumn::auto();
        grid.columns[1] = GridColumn::auto();

        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        // Add content
        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        for _ in 0..2 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
            let para = Paragraph::new();
            let para_id = tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
            let run = Run::new("Test content");
            tree.insert_run(run, para_id, None).unwrap();
        }

        let engine = TableLayoutEngine::new();
        let config = TableLayoutConfig::default();

        let table = tree.get_table(table_id).unwrap();
        let widths = engine.calculate_column_widths(&tree, table, &config).unwrap();

        // Auto columns should have calculated widths
        assert!(widths[0] >= config.min_cell_width);
        assert!(widths[1] >= config.min_cell_width);
    }

    #[test]
    fn test_horizontal_merge_layout() {
        let mut tree = DocumentTree::new();

        // Create a 2x3 table with merged cells in first row
        let grid = TableGrid::with_fixed_columns(&[100.0, 100.0, 100.0]);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        // Row 1: merged cell spanning 2 columns
        let row1 = TableRow::new();
        let row1_id = tree.insert_table_row(row1, table_id, None).unwrap();

        let merged_cell = TableCell::spanning_columns(2);
        let cell1_id = tree.insert_table_cell(merged_cell, row1_id, None).unwrap();
        let para = Paragraph::new();
        tree.insert_paragraph_into_cell(para, cell1_id, None).unwrap();

        let normal_cell = TableCell::new();
        let cell2_id = tree.insert_table_cell(normal_cell, row1_id, None).unwrap();
        let para = Paragraph::new();
        tree.insert_paragraph_into_cell(para, cell2_id, None).unwrap();

        // Row 2: normal cells
        let row2 = TableRow::new();
        let row2_id = tree.insert_table_row(row2, table_id, None).unwrap();

        for _ in 0..3 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, row2_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        let mut engine = TableLayoutEngine::new();
        let config = TableLayoutConfig::default();

        let layout = engine.layout_table(&tree, table_id, &config).unwrap();

        assert_eq!(layout.rows.len(), 2);
        // First row should have 2 cells (one spanning 2 columns)
        assert_eq!(layout.rows[0].cells.len(), 2);
        // First cell should span 2 columns
        assert_eq!(layout.rows[0].cells[0].grid_span, 2);
    }

    #[test]
    fn test_header_row_tracking() {
        let mut tree = DocumentTree::new();

        let grid = TableGrid::with_fixed_columns(&[100.0, 100.0]);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        // Header row
        let header_row = TableRow::with_properties(RowProperties::new().as_header());
        let header_id = tree.insert_table_row(header_row, table_id, None).unwrap();

        for _ in 0..2 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, header_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        // Data row
        let data_row = TableRow::new();
        let data_id = tree.insert_table_row(data_row, table_id, None).unwrap();

        for _ in 0..2 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, data_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        let mut engine = TableLayoutEngine::new();
        let config = TableLayoutConfig::default();

        let layout = engine.layout_table(&tree, table_id, &config).unwrap();

        assert_eq!(layout.header_rows.len(), 1);
        assert_eq!(layout.header_rows[0], 0);
        assert!(layout.rows[0].is_header);
        assert!(!layout.rows[1].is_header);
    }

    #[test]
    fn test_auto_fit_window_mode() {
        let mut tree = DocumentTree::new();

        let grid = TableGrid::new(3); // 3 auto columns
        let props = TableProperties::new()
            .with_auto_fit(TableAutoFitMode::AutoFitWindow);
        let table = Table::with_grid_and_properties(grid, props);
        let table_id = tree.insert_table(table, None).unwrap();

        let row = TableRow::new();
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        for _ in 0..3 {
            let cell = TableCell::new();
            let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
            let para = Paragraph::new();
            tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();
        }

        let mut engine = TableLayoutEngine::new();
        let config = TableLayoutConfig {
            available_width: 300.0,
            ..Default::default()
        };

        let layout = engine.layout_table(&tree, table_id, &config).unwrap();

        // Total width should equal available width
        let total_width: f32 = layout.column_widths.iter().sum();
        assert!((total_width - 300.0).abs() < 1.0);
    }

    #[test]
    fn test_nested_config() {
        let config = TableLayoutConfig::default();
        assert!(config.can_nest());

        let nested = config.for_nested(200.0);
        assert_eq!(nested.current_nesting_depth, 1);
        assert_eq!(nested.available_width, 200.0);
        assert!(nested.can_nest());

        // Simulate deep nesting
        let mut deep = config;
        for _ in 0..MAX_TABLE_NESTING_DEPTH {
            deep = deep.for_nested(100.0);
        }
        assert!(!deep.can_nest());
    }

    #[test]
    fn test_row_split_properties() {
        let mut tree = DocumentTree::new();

        let grid = TableGrid::with_fixed_columns(&[100.0]);
        let table = Table::with_grid(grid);
        let table_id = tree.insert_table(table, None).unwrap();

        // Non-splittable row
        let row = TableRow::with_properties(RowProperties::new().prevent_split());
        let row_id = tree.insert_table_row(row, table_id, None).unwrap();

        let cell = TableCell::new();
        let cell_id = tree.insert_table_cell(cell, row_id, None).unwrap();
        let para = Paragraph::new();
        tree.insert_paragraph_into_cell(para, cell_id, None).unwrap();

        let mut engine = TableLayoutEngine::new();
        let config = TableLayoutConfig::default();

        let layout = engine.layout_table(&tree, table_id, &config).unwrap();

        assert!(!layout.rows[0].can_split);
    }
}
