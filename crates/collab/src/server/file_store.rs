//! File-based operation store implementation.
//!
//! This module provides `FileOperationStore`, a persistent implementation of the
//! `OperationStore` trait that stores operations and snapshots as files on disk.
//!
//! # Directory Structure
//!
//! ```text
//! data/
//! └── {doc_id}/
//!     ├── operations.jsonl    # Append-only log of operations (JSON lines format)
//!     └── snapshot.json       # Latest document snapshot
//! ```
//!
//! # Features
//!
//! - Persistent storage across restarts
//! - Append-only operation log for durability
//! - JSON Lines format for efficient streaming reads
//! - Thread-safe with internal locking per document
//!
//! # Example
//!
//! ```ignore
//! use collab::server::file_store::FileOperationStore;
//! use collab::server::storage::OperationStore;
//! use std::path::Path;
//!
//! let store = FileOperationStore::new(Path::new("./data"))?;
//!
//! // Save an operation
//! let version = store.save_operation(&doc_id, operation)?;
//!
//! // Operations are persisted to disk immediately
//! ```

use crate::clock::VectorClock;
use crate::operation::CrdtOp;
use crate::permissions::DocId;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

use super::storage::{
    OperationStore, Snapshot, StorageError, StorageResult, StoredOperation, Version,
};

/// File names used for storage
const OPERATIONS_FILE: &str = "operations.jsonl";
const SNAPSHOT_FILE: &str = "snapshot.json";
const METADATA_FILE: &str = "metadata.json";

/// Metadata stored for each document
#[derive(serde::Serialize, serde::Deserialize)]
struct DocumentMetadata {
    version: Version,
    clock: VectorClock,
    operation_count: usize,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            version: Version::initial(),
            clock: VectorClock::new(),
            operation_count: 0,
        }
    }
}

/// Per-document lock for concurrent access
struct DocumentLock {
    lock: Mutex<()>,
    metadata: RwLock<DocumentMetadata>,
}

impl DocumentLock {
    fn new(metadata: DocumentMetadata) -> Self {
        Self {
            lock: Mutex::new(()),
            metadata: RwLock::new(metadata),
        }
    }
}

/// File-based implementation of `OperationStore`
///
/// This implementation stores operations and snapshots as files on disk.
/// Operations are stored in an append-only JSON Lines file for durability,
/// while snapshots are stored as separate JSON files.
///
/// # Thread Safety
///
/// The store is thread-safe and can be shared across threads using `Arc`.
/// Each document has its own lock to allow concurrent access to different
/// documents while serializing access to the same document.
///
/// # Durability
///
/// Operations are flushed to disk immediately after being written.
/// This ensures durability at the cost of some performance. For high-throughput
/// scenarios, consider batching operations before calling `save_operation`.
pub struct FileOperationStore {
    /// Base directory for all document data
    base_path: PathBuf,
    /// Per-document locks for concurrent access
    document_locks: RwLock<HashMap<String, DocumentLock>>,
}

impl FileOperationStore {
    /// Create a new file-based store at the given path
    ///
    /// Creates the base directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base directory for storing document data
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn new(base_path: impl AsRef<Path>) -> StorageResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;

        let store = Self {
            base_path,
            document_locks: RwLock::new(HashMap::new()),
        };

        // Load existing documents
        store.scan_existing_documents()?;

        Ok(store)
    }

    /// Scan the base directory for existing documents and load their metadata
    fn scan_existing_documents(&self) -> StorageResult<()> {
        if !self.base_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let doc_id = entry
                    .file_name()
                    .to_string_lossy()
                    .to_string();

                // Try to load metadata
                let metadata = self.load_metadata(&doc_id)?;
                let mut locks = self.document_locks.write().unwrap();
                locks.insert(doc_id, DocumentLock::new(metadata));
            }
        }

        Ok(())
    }

    /// Get the directory path for a document
    fn doc_path(&self, doc_id: &str) -> PathBuf {
        self.base_path.join(doc_id)
    }

    /// Get the operations file path for a document
    fn operations_path(&self, doc_id: &str) -> PathBuf {
        self.doc_path(doc_id).join(OPERATIONS_FILE)
    }

    /// Get the snapshot file path for a document
    fn snapshot_path(&self, doc_id: &str) -> PathBuf {
        self.doc_path(doc_id).join(SNAPSHOT_FILE)
    }

    /// Get the metadata file path for a document
    fn metadata_path(&self, doc_id: &str) -> PathBuf {
        self.doc_path(doc_id).join(METADATA_FILE)
    }

    /// Load or create document metadata
    fn load_metadata(&self, doc_id: &str) -> StorageResult<DocumentMetadata> {
        let metadata_path = self.metadata_path(doc_id);

        if metadata_path.exists() {
            let file = File::open(&metadata_path)?;
            let reader = BufReader::new(file);
            let metadata: DocumentMetadata = serde_json::from_reader(reader)?;
            Ok(metadata)
        } else {
            // If no metadata file, try to reconstruct from operations
            let ops_path = self.operations_path(doc_id);
            if ops_path.exists() {
                self.reconstruct_metadata(doc_id)
            } else {
                Ok(DocumentMetadata::default())
            }
        }
    }

    /// Reconstruct metadata by reading all operations
    fn reconstruct_metadata(&self, doc_id: &str) -> StorageResult<DocumentMetadata> {
        let ops_path = self.operations_path(doc_id);

        if !ops_path.exists() {
            return Ok(DocumentMetadata::default());
        }

        let file = File::open(&ops_path)?;
        let reader = BufReader::new(file);

        let mut metadata = DocumentMetadata::default();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let stored: StoredOperation = serde_json::from_str(&line)?;

            // Update version
            if stored.version > metadata.version {
                metadata.version = stored.version;
            }

            // Update clock
            let op_id = stored.operation.id();
            let current_seq = metadata.clock.get(op_id.client_id);
            if op_id.seq > current_seq {
                metadata.clock.set(op_id.client_id, op_id.seq);
            }

            metadata.operation_count += 1;
        }

        // Save the reconstructed metadata
        self.save_metadata(doc_id, &metadata)?;

        Ok(metadata)
    }

    /// Save document metadata
    fn save_metadata(&self, doc_id: &str, metadata: &DocumentMetadata) -> StorageResult<()> {
        let metadata_path = self.metadata_path(doc_id);
        let temp_path = metadata_path.with_extension("json.tmp");

        // Write to temp file first
        let file = File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, metadata)?;
        writer.flush()?;

        // Atomic rename
        fs::rename(temp_path, metadata_path)?;

        Ok(())
    }

    /// Ensure the document directory exists
    fn ensure_doc_dir(&self, doc_id: &str) -> StorageResult<()> {
        let doc_path = self.doc_path(doc_id);
        if !doc_path.exists() {
            fs::create_dir_all(&doc_path)?;
        }
        Ok(())
    }

    /// Get or create a document lock
    fn get_or_create_lock(&self, doc_id: &str) -> StorageResult<()> {
        let mut locks = self.document_locks.write().unwrap();
        if !locks.contains_key(doc_id) {
            let metadata = self.load_metadata(doc_id)?;
            locks.insert(doc_id.to_string(), DocumentLock::new(metadata));
        }
        Ok(())
    }

    /// Get a list of all document IDs in the store
    pub fn list_documents(&self) -> StorageResult<Vec<String>> {
        let mut docs = Vec::new();

        if !self.base_path.exists() {
            return Ok(docs);
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let doc_id = entry.file_name().to_string_lossy().to_string();
                docs.push(doc_id);
            }
        }

        Ok(docs)
    }

    /// Get the total size of stored data in bytes
    pub fn total_size(&self) -> StorageResult<u64> {
        let mut total = 0;

        for doc_id in self.list_documents()? {
            let ops_path = self.operations_path(&doc_id);
            if ops_path.exists() {
                total += fs::metadata(&ops_path)?.len();
            }

            let snapshot_path = self.snapshot_path(&doc_id);
            if snapshot_path.exists() {
                total += fs::metadata(&snapshot_path)?.len();
            }

            let metadata_path = self.metadata_path(&doc_id);
            if metadata_path.exists() {
                total += fs::metadata(&metadata_path)?.len();
            }
        }

        Ok(total)
    }

    /// Compact the operations file by rewriting it
    ///
    /// This is useful after many operations to reduce file fragmentation
    /// and potentially remove any corrupted entries.
    pub fn compact(&self, doc_id: &DocId) -> StorageResult<usize> {
        let doc_id_str = doc_id.to_string();
        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        let doc_lock = locks.get(&doc_id_str).ok_or_else(|| {
            StorageError::DocumentNotFound(doc_id_str.clone())
        })?;

        let _guard = doc_lock.lock.lock().unwrap();

        // Read all operations
        let ops = self.read_operations_unlocked(&doc_id_str)?;
        let count = ops.len();

        if count == 0 {
            return Ok(0);
        }

        // Rewrite the file
        let ops_path = self.operations_path(&doc_id_str);
        let temp_path = ops_path.with_extension("jsonl.tmp");

        {
            let file = File::create(&temp_path)?;
            let mut writer = BufWriter::new(file);

            for op in &ops {
                serde_json::to_writer(&mut writer, op)?;
                writeln!(writer)?;
            }

            writer.flush()?;
        }

        // Atomic rename
        fs::rename(temp_path, ops_path)?;

        Ok(count)
    }

    /// Read operations without holding a lock (internal use)
    fn read_operations_unlocked(&self, doc_id: &str) -> StorageResult<Vec<StoredOperation>> {
        let ops_path = self.operations_path(doc_id);

        if !ops_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&ops_path)?;
        let reader = BufReader::new(file);
        let mut operations = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let stored: StoredOperation = serde_json::from_str(&line).map_err(|e| {
                StorageError::DeserializationError(format!(
                    "Failed to parse operation: {}",
                    e
                ))
            })?;
            operations.push(stored);
        }

        Ok(operations)
    }
}

impl OperationStore for FileOperationStore {
    fn save_operation(&self, doc_id: &DocId, operation: CrdtOp) -> StorageResult<Version> {
        let doc_id_str = doc_id.to_string();

        // Ensure directory and lock exist
        self.ensure_doc_dir(&doc_id_str)?;
        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        let doc_lock = locks.get(&doc_id_str).ok_or_else(|| {
            StorageError::InternalError("Document lock not found".to_string())
        })?;

        // Acquire the document lock
        let _guard = doc_lock.lock.lock().unwrap();

        // Update metadata
        let version = {
            let mut metadata = doc_lock.metadata.write().unwrap();

            // Update clock
            let op_id = operation.id();
            let current_seq = metadata.clock.get(op_id.client_id);
            if op_id.seq > current_seq {
                metadata.clock.set(op_id.client_id, op_id.seq);
            }

            // Increment version
            let version = metadata.version.increment();
            metadata.operation_count += 1;

            version
        };

        // Create stored operation
        let stored = {
            let metadata = doc_lock.metadata.read().unwrap();
            StoredOperation::new(operation, version.clone(), metadata.clock.clone())
        };

        // Append to operations file
        let ops_path = self.operations_path(&doc_id_str);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ops_path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &stored)?;
        writeln!(writer)?;
        writer.flush()?;

        // Save metadata
        {
            let metadata = doc_lock.metadata.read().unwrap();
            self.save_metadata(&doc_id_str, &metadata)?;
        }

        Ok(version)
    }

    fn save_operations(
        &self,
        doc_id: &DocId,
        operations: Vec<CrdtOp>,
    ) -> StorageResult<Vec<Version>> {
        if operations.is_empty() {
            return Ok(Vec::new());
        }

        let doc_id_str = doc_id.to_string();

        // Ensure directory and lock exist
        self.ensure_doc_dir(&doc_id_str)?;
        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        let doc_lock = locks.get(&doc_id_str).ok_or_else(|| {
            StorageError::InternalError("Document lock not found".to_string())
        })?;

        // Acquire the document lock
        let _guard = doc_lock.lock.lock().unwrap();

        let mut versions = Vec::with_capacity(operations.len());
        let mut stored_ops = Vec::with_capacity(operations.len());

        // Prepare all operations
        {
            let mut metadata = doc_lock.metadata.write().unwrap();

            for operation in operations {
                // Update clock
                let op_id = operation.id();
                let current_seq = metadata.clock.get(op_id.client_id);
                if op_id.seq > current_seq {
                    metadata.clock.set(op_id.client_id, op_id.seq);
                }

                // Increment version
                let version = metadata.version.increment();
                metadata.operation_count += 1;

                let stored = StoredOperation::new(operation, version.clone(), metadata.clock.clone());
                versions.push(version);
                stored_ops.push(stored);
            }
        }

        // Write all operations to file
        let ops_path = self.operations_path(&doc_id_str);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ops_path)?;

        let mut writer = BufWriter::new(file);
        for stored in &stored_ops {
            serde_json::to_writer(&mut writer, stored)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Save metadata
        {
            let metadata = doc_lock.metadata.read().unwrap();
            self.save_metadata(&doc_id_str, &metadata)?;
        }

        Ok(versions)
    }

    fn get_operations_since(
        &self,
        doc_id: &DocId,
        version: &Version,
    ) -> StorageResult<Vec<StoredOperation>> {
        let doc_id_str = doc_id.to_string();
        let ops_path = self.operations_path(&doc_id_str);

        if !ops_path.exists() {
            return Ok(Vec::new());
        }

        // Get or create lock for thread safety
        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        if let Some(doc_lock) = locks.get(&doc_id_str) {
            let _guard = doc_lock.lock.lock().unwrap();

            let file = File::open(&ops_path)?;
            let reader = BufReader::new(file);
            let mut operations = Vec::new();

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                let stored: StoredOperation = serde_json::from_str(&line).map_err(|e| {
                    StorageError::DeserializationError(format!(
                        "Failed to parse operation: {}",
                        e
                    ))
                })?;

                if stored.version > *version {
                    operations.push(stored);
                }
            }

            Ok(operations)
        } else {
            Ok(Vec::new())
        }
    }

    fn get_latest_version(&self, doc_id: &DocId) -> StorageResult<Version> {
        let doc_id_str = doc_id.to_string();
        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        if let Some(doc_lock) = locks.get(&doc_id_str) {
            let metadata = doc_lock.metadata.read().unwrap();
            Ok(metadata.version.clone())
        } else {
            Ok(Version::initial())
        }
    }

    fn save_snapshot(&self, doc_id: &DocId, snapshot: Snapshot) -> StorageResult<()> {
        let doc_id_str = doc_id.to_string();

        // Ensure directory exists
        self.ensure_doc_dir(&doc_id_str)?;
        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        let doc_lock = locks.get(&doc_id_str).ok_or_else(|| {
            StorageError::InternalError("Document lock not found".to_string())
        })?;

        let _guard = doc_lock.lock.lock().unwrap();

        let snapshot_path = self.snapshot_path(&doc_id_str);
        let temp_path = snapshot_path.with_extension("json.tmp");

        // Write to temp file first
        {
            let file = File::create(&temp_path)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, &snapshot)?;
            writer.flush()?;
        }

        // Atomic rename
        fs::rename(temp_path, snapshot_path)?;

        Ok(())
    }

    fn get_latest_snapshot(&self, doc_id: &DocId) -> StorageResult<Option<Snapshot>> {
        let doc_id_str = doc_id.to_string();
        let snapshot_path = self.snapshot_path(&doc_id_str);

        if !snapshot_path.exists() {
            return Ok(None);
        }

        self.get_or_create_lock(&doc_id_str)?;

        let locks = self.document_locks.read().unwrap();
        if let Some(doc_lock) = locks.get(&doc_id_str) {
            let _guard = doc_lock.lock.lock().unwrap();

            let file = File::open(&snapshot_path)?;
            let reader = BufReader::new(file);
            let snapshot: Snapshot = serde_json::from_reader(reader)?;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    fn delete_document(&self, doc_id: &DocId) -> StorageResult<()> {
        let doc_id_str = doc_id.to_string();
        let doc_path = self.doc_path(&doc_id_str);

        if doc_path.exists() {
            // Get lock first to ensure no concurrent access
            self.get_or_create_lock(&doc_id_str)?;

            {
                let locks = self.document_locks.read().unwrap();
                if let Some(doc_lock) = locks.get(&doc_id_str) {
                    let _guard = doc_lock.lock.lock().unwrap();
                    fs::remove_dir_all(&doc_path)?;
                }
            }

            // Remove from locks
            let mut locks = self.document_locks.write().unwrap();
            locks.remove(&doc_id_str);
        }

        Ok(())
    }

    fn document_exists(&self, doc_id: &DocId) -> StorageResult<bool> {
        let doc_id_str = doc_id.to_string();
        let doc_path = self.doc_path(&doc_id_str);
        Ok(doc_path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op_id::{ClientId, OpId};
    use doc_model::NodeId;
    use tempfile::TempDir;

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

    fn create_temp_store() -> (FileOperationStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = FileOperationStore::new(temp_dir.path()).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn test_new_store() {
        let (_store, _temp_dir) = create_temp_store();
        // Store should be created successfully
    }

    #[test]
    fn test_save_operation() {
        let (store, _temp_dir) = create_temp_store();
        let doc_id = make_doc_id("doc1");
        let op = make_text_insert(1, 1, 0, 'a');

        let version = store.save_operation(&doc_id, op).unwrap();

        assert_eq!(version.value(), 1);
    }

    #[test]
    fn test_save_multiple_operations() {
        let (store, _temp_dir) = create_temp_store();
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
        let (store, _temp_dir) = create_temp_store();
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
        let (store, _temp_dir) = create_temp_store();
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
    }

    #[test]
    fn test_get_latest_version() {
        let (store, _temp_dir) = create_temp_store();
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
        let (store, _temp_dir) = create_temp_store();
        let doc_id = make_doc_id("doc1");

        // No snapshot initially
        let snapshot = store.get_latest_snapshot(&doc_id).unwrap();
        assert!(snapshot.is_none());

        // Save a snapshot
        let snapshot = Snapshot::new(Version::new(5), VectorClock::new(), vec![1, 2, 3, 4, 5]);
        store.save_snapshot(&doc_id, snapshot).unwrap();

        // Get the snapshot
        let retrieved = store.get_latest_snapshot(&doc_id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.version, Version::new(5));
        assert_eq!(retrieved.data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_delete_document() {
        let (store, _temp_dir) = create_temp_store();
        let doc_id = make_doc_id("doc1");

        // Create document
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
    }

    #[test]
    fn test_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = make_doc_id("doc1");

        // First instance: save operations
        {
            let store = FileOperationStore::new(temp_dir.path()).unwrap();
            store
                .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
                .unwrap();
            store
                .save_operation(&doc_id, make_text_insert(1, 2, 1, 'b'))
                .unwrap();
            store
                .save_snapshot(&doc_id, Snapshot::new(Version::new(2), VectorClock::new(), vec![1, 2]))
                .unwrap();
        }

        // Second instance: read operations
        {
            let store = FileOperationStore::new(temp_dir.path()).unwrap();

            let version = store.get_latest_version(&doc_id).unwrap();
            assert_eq!(version.value(), 2);

            let ops = store.get_all_operations(&doc_id).unwrap();
            assert_eq!(ops.len(), 2);

            let snapshot = store.get_latest_snapshot(&doc_id).unwrap();
            assert!(snapshot.is_some());
            assert_eq!(snapshot.unwrap().data, vec![1, 2]);
        }
    }

    #[test]
    fn test_list_documents() {
        let (store, _temp_dir) = create_temp_store();

        store
            .save_operation(&make_doc_id("doc1"), make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_operation(&make_doc_id("doc2"), make_text_insert(2, 1, 0, 'b'))
            .unwrap();

        let docs = store.list_documents().unwrap();
        assert_eq!(docs.len(), 2);
        assert!(docs.contains(&"doc1".to_string()));
        assert!(docs.contains(&"doc2".to_string()));
    }

    #[test]
    fn test_total_size() {
        let (store, _temp_dir) = create_temp_store();
        let doc_id = make_doc_id("doc1");

        // Initial size should be 0
        let size = store.total_size().unwrap();
        assert_eq!(size, 0);

        // Add some data
        store
            .save_operation(&doc_id, make_text_insert(1, 1, 0, 'a'))
            .unwrap();
        store
            .save_snapshot(&doc_id, Snapshot::new(Version::new(1), VectorClock::new(), vec![0; 100]))
            .unwrap();

        let size = store.total_size().unwrap();
        assert!(size > 100);
    }

    #[test]
    fn test_compact() {
        let (store, _temp_dir) = create_temp_store();
        let doc_id = make_doc_id("doc1");

        // Add operations
        for i in 1..=10 {
            store
                .save_operation(&doc_id, make_text_insert(1, i, i - 1, 'a'))
                .unwrap();
        }

        // Compact
        let count = store.compact(&doc_id).unwrap();
        assert_eq!(count, 10);

        // Verify operations still exist
        let ops = store.get_all_operations(&doc_id).unwrap();
        assert_eq!(ops.len(), 10);
    }

    #[test]
    fn test_multiple_documents() {
        let (store, _temp_dir) = create_temp_store();
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

        assert_eq!(store.get_latest_version(&doc1).unwrap().value(), 2);
        assert_eq!(store.get_latest_version(&doc2).unwrap().value(), 1);
    }

    #[test]
    fn test_vector_clock_tracking() {
        let (store, _temp_dir) = create_temp_store();
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

        // Check vector clock progression
        assert_eq!(ops[0].clock.get(ClientId::new(1)), 1);
        assert_eq!(ops[1].clock.get(ClientId::new(1)), 1);
        assert_eq!(ops[1].clock.get(ClientId::new(2)), 1);
        assert_eq!(ops[2].clock.get(ClientId::new(1)), 2);
        assert_eq!(ops[2].clock.get(ClientId::new(2)), 1);
    }

    #[test]
    fn test_snapshot_with_description() {
        let (store, _temp_dir) = create_temp_store();
        let doc_id = make_doc_id("doc1");

        let snapshot = Snapshot::with_description(
            Version::new(10),
            VectorClock::new(),
            vec![1, 2, 3],
            "Test snapshot",
        );
        store.save_snapshot(&doc_id, snapshot).unwrap();

        let retrieved = store.get_latest_snapshot(&doc_id).unwrap().unwrap();
        assert_eq!(retrieved.description, Some("Test snapshot".to_string()));
    }

    #[test]
    fn test_get_stats() {
        let (store, _temp_dir) = create_temp_store();
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
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(FileOperationStore::new(temp_dir.path()).unwrap());
        let doc_id = make_doc_id("doc1");

        let mut handles = vec![];

        // Spawn multiple threads
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
        assert_eq!(ops.len(), 40);
    }

    #[test]
    fn test_metadata_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = make_doc_id("doc1");

        // First instance
        {
            let store = FileOperationStore::new(temp_dir.path()).unwrap();
            for i in 1..=5 {
                store
                    .save_operation(&doc_id, make_text_insert(1, i, i - 1, 'a'))
                    .unwrap();
            }
        }

        // Second instance should have correct metadata
        {
            let store = FileOperationStore::new(temp_dir.path()).unwrap();
            let version = store.get_latest_version(&doc_id).unwrap();
            assert_eq!(version.value(), 5);
        }
    }
}
