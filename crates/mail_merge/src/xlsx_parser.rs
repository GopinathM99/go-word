//! XLSX parser for mail merge data sources

use std::collections::HashSet;
use std::io::{Read, Seek};
use std::path::Path;

use calamine::{open_workbook, Data, Range, Reader, Xlsx};
use chrono::NaiveDate;

use crate::data_source::{ColumnDef, DataSource, DataSourceType, DataType, Record, Value};
use crate::error::{MailMergeError, Result};

/// Selector for which sheet to read from an Excel workbook
#[derive(Debug, Clone)]
pub enum SheetSelector {
    /// Select sheet by name
    ByName(String),
    /// Select sheet by index (0-based)
    ByIndex(usize),
    /// Select the first sheet
    First,
}

impl Default for SheetSelector {
    fn default() -> Self {
        SheetSelector::First
    }
}

/// Range of cells to read from a sheet
#[derive(Debug, Clone)]
pub struct CellRange {
    /// Starting row (0-based, inclusive)
    pub start_row: u32,
    /// Starting column (0-based, inclusive)
    pub start_col: u32,
    /// Ending row (0-based, inclusive), None means read to the end
    pub end_row: Option<u32>,
    /// Ending column (0-based, inclusive), None means read to the end
    pub end_col: Option<u32>,
}

impl CellRange {
    /// Create a new cell range
    pub fn new(start_row: u32, start_col: u32, end_row: Option<u32>, end_col: Option<u32>) -> Self {
        Self {
            start_row,
            start_col,
            end_row,
            end_col,
        }
    }

    /// Create a range starting from a specific cell to the end
    pub fn from(start_row: u32, start_col: u32) -> Self {
        Self {
            start_row,
            start_col,
            end_row: None,
            end_col: None,
        }
    }
}

/// XLSX parser configuration
#[derive(Debug, Clone)]
pub struct XlsxConfig {
    /// Sheet name or index to read from
    pub sheet: SheetSelector,
    /// Whether the first row contains headers
    pub has_header: bool,
    /// Range of cells to read (optional)
    pub range: Option<CellRange>,
    /// Whether to auto-detect data types
    pub auto_detect_types: bool,
    /// Whether to trim whitespace from string values
    pub trim_whitespace: bool,
    /// Skip empty rows
    pub skip_empty_rows: bool,
}

impl Default for XlsxConfig {
    fn default() -> Self {
        Self {
            sheet: SheetSelector::First,
            has_header: true,
            range: None,
            auto_detect_types: true,
            trim_whitespace: true,
            skip_empty_rows: true,
        }
    }
}

impl XlsxConfig {
    /// Create a new XLSX config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sheet to read by name
    pub fn with_sheet_name(mut self, name: impl Into<String>) -> Self {
        self.sheet = SheetSelector::ByName(name.into());
        self
    }

    /// Set the sheet to read by index (0-based)
    pub fn with_sheet_index(mut self, index: usize) -> Self {
        self.sheet = SheetSelector::ByIndex(index);
        self
    }

    /// Set whether the first row contains headers
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Set the cell range to read
    pub fn with_range(mut self, range: CellRange) -> Self {
        self.range = Some(range);
        self
    }

    /// Set whether to auto-detect data types
    pub fn with_auto_detect(mut self, auto_detect: bool) -> Self {
        self.auto_detect_types = auto_detect;
        self
    }

    /// Set whether to trim whitespace
    pub fn with_trim(mut self, trim: bool) -> Self {
        self.trim_whitespace = trim;
        self
    }

    /// Set whether to skip empty rows
    pub fn with_skip_empty_rows(mut self, skip: bool) -> Self {
        self.skip_empty_rows = skip;
        self
    }
}

/// XLSX parser for creating data sources from Excel files
pub struct XlsxParser {
    config: XlsxConfig,
}

impl XlsxParser {
    /// Create a new XLSX parser with default configuration
    pub fn new() -> Self {
        Self {
            config: XlsxConfig::default(),
        }
    }

    /// Create a new XLSX parser with custom configuration
    pub fn with_config(config: XlsxConfig) -> Self {
        Self { config }
    }

    /// Parse an XLSX file and return a DataSource
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<DataSource> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(MailMergeError::FileNotFound(path.display().to_string()));
        }

        let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| {
            MailMergeError::XlsxParse(format!("Failed to open workbook: {}", e))
        })?;

        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("xlsx_source")
            .to_string();

        let sheet_name = self.get_sheet_name(&workbook)?;

        let source_type = DataSourceType::Xlsx {
            path: path.display().to_string(),
            sheet: sheet_name.clone(),
        };

        self.parse_workbook(&mut workbook, &sheet_name, id, source_type)
    }

    /// Parse XLSX from bytes and return a DataSource
    pub fn parse_bytes(&self, data: &[u8], id: impl Into<String>) -> Result<DataSource> {
        let cursor = std::io::Cursor::new(data);
        let mut workbook: Xlsx<_> = Xlsx::new(cursor).map_err(|e| {
            MailMergeError::XlsxParse(format!("Failed to read workbook from bytes: {}", e))
        })?;

        let sheet_name = self.get_sheet_name(&workbook)?;

        let source_type = DataSourceType::Inline { data: Vec::new() };

        self.parse_workbook(&mut workbook, &sheet_name, id.into(), source_type)
    }

    /// Get the sheet name based on the selector
    fn get_sheet_name<RS: Read + Seek>(&self, workbook: &Xlsx<RS>) -> Result<String> {
        let sheet_names = workbook.sheet_names();

        if sheet_names.is_empty() {
            return Err(MailMergeError::EmptyDataSource(
                "Workbook has no sheets".to_string(),
            ));
        }

        match &self.config.sheet {
            SheetSelector::ByName(name) => {
                if sheet_names.contains(name) {
                    Ok(name.clone())
                } else {
                    Err(MailMergeError::XlsxParse(format!(
                        "Sheet '{}' not found. Available sheets: {:?}",
                        name, sheet_names
                    )))
                }
            }
            SheetSelector::ByIndex(index) => {
                sheet_names.get(*index).cloned().ok_or_else(|| {
                    MailMergeError::XlsxParse(format!(
                        "Sheet index {} out of range. Workbook has {} sheets",
                        index,
                        sheet_names.len()
                    ))
                })
            }
            SheetSelector::First => Ok(sheet_names[0].clone()),
        }
    }

    /// Parse a workbook into a DataSource
    fn parse_workbook<RS: Read + Seek>(
        &self,
        workbook: &mut Xlsx<RS>,
        sheet_name: &str,
        id: String,
        source_type: DataSourceType,
    ) -> Result<DataSource> {
        let range = workbook.worksheet_range(sheet_name).map_err(|e| {
            MailMergeError::XlsxParse(format!("Failed to read sheet '{}': {}", sheet_name, e))
        })?;

        self.parse_range(&range, id, source_type)
    }

    /// Parse a range of cells into a DataSource
    fn parse_range(
        &self,
        range: &Range<Data>,
        id: String,
        source_type: DataSourceType,
    ) -> Result<DataSource> {
        let (start_row, start_col, end_row, end_col) = if let Some(ref cell_range) = self.config.range {
            (
                cell_range.start_row as usize,
                cell_range.start_col as usize,
                cell_range.end_row.map(|r| r as usize).unwrap_or_else(|| range.height().saturating_sub(1)),
                cell_range.end_col.map(|c| c as usize).unwrap_or_else(|| range.width().saturating_sub(1)),
            )
        } else {
            (0, 0, range.height().saturating_sub(1), range.width().saturating_sub(1))
        };

        // Check for empty range
        if range.is_empty() {
            return Err(MailMergeError::EmptyDataSource(
                "Excel sheet is empty".to_string(),
            ));
        }

        let mut data_source = DataSource::new(id, source_type);

        // Get headers
        let headers: Vec<String> = if self.config.has_header {
            self.extract_headers(range, start_row, start_col, end_col)
        } else {
            // Generate column names
            (0..=(end_col - start_col))
                .map(|i| format!("Column{}", i + 1))
                .collect()
        };

        // Check for duplicate headers
        let mut seen = HashSet::new();
        for header in &headers {
            if !seen.insert(header.clone()) {
                return Err(MailMergeError::DuplicateColumn(header.clone()));
            }
        }

        // Collect all data rows for type detection
        let data_start_row = if self.config.has_header {
            start_row + 1
        } else {
            start_row
        };

        let mut raw_rows: Vec<Vec<Data>> = Vec::new();
        for row_idx in data_start_row..=end_row {
            let row: Vec<Data> = (start_col..=end_col)
                .map(|col_idx| {
                    range
                        .get((row_idx, col_idx))
                        .cloned()
                        .unwrap_or(Data::Empty)
                })
                .collect();

            // Skip empty rows if configured
            if self.config.skip_empty_rows && row.iter().all(|cell| matches!(cell, Data::Empty)) {
                continue;
            }

            raw_rows.push(row);
        }

        if raw_rows.is_empty() && headers.is_empty() {
            return Err(MailMergeError::EmptyDataSource(
                "Excel sheet has no data".to_string(),
            ));
        }

        // Detect column types
        let column_types = if self.config.auto_detect_types {
            detect_column_types(&headers, &raw_rows)
        } else {
            vec![DataType::Text; headers.len()]
        };

        // Add column definitions
        for (i, header) in headers.iter().enumerate() {
            let data_type = column_types.get(i).copied().unwrap_or(DataType::Text);
            data_source.add_column(ColumnDef::new(header.clone(), data_type));
        }

        // Convert rows to records
        for raw_row in raw_rows {
            let mut record = Record::new();
            for (i, cell) in raw_row.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    let value = self.cell_to_value(cell);
                    record.insert(header.clone(), value);
                }
            }
            data_source.add_record(record);
        }

        Ok(data_source)
    }

    /// Extract headers from the first row
    fn extract_headers(
        &self,
        range: &Range<Data>,
        row_idx: usize,
        start_col: usize,
        end_col: usize,
    ) -> Vec<String> {
        (start_col..=end_col)
            .enumerate()
            .map(|(i, col_idx)| {
                let cell = range.get((row_idx, col_idx));
                match cell {
                    Some(Data::String(s)) => {
                        let header = if self.config.trim_whitespace {
                            s.trim().to_string()
                        } else {
                            s.clone()
                        };
                        if header.is_empty() {
                            format!("Column{}", i + 1)
                        } else {
                            header
                        }
                    }
                    Some(Data::Int(n)) => n.to_string(),
                    Some(Data::Float(n)) => format_float(*n),
                    Some(Data::Bool(b)) => b.to_string(),
                    Some(Data::DateTime(dt)) => format_excel_datetime(dt.as_f64()),
                    Some(Data::DateTimeIso(s)) => s.clone(),
                    Some(Data::DurationIso(s)) => s.clone(),
                    Some(Data::Error(e)) => format!("#ERROR:{:?}", e),
                    Some(Data::Empty) | None => format!("Column{}", i + 1),
                }
            })
            .collect()
    }

    /// Convert an Excel cell to our Value type
    fn cell_to_value(&self, cell: &Data) -> Value {
        match cell {
            Data::Empty => Value::Null,
            Data::String(s) => {
                let text = if self.config.trim_whitespace {
                    s.trim().to_string()
                } else {
                    s.clone()
                };

                // If auto-detect is enabled, try to parse the string
                if self.config.auto_detect_types {
                    // Check for null-like values
                    let lower = text.to_lowercase();
                    if text.is_empty()
                        || lower == "null"
                        || lower == "na"
                        || lower == "n/a"
                        || lower == "#n/a"
                    {
                        return Value::Null;
                    }
                    Value::Text(text)
                } else {
                    Value::Text(text)
                }
            }
            Data::Int(n) => Value::Number(*n as f64),
            Data::Float(n) => {
                // Check if it's actually a date (Excel stores dates as floats)
                // Excel dates are days since 1900-01-01 (or 1904-01-01 on Mac)
                // Typical date range is 1 to ~50000+
                // We won't auto-convert floats to dates here as it could be misleading
                Value::Number(*n)
            }
            Data::Bool(b) => Value::Boolean(*b),
            Data::DateTime(dt) => {
                // Convert Excel datetime to NaiveDate
                let dt_value = dt.as_f64();
                if let Some(date) = excel_datetime_to_date(dt_value) {
                    Value::Date(date)
                } else {
                    // If conversion fails, store as number
                    Value::Number(dt_value)
                }
            }
            Data::DateTimeIso(s) => {
                // Try to parse ISO datetime string
                if let Some(date) = try_parse_iso_date(s) {
                    Value::Date(date)
                } else {
                    Value::Text(s.clone())
                }
            }
            Data::DurationIso(s) => {
                // Store duration as text
                Value::Text(s.clone())
            }
            Data::Error(e) => {
                // Convert Excel error to null with a note
                Value::Text(format!("#ERROR:{:?}", e))
            }
        }
    }
}

impl Default for XlsxParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect column types from raw data
fn detect_column_types(headers: &[String], rows: &[Vec<Data>]) -> Vec<DataType> {
    headers
        .iter()
        .enumerate()
        .map(|(col_idx, _)| detect_column_type(rows, col_idx))
        .collect()
}

/// Detect the data type for a single column
fn detect_column_type(rows: &[Vec<Data>], col_idx: usize) -> DataType {
    let mut has_number = false;
    let mut has_date = false;
    let mut has_boolean = false;
    let mut has_text = false;
    let mut all_empty = true;

    for row in rows {
        if let Some(cell) = row.get(col_idx) {
            match cell {
                Data::Empty => {}
                Data::String(s) => {
                    let trimmed = s.trim();
                    if !trimmed.is_empty()
                        && !trimmed.eq_ignore_ascii_case("null")
                        && !trimmed.eq_ignore_ascii_case("na")
                        && !trimmed.eq_ignore_ascii_case("n/a")
                    {
                        all_empty = false;
                        has_text = true;
                    }
                }
                Data::Int(_) | Data::Float(_) => {
                    all_empty = false;
                    has_number = true;
                }
                Data::Bool(_) => {
                    all_empty = false;
                    has_boolean = true;
                }
                Data::DateTime(_) | Data::DateTimeIso(_) => {
                    all_empty = false;
                    has_date = true;
                }
                Data::DurationIso(_) => {
                    all_empty = false;
                    has_text = true;
                }
                Data::Error(_) => {
                    all_empty = false;
                    has_text = true;
                }
            }
        }
    }

    // Determine the best type
    if all_empty {
        DataType::Text
    } else if has_text {
        DataType::Text
    } else if has_date && !has_number && !has_boolean {
        DataType::Date
    } else if has_boolean && !has_number && !has_date {
        DataType::Boolean
    } else if has_number && !has_date && !has_boolean {
        DataType::Number
    } else {
        // Mixed types default to text
        DataType::Text
    }
}

/// Convert Excel datetime (days since 1900-01-01) to NaiveDate
fn excel_datetime_to_date(excel_date: f64) -> Option<NaiveDate> {
    // Excel uses a different epoch: 1899-12-30 (day 0)
    // Also, Excel incorrectly considers 1900 a leap year (bug for Lotus 1-2-3 compatibility)
    let days = excel_date.floor() as i64;

    // Adjust for Excel's leap year bug (dates >= 60 need adjustment)
    let adjusted_days = if days >= 60 { days - 1 } else { days };

    // Excel epoch is 1899-12-30
    let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    excel_epoch.checked_add_signed(chrono::Duration::days(adjusted_days))
}

/// Try to parse an ISO date string
fn try_parse_iso_date(s: &str) -> Option<NaiveDate> {
    // Try common ISO formats
    let formats = [
        "%Y-%m-%d",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.fZ",
    ];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, format) {
            return Some(date);
        }
        // Try parsing as datetime and extracting date
        if let Ok(datetime) = chrono::NaiveDateTime::parse_from_str(s, format) {
            return Some(datetime.date());
        }
    }

    // Try just the date part if there's a T separator
    if let Some(date_part) = s.split('T').next() {
        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            return Some(date);
        }
    }

    None
}

/// Format a float without unnecessary decimal places
fn format_float(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Format an Excel datetime for display
fn format_excel_datetime(dt: f64) -> String {
    if let Some(date) = excel_datetime_to_date(dt) {
        date.format("%Y-%m-%d").to_string()
    } else {
        format_float(dt)
    }
}

/// Get list of sheet names from an Excel file
pub fn get_sheet_names(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(MailMergeError::FileNotFound(path.display().to_string()));
    }

    let workbook: Xlsx<_> = open_workbook(path).map_err(|e| {
        MailMergeError::XlsxParse(format!("Failed to open workbook: {}", e))
    })?;

    Ok(workbook.sheet_names().to_vec())
}

/// Get list of sheet names from Excel file bytes
pub fn get_sheet_names_from_bytes(data: &[u8]) -> Result<Vec<String>> {
    let cursor = std::io::Cursor::new(data);
    let workbook: Xlsx<_> = Xlsx::new(cursor).map_err(|e| {
        MailMergeError::XlsxParse(format!("Failed to read workbook from bytes: {}", e))
    })?;

    Ok(workbook.sheet_names().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{ExcelDateTime, ExcelDateTimeType};
    use chrono::Datelike;

    // Helper to create an ExcelDateTime from a float value for testing
    fn make_excel_datetime(value: f64) -> ExcelDateTime {
        ExcelDateTime::new(value, ExcelDateTimeType::DateTime, false)
    }

    // Helper to create a simple test XLSX file
    // Note: For actual testing, we would need to use a library to create XLSX files
    // or include test fixtures. These tests demonstrate the API usage.

    #[test]
    fn test_xlsx_config_default() {
        let config = XlsxConfig::default();
        assert!(config.has_header);
        assert!(config.auto_detect_types);
        assert!(config.trim_whitespace);
        assert!(config.skip_empty_rows);
        assert!(matches!(config.sheet, SheetSelector::First));
    }

    #[test]
    fn test_xlsx_config_builder() {
        let config = XlsxConfig::new()
            .with_sheet_name("Data")
            .with_header(false)
            .with_auto_detect(false)
            .with_trim(false)
            .with_skip_empty_rows(false);

        assert!(!config.has_header);
        assert!(!config.auto_detect_types);
        assert!(!config.trim_whitespace);
        assert!(!config.skip_empty_rows);
        assert!(matches!(config.sheet, SheetSelector::ByName(ref name) if name == "Data"));
    }

    #[test]
    fn test_xlsx_config_with_sheet_index() {
        let config = XlsxConfig::new().with_sheet_index(2);
        assert!(matches!(config.sheet, SheetSelector::ByIndex(2)));
    }

    #[test]
    fn test_xlsx_config_with_range() {
        let config = XlsxConfig::new().with_range(CellRange::new(1, 0, Some(10), Some(5)));
        assert!(config.range.is_some());
        let range = config.range.unwrap();
        assert_eq!(range.start_row, 1);
        assert_eq!(range.start_col, 0);
        assert_eq!(range.end_row, Some(10));
        assert_eq!(range.end_col, Some(5));
    }

    #[test]
    fn test_cell_range_from() {
        let range = CellRange::from(5, 2);
        assert_eq!(range.start_row, 5);
        assert_eq!(range.start_col, 2);
        assert!(range.end_row.is_none());
        assert!(range.end_col.is_none());
    }

    #[test]
    fn test_excel_datetime_to_date() {
        // Test known Excel dates
        // January 1, 2024 is Excel date 45293
        let date = excel_datetime_to_date(45293.0);
        assert!(date.is_some());
        let date = date.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);

        // Test epoch date (day 1 = 1899-12-31 due to Excel's leap year bug)
        let date = excel_datetime_to_date(1.0);
        assert!(date.is_some());
        let date = date.unwrap();
        assert_eq!(date.year(), 1899);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 31);
    }

    #[test]
    fn test_try_parse_iso_date() {
        let date = try_parse_iso_date("2024-01-15");
        assert!(date.is_some());
        let date = date.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);

        let date = try_parse_iso_date("2024-01-15T10:30:00");
        assert!(date.is_some());

        let date = try_parse_iso_date("invalid");
        assert!(date.is_none());
    }

    #[test]
    fn test_format_float() {
        assert_eq!(format_float(42.0), "42");
        assert_eq!(format_float(3.14), "3.14");
        assert_eq!(format_float(-5.0), "-5");
    }

    #[test]
    fn test_parser_file_not_found() {
        let parser = XlsxParser::new();
        let result = parser.parse_file("/nonexistent/file.xlsx");
        assert!(matches!(result, Err(MailMergeError::FileNotFound(_))));
    }

    #[test]
    fn test_sheet_selector_default() {
        let selector = SheetSelector::default();
        assert!(matches!(selector, SheetSelector::First));
    }

    #[test]
    fn test_cell_to_value_empty() {
        let parser = XlsxParser::new();
        let value = parser.cell_to_value(&Data::Empty);
        assert!(matches!(value, Value::Null));
    }

    #[test]
    fn test_cell_to_value_string() {
        let parser = XlsxParser::new();

        let value = parser.cell_to_value(&Data::String("Hello".to_string()));
        assert!(matches!(value, Value::Text(ref s) if s == "Hello"));

        let value = parser.cell_to_value(&Data::String("  trimmed  ".to_string()));
        assert!(matches!(value, Value::Text(ref s) if s == "trimmed"));

        let value = parser.cell_to_value(&Data::String("null".to_string()));
        assert!(matches!(value, Value::Null));

        let value = parser.cell_to_value(&Data::String("N/A".to_string()));
        assert!(matches!(value, Value::Null));
    }

    #[test]
    fn test_cell_to_value_numbers() {
        let parser = XlsxParser::new();

        let value = parser.cell_to_value(&Data::Int(42));
        assert!(matches!(value, Value::Number(n) if n == 42.0));

        let value = parser.cell_to_value(&Data::Float(3.14));
        assert!(matches!(value, Value::Number(n) if (n - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_cell_to_value_boolean() {
        let parser = XlsxParser::new();

        let value = parser.cell_to_value(&Data::Bool(true));
        assert!(matches!(value, Value::Boolean(true)));

        let value = parser.cell_to_value(&Data::Bool(false));
        assert!(matches!(value, Value::Boolean(false)));
    }

    #[test]
    fn test_cell_to_value_datetime() {
        let parser = XlsxParser::new();

        // Excel date for 2024-01-01
        let excel_dt = make_excel_datetime(45292.0);
        let value = parser.cell_to_value(&Data::DateTime(excel_dt));
        assert!(matches!(value, Value::Date(_)));
    }

    #[test]
    fn test_cell_to_value_iso_datetime() {
        let parser = XlsxParser::new();

        let value = parser.cell_to_value(&Data::DateTimeIso("2024-01-15".to_string()));
        assert!(matches!(value, Value::Date(_)));

        let value = parser.cell_to_value(&Data::DateTimeIso("invalid".to_string()));
        assert!(matches!(value, Value::Text(_)));
    }

    #[test]
    fn test_detect_column_type_empty() {
        let rows: Vec<Vec<Data>> = vec![
            vec![Data::Empty],
            vec![Data::Empty],
        ];
        let data_type = detect_column_type(&rows, 0);
        assert_eq!(data_type, DataType::Text);
    }

    #[test]
    fn test_detect_column_type_number() {
        let rows: Vec<Vec<Data>> = vec![
            vec![Data::Int(1)],
            vec![Data::Float(2.5)],
            vec![Data::Int(3)],
        ];
        let data_type = detect_column_type(&rows, 0);
        assert_eq!(data_type, DataType::Number);
    }

    #[test]
    fn test_detect_column_type_boolean() {
        let rows: Vec<Vec<Data>> = vec![
            vec![Data::Bool(true)],
            vec![Data::Bool(false)],
            vec![Data::Empty],
        ];
        let data_type = detect_column_type(&rows, 0);
        assert_eq!(data_type, DataType::Boolean);
    }

    #[test]
    fn test_detect_column_type_date() {
        let rows: Vec<Vec<Data>> = vec![
            vec![Data::DateTime(make_excel_datetime(45292.0))],
            vec![Data::DateTimeIso("2024-01-15".to_string())],
        ];
        let data_type = detect_column_type(&rows, 0);
        assert_eq!(data_type, DataType::Date);
    }

    #[test]
    fn test_detect_column_type_mixed() {
        let rows: Vec<Vec<Data>> = vec![
            vec![Data::Int(1)],
            vec![Data::String("text".to_string())],
            vec![Data::Bool(true)],
        ];
        let data_type = detect_column_type(&rows, 0);
        // Mixed types default to text
        assert_eq!(data_type, DataType::Text);
    }

    #[test]
    fn test_xlsx_parser_default() {
        let parser = XlsxParser::default();
        assert!(parser.config.has_header);
    }
}
