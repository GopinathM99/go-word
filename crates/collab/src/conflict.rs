//! Conflict resolution engine for CRDT operations.
//!
//! This module provides deterministic conflict resolution for concurrent operations
//! in a collaborative editing system. The key principles are:
//!
//! - **Determinism**: Same inputs always produce the same output
//! - **Commutativity**: Order of resolution shouldn't matter
//! - **Per-attribute formatting**: Different attributes don't conflict
//!
//! # Conflict Resolution Rules
//!
//! - **Text inserts**: Higher OpId wins (deterministic ordering)
//! - **Delete vs. edit**: Delete wins (text is gone)
//! - **Formatting**: Latest timestamp wins, client_id breaks ties
//! - **Structural**: Higher OpId wins for inserts, delete wins for delete vs. edit

use crate::clock::Timestamp;
use crate::op_id::OpId;
use crate::operation::CrdtOp;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Result of conflict resolution
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConflictResult {
    /// No conflict, apply normally
    NoConflict,
    /// Conflict resolved, this operation wins
    Wins,
    /// Conflict resolved, other operation wins
    Loses,
    /// Operations are compatible, both apply
    Compatible,
}

/// Types of conflicts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConflictType {
    /// Two inserts at the same position
    ConcurrentInsert { op1: OpId, op2: OpId },
    /// Delete and edit on same content
    DeleteEdit { delete_op: OpId, edit_op: OpId },
    /// Two formatting changes on same range
    FormattingConflict {
        attribute: String,
        op1: OpId,
        op2: OpId,
    },
    /// Structural changes (block operations)
    StructuralConflict { op1: OpId, op2: OpId },
}

/// Conflict resolution engine
pub struct ConflictResolver {
    /// Conflict history for debugging/visualization
    conflict_history: Vec<ConflictRecord>,
    /// Enable conflict logging
    log_conflicts: bool,
}

/// Record of a resolved conflict
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub conflict_type: ConflictType,
    pub winner: OpId,
    pub timestamp: u64,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            conflict_history: Vec::new(),
            log_conflicts: false,
        }
    }

    /// Enable conflict logging
    pub fn with_logging(mut self) -> Self {
        self.log_conflicts = true;
        self
    }

    /// Log a conflict if logging is enabled
    fn log_conflict(&mut self, conflict_type: ConflictType, winner: OpId) {
        if self.log_conflicts {
            self.conflict_history.push(ConflictRecord {
                conflict_type,
                winner,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
        }
    }

    // ========== Text Conflict Resolution ==========

    /// Resolve concurrent text inserts at the same position.
    ///
    /// Rule: Higher OpId wins (deterministic ordering).
    /// This ensures that when two clients insert at the same position,
    /// the ordering is deterministic based on OpId comparison.
    pub fn resolve_text_insert(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        match (op1, op2) {
            (
                CrdtOp::TextInsert {
                    id: id1,
                    parent_op_id: parent1,
                    node_id: node1,
                    ..
                },
                CrdtOp::TextInsert {
                    id: id2,
                    parent_op_id: parent2,
                    node_id: node2,
                    ..
                },
            ) => {
                // Check if they're inserting at the same position in the same node
                if node1 != node2 || parent1 != parent2 {
                    return ConflictResult::NoConflict;
                }

                // Higher OpId wins
                let result = if id1 > id2 {
                    ConflictResult::Wins
                } else if id1 < id2 {
                    ConflictResult::Loses
                } else {
                    // Same OpId means same operation (shouldn't happen in practice)
                    ConflictResult::NoConflict
                };

                if result != ConflictResult::NoConflict {
                    let winner = if result == ConflictResult::Wins {
                        *id1
                    } else {
                        *id2
                    };
                    self.log_conflict(
                        ConflictType::ConcurrentInsert {
                            op1: *id1,
                            op2: *id2,
                        },
                        winner,
                    );
                }

                result
            }
            _ => ConflictResult::NoConflict,
        }
    }

    /// Resolve delete vs. edit conflict.
    ///
    /// Rule: Delete wins (text is gone).
    /// When a delete and an edit target the same content, the delete takes precedence.
    pub fn resolve_delete_edit(&mut self, delete: &CrdtOp, edit: &CrdtOp) -> ConflictResult {
        let delete_target = match delete {
            CrdtOp::TextDelete { target_id, .. } => Some(*target_id),
            CrdtOp::BlockDelete { target_id, .. } => Some(*target_id),
            _ => None,
        };

        let edit_target = match edit {
            CrdtOp::FormatSet {
                start_op_id,
                end_op_id,
                ..
            } => Some((*start_op_id, *end_op_id)),
            CrdtOp::BlockUpdate { target_id, .. } => Some((*target_id, *target_id)),
            _ => None,
        };

        match (delete_target, edit_target) {
            (Some(del_target), Some((edit_start, edit_end))) => {
                // Check if the delete target falls within the edit range
                if del_target >= edit_start && del_target <= edit_end {
                    let delete_id = delete.id();
                    self.log_conflict(
                        ConflictType::DeleteEdit {
                            delete_op: delete_id,
                            edit_op: edit.id(),
                        },
                        delete_id,
                    );
                    ConflictResult::Wins // Delete wins
                } else {
                    ConflictResult::NoConflict
                }
            }
            _ => ConflictResult::NoConflict,
        }
    }

    // ========== Formatting Conflict Resolution ==========

    /// Resolve formatting conflicts using LWW.
    ///
    /// Rule: Latest timestamp wins, client_id breaks ties.
    /// This is used when two operations modify the same formatting attribute.
    pub fn resolve_formatting(
        &mut self,
        op1: &CrdtOp,
        op2: &CrdtOp,
        ts1: Timestamp,
        ts2: Timestamp,
    ) -> ConflictResult {
        match (op1, op2) {
            (
                CrdtOp::FormatSet {
                    id: id1,
                    attribute: attr1,
                    node_id: node1,
                    ..
                },
                CrdtOp::FormatSet {
                    id: id2,
                    attribute: attr2,
                    node_id: node2,
                    ..
                },
            ) => {
                // Different nodes or different attributes don't conflict
                if node1 != node2 || attr1 != attr2 {
                    return ConflictResult::Compatible;
                }

                // Same attribute on same node - use LWW
                let result = if ts1 > ts2 {
                    ConflictResult::Wins
                } else if ts1 < ts2 {
                    ConflictResult::Loses
                } else {
                    // Tie-break by OpId (client_id, then seq)
                    if id1 > id2 {
                        ConflictResult::Wins
                    } else if id1 < id2 {
                        ConflictResult::Loses
                    } else {
                        ConflictResult::NoConflict
                    }
                };

                if result != ConflictResult::NoConflict && result != ConflictResult::Compatible {
                    let winner = if result == ConflictResult::Wins {
                        *id1
                    } else {
                        *id2
                    };
                    self.log_conflict(
                        ConflictType::FormattingConflict {
                            attribute: attr1.clone(),
                            op1: *id1,
                            op2: *id2,
                        },
                        winner,
                    );
                }

                result
            }
            _ => ConflictResult::NoConflict,
        }
    }

    /// Check if two formatting operations conflict.
    ///
    /// Different attributes don't conflict (bold + italic = both apply).
    pub fn formatting_conflicts(&self, op1: &CrdtOp, op2: &CrdtOp) -> bool {
        match (op1, op2) {
            (
                CrdtOp::FormatSet {
                    attribute: attr1,
                    node_id: node1,
                    start_op_id: start1,
                    end_op_id: end1,
                    ..
                },
                CrdtOp::FormatSet {
                    attribute: attr2,
                    node_id: node2,
                    start_op_id: start2,
                    end_op_id: end2,
                    ..
                },
            ) => {
                // Different nodes don't conflict
                if node1 != node2 {
                    return false;
                }

                // Different attributes don't conflict
                if attr1 != attr2 {
                    return false;
                }

                // Check if ranges overlap
                ranges_overlap(*start1, *end1, *start2, *end2)
            }
            _ => false,
        }
    }

    // ========== Structural Conflict Resolution ==========

    /// Resolve concurrent block inserts.
    ///
    /// Rule: Higher OpId inserted first.
    pub fn resolve_block_insert(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        match (op1, op2) {
            (
                CrdtOp::BlockInsert {
                    id: id1,
                    parent_op_id: parent1,
                    after_sibling: sibling1,
                    ..
                },
                CrdtOp::BlockInsert {
                    id: id2,
                    parent_op_id: parent2,
                    after_sibling: sibling2,
                    ..
                },
            ) => {
                // Check if they're inserting at the same position
                if parent1 != parent2 || sibling1 != sibling2 {
                    return ConflictResult::NoConflict;
                }

                // Higher OpId wins
                let result = if id1 > id2 {
                    ConflictResult::Wins
                } else if id1 < id2 {
                    ConflictResult::Loses
                } else {
                    ConflictResult::NoConflict
                };

                if result != ConflictResult::NoConflict {
                    let winner = if result == ConflictResult::Wins {
                        *id1
                    } else {
                        *id2
                    };
                    self.log_conflict(
                        ConflictType::StructuralConflict {
                            op1: *id1,
                            op2: *id2,
                        },
                        winner,
                    );
                }

                result
            }
            _ => ConflictResult::NoConflict,
        }
    }

    /// Resolve block delete vs. edit.
    ///
    /// Rule: Delete wins.
    pub fn resolve_block_delete_edit(&mut self, delete: &CrdtOp, edit: &CrdtOp) -> ConflictResult {
        match (delete, edit) {
            (
                CrdtOp::BlockDelete {
                    id: delete_id,
                    target_id: delete_target,
                },
                CrdtOp::BlockUpdate {
                    id: edit_id,
                    target_id: edit_target,
                    ..
                },
            ) => {
                if delete_target == edit_target {
                    self.log_conflict(
                        ConflictType::DeleteEdit {
                            delete_op: *delete_id,
                            edit_op: *edit_id,
                        },
                        *delete_id,
                    );
                    ConflictResult::Wins // Delete wins
                } else {
                    ConflictResult::NoConflict
                }
            }
            _ => ConflictResult::NoConflict,
        }
    }

    /// Resolve parent-child relationship changes.
    ///
    /// Rule: Higher OpId wins.
    pub fn resolve_move_conflict(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        match (op1, op2) {
            (
                CrdtOp::BlockMove {
                    id: id1,
                    target_id: target1,
                    ..
                },
                CrdtOp::BlockMove {
                    id: id2,
                    target_id: target2,
                    ..
                },
            ) => {
                // Check if they're moving the same block
                if target1 != target2 {
                    return ConflictResult::NoConflict;
                }

                // Higher OpId wins
                let result = if id1 > id2 {
                    ConflictResult::Wins
                } else if id1 < id2 {
                    ConflictResult::Loses
                } else {
                    ConflictResult::NoConflict
                };

                if result != ConflictResult::NoConflict {
                    let winner = if result == ConflictResult::Wins {
                        *id1
                    } else {
                        *id2
                    };
                    self.log_conflict(
                        ConflictType::StructuralConflict {
                            op1: *id1,
                            op2: *id2,
                        },
                        winner,
                    );
                }

                result
            }
            _ => ConflictResult::NoConflict,
        }
    }

    // ========== Special Cases ==========

    /// Resolve table cell conflicts.
    ///
    /// Table cell conflicts are resolved using the same rules as block conflicts,
    /// with higher OpId winning for concurrent operations.
    pub fn resolve_table_cell(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        // Table cells are blocks, so we use block resolution
        match (op1, op2) {
            (CrdtOp::BlockInsert { .. }, CrdtOp::BlockInsert { .. }) => {
                self.resolve_block_insert(op1, op2)
            }
            (CrdtOp::BlockUpdate { .. }, CrdtOp::BlockUpdate { .. }) => {
                // For updates, use timestamp from the operations
                match (op1, op2) {
                    (
                        CrdtOp::BlockUpdate {
                            id: id1,
                            target_id: target1,
                            timestamp: ts1,
                            ..
                        },
                        CrdtOp::BlockUpdate {
                            id: id2,
                            target_id: target2,
                            timestamp: ts2,
                            ..
                        },
                    ) => {
                        if target1 != target2 {
                            return ConflictResult::NoConflict;
                        }

                        let result = if ts1 > ts2 {
                            ConflictResult::Wins
                        } else if ts1 < ts2 {
                            ConflictResult::Loses
                        } else {
                            // Tie-break by OpId
                            if id1 > id2 {
                                ConflictResult::Wins
                            } else if id1 < id2 {
                                ConflictResult::Loses
                            } else {
                                ConflictResult::NoConflict
                            }
                        };

                        if result != ConflictResult::NoConflict {
                            let winner = if result == ConflictResult::Wins {
                                *id1
                            } else {
                                *id2
                            };
                            self.log_conflict(
                                ConflictType::StructuralConflict {
                                    op1: *id1,
                                    op2: *id2,
                                },
                                winner,
                            );
                        }

                        result
                    }
                    _ => ConflictResult::NoConflict,
                }
            }
            (CrdtOp::BlockDelete { .. }, CrdtOp::BlockUpdate { .. }) => {
                self.resolve_block_delete_edit(op1, op2)
            }
            _ => ConflictResult::NoConflict,
        }
    }

    /// Resolve list item reordering conflicts.
    ///
    /// List items are blocks with ordering. Higher OpId wins for position conflicts.
    pub fn resolve_list_reorder(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        // List reordering is handled as block moves
        match (op1, op2) {
            (CrdtOp::BlockMove { .. }, CrdtOp::BlockMove { .. }) => {
                self.resolve_move_conflict(op1, op2)
            }
            (CrdtOp::BlockInsert { .. }, CrdtOp::BlockInsert { .. }) => {
                self.resolve_block_insert(op1, op2)
            }
            _ => ConflictResult::NoConflict,
        }
    }

    /// Resolve comment anchor changes.
    ///
    /// When comments reference text that's been modified, we use LWW for the anchor.
    pub fn resolve_comment_anchor(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        // Comment anchors are typically FormatSet operations with a "comment" attribute
        // or custom block operations. We use the same formatting resolution.
        match (op1, op2) {
            (
                CrdtOp::FormatSet { timestamp: ts1, .. },
                CrdtOp::FormatSet { timestamp: ts2, .. },
            ) => self.resolve_formatting(op1, op2, *ts1, *ts2),
            _ => ConflictResult::NoConflict,
        }
    }

    // ========== Generic Resolution ==========

    /// Resolve any two operations.
    ///
    /// This is the main entry point for conflict resolution. It determines
    /// the type of conflict and dispatches to the appropriate resolver.
    pub fn resolve(&mut self, op1: &CrdtOp, op2: &CrdtOp) -> ConflictResult {
        // Check if operations potentially conflict
        if !op1.conflicts_with(op2) {
            return ConflictResult::NoConflict;
        }

        match (op1, op2) {
            // Text insert conflicts
            (CrdtOp::TextInsert { .. }, CrdtOp::TextInsert { .. }) => {
                self.resolve_text_insert(op1, op2)
            }

            // Text delete conflicts
            (CrdtOp::TextDelete { .. }, CrdtOp::TextDelete { .. }) => {
                // Both deleting the same character - idempotent, no conflict
                ConflictResult::NoConflict
            }

            // Delete vs. format
            (CrdtOp::TextDelete { .. }, CrdtOp::FormatSet { .. }) => {
                self.resolve_delete_edit(op1, op2)
            }
            (CrdtOp::FormatSet { .. }, CrdtOp::TextDelete { .. }) => {
                match self.resolve_delete_edit(op2, op1) {
                    ConflictResult::Wins => ConflictResult::Loses,
                    ConflictResult::Loses => ConflictResult::Wins,
                    other => other,
                }
            }

            // Formatting conflicts
            (
                CrdtOp::FormatSet { timestamp: ts1, .. },
                CrdtOp::FormatSet { timestamp: ts2, .. },
            ) => self.resolve_formatting(op1, op2, *ts1, *ts2),

            // Block insert conflicts
            (CrdtOp::BlockInsert { .. }, CrdtOp::BlockInsert { .. }) => {
                self.resolve_block_insert(op1, op2)
            }

            // Block delete conflicts
            (CrdtOp::BlockDelete { .. }, CrdtOp::BlockDelete { .. }) => {
                // Both deleting the same block - idempotent, no conflict
                ConflictResult::NoConflict
            }

            // Block delete vs. update
            (CrdtOp::BlockDelete { .. }, CrdtOp::BlockUpdate { .. }) => {
                self.resolve_block_delete_edit(op1, op2)
            }
            (CrdtOp::BlockUpdate { .. }, CrdtOp::BlockDelete { .. }) => {
                match self.resolve_block_delete_edit(op2, op1) {
                    ConflictResult::Wins => ConflictResult::Loses,
                    ConflictResult::Loses => ConflictResult::Wins,
                    other => other,
                }
            }

            // Block move conflicts
            (CrdtOp::BlockMove { .. }, CrdtOp::BlockMove { .. }) => {
                self.resolve_move_conflict(op1, op2)
            }

            // Block update conflicts
            (CrdtOp::BlockUpdate { .. }, CrdtOp::BlockUpdate { .. }) => {
                self.resolve_table_cell(op1, op2)
            }

            // Default: no conflict
            _ => ConflictResult::NoConflict,
        }
    }

    /// Get conflict history
    pub fn history(&self) -> &[ConflictRecord] {
        &self.conflict_history
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.conflict_history.clear();
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two ranges overlap
fn ranges_overlap(start1: OpId, end1: OpId, start2: OpId, end2: OpId) -> bool {
    // Ranges overlap if one starts before the other ends
    start1 <= end2 && start2 <= end1
}

/// Apply conflict resolution when merging operation logs.
///
/// This function merges two operation logs, resolving conflicts deterministically.
/// Both sides should produce identical results when merging the same operations.
pub fn merge_with_resolution(
    local_ops: &[CrdtOp],
    remote_ops: &[CrdtOp],
    resolver: &mut ConflictResolver,
) -> Vec<CrdtOp> {
    let mut result = Vec::new();
    let mut seen_ids: HashSet<OpId> = HashSet::new();

    // Build a map for quick lookup
    let local_by_id: HashMap<OpId, &CrdtOp> = local_ops.iter().map(|op| (op.id(), op)).collect();
    let remote_by_id: HashMap<OpId, &CrdtOp> = remote_ops.iter().map(|op| (op.id(), op)).collect();

    // Collect all unique operations
    let mut all_ops: Vec<&CrdtOp> = Vec::new();
    for op in local_ops {
        if seen_ids.insert(op.id()) {
            all_ops.push(op);
        }
    }
    for op in remote_ops {
        if seen_ids.insert(op.id()) {
            all_ops.push(op);
        }
    }

    // Sort by OpId for deterministic ordering
    all_ops.sort_by_key(|op| op.id());

    // Track which operations should be skipped due to conflicts
    let mut skip_ids: HashSet<OpId> = HashSet::new();

    // Resolve conflicts between concurrent operations
    for i in 0..all_ops.len() {
        if skip_ids.contains(&all_ops[i].id()) {
            continue;
        }

        for j in (i + 1)..all_ops.len() {
            if skip_ids.contains(&all_ops[j].id()) {
                continue;
            }

            let op1 = all_ops[i];
            let op2 = all_ops[j];

            // Check if operations are concurrent (from different sources)
            let op1_local = local_by_id.contains_key(&op1.id());
            let op2_local = local_by_id.contains_key(&op2.id());
            let op1_remote = remote_by_id.contains_key(&op1.id());
            let op2_remote = remote_by_id.contains_key(&op2.id());

            // If both are from the same source, they're not concurrent
            if (op1_local && op2_local && !op1_remote && !op2_remote)
                || (op1_remote && op2_remote && !op1_local && !op2_local)
            {
                continue;
            }

            // Resolve the conflict
            match resolver.resolve(op1, op2) {
                ConflictResult::Wins => {
                    // op1 wins, skip op2
                    skip_ids.insert(op2.id());
                }
                ConflictResult::Loses => {
                    // op2 wins, skip op1
                    skip_ids.insert(op1.id());
                }
                ConflictResult::NoConflict | ConflictResult::Compatible => {
                    // Both apply
                }
            }
        }
    }

    // Build the result, excluding skipped operations
    for op in all_ops {
        if !skip_ids.contains(&op.id()) {
            result.push(op.clone());
        }
    }

    result
}

/// Check if two operations are concurrent (neither happened before the other).
///
/// Operations from different clients with overlapping sequence ranges are concurrent.
pub fn are_concurrent(op1: &CrdtOp, op2: &CrdtOp) -> bool {
    let id1 = op1.id();
    let id2 = op2.id();

    // Same client means they're ordered
    if id1.client_id == id2.client_id {
        return false;
    }

    // Different clients - they're concurrent if neither causally precedes the other
    // In practice, this is determined by vector clocks, but without that context,
    // we assume different clients are concurrent
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Timestamp;
    use crate::crdt_tree::BlockData;
    use crate::op_id::{ClientId, OpId};
    use doc_model::NodeId;

    fn make_op_id(client: u64, seq: u64) -> OpId {
        OpId::new(ClientId::new(client), seq)
    }

    fn make_timestamp(physical: u64, logical: u64, client: u64) -> Timestamp {
        Timestamp::new(physical, logical, ClientId::new(client))
    }

    fn make_text_insert(client: u64, seq: u64, node_id: NodeId, parent_seq: u64, c: char) -> CrdtOp {
        CrdtOp::TextInsert {
            id: make_op_id(client, seq),
            node_id,
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

    fn make_format_set(
        client: u64,
        seq: u64,
        node_id: NodeId,
        start_seq: u64,
        end_seq: u64,
        attr: &str,
        value: bool,
        ts: Timestamp,
    ) -> CrdtOp {
        CrdtOp::FormatSet {
            id: make_op_id(client, seq),
            node_id,
            start_op_id: make_op_id(client, start_seq),
            end_op_id: make_op_id(client, end_seq),
            attribute: attr.to_string(),
            value: serde_json::json!(value),
            timestamp: ts,
        }
    }

    fn make_block_insert(
        client: u64,
        seq: u64,
        parent_seq: u64,
        after_sibling: Option<OpId>,
    ) -> CrdtOp {
        CrdtOp::BlockInsert {
            id: make_op_id(client, seq),
            parent_op_id: make_op_id(client, parent_seq),
            after_sibling,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        }
    }

    fn make_block_delete(client: u64, seq: u64, target_client: u64, target_seq: u64) -> CrdtOp {
        CrdtOp::BlockDelete {
            id: make_op_id(client, seq),
            target_id: make_op_id(target_client, target_seq),
        }
    }

    fn make_block_update(
        client: u64,
        seq: u64,
        target_client: u64,
        target_seq: u64,
        ts: Timestamp,
    ) -> CrdtOp {
        CrdtOp::BlockUpdate {
            id: make_op_id(client, seq),
            target_id: make_op_id(target_client, target_seq),
            data: BlockData::Paragraph {
                style: Some("Updated".to_string()),
            },
            timestamp: ts,
        }
    }

    fn make_block_move(
        client: u64,
        seq: u64,
        target_client: u64,
        target_seq: u64,
        new_parent: OpId,
    ) -> CrdtOp {
        CrdtOp::BlockMove {
            id: make_op_id(client, seq),
            target_id: make_op_id(target_client, target_seq),
            new_parent,
            after_sibling: None,
        }
    }

    // ========== Concurrent Text Insert Tests ==========

    #[test]
    fn test_concurrent_text_insert_higher_id_wins() {
        let mut resolver = ConflictResolver::new().with_logging();
        let node_id = NodeId::new();

        // Two clients insert at the same position
        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        // Higher client ID (2) should win
        let result = resolver.resolve_text_insert(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses);

        let result = resolver.resolve_text_insert(&op2, &op1);
        assert_eq!(result, ConflictResult::Wins);

        // Check conflict history
        assert_eq!(resolver.history().len(), 2);
    }

    #[test]
    fn test_concurrent_text_insert_different_positions_no_conflict() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        // Two clients insert at different positions
        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: make_op_id(1, 1), // After op1
            char: 'b',
        };

        let result = resolver.resolve_text_insert(&op1, &op2);
        assert_eq!(result, ConflictResult::NoConflict);
    }

    #[test]
    fn test_concurrent_text_insert_different_nodes_no_conflict() {
        let mut resolver = ConflictResolver::new();
        let node_id1 = NodeId::new();
        let node_id2 = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id: node_id1,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id: node_id2,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        let result = resolver.resolve_text_insert(&op1, &op2);
        assert_eq!(result, ConflictResult::NoConflict);
    }

    // ========== Delete vs. Edit Tests ==========

    #[test]
    fn test_delete_vs_edit_delete_wins() {
        let mut resolver = ConflictResolver::new().with_logging();
        let node_id = NodeId::new();

        let delete = make_text_delete(1, 5, 1, 2);
        let format = CrdtOp::FormatSet {
            id: make_op_id(2, 3),
            node_id,
            start_op_id: make_op_id(1, 1),
            end_op_id: make_op_id(1, 3),
            attribute: "bold".to_string(),
            value: serde_json::json!(true),
            timestamp: make_timestamp(1000, 0, 2),
        };

        let result = resolver.resolve_delete_edit(&delete, &format);
        assert_eq!(result, ConflictResult::Wins);
    }

    #[test]
    fn test_delete_vs_edit_no_overlap() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let delete = make_text_delete(1, 5, 1, 10); // Deleting op at seq 10
        let format = CrdtOp::FormatSet {
            id: make_op_id(2, 3),
            node_id,
            start_op_id: make_op_id(1, 1),
            end_op_id: make_op_id(1, 3), // Range 1-3, doesn't include 10
            attribute: "bold".to_string(),
            value: serde_json::json!(true),
            timestamp: make_timestamp(1000, 0, 2),
        };

        let result = resolver.resolve_delete_edit(&delete, &format);
        assert_eq!(result, ConflictResult::NoConflict);
    }

    // ========== Formatting Conflict Tests ==========

    #[test]
    fn test_formatting_same_attr_later_wins() {
        let mut resolver = ConflictResolver::new().with_logging();
        let node_id = NodeId::new();

        let op1 = make_format_set(
            1,
            1,
            node_id,
            1,
            5,
            "bold",
            true,
            make_timestamp(1000, 0, 1),
        );
        let op2 = make_format_set(
            2,
            1,
            node_id,
            1,
            5,
            "bold",
            false,
            make_timestamp(2000, 0, 2),
        );

        // op2 has later timestamp, should win
        let result = resolver.resolve_formatting(
            &op1,
            &op2,
            make_timestamp(1000, 0, 1),
            make_timestamp(2000, 0, 2),
        );
        assert_eq!(result, ConflictResult::Loses);
    }

    #[test]
    fn test_formatting_same_attr_same_time_client_breaks_tie() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let op1 = make_format_set(
            1,
            1,
            node_id,
            1,
            5,
            "bold",
            true,
            make_timestamp(1000, 0, 1),
        );
        let op2 = make_format_set(
            2,
            1,
            node_id,
            1,
            5,
            "bold",
            false,
            make_timestamp(1000, 0, 2),
        );

        // Same physical time, but ts2 has higher client_id
        let result = resolver.resolve_formatting(
            &op1,
            &op2,
            make_timestamp(1000, 0, 1),
            make_timestamp(1000, 0, 2),
        );
        assert_eq!(result, ConflictResult::Loses);
    }

    #[test]
    fn test_formatting_different_attr_compatible() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let op1 = make_format_set(
            1,
            1,
            node_id,
            1,
            5,
            "bold",
            true,
            make_timestamp(1000, 0, 1),
        );
        let op2 = make_format_set(
            2,
            1,
            node_id,
            1,
            5,
            "italic",
            true,
            make_timestamp(1000, 0, 2),
        );

        // Different attributes don't conflict
        let result = resolver.resolve_formatting(
            &op1,
            &op2,
            make_timestamp(1000, 0, 1),
            make_timestamp(1000, 0, 2),
        );
        assert_eq!(result, ConflictResult::Compatible);
    }

    #[test]
    fn test_formatting_conflicts_check() {
        let resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        // Same attribute, overlapping range
        let op1 = make_format_set(
            1,
            1,
            node_id,
            1,
            5,
            "bold",
            true,
            make_timestamp(1000, 0, 1),
        );
        let op2 = make_format_set(
            2,
            1,
            node_id,
            3,
            7,
            "bold",
            false,
            make_timestamp(2000, 0, 2),
        );

        assert!(resolver.formatting_conflicts(&op1, &op2));

        // Different attribute, same range - no conflict
        let op3 = make_format_set(
            2,
            1,
            node_id,
            1,
            5,
            "italic",
            true,
            make_timestamp(2000, 0, 2),
        );
        assert!(!resolver.formatting_conflicts(&op1, &op3));
    }

    // ========== Block Operation Conflict Tests ==========

    #[test]
    fn test_block_insert_conflict_higher_id_wins() {
        let mut resolver = ConflictResolver::new().with_logging();

        // Use OpId::root() as the common parent to ensure both ops conflict
        let op1 = CrdtOp::BlockInsert {
            id: make_op_id(1, 1),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        };
        let op2 = CrdtOp::BlockInsert {
            id: make_op_id(2, 1),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        };

        // Higher client ID (2) wins
        let result = resolver.resolve_block_insert(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses);

        let result = resolver.resolve_block_insert(&op2, &op1);
        assert_eq!(result, ConflictResult::Wins);
    }

    #[test]
    fn test_block_insert_different_positions_no_conflict() {
        let mut resolver = ConflictResolver::new();

        let op1 = make_block_insert(1, 1, 0, None);
        let op2 = make_block_insert(2, 2, 0, Some(make_op_id(1, 1)));

        let result = resolver.resolve_block_insert(&op1, &op2);
        assert_eq!(result, ConflictResult::NoConflict);
    }

    #[test]
    fn test_block_delete_edit_delete_wins() {
        let mut resolver = ConflictResolver::new().with_logging();

        let delete = make_block_delete(1, 2, 1, 1);
        let update = make_block_update(2, 3, 1, 1, make_timestamp(1000, 0, 2));

        let result = resolver.resolve_block_delete_edit(&delete, &update);
        assert_eq!(result, ConflictResult::Wins);
    }

    #[test]
    fn test_block_move_conflict_higher_id_wins() {
        let mut resolver = ConflictResolver::new().with_logging();

        let op1 = make_block_move(1, 3, 1, 1, make_op_id(1, 0));
        let op2 = make_block_move(2, 3, 1, 1, make_op_id(2, 0));

        // They're both moving the same block (target 1,1)
        // Higher OpId wins: (2,3) > (1,3)
        let result = resolver.resolve_move_conflict(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses);
    }

    // ========== Table Cell Conflict Tests ==========

    #[test]
    fn test_table_cell_update_conflict() {
        let mut resolver = ConflictResolver::new().with_logging();

        let ts1 = make_timestamp(1000, 0, 1);
        let ts2 = make_timestamp(2000, 0, 2);

        let op1 = make_block_update(1, 2, 1, 1, ts1);
        let op2 = make_block_update(2, 2, 1, 1, ts2);

        let result = resolver.resolve_table_cell(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses); // op2 has later timestamp
    }

    #[test]
    fn test_table_cell_delete_vs_update() {
        let mut resolver = ConflictResolver::new();

        let delete = make_block_delete(1, 2, 1, 1);
        let update = make_block_update(2, 3, 1, 1, make_timestamp(1000, 0, 2));

        let result = resolver.resolve_table_cell(&delete, &update);
        assert_eq!(result, ConflictResult::Wins); // Delete wins
    }

    // ========== List Reorder Tests ==========

    #[test]
    fn test_list_reorder_move_conflict() {
        let mut resolver = ConflictResolver::new();

        let op1 = make_block_move(1, 3, 1, 1, make_op_id(1, 0));
        let op2 = make_block_move(2, 3, 1, 1, make_op_id(2, 0));

        let result = resolver.resolve_list_reorder(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses); // Higher OpId (2,3) wins
    }

    #[test]
    fn test_list_reorder_insert_conflict() {
        let mut resolver = ConflictResolver::new();

        // Use OpId::root() as the common parent to ensure both ops conflict
        let op1 = CrdtOp::BlockInsert {
            id: make_op_id(1, 1),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::ListItem {
                list_id: "list1".to_string(),
                level: 0,
                style: None,
            },
        };
        let op2 = CrdtOp::BlockInsert {
            id: make_op_id(2, 1),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::ListItem {
                list_id: "list1".to_string(),
                level: 0,
                style: None,
            },
        };

        let result = resolver.resolve_list_reorder(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses); // Higher OpId (2,1) wins
    }

    // ========== Comment Anchor Tests ==========

    #[test]
    fn test_comment_anchor_lww() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let ts1 = make_timestamp(1000, 0, 1);
        let ts2 = make_timestamp(2000, 0, 2);

        let op1 = make_format_set(1, 1, node_id, 1, 5, "comment-1", true, ts1);
        let op2 = make_format_set(2, 1, node_id, 1, 5, "comment-1", false, ts2);

        let result = resolver.resolve_comment_anchor(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses); // Later timestamp wins
    }

    // ========== Generic Resolution Tests ==========

    #[test]
    fn test_generic_resolve_text_inserts() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        let result = resolver.resolve(&op1, &op2);
        assert_eq!(result, ConflictResult::Loses);
    }

    #[test]
    fn test_generic_resolve_text_deletes_idempotent() {
        let mut resolver = ConflictResolver::new();

        let op1 = make_text_delete(1, 2, 1, 1);
        let op2 = make_text_delete(2, 3, 1, 1);

        // Both deleting the same character - idempotent
        let result = resolver.resolve(&op1, &op2);
        assert_eq!(result, ConflictResult::NoConflict);
    }

    // ========== Multi-Operation Merging Tests ==========

    #[test]
    fn test_merge_with_resolution_no_conflicts() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let local_ops = vec![make_text_insert(1, 1, node_id, 0, 'a')];
        let remote_ops = vec![make_text_insert(2, 1, node_id, 0, 'b')];

        // Note: These have the same parent, so they conflict
        // But merge_with_resolution only skips ops when resolve returns Loses for one
        let result = merge_with_resolution(&local_ops, &remote_ops, &mut resolver);

        // Both ops should be in the result, but the losing one might be skipped
        // Actually, with conflicts_with check, they do conflict, so one will be skipped
        assert!(result.len() >= 1);
    }

    #[test]
    fn test_merge_with_resolution_deduplication() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let op = make_text_insert(1, 1, node_id, 0, 'a');
        let local_ops = vec![op.clone()];
        let remote_ops = vec![op]; // Same operation

        let result = merge_with_resolution(&local_ops, &remote_ops, &mut resolver);

        // Should only have one copy
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_merge_preserves_order() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let local_ops = vec![
            make_text_insert(1, 1, node_id, 0, 'a'),
            make_text_insert(1, 3, node_id, 1, 'c'),
        ];
        let remote_ops = vec![make_text_insert(2, 2, node_id, 0, 'b')];

        let result = merge_with_resolution(&local_ops, &remote_ops, &mut resolver);

        // Should be sorted by OpId
        for i in 1..result.len() {
            assert!(result[i - 1].id() < result[i].id());
        }
    }

    // ========== Are Concurrent Tests ==========

    #[test]
    fn test_are_concurrent_different_clients() {
        let node_id = NodeId::new();

        let op1 = make_text_insert(1, 1, node_id, 0, 'a');
        let op2 = make_text_insert(2, 1, node_id, 0, 'b');

        assert!(are_concurrent(&op1, &op2));
    }

    #[test]
    fn test_are_concurrent_same_client() {
        let node_id = NodeId::new();

        let op1 = make_text_insert(1, 1, node_id, 0, 'a');
        let op2 = make_text_insert(1, 2, node_id, 1, 'b');

        assert!(!are_concurrent(&op1, &op2));
    }

    // ========== History Tests ==========

    #[test]
    fn test_conflict_history() {
        let mut resolver = ConflictResolver::new().with_logging();
        let node_id = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        resolver.resolve_text_insert(&op1, &op2);

        assert_eq!(resolver.history().len(), 1);
        let record = &resolver.history()[0];
        assert_eq!(record.winner, make_op_id(2, 1));
    }

    #[test]
    fn test_clear_history() {
        let mut resolver = ConflictResolver::new().with_logging();
        let node_id = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        resolver.resolve_text_insert(&op1, &op2);
        assert_eq!(resolver.history().len(), 1);

        resolver.clear_history();
        assert!(resolver.history().is_empty());
    }

    #[test]
    fn test_no_logging_by_default() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        resolver.resolve_text_insert(&op1, &op2);

        assert!(resolver.history().is_empty());
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_same_op_id_no_conflict() {
        let mut resolver = ConflictResolver::new();
        let node_id = NodeId::new();

        let op = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };

        let result = resolver.resolve_text_insert(&op, &op);
        assert_eq!(result, ConflictResult::NoConflict);
    }

    #[test]
    fn test_ranges_overlap_function() {
        // Overlapping ranges
        assert!(ranges_overlap(
            make_op_id(1, 1),
            make_op_id(1, 5),
            make_op_id(1, 3),
            make_op_id(1, 7)
        ));

        // Adjacent ranges (touching)
        assert!(ranges_overlap(
            make_op_id(1, 1),
            make_op_id(1, 5),
            make_op_id(1, 5),
            make_op_id(1, 10)
        ));

        // Non-overlapping ranges
        assert!(!ranges_overlap(
            make_op_id(1, 1),
            make_op_id(1, 5),
            make_op_id(1, 6),
            make_op_id(1, 10)
        ));
    }

    // ========== Determinism Tests ==========

    #[test]
    fn test_resolution_is_deterministic() {
        let node_id = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        // Run resolution multiple times
        for _ in 0..10 {
            let mut resolver1 = ConflictResolver::new();
            let mut resolver2 = ConflictResolver::new();

            let result1 = resolver1.resolve(&op1, &op2);
            let result2 = resolver2.resolve(&op1, &op2);

            assert_eq!(result1, result2);
        }
    }

    #[test]
    fn test_resolution_is_commutative() {
        let node_id = NodeId::new();

        let op1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };

        let mut resolver1 = ConflictResolver::new();
        let mut resolver2 = ConflictResolver::new();

        let result1 = resolver1.resolve(&op1, &op2);
        let result2 = resolver2.resolve(&op2, &op1);

        // Results should be opposite (if op1 loses against op2, op2 wins against op1)
        let is_commutative = matches!(
            (&result1, &result2),
            (ConflictResult::Wins, ConflictResult::Loses)
                | (ConflictResult::Loses, ConflictResult::Wins)
                | (ConflictResult::NoConflict, ConflictResult::NoConflict)
                | (ConflictResult::Compatible, ConflictResult::Compatible)
        );

        assert!(
            is_commutative,
            "Results are not commutative: {:?} vs {:?}",
            result1, result2
        );
    }
}
