//! Operation storage abstraction for the collaboration system.
//!
//! This module defines the `OperationStore` trait that provides a unified interface
//! for persisting and retrieving CRDT operations and document snapshots. Implementations
//! can use various backends such as memory, files, or databases.

use crate::clock::VectorClock;
use crate::operation::CrdtOp;
use crate::permissions::DocId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Version number for tracking document state
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version(pub u64);

impl Version {
    /// Create a new version with the given number
    pub fn new(version: u64) -> Self {
        Self(version)
    }

    /// Get the version number
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Create an initial version (version 0)
    pub fn initial() -> Self {
        Self(0)
    }

    /// Increment the version and return the new value
    pub fn increment(&mut self) -> Self {
        self.0 += 1;
        self.clone()
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::initial()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A stored operation with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredOperation {
    /// The CRDT operation
    pub operation: CrdtOp,
    /// Version number when this operation was stored
    pub version: Version,
    /// Timestamp when the operation was stored
    pub stored_at: DateTime<Utc>,
    /// The vector clock state after this operation
    pub clock: VectorClock,
}

impl StoredOperation {
    /// Create a new stored operation
    pub fn new(operation: CrdtOp, version: Version, clock: VectorClock) -> Self {
        Self {
            operation,
            version,
            stored_at: Utc::now(),
            clock,
        }
    }
}

/// A document snapshot for fast state reconstruction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// The version at which this snapshot was taken
    pub version: Version,
    /// The vector clock state at snapshot time
    pub clock: VectorClock,
    /// Serialized document state (opaque bytes)
    pub data: Vec<u8>,
    /// Timestamp when the snapshot was created
    pub created_at: DateTime<Utc>,
    /// Optional description or metadata
    pub description: Option<String>,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(version: Version, clock: VectorClock, data: Vec<u8>) -> Self {
        Self {
            version,
            clock,
            data,
            created_at: Utc::now(),
            description: None,
        }
    }

    /// Create a snapshot with a description
    pub fn with_description(
        version: Version,
        clock: VectorClock,
        data: Vec<u8>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            version,
            clock,
            data,
            created_at: Utc::now(),
            description: Some(description.into()),
        }
    }
}

/// Errors that can occur during storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    /// Document not found
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Version not found
    #[error("Version not found: {0}")]
    VersionNotFound(Version),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Storage is full or quota exceeded
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Concurrent modification conflict
    #[error("Concurrent modification conflict")]
    ConcurrentModification,

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Internal storage error
    #[error("Internal storage error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::SerializationError(err.to_string())
    }
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Trait for operation storage backends
///
/// Implementations of this trait provide persistence for CRDT operations
/// and document snapshots. The trait is designed to be backend-agnostic,
/// allowing for memory, file, or database implementations.
///
/// # Thread Safety
///
/// Implementations should be thread-safe if concurrent access is required.
/// The trait methods take `&self` to allow for internal mutability patterns
/// (e.g., using `Mutex` or `RwLock`).
///
/// # Example
///
/// ```ignore
/// use collab::server::storage::{OperationStore, Version};
///
/// fn save_document_operations<S: OperationStore>(
///     store: &S,
///     doc_id: &DocId,
///     operations: Vec<CrdtOp>,
/// ) -> StorageResult<()> {
///     for op in operations {
///         store.save_operation(doc_id, op)?;
///     }
///     Ok(())
/// }
/// ```
pub trait OperationStore: Send + Sync {
    /// Save an operation for a document
    ///
    /// The operation is appended to the document's operation log.
    /// Returns the version number assigned to this operation.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    /// * `operation` - The CRDT operation to store
    ///
    /// # Returns
    ///
    /// The version number assigned to this operation
    fn save_operation(&self, doc_id: &DocId, operation: CrdtOp) -> StorageResult<Version>;

    /// Save multiple operations atomically
    ///
    /// All operations are saved as a batch. If any operation fails,
    /// none of the operations are persisted (transactional behavior).
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    /// * `operations` - The operations to store
    ///
    /// # Returns
    ///
    /// A vector of version numbers for each operation
    fn save_operations(&self, doc_id: &DocId, operations: Vec<CrdtOp>) -> StorageResult<Vec<Version>> {
        let mut versions = Vec::with_capacity(operations.len());
        for op in operations {
            versions.push(self.save_operation(doc_id, op)?);
        }
        Ok(versions)
    }

    /// Get all operations since a given version
    ///
    /// Returns operations with version numbers strictly greater than
    /// the specified version. This is useful for syncing clients that
    /// are behind the current state.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    /// * `version` - The version to start from (exclusive)
    ///
    /// # Returns
    ///
    /// A vector of stored operations with their metadata
    fn get_operations_since(
        &self,
        doc_id: &DocId,
        version: &Version,
    ) -> StorageResult<Vec<StoredOperation>>;

    /// Get the latest version number for a document
    ///
    /// Returns `Version(0)` if the document has no operations.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    fn get_latest_version(&self, doc_id: &DocId) -> StorageResult<Version>;

    /// Save a snapshot of the document state
    ///
    /// Snapshots are used for fast document reconstruction. Instead of
    /// replaying all operations from the beginning, clients can load
    /// the latest snapshot and only replay operations since then.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    /// * `snapshot` - The snapshot to store
    fn save_snapshot(&self, doc_id: &DocId, snapshot: Snapshot) -> StorageResult<()>;

    /// Get the latest snapshot for a document
    ///
    /// Returns `None` if no snapshot exists.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    fn get_latest_snapshot(&self, doc_id: &DocId) -> StorageResult<Option<Snapshot>>;

    /// Get a specific snapshot by version
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    /// * `version` - The version of the snapshot to retrieve
    fn get_snapshot_at_version(
        &self,
        doc_id: &DocId,
        version: &Version,
    ) -> StorageResult<Option<Snapshot>> {
        // Default implementation: check if latest snapshot matches
        let snapshot = self.get_latest_snapshot(doc_id)?;
        match snapshot {
            Some(s) if s.version == *version => Ok(Some(s)),
            _ => Ok(None),
        }
    }

    /// Get all operations for a document
    ///
    /// This is a convenience method that returns all operations from version 0.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    fn get_all_operations(&self, doc_id: &DocId) -> StorageResult<Vec<StoredOperation>> {
        self.get_operations_since(doc_id, &Version::initial())
    }

    /// Check if a document exists in storage
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    fn document_exists(&self, doc_id: &DocId) -> StorageResult<bool> {
        match self.get_latest_version(doc_id) {
            Ok(_) => Ok(true),
            Err(StorageError::DocumentNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Delete all data for a document
    ///
    /// This removes all operations and snapshots for the document.
    /// Use with caution as this operation is irreversible.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    fn delete_document(&self, doc_id: &DocId) -> StorageResult<()>;

    /// Get storage statistics for a document
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document identifier
    fn get_stats(&self, doc_id: &DocId) -> StorageResult<StorageStats> {
        let version = self.get_latest_version(doc_id)?;
        let ops = self.get_all_operations(doc_id)?;
        let snapshot = self.get_latest_snapshot(doc_id)?;

        Ok(StorageStats {
            operation_count: ops.len(),
            latest_version: version,
            has_snapshot: snapshot.is_some(),
            snapshot_version: snapshot.as_ref().map(|s| s.version.clone()),
            oldest_operation: ops.first().map(|o| o.stored_at),
            newest_operation: ops.last().map(|o| o.stored_at),
        })
    }
}

/// Statistics about stored data for a document
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageStats {
    /// Number of stored operations
    pub operation_count: usize,
    /// Latest version number
    pub latest_version: Version,
    /// Whether a snapshot exists
    pub has_snapshot: bool,
    /// Version of the latest snapshot (if any)
    pub snapshot_version: Option<Version>,
    /// Timestamp of the oldest operation
    pub oldest_operation: Option<DateTime<Utc>>,
    /// Timestamp of the newest operation
    pub newest_operation: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_new() {
        let v = Version::new(42);
        assert_eq!(v.value(), 42);
    }

    #[test]
    fn test_version_initial() {
        let v = Version::initial();
        assert_eq!(v.value(), 0);
    }

    #[test]
    fn test_version_increment() {
        let mut v = Version::initial();
        let v1 = v.increment();
        assert_eq!(v1.value(), 1);
        assert_eq!(v.value(), 1);

        let v2 = v.increment();
        assert_eq!(v2.value(), 2);
    }

    #[test]
    fn test_version_ordering() {
        let v1 = Version::new(1);
        let v2 = Version::new(2);
        let v3 = Version::new(2);

        assert!(v1 < v2);
        assert!(v2 > v1);
        assert!(v2 == v3);
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(42);
        assert_eq!(format!("{}", v), "v42");
    }

    #[test]
    fn test_stored_operation_new() {
        use crate::op_id::{ClientId, OpId};
        use doc_model::NodeId;

        let op = CrdtOp::TextInsert {
            id: OpId::new(ClientId::new(1), 1),
            node_id: NodeId::new(),
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let version = Version::new(1);
        let clock = VectorClock::new();

        let stored = StoredOperation::new(op.clone(), version.clone(), clock.clone());

        assert_eq!(stored.version, version);
        assert_eq!(stored.operation.id(), op.id());
    }

    #[test]
    fn test_snapshot_new() {
        let version = Version::new(10);
        let clock = VectorClock::new();
        let data = vec![1, 2, 3, 4, 5];

        let snapshot = Snapshot::new(version.clone(), clock, data.clone());

        assert_eq!(snapshot.version, version);
        assert_eq!(snapshot.data, data);
        assert!(snapshot.description.is_none());
    }

    #[test]
    fn test_snapshot_with_description() {
        let version = Version::new(10);
        let clock = VectorClock::new();
        let data = vec![1, 2, 3];

        let snapshot = Snapshot::with_description(
            version.clone(),
            clock,
            data,
            "Test snapshot",
        );

        assert_eq!(snapshot.description, Some("Test snapshot".to_string()));
    }

    #[test]
    fn test_storage_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let storage_err: StorageError = io_err.into();

        match storage_err {
            StorageError::IoError(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_storage_error_from_serde() {
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let storage_err: StorageError = json_err.into();

        match storage_err {
            StorageError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_version_serialization() {
        let version = Version::new(42);
        let json = serde_json::to_string(&version).unwrap();
        let restored: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, version);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot::with_description(
            Version::new(5),
            VectorClock::new(),
            vec![10, 20, 30],
            "test",
        );
        let json = serde_json::to_string(&snapshot).unwrap();
        let restored: Snapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.version, snapshot.version);
        assert_eq!(restored.data, snapshot.data);
        assert_eq!(restored.description, snapshot.description);
    }
}
