//! Error types for RTF operations

use thiserror::Error;

/// Errors that can occur during RTF import/export
#[derive(Debug, Error)]
pub enum RtfError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error in RTF content
    #[error("Parse error at position {position}: {message}")]
    ParseError {
        position: usize,
        message: String,
    },

    /// Invalid RTF structure
    #[error("Invalid RTF structure: {0}")]
    InvalidStructure(String),

    /// Missing required element
    #[error("Missing required element: {0}")]
    MissingElement(String),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Invalid control word
    #[error("Invalid control word: {0}")]
    InvalidControlWord(String),

    /// Character encoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// Image processing error
    #[error("Image error: {0}")]
    ImageError(String),

    /// Table structure error
    #[error("Table error: {0}")]
    TableError(String),

    /// Document model error
    #[error("Document model error: {0}")]
    DocModel(#[from] doc_model::DocModelError),

    /// UTF-8 encoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Unexpected end of input
    #[error("Unexpected end of input")]
    UnexpectedEof,

    /// Unmatched braces
    #[error("Unmatched braces at position {0}")]
    UnmatchedBraces(usize),
}

impl RtfError {
    /// Create a parse error at a specific position
    pub fn parse_error(position: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            position,
            message: message.into(),
        }
    }

    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure(message.into())
    }

    /// Create an unsupported feature error
    pub fn unsupported(feature: impl Into<String>) -> Self {
        Self::UnsupportedFeature(feature.into())
    }
}

/// Result type for RTF operations
pub type RtfResult<T> = std::result::Result<T, RtfError>;
