//! Error types for ODT operations

use thiserror::Error;

/// Errors that can occur during ODT import
#[derive(Debug, Error)]
pub enum OdtError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP archive error
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// Invalid ODT structure
    #[error("Invalid ODT structure: {0}")]
    InvalidStructure(String),

    /// Missing required part
    #[error("Missing required part: {0}")]
    MissingPart(String),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

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

    /// Invalid measurement value
    #[error("Invalid measurement: {0}")]
    InvalidMeasurement(String),
}

impl From<quick_xml::Error> for OdtError {
    fn from(err: quick_xml::Error) -> Self {
        OdtError::XmlParse(err.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for OdtError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        OdtError::XmlParse(format!("Attribute error: {}", err))
    }
}

impl OdtError {
    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure(message.into())
    }

    /// Create a missing part error
    pub fn missing_part(part: impl Into<String>) -> Self {
        Self::MissingPart(part.into())
    }

    /// Create an unsupported feature error
    pub fn unsupported(feature: impl Into<String>) -> Self {
        Self::UnsupportedFeature(feature.into())
    }
}

/// Result type for ODT operations
pub type OdtResult<T> = std::result::Result<T, OdtError>;
