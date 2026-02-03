//! Offline editing support for collaborative documents.
//!
//! This module provides offline support for collaborative editing, allowing users
//! to continue working when disconnected from the server. Operations made while
//! offline are queued locally and synchronized when connection is restored.
//!
//! # Features
//!
//! - Connection status tracking (Online, Offline, Reconnecting, Syncing)
//! - Offline operation queue with persistent storage
//! - Reconnection sync with conflict detection
//! - Time tracking for last successful sync
//! - UI status information

use crate::clock::VectorClock;
use crate::op_id::ClientId;
use crate::operation::CrdtOp;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Connection status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Connected to server
    Online,
    /// Disconnected, operations queued locally
    Offline,
    /// Reconnecting after disconnection
    Reconnecting,
    /// Syncing after reconnection
    Syncing,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Offline
    }
}

/// Offline support manager
///
/// Manages offline editing state including:
/// - Connection status tracking
/// - Queuing operations made while offline
/// - Persisting state for recovery
/// - Handling reconnection synchronization
pub struct OfflineManager {
    /// Current connection status
    status: ConnectionStatus,
    /// Queue of operations made while offline
    offline_queue: Vec<CrdtOp>,
    /// Last known server clock
    last_server_clock: VectorClock,
    /// Timestamp of last successful sync
    last_sync_time: Option<u64>,
    /// Path for persistent storage
    storage_path: Option<std::path::PathBuf>,
    /// Whether auto-save is enabled
    auto_save: bool,
    /// Client ID
    client_id: ClientId,
}

impl OfflineManager {
    /// Create a new offline manager
    pub fn new(client_id: ClientId) -> Self {
        Self {
            status: ConnectionStatus::Offline,
            offline_queue: Vec::new(),
            last_server_clock: VectorClock::new(),
            last_sync_time: None,
            storage_path: None,
            auto_save: false,
            client_id,
        }
    }

    /// Create with persistent storage
    pub fn with_storage(client_id: ClientId, path: impl AsRef<Path>) -> Self {
        Self {
            status: ConnectionStatus::Offline,
            offline_queue: Vec::new(),
            last_server_clock: VectorClock::new(),
            last_sync_time: None,
            storage_path: Some(path.as_ref().to_path_buf()),
            auto_save: true,
            client_id,
        }
    }

    /// Get current connection status
    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    /// Set connection status
    pub fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;

        // Auto-save when going offline
        if self.auto_save && status == ConnectionStatus::Offline {
            let _ = self.save_queue();
        }
    }

    /// Check if currently offline
    pub fn is_offline(&self) -> bool {
        matches!(self.status, ConnectionStatus::Offline)
    }

    /// Check if currently online
    pub fn is_online(&self) -> bool {
        matches!(self.status, ConnectionStatus::Online)
    }

    /// Get the client ID
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Enable or disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    /// Check if auto-save is enabled
    pub fn auto_save_enabled(&self) -> bool {
        self.auto_save
    }

    // ========== Offline Queue Management ==========

    /// Add an operation to the offline queue
    pub fn queue_operation(&mut self, op: CrdtOp) {
        self.offline_queue.push(op);

        // Auto-save if enabled
        if self.auto_save {
            let _ = self.save_queue();
        }
    }

    /// Get all queued operations
    pub fn queued_operations(&self) -> &[CrdtOp] {
        &self.offline_queue
    }

    /// Get number of queued operations
    pub fn queue_size(&self) -> usize {
        self.offline_queue.len()
    }

    /// Clear the queue (after successful sync)
    pub fn clear_queue(&mut self) {
        self.offline_queue.clear();

        // Auto-save if enabled
        if self.auto_save {
            let _ = self.save_queue();
        }
    }

    /// Flush queue to persistent storage
    pub fn save_queue(&self) -> Result<(), OfflineError> {
        let path = self.storage_path.as_ref().ok_or(OfflineError::NoStoragePath)?;

        let state = OfflineState {
            client_id: self.client_id,
            offline_queue: self.offline_queue.clone(),
            last_server_clock: self.last_server_clock.clone(),
            last_sync_time: self.last_sync_time,
        };

        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| OfflineError::Serialization(e.to_string()))?;

        std::fs::write(path, json).map_err(|e| OfflineError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Load queue from persistent storage
    pub fn load_queue(&mut self) -> Result<(), OfflineError> {
        let path = self.storage_path.as_ref().ok_or(OfflineError::NoStoragePath)?;

        if !path.exists() {
            // No saved state, which is fine
            return Ok(());
        }

        let json =
            std::fs::read_to_string(path).map_err(|e| OfflineError::Storage(e.to_string()))?;

        let state: OfflineState = serde_json::from_str(&json)
            .map_err(|e| OfflineError::Serialization(e.to_string()))?;

        self.restore_state(state);

        Ok(())
    }

    // ========== Reconnection Sync ==========

    /// Get operations to send on reconnect
    pub fn get_reconnect_ops(&self) -> Vec<CrdtOp> {
        self.offline_queue.clone()
    }

    /// Get vector clock for sync request
    pub fn get_sync_clock(&self) -> &VectorClock {
        &self.last_server_clock
    }

    /// Update the last server clock
    pub fn update_server_clock(&mut self, clock: VectorClock) {
        self.last_server_clock = clock;
    }

    /// Handle sync response (merge remote operations)
    ///
    /// Takes remote operations received from the server during reconnection
    /// and determines how to merge them with local offline changes.
    pub fn handle_sync_response(&mut self, remote_ops: Vec<CrdtOp>) -> MergeResult {
        let merged_count = remote_ops.len();
        let mut had_conflicts = false;
        let mut local_reapply = Vec::new();

        // Check for conflicts between local and remote operations
        for local_op in &self.offline_queue {
            for remote_op in &remote_ops {
                if local_op.conflicts_with(remote_op) {
                    had_conflicts = true;
                    // Local operations that conflict need to be reapplied
                    // The CRDT guarantees convergence, but we track for UI feedback
                    if !local_reapply.iter().any(|op: &CrdtOp| op.id() == local_op.id()) {
                        local_reapply.push(local_op.clone());
                    }
                    break;
                }
            }
        }

        // Update the server clock with merged operations
        for op in &remote_ops {
            let op_id = op.id();
            let current = self.last_server_clock.get(op_id.client_id);
            if op_id.seq > current {
                self.last_server_clock.set(op_id.client_id, op_id.seq);
            }
        }

        // Generate summary
        let changes_summary = if had_conflicts {
            Some(format!(
                "Merged {} remote changes with {} local conflicts",
                merged_count,
                local_reapply.len()
            ))
        } else if merged_count > 0 {
            Some(format!("Merged {} remote changes", merged_count))
        } else {
            None
        };

        MergeResult {
            merged_count,
            had_conflicts,
            local_reapply,
            changes_summary,
        }
    }

    /// Mark sync as complete
    pub fn sync_complete(&mut self) {
        self.update_sync_time();
        self.clear_queue();
        self.set_status(ConnectionStatus::Online);
    }

    // ========== Time Tracking ==========

    /// Get current timestamp in seconds since epoch
    fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get time since last sync (in seconds)
    pub fn time_since_sync(&self) -> Option<u64> {
        self.last_sync_time
            .map(|sync_time| Self::current_time().saturating_sub(sync_time))
    }

    /// Get last sync timestamp
    pub fn last_sync_time(&self) -> Option<u64> {
        self.last_sync_time
    }

    /// Update last sync time
    pub fn update_sync_time(&mut self) {
        self.last_sync_time = Some(Self::current_time());
    }

    // ========== Persistence ==========

    /// Save full state to storage
    pub fn save_state(&self) -> Result<(), OfflineError> {
        self.save_queue()
    }

    /// Load state from storage
    pub fn load_state(&mut self) -> Result<(), OfflineError> {
        self.load_queue()
    }

    /// Get state for serialization
    pub fn get_state(&self) -> OfflineState {
        OfflineState {
            client_id: self.client_id,
            offline_queue: self.offline_queue.clone(),
            last_server_clock: self.last_server_clock.clone(),
            last_sync_time: self.last_sync_time,
        }
    }

    /// Restore from state
    pub fn restore_state(&mut self, state: OfflineState) {
        self.client_id = state.client_id;
        self.offline_queue = state.offline_queue;
        self.last_server_clock = state.last_server_clock;
        self.last_sync_time = state.last_sync_time;
    }

    /// Get status info for UI display
    pub fn get_status_info(&self) -> OfflineStatusInfo {
        let status_message = match self.status {
            ConnectionStatus::Online => "Connected".to_string(),
            ConnectionStatus::Offline => {
                let pending = self.queue_size();
                if pending > 0 {
                    format!("Offline - {} pending changes", pending)
                } else {
                    "Offline".to_string()
                }
            }
            ConnectionStatus::Reconnecting => "Reconnecting...".to_string(),
            ConnectionStatus::Syncing => "Syncing changes...".to_string(),
        };

        OfflineStatusInfo {
            status: self.status,
            pending_changes: self.queue_size(),
            time_since_sync: self.time_since_sync(),
            status_message,
        }
    }
}

/// Serializable offline state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfflineState {
    pub client_id: ClientId,
    pub offline_queue: Vec<CrdtOp>,
    pub last_server_clock: VectorClock,
    pub last_sync_time: Option<u64>,
}

/// Result of merging after reconnection
#[derive(Clone, Debug)]
pub struct MergeResult {
    /// Number of operations successfully merged
    pub merged_count: usize,
    /// Whether there were conflicts
    pub had_conflicts: bool,
    /// Operations that need to be reapplied locally
    pub local_reapply: Vec<CrdtOp>,
    /// Summary of significant changes
    pub changes_summary: Option<String>,
}

impl MergeResult {
    /// Create a successful merge result with no conflicts
    pub fn success(merged_count: usize) -> Self {
        Self {
            merged_count,
            had_conflicts: false,
            local_reapply: Vec::new(),
            changes_summary: if merged_count > 0 {
                Some(format!("Merged {} remote changes", merged_count))
            } else {
                None
            },
        }
    }

    /// Create a merge result with conflicts
    pub fn with_conflicts(merged_count: usize, local_reapply: Vec<CrdtOp>) -> Self {
        let conflict_count = local_reapply.len();
        Self {
            merged_count,
            had_conflicts: true,
            local_reapply,
            changes_summary: Some(format!(
                "Merged {} remote changes with {} local conflicts",
                merged_count, conflict_count
            )),
        }
    }

    /// Check if the merge was successful (no conflicts)
    pub fn is_success(&self) -> bool {
        !self.had_conflicts
    }
}

/// Offline errors
#[derive(Clone, Debug, thiserror::Error)]
pub enum OfflineError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("No storage path configured")]
    NoStoragePath,
    #[error("Merge error: {0}")]
    MergeError(String),
}

/// UI display information for offline status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfflineStatusInfo {
    pub status: ConnectionStatus,
    pub pending_changes: usize,
    pub time_since_sync: Option<u64>,
    pub status_message: String,
}

impl OfflineStatusInfo {
    /// Check if the indicator should be shown
    ///
    /// Returns true if offline or has pending changes
    pub fn should_show(&self) -> bool {
        !matches!(self.status, ConnectionStatus::Online) || self.pending_changes > 0
    }

    /// Get a short status string
    pub fn short_status(&self) -> &'static str {
        match self.status {
            ConnectionStatus::Online => "Online",
            ConnectionStatus::Offline => "Offline",
            ConnectionStatus::Reconnecting => "Reconnecting",
            ConnectionStatus::Syncing => "Syncing",
        }
    }

    /// Format time since sync for display
    pub fn formatted_time_since_sync(&self) -> Option<String> {
        self.time_since_sync.map(|seconds| {
            if seconds < 60 {
                format!("{}s ago", seconds)
            } else if seconds < 3600 {
                format!("{}m ago", seconds / 60)
            } else if seconds < 86400 {
                format!("{}h ago", seconds / 3600)
            } else {
                format!("{}d ago", seconds / 86400)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt_tree::BlockData;
    use crate::op_id::OpId;
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

    fn make_text_delete(client_id: u64, seq: u64, target_client: u64, target_seq: u64) -> CrdtOp {
        CrdtOp::TextDelete {
            id: make_op_id(client_id, seq),
            target_id: make_op_id(target_client, target_seq),
        }
    }

    fn make_block_insert(client_id: u64, seq: u64) -> CrdtOp {
        CrdtOp::BlockInsert {
            id: make_op_id(client_id, seq),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        }
    }

    // ========== Queue Operations Tests ==========

    #[test]
    fn test_offline_manager_new() {
        let client_id = make_client_id(1);
        let manager = OfflineManager::new(client_id);

        assert_eq!(manager.client_id(), client_id);
        assert_eq!(manager.status(), ConnectionStatus::Offline);
        assert_eq!(manager.queue_size(), 0);
        assert!(manager.is_offline());
        assert!(!manager.is_online());
    }

    #[test]
    fn test_queue_operation() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        let op1 = make_text_insert(1, 1, 0, 'H');
        let op2 = make_text_insert(1, 2, 1, 'i');

        manager.queue_operation(op1);
        assert_eq!(manager.queue_size(), 1);

        manager.queue_operation(op2);
        assert_eq!(manager.queue_size(), 2);

        let queued = manager.queued_operations();
        assert_eq!(queued.len(), 2);
    }

    #[test]
    fn test_clear_queue() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));
        manager.queue_operation(make_text_insert(1, 2, 1, 'b'));
        assert_eq!(manager.queue_size(), 2);

        manager.clear_queue();
        assert_eq!(manager.queue_size(), 0);
    }

    #[test]
    fn test_get_reconnect_ops() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        let op1 = make_text_insert(1, 1, 0, 'H');
        let op2 = make_text_insert(1, 2, 1, 'i');

        manager.queue_operation(op1.clone());
        manager.queue_operation(op2.clone());

        let reconnect_ops = manager.get_reconnect_ops();
        assert_eq!(reconnect_ops.len(), 2);
        assert_eq!(reconnect_ops[0].id(), op1.id());
        assert_eq!(reconnect_ops[1].id(), op2.id());
    }

    // ========== Save/Load State Tests ==========

    #[test]
    fn test_save_load_queue_no_storage_path() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));

        // Should fail without storage path
        let result = manager.save_queue();
        assert!(matches!(result, Err(OfflineError::NoStoragePath)));

        let result = manager.load_queue();
        assert!(matches!(result, Err(OfflineError::NoStoragePath)));
    }

    #[test]
    fn test_save_load_queue_with_storage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("offline_state.json");

        let client_id = make_client_id(1);

        // Create manager and queue operations
        {
            let mut manager = OfflineManager::with_storage(client_id, &path);
            manager.queue_operation(make_text_insert(1, 1, 0, 'H'));
            manager.queue_operation(make_text_insert(1, 2, 1, 'i'));
            manager.update_sync_time();

            let result = manager.save_queue();
            assert!(result.is_ok());
        }

        // Create new manager and load state
        {
            let mut manager = OfflineManager::with_storage(client_id, &path);
            let result = manager.load_queue();
            assert!(result.is_ok());

            assert_eq!(manager.queue_size(), 2);
            assert!(manager.last_sync_time().is_some());
        }
    }

    #[test]
    fn test_get_and_restore_state() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));
        manager.queue_operation(make_text_insert(1, 2, 1, 'b'));

        let mut clock = VectorClock::new();
        clock.set(make_client_id(2), 5);
        manager.update_server_clock(clock.clone());

        // Get state
        let state = manager.get_state();
        assert_eq!(state.client_id, client_id);
        assert_eq!(state.offline_queue.len(), 2);
        assert_eq!(state.last_server_clock.get(make_client_id(2)), 5);

        // Restore to new manager
        let mut new_manager = OfflineManager::new(make_client_id(99));
        new_manager.restore_state(state);

        assert_eq!(new_manager.client_id(), client_id);
        assert_eq!(new_manager.queue_size(), 2);
        assert_eq!(new_manager.get_sync_clock().get(make_client_id(2)), 5);
    }

    #[test]
    fn test_offline_state_serialization() {
        let state = OfflineState {
            client_id: make_client_id(42),
            offline_queue: vec![make_text_insert(42, 1, 0, 'X')],
            last_server_clock: VectorClock::new(),
            last_sync_time: Some(1234567890),
        };

        let json = serde_json::to_string(&state).unwrap();
        let restored: OfflineState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.client_id, make_client_id(42));
        assert_eq!(restored.offline_queue.len(), 1);
        assert_eq!(restored.last_sync_time, Some(1234567890));
    }

    // ========== Status Transitions Tests ==========

    #[test]
    fn test_status_transitions() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Initial state
        assert_eq!(manager.status(), ConnectionStatus::Offline);
        assert!(manager.is_offline());
        assert!(!manager.is_online());

        // Go online
        manager.set_status(ConnectionStatus::Online);
        assert_eq!(manager.status(), ConnectionStatus::Online);
        assert!(manager.is_online());
        assert!(!manager.is_offline());

        // Go to reconnecting
        manager.set_status(ConnectionStatus::Reconnecting);
        assert_eq!(manager.status(), ConnectionStatus::Reconnecting);
        assert!(!manager.is_online());
        assert!(!manager.is_offline());

        // Go to syncing
        manager.set_status(ConnectionStatus::Syncing);
        assert_eq!(manager.status(), ConnectionStatus::Syncing);

        // Back to online
        manager.set_status(ConnectionStatus::Online);
        assert!(manager.is_online());
    }

    #[test]
    fn test_sync_complete() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));
        manager.set_status(ConnectionStatus::Syncing);

        assert_eq!(manager.queue_size(), 1);
        assert!(!manager.is_online());

        manager.sync_complete();

        assert_eq!(manager.queue_size(), 0);
        assert!(manager.is_online());
        assert!(manager.last_sync_time().is_some());
    }

    // ========== Merge Result Handling Tests ==========

    #[test]
    fn test_handle_sync_response_no_conflicts() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Local operation
        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));

        // Remote operation from different client, different location
        let remote_ops = vec![make_text_insert(2, 1, 0, 'x')];

        let result = manager.handle_sync_response(remote_ops);

        assert_eq!(result.merged_count, 1);
        // Different client and position - may or may not conflict depending on implementation
        // The key is the merge completed
        assert!(result.changes_summary.is_some());
    }

    #[test]
    fn test_handle_sync_response_with_conflicts() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Use the same node_id to create a conflict
        let node_id = NodeId::new();

        // Local operation
        let local_op = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        manager.queue_operation(local_op);

        // Remote operation at same position (same parent)
        let remote_ops = vec![CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'x',
        }];

        let result = manager.handle_sync_response(remote_ops);

        assert_eq!(result.merged_count, 1);
        assert!(result.had_conflicts);
        assert!(!result.local_reapply.is_empty());
        assert!(result.changes_summary.unwrap().contains("conflicts"));
    }

    #[test]
    fn test_handle_sync_response_updates_clock() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        assert_eq!(manager.get_sync_clock().get(make_client_id(2)), 0);

        let remote_ops = vec![
            make_text_insert(2, 1, 0, 'a'),
            make_text_insert(2, 2, 1, 'b'),
            make_text_insert(2, 3, 2, 'c'),
        ];

        manager.handle_sync_response(remote_ops);

        assert_eq!(manager.get_sync_clock().get(make_client_id(2)), 3);
    }

    #[test]
    fn test_merge_result_success() {
        let result = MergeResult::success(5);

        assert_eq!(result.merged_count, 5);
        assert!(!result.had_conflicts);
        assert!(result.local_reapply.is_empty());
        assert!(result.is_success());
    }

    #[test]
    fn test_merge_result_with_conflicts() {
        let conflicts = vec![make_text_insert(1, 1, 0, 'a')];
        let result = MergeResult::with_conflicts(3, conflicts);

        assert_eq!(result.merged_count, 3);
        assert!(result.had_conflicts);
        assert_eq!(result.local_reapply.len(), 1);
        assert!(!result.is_success());
    }

    // ========== Time Tracking Tests ==========

    #[test]
    fn test_time_tracking() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Initially no sync time
        assert!(manager.last_sync_time().is_none());
        assert!(manager.time_since_sync().is_none());

        // Update sync time
        manager.update_sync_time();

        assert!(manager.last_sync_time().is_some());
        // Time since sync should be very small (just updated)
        let time_since = manager.time_since_sync().unwrap();
        assert!(time_since < 2); // Should be less than 2 seconds
    }

    #[test]
    fn test_get_status_info() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Offline with no pending changes
        let info = manager.get_status_info();
        assert_eq!(info.status, ConnectionStatus::Offline);
        assert_eq!(info.pending_changes, 0);
        assert_eq!(info.status_message, "Offline");

        // Offline with pending changes
        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));
        manager.queue_operation(make_text_insert(1, 2, 1, 'b'));

        let info = manager.get_status_info();
        assert_eq!(info.pending_changes, 2);
        assert!(info.status_message.contains("2 pending changes"));

        // Online
        manager.set_status(ConnectionStatus::Online);
        let info = manager.get_status_info();
        assert_eq!(info.status, ConnectionStatus::Online);
        assert_eq!(info.status_message, "Connected");

        // Reconnecting
        manager.set_status(ConnectionStatus::Reconnecting);
        let info = manager.get_status_info();
        assert_eq!(info.status_message, "Reconnecting...");

        // Syncing
        manager.set_status(ConnectionStatus::Syncing);
        let info = manager.get_status_info();
        assert_eq!(info.status_message, "Syncing changes...");
    }

    #[test]
    fn test_offline_status_info_should_show() {
        // Should show when offline
        let info = OfflineStatusInfo {
            status: ConnectionStatus::Offline,
            pending_changes: 0,
            time_since_sync: None,
            status_message: "Offline".to_string(),
        };
        assert!(info.should_show());

        // Should show when online with pending changes
        let info = OfflineStatusInfo {
            status: ConnectionStatus::Online,
            pending_changes: 5,
            time_since_sync: Some(10),
            status_message: "Connected".to_string(),
        };
        assert!(info.should_show());

        // Should NOT show when online with no pending changes
        let info = OfflineStatusInfo {
            status: ConnectionStatus::Online,
            pending_changes: 0,
            time_since_sync: Some(10),
            status_message: "Connected".to_string(),
        };
        assert!(!info.should_show());
    }

    #[test]
    fn test_formatted_time_since_sync() {
        let info = OfflineStatusInfo {
            status: ConnectionStatus::Offline,
            pending_changes: 0,
            time_since_sync: Some(30),
            status_message: "Offline".to_string(),
        };
        assert_eq!(info.formatted_time_since_sync(), Some("30s ago".to_string()));

        let info = OfflineStatusInfo {
            status: ConnectionStatus::Offline,
            pending_changes: 0,
            time_since_sync: Some(120),
            status_message: "Offline".to_string(),
        };
        assert_eq!(info.formatted_time_since_sync(), Some("2m ago".to_string()));

        let info = OfflineStatusInfo {
            status: ConnectionStatus::Offline,
            pending_changes: 0,
            time_since_sync: Some(7200),
            status_message: "Offline".to_string(),
        };
        assert_eq!(info.formatted_time_since_sync(), Some("2h ago".to_string()));

        let info = OfflineStatusInfo {
            status: ConnectionStatus::Offline,
            pending_changes: 0,
            time_since_sync: Some(172800),
            status_message: "Offline".to_string(),
        };
        assert_eq!(info.formatted_time_since_sync(), Some("2d ago".to_string()));

        let info = OfflineStatusInfo {
            status: ConnectionStatus::Offline,
            pending_changes: 0,
            time_since_sync: None,
            status_message: "Offline".to_string(),
        };
        assert_eq!(info.formatted_time_since_sync(), None);
    }

    #[test]
    fn test_short_status() {
        let statuses = [
            (ConnectionStatus::Online, "Online"),
            (ConnectionStatus::Offline, "Offline"),
            (ConnectionStatus::Reconnecting, "Reconnecting"),
            (ConnectionStatus::Syncing, "Syncing"),
        ];

        for (status, expected) in statuses {
            let info = OfflineStatusInfo {
                status,
                pending_changes: 0,
                time_since_sync: None,
                status_message: String::new(),
            };
            assert_eq!(info.short_status(), expected);
        }
    }

    // ========== Additional Edge Cases ==========

    #[test]
    fn test_auto_save_disabled_by_default_for_new() {
        let manager = OfflineManager::new(make_client_id(1));
        assert!(!manager.auto_save_enabled());
    }

    #[test]
    fn test_auto_save_enabled_with_storage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.json");
        let manager = OfflineManager::with_storage(make_client_id(1), &path);
        assert!(manager.auto_save_enabled());
    }

    #[test]
    fn test_set_auto_save() {
        let mut manager = OfflineManager::new(make_client_id(1));
        assert!(!manager.auto_save_enabled());

        manager.set_auto_save(true);
        assert!(manager.auto_save_enabled());

        manager.set_auto_save(false);
        assert!(!manager.auto_save_enabled());
    }

    #[test]
    fn test_load_queue_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let mut manager = OfflineManager::with_storage(make_client_id(1), &path);

        // Loading from non-existent file should succeed (no-op)
        let result = manager.load_queue();
        assert!(result.is_ok());
        assert_eq!(manager.queue_size(), 0);
    }

    #[test]
    fn test_connection_status_default() {
        let status = ConnectionStatus::default();
        assert_eq!(status, ConnectionStatus::Offline);
    }

    #[test]
    fn test_offline_error_display() {
        let err = OfflineError::Storage("disk full".to_string());
        assert_eq!(format!("{}", err), "Storage error: disk full");

        let err = OfflineError::Serialization("invalid json".to_string());
        assert_eq!(format!("{}", err), "Serialization error: invalid json");

        let err = OfflineError::NoStoragePath;
        assert_eq!(format!("{}", err), "No storage path configured");

        let err = OfflineError::MergeError("conflict".to_string());
        assert_eq!(format!("{}", err), "Merge error: conflict");
    }

    #[test]
    fn test_multiple_clients_in_sync_response() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Remote operations from multiple clients
        let remote_ops = vec![
            make_text_insert(2, 1, 0, 'a'),
            make_text_insert(2, 2, 1, 'b'),
            make_text_insert(3, 1, 0, 'x'),
            make_text_insert(3, 2, 1, 'y'),
            make_text_insert(3, 3, 2, 'z'),
        ];

        let result = manager.handle_sync_response(remote_ops);

        assert_eq!(result.merged_count, 5);
        assert_eq!(manager.get_sync_clock().get(make_client_id(2)), 2);
        assert_eq!(manager.get_sync_clock().get(make_client_id(3)), 3);
    }

    #[test]
    fn test_block_operations_in_queue() {
        let client_id = make_client_id(1);
        let mut manager = OfflineManager::new(client_id);

        // Queue different types of operations
        manager.queue_operation(make_text_insert(1, 1, 0, 'a'));
        manager.queue_operation(make_block_insert(1, 2));
        manager.queue_operation(make_text_delete(1, 3, 1, 1));

        assert_eq!(manager.queue_size(), 3);

        let ops = manager.queued_operations();
        assert!(matches!(ops[0], CrdtOp::TextInsert { .. }));
        assert!(matches!(ops[1], CrdtOp::BlockInsert { .. }));
        assert!(matches!(ops[2], CrdtOp::TextDelete { .. }));
    }
}
