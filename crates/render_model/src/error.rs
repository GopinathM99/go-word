//! Error types for render model

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Render conversion failed: {0}")]
    ConversionFailed(String),

    #[error("Invalid layout: {0}")]
    InvalidLayout(String),
}

pub type Result<T> = std::result::Result<T, RenderError>;
