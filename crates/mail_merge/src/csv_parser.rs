//! CSV parser for mail merge data sources

use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use crate::data_source::{ColumnDef, DataSource, DataSourceType, DataType, Record, Value};
use crate::error::{MailMergeError, Result};

/// CSV parser configuration
#[derive(Debug, Clone)]
pub struct CsvConfig {
    /// Delimiter character
    pub delimiter: char,
    /// Whether the first row contains headers
    pub has_header: bool,
    /// Whether to trim whitespace from values
    pub trim_whitespace: bool,
    /// Whether to auto-detect data types
    pub auto_detect_types: bool,
    /// Character encoding (currently only UTF-8 is supported)
    pub encoding: String,
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            delimiter: ',',
            has_header: true,
            trim_whitespace: true,
            auto_detect_types: true,
            encoding: "utf-8".to_string(),
        }
    }
}

impl CsvConfig {
    /// Create a new CSV config with comma delimiter
    pub fn comma() -> Self {
        Self::default()
    }

    /// Create a new CSV config with semicolon delimiter
    pub fn semicolon() -> Self {
        Self {
            delimiter: ';',
            ..Default::default()
        }
    }

    /// Create a new CSV config with tab delimiter
    pub fn tab() -> Self {
        Self {
            delimiter: '\t',
            ..Default::default()
        }
    }

    /// Set the delimiter
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Set whether the first row is a header
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Set whether to trim whitespace
    pub fn with_trim(mut self, trim: bool) -> Self {
        self.trim_whitespace = trim;
        self
    }

    /// Set whether to auto-detect types
    pub fn with_auto_detect(mut self, auto_detect: bool) -> Self {
        self.auto_detect_types = auto_detect;
        self
    }
}

/// CSV parser for creating data sources from CSV files or strings
pub struct CsvParser {
    config: CsvConfig,
}

impl CsvParser {
    /// Create a new CSV parser with default configuration
    pub fn new() -> Self {
        Self {
            config: CsvConfig::default(),
        }
    }

    /// Create a new CSV parser with custom configuration
    pub fn with_config(config: CsvConfig) -> Self {
        Self { config }
    }

    /// Parse a CSV file and return a DataSource
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<DataSource> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(MailMergeError::FileNotFound(
                path.display().to_string()
            ));
        }

        let file = std::fs::File::open(path)?;
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("csv_source")
            .to_string();

        let source_type = DataSourceType::Csv {
            path: path.display().to_string(),
            delimiter: self.config.delimiter,
            has_header: self.config.has_header,
        };

        self.parse_reader(file, id, source_type)
    }

    /// Parse CSV from a string and return a DataSource
    pub fn parse_string(&self, data: &str, id: impl Into<String>) -> Result<DataSource> {
        let source_type = DataSourceType::Inline {
            data: Vec::new(),
        };
        self.parse_reader(data.as_bytes(), id.into(), source_type)
    }

    /// Parse CSV from any reader
    fn parse_reader<R: Read>(
        &self,
        reader: R,
        id: String,
        source_type: DataSourceType,
    ) -> Result<DataSource> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(self.config.delimiter as u8)
            .has_headers(self.config.has_header)
            .trim(if self.config.trim_whitespace {
                csv::Trim::All
            } else {
                csv::Trim::None
            })
            .flexible(true) // Allow records with varying number of fields
            .from_reader(reader);

        let mut data_source = DataSource::new(id, source_type);

        // Get headers
        let headers: Vec<String> = if self.config.has_header {
            csv_reader
                .headers()?
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            // Generate column names if no header
            Vec::new()
        };

        // Check for duplicate headers
        if self.config.has_header {
            let mut seen = HashSet::new();
            for header in &headers {
                if !seen.insert(header.clone()) {
                    return Err(MailMergeError::DuplicateColumn(header.clone()));
                }
            }
        }

        // Collect all records first to detect types
        let mut raw_records: Vec<csv::StringRecord> = Vec::new();
        for result in csv_reader.records() {
            raw_records.push(result?);
        }

        if raw_records.is_empty() && headers.is_empty() {
            return Err(MailMergeError::EmptyDataSource(
                "CSV file is empty".to_string()
            ));
        }

        // Determine column count from headers or first record
        let column_count = if !headers.is_empty() {
            headers.len()
        } else if let Some(first) = raw_records.first() {
            first.len()
        } else {
            0
        };

        // Generate headers if needed
        let final_headers: Vec<String> = if headers.is_empty() {
            (0..column_count)
                .map(|i| format!("Column{}", i + 1))
                .collect()
        } else {
            headers
        };

        // Detect types for each column if auto-detect is enabled
        let column_types = if self.config.auto_detect_types {
            detect_column_types(&final_headers, &raw_records)
        } else {
            // Default to Text for all columns
            vec![DataType::Text; column_count]
        };

        // Create column definitions
        for (i, header) in final_headers.iter().enumerate() {
            let data_type = column_types.get(i).copied().unwrap_or(DataType::Text);
            data_source.add_column(ColumnDef::new(header.clone(), data_type));
        }

        // Convert records
        for raw_record in raw_records {
            let mut record = Record::new();
            for (i, field) in raw_record.iter().enumerate() {
                if let Some(header) = final_headers.get(i) {
                    let value = if self.config.auto_detect_types {
                        Value::parse_auto(field)
                    } else {
                        Value::Text(field.to_string())
                    };
                    record.insert(header.clone(), value);
                }
            }
            data_source.add_record(record);
        }

        Ok(data_source)
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect the data type for each column based on all values
fn detect_column_types(headers: &[String], records: &[csv::StringRecord]) -> Vec<DataType> {
    headers
        .iter()
        .enumerate()
        .map(|(col_idx, _)| {
            detect_column_type(records, col_idx)
        })
        .collect()
}

/// Detect the most appropriate data type for a single column
fn detect_column_type(records: &[csv::StringRecord], col_idx: usize) -> DataType {
    let mut has_number = false;
    let mut has_date = false;
    let mut has_boolean = false;
    let mut has_text = false;
    let mut all_null = true;

    for record in records {
        if let Some(field) = record.get(col_idx) {
            let value = Value::parse_auto(field);
            all_null = false;

            match value {
                Value::Number(_) => has_number = true,
                Value::Date(_) => has_date = true,
                Value::Boolean(_) => has_boolean = true,
                Value::Text(_) => has_text = true,
                Value::Null => {}
            }
        }
    }

    // If we have any text values, treat the whole column as text
    if has_text {
        return DataType::Text;
    }

    // If all values are consistent, use that type
    if all_null {
        DataType::Text
    } else if has_date && !has_number && !has_boolean {
        DataType::Date
    } else if has_boolean && !has_number && !has_date {
        DataType::Boolean
    } else if has_number && !has_date && !has_boolean {
        DataType::Number
    } else {
        // Mixed types - default to text
        DataType::Text
    }
}

/// Detect the delimiter used in a CSV file
pub fn detect_delimiter(content: &str) -> char {
    let first_line = content.lines().next().unwrap_or("");

    let delimiters = [',', ';', '\t', '|'];
    let mut best_delimiter = ',';
    let mut best_count = 0;

    for &delim in &delimiters {
        let count = first_line.matches(delim).count();
        if count > best_count {
            best_count = count;
            best_delimiter = delim;
        }
    }

    best_delimiter
}

/// Detect if the first row is likely a header
pub fn detect_has_header(content: &str, delimiter: char) -> bool {
    let mut lines = content.lines();

    let first_line = match lines.next() {
        Some(l) => l,
        None => return true, // Default to true for empty content
    };

    let second_line = match lines.next() {
        Some(l) => l,
        None => return true, // Only one line, assume it's a header
    };

    // Split both lines
    let first_fields: Vec<&str> = first_line.split(delimiter).collect();
    let second_fields: Vec<&str> = second_line.split(delimiter).collect();

    // Compare field types
    let mut first_all_text = true;
    let mut second_has_numbers = false;

    for field in &first_fields {
        let trimmed = field.trim();
        if !trimmed.is_empty() && trimmed.parse::<f64>().is_ok() {
            first_all_text = false;
            break;
        }
    }

    for field in &second_fields {
        let trimmed = field.trim();
        if !trimmed.is_empty() && trimmed.parse::<f64>().is_ok() {
            second_has_numbers = true;
            break;
        }
    }

    // If first row is all text and second row has numbers, likely has header
    first_all_text && second_has_numbers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_csv() {
        let csv_data = "name,age,city\nAlice,30,New York\nBob,25,Los Angeles";
        let parser = CsvParser::new();
        let ds = parser.parse_string(csv_data, "test").unwrap();

        assert_eq!(ds.column_count(), 3);
        assert_eq!(ds.record_count(), 2);
        assert!(ds.has_column("name"));
        assert!(ds.has_column("age"));
        assert!(ds.has_column("city"));
    }

    #[test]
    fn test_parse_semicolon_delimiter() {
        let csv_data = "name;age;city\nAlice;30;New York\nBob;25;Los Angeles";
        let parser = CsvParser::with_config(CsvConfig::semicolon());
        let ds = parser.parse_string(csv_data, "test").unwrap();

        assert_eq!(ds.column_count(), 3);
        assert_eq!(ds.record_count(), 2);
    }

    #[test]
    fn test_parse_tab_delimiter() {
        let csv_data = "name\tage\tcity\nAlice\t30\tNew York\nBob\t25\tLos Angeles";
        let parser = CsvParser::with_config(CsvConfig::tab());
        let ds = parser.parse_string(csv_data, "test").unwrap();

        assert_eq!(ds.column_count(), 3);
        assert_eq!(ds.record_count(), 2);
    }

    #[test]
    fn test_parse_no_header() {
        let csv_data = "Alice,30,New York\nBob,25,Los Angeles";
        let parser = CsvParser::with_config(CsvConfig::default().with_header(false));
        let ds = parser.parse_string(csv_data, "test").unwrap();

        assert_eq!(ds.column_count(), 3);
        assert_eq!(ds.record_count(), 2);
        assert!(ds.has_column("Column1"));
        assert!(ds.has_column("Column2"));
        assert!(ds.has_column("Column3"));
    }

    #[test]
    fn test_parse_with_quotes() {
        let csv_data = r#"name,address,age
"Alice Smith","123 Main St, Apt 4",30
"Bob Jones","456 Oak Ave",25"#;
        let parser = CsvParser::new();
        let ds = parser.parse_string(csv_data, "test").unwrap();

        assert_eq!(ds.record_count(), 2);

        let first_record = ds.get_record(0).unwrap();
        assert_eq!(first_record.get("name").unwrap().to_string_value(), "Alice Smith");
        assert_eq!(first_record.get("address").unwrap().to_string_value(), "123 Main St, Apt 4");
    }

    #[test]
    fn test_auto_type_detection() {
        let csv_data = "name,age,active,joined
Alice,30,true,2024-01-15
Bob,25,false,2023-06-20";
        let parser = CsvParser::new();
        let ds = parser.parse_string(csv_data, "test").unwrap();

        // Check column types
        assert_eq!(ds.get_column("name").unwrap().data_type, DataType::Text);
        assert_eq!(ds.get_column("age").unwrap().data_type, DataType::Number);
        assert_eq!(ds.get_column("active").unwrap().data_type, DataType::Boolean);
        assert_eq!(ds.get_column("joined").unwrap().data_type, DataType::Date);
    }

    #[test]
    fn test_detect_delimiter() {
        assert_eq!(detect_delimiter("a,b,c,d"), ',');
        assert_eq!(detect_delimiter("a;b;c;d"), ';');
        assert_eq!(detect_delimiter("a\tb\tc\td"), '\t');
        assert_eq!(detect_delimiter("a|b|c|d"), '|');
    }

    #[test]
    fn test_detect_has_header() {
        // First row text, second row numbers - likely has header
        assert!(detect_has_header("name,age\nAlice,30", ','));

        // Both rows have numbers - likely no header
        assert!(!detect_has_header("1,2,3\n4,5,6", ','));
    }

    #[test]
    fn test_duplicate_headers() {
        let csv_data = "name,age,name\nAlice,30,Smith";
        let parser = CsvParser::new();
        let result = parser.parse_string(csv_data, "test");

        assert!(matches!(result, Err(MailMergeError::DuplicateColumn(_))));
    }

    #[test]
    fn test_empty_csv() {
        let csv_data = "";
        let parser = CsvParser::with_config(CsvConfig::default().with_header(false));
        let result = parser.parse_string(csv_data, "test");

        assert!(matches!(result, Err(MailMergeError::EmptyDataSource(_))));
    }

    #[test]
    fn test_null_values() {
        let csv_data = "name,value\nAlice,\nBob,null\nCharlie,NA";
        let parser = CsvParser::new();
        let ds = parser.parse_string(csv_data, "test").unwrap();

        let r1 = ds.get_record(0).unwrap();
        let r2 = ds.get_record(1).unwrap();
        let r3 = ds.get_record(2).unwrap();

        assert!(r1.get("value").unwrap().is_null());
        assert!(r2.get("value").unwrap().is_null());
        assert!(r3.get("value").unwrap().is_null());
    }

    #[test]
    fn test_whitespace_trimming() {
        let csv_data = "name,age\n  Alice  ,  30  ";
        let parser = CsvParser::with_config(CsvConfig::default().with_trim(true));
        let ds = parser.parse_string(csv_data, "test").unwrap();

        let record = ds.get_record(0).unwrap();
        assert_eq!(record.get("name").unwrap().to_string_value(), "Alice");
    }

    #[test]
    fn test_preview() {
        let csv_data = "name\nAlice\nBob\nCharlie\nDavid\nEve";
        let parser = CsvParser::new();
        let ds = parser.parse_string(csv_data, "test").unwrap();

        let preview = ds.preview(3);
        assert_eq!(preview.len(), 3);
    }
}
