//! Error types for editing operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Document model error: {0}")]
    DocModel(#[from] doc_model::DocModelError),

    #[error("Undo stack is empty")]
    UndoStackEmpty,

    #[error("Redo stack is empty")]
    RedoStackEmpty,
}

pub type Result<T> = std::result::Result<T, EditError>;
