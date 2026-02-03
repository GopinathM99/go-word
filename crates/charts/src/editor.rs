//! Chart data editor
//!
//! This module provides a spreadsheet-like interface for editing chart data,
//! with support for adding/removing series and categories, and data validation.

use crate::error::{ChartError, ChartResult};
use crate::model::*;
use serde::{Deserialize, Serialize};

/// A spreadsheet-like editor for chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataEditor {
    /// The chart data being edited
    data: ChartData,
    /// Whether values are validated (e.g., no NaN/Inf)
    strict_validation: bool,
    /// Minimum allowed value (if set)
    min_value: Option<f64>,
    /// Maximum allowed value (if set)
    max_value: Option<f64>,
    /// History of changes for undo
    #[serde(skip)]
    undo_stack: Vec<ChartData>,
    /// Future states for redo
    #[serde(skip)]
    redo_stack: Vec<ChartData>,
    /// Maximum history size
    max_history: usize,
}

impl Default for ChartDataEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartDataEditor {
    /// Create a new empty chart data editor
    pub fn new() -> Self {
        Self {
            data: ChartData::default(),
            strict_validation: true,
            min_value: None,
            max_value: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }

    /// Create an editor from existing chart data
    pub fn from_data(data: ChartData) -> Self {
        Self {
            data,
            strict_validation: true,
            min_value: None,
            max_value: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }

    /// Enable or disable strict validation
    pub fn with_strict_validation(mut self, enabled: bool) -> Self {
        self.strict_validation = enabled;
        self
    }

    /// Set minimum allowed value
    pub fn with_min_value(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Set maximum allowed value
    pub fn with_max_value(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Set value range constraints
    pub fn with_value_range(mut self, min: f64, max: f64) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    /// Get the current chart data
    pub fn data(&self) -> &ChartData {
        &self.data
    }

    /// Get a mutable reference to the chart data
    pub fn data_mut(&mut self) -> &mut ChartData {
        &mut self.data
    }

    /// Consume the editor and return the chart data
    pub fn into_data(self) -> ChartData {
        self.data
    }

    /// Save current state for undo
    fn save_state(&mut self) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.data.clone());
        self.redo_stack.clear();
    }

    /// Undo the last change
    pub fn undo(&mut self) -> bool {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(self.data.clone());
            self.data = previous;
            true
        } else {
            false
        }
    }

    /// Redo the last undone change
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.data.clone());
            self.data = next;
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Validate a value according to the editor's constraints
    pub fn validate_value(&self, value: f64) -> ChartResult<f64> {
        if self.strict_validation {
            if value.is_nan() {
                return Err(ChartError::InvalidData("NaN values are not allowed".to_string()));
            }
            if value.is_infinite() {
                return Err(ChartError::InvalidData(
                    "Infinite values are not allowed".to_string(),
                ));
            }
        }

        if let Some(min) = self.min_value {
            if value < min {
                return Err(ChartError::InvalidData(format!(
                    "Value {} is below minimum {}",
                    value, min
                )));
            }
        }

        if let Some(max) = self.max_value {
            if value > max {
                return Err(ChartError::InvalidData(format!(
                    "Value {} is above maximum {}",
                    value, max
                )));
            }
        }

        Ok(value)
    }

    // === Series Management ===

    /// Get the number of series
    pub fn series_count(&self) -> usize {
        self.data.series.len()
    }

    /// Get a series by index
    pub fn get_series(&self, index: usize) -> Option<&DataSeries> {
        self.data.series.get(index)
    }

    /// Get a mutable reference to a series by index
    pub fn get_series_mut(&mut self, index: usize) -> Option<&mut DataSeries> {
        self.data.series.get_mut(index)
    }

    /// Add a new series
    pub fn add_series(&mut self, series: DataSeries) -> ChartResult<usize> {
        // Validate all values in the series
        for &value in &series.values {
            self.validate_value(value)?;
        }

        self.save_state();
        self.data.series.push(series);
        Ok(self.data.series.len() - 1)
    }

    /// Add a new empty series with a name
    pub fn add_empty_series(&mut self, name: impl Into<String>) -> usize {
        self.save_state();
        let values = vec![0.0; self.data.categories.len()];
        self.data.series.push(DataSeries::new(name, values));
        self.data.series.len() - 1
    }

    /// Remove a series by index
    pub fn remove_series(&mut self, index: usize) -> ChartResult<DataSeries> {
        if index >= self.data.series.len() {
            return Err(ChartError::InvalidData(format!(
                "Series index {} out of bounds",
                index
            )));
        }
        self.save_state();
        Ok(self.data.series.remove(index))
    }

    /// Move a series to a new position
    pub fn move_series(&mut self, from: usize, to: usize) -> ChartResult<()> {
        if from >= self.data.series.len() || to >= self.data.series.len() {
            return Err(ChartError::InvalidData(
                "Series index out of bounds".to_string(),
            ));
        }
        if from == to {
            return Ok(());
        }

        self.save_state();
        let series = self.data.series.remove(from);
        self.data.series.insert(to, series);
        Ok(())
    }

    /// Rename a series
    pub fn rename_series(&mut self, index: usize, name: impl Into<String>) -> ChartResult<()> {
        if index >= self.data.series.len() {
            return Err(ChartError::InvalidData(format!(
                "Series index {} out of bounds",
                index
            )));
        }
        self.save_state();
        self.data.series[index].name = name.into();
        Ok(())
    }

    /// Duplicate a series
    pub fn duplicate_series(&mut self, index: usize) -> ChartResult<usize> {
        if index >= self.data.series.len() {
            return Err(ChartError::InvalidData(format!(
                "Series index {} out of bounds",
                index
            )));
        }
        self.save_state();
        let mut series = self.data.series[index].clone();
        series.name = format!("{} (copy)", series.name);
        self.data.series.push(series);
        Ok(self.data.series.len() - 1)
    }

    // === Category Management ===

    /// Get the number of categories
    pub fn category_count(&self) -> usize {
        self.data.categories.len()
    }

    /// Get a category by index
    pub fn get_category(&self, index: usize) -> Option<&str> {
        self.data.categories.get(index).map(|s| s.as_str())
    }

    /// Add a new category
    pub fn add_category(&mut self, name: impl Into<String>) -> usize {
        self.save_state();
        self.data.categories.push(name.into());

        // Add a default value to each series
        for series in &mut self.data.series {
            series.values.push(0.0);
        }

        self.data.categories.len() - 1
    }

    /// Insert a category at a specific position
    pub fn insert_category(&mut self, index: usize, name: impl Into<String>) -> ChartResult<()> {
        if index > self.data.categories.len() {
            return Err(ChartError::InvalidData(format!(
                "Category index {} out of bounds",
                index
            )));
        }

        self.save_state();
        self.data.categories.insert(index, name.into());

        // Insert a default value in each series
        for series in &mut self.data.series {
            if index <= series.values.len() {
                series.values.insert(index, 0.0);
            }
        }

        Ok(())
    }

    /// Remove a category by index
    pub fn remove_category(&mut self, index: usize) -> ChartResult<String> {
        if index >= self.data.categories.len() {
            return Err(ChartError::InvalidData(format!(
                "Category index {} out of bounds",
                index
            )));
        }

        self.save_state();
        let removed = self.data.categories.remove(index);

        // Remove the corresponding value from each series
        for series in &mut self.data.series {
            if index < series.values.len() {
                series.values.remove(index);
            }
        }

        Ok(removed)
    }

    /// Rename a category
    pub fn rename_category(&mut self, index: usize, name: impl Into<String>) -> ChartResult<()> {
        if index >= self.data.categories.len() {
            return Err(ChartError::InvalidData(format!(
                "Category index {} out of bounds",
                index
            )));
        }
        self.save_state();
        self.data.categories[index] = name.into();
        Ok(())
    }

    /// Move a category to a new position
    pub fn move_category(&mut self, from: usize, to: usize) -> ChartResult<()> {
        if from >= self.data.categories.len() || to >= self.data.categories.len() {
            return Err(ChartError::InvalidData(
                "Category index out of bounds".to_string(),
            ));
        }
        if from == to {
            return Ok(());
        }

        self.save_state();

        // Move category
        let category = self.data.categories.remove(from);
        self.data.categories.insert(to, category);

        // Move corresponding values in each series
        for series in &mut self.data.series {
            if from < series.values.len() {
                let value = series.values.remove(from);
                let insert_at = to.min(series.values.len());
                series.values.insert(insert_at, value);
            }
        }

        Ok(())
    }

    /// Set all categories at once
    pub fn set_categories(&mut self, categories: Vec<String>) {
        self.save_state();
        let old_count = self.data.categories.len();
        let new_count = categories.len();

        self.data.categories = categories;

        // Adjust series values to match new category count
        for series in &mut self.data.series {
            if series.values.len() < new_count {
                series.values.resize(new_count, 0.0);
            } else if series.values.len() > new_count {
                series.values.truncate(new_count);
            }
        }

        let _ = old_count; // Suppress unused warning
    }

    // === Value Management ===

    /// Get a value at a specific position
    pub fn get_value(&self, series_index: usize, category_index: usize) -> Option<f64> {
        self.data
            .series
            .get(series_index)
            .and_then(|s| s.values.get(category_index))
            .copied()
    }

    /// Set a value at a specific position
    pub fn set_value(
        &mut self,
        series_index: usize,
        category_index: usize,
        value: f64,
    ) -> ChartResult<()> {
        let validated = self.validate_value(value)?;

        if series_index >= self.data.series.len() {
            return Err(ChartError::InvalidData(format!(
                "Series index {} out of bounds",
                series_index
            )));
        }

        if category_index >= self.data.series[series_index].values.len() {
            return Err(ChartError::InvalidData(format!(
                "Category index {} out of bounds",
                category_index
            )));
        }

        self.save_state();
        self.data.series[series_index].values[category_index] = validated;
        Ok(())
    }

    /// Set multiple values in a series
    pub fn set_series_values(&mut self, series_index: usize, values: Vec<f64>) -> ChartResult<()> {
        // Validate all values
        for &value in &values {
            self.validate_value(value)?;
        }

        if series_index >= self.data.series.len() {
            return Err(ChartError::InvalidData(format!(
                "Series index {} out of bounds",
                series_index
            )));
        }

        self.save_state();
        self.data.series[series_index].values = values;
        Ok(())
    }

    /// Increment a value by a delta
    pub fn increment_value(
        &mut self,
        series_index: usize,
        category_index: usize,
        delta: f64,
    ) -> ChartResult<f64> {
        let current = self
            .get_value(series_index, category_index)
            .ok_or_else(|| ChartError::InvalidData("Index out of bounds".to_string()))?;

        let new_value = current + delta;
        self.set_value(series_index, category_index, new_value)?;
        Ok(new_value)
    }

    /// Scale a value by a factor
    pub fn scale_value(
        &mut self,
        series_index: usize,
        category_index: usize,
        factor: f64,
    ) -> ChartResult<f64> {
        let current = self
            .get_value(series_index, category_index)
            .ok_or_else(|| ChartError::InvalidData("Index out of bounds".to_string()))?;

        let new_value = current * factor;
        self.set_value(series_index, category_index, new_value)?;
        Ok(new_value)
    }

    /// Scale all values in a series
    pub fn scale_series(&mut self, series_index: usize, factor: f64) -> ChartResult<()> {
        if series_index >= self.data.series.len() {
            return Err(ChartError::InvalidData(format!(
                "Series index {} out of bounds",
                series_index
            )));
        }

        let new_values: Vec<f64> = self.data.series[series_index]
            .values
            .iter()
            .map(|&v| v * factor)
            .collect();

        // Validate all new values
        for &value in &new_values {
            self.validate_value(value)?;
        }

        self.save_state();
        self.data.series[series_index].values = new_values;
        Ok(())
    }

    // === Bulk Operations ===

    /// Fill a range with a value
    pub fn fill_range(
        &mut self,
        series_start: usize,
        series_end: usize,
        category_start: usize,
        category_end: usize,
        value: f64,
    ) -> ChartResult<()> {
        self.validate_value(value)?;

        if series_end > self.data.series.len() || category_end > self.data.categories.len() {
            return Err(ChartError::InvalidData("Range out of bounds".to_string()));
        }

        self.save_state();

        for series_idx in series_start..series_end {
            for cat_idx in category_start..category_end {
                if cat_idx < self.data.series[series_idx].values.len() {
                    self.data.series[series_idx].values[cat_idx] = value;
                }
            }
        }

        Ok(())
    }

    /// Clear all values (set to zero)
    pub fn clear_values(&mut self) {
        self.save_state();
        for series in &mut self.data.series {
            for value in &mut series.values {
                *value = 0.0;
            }
        }
    }

    /// Clear all data
    pub fn clear_all(&mut self) {
        self.save_state();
        self.data = ChartData::default();
    }

    // === Import/Export ===

    /// Import data from a 2D array (first row = categories, first column = series names)
    pub fn import_from_table(&mut self, table: Vec<Vec<String>>) -> ChartResult<()> {
        if table.is_empty() || table[0].is_empty() {
            return Err(ChartError::InvalidData(
                "Table must have at least one row and column".to_string(),
            ));
        }

        self.save_state();

        // First row (excluding first cell) = categories
        let categories: Vec<String> = table[0][1..].to_vec();

        // Remaining rows = series
        let mut series = Vec::new();
        for row in table.iter().skip(1) {
            if row.is_empty() {
                continue;
            }
            let name = row[0].clone();
            let values: Vec<f64> = row[1..]
                .iter()
                .map(|s| s.parse::<f64>().unwrap_or(0.0))
                .collect();

            // Validate values
            for &value in &values {
                self.validate_value(value)?;
            }

            series.push(DataSeries::new(name, values));
        }

        self.data.categories = categories;
        self.data.series = series;

        Ok(())
    }

    /// Export data to a 2D array (first row = categories, first column = series names)
    pub fn export_to_table(&self) -> Vec<Vec<String>> {
        let mut table = Vec::new();

        // Header row
        let mut header = vec![String::new()]; // Empty cell for top-left
        header.extend(self.data.categories.clone());
        table.push(header);

        // Data rows
        for series in &self.data.series {
            let mut row = vec![series.name.clone()];
            row.extend(series.values.iter().map(|v| v.to_string()));
            table.push(row);
        }

        table
    }

    /// Import data from CSV string
    pub fn import_from_csv(&mut self, csv: &str) -> ChartResult<()> {
        let table: Vec<Vec<String>> = csv
            .lines()
            .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
            .collect();

        self.import_from_table(table)
    }

    /// Export data to CSV string
    pub fn export_to_csv(&self) -> String {
        self.export_to_table()
            .iter()
            .map(|row| row.join(","))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // === Statistics ===

    /// Get statistics for a series
    pub fn series_statistics(&self, series_index: usize) -> Option<SeriesStatistics> {
        let series = self.data.series.get(series_index)?;
        if series.values.is_empty() {
            return None;
        }

        let sum: f64 = series.values.iter().sum();
        let count = series.values.len();
        let mean = sum / count as f64;

        let min = series.values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = series
            .values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        let variance: f64 =
            series.values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(SeriesStatistics {
            count,
            sum,
            mean,
            min,
            max,
            std_dev,
        })
    }
}

/// Statistics for a data series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesStatistics {
    /// Number of values
    pub count: usize,
    /// Sum of values
    pub sum: f64,
    /// Mean (average) value
    pub mean: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Standard deviation
    pub std_dev: f64,
}

/// A cell reference in the chart data grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellRef {
    /// Series index (row in typical spreadsheet view)
    pub series: usize,
    /// Category index (column in typical spreadsheet view)
    pub category: usize,
}

impl CellRef {
    /// Create a new cell reference
    pub fn new(series: usize, category: usize) -> Self {
        Self { series, category }
    }
}

/// A range of cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellRange {
    /// Start cell (inclusive)
    pub start: CellRef,
    /// End cell (inclusive)
    pub end: CellRef,
}

impl CellRange {
    /// Create a new cell range
    pub fn new(start: CellRef, end: CellRef) -> Self {
        Self { start, end }
    }

    /// Create a range from coordinates
    pub fn from_coords(
        start_series: usize,
        start_category: usize,
        end_series: usize,
        end_category: usize,
    ) -> Self {
        Self {
            start: CellRef::new(start_series, start_category),
            end: CellRef::new(end_series, end_category),
        }
    }

    /// Check if a cell is within this range
    pub fn contains(&self, cell: CellRef) -> bool {
        let min_series = self.start.series.min(self.end.series);
        let max_series = self.start.series.max(self.end.series);
        let min_category = self.start.category.min(self.end.category);
        let max_category = self.start.category.max(self.end.category);

        cell.series >= min_series
            && cell.series <= max_series
            && cell.category >= min_category
            && cell.category <= max_category
    }

    /// Iterate over all cells in the range
    pub fn cells(&self) -> impl Iterator<Item = CellRef> {
        let min_series = self.start.series.min(self.end.series);
        let max_series = self.start.series.max(self.end.series);
        let min_category = self.start.category.min(self.end.category);
        let max_category = self.start.category.max(self.end.category);

        (min_series..=max_series).flat_map(move |s| {
            (min_category..=max_category).map(move |c| CellRef::new(s, c))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_editor() -> ChartDataEditor {
        let mut data = ChartData::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        data.series
            .push(DataSeries::new("Series 1", vec![10.0, 20.0, 30.0]));
        data.series
            .push(DataSeries::new("Series 2", vec![15.0, 25.0, 35.0]));
        ChartDataEditor::from_data(data)
    }

    #[test]
    fn test_new_editor() {
        let editor = ChartDataEditor::new();
        assert_eq!(editor.series_count(), 0);
        assert_eq!(editor.category_count(), 0);
    }

    #[test]
    fn test_from_data() {
        let editor = create_test_editor();
        assert_eq!(editor.series_count(), 2);
        assert_eq!(editor.category_count(), 3);
    }

    #[test]
    fn test_add_series() {
        let mut editor = ChartDataEditor::new();
        editor.set_categories(vec!["A".to_string(), "B".to_string()]);

        let idx = editor
            .add_series(DataSeries::new("Test", vec![1.0, 2.0]))
            .unwrap();

        assert_eq!(idx, 0);
        assert_eq!(editor.series_count(), 1);
        assert_eq!(editor.get_series(0).unwrap().name, "Test");
    }

    #[test]
    fn test_add_empty_series() {
        let mut editor = create_test_editor();
        let idx = editor.add_empty_series("New Series");

        assert_eq!(idx, 2);
        assert_eq!(editor.get_series(2).unwrap().values.len(), 3);
    }

    #[test]
    fn test_remove_series() {
        let mut editor = create_test_editor();
        let removed = editor.remove_series(0).unwrap();

        assert_eq!(removed.name, "Series 1");
        assert_eq!(editor.series_count(), 1);
        assert_eq!(editor.get_series(0).unwrap().name, "Series 2");
    }

    #[test]
    fn test_remove_series_out_of_bounds() {
        let mut editor = create_test_editor();
        assert!(editor.remove_series(10).is_err());
    }

    #[test]
    fn test_add_category() {
        let mut editor = create_test_editor();
        let idx = editor.add_category("D");

        assert_eq!(idx, 3);
        assert_eq!(editor.category_count(), 4);
        // Check that all series got a new value
        assert_eq!(editor.get_series(0).unwrap().values.len(), 4);
        assert_eq!(editor.get_series(1).unwrap().values.len(), 4);
    }

    #[test]
    fn test_insert_category() {
        let mut editor = create_test_editor();
        editor.insert_category(1, "X").unwrap();

        assert_eq!(editor.category_count(), 4);
        assert_eq!(editor.get_category(1), Some("X"));
        assert_eq!(editor.get_category(2), Some("B"));
    }

    #[test]
    fn test_remove_category() {
        let mut editor = create_test_editor();
        let removed = editor.remove_category(1).unwrap();

        assert_eq!(removed, "B");
        assert_eq!(editor.category_count(), 2);
        assert_eq!(editor.get_series(0).unwrap().values, vec![10.0, 30.0]);
    }

    #[test]
    fn test_set_value() {
        let mut editor = create_test_editor();
        editor.set_value(0, 1, 99.0).unwrap();

        assert_eq!(editor.get_value(0, 1), Some(99.0));
    }

    #[test]
    fn test_set_value_validation() {
        let mut editor = create_test_editor().with_value_range(0.0, 100.0);

        assert!(editor.set_value(0, 0, 50.0).is_ok());
        assert!(editor.set_value(0, 0, -10.0).is_err());
        assert!(editor.set_value(0, 0, 150.0).is_err());
    }

    #[test]
    fn test_set_value_nan_validation() {
        let mut editor = create_test_editor();
        assert!(editor.set_value(0, 0, f64::NAN).is_err());
        assert!(editor.set_value(0, 0, f64::INFINITY).is_err());
    }

    #[test]
    fn test_disable_strict_validation() {
        let mut editor = create_test_editor().with_strict_validation(false);
        assert!(editor.set_value(0, 0, f64::NAN).is_ok());
    }

    #[test]
    fn test_increment_value() {
        let mut editor = create_test_editor();
        let new_value = editor.increment_value(0, 0, 5.0).unwrap();

        assert_eq!(new_value, 15.0);
        assert_eq!(editor.get_value(0, 0), Some(15.0));
    }

    #[test]
    fn test_scale_value() {
        let mut editor = create_test_editor();
        let new_value = editor.scale_value(0, 0, 2.0).unwrap();

        assert_eq!(new_value, 20.0);
    }

    #[test]
    fn test_scale_series() {
        let mut editor = create_test_editor();
        editor.scale_series(0, 2.0).unwrap();

        assert_eq!(editor.get_series(0).unwrap().values, vec![20.0, 40.0, 60.0]);
    }

    #[test]
    fn test_undo_redo() {
        let mut editor = create_test_editor();
        let original_value = editor.get_value(0, 0).unwrap();

        editor.set_value(0, 0, 99.0).unwrap();
        assert_eq!(editor.get_value(0, 0), Some(99.0));
        assert!(editor.can_undo());

        editor.undo();
        assert_eq!(editor.get_value(0, 0), Some(original_value));
        assert!(editor.can_redo());

        editor.redo();
        assert_eq!(editor.get_value(0, 0), Some(99.0));
    }

    #[test]
    fn test_move_series() {
        let mut editor = create_test_editor();
        editor.move_series(0, 1).unwrap();

        assert_eq!(editor.get_series(0).unwrap().name, "Series 2");
        assert_eq!(editor.get_series(1).unwrap().name, "Series 1");
    }

    #[test]
    fn test_move_category() {
        let mut editor = create_test_editor();
        editor.move_category(0, 2).unwrap();

        assert_eq!(editor.get_category(0), Some("B"));
        assert_eq!(editor.get_category(2), Some("A"));
        // Check that values moved correctly
        assert_eq!(
            editor.get_series(0).unwrap().values,
            vec![20.0, 30.0, 10.0]
        );
    }

    #[test]
    fn test_duplicate_series() {
        let mut editor = create_test_editor();
        let new_idx = editor.duplicate_series(0).unwrap();

        assert_eq!(editor.series_count(), 3);
        assert_eq!(
            editor.get_series(new_idx).unwrap().name,
            "Series 1 (copy)"
        );
        assert_eq!(
            editor.get_series(new_idx).unwrap().values,
            editor.get_series(0).unwrap().values
        );
    }

    #[test]
    fn test_fill_range() {
        let mut editor = create_test_editor();
        editor.fill_range(0, 2, 0, 2, 100.0).unwrap();

        assert_eq!(editor.get_value(0, 0), Some(100.0));
        assert_eq!(editor.get_value(0, 1), Some(100.0));
        assert_eq!(editor.get_value(1, 0), Some(100.0));
        assert_eq!(editor.get_value(1, 1), Some(100.0));
        // Unchanged
        assert_eq!(editor.get_value(0, 2), Some(30.0));
    }

    #[test]
    fn test_clear_values() {
        let mut editor = create_test_editor();
        editor.clear_values();

        for s in 0..editor.series_count() {
            for c in 0..editor.category_count() {
                assert_eq!(editor.get_value(s, c), Some(0.0));
            }
        }
    }

    #[test]
    fn test_import_export_table() {
        let mut editor = ChartDataEditor::new();
        let table = vec![
            vec!["".to_string(), "Q1".to_string(), "Q2".to_string()],
            vec!["Sales".to_string(), "100".to_string(), "150".to_string()],
            vec!["Costs".to_string(), "80".to_string(), "90".to_string()],
        ];

        editor.import_from_table(table).unwrap();

        assert_eq!(editor.category_count(), 2);
        assert_eq!(editor.series_count(), 2);
        assert_eq!(editor.get_category(0), Some("Q1"));
        assert_eq!(editor.get_series(0).unwrap().name, "Sales");
        assert_eq!(editor.get_value(0, 0), Some(100.0));
    }

    #[test]
    fn test_import_export_csv() {
        let mut editor = ChartDataEditor::new();
        let csv = ",A,B,C\nX,1,2,3\nY,4,5,6";

        editor.import_from_csv(csv).unwrap();

        assert_eq!(editor.category_count(), 3);
        assert_eq!(editor.series_count(), 2);

        let exported = editor.export_to_csv();
        assert!(exported.contains("A,B,C"));
        assert!(exported.contains("X,1,2,3"));
    }

    #[test]
    fn test_series_statistics() {
        let editor = create_test_editor();
        let stats = editor.series_statistics(0).unwrap();

        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_cell_range_contains() {
        let range = CellRange::from_coords(0, 0, 2, 2);

        assert!(range.contains(CellRef::new(1, 1)));
        assert!(range.contains(CellRef::new(0, 0)));
        assert!(range.contains(CellRef::new(2, 2)));
        assert!(!range.contains(CellRef::new(3, 1)));
    }

    #[test]
    fn test_cell_range_cells() {
        let range = CellRange::from_coords(0, 0, 1, 1);
        let cells: Vec<_> = range.cells().collect();

        assert_eq!(cells.len(), 4);
        assert!(cells.contains(&CellRef::new(0, 0)));
        assert!(cells.contains(&CellRef::new(0, 1)));
        assert!(cells.contains(&CellRef::new(1, 0)));
        assert!(cells.contains(&CellRef::new(1, 1)));
    }

    #[test]
    fn test_into_data() {
        let editor = create_test_editor();
        let data = editor.into_data();

        assert_eq!(data.series.len(), 2);
        assert_eq!(data.categories.len(), 3);
    }

    #[test]
    fn test_rename_series() {
        let mut editor = create_test_editor();
        editor.rename_series(0, "Renamed").unwrap();

        assert_eq!(editor.get_series(0).unwrap().name, "Renamed");
    }

    #[test]
    fn test_rename_category() {
        let mut editor = create_test_editor();
        editor.rename_category(0, "Renamed").unwrap();

        assert_eq!(editor.get_category(0), Some("Renamed"));
    }
}
