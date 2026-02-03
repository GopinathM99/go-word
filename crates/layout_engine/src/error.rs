//! Error types for layout engine

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Layout failed: {0}")]
    LayoutFailed(String),

    #[error("Invalid page setup: {0}")]
    InvalidPageSetup(String),

    #[error("Document model error: {0}")]
    DocModel(#[from] doc_model::DocModelError),
}

pub type Result<T> = std::result::Result<T, LayoutError>;
