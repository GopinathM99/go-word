//! Error types for text engine

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextError {
    #[error("Font not found: {0}")]
    FontNotFound(String),

    #[error("Shaping failed: {0}")]
    ShapingFailed(String),

    #[error("Invalid font data: {0}")]
    InvalidFontData(String),

    #[error("Font discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("Font loading failed: {0}")]
    LoadingFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TextError>;
