//! Error types for revision tracking operations

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RevisionError {
    #[error("Revision not found: {0}")]
    RevisionNotFound(Uuid),

    #[error("Invalid revision operation: {0}")]
    InvalidOperation(String),

    #[error("Revision already accepted or rejected: {0}")]
    RevisionAlreadyProcessed(Uuid),

    #[error("Cannot modify revision while tracking is disabled")]
    TrackingDisabled,

    #[error("Document model error: {0}")]
    DocModel(#[from] doc_model::DocModelError),

    #[error("Revision range conflict: {0}")]
    RangeConflict(String),

    #[error("Invalid author: {0}")]
    InvalidAuthor(String),
}

pub type Result<T> = std::result::Result<T, RevisionError>;
