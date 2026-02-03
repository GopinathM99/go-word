//! Sync engine for CRDT operation synchronization.
//!
//! This module provides the sync engine that manages operation synchronization
//! between local clients and remote servers. It handles:
//!
//! - Queuing local operations for sending
//! - Batching operations for efficient network transmission
//! - Handling server acknowledgments
//! - Applying remote operations
//! - State persistence and recovery
//! - Managing multiple document sync sessions

use crate::clock::VectorClock;
use crate::op_id::{ClientId, OpId};
use crate::operation::{CrdtOp, OpBatch, OpLog};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// State of an operation in the sync pipeline
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpState {
    /// Queued locally, not yet sent
    Pending,
    /// Sent to server, awaiting acknowledgment
    Sent,
    /// Acknowledged by server
    Acknowledged,
}

/// Sync engine manages operation synchronization
pub struct SyncEngine {
    /// Client ID
    client_id: ClientId,
    /// Queue of pending local operations
    pending_queue: VecDeque<CrdtOp>,
    /// Operations sent but not yet acknowledged
    sent_ops: HashMap<OpId, CrdtOp>,
    /// The full operation log
    op_log: OpLog,
    /// Current vector clock
    vector_clock: VectorClock,
    /// Sequence number for batches
    batch_seq: u64,
    /// Batch window in milliseconds
    batch_window_ms: u64,
    /// Max batch size
    max_batch_size: usize,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            pending_queue: VecDeque::new(),
            sent_ops: HashMap::new(),
            op_log: OpLog::new(),
            vector_clock: VectorClock::new(),
            batch_seq: 0,
            batch_window_ms: 50,
            max_batch_size: 100,
        }
    }

    /// Get the client ID for this sync engine
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Queue a local operation for syncing
    pub fn queue_local(&mut self, op: CrdtOp) {
        // Update vector clock for local operation
        let op_id = op.id();
        let current = self.vector_clock.get(op_id.client_id);
        if op_id.seq > current {
            self.vector_clock.set(op_id.client_id, op_id.seq);
        }

        self.pending_queue.push_back(op.clone());
        self.op_log.add(op);
    }

    /// Get pending operations to send (creates a batch)
    pub fn get_pending_batch(&mut self) -> Option<OpBatch> {
        if self.pending_queue.is_empty() {
            return None;
        }

        let mut batch = OpBatch::new(self.client_id, self.batch_seq);
        self.batch_seq += 1;

        while let Some(op) = self.pending_queue.pop_front() {
            let op_id = op.id();
            batch.add(op.clone());
            self.sent_ops.insert(op_id, op);

            if batch.len() >= self.max_batch_size {
                break;
            }
        }

        batch.clock = self.vector_clock.clone();
        Some(batch)
    }

    /// Handle acknowledgment from server
    pub fn handle_ack(&mut self, op_ids: Vec<OpId>) {
        for op_id in op_ids {
            self.sent_ops.remove(&op_id);
        }
    }

    /// Apply remote operations
    ///
    /// Returns the IDs of operations that were successfully applied (not duplicates)
    pub fn apply_remote(&mut self, ops: Vec<CrdtOp>) -> Vec<OpId> {
        let mut applied = Vec::new();
        for op in ops {
            let op_id = op.id();
            if self.op_log.add(op) {
                self.vector_clock.increment(op_id.client_id);
                applied.push(op_id);
            }
        }
        applied
    }

    /// Get operations since a vector clock (for catch-up sync)
    pub fn ops_since(&self, clock: &VectorClock) -> Vec<&CrdtOp> {
        self.op_log.ops_since(clock)
    }

    /// Get current vector clock
    pub fn clock(&self) -> &VectorClock {
        &self.vector_clock
    }

    /// Check if there are pending operations
    pub fn has_pending(&self) -> bool {
        !self.pending_queue.is_empty() || !self.sent_ops.is_empty()
    }

    /// Get number of pending operations
    pub fn pending_count(&self) -> usize {
        self.pending_queue.len() + self.sent_ops.len()
    }

    /// Retry sent operations (after reconnect)
    pub fn retry_sent(&mut self) {
        for (_, op) in self.sent_ops.drain() {
            self.pending_queue.push_front(op);
        }
    }

    /// Save state for persistence
    pub fn save_state(&self) -> SyncState {
        SyncState {
            client_id: self.client_id,
            vector_clock: self.vector_clock.clone(),
            op_log: self.op_log.clone(),
            pending: self.pending_queue.iter().cloned().collect(),
            batch_seq: self.batch_seq,
        }
    }

    /// Restore state from persistence
    pub fn restore_state(state: SyncState) -> Self {
        Self {
            client_id: state.client_id,
            pending_queue: state.pending.into(),
            sent_ops: HashMap::new(),
            op_log: state.op_log,
            vector_clock: state.vector_clock,
            batch_seq: state.batch_seq,
            batch_window_ms: 50,
            max_batch_size: 100,
        }
    }

    /// Get the operation log
    pub fn op_log(&self) -> &OpLog {
        &self.op_log
    }

    /// Get the batch window in milliseconds
    pub fn batch_window_ms(&self) -> u64 {
        self.batch_window_ms
    }

    /// Set the batch window in milliseconds
    pub fn set_batch_window_ms(&mut self, ms: u64) {
        self.batch_window_ms = ms;
    }

    /// Get the max batch size
    pub fn max_batch_size(&self) -> usize {
        self.max_batch_size
    }

    /// Set the max batch size
    pub fn set_max_batch_size(&mut self, size: usize) {
        self.max_batch_size = size;
    }

    /// Get the number of sent but unacknowledged operations
    pub fn sent_count(&self) -> usize {
        self.sent_ops.len()
    }

    /// Get an operation from the log by ID
    pub fn get_op(&self, id: OpId) -> Option<&CrdtOp> {
        self.op_log.get(id)
    }
}

/// Serializable sync state for persistence
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncState {
    pub client_id: ClientId,
    pub vector_clock: VectorClock,
    pub op_log: OpLog,
    pub pending: Vec<CrdtOp>,
    pub batch_seq: u64,
}

/// Sync status for UI display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    pub connected: bool,
    pub pending_count: usize,
    pub last_sync_time: Option<u64>,
    pub is_syncing: bool,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            connected: false,
            pending_count: 0,
            last_sync_time: None,
            is_syncing: false,
        }
    }
}

/// Manages multiple document sync sessions
pub struct SyncManager {
    /// Active sync engines per document
    engines: HashMap<String, SyncEngine>,
    /// Client ID (shared across documents)
    client_id: ClientId,
    /// Connection status per document
    connection_status: HashMap<String, bool>,
    /// Last sync time per document
    last_sync_times: HashMap<String, u64>,
    /// Syncing status per document
    syncing_status: HashMap<String, bool>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(client_id: ClientId) -> Self {
        Self {
            engines: HashMap::new(),
            client_id,
            connection_status: HashMap::new(),
            last_sync_times: HashMap::new(),
            syncing_status: HashMap::new(),
        }
    }

    /// Get the client ID
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Get or create sync engine for a document
    pub fn get_engine(&mut self, doc_id: &str) -> &mut SyncEngine {
        self.engines
            .entry(doc_id.to_string())
            .or_insert_with(|| SyncEngine::new(self.client_id))
    }

    /// Get sync engine for a document (immutable)
    pub fn get_engine_ref(&self, doc_id: &str) -> Option<&SyncEngine> {
        self.engines.get(doc_id)
    }

    /// Remove sync engine for a document
    pub fn remove_engine(&mut self, doc_id: &str) {
        self.engines.remove(doc_id);
        self.connection_status.remove(doc_id);
        self.last_sync_times.remove(doc_id);
        self.syncing_status.remove(doc_id);
    }

    /// Set connection status for a document
    pub fn set_connected(&mut self, doc_id: &str, connected: bool) {
        self.connection_status.insert(doc_id.to_string(), connected);
    }

    /// Set last sync time for a document
    pub fn set_last_sync_time(&mut self, doc_id: &str, time: u64) {
        self.last_sync_times.insert(doc_id.to_string(), time);
    }

    /// Set syncing status for a document
    pub fn set_syncing(&mut self, doc_id: &str, syncing: bool) {
        self.syncing_status.insert(doc_id.to_string(), syncing);
    }

    /// Get sync status for all documents
    pub fn status(&self) -> HashMap<String, SyncStatus> {
        let mut result = HashMap::new();

        for (doc_id, engine) in &self.engines {
            let status = SyncStatus {
                connected: self.connection_status.get(doc_id).copied().unwrap_or(false),
                pending_count: engine.pending_count(),
                last_sync_time: self.last_sync_times.get(doc_id).copied(),
                is_syncing: self.syncing_status.get(doc_id).copied().unwrap_or(false),
            };
            result.insert(doc_id.clone(), status);
        }

        result
    }

    /// Get sync status for a specific document
    pub fn status_for(&self, doc_id: &str) -> SyncStatus {
        if let Some(engine) = self.engines.get(doc_id) {
            SyncStatus {
                connected: self.connection_status.get(doc_id).copied().unwrap_or(false),
                pending_count: engine.pending_count(),
                last_sync_time: self.last_sync_times.get(doc_id).copied(),
                is_syncing: self.syncing_status.get(doc_id).copied().unwrap_or(false),
            }
        } else {
            SyncStatus::default()
        }
    }

    /// Get the list of active document IDs
    pub fn active_documents(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }

    /// Check if a document has an active sync engine
    pub fn has_document(&self, doc_id: &str) -> bool {
        self.engines.contains_key(doc_id)
    }

    /// Retry sent operations for all documents (after reconnect)
    pub fn retry_all_sent(&mut self) {
        for engine in self.engines.values_mut() {
            engine.retry_sent();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt_tree::BlockData;
    use doc_model::NodeId;

    fn make_client_id(id: u64) -> ClientId {
        ClientId::new(id)
    }

    fn make_op_id(client_id: u64, seq: u64) -> OpId {
        OpId::new(client_id, seq)
    }

    fn make_text_insert(client_id: u64, seq: u64, parent_seq: u64, c: char) -> CrdtOp {
        CrdtOp::TextInsert {
            id: make_op_id(client_id, seq),
            node_id: NodeId::new(),
            parent_op_id: make_op_id(client_id, parent_seq),
            char: c,
        }
    }

    fn make_block_insert(client_id: u64, seq: u64, parent_seq: u64) -> CrdtOp {
        CrdtOp::BlockInsert {
            id: make_op_id(client_id, seq),
            parent_op_id: make_op_id(client_id, parent_seq),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        }
    }

    // ========== SyncEngine Tests ==========

    #[test]
    fn test_sync_engine_new() {
        let client_id = make_client_id(1);
        let engine = SyncEngine::new(client_id);

        assert_eq!(engine.client_id(), client_id);
        assert!(!engine.has_pending());
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_queue_local_and_get_pending_batch() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Queue some operations
        let op1 = make_text_insert(1, 1, 0, 'H');
        let op2 = make_text_insert(1, 2, 1, 'i');

        engine.queue_local(op1.clone());
        engine.queue_local(op2.clone());

        assert!(engine.has_pending());
        assert_eq!(engine.pending_count(), 2);

        // Get pending batch
        let batch = engine.get_pending_batch().unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch.client_id, client_id);
        assert_eq!(batch.batch_seq, 0);

        // Operations should now be in sent_ops
        assert_eq!(engine.sent_count(), 2);
        assert_eq!(engine.pending_count(), 2); // 2 sent, 0 pending
    }

    #[test]
    fn test_get_pending_batch_respects_max_size() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);
        engine.set_max_batch_size(2);

        // Queue more operations than max batch size
        for i in 1..=5 {
            engine.queue_local(make_text_insert(1, i, i - 1, 'a'));
        }

        assert_eq!(engine.pending_count(), 5);

        // First batch should have 2 operations
        let batch1 = engine.get_pending_batch().unwrap();
        assert_eq!(batch1.len(), 2);
        assert_eq!(batch1.batch_seq, 0);

        // Second batch should have 2 operations
        let batch2 = engine.get_pending_batch().unwrap();
        assert_eq!(batch2.len(), 2);
        assert_eq!(batch2.batch_seq, 1);

        // Third batch should have 1 operation
        let batch3 = engine.get_pending_batch().unwrap();
        assert_eq!(batch3.len(), 1);
        assert_eq!(batch3.batch_seq, 2);

        // No more pending operations to batch
        assert!(engine.get_pending_batch().is_none());
    }

    #[test]
    fn test_handle_ack() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Queue and send operations
        let op1 = make_text_insert(1, 1, 0, 'H');
        let op2 = make_text_insert(1, 2, 1, 'i');
        let op1_id = op1.id();
        let op2_id = op2.id();

        engine.queue_local(op1);
        engine.queue_local(op2);

        let _batch = engine.get_pending_batch().unwrap();
        assert_eq!(engine.sent_count(), 2);

        // Acknowledge first operation
        engine.handle_ack(vec![op1_id]);
        assert_eq!(engine.sent_count(), 1);

        // Acknowledge second operation
        engine.handle_ack(vec![op2_id]);
        assert_eq!(engine.sent_count(), 0);

        // No pending operations
        assert!(!engine.has_pending());
    }

    #[test]
    fn test_apply_remote_operations() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Remote operations from client 2
        let remote_op1 = make_text_insert(2, 1, 0, 'X');
        let remote_op2 = make_text_insert(2, 2, 1, 'Y');

        let applied = engine.apply_remote(vec![remote_op1.clone(), remote_op2.clone()]);

        // Both should be applied
        assert_eq!(applied.len(), 2);
        assert_eq!(applied[0], remote_op1.id());
        assert_eq!(applied[1], remote_op2.id());

        // Vector clock should be updated
        assert_eq!(engine.clock().get(make_client_id(2)), 2);

        // Operations should be in the log
        assert!(engine.get_op(remote_op1.id()).is_some());
        assert!(engine.get_op(remote_op2.id()).is_some());
    }

    #[test]
    fn test_apply_remote_duplicate_rejected() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        let remote_op = make_text_insert(2, 1, 0, 'X');

        // Apply once
        let applied1 = engine.apply_remote(vec![remote_op.clone()]);
        assert_eq!(applied1.len(), 1);

        // Apply again (duplicate)
        let applied2 = engine.apply_remote(vec![remote_op]);
        assert_eq!(applied2.len(), 0); // Should be rejected
    }

    #[test]
    fn test_retry_sent_on_reconnect() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Queue and send operations
        let op1 = make_text_insert(1, 1, 0, 'H');
        let op2 = make_text_insert(1, 2, 1, 'i');

        engine.queue_local(op1);
        engine.queue_local(op2);

        let _batch = engine.get_pending_batch().unwrap();
        assert_eq!(engine.sent_count(), 2);
        assert_eq!(engine.pending_count(), 2); // All in sent

        // Simulate disconnect - retry sent operations
        engine.retry_sent();

        assert_eq!(engine.sent_count(), 0);
        assert_eq!(engine.pending_count(), 2); // Back in pending queue

        // Should be able to get a new batch
        let retry_batch = engine.get_pending_batch().unwrap();
        assert_eq!(retry_batch.len(), 2);
    }

    #[test]
    fn test_save_and_restore_state() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Queue some operations
        let op1 = make_text_insert(1, 1, 0, 'H');
        let op2 = make_text_insert(1, 2, 1, 'i');

        engine.queue_local(op1);
        engine.queue_local(op2);

        // Get one batch
        let _batch = engine.get_pending_batch();

        // Save state
        let state = engine.save_state();

        assert_eq!(state.client_id, client_id);
        assert_eq!(state.batch_seq, 1); // One batch was created
        assert_eq!(state.op_log.len(), 2);

        // Restore state
        let restored = SyncEngine::restore_state(state);

        assert_eq!(restored.client_id(), client_id);
        assert_eq!(restored.op_log().len(), 2);
        assert_eq!(restored.clock().get(client_id), 2);
    }

    #[test]
    fn test_vector_clock_updates_on_local_ops() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Initially clock is empty
        assert_eq!(engine.clock().get(client_id), 0);

        // Queue operations
        engine.queue_local(make_text_insert(1, 1, 0, 'a'));
        assert_eq!(engine.clock().get(client_id), 1);

        engine.queue_local(make_text_insert(1, 2, 1, 'b'));
        assert_eq!(engine.clock().get(client_id), 2);

        // Out-of-order seq should still update if higher
        engine.queue_local(make_text_insert(1, 5, 2, 'c'));
        assert_eq!(engine.clock().get(client_id), 5);

        // Lower seq should not decrease the clock
        engine.queue_local(make_text_insert(1, 3, 5, 'd'));
        assert_eq!(engine.clock().get(client_id), 5);
    }

    #[test]
    fn test_ops_since() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Queue operations from two "clients" (simulated)
        engine.queue_local(make_text_insert(1, 1, 0, 'a'));
        engine.queue_local(make_text_insert(1, 2, 1, 'b'));

        // Apply remote operations
        engine.apply_remote(vec![
            make_text_insert(2, 1, 0, 'x'),
            make_text_insert(2, 2, 1, 'y'),
        ]);

        // Get ops since empty clock
        let empty_clock = VectorClock::new();
        let all_ops = engine.ops_since(&empty_clock);
        assert_eq!(all_ops.len(), 4);

        // Get ops since partial clock
        let mut partial_clock = VectorClock::new();
        partial_clock.set(make_client_id(1), 1);
        let some_ops = engine.ops_since(&partial_clock);
        assert_eq!(some_ops.len(), 3); // 1 from client 1, 2 from client 2
    }

    // ========== SyncManager Tests ==========

    #[test]
    fn test_sync_manager_new() {
        let client_id = make_client_id(42);
        let manager = SyncManager::new(client_id);

        assert_eq!(manager.client_id(), client_id);
        assert!(manager.active_documents().is_empty());
    }

    #[test]
    fn test_sync_manager_get_engine() {
        let client_id = make_client_id(1);
        let mut manager = SyncManager::new(client_id);

        // Get engine for new document
        let engine = manager.get_engine("doc1");
        assert_eq!(engine.client_id(), client_id);

        // Queue an operation
        engine.queue_local(make_text_insert(1, 1, 0, 'H'));
        assert_eq!(engine.pending_count(), 1);

        // Get same engine again
        let engine2 = manager.get_engine("doc1");
        assert_eq!(engine2.pending_count(), 1); // Same engine

        assert!(manager.has_document("doc1"));
        assert!(!manager.has_document("doc2"));
    }

    #[test]
    fn test_sync_manager_remove_engine() {
        let client_id = make_client_id(1);
        let mut manager = SyncManager::new(client_id);

        // Create engines
        manager.get_engine("doc1");
        manager.get_engine("doc2");

        assert_eq!(manager.active_documents().len(), 2);

        // Remove one
        manager.remove_engine("doc1");

        assert!(!manager.has_document("doc1"));
        assert!(manager.has_document("doc2"));
        assert_eq!(manager.active_documents().len(), 1);
    }

    #[test]
    fn test_sync_manager_status() {
        let client_id = make_client_id(1);
        let mut manager = SyncManager::new(client_id);

        // Create engine and queue operations
        let engine = manager.get_engine("doc1");
        engine.queue_local(make_text_insert(1, 1, 0, 'H'));
        engine.queue_local(make_text_insert(1, 2, 1, 'i'));

        // Set status
        manager.set_connected("doc1", true);
        manager.set_last_sync_time("doc1", 1234567890);
        manager.set_syncing("doc1", true);

        // Get status for specific document
        let status = manager.status_for("doc1");
        assert!(status.connected);
        assert_eq!(status.pending_count, 2);
        assert_eq!(status.last_sync_time, Some(1234567890));
        assert!(status.is_syncing);

        // Get all statuses
        let all_status = manager.status();
        assert_eq!(all_status.len(), 1);
        assert!(all_status.contains_key("doc1"));
    }

    #[test]
    fn test_sync_manager_status_for_unknown_doc() {
        let client_id = make_client_id(1);
        let manager = SyncManager::new(client_id);

        let status = manager.status_for("unknown");
        assert!(!status.connected);
        assert_eq!(status.pending_count, 0);
        assert!(status.last_sync_time.is_none());
        assert!(!status.is_syncing);
    }

    #[test]
    fn test_sync_manager_retry_all_sent() {
        let client_id = make_client_id(1);
        let mut manager = SyncManager::new(client_id);

        // Create two engines with sent operations
        {
            let engine1 = manager.get_engine("doc1");
            engine1.queue_local(make_text_insert(1, 1, 0, 'a'));
            let _batch1 = engine1.get_pending_batch();
        }

        {
            let engine2 = manager.get_engine("doc2");
            engine2.queue_local(make_text_insert(1, 1, 0, 'b'));
            let _batch2 = engine2.get_pending_batch();
        }

        // Both should have sent operations
        assert_eq!(manager.get_engine_ref("doc1").unwrap().sent_count(), 1);
        assert_eq!(manager.get_engine_ref("doc2").unwrap().sent_count(), 1);

        // Retry all
        manager.retry_all_sent();

        // All should be back in pending queue
        assert_eq!(manager.get_engine_ref("doc1").unwrap().sent_count(), 0);
        assert_eq!(manager.get_engine_ref("doc2").unwrap().sent_count(), 0);
    }

    #[test]
    fn test_op_state_enum() {
        let pending = OpState::Pending;
        let sent = OpState::Sent;
        let acked = OpState::Acknowledged;

        assert_eq!(pending, OpState::Pending);
        assert_eq!(sent, OpState::Sent);
        assert_eq!(acked, OpState::Acknowledged);

        assert_ne!(pending, sent);
        assert_ne!(sent, acked);
    }

    #[test]
    fn test_sync_status_default() {
        let status = SyncStatus::default();

        assert!(!status.connected);
        assert_eq!(status.pending_count, 0);
        assert!(status.last_sync_time.is_none());
        assert!(!status.is_syncing);
    }

    #[test]
    fn test_sync_engine_batch_window_settings() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Default values
        assert_eq!(engine.batch_window_ms(), 50);
        assert_eq!(engine.max_batch_size(), 100);

        // Change settings
        engine.set_batch_window_ms(100);
        engine.set_max_batch_size(50);

        assert_eq!(engine.batch_window_ms(), 100);
        assert_eq!(engine.max_batch_size(), 50);
    }

    #[test]
    fn test_sync_engine_empty_pending_batch() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // No operations queued
        assert!(engine.get_pending_batch().is_none());
    }

    #[test]
    fn test_sync_state_serialization() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        engine.queue_local(make_text_insert(1, 1, 0, 'H'));

        let state = engine.save_state();

        // Serialize
        let json = serde_json::to_string(&state).unwrap();

        // Deserialize
        let restored_state: SyncState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored_state.client_id, client_id);
        assert_eq!(restored_state.batch_seq, 0);
        assert_eq!(restored_state.pending.len(), 1);
    }

    #[test]
    fn test_block_operations_in_sync() {
        let client_id = make_client_id(1);
        let mut engine = SyncEngine::new(client_id);

        // Queue block operations
        let block_op = make_block_insert(1, 1, 0);
        engine.queue_local(block_op.clone());

        assert_eq!(engine.pending_count(), 1);

        let batch = engine.get_pending_batch().unwrap();
        assert_eq!(batch.len(), 1);

        // Apply remote block operation
        let remote_block = make_block_insert(2, 1, 0);
        let applied = engine.apply_remote(vec![remote_block]);
        assert_eq!(applied.len(), 1);
    }
}
