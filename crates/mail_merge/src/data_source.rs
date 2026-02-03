//! Data source types for mail merge

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// A data source for mail merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Unique identifier for this data source
    pub id: String,
    /// Type of data source (CSV, JSON, inline)
    pub source_type: DataSourceType,
    /// Column definitions
    pub columns: Vec<ColumnDef>,
    /// Data records
    pub records: Vec<Record>,
}

impl DataSource {
    /// Create a new data source with the given ID and source type
    pub fn new(id: impl Into<String>, source_type: DataSourceType) -> Self {
        Self {
            id: id.into(),
            source_type,
            columns: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Create a new inline data source
    pub fn inline(id: impl Into<String>) -> Self {
        Self::new(id, DataSourceType::Inline { data: Vec::new() })
    }

    /// Add a column definition
    pub fn add_column(&mut self, column: ColumnDef) {
        self.columns.push(column);
    }

    /// Add a record
    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    /// Get the number of records
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get column names
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// Get a record by index
    pub fn get_record(&self, index: usize) -> Option<&Record> {
        self.records.get(index)
    }

    /// Get a preview of records (first N records)
    pub fn preview(&self, limit: usize) -> Vec<&Record> {
        self.records.iter().take(limit).collect()
    }

    /// Check if a column exists
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.iter().any(|c| c.name == name)
    }

    /// Get column definition by name
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Get value from a specific record and column
    pub fn get_value(&self, record_index: usize, column_name: &str) -> Option<&Value> {
        self.records.get(record_index).and_then(|r| r.get(column_name))
    }
}

/// Type of data source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataSourceType {
    /// CSV file data source
    Csv {
        /// Path to the CSV file
        path: String,
        /// Delimiter character (comma, semicolon, tab)
        delimiter: char,
        /// Whether the first row contains headers
        has_header: bool,
    },
    /// JSON file data source
    Json {
        /// Path to the JSON file
        path: String,
        /// Optional root path to the data array (e.g., "data.customers")
        root_path: Option<String>,
    },
    /// XLSX/XLS file data source
    Xlsx {
        /// Path to the Excel file
        path: String,
        /// Name of the sheet being used
        sheet: String,
    },
    /// Inline data (manually provided records)
    Inline {
        /// Inline data records
        data: Vec<Record>,
    },
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// Column name (used for field mapping)
    pub name: String,
    /// Data type of the column
    pub data_type: DataType,
    /// Optional display name
    pub display_name: Option<String>,
    /// Optional description
    pub description: Option<String>,
}

impl ColumnDef {
    /// Create a new column definition
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            display_name: None,
            description: None,
        }
    }

    /// Set a display name
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Set a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get the display name, falling back to the column name
    pub fn display(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// Data type for column values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    /// Text/string value
    Text,
    /// Numeric value (floating point)
    Number,
    /// Date value
    Date,
    /// Boolean value
    Boolean,
}

impl DataType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::Text => "text",
            DataType::Number => "number",
            DataType::Date => "date",
            DataType::Boolean => "boolean",
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single record (row) of data
pub type Record = HashMap<String, Value>;

/// A value in a record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Text/string value
    Text(String),
    /// Numeric value
    Number(f64),
    /// Date value
    Date(NaiveDate),
    /// Boolean value
    Boolean(bool),
    /// Null/missing value
    Null,
}

impl Value {
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Get the data type of this value
    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Value::Text(_) => Some(DataType::Text),
            Value::Number(_) => Some(DataType::Number),
            Value::Date(_) => Some(DataType::Date),
            Value::Boolean(_) => Some(DataType::Boolean),
            Value::Null => None,
        }
    }

    /// Convert to string representation
    pub fn to_string_value(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            Value::Number(n) => {
                // Format integers without decimal places
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Date(d) => d.format("%Y-%m-%d").to_string(),
            Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Null => String::new(),
        }
    }

    /// Try to get as text
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get as date
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            Value::Date(d) => Some(*d),
            _ => None,
        }
    }

    /// Try to get as boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Parse a string value with automatic type detection
    pub fn parse_auto(s: &str) -> Value {
        let trimmed = s.trim();

        // Check for empty/null
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") || trimmed.eq_ignore_ascii_case("na") {
            return Value::Null;
        }

        // Check for boolean
        if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("yes") || trimmed == "1" {
            return Value::Boolean(true);
        }
        if trimmed.eq_ignore_ascii_case("false") || trimmed.eq_ignore_ascii_case("no") || trimmed == "0" {
            return Value::Boolean(false);
        }

        // Check for date (common formats)
        if let Some(date) = try_parse_date(trimmed) {
            return Value::Date(date);
        }

        // Check for number
        if let Ok(n) = trimmed.parse::<f64>() {
            return Value::Number(n);
        }

        // Default to text
        Value::Text(s.to_string())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_value())
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n as f64)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<NaiveDate> for Value {
    fn from(d: NaiveDate) -> Self {
        Value::Date(d)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

/// Try to parse a date from various common formats
fn try_parse_date(s: &str) -> Option<NaiveDate> {
    // Common date formats
    let formats = [
        "%Y-%m-%d",      // ISO format: 2024-01-15
        "%d/%m/%Y",      // European: 15/01/2024
        "%m/%d/%Y",      // American: 01/15/2024
        "%Y/%m/%d",      // ISO with slashes: 2024/01/15
        "%d-%m-%Y",      // European with dashes: 15-01-2024
        "%m-%d-%Y",      // American with dashes: 01-15-2024
        "%d.%m.%Y",      // German: 15.01.2024
        "%B %d, %Y",     // Long form: January 15, 2024
        "%b %d, %Y",     // Short form: Jan 15, 2024
        "%d %B %Y",      // European long: 15 January 2024
        "%d %b %Y",      // European short: 15 Jan 2024
    ];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, format) {
            return Some(date);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_parse_auto() {
        // Null values
        assert!(matches!(Value::parse_auto(""), Value::Null));
        assert!(matches!(Value::parse_auto("null"), Value::Null));
        assert!(matches!(Value::parse_auto("NA"), Value::Null));

        // Booleans
        assert!(matches!(Value::parse_auto("true"), Value::Boolean(true)));
        assert!(matches!(Value::parse_auto("false"), Value::Boolean(false)));
        assert!(matches!(Value::parse_auto("yes"), Value::Boolean(true)));
        assert!(matches!(Value::parse_auto("no"), Value::Boolean(false)));

        // Numbers
        if let Value::Number(n) = Value::parse_auto("42") {
            assert_eq!(n, 42.0);
        } else {
            panic!("Expected number");
        }

        if let Value::Number(n) = Value::parse_auto("3.14") {
            assert!((n - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected number");
        }

        // Dates
        if let Value::Date(d) = Value::parse_auto("2024-01-15") {
            assert_eq!(d, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        } else {
            panic!("Expected date");
        }

        // Text
        if let Value::Text(s) = Value::parse_auto("Hello World") {
            assert_eq!(s, "Hello World");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_data_source_basic() {
        let mut ds = DataSource::inline("test");
        ds.add_column(ColumnDef::new("name", DataType::Text));
        ds.add_column(ColumnDef::new("age", DataType::Number));

        let mut record = Record::new();
        record.insert("name".to_string(), Value::Text("Alice".to_string()));
        record.insert("age".to_string(), Value::Number(30.0));
        ds.add_record(record);

        assert_eq!(ds.record_count(), 1);
        assert_eq!(ds.column_count(), 2);
        assert!(ds.has_column("name"));
        assert!(!ds.has_column("email"));
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::Text("hello".to_string()).to_string_value(), "hello");
        assert_eq!(Value::Number(42.0).to_string_value(), "42");
        assert_eq!(Value::Number(3.14).to_string_value(), "3.14");
        assert_eq!(Value::Boolean(true).to_string_value(), "true");
        assert_eq!(Value::Null.to_string_value(), "");
    }
}
