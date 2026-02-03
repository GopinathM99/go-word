//! Error types for template operations

use thiserror::Error;

/// Errors that can occur during template operations
#[derive(Debug, Error)]
pub enum TemplateError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP archive error
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Template not found
    #[error("Template not found: {0}")]
    NotFound(String),

    /// Invalid template format
    #[error("Invalid template format: {0}")]
    InvalidFormat(String),

    /// Missing required file in template package
    #[error("Missing required file in template: {0}")]
    MissingFile(String),

    /// Template already exists
    #[error("Template already exists: {0}")]
    AlreadyExists(String),

    /// Invalid template ID
    #[error("Invalid template ID: {0}")]
    InvalidId(String),

    /// Locked region error
    #[error("Cannot edit locked region: {0}")]
    LockedRegion(String),

    /// Document model error
    #[error("Document error: {0}")]
    Document(String),

    /// Store error
    #[error("Store error: {0}")]
    Store(#[from] crate::StoreError),
}

/// Result type for template operations
pub type TemplateResult<T> = std::result::Result<T, TemplateError>;
