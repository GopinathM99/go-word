//! In-memory operation store implementation.
//!
//! This module provides `MemoryOperationStore`, an in-memory implementation of the
//! `OperationStore` trait. It's primarily intended for development, testing, and
//! scenarios where persistence across restarts is not required.
//!
//! # Features
//!
//! - Fast read/write operations (no I/O overhead)
//! - Thread-safe with `RwLock` for concurrent access
//! - Suitable for unit tests and development
//! - No persistence - data is lost on restart
//!
//! # Example
//!
//! ```ignore
//! use collab::server::memory_store::MemoryOperationStore;
//! use collab::server::storage::OperationStore;
//!
//! let store = MemoryOperationStore::new();
//!
//! // Save an operation
//! let version = store.save_operation(&doc_id, operation)?;
//!
//! // Retrieve operations
//! let ops = store.get_operations_since(&doc_id, &Version::initial())?;
//! ```

use crate::clock::VectorClock;
use crate::operation::CrdtOp;
use crate::permissions::DocId;
use std::collections::HashMap;
use std::sync::RwLock;

use super::storage::{OperationStore, Snapshot, StorageResult, StoredOperation, Version};

/// In-memory storage for document operations
struct DocumentStorage {
    /// Operations stored in order
    operations: Vec<StoredOperation>,
    /// Current version counter
    version: Version,
    /// Current vector clock state
    clock: VectorClock,
    /// Latest snapshot (if any)
    snapshot: Option<Snapshot>,
}

impl DocumentStorage {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
            version: Version::initial(),
            clock: VectorClock::new(),
            snapshot: None,
        }
    }
}

/// In-memory implementation of `OperationStore`
///
/// This implementation stores all operations and snapshots in memory using
/// `HashMap` structures protected by `RwLock` for thread-safe access.
///
/// # Thread Safety
///
/// The store is thread-safe and can be shared across threads using `Arc`.
/// Read operations acquire a read lock, allowing concurrent reads.
/// Write operations acquire a write lock, ensuring exclusive access.
///
/// # Memory Usage
///
/// All operations are stored in memory indefinitely. For long-running
/// applications with many operations, consider:
/// - Using snapshots to reduce operation count
/// - Implementing operation compaction
/// - Using a file or database-backed store instead
pub struct MemoryOperationStore {
    /// Storage for each document, keyed by document ID
    documents: RwLock<HashMap<String, DocumentStorage>>,
}

impl MemoryOperationStore {
    /// Create a new empty in-memory store
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Get the number of documents in the store
    pub fn document_count(&self) -> usize {
        self.documents.read().unwrap().len()
    }

    /// Get a list of all document IDs in the store
    pub fn list_documents(&self) -> Vec<String> {
        self.documents.read().unwrap().keys().cloned().collect()
    }

    /// Clear all documents from the store
    pub fn clear(&self) {
        self.documents.write().unwrap().clear();
    }

    /// Get memory usage estimate in bytes
    ///
    /// This is a rough estimate based on operation count and snapshot sizes.
    pub fn estimate_memory_usage(&self) -> usize {
        let docs = self.documents.read().unwrap();
        let mut total = 0;

        for storage in docs.values() {
            // Estimate operation size (rough approximation)
            total += storage.operations.len() * 256;

            // Add snapshot size if present
            if let Some(ref snapshot) = storage.snapshot {
                total += snapshot.data.len();
            }
        }

        total
    }
}

impl Default for MemoryOperationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationStore for MemoryOperationStore {
    fn save_operation(&self, doc_id: &DocId, operation: CrdtOp) -> StorageResult<Version> {
        let mut docs = self.documents.write().unwrap();
        let doc_id_str = doc_id.to_string();

        let storage = docs.entry(doc_id_str).or_insert_with(DocumentStorage::new);

        // Update the vector clock with this operation
        let op_id = operation.id();
        let current_seq = storage.clock.get(op_id.client_id);
        if op_id.seq > current_seq {
            storage.clock.set(op_id.client_id, op_id.seq);
        }

        // Increment version and store operation
        let version = storage.version.increment();
        let stored = StoredOperation::new(operation, version.clone(), storage.clock.clone());
        storage.operations.push(stored);

        Ok(version)
    }

    fn save_operations(
        &self,
        doc_id: &DocId,
        operations: Vec<CrdtOp>,
    ) -> StorageResult<Vec<Version>> {
        let mut docs = self.documents.write().unwrap();
        let doc_id_str = doc_id.to_string();

        let storage = docs.entry(doc_id_str).or_insert_with(DocumentStorage::new);

        let mut versions = Vec::with_capacity(operations.len());

        for operation in operations {
            // Update the vector clock
            let op_id = operation.id();
            let current_seq = storage.clock.get(op_id.client_id);
            if op_id.seq > current_seq {
                storage.clock.set(op_id.client_id, op_id.seq);
            }

            // Increment version and store
            let version = storage.version.increment();
            let stored = StoredOperation::new(operation, version.clone(), storage.clock.clone());
            storage.operations.push(stored);
            versions.push(version);
        }

        Ok(versions)
    }

    fn get_operations_since(
        &self,
        doc_id: &DocId,
        version: &Version,
    ) -> StorageResult<Vec<StoredOperation>> {
        let docs = self.documents.read().unwrap();
        let doc_id_str = doc_id.to_string();

        match docs.get(&doc_id_str) {
            Some(storage) => {
                let ops: Vec<StoredOperation> = storage
                    .operations
                    .iter()
                    .filter(|op| op.version > *version)
                    .cloned()
                    .collect();
                Ok(ops)
            }
            None => {
                // Document doesn't exist yet, return empty list
                // (this allows queries for non-existent documents without error)
                Ok(Vec::new())
            }
        }
    }

    fn get_latest_version(&self, doc_id: &DocId) -> StorageResult<Version> {
        let docs = self.documents.read().unwrap();
        let doc_id_str = doc_id.to_string();

        match docs.get(&doc_id_str) {
            Some(storage) => Ok(storage.version.clone()),
            None => Ok(Version::initial()),
        }
    }

    fn save_snapshot(&self, doc_id: &DocId, snapshot: Snapshot) -> StorageResult<()> {
        let mut docs = self.documents.write().unwrap();
        let doc_id_str = doc_id.to_string();

        let storage = docs.entry(doc_id_str).or_insert_with(DocumentStorage::new);
        storage.snapshot = Some(snapshot);

        Ok(())
    }

    fn get_latest_snapshot(&self, doc_id: &DocId) -> StorageResult<Option<Snapshot>> {
        let docs = self.documents.read().unwrap();
        let doc_id_str = doc_id.to_string();

        match docs.get(&doc_id_str) {
            Some(storage) => Ok(storage.snapshot.clone()),
            None => Ok(None),
        }
    }

    fn delete_document(&self, doc_id: &DocId) -> StorageResult<()> {
        let mut docs = self.documents.write().unwrap();
        let doc_id_str = doc_id.to_string();

        docs.remove(&doc_id_str);
        Ok(())
    }

    fn document_exists(&self, doc_id: &DocId) -> StorageResult<bool> {
        let docs = self.documents.read().unwrap();
        let doc_id_str = doc_id.to_string();

        Ok(docs.contains_key(&doc_id_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::VectorClock;
    use crate::op_id::{ClientId, OpId};
    use doc_model::NodeId;

    fn make_doc_id(id: &str) -> DocId {
        DocId::from(id)
    }

    fn make_text_insert(client: u64, seq: u64, parent_seq: u64, c: char) -> CrdtOp {
        CrdtOp::TextInsert {
            id: OpId::new(ClientId::new(client), seq),
            node_id: NodeId::new(),
            parent_op_id: OpId::new(ClientId::new(client), parent_seq),
            char: c,
        }
    }

    fn make_text_delete(client: u64, seq: u64, target_client: u64, target_seq: u64) -> CrdtOp {
        CrdtOp::TextDelete {
            id: OpId::new(ClientId::new(client), seq),
            target_id: OpId::new(ClientId::new(target_client), target_seq),
        }
    }

    #[test]
    fn test_new_store() {
        let store = MemoryOperationStore::new();
        assert_eq!(store.document_count(), 0);
    }

    #[test]
    fn test_save_operation() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");
        let op = make_text_insert(1, 1, 0, 'a');

        let version = store.save_operation(&doc_id, op).unwrap();

        assert_eq!(version.value(), 1);
        assert_eq!(store.document_count(), 1);
    }

    #[test]
    fn test_save_multiple_operations() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        let v1 = store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        let v2 = store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();
        let v3 = store
            .save_operation(&doc_id, make_text_insert(1, 3, 2, 'c'))
            .unwrap();

        assert_eq!(v1.value(), 1);
        assert_eq!(v2.value(), 2);
        assert_eq!(v3.value(), 3);
    }

    #[test]
    fn test_save_operations_batch() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        let ops = vec![
            make_text_insert(1, 1, 0, 'a'),
            make_text_insert(1, 2, 1, 'b'),
            make_text_insert(1, 3, 2, 'c'),
        ];

        let versions = store.save_operations(&doc_id, ops).unwrap();

        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].value(), 1);
        assert_eq!(versions[1].value(), 2);
        assert_eq!(versions[2].value(), 3);
    }

    #[test]
    fn test_get_operations_since() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 3, 2, 'c'))
            .unwrap();

        // Get all operations
        let all_ops = store
            .get_operations_since(&doc_id, &Version::initial())
            .unwrap();
        assert_eq!(all_ops.len(), 3);

        // Get operations since version 1
        let ops = store
            .get_operations_since(&doc_id, &Version::new(1))
            .unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].version.value(), 2);
        assert_eq!(ops[1].version.value(), 3);

        // Get operations since version 3 (should be empty)
        let ops = store
            .get_operations_since(&doc_id, &Version::new(3))
            .unwrap();
        assert_eq!(ops.len(), 0);
    }

    #[test]
    fn test_get_operations_since_nonexistent_doc() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("nonexistent");

        let ops = store
            .get_operations_since(&doc_id, &Version::initial())
            .unwrap();
        assert_eq!(ops.len(), 0);
    }

    #[test]
    fn test_get_latest_version() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Non-existent document returns version 0
        let version = store.get_latest_version(&doc_id).unwrap();
        assert_eq!(version.value(), 0);

        // After saving operations
        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();

        let version = store.get_latest_version(&doc_id).unwrap();
        assert_eq!(version.value(), 2);
    }

    #[test]
    fn test_save_and_get_snapshot() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // No snapshot initially
        let snapshot = store.get_latest_snapshot(&doc_id).unwrap();
        assert!(snapshot.is_none());

        // Save a snapshot
        let snapshot = Snapshot::new(Version::new(5), VectorClock::new(), vec![1, 2, 3, 4, 5]);
        store.save_snapshot(&doc_id, snapshot.clone()).unwrap();

        // Get the snapshot
        let retrieved = store.get_latest_snapshot(&doc_id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.version, Version::new(5));
        assert_eq!(retrieved.data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_snapshot_overwrite() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Save first snapshot
        let snapshot1 = Snapshot::new(Version::new(5), VectorClock::new(), vec![1, 2, 3]);
        store.save_snapshot(&doc_id, snapshot1).unwrap();

        // Save second snapshot (should overwrite)
        let snapshot2 = Snapshot::new(Version::new(10), VectorClock::new(), vec![4, 5, 6, 7]);
        store.save_snapshot(&doc_id, snapshot2).unwrap();

        // Get the snapshot - should be the second one
        let retrieved = store.get_latest_snapshot(&doc_id).unwrap().unwrap();
        assert_eq!(retrieved.version, Version::new(10));
        assert_eq!(retrieved.data, vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_delete_document() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Create document with operations and snapshot
        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_snapshot(&doc_id, Snapshot::new(Version::new(1), VectorClock::new(), vec![]))
            .unwrap();

        assert!(store.document_exists(&doc_id).unwrap());

        // Delete document
        store.delete_document(&doc_id).unwrap();

        // Verify deletion
        assert!(!store.document_exists(&doc_id).unwrap());
        assert!(store.get_latest_snapshot(&doc_id).unwrap().is_none());
        assert!(store
            .get_operations_since(&doc_id, &Version::initial())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_document_exists() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        assert!(!store.document_exists(&doc_id).unwrap());

        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();

        assert!(store.document_exists(&doc_id).unwrap());
    }

    #[test]
    fn test_get_all_operations() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();

        let ops = store.get_all_operations(&doc_id).unwrap();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_get_stats() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();
        store
            .save_snapshot(&doc_id, Snapshot::new(Version::new(2), VectorClock::new(), vec![1, 2]))
            .unwrap();

        let stats = store.get_stats(&doc_id).unwrap();
        assert_eq!(stats.operation_count, 2);
        assert_eq!(stats.latest_version.value(), 2);
        assert!(stats.has_snapshot);
        assert_eq!(stats.snapshot_version, Some(Version::new(2)));
    }

    #[test]
    fn test_multiple_documents() {
        let store = MemoryOperationStore::new();
        let doc1 = make_doc_id("doc1");
        let doc2 = make_doc_id("doc2");

        store
            .save_operation(&doc1, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc1, make_text_insert(1, 2, 1, 'b'))
            .unwrap();
        store
            .save_operation(&doc2, make_text_insert(2, 1, 0, 'x'))
            .unwrap();

        assert_eq!(store.document_count(), 2);
        assert_eq!(store.get_latest_version(&doc1).unwrap().value(), 2);
        assert_eq!(store.get_latest_version(&doc2).unwrap().value(), 1);
    }

    #[test]
    fn test_list_documents() {
        let store = MemoryOperationStore::new();

        store
            .save_operation(&make_doc_id("doc1"), make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&make_doc_id("doc2"), make_text_insert(2, 1, 0, 'b'))
            .unwrap();

        let docs = store.list_documents();
        assert_eq!(docs.len(), 2);
        assert!(docs.contains(&"doc1".to_string()));
        assert!(docs.contains(&"doc2".to_string()));
    }

    #[test]
    fn test_clear() {
        let store = MemoryOperationStore::new();

        store
            .save_operation(&make_doc_id("doc1"), make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&make_doc_id("doc2"), make_text_insert(2, 1, 0, 'b'))
            .unwrap();

        assert_eq!(store.document_count(), 2);

        store.clear();

        assert_eq!(store.document_count(), 0);
    }

    #[test]
    fn test_vector_clock_tracking() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Operations from multiple clients
        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(2, 1, 0, 'x'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();

        let ops = store.get_all_operations(&doc_id).unwrap();

        // Check that the vector clock is properly updated
        assert_eq!(ops[0].clock.get(ClientId::new(1)), 1);
        assert_eq!(ops[1].clock.get(ClientId::new(1)), 1);
        assert_eq!(ops[1].clock.get(ClientId::new(2)), 1);
        assert_eq!(ops[2].clock.get(ClientId::new(1)), 2);
        assert_eq!(ops[2].clock.get(ClientId::new(2)), 1);
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(MemoryOperationStore::new());
        let doc_id = make_doc_id("doc1");

        let mut handles = vec![];

        // Spawn multiple threads to write operations
        for client_id in 0..4 {
            let store_clone = Arc::clone(&store);
            let doc_id_clone = doc_id.clone();

            let handle = thread::spawn(move || {
                for seq in 1..=10 {
                    store_clone
                        .save_operation(
                            &doc_id_clone,
                            make_text_insert(client_id, seq, seq - 1, 'x'),
                        )
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all operations were saved
        let ops = store.get_all_operations(&doc_id).unwrap();
        assert_eq!(ops.len(), 40); // 4 clients * 10 operations
    }

    #[test]
    fn test_mixed_operation_types() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Insert operations
        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
            .unwrap();

        // Delete operation
        store
            .save_operation(&doc_id, make_text_delete(1, 3, 1, 2))
            .unwrap();

        let ops = store.get_all_operations(&doc_id).unwrap();
        assert_eq!(ops.len(), 3);

        // Verify operation types
        assert!(matches!(ops[0].operation, CrdtOp::TextInsert { .. }));
        assert!(matches!(ops[1].operation, CrdtOp::TextInsert { .. }));
        assert!(matches!(ops[2].operation, CrdtOp::TextDelete { .. }));
    }

    #[test]
    fn test_estimate_memory_usage() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Initial estimate should be 0
        assert_eq!(store.estimate_memory_usage(), 0);

        // Add some operations
        for i in 1..=10 {
            store
                .save_operation(&doc_id, make_text_insert(1, i, i - 1, 'a'))
                .unwrap();
        }

        // Add a snapshot
        store
            .save_snapshot(
                &doc_id,
                Snapshot::new(Version::new(10), VectorClock::new(), vec![0; 1000]),
            )
            .unwrap();

        let usage = store.estimate_memory_usage();
        assert!(usage > 0);
        assert!(usage >= 1000); // At least the snapshot data
    }

    #[test]
    fn test_get_snapshot_at_version() {
        let store = MemoryOperationStore::new();
        let doc_id = make_doc_id("doc1");

        // Save a snapshot at version 5
        store
            .save_snapshot(
                &doc_id,
                Snapshot::new(Version::new(5), VectorClock::new(), vec![1, 2, 3]),
            )
            .unwrap();

        // Get snapshot at version 5 (should succeed)
        let snapshot = store
            .get_snapshot_at_version(&doc_id, &Version::new(5))
            .unwrap();
        assert!(snapshot.is_some());

        // Get snapshot at version 3 (should fail - no snapshot at that version)
        let snapshot = store
            .get_snapshot_at_version(&doc_id, &Version::new(3))
            .unwrap();
        assert!(snapshot.is_none());
    }
}
