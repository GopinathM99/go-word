//! Error types for mail merge operations

use thiserror::Error;

/// Errors that can occur during mail merge operations
#[derive(Debug, Error)]
pub enum MailMergeError {
    /// IO error reading/writing files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing CSV data
    #[error("CSV parse error: {0}")]
    CsvParse(#[from] csv::Error),

    /// Error parsing JSON data
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Invalid data source configuration
    #[error("Invalid data source: {0}")]
    InvalidDataSource(String),

    /// Column not found in data source
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Invalid data type conversion
    #[error("Invalid data type: expected {expected}, got {actual}")]
    InvalidDataType {
        expected: String,
        actual: String,
    },

    /// Record not found
    #[error("Record not found at index {0}")]
    RecordNotFound(usize),

    /// Invalid path expression for JSON traversal
    #[error("Invalid path expression: {0}")]
    InvalidPath(String),

    /// Empty data source
    #[error("Data source is empty: {0}")]
    EmptyDataSource(String),

    /// Duplicate column names
    #[error("Duplicate column name: {0}")]
    DuplicateColumn(String),

    /// Invalid delimiter
    #[error("Invalid delimiter: {0}")]
    InvalidDelimiter(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Unsupported file format
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    /// Error parsing XLSX data
    #[error("XLSX parse error: {0}")]
    XlsxParse(String),
}

/// Result type for mail merge operations
pub type Result<T> = std::result::Result<T, MailMergeError>;
