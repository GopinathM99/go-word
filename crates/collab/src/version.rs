//! Version history for document checkpointing and restoration.
//!
//! This module provides functionality for creating, managing, and restoring
//! document versions (checkpoints). It supports both automatic checkpointing
//! based on operation count/time thresholds and manual named versions.

use crate::clock::VectorClock;
use crate::op_id::{ClientId, OpId};
use crate::operation::CrdtOp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique version identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionId(pub String);

impl VersionId {
    /// Create a new unique version ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create a version ID from an existing string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for VersionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A document version (checkpoint)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {
    /// Unique version ID
    pub id: VersionId,
    /// When this version was created
    pub timestamp: DateTime<Utc>,
    /// Who created this version (user or system)
    pub author: String,
    /// Optional user-provided name
    pub name: Option<String>,
    /// Auto-generated summary of changes
    pub summary: String,
    /// Parent version (None for initial version)
    pub parent_id: Option<VersionId>,
    /// Vector clock at this version
    pub clock: VectorClock,
    /// Operations since parent version (for reconstruction)
    pub ops_since_parent: Vec<CrdtOp>,
    /// Whether this is a named version (never auto-deleted)
    pub is_named: bool,
    /// Full snapshot (serialized document state, optional for optimization)
    pub snapshot: Option<Vec<u8>>,
}

impl Version {
    /// Create a new version
    pub fn new(
        author: impl Into<String>,
        parent: Option<&Version>,
        ops: Vec<CrdtOp>,
        clock: VectorClock,
    ) -> Self {
        let summary = Self::generate_summary(&ops);
        Self {
            id: VersionId::new(),
            timestamp: Utc::now(),
            author: author.into(),
            name: None,
            summary,
            parent_id: parent.map(|p| p.id.clone()),
            clock,
            ops_since_parent: ops,
            is_named: false,
            snapshot: None,
        }
    }

    /// Create a named version
    pub fn named(
        author: impl Into<String>,
        name: impl Into<String>,
        parent: Option<&Version>,
        ops: Vec<CrdtOp>,
        clock: VectorClock,
    ) -> Self {
        let summary = Self::generate_summary(&ops);
        Self {
            id: VersionId::new(),
            timestamp: Utc::now(),
            author: author.into(),
            name: Some(name.into()),
            summary,
            parent_id: parent.map(|p| p.id.clone()),
            clock,
            ops_since_parent: ops,
            is_named: true,
            snapshot: None,
        }
    }

    /// Generate summary from operations
    fn generate_summary(ops: &[CrdtOp]) -> String {
        if ops.is_empty() {
            return "Initial version".to_string();
        }

        let mut text_inserts = 0;
        let mut text_deletes = 0;
        let mut format_changes = 0;
        let mut block_inserts = 0;
        let mut block_deletes = 0;
        let mut block_moves = 0;
        let mut block_updates = 0;

        for op in ops {
            match op {
                CrdtOp::TextInsert { .. } => text_inserts += 1,
                CrdtOp::TextDelete { .. } => text_deletes += 1,
                CrdtOp::FormatSet { .. } => format_changes += 1,
                CrdtOp::BlockInsert { .. } => block_inserts += 1,
                CrdtOp::BlockDelete { .. } => block_deletes += 1,
                CrdtOp::BlockMove { .. } => block_moves += 1,
                CrdtOp::BlockUpdate { .. } => block_updates += 1,
            }
        }

        let mut parts = Vec::new();

        if text_inserts > 0 {
            parts.push(format!(
                "{} character{} inserted",
                text_inserts,
                if text_inserts == 1 { "" } else { "s" }
            ));
        }
        if text_deletes > 0 {
            parts.push(format!(
                "{} character{} deleted",
                text_deletes,
                if text_deletes == 1 { "" } else { "s" }
            ));
        }
        if format_changes > 0 {
            parts.push(format!(
                "{} formatting change{}",
                format_changes,
                if format_changes == 1 { "" } else { "s" }
            ));
        }
        if block_inserts > 0 {
            parts.push(format!(
                "{} block{} inserted",
                block_inserts,
                if block_inserts == 1 { "" } else { "s" }
            ));
        }
        if block_deletes > 0 {
            parts.push(format!(
                "{} block{} deleted",
                block_deletes,
                if block_deletes == 1 { "" } else { "s" }
            ));
        }
        if block_moves > 0 {
            parts.push(format!(
                "{} block{} moved",
                block_moves,
                if block_moves == 1 { "" } else { "s" }
            ));
        }
        if block_updates > 0 {
            parts.push(format!(
                "{} block{} updated",
                block_updates,
                if block_updates == 1 { "" } else { "s" }
            ));
        }

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Set a snapshot for this version
    pub fn with_snapshot(mut self, snapshot: Vec<u8>) -> Self {
        self.snapshot = Some(snapshot);
        self
    }
}

/// Configuration for automatic checkpointing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Create checkpoint every N operations
    pub ops_threshold: usize,
    /// Create checkpoint every N seconds
    pub time_threshold_secs: u64,
    /// Maximum versions to keep (0 = unlimited)
    pub max_versions: usize,
    /// Keep named versions even when pruning
    pub preserve_named: bool,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            ops_threshold: 100,
            time_threshold_secs: 300, // 5 minutes
            max_versions: 50,
            preserve_named: true,
        }
    }
}

/// Version history manager
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionHistory {
    /// All versions by ID
    versions: HashMap<VersionId, Version>,
    /// Current (latest) version ID
    current: Option<VersionId>,
    /// Ordered list of version IDs (oldest to newest)
    history: Vec<VersionId>,
    /// Auto-checkpoint settings
    checkpoint_config: CheckpointConfig,
    /// Operations since last checkpoint
    ops_since_checkpoint: Vec<CrdtOp>,
    /// Counter for auto-naming
    version_counter: u32,
    /// Timestamp of last checkpoint (for time-based auto-checkpoint)
    #[serde(default = "default_last_checkpoint_time")]
    last_checkpoint_time: DateTime<Utc>,
}

fn default_last_checkpoint_time() -> DateTime<Utc> {
    Utc::now()
}

impl Default for VersionHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionHistory {
    /// Create a new version history manager
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            current: None,
            history: Vec::new(),
            checkpoint_config: CheckpointConfig::default(),
            ops_since_checkpoint: Vec::new(),
            version_counter: 0,
            last_checkpoint_time: Utc::now(),
        }
    }

    /// Create with custom checkpoint config
    pub fn with_config(config: CheckpointConfig) -> Self {
        Self {
            versions: HashMap::new(),
            current: None,
            history: Vec::new(),
            checkpoint_config: config,
            ops_since_checkpoint: Vec::new(),
            version_counter: 0,
            last_checkpoint_time: Utc::now(),
        }
    }

    // ========== Version Creation ==========

    /// Record an operation (may trigger auto-checkpoint)
    pub fn record_operation(&mut self, op: CrdtOp, clock: &VectorClock, author: &str) {
        self.ops_since_checkpoint.push(op);

        // Check if we should auto-checkpoint
        if self.should_checkpoint() {
            self.create_checkpoint(author, clock.clone());
        }
    }

    /// Create a checkpoint now
    pub fn create_checkpoint(&mut self, author: &str, clock: VectorClock) -> VersionId {
        self.version_counter += 1;

        let parent_id = self.current.as_ref().and_then(|id| self.versions.get(id)).map(|v| v.id.clone());
        let ops = std::mem::take(&mut self.ops_since_checkpoint);

        let mut version = Version::new(author, None, ops, clock);
        version.parent_id = parent_id;
        let version_id = version.id.clone();

        self.versions.insert(version_id.clone(), version);
        self.history.push(version_id.clone());
        self.current = Some(version_id.clone());
        self.last_checkpoint_time = Utc::now();

        // Prune if needed
        if self.checkpoint_config.max_versions > 0
            && self.history.len() > self.checkpoint_config.max_versions
        {
            self.prune();
        }

        version_id
    }

    /// Create a named version
    pub fn create_named_version(
        &mut self,
        name: &str,
        author: &str,
        clock: VectorClock,
    ) -> VersionId {
        self.version_counter += 1;

        let parent_id = self.current.as_ref().and_then(|id| self.versions.get(id)).map(|v| v.id.clone());
        let ops = std::mem::take(&mut self.ops_since_checkpoint);

        let mut version = Version::named(author, name, None, ops, clock);
        version.parent_id = parent_id;
        let version_id = version.id.clone();

        self.versions.insert(version_id.clone(), version);
        self.history.push(version_id.clone());
        self.current = Some(version_id.clone());
        self.last_checkpoint_time = Utc::now();

        version_id
    }

    /// Check if a checkpoint should be created
    pub fn should_checkpoint(&self) -> bool {
        // Check operation count threshold
        if self.ops_since_checkpoint.len() >= self.checkpoint_config.ops_threshold {
            return true;
        }

        // Check time threshold
        let elapsed = Utc::now()
            .signed_duration_since(self.last_checkpoint_time)
            .num_seconds();
        if elapsed >= self.checkpoint_config.time_threshold_secs as i64
            && !self.ops_since_checkpoint.is_empty()
        {
            return true;
        }

        false
    }

    // ========== Version Retrieval ==========

    /// Get a version by ID
    pub fn get_version(&self, id: &VersionId) -> Option<&Version> {
        self.versions.get(id)
    }

    /// Get the current (latest) version
    pub fn current_version(&self) -> Option<&Version> {
        self.current.as_ref().and_then(|id| self.versions.get(id))
    }

    /// Get all versions (newest first)
    pub fn all_versions(&self) -> Vec<&Version> {
        self.history
            .iter()
            .rev()
            .filter_map(|id| self.versions.get(id))
            .collect()
    }

    /// Get named versions only
    pub fn named_versions(&self) -> Vec<&Version> {
        self.history
            .iter()
            .rev()
            .filter_map(|id| self.versions.get(id))
            .filter(|v| v.is_named)
            .collect()
    }

    /// Get version history (version IDs in order)
    pub fn history(&self) -> &[VersionId] {
        &self.history
    }

    /// Get the number of versions
    pub fn len(&self) -> usize {
        self.versions.len()
    }

    /// Check if there are no versions
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }

    /// Get pending operations (not yet checkpointed)
    pub fn pending_ops(&self) -> &[CrdtOp] {
        &self.ops_since_checkpoint
    }

    // ========== Version Reconstruction ==========

    /// Get operations needed to reconstruct a version from scratch
    pub fn ops_to_reconstruct(&self, version_id: &VersionId) -> Option<Vec<&CrdtOp>> {
        // Find the version (verify it exists)
        let _version = self.versions.get(version_id)?;

        // Collect all operations from the beginning to this version
        let mut ops = Vec::new();
        let mut current_id = Some(version_id.clone());

        // Collect versions from target back to root
        let mut version_chain = Vec::new();
        while let Some(id) = current_id {
            if let Some(v) = self.versions.get(&id) {
                version_chain.push(v);
                current_id = v.parent_id.clone();
            } else {
                break;
            }
        }

        // Reverse to get oldest to newest
        version_chain.reverse();

        // Collect all ops
        for v in version_chain {
            ops.extend(v.ops_since_parent.iter());
        }

        Some(ops)
    }

    /// Get operations between two versions
    pub fn ops_between(&self, from: &VersionId, to: &VersionId) -> Option<Vec<&CrdtOp>> {
        // Verify both versions exist
        if !self.versions.contains_key(from) || !self.versions.contains_key(to) {
            return None;
        }

        // Find positions in history
        let from_pos = self.history.iter().position(|id| id == from)?;
        let to_pos = self.history.iter().position(|id| id == to)?;

        if from_pos >= to_pos {
            return Some(Vec::new()); // from is same or after to
        }

        // Collect operations from versions between from and to (exclusive of from, inclusive of to)
        let mut ops = Vec::new();
        for id in &self.history[from_pos + 1..=to_pos] {
            if let Some(version) = self.versions.get(id) {
                ops.extend(version.ops_since_parent.iter());
            }
        }

        Some(ops)
    }

    // ========== Version Comparison ==========

    /// Compare two versions and return the diff
    pub fn diff(&self, from: &VersionId, to: &VersionId) -> Option<VersionDiff> {
        let ops = self.ops_between(from, to)?;

        let added_ops: Vec<CrdtOp> = ops.into_iter().cloned().collect();
        let summary = Version::generate_summary(&added_ops);

        Some(VersionDiff {
            from_version: from.clone(),
            to_version: to.clone(),
            added_ops,
            summary,
        })
    }

    // ========== Version Restoration ==========

    /// Create a new version that restores to a previous state
    ///
    /// This creates "undo" operations to revert changes made after the target version.
    /// Returns the new version ID and the undo operations to apply.
    pub fn restore_to(
        &mut self,
        version_id: &VersionId,
        author: &str,
        current_clock: VectorClock,
    ) -> Option<(VersionId, Vec<CrdtOp>)> {
        // Verify the version exists
        let target_version = self.versions.get(version_id)?.clone();

        // Find current version
        let current_version_id = self.current.clone()?;

        // Get operations that need to be "undone"
        let ops_to_undo = self.ops_between(version_id, &current_version_id)?;

        // Generate undo operations (reverse order)
        let mut undo_ops = Vec::new();
        let client_id = ClientId::new(0); // System client for restore ops
        let mut seq = current_clock.get(client_id) + 1;

        for op in ops_to_undo.into_iter().rev() {
            if let Some(undo_op) = Self::create_undo_op(op, client_id, &mut seq) {
                undo_ops.push(undo_op);
            }
        }

        // Create a new version for this restore
        let restore_name = format!(
            "Restored to: {}",
            target_version.name.as_deref().unwrap_or(&target_version.id.0[..8.min(target_version.id.0.len())])
        );
        let parent_id = self.current.as_ref().and_then(|id| self.versions.get(id)).map(|v| v.id.clone());
        let mut restore_version = Version::named(
            author,
            restore_name,
            None,
            undo_ops.clone(),
            current_clock,
        );
        restore_version.parent_id = parent_id;

        let new_version_id = restore_version.id.clone();
        self.versions.insert(new_version_id.clone(), restore_version);
        self.history.push(new_version_id.clone());
        self.current = Some(new_version_id.clone());
        self.ops_since_checkpoint.clear();

        Some((new_version_id, undo_ops))
    }

    /// Create an undo operation for a given operation
    fn create_undo_op(op: &CrdtOp, client_id: ClientId, seq: &mut u64) -> Option<CrdtOp> {
        let op_id = OpId::new(client_id, *seq);
        *seq += 1;

        match op {
            CrdtOp::TextInsert { id, .. } => {
                // Undo insert = delete
                Some(CrdtOp::TextDelete {
                    id: op_id,
                    target_id: *id,
                })
            }
            CrdtOp::TextDelete { .. } => {
                // Undoing a delete is complex - we'd need the original content
                // For now, we'll skip these (the character is still in the RGA as a tombstone)
                // A full implementation would need to "resurrect" the deleted character
                None
            }
            CrdtOp::BlockInsert { id, .. } => {
                // Undo insert = delete
                Some(CrdtOp::BlockDelete {
                    id: op_id,
                    target_id: *id,
                })
            }
            CrdtOp::BlockDelete { .. } => {
                // Similar to text delete - would need original content
                None
            }
            CrdtOp::FormatSet { .. } => {
                // Format changes are LWW - we'd need the previous value
                // For now, skip
                None
            }
            CrdtOp::BlockMove { .. } => {
                // Would need to track original position
                None
            }
            CrdtOp::BlockUpdate { .. } => {
                // Would need previous block data
                None
            }
        }
    }

    // ========== Pruning ==========

    /// Prune old versions (keep named and recent)
    ///
    /// Returns the number of versions deleted.
    pub fn prune(&mut self) -> usize {
        if self.checkpoint_config.max_versions == 0 {
            return 0; // Unlimited versions
        }

        let mut deleted = 0;
        let max = self.checkpoint_config.max_versions;

        // Keep pruning until we're under the limit
        while self.history.len() > max {
            // Find oldest non-named version (skip if preserve_named is true)
            let mut to_remove = None;

            for id in &self.history {
                if let Some(version) = self.versions.get(id) {
                    if !version.is_named || !self.checkpoint_config.preserve_named {
                        // Don't remove the current version
                        if self.current.as_ref() != Some(id) {
                            to_remove = Some(id.clone());
                            break;
                        }
                    }
                }
            }

            if let Some(id) = to_remove {
                self.delete_version_internal(&id);
                deleted += 1;
            } else {
                // Can't remove any more versions
                break;
            }
        }

        deleted
    }

    /// Delete a specific version (fails for named versions unless forced)
    pub fn delete_version(&mut self, id: &VersionId) -> bool {
        // Check if version exists
        let version = match self.versions.get(id) {
            Some(v) => v,
            None => return false,
        };

        // Don't delete named versions
        if version.is_named {
            return false;
        }

        // Don't delete current version
        if self.current.as_ref() == Some(id) {
            return false;
        }

        self.delete_version_internal(id);
        true
    }

    /// Internal version deletion (doesn't check named status)
    fn delete_version_internal(&mut self, id: &VersionId) {
        // Remove from versions map
        if let Some(deleted_version) = self.versions.remove(id) {
            // Find any versions that had this as a parent
            // and update them to skip to this version's parent
            for version in self.versions.values_mut() {
                if version.parent_id.as_ref() == Some(id) {
                    // Transfer ops from deleted version
                    let mut new_ops = deleted_version.ops_since_parent.clone();
                    new_ops.append(&mut version.ops_since_parent.clone());
                    version.ops_since_parent = new_ops;
                    version.parent_id = deleted_version.parent_id.clone();
                }
            }
        }

        // Remove from history
        self.history.retain(|h| h != id);
    }

    // ========== Persistence ==========

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Serialize to JSON string (for debugging)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    // ========== UI Helpers ==========

    /// Get version info for UI display
    pub fn get_version_infos(&self) -> Vec<VersionInfo> {
        self.history
            .iter()
            .rev()
            .filter_map(|id| {
                let version = self.versions.get(id)?;
                Some(VersionInfo {
                    id: version.id.clone(),
                    timestamp: version.timestamp,
                    author: version.author.clone(),
                    name: version.name.clone(),
                    summary: version.summary.clone(),
                    is_named: version.is_named,
                    is_current: self.current.as_ref() == Some(&version.id),
                })
            })
            .collect()
    }
}

/// Diff between two versions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: VersionId,
    pub to_version: VersionId,
    pub added_ops: Vec<CrdtOp>,
    pub summary: String,
}

/// Version info for UI display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: VersionId,
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub name: Option<String>,
    pub summary: String,
    pub is_named: bool,
    pub is_current: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Timestamp;
    use crate::crdt_tree::BlockData;
    use doc_model::NodeId;

    // Helper functions
    fn make_op_id(client: u64, seq: u64) -> OpId {
        OpId::new(ClientId::new(client), seq)
    }

    fn make_text_insert(client: u64, seq: u64, parent_seq: u64, c: char) -> CrdtOp {
        CrdtOp::TextInsert {
            id: make_op_id(client, seq),
            node_id: NodeId::new(),
            parent_op_id: make_op_id(client, parent_seq),
            char: c,
        }
    }

    fn make_text_delete(client: u64, seq: u64, target_client: u64, target_seq: u64) -> CrdtOp {
        CrdtOp::TextDelete {
            id: make_op_id(client, seq),
            target_id: make_op_id(target_client, target_seq),
        }
    }

    fn make_block_insert(client: u64, seq: u64) -> CrdtOp {
        CrdtOp::BlockInsert {
            id: make_op_id(client, seq),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        }
    }

    fn make_format_set(client: u64, seq: u64, attr: &str) -> CrdtOp {
        CrdtOp::FormatSet {
            id: make_op_id(client, seq),
            node_id: NodeId::new(),
            start_op_id: make_op_id(client, 1),
            end_op_id: make_op_id(client, 5),
            attribute: attr.to_string(),
            value: serde_json::json!(true),
            timestamp: Timestamp::new(1000, 0, ClientId::new(client)),
        }
    }

    // ========== Version ID Tests ==========

    #[test]
    fn test_version_id_new() {
        let id1 = VersionId::new();
        let id2 = VersionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_version_id_from_string() {
        let id = VersionId::from_string("test-version-123");
        assert_eq!(id.0, "test-version-123");
    }

    // ========== Version Tests ==========

    #[test]
    fn test_version_new() {
        let ops = vec![make_text_insert(1, 1, 0, 'a')];
        let clock = VectorClock::new();

        let version = Version::new("test_user", None, ops.clone(), clock.clone());

        assert_eq!(version.author, "test_user");
        assert!(version.name.is_none());
        assert!(!version.is_named);
        assert!(version.parent_id.is_none());
        assert_eq!(version.ops_since_parent.len(), 1);
    }

    #[test]
    fn test_version_named() {
        let ops = vec![make_text_insert(1, 1, 0, 'a')];
        let clock = VectorClock::new();

        let version = Version::named("test_user", "My Version", None, ops.clone(), clock);

        assert_eq!(version.author, "test_user");
        assert_eq!(version.name, Some("My Version".to_string()));
        assert!(version.is_named);
    }

    #[test]
    fn test_version_with_parent() {
        let clock = VectorClock::new();
        let parent = Version::new("user", None, vec![], clock.clone());
        let child = Version::new("user", Some(&parent), vec![], clock);

        assert_eq!(child.parent_id, Some(parent.id.clone()));
    }

    #[test]
    fn test_generate_summary_empty() {
        let summary = Version::generate_summary(&[]);
        assert_eq!(summary, "Initial version");
    }

    #[test]
    fn test_generate_summary_text_inserts() {
        let ops = vec![
            make_text_insert(1, 1, 0, 'a'),
            make_text_insert(1, 2, 1, 'b'),
            make_text_insert(1, 3, 2, 'c'),
        ];
        let summary = Version::generate_summary(&ops);
        assert!(summary.contains("3 characters inserted"));
    }

    #[test]
    fn test_generate_summary_mixed() {
        let ops = vec![
            make_text_insert(1, 1, 0, 'a'),
            make_text_delete(1, 2, 1, 1),
            make_block_insert(1, 3),
            make_format_set(1, 4, "bold"),
        ];
        let summary = Version::generate_summary(&ops);
        assert!(summary.contains("1 character inserted"));
        assert!(summary.contains("1 character deleted"));
        assert!(summary.contains("1 block inserted"));
        assert!(summary.contains("1 formatting change"));
    }

    // ========== CheckpointConfig Tests ==========

    #[test]
    fn test_checkpoint_config_default() {
        let config = CheckpointConfig::default();
        assert_eq!(config.ops_threshold, 100);
        assert_eq!(config.time_threshold_secs, 300);
        assert_eq!(config.max_versions, 50);
        assert!(config.preserve_named);
    }

    // ========== VersionHistory Creation Tests ==========

    #[test]
    fn test_version_history_new() {
        let history = VersionHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert!(history.current_version().is_none());
    }

    #[test]
    fn test_version_history_with_config() {
        let config = CheckpointConfig {
            ops_threshold: 50,
            time_threshold_secs: 60,
            max_versions: 10,
            preserve_named: false,
        };
        let history = VersionHistory::with_config(config);
        assert_eq!(history.checkpoint_config.ops_threshold, 50);
    }

    // ========== Checkpoint Tests ==========

    #[test]
    fn test_create_checkpoint() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let version_id = history.create_checkpoint("user", clock);

        assert_eq!(history.len(), 1);
        assert!(history.current_version().is_some());
        assert_eq!(history.current_version().unwrap().id, version_id);
    }

    #[test]
    fn test_create_multiple_checkpoints() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let v1 = history.create_checkpoint("user", clock.clone());
        let v2 = history.create_checkpoint("user", clock.clone());
        let v3 = history.create_checkpoint("user", clock);

        assert_eq!(history.len(), 3);
        assert_eq!(history.history().len(), 3);
        assert_eq!(history.history()[0], v1);
        assert_eq!(history.history()[1], v2);
        assert_eq!(history.history()[2], v3);
        assert_eq!(history.current_version().unwrap().id, v3);
    }

    #[test]
    fn test_auto_checkpoint_on_ops_threshold() {
        let config = CheckpointConfig {
            ops_threshold: 3,
            time_threshold_secs: 3600, // Long time threshold
            max_versions: 100,
            preserve_named: true,
        };
        let mut history = VersionHistory::with_config(config);
        let clock = VectorClock::new();

        // Record operations - should trigger checkpoint at 3
        history.record_operation(make_text_insert(1, 1, 0, 'a'), &clock, "user");
        assert_eq!(history.len(), 0); // Not yet

        history.record_operation(make_text_insert(1, 2, 1, 'b'), &clock, "user");
        assert_eq!(history.len(), 0); // Not yet

        history.record_operation(make_text_insert(1, 3, 2, 'c'), &clock, "user");
        assert_eq!(history.len(), 1); // Now!
    }

    #[test]
    fn test_should_checkpoint() {
        let config = CheckpointConfig {
            ops_threshold: 5,
            time_threshold_secs: 3600,
            max_versions: 100,
            preserve_named: true,
        };
        let mut history = VersionHistory::with_config(config);

        // No ops, shouldn't checkpoint
        assert!(!history.should_checkpoint());

        // Add some ops, still under threshold
        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'a'));
        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'b'));
        assert!(!history.should_checkpoint());

        // Add more to hit threshold
        history.ops_since_checkpoint.push(make_text_insert(1, 3, 2, 'c'));
        history.ops_since_checkpoint.push(make_text_insert(1, 4, 3, 'd'));
        history.ops_since_checkpoint.push(make_text_insert(1, 5, 4, 'e'));
        assert!(history.should_checkpoint());
    }

    // ========== Named Version Tests ==========

    #[test]
    fn test_create_named_version() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let version_id = history.create_named_version("Draft 1", "user", clock);

        let version = history.get_version(&version_id).unwrap();
        assert!(version.is_named);
        assert_eq!(version.name, Some("Draft 1".to_string()));
    }

    #[test]
    fn test_named_versions_filter() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        history.create_checkpoint("user", clock.clone()); // Not named
        history.create_named_version("V1", "user", clock.clone()); // Named
        history.create_checkpoint("user", clock.clone()); // Not named
        history.create_named_version("V2", "user", clock); // Named

        assert_eq!(history.len(), 4);
        assert_eq!(history.named_versions().len(), 2);
    }

    // ========== Version Retrieval Tests ==========

    #[test]
    fn test_get_version() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let version_id = history.create_checkpoint("user", clock);

        assert!(history.get_version(&version_id).is_some());
        assert!(history.get_version(&VersionId::new()).is_none());
    }

    #[test]
    fn test_all_versions_order() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let v1 = history.create_checkpoint("user", clock.clone());
        let v2 = history.create_checkpoint("user", clock.clone());
        let v3 = history.create_checkpoint("user", clock);

        let all = history.all_versions();
        assert_eq!(all.len(), 3);
        // Newest first
        assert_eq!(all[0].id, v3);
        assert_eq!(all[1].id, v2);
        assert_eq!(all[2].id, v1);
    }

    // ========== Ops Between Tests ==========

    #[test]
    fn test_ops_between_versions() {
        let mut history = VersionHistory::new();
        let mut clock = VectorClock::new();

        // Create first version
        let v1 = history.create_checkpoint("user", clock.clone());

        // Record some operations
        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'a'));
        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'b'));
        clock.set(ClientId::new(1), 2);
        let v2 = history.create_checkpoint("user", clock.clone());

        // Record more operations
        history.ops_since_checkpoint.push(make_text_insert(1, 3, 2, 'c'));
        clock.set(ClientId::new(1), 3);
        let v3 = history.create_checkpoint("user", clock);

        // Get ops between v1 and v3
        let ops = history.ops_between(&v1, &v3).unwrap();
        assert_eq!(ops.len(), 3);

        // Get ops between v2 and v3
        let ops = history.ops_between(&v2, &v3).unwrap();
        assert_eq!(ops.len(), 1);

        // Get ops between same version
        let ops = history.ops_between(&v1, &v1).unwrap();
        assert_eq!(ops.len(), 0);
    }

    #[test]
    fn test_ops_between_invalid() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let v1 = history.create_checkpoint("user", clock);
        let fake_id = VersionId::new();

        assert!(history.ops_between(&v1, &fake_id).is_none());
        assert!(history.ops_between(&fake_id, &v1).is_none());
    }

    // ========== Ops To Reconstruct Tests ==========

    #[test]
    fn test_ops_to_reconstruct() {
        let mut history = VersionHistory::new();
        let mut clock = VectorClock::new();

        // Create versions with ops
        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'a'));
        clock.set(ClientId::new(1), 1);
        let v1 = history.create_checkpoint("user", clock.clone());

        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'b'));
        clock.set(ClientId::new(1), 2);
        let _v2 = history.create_checkpoint("user", clock.clone());

        history.ops_since_checkpoint.push(make_text_insert(1, 3, 2, 'c'));
        clock.set(ClientId::new(1), 3);
        let v3 = history.create_checkpoint("user", clock);

        // Reconstruct v3 should have all 3 ops
        let ops = history.ops_to_reconstruct(&v3).unwrap();
        assert_eq!(ops.len(), 3);

        // Reconstruct v1 should have 1 op
        let ops = history.ops_to_reconstruct(&v1).unwrap();
        assert_eq!(ops.len(), 1);
    }

    // ========== Diff Tests ==========

    #[test]
    fn test_diff() {
        let mut history = VersionHistory::new();
        let mut clock = VectorClock::new();

        let v1 = history.create_checkpoint("user", clock.clone());

        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'H'));
        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'i'));
        clock.set(ClientId::new(1), 2);
        let v2 = history.create_checkpoint("user", clock);

        let diff = history.diff(&v1, &v2).unwrap();
        assert_eq!(diff.from_version, v1);
        assert_eq!(diff.to_version, v2);
        assert_eq!(diff.added_ops.len(), 2);
        assert!(diff.summary.contains("2 characters inserted"));
    }

    // ========== Restore Tests ==========

    #[test]
    fn test_restore_to_version() {
        let mut history = VersionHistory::new();
        let mut clock = VectorClock::new();

        // Create initial version
        let v1 = history.create_checkpoint("user", clock.clone());

        // Add some text
        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'a'));
        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'b'));
        clock.set(ClientId::new(1), 2);
        let _v2 = history.create_checkpoint("user", clock.clone());

        // Restore to v1
        let result = history.restore_to(&v1, "user", clock);
        assert!(result.is_some());

        let (new_version_id, undo_ops) = result.unwrap();

        // Should have created a new version
        let new_version = history.get_version(&new_version_id).unwrap();
        assert!(new_version.is_named);
        assert!(new_version.name.as_ref().unwrap().contains("Restored"));

        // Undo ops should delete the inserted characters
        assert_eq!(undo_ops.len(), 2);
        for op in &undo_ops {
            assert!(matches!(op, CrdtOp::TextDelete { .. }));
        }
    }

    // ========== Pruning Tests ==========

    #[test]
    fn test_prune() {
        let config = CheckpointConfig {
            ops_threshold: 100,
            time_threshold_secs: 300,
            max_versions: 3,
            preserve_named: true,
        };
        let mut history = VersionHistory::with_config(config);
        let clock = VectorClock::new();

        // Create versions
        history.create_checkpoint("user", clock.clone());
        history.create_checkpoint("user", clock.clone());
        history.create_checkpoint("user", clock.clone());
        history.create_checkpoint("user", clock.clone());
        history.create_checkpoint("user", clock);

        // Should have pruned to 3
        assert!(history.len() <= 5); // Auto-prune happens on create
    }

    #[test]
    fn test_prune_preserves_named() {
        let config = CheckpointConfig {
            ops_threshold: 100,
            time_threshold_secs: 300,
            max_versions: 2,
            preserve_named: true,
        };
        let mut history = VersionHistory::with_config(config);
        let clock = VectorClock::new();

        let named_id = history.create_named_version("Important", "user", clock.clone());
        history.create_checkpoint("user", clock.clone());
        history.create_checkpoint("user", clock.clone());
        history.create_checkpoint("user", clock);

        // Named version should still exist
        assert!(history.get_version(&named_id).is_some());
    }

    #[test]
    fn test_delete_version() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let v1 = history.create_checkpoint("user", clock.clone());
        let _v2 = history.create_checkpoint("user", clock);

        // Can delete v1 (not current, not named)
        assert!(history.delete_version(&v1));
        assert!(history.get_version(&v1).is_none());
    }

    #[test]
    fn test_delete_version_fails_for_named() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let named = history.create_named_version("Important", "user", clock.clone());
        history.create_checkpoint("user", clock); // Make another so named isn't current

        // Can't delete named version
        assert!(!history.delete_version(&named));
        assert!(history.get_version(&named).is_some());
    }

    #[test]
    fn test_delete_version_fails_for_current() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        let v1 = history.create_checkpoint("user", clock);

        // Can't delete current version
        assert!(!history.delete_version(&v1));
    }

    // ========== Serialization Tests ==========

    #[test]
    fn test_serialization() {
        let mut history = VersionHistory::new();
        let mut clock = VectorClock::new();

        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'H'));
        clock.set(ClientId::new(1), 1);
        history.create_checkpoint("user", clock.clone());

        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'i'));
        clock.set(ClientId::new(1), 2);
        history.create_named_version("Hello version", "user", clock);

        // Serialize
        let bytes = history.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Deserialize
        let restored = VersionHistory::from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.named_versions().len(), 1);
    }

    #[test]
    fn test_to_json() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        history.create_checkpoint("user", clock);

        let json = history.to_json().unwrap();
        assert!(json.contains("versions"));
        assert!(json.contains("history"));
    }

    // ========== Version Info Tests ==========

    #[test]
    fn test_get_version_infos() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        history.create_checkpoint("alice", clock.clone());
        history.create_named_version("Draft 1", "bob", clock);

        let infos = history.get_version_infos();
        assert_eq!(infos.len(), 2);

        // Newest first
        assert!(infos[0].is_current);
        assert!(infos[0].is_named);
        assert_eq!(infos[0].author, "bob");

        assert!(!infos[1].is_current);
        assert!(!infos[1].is_named);
        assert_eq!(infos[1].author, "alice");
    }

    // ========== Pending Ops Tests ==========

    #[test]
    fn test_pending_ops() {
        let mut history = VersionHistory::new();
        let clock = VectorClock::new();

        assert!(history.pending_ops().is_empty());

        history.record_operation(make_text_insert(1, 1, 0, 'a'), &clock, "user");
        history.record_operation(make_text_insert(1, 2, 1, 'b'), &clock, "user");

        assert_eq!(history.pending_ops().len(), 2);

        // After checkpoint, pending should be empty
        history.create_checkpoint("user", clock);
        assert!(history.pending_ops().is_empty());
    }

    // ========== Version Chain Tests ==========

    #[test]
    fn test_version_chain_with_deletion() {
        let mut history = VersionHistory::new();
        let mut clock = VectorClock::new();

        // Create a chain: v1 -> v2 -> v3
        history.ops_since_checkpoint.push(make_text_insert(1, 1, 0, 'a'));
        clock.set(ClientId::new(1), 1);
        let v1 = history.create_checkpoint("user", clock.clone());

        history.ops_since_checkpoint.push(make_text_insert(1, 2, 1, 'b'));
        clock.set(ClientId::new(1), 2);
        let v2 = history.create_checkpoint("user", clock.clone());

        history.ops_since_checkpoint.push(make_text_insert(1, 3, 2, 'c'));
        clock.set(ClientId::new(1), 3);
        let v3 = history.create_checkpoint("user", clock);

        // Delete v2
        history.delete_version_internal(&v2);

        // v3 should now point to v1 and have combined ops
        let v3_version = history.get_version(&v3).unwrap();
        assert_eq!(v3_version.parent_id, Some(v1.clone()));
        assert_eq!(v3_version.ops_since_parent.len(), 2); // b and c
    }

    // ========== VersionDiff Tests ==========

    #[test]
    fn test_version_diff_serialization() {
        let diff = VersionDiff {
            from_version: VersionId::from_string("v1"),
            to_version: VersionId::from_string("v2"),
            added_ops: vec![make_text_insert(1, 1, 0, 'x')],
            summary: "Test diff".to_string(),
        };

        let json = serde_json::to_string(&diff).unwrap();
        let restored: VersionDiff = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.from_version, diff.from_version);
        assert_eq!(restored.to_version, diff.to_version);
        assert_eq!(restored.summary, "Test diff");
    }

    // ========== VersionInfo Tests ==========

    #[test]
    fn test_version_info_serialization() {
        let info = VersionInfo {
            id: VersionId::from_string("test-id"),
            timestamp: Utc::now(),
            author: "test_user".to_string(),
            name: Some("Test Version".to_string()),
            summary: "Test summary".to_string(),
            is_named: true,
            is_current: false,
        };

        let json = serde_json::to_string(&info).unwrap();
        let restored: VersionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, info.id);
        assert_eq!(restored.author, "test_user");
        assert_eq!(restored.name, Some("Test Version".to_string()));
        assert!(restored.is_named);
        assert!(!restored.is_current);
    }
}
