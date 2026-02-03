//! RGA (Replicated Growable Array) CRDT for text sequences
//!
//! RGA is a CRDT that provides:
//! - Ordered sequence semantics (like a list/array)
//! - Concurrent insert/delete operations that converge
//! - Deterministic conflict resolution for concurrent inserts at same position
//!
//! # Algorithm Overview
//!
//! Each element in the sequence is stored as a node with:
//! - A unique OpId (client_id, sequence_number)
//! - An optional value (None = tombstone for deleted items)
//! - A parent_id (the node this was inserted after)
//! - Children (nodes inserted immediately after this one)
//!
//! When multiple nodes have the same parent (concurrent inserts at same position),
//! they are ordered by their OpId in descending order (higher IDs come first in the
//! children list, then we traverse in reverse for left-to-right reading order).
//! This ensures all replicas converge to the same sequence.

use crate::op_id::{ClientId, OpId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the RGA sequence
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RgaNode<T> {
    /// Unique operation ID for this node
    pub id: OpId,
    /// The value (None = tombstone for deleted items)
    pub value: Option<T>,
    /// ID of the parent node (where this was inserted after)
    pub parent_id: Option<OpId>,
    /// IDs of nodes inserted immediately after this one (sorted descending by OpId)
    children: Vec<OpId>,
}

impl<T> RgaNode<T> {
    /// Create a new node
    fn new(id: OpId, value: Option<T>, parent_id: Option<OpId>) -> Self {
        Self {
            id,
            value,
            parent_id,
            children: Vec::new(),
        }
    }

    /// Check if this node is a tombstone (deleted)
    pub fn is_tombstone(&self) -> bool {
        self.value.is_none()
    }

    /// Get the children of this node
    pub fn children(&self) -> &[OpId] {
        &self.children
    }
}

/// Replicated Growable Array - a CRDT for ordered sequences
///
/// # Example
///
/// ```
/// use collab::rga::Rga;
///
/// // Create two replicas
/// let mut replica1 = Rga::<char>::new(1);
/// let mut replica2 = Rga::<char>::new(2);
///
/// // Replica 1 inserts 'a' at the beginning
/// let id_a = replica1.insert(None, 'a');
///
/// // Get operations and apply to replica 2
/// for op in replica1.all_ops() {
///     replica2.apply_operation(&op);
/// }
///
/// // Both replicas now have the same content
/// assert_eq!(replica1.to_vec(), replica2.to_vec());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rga<T> {
    /// All nodes by their OpId
    nodes: HashMap<OpId, RgaNode<T>>,
    /// The root sentinel node ID
    root: OpId,
    /// Current sequence number for this client
    seq: u64,
    /// This client's ID
    client_id: ClientId,
}

impl<T: Clone> Rga<T> {
    /// Create a new RGA for a client
    pub fn new(client_id: impl Into<ClientId>) -> Self {
        let client_id = client_id.into();
        let root_id = OpId::root();
        let mut nodes = HashMap::new();

        // Create the root sentinel node
        let root_node = RgaNode::new(root_id, None, None);
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            root: root_id,
            seq: 0,
            client_id,
        }
    }

    /// Get the client ID
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Get the root OpId
    pub fn root_id(&self) -> OpId {
        self.root
    }

    /// Get the current sequence number
    pub fn current_seq(&self) -> u64 {
        self.seq
    }

    /// Generate the next OpId for this client
    fn next_op_id(&mut self) -> OpId {
        self.seq += 1;
        OpId::new(self.client_id, self.seq)
    }

    /// Insert a value after the given position (OpId)
    /// If parent_id is None, insert after the root (at the beginning)
    /// Returns the OpId of the new node
    pub fn insert(&mut self, parent_id: Option<OpId>, value: T) -> OpId {
        let id = self.next_op_id();
        self.apply_insert(id, parent_id, value);
        id
    }

    /// Apply a remote insert operation
    ///
    /// This is idempotent - applying the same insert twice has no effect.
    pub fn apply_insert(&mut self, id: OpId, parent_id: Option<OpId>, value: T) {
        // Idempotency check - if node already exists, do nothing
        if self.nodes.contains_key(&id) {
            return;
        }

        // Resolve parent_id - use root if None
        let actual_parent_id = parent_id.unwrap_or(self.root);

        // Create the new node
        let node = RgaNode::new(id, Some(value), Some(actual_parent_id));
        self.nodes.insert(id, node);

        // Add to parent's children in sorted order (descending by OpId)
        if let Some(parent) = self.nodes.get_mut(&actual_parent_id) {
            // Find insertion position to maintain descending order
            let pos = parent
                .children
                .iter()
                .position(|&child_id| child_id < id)
                .unwrap_or(parent.children.len());
            parent.children.insert(pos, id);
        }

        // Update our seq if we see a higher sequence number from this client
        if id.client_id == self.client_id && id.seq > self.seq {
            self.seq = id.seq;
        }
    }

    /// Delete the node with the given OpId (marks as tombstone)
    /// Returns true if the node was found and deleted, false otherwise
    pub fn delete(&mut self, id: OpId) -> bool {
        self.apply_delete(id)
    }

    /// Apply a remote delete operation
    ///
    /// This is idempotent - deleting an already-deleted node has no effect.
    /// Returns true if the node was found (even if already deleted).
    pub fn apply_delete(&mut self, id: OpId) -> bool {
        if let Some(node) = self.nodes.get_mut(&id) {
            // Mark as tombstone
            node.value = None;
            true
        } else {
            false
        }
    }

    /// Apply any RGA operation
    pub fn apply_operation(&mut self, op: &RgaOperation<T>) {
        match op {
            RgaOperation::Insert {
                id,
                parent_id,
                value,
            } => {
                self.apply_insert(*id, *parent_id, value.clone());
            }
            RgaOperation::Delete { id } => {
                self.apply_delete(*id);
            }
        }
    }

    /// Perform a depth-first traversal from the root, collecting nodes in order
    fn traverse(&self) -> Vec<OpId> {
        let mut result = Vec::new();
        self.traverse_recursive(self.root, &mut result);
        result
    }

    /// Recursive helper for depth-first traversal
    ///
    /// Children are stored in descending OpId order, so we iterate in reverse
    /// to get the correct left-to-right reading order (older inserts first,
    /// which is the intuitive "left" position).
    fn traverse_recursive(&self, node_id: OpId, result: &mut Vec<OpId>) {
        if let Some(node) = self.nodes.get(&node_id) {
            // Add this node (unless it's the root)
            if !node_id.is_root() {
                result.push(node_id);
            }

            // Recursively traverse children in reverse order
            // (children are sorted descending, reverse gives ascending/left-to-right)
            for &child_id in node.children.iter().rev() {
                self.traverse_recursive(child_id, result);
            }
        }
    }

    /// Get the current sequence as a vector (excluding tombstones)
    pub fn to_vec(&self) -> Vec<&T> {
        self.traverse()
            .iter()
            .filter_map(|&id| self.nodes.get(&id).and_then(|n| n.value.as_ref()))
            .collect()
    }

    /// Get all nodes in order (including tombstones)
    pub fn nodes_in_order(&self) -> Vec<&RgaNode<T>> {
        self.traverse()
            .iter()
            .filter_map(|&id| self.nodes.get(&id))
            .collect()
    }

    /// Get the OpId at a logical index (0-based, excluding tombstones)
    pub fn id_at_index(&self, index: usize) -> Option<OpId> {
        let mut count = 0;
        for id in self.traverse() {
            if let Some(node) = self.nodes.get(&id) {
                if node.value.is_some() {
                    if count == index {
                        return Some(id);
                    }
                    count += 1;
                }
            }
        }
        None
    }

    /// Get the logical index for an OpId (excluding tombstones from count)
    pub fn index_of(&self, id: OpId) -> Option<usize> {
        let mut count = 0;
        for current_id in self.traverse() {
            if let Some(node) = self.nodes.get(&current_id) {
                if current_id == id {
                    return if node.value.is_some() {
                        Some(count)
                    } else {
                        None // Tombstoned node has no visible index
                    };
                }
                if node.value.is_some() {
                    count += 1;
                }
            }
        }
        None
    }

    /// Get the length (excluding tombstones)
    pub fn len(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.value.is_some() && !n.id.is_root())
            .count()
    }

    /// Check if empty (no visible elements)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a node by its OpId
    pub fn get_node(&self, id: OpId) -> Option<&RgaNode<T>> {
        self.nodes.get(&id)
    }

    /// Get a value by its OpId
    pub fn get(&self, id: OpId) -> Option<&T> {
        self.nodes.get(&id).and_then(|n| n.value.as_ref())
    }

    /// Get all operations for sync (useful for replication)
    ///
    /// Note: For tombstoned nodes, only the Delete operation is included.
    /// A complete sync would require tracking original values separately.
    pub fn all_ops(&self) -> Vec<RgaOperation<T>> {
        let mut ops = Vec::new();

        for (&id, node) in &self.nodes {
            // Skip the root sentinel
            if id.is_root() {
                continue;
            }

            if let Some(ref value) = node.value {
                // Node is alive - emit Insert
                ops.push(RgaOperation::Insert {
                    id,
                    parent_id: node.parent_id,
                    value: value.clone(),
                });
            } else {
                // Node is tombstoned - we can't reconstruct the Insert
                // In a real implementation, we'd store the original value
                ops.push(RgaOperation::Delete { id });
            }
        }

        // Sort ops by OpId to ensure deterministic ordering
        ops.sort_by(|a, b| a.op_id().cmp(&b.op_id()));

        ops
    }

    /// Merge another RGA into this one
    ///
    /// This applies all operations from the other RGA that we don't have.
    pub fn merge(&mut self, other: &Rga<T>) {
        for op in other.all_ops() {
            self.apply_operation(&op);
        }
    }
}

/// Operations that can be performed on an RGA
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RgaOperation<T> {
    /// Insert a new element
    Insert {
        /// The unique ID for this insert
        id: OpId,
        /// The parent node (where to insert after, None = beginning)
        parent_id: Option<OpId>,
        /// The value to insert
        value: T,
    },
    /// Delete an element (by marking it as tombstone)
    Delete {
        /// The ID of the node to delete
        id: OpId,
    },
}

impl<T> RgaOperation<T> {
    /// Get the OpId associated with this operation
    pub fn op_id(&self) -> OpId {
        match self {
            RgaOperation::Insert { id, .. } => *id,
            RgaOperation::Delete { id } => *id,
        }
    }

    /// Check if this is an insert operation
    pub fn is_insert(&self) -> bool {
        matches!(self, RgaOperation::Insert { .. })
    }

    /// Check if this is a delete operation
    pub fn is_delete(&self) -> bool {
        matches!(self, RgaOperation::Delete { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_single_element() {
        let mut rga = Rga::<char>::new(1u64);

        let id = rga.insert(None, 'a');
        assert_eq!(rga.len(), 1);
        assert_eq!(rga.to_vec(), vec![&'a']);
        assert_eq!(rga.get(id), Some(&'a'));
    }

    #[test]
    fn test_insert_multiple_elements() {
        let mut rga = Rga::<char>::new(1u64);

        // Insert 'a' at beginning
        let id_a = rga.insert(None, 'a');
        // Insert 'b' after 'a'
        let id_b = rga.insert(Some(id_a), 'b');
        // Insert 'c' after 'b'
        let _id_c = rga.insert(Some(id_b), 'c');

        assert_eq!(rga.len(), 3);
        assert_eq!(rga.to_vec(), vec![&'a', &'b', &'c']);
    }

    #[test]
    fn test_insert_at_beginning_multiple_times() {
        let mut rga = Rga::<char>::new(1u64);

        // Insert 'a' at beginning
        let _id_a = rga.insert(None, 'a');
        // Insert 'b' at beginning (before 'a')
        let _id_b = rga.insert(None, 'b');
        // Insert 'c' at beginning (before 'b')
        let _id_c = rga.insert(None, 'c');

        assert_eq!(rga.len(), 3);
        // Each new insert at root has higher seq, so it goes to the front
        // Children of root: [c(1,3), b(1,2), a(1,1)] (descending by seq)
        // Traverse in reverse: a, b, c
        assert_eq!(rga.to_vec(), vec![&'a', &'b', &'c']);
    }

    #[test]
    fn test_delete_element() {
        let mut rga = Rga::<char>::new(1u64);

        let id_a = rga.insert(None, 'a');
        let id_b = rga.insert(Some(id_a), 'b');
        let _id_c = rga.insert(Some(id_b), 'c');

        assert_eq!(rga.len(), 3);

        // Delete 'b'
        let deleted = rga.delete(id_b);
        assert!(deleted);
        assert_eq!(rga.len(), 2);
        assert_eq!(rga.to_vec(), vec![&'a', &'c']);

        // Node still exists as tombstone
        assert!(rga.get_node(id_b).is_some());
        assert!(rga.get_node(id_b).unwrap().is_tombstone());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut rga = Rga::<char>::new(1u64);
        let fake_id = OpId::new(999u64, 999);
        assert!(!rga.delete(fake_id));
    }

    #[test]
    fn test_delete_idempotent() {
        let mut rga = Rga::<char>::new(1u64);

        let id = rga.insert(None, 'a');
        assert!(rga.delete(id));
        assert!(rga.delete(id)); // Should still return true but no-op
        assert_eq!(rga.len(), 0);
    }

    #[test]
    fn test_concurrent_inserts_same_position() {
        // Simulate two clients inserting at the same position concurrently
        let mut rga1 = Rga::<char>::new(1u64);
        let mut rga2 = Rga::<char>::new(2u64);

        // Both start with 'x' (synced)
        let id_x = rga1.insert(None, 'x');
        rga2.apply_insert(id_x, None, 'x');

        // Client 1 inserts 'a' after 'x'
        let id_a = rga1.insert(Some(id_x), 'a');

        // Client 2 inserts 'b' after 'x' (concurrent)
        let id_b = rga2.insert(Some(id_x), 'b');

        // Now sync: apply each other's operations
        rga1.apply_insert(id_b, Some(id_x), 'b');
        rga2.apply_insert(id_a, Some(id_x), 'a');

        // Both should have the same order
        let vec1 = rga1.to_vec();
        let vec2 = rga2.to_vec();

        assert_eq!(vec1, vec2);
        assert_eq!(vec1.len(), 3);

        // id_x = (client=1, seq=1) - first insert on client 1
        // id_a = (client=1, seq=2) - second insert on client 1
        // id_b = (client=2, seq=1) - first insert on client 2
        // OpId ordering: seq first, then client_id
        // id_a (seq=2) > id_b (seq=1), so:
        // Children of x sorted descending: [id_a, id_b]
        // Traverse in reverse: id_b first, then id_a
        // So order is: x, b, a
        assert_eq!(vec1, vec![&'x', &'b', &'a']);
    }

    #[test]
    fn test_concurrent_insert_delete() {
        let mut rga1 = Rga::<char>::new(1u64);
        let mut rga2 = Rga::<char>::new(2u64);

        // Both start with 'a', 'b', 'c'
        let id_a = rga1.insert(None, 'a');
        let id_b = rga1.insert(Some(id_a), 'b');
        let id_c = rga1.insert(Some(id_b), 'c');

        // Sync to replica 2
        rga2.apply_insert(id_a, None, 'a');
        rga2.apply_insert(id_b, Some(id_a), 'b');
        rga2.apply_insert(id_c, Some(id_b), 'c');

        assert_eq!(rga1.to_vec(), rga2.to_vec());

        // Client 1 deletes 'b'
        rga1.delete(id_b);

        // Client 2 inserts 'x' after 'b' (before knowing about delete)
        let id_x = rga2.insert(Some(id_b), 'x');

        // Sync operations
        rga2.apply_delete(id_b);
        rga1.apply_insert(id_x, Some(id_b), 'x');

        // Both should converge
        let vec1 = rga1.to_vec();
        let vec2 = rga2.to_vec();

        assert_eq!(vec1, vec2);
        // 'b' is deleted, 'x' remains because tombstones preserve structure
        assert_eq!(vec1, vec![&'a', &'x', &'c']);
    }

    #[test]
    fn test_reconstruct_from_operations() {
        let mut rga1 = Rga::<char>::new(1u64);

        // Build a sequence
        let id_a = rga1.insert(None, 'a');
        let id_b = rga1.insert(Some(id_a), 'b');
        let _id_c = rga1.insert(Some(id_b), 'c');

        // Get all operations
        let ops = rga1.all_ops();

        // Create new replica and apply operations
        let mut rga2 = Rga::<char>::new(2u64);
        for op in &ops {
            rga2.apply_operation(op);
        }

        // Should have same content
        assert_eq!(rga1.to_vec(), rga2.to_vec());
        assert_eq!(rga2.to_vec(), vec![&'a', &'b', &'c']);
    }

    #[test]
    fn test_id_at_index() {
        let mut rga = Rga::<char>::new(1u64);

        let id_a = rga.insert(None, 'a');
        let id_b = rga.insert(Some(id_a), 'b');
        let id_c = rga.insert(Some(id_b), 'c');

        assert_eq!(rga.id_at_index(0), Some(id_a));
        assert_eq!(rga.id_at_index(1), Some(id_b));
        assert_eq!(rga.id_at_index(2), Some(id_c));
        assert_eq!(rga.id_at_index(3), None);
    }

    #[test]
    fn test_index_of() {
        let mut rga = Rga::<char>::new(1u64);

        let id_a = rga.insert(None, 'a');
        let id_b = rga.insert(Some(id_a), 'b');
        let id_c = rga.insert(Some(id_b), 'c');

        assert_eq!(rga.index_of(id_a), Some(0));
        assert_eq!(rga.index_of(id_b), Some(1));
        assert_eq!(rga.index_of(id_c), Some(2));

        // Delete 'b' - it should no longer have a visible index
        rga.delete(id_b);
        assert_eq!(rga.index_of(id_b), None);
        assert_eq!(rga.index_of(id_c), Some(1)); // c is now at index 1
    }

    #[test]
    fn test_is_empty() {
        let mut rga = Rga::<char>::new(1u64);
        assert!(rga.is_empty());

        let id = rga.insert(None, 'a');
        assert!(!rga.is_empty());

        rga.delete(id);
        assert!(rga.is_empty());
    }

    #[test]
    fn test_insert_idempotent() {
        let mut rga = Rga::<char>::new(1u64);

        let id = OpId::new(1u64, 1);

        // Apply same insert twice
        rga.apply_insert(id, None, 'a');
        rga.apply_insert(id, None, 'b'); // Should be ignored

        assert_eq!(rga.len(), 1);
        assert_eq!(rga.get(id), Some(&'a')); // Original value preserved
    }

    #[test]
    fn test_three_way_merge() {
        // Three clients all inserting at the same position
        let mut rga1 = Rga::<char>::new(1u64);
        let mut rga2 = Rga::<char>::new(2u64);
        let mut rga3 = Rga::<char>::new(3u64);

        // All three insert at beginning (after root)
        let id_a = rga1.insert(None, 'a'); // OpId(client=1, seq=1)
        let id_b = rga2.insert(None, 'b'); // OpId(client=2, seq=1)
        let id_c = rga3.insert(None, 'c'); // OpId(client=3, seq=1)

        // Sync all operations to all replicas
        rga1.apply_insert(id_b, None, 'b');
        rga1.apply_insert(id_c, None, 'c');

        rga2.apply_insert(id_a, None, 'a');
        rga2.apply_insert(id_c, None, 'c');

        rga3.apply_insert(id_a, None, 'a');
        rga3.apply_insert(id_b, None, 'b');

        // All should converge
        let vec1 = rga1.to_vec();
        let vec2 = rga2.to_vec();
        let vec3 = rga3.to_vec();

        assert_eq!(vec1, vec2);
        assert_eq!(vec2, vec3);

        // All have seq=1, different client_ids: 1, 2, 3
        // OpId ordering: seq first, then client_id
        // (1,1) < (2,1) < (3,1) by client_id
        // Children of root sorted descending: [id_c, id_b, id_a]
        // Traverse in reverse: id_a, id_b, id_c
        assert_eq!(vec1, vec![&'a', &'b', &'c']);
    }

    #[test]
    fn test_interleaved_inserts() {
        let mut rga = Rga::<char>::new(1u64);

        // Insert 'a', then 'c' after 'a', then 'b' after 'a'
        let id_a = rga.insert(None, 'a');
        let _id_c = rga.insert(Some(id_a), 'c');
        let _id_b = rga.insert(Some(id_a), 'b');

        // Children of 'a' sorted descending: [id_b(1,3), id_c(1,2)]
        // Traverse in reverse: id_c(1,2), id_b(1,3)
        // So order is: a, c, b
        assert_eq!(rga.to_vec(), vec![&'a', &'c', &'b']);
    }

    #[test]
    fn test_operations_serialization() {
        let op: RgaOperation<char> = RgaOperation::Insert {
            id: OpId::new(1u64, 1),
            parent_id: None,
            value: 'a',
        };

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: RgaOperation<char> = serde_json::from_str(&json).unwrap();

        match deserialized {
            RgaOperation::Insert { id, value, .. } => {
                assert_eq!(id.seq, 1);
                assert_eq!(value, 'a');
            }
            _ => panic!("Expected Insert operation"),
        }
    }

    #[test]
    fn test_merge_replicas() {
        let mut rga1 = Rga::<char>::new(1u64);
        let mut rga2 = Rga::<char>::new(2u64);

        // Each client makes different changes
        let id_a = rga1.insert(None, 'a');
        let _id_b = rga1.insert(Some(id_a), 'b');

        let id_x = rga2.insert(None, 'x');
        let _id_y = rga2.insert(Some(id_x), 'y');

        // Merge
        rga1.merge(&rga2);
        rga2.merge(&rga1);

        // Both should have same content
        assert_eq!(rga1.to_vec(), rga2.to_vec());
        assert_eq!(rga1.len(), 4);
    }

    #[test]
    fn test_nodes_in_order() {
        let mut rga = Rga::<char>::new(1u64);

        let id_a = rga.insert(None, 'a');
        let id_b = rga.insert(Some(id_a), 'b');
        rga.delete(id_b);
        let _id_c = rga.insert(Some(id_a), 'c');

        let nodes = rga.nodes_in_order();
        assert_eq!(nodes.len(), 3);

        // Check tombstone is included
        let tombstone_count = nodes.iter().filter(|n| n.is_tombstone()).count();
        assert_eq!(tombstone_count, 1);
    }

    #[test]
    fn test_get_node() {
        let mut rga = Rga::<char>::new(1u64);

        let id_a = rga.insert(None, 'a');
        let node = rga.get_node(id_a).unwrap();

        assert_eq!(node.id, id_a);
        assert_eq!(node.value, Some('a'));
        assert_eq!(node.parent_id, Some(rga.root_id()));
    }

    #[test]
    fn test_op_id_helpers() {
        let insert_op: RgaOperation<char> = RgaOperation::Insert {
            id: OpId::new(1u64, 1),
            parent_id: None,
            value: 'a',
        };
        let delete_op: RgaOperation<char> = RgaOperation::Delete {
            id: OpId::new(1u64, 2),
        };

        assert!(insert_op.is_insert());
        assert!(!insert_op.is_delete());
        assert_eq!(insert_op.op_id(), OpId::new(1u64, 1));

        assert!(!delete_op.is_insert());
        assert!(delete_op.is_delete());
        assert_eq!(delete_op.op_id(), OpId::new(1u64, 2));
    }
}
