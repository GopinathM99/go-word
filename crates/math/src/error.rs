//! Error types for the math crate

use thiserror::Error;

/// Errors that can occur in math operations
#[derive(Error, Debug)]
pub enum MathError {
    /// Error parsing OMML XML
    #[error("OMML parsing error: {0}")]
    OmmlParse(String),

    /// Error writing OMML XML
    #[error("OMML writing error: {0}")]
    OmmlWrite(String),

    /// Error parsing linear notation
    #[error("Linear notation parsing error: {0}")]
    LinearParse(String),

    /// Error during layout calculation
    #[error("Layout error: {0}")]
    Layout(String),

    /// Error during rendering
    #[error("Render error: {0}")]
    Render(String),

    /// Invalid math structure
    #[error("Invalid math structure: {0}")]
    InvalidStructure(String),

    /// XML error from quick-xml
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// UTF-8 decoding error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

/// Result type for math operations
pub type MathResult<T> = Result<T, MathError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MathError::OmmlParse("unexpected element".to_string());
        assert_eq!(err.to_string(), "OMML parsing error: unexpected element");
    }

    #[test]
    fn test_error_from_xml() {
        let xml_err = quick_xml::Error::Io(std::sync::Arc::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error",
        )));
        let math_err: MathError = xml_err.into();
        assert!(matches!(math_err, MathError::Xml(_)));
    }
}
