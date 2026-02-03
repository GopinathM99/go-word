//! Error types for document model operations

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DocModelError {
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Invalid position: node {node_id}, offset {offset}")]
    InvalidPosition { node_id: Uuid, offset: usize },

    #[error("Invalid selection: {0}")]
    InvalidSelection(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Tree structure error: {0}")]
    TreeStructureError(String),
}

pub type Result<T> = std::result::Result<T, DocModelError>;
