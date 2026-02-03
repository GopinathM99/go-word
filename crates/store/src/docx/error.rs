//! Error types for DOCX operations

use thiserror::Error;

/// Errors that can occur during DOCX import/export
#[derive(Debug, Error)]
pub enum DocxError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP archive error
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// Invalid DOCX structure
    #[error("Invalid DOCX structure: {0}")]
    InvalidStructure(String),

    /// Missing required part
    #[error("Missing required part: {0}")]
    MissingPart(String),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Invalid content type
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),

    /// Relationship error
    #[error("Relationship error: {0}")]
    RelationshipError(String),

    /// Style parsing error
    #[error("Style error: {0}")]
    StyleError(String),

    /// Image processing error
    #[error("Image error: {0}")]
    ImageError(String),

    /// Document model error
    #[error("Document model error: {0}")]
    DocModel(#[from] doc_model::DocModelError),

    /// UTF-8 encoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl From<quick_xml::Error> for DocxError {
    fn from(err: quick_xml::Error) -> Self {
        DocxError::XmlParse(err.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for DocxError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        DocxError::XmlParse(format!("Attribute error: {}", err))
    }
}

/// Result type for DOCX operations
pub type DocxResult<T> = std::result::Result<T, DocxError>;
