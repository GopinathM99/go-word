//! Mail Merge Data Sources
//!
//! This crate provides functionality for loading and parsing data sources
//! for mail merge operations in Go Word.
//!
//! # Features
//!
//! - CSV parsing with configurable delimiters (comma, semicolon, tab)
//! - JSON parsing with nested object support and configurable root paths
//! - XLSX/XLS parsing with sheet selection and cell range support
//! - Automatic data type detection
//! - Column mapping and field access
//!
//! # Example
//!
//! ```rust
//! use mail_merge::{CsvParser, CsvConfig, JsonParser, JsonConfig, XlsxParser, XlsxConfig};
//!
//! // Parse a CSV string
//! let csv_data = "name,age\nAlice,30\nBob,25";
//! let parser = CsvParser::new();
//! let data_source = parser.parse_string(csv_data, "contacts").unwrap();
//!
//! assert_eq!(data_source.record_count(), 2);
//! assert!(data_source.has_column("name"));
//!
//! // Parse a JSON string
//! let json_data = r#"[{"name": "Alice"}, {"name": "Bob"}]"#;
//! let parser = JsonParser::new();
//! let data_source = parser.parse_string(json_data, "contacts").unwrap();
//!
//! assert_eq!(data_source.record_count(), 2);
//!
//! // Parse an XLSX file
//! // let parser = XlsxParser::new();
//! // let data_source = parser.parse_file("contacts.xlsx").unwrap();
//! ```

mod csv_parser;
mod data_source;
mod error;
mod json_parser;
mod xlsx_parser;
pub mod merge_field;
pub mod merge_engine;

// Re-export main types
pub use csv_parser::{CsvConfig, CsvParser, detect_delimiter, detect_has_header};
pub use data_source::{ColumnDef, DataSource, DataSourceType, DataType, Record, Value};
pub use error::{MailMergeError, Result};
pub use json_parser::{JsonConfig, JsonParser, get_nested_value};
pub use xlsx_parser::{XlsxConfig, XlsxParser, SheetSelector, CellRange, get_sheet_names, get_sheet_names_from_bytes};
pub use merge_field::{MergeField, MergeFieldInstruction, ComparisonOperator, ConditionalField};
pub use merge_engine::{MergeEngine, MergeOptions, MergeOutputType, RecordRange, MergeResult, MergedRecord, MergeStatus, MergeProgress, MergeError as MergeExecutionError};

/// Load a data source from a file, automatically detecting the format
pub fn load_from_file(path: &str) -> Result<DataSource> {
    let path_lower = path.to_lowercase();

    if path_lower.ends_with(".csv") {
        // Try to detect delimiter from file content
        let content = std::fs::read_to_string(path)?;
        let delimiter = detect_delimiter(&content);
        let has_header = detect_has_header(&content, delimiter);

        let config = CsvConfig::default()
            .with_delimiter(delimiter)
            .with_header(has_header);

        CsvParser::with_config(config).parse_file(path)
    } else if path_lower.ends_with(".json") {
        JsonParser::new().parse_file(path)
    } else if path_lower.ends_with(".tsv") {
        CsvParser::with_config(CsvConfig::tab()).parse_file(path)
    } else if path_lower.ends_with(".xlsx") || path_lower.ends_with(".xls") {
        XlsxParser::new().parse_file(path)
    } else {
        Err(MailMergeError::UnsupportedFormat(
            format!("Unknown file extension for: {}", path)
        ))
    }
}

/// Create an inline data source from records
pub fn create_inline_source(id: &str, columns: Vec<(&str, DataType)>, records: Vec<Vec<(&str, Value)>>) -> DataSource {
    let mut ds = DataSource::inline(id);

    for (name, data_type) in columns {
        ds.add_column(ColumnDef::new(name, data_type));
    }

    for record_data in records {
        let mut record = Record::new();
        for (key, value) in record_data {
            record.insert(key.to_string(), value);
        }
        ds.add_record(record);
    }

    ds
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv_file() {
        let mut file = NamedTempFile::with_suffix(".csv").unwrap();
        writeln!(file, "name,age").unwrap();
        writeln!(file, "Alice,30").unwrap();
        writeln!(file, "Bob,25").unwrap();

        let ds = load_from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(ds.record_count(), 2);
        assert!(ds.has_column("name"));
        assert!(ds.has_column("age"));
    }

    #[test]
    fn test_load_json_file() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(file, r#"[{{"name": "Alice"}}, {{"name": "Bob"}}]"#).unwrap();

        let ds = load_from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(ds.record_count(), 2);
        assert!(ds.has_column("name"));
    }

    #[test]
    fn test_load_tsv_file() {
        let mut file = NamedTempFile::with_suffix(".tsv").unwrap();
        writeln!(file, "name\tage").unwrap();
        writeln!(file, "Alice\t30").unwrap();

        let ds = load_from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(ds.record_count(), 1);
        assert!(ds.has_column("name"));
    }

    #[test]
    fn test_unsupported_format() {
        let result = load_from_file("/path/to/file.xyz");
        assert!(matches!(result, Err(MailMergeError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_create_inline_source() {
        let ds = create_inline_source(
            "test",
            vec![("name", DataType::Text), ("age", DataType::Number)],
            vec![
                vec![("name", Value::Text("Alice".to_string())), ("age", Value::Number(30.0))],
                vec![("name", Value::Text("Bob".to_string())), ("age", Value::Number(25.0))],
            ],
        );

        assert_eq!(ds.record_count(), 2);
        assert_eq!(ds.column_count(), 2);

        let record = ds.get_record(0).unwrap();
        assert_eq!(record.get("name").unwrap().to_string_value(), "Alice");
    }

    #[test]
    fn test_data_source_get_value() {
        let csv_data = "name,age\nAlice,30\nBob,25";
        let ds = CsvParser::new().parse_string(csv_data, "test").unwrap();

        let value = ds.get_value(0, "name").unwrap();
        assert_eq!(value.to_string_value(), "Alice");

        let value = ds.get_value(1, "age").unwrap();
        if let Value::Number(n) = value {
            assert_eq!(*n, 25.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_column_def_display_name() {
        let col = ColumnDef::new("first_name", DataType::Text)
            .with_display_name("First Name")
            .with_description("The person's first name");

        assert_eq!(col.name, "first_name");
        assert_eq!(col.display(), "First Name");
        assert_eq!(col.description, Some("The person's first name".to_string()));
    }

    #[test]
    fn test_value_conversions() {
        // From string
        let v: Value = "hello".into();
        assert!(matches!(v, Value::Text(_)));

        // From number
        let v: Value = 42_i32.into();
        assert!(matches!(v, Value::Number(n) if n == 42.0));

        // From bool
        let v: Value = true.into();
        assert!(matches!(v, Value::Boolean(true)));

        // From Option
        let v: Value = Some("hello").into();
        assert!(matches!(v, Value::Text(_)));

        let v: Value = Option::<String>::None.into();
        assert!(matches!(v, Value::Null));
    }
}
