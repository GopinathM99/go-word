//! Error types for the collaboration crate.

use crate::op_id::{ClientId, OpId};
use thiserror::Error;

/// Result type alias for collaboration operations.
pub type CollabResult<T> = Result<T, CollabError>;

/// Errors that can occur during collaboration operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CollabError {
    /// Operation references a non-existent parent.
    #[error("Parent operation not found: {0}")]
    ParentNotFound(OpId),

    /// Operation references a non-existent node.
    #[error("Node not found: {0}")]
    NodeNotFound(OpId),

    /// Duplicate operation ID detected.
    #[error("Duplicate operation ID: {0}")]
    DuplicateOpId(OpId),

    /// Invalid operation sequence number.
    #[error("Invalid sequence number for client {client_id}: expected {expected}, got {actual}")]
    InvalidSequence {
        client_id: ClientId,
        expected: u64,
        actual: u64,
    },

    /// Operation is causally invalid (missing dependencies).
    #[error("Causal dependency not satisfied: operation {op} depends on {dependency}")]
    CausalityViolation { op: OpId, dependency: OpId },

    /// Concurrent modification conflict that couldn't be automatically resolved.
    #[error("Unresolvable conflict between operations {op1} and {op2}")]
    UnresolvableConflict { op1: OpId, op2: OpId },

    /// Permission denied for the operation.
    #[error("Permission denied for client {client_id}: {reason}")]
    PermissionDenied { client_id: ClientId, reason: String },

    /// Clock synchronization error.
    #[error("Clock synchronization error: {0}")]
    ClockError(String),

    /// Serialization or deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid operation type for the target.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// The document structure is corrupted.
    #[error("Document structure corruption: {0}")]
    StructureCorruption(String),

    /// Network or communication error.
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Operation has already been applied.
    #[error("Operation already applied: {0}")]
    AlreadyApplied(OpId),

    /// Operation has been tombstoned (deleted).
    #[error("Operation has been deleted: {0}")]
    Tombstoned(OpId),
}
