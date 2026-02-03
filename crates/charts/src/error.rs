//! Error types for the charts crate

use thiserror::Error;

/// Errors that can occur when working with charts
#[derive(Error, Debug)]
pub enum ChartError {
    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// Invalid chart type
    #[error("Invalid chart type: {0}")]
    InvalidChartType(String),

    /// Missing required element
    #[error("Missing required element: {0}")]
    MissingElement(String),

    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Layout calculation error
    #[error("Layout error: {0}")]
    Layout(String),

    /// Rendering error
    #[error("Rendering error: {0}")]
    Render(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<quick_xml::Error> for ChartError {
    fn from(err: quick_xml::Error) -> Self {
        ChartError::XmlParse(err.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for ChartError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        ChartError::XmlParse(err.to_string())
    }
}

/// Result type for chart operations
pub type ChartResult<T> = Result<T, ChartError>;
