//! CRDT operation types and serialization
//!
//! This module defines all possible CRDT operations for collaborative editing,
//! along with serialization support and operation batching.

use crate::clock::{Timestamp, VectorClock};
use crate::crdt_tree::BlockData;
use crate::op_id::{ClientId, OpId};
use doc_model::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All possible CRDT operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CrdtOp {
    /// Insert text character
    TextInsert {
        id: OpId,
        node_id: NodeId,    // Which paragraph/text node
        parent_op_id: OpId, // Insert after this character
        char: char,
    },

    /// Delete text character
    TextDelete {
        id: OpId,
        target_id: OpId, // The character to delete
    },

    /// Set formatting attribute
    FormatSet {
        id: OpId,
        node_id: NodeId,
        start_op_id: OpId, // Start of range
        end_op_id: OpId,   // End of range
        attribute: String,
        value: serde_json::Value,
        timestamp: Timestamp,
    },

    /// Insert a block (paragraph, table, image, etc.)
    BlockInsert {
        id: OpId,
        parent_op_id: OpId,
        after_sibling: Option<OpId>,
        node_id: NodeId,
        data: BlockData,
    },

    /// Delete a block
    BlockDelete { id: OpId, target_id: OpId },

    /// Move a block to new position
    BlockMove {
        id: OpId,
        target_id: OpId,
        new_parent: OpId,
        after_sibling: Option<OpId>,
    },

    /// Update block data (e.g., image src, table dimensions)
    BlockUpdate {
        id: OpId,
        target_id: OpId,
        data: BlockData,
        timestamp: Timestamp,
    },
}

impl CrdtOp {
    /// Get the operation ID
    pub fn id(&self) -> OpId {
        match self {
            CrdtOp::TextInsert { id, .. } => *id,
            CrdtOp::TextDelete { id, .. } => *id,
            CrdtOp::FormatSet { id, .. } => *id,
            CrdtOp::BlockInsert { id, .. } => *id,
            CrdtOp::BlockDelete { id, .. } => *id,
            CrdtOp::BlockMove { id, .. } => *id,
            CrdtOp::BlockUpdate { id, .. } => *id,
        }
    }

    /// Get the client who created this operation
    pub fn client_id(&self) -> ClientId {
        self.id().client_id
    }

    /// Check if this operation conflicts with another
    ///
    /// Two operations conflict if they:
    /// - Operate on the same target (character, block)
    /// - Are concurrent (neither causally depends on the other)
    pub fn conflicts_with(&self, other: &CrdtOp) -> bool {
        match (self, other) {
            // Text operations on the same parent/position
            (
                CrdtOp::TextInsert {
                    node_id: n1,
                    parent_op_id: p1,
                    ..
                },
                CrdtOp::TextInsert {
                    node_id: n2,
                    parent_op_id: p2,
                    ..
                },
            ) => n1 == n2 && p1 == p2,

            // Deleting the same character
            (
                CrdtOp::TextDelete { target_id: t1, .. },
                CrdtOp::TextDelete { target_id: t2, .. },
            ) => t1 == t2,

            // Formatting the same range
            (
                CrdtOp::FormatSet {
                    node_id: n1,
                    attribute: a1,
                    ..
                },
                CrdtOp::FormatSet {
                    node_id: n2,
                    attribute: a2,
                    ..
                },
            ) => n1 == n2 && a1 == a2,

            // Block operations on the same target
            (
                CrdtOp::BlockInsert {
                    parent_op_id: p1,
                    after_sibling: s1,
                    ..
                },
                CrdtOp::BlockInsert {
                    parent_op_id: p2,
                    after_sibling: s2,
                    ..
                },
            ) => p1 == p2 && s1 == s2,

            (
                CrdtOp::BlockDelete { target_id: t1, .. },
                CrdtOp::BlockDelete { target_id: t2, .. },
            ) => t1 == t2,

            (CrdtOp::BlockMove { target_id: t1, .. }, CrdtOp::BlockMove { target_id: t2, .. }) => {
                t1 == t2
            }

            (
                CrdtOp::BlockUpdate { target_id: t1, .. },
                CrdtOp::BlockUpdate { target_id: t2, .. },
            ) => t1 == t2,

            // Different operation types don't conflict directly
            // (though there may be semantic conflicts)
            _ => false,
        }
    }

    /// Get the target ID if this operation modifies an existing element
    pub fn target_id(&self) -> Option<OpId> {
        match self {
            CrdtOp::TextInsert { .. } => None,
            CrdtOp::TextDelete { target_id, .. } => Some(*target_id),
            CrdtOp::FormatSet { .. } => None,
            CrdtOp::BlockInsert { .. } => None,
            CrdtOp::BlockDelete { target_id, .. } => Some(*target_id),
            CrdtOp::BlockMove { target_id, .. } => Some(*target_id),
            CrdtOp::BlockUpdate { target_id, .. } => Some(*target_id),
        }
    }

    /// Check if this is a delete operation
    pub fn is_delete(&self) -> bool {
        matches!(self, CrdtOp::TextDelete { .. } | CrdtOp::BlockDelete { .. })
    }

    /// Check if this is an insert operation
    pub fn is_insert(&self) -> bool {
        matches!(self, CrdtOp::TextInsert { .. } | CrdtOp::BlockInsert { .. })
    }
}

/// A batch of operations with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpBatch {
    /// The operations in this batch
    pub ops: Vec<CrdtOp>,
    /// Vector clock after these operations
    pub clock: VectorClock,
    /// Client who sent this batch
    pub client_id: ClientId,
    /// Sequence number for this batch
    pub batch_seq: u64,
}

impl OpBatch {
    /// Create a new empty batch
    pub fn new(client_id: ClientId, batch_seq: u64) -> Self {
        Self {
            ops: Vec::new(),
            clock: VectorClock::new(),
            client_id,
            batch_seq,
        }
    }

    /// Add an operation to the batch
    pub fn add(&mut self, op: CrdtOp) {
        // Update the clock with this operation's ID
        let op_id = op.id();
        let current = self.clock.get(op_id.client_id);
        if op_id.seq > current {
            self.clock.set(op_id.client_id, op_id.seq);
        }
        self.ops.push(op);
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Get the number of operations in the batch
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Iterate over operations
    pub fn iter(&self) -> impl Iterator<Item = &CrdtOp> {
        self.ops.iter()
    }
}

/// Operation log for persistence and sync
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OpLog {
    /// All operations in order received
    ops: Vec<CrdtOp>,
    /// Index by operation ID for fast lookup
    #[serde(skip)]
    index: HashMap<OpId, usize>,
    /// Current vector clock
    clock: VectorClock,
}

impl OpLog {
    /// Create a new empty operation log
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            index: HashMap::new(),
            clock: VectorClock::new(),
        }
    }

    /// Add an operation (returns false if already exists)
    pub fn add(&mut self, op: CrdtOp) -> bool {
        let op_id = op.id();

        // Check for duplicate
        if self.index.contains_key(&op_id) {
            return false;
        }

        // Add to log
        let idx = self.ops.len();
        self.index.insert(op_id, idx);
        self.ops.push(op);

        // Update clock
        let current = self.clock.get(op_id.client_id);
        if op_id.seq > current {
            self.clock.set(op_id.client_id, op_id.seq);
        }

        true
    }

    /// Get operation by ID
    pub fn get(&self, id: OpId) -> Option<&CrdtOp> {
        self.index.get(&id).map(|&idx| &self.ops[idx])
    }

    /// Check if operation exists
    pub fn contains(&self, id: OpId) -> bool {
        self.index.contains_key(&id)
    }

    /// Get all operations after a given vector clock
    ///
    /// Returns operations whose ID is not dominated by the given clock.
    pub fn ops_since(&self, clock: &VectorClock) -> Vec<&CrdtOp> {
        self.ops
            .iter()
            .filter(|op| {
                let op_id = op.id();
                clock.get(op_id.client_id) < op_id.seq
            })
            .collect()
    }

    /// Get current clock
    pub fn clock(&self) -> &VectorClock {
        &self.clock
    }

    /// Get number of operations
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Iterate over all operations
    pub fn iter(&self) -> impl Iterator<Item = &CrdtOp> {
        self.ops.iter()
    }

    /// Serialize to binary (for persistence)
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from binary
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let mut log: OpLog = serde_json::from_slice(bytes)?;
        // Rebuild the index
        log.rebuild_index();
        Ok(log)
    }

    /// Serialize to JSON (for debugging)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Rebuild the index from the operations list
    fn rebuild_index(&mut self) {
        self.index.clear();
        for (idx, op) in self.ops.iter().enumerate() {
            self.index.insert(op.id(), idx);
        }
    }

    /// Get operations for a specific client
    pub fn ops_for_client(&self, client_id: ClientId) -> Vec<&CrdtOp> {
        self.ops
            .iter()
            .filter(|op| op.client_id() == client_id)
            .collect()
    }

    /// Get the latest sequence number for a client
    pub fn latest_seq(&self, client_id: ClientId) -> u64 {
        self.clock.get(client_id)
    }
}

/// Wire format for sending operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireOp {
    /// Binary-encoded operation
    pub data: Vec<u8>,
    /// Version for forward compatibility
    pub version: u32,
}

/// Current wire protocol version
pub const WIRE_VERSION: u32 = 1;

impl WireOp {
    /// Encode an operation to wire format
    pub fn encode(op: &CrdtOp) -> Result<Self, serde_json::Error> {
        let data = serde_json::to_vec(op)?;
        Ok(Self {
            data,
            version: WIRE_VERSION,
        })
    }

    /// Decode an operation from wire format
    pub fn decode(&self) -> Result<CrdtOp, serde_json::Error> {
        // In the future, we might need version-specific decoding
        if self.version != WIRE_VERSION {
            // For now, try to decode anyway (forward compatibility)
        }
        serde_json::from_slice(&self.data)
    }

    /// Encode multiple operations
    pub fn encode_batch(ops: &[CrdtOp]) -> Result<Vec<Self>, serde_json::Error> {
        ops.iter().map(Self::encode).collect()
    }

    /// Decode multiple operations
    pub fn decode_batch(wire_ops: &[WireOp]) -> Result<Vec<CrdtOp>, serde_json::Error> {
        wire_ops.iter().map(|w| w.decode()).collect()
    }
}

/// Wire format for a batch of operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireBatch {
    /// Encoded operations
    pub ops: Vec<WireOp>,
    /// Vector clock after these operations
    pub clock: VectorClock,
    /// Client who sent this batch
    pub client_id: ClientId,
    /// Batch sequence number
    pub batch_seq: u64,
    /// Protocol version
    pub version: u32,
}

impl WireBatch {
    /// Encode an OpBatch to wire format
    pub fn encode(batch: &OpBatch) -> Result<Self, serde_json::Error> {
        let ops = WireOp::encode_batch(&batch.ops)?;
        Ok(Self {
            ops,
            clock: batch.clock.clone(),
            client_id: batch.client_id,
            batch_seq: batch.batch_seq,
            version: WIRE_VERSION,
        })
    }

    /// Decode wire format to OpBatch
    pub fn decode(&self) -> Result<OpBatch, serde_json::Error> {
        let ops = WireOp::decode_batch(&self.ops)?;
        Ok(OpBatch {
            ops,
            clock: self.clock.clone(),
            client_id: self.client_id,
            batch_seq: self.batch_seq,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn make_block_insert(client_id: u64, seq: u64, parent_seq: u64) -> CrdtOp {
        CrdtOp::BlockInsert {
            id: make_op_id(client_id, seq),
            parent_op_id: make_op_id(client_id, parent_seq),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Paragraph { style: None },
        }
    }

    #[test]
    fn test_crdt_op_id() {
        let op = make_text_insert(42, 1, 0, 'a');
        assert_eq!(op.id().seq, 1);
        assert_eq!(op.client_id(), ClientId::new(42));
    }

    #[test]
    fn test_crdt_op_is_delete() {
        let insert = make_text_insert(1, 1, 0, 'a');
        assert!(!insert.is_delete());
        assert!(insert.is_insert());

        let delete = make_text_delete(1, 2, 1, 1);
        assert!(delete.is_delete());
        assert!(!delete.is_insert());
    }

    #[test]
    fn test_crdt_op_conflicts() {
        let node_id = NodeId::new();

        // Two inserts after the same parent conflict
        let insert1 = CrdtOp::TextInsert {
            id: make_op_id(1, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let insert2 = CrdtOp::TextInsert {
            id: make_op_id(2, 1),
            node_id,
            parent_op_id: OpId::root(),
            char: 'b',
        };
        assert!(insert1.conflicts_with(&insert2));

        // Inserts after different parents don't conflict
        let insert3 = CrdtOp::TextInsert {
            id: make_op_id(2, 2),
            node_id,
            parent_op_id: make_op_id(1, 1),
            char: 'c',
        };
        assert!(!insert1.conflicts_with(&insert3));
    }

    #[test]
    fn test_serialize_deserialize_text_insert() {
        let op = make_text_insert(42, 1, 0, 'H');

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: CrdtOp = serde_json::from_str(&json).unwrap();

        assert_eq!(op.id(), deserialized.id());
        if let CrdtOp::TextInsert { char: c, .. } = deserialized {
            assert_eq!(c, 'H');
        } else {
            panic!("Expected TextInsert");
        }
    }

    #[test]
    fn test_serialize_deserialize_format_set() {
        let op = CrdtOp::FormatSet {
            id: make_op_id(1, 5),
            node_id: NodeId::new(),
            start_op_id: make_op_id(1, 1),
            end_op_id: make_op_id(1, 4),
            attribute: "bold".to_string(),
            value: serde_json::json!(true),
            timestamp: Timestamp::new(1234567890, 0, ClientId::new(1)),
        };

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: CrdtOp = serde_json::from_str(&json).unwrap();

        if let CrdtOp::FormatSet {
            attribute, value, ..
        } = deserialized
        {
            assert_eq!(attribute, "bold");
            assert_eq!(value, serde_json::json!(true));
        } else {
            panic!("Expected FormatSet");
        }
    }

    #[test]
    fn test_serialize_deserialize_block_insert() {
        let op = CrdtOp::BlockInsert {
            id: make_op_id(1, 1),
            parent_op_id: OpId::root(),
            after_sibling: None,
            node_id: NodeId::new(),
            data: BlockData::Image {
                src: "https://example.com/img.png".to_string(),
                alt: Some("Test image".to_string()),
                width: Some(100),
                height: Some(200),
            },
        };

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: CrdtOp = serde_json::from_str(&json).unwrap();

        if let CrdtOp::BlockInsert { data, .. } = deserialized {
            if let BlockData::Image { src, alt, .. } = data {
                assert_eq!(src, "https://example.com/img.png");
                assert_eq!(alt, Some("Test image".to_string()));
            } else {
                panic!("Expected Image BlockData");
            }
        } else {
            panic!("Expected BlockInsert");
        }
    }

    #[test]
    fn test_op_batch_new() {
        let batch = OpBatch::new(make_client_id(1), 0);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.client_id, ClientId::new(1));
        assert_eq!(batch.batch_seq, 0);
    }

    #[test]
    fn test_op_batch_add() {
        let mut batch = OpBatch::new(make_client_id(1), 0);

        batch.add(make_text_insert(1, 1, 0, 'H'));
        batch.add(make_text_insert(1, 2, 1, 'i'));

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());

        // Clock should be updated
        assert_eq!(batch.clock.get(ClientId::new(1)), 2);
    }

    #[test]
    fn test_op_log_new() {
        let log = OpLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_op_log_add_and_get() {
        let mut log = OpLog::new();

        let op = make_text_insert(1, 1, 0, 'a');
        let op_id = op.id();

        assert!(log.add(op.clone()));
        assert!(!log.add(op)); // Duplicate should return false

        assert_eq!(log.len(), 1);
        assert!(log.contains(op_id));

        let retrieved = log.get(op_id).unwrap();
        assert_eq!(retrieved.id(), op_id);
    }

    #[test]
    fn test_op_log_ops_since() {
        let mut log = OpLog::new();

        // Add ops from two clients
        log.add(make_text_insert(1, 1, 0, 'a'));
        log.add(make_text_insert(1, 2, 1, 'b'));
        log.add(make_text_insert(2, 1, 0, 'x'));
        log.add(make_text_insert(2, 2, 1, 'y'));

        // Empty clock should return all ops
        let empty_clock = VectorClock::new();
        let all_ops = log.ops_since(&empty_clock);
        assert_eq!(all_ops.len(), 4);

        // Clock at (client 1: seq 1) should return ops after that
        let mut partial_clock = VectorClock::new();
        partial_clock.set(ClientId::new(1), 1);
        let ops = log.ops_since(&partial_clock);
        // Should return ops with seq > 1 for client 1, and all for client 2
        assert_eq!(ops.len(), 3);

        // Full clock should return no ops
        let mut full_clock = VectorClock::new();
        full_clock.set(ClientId::new(1), 2);
        full_clock.set(ClientId::new(2), 2);
        let no_ops = log.ops_since(&full_clock);
        assert_eq!(no_ops.len(), 0);
    }

    #[test]
    fn test_op_log_serialization() {
        let mut log = OpLog::new();
        log.add(make_text_insert(1, 1, 0, 'H'));
        log.add(make_text_insert(1, 2, 1, 'i'));

        // Serialize to bytes
        let bytes = log.to_bytes().unwrap();

        // Deserialize
        let restored = OpLog::from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert!(restored.contains(make_op_id(1, 1)));
        assert!(restored.contains(make_op_id(1, 2)));
    }

    #[test]
    fn test_op_log_to_json() {
        let mut log = OpLog::new();
        log.add(make_text_insert(1, 1, 0, 'X'));

        let json = log.to_json().unwrap();
        assert!(json.contains("TextInsert"));
        // Check for the seq value in the JSON (format may vary)
        assert!(json.contains("\"seq\"") || json.contains(r#""seq": 1"#) || json.contains(r#""seq":1"#));
    }

    #[test]
    fn test_wire_op_encode_decode() {
        let op = make_text_insert(42, 1, 0, 'Z');

        let wire = WireOp::encode(&op).unwrap();
        assert_eq!(wire.version, WIRE_VERSION);
        assert!(!wire.data.is_empty());

        let decoded = wire.decode().unwrap();
        assert_eq!(decoded.id(), op.id());
    }

    #[test]
    fn test_wire_batch_encode_decode() {
        let mut batch = OpBatch::new(make_client_id(1), 5);
        batch.add(make_text_insert(1, 1, 0, 'A'));
        batch.add(make_text_insert(1, 2, 1, 'B'));

        let wire = WireBatch::encode(&batch).unwrap();
        assert_eq!(wire.version, WIRE_VERSION);
        assert_eq!(wire.ops.len(), 2);
        assert_eq!(wire.client_id, ClientId::new(1));
        assert_eq!(wire.batch_seq, 5);

        let decoded = wire.decode().unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.client_id, batch.client_id);
        assert_eq!(decoded.batch_seq, batch.batch_seq);
    }

    #[test]
    fn test_op_log_ops_for_client() {
        let mut log = OpLog::new();

        log.add(make_text_insert(1, 1, 0, 'a'));
        log.add(make_text_insert(1, 2, 1, 'b'));
        log.add(make_text_insert(2, 1, 0, 'x'));

        let client1_ops = log.ops_for_client(ClientId::new(1));
        assert_eq!(client1_ops.len(), 2);

        let client2_ops = log.ops_for_client(ClientId::new(2));
        assert_eq!(client2_ops.len(), 1);

        let client3_ops = log.ops_for_client(ClientId::new(3));
        assert_eq!(client3_ops.len(), 0);
    }

    #[test]
    fn test_op_log_latest_seq() {
        let mut log = OpLog::new();

        assert_eq!(log.latest_seq(ClientId::new(1)), 0);

        log.add(make_text_insert(1, 1, 0, 'a'));
        assert_eq!(log.latest_seq(ClientId::new(1)), 1);

        log.add(make_text_insert(1, 5, 1, 'b'));
        assert_eq!(log.latest_seq(ClientId::new(1)), 5);

        // Out of order shouldn't decrease
        log.add(make_text_insert(1, 3, 2, 'c'));
        assert_eq!(log.latest_seq(ClientId::new(1)), 5);
    }

    #[test]
    fn test_block_operations() {
        let block_insert = make_block_insert(1, 1, 0);
        assert!(block_insert.is_insert());
        assert!(!block_insert.is_delete());
        assert!(block_insert.target_id().is_none());

        let block_delete = CrdtOp::BlockDelete {
            id: make_op_id(1, 2),
            target_id: make_op_id(1, 1),
        };
        assert!(block_delete.is_delete());
        assert!(!block_delete.is_insert());
        assert_eq!(block_delete.target_id(), Some(make_op_id(1, 1)));

        let block_move = CrdtOp::BlockMove {
            id: make_op_id(1, 3),
            target_id: make_op_id(1, 1),
            new_parent: OpId::root(),
            after_sibling: None,
        };
        assert!(!block_move.is_delete());
        assert!(!block_move.is_insert());
        assert_eq!(block_move.target_id(), Some(make_op_id(1, 1)));
    }

    #[test]
    fn test_block_update_serialization() {
        let op = CrdtOp::BlockUpdate {
            id: make_op_id(1, 10),
            target_id: make_op_id(1, 5),
            data: BlockData::Table {
                rows: 3,
                cols: 4,
                properties: serde_json::json!({"border": true}),
            },
            timestamp: Timestamp::new(1234567890, 0, ClientId::new(1)),
        };

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: CrdtOp = serde_json::from_str(&json).unwrap();

        if let CrdtOp::BlockUpdate {
            data, timestamp, ..
        } = deserialized
        {
            assert_eq!(timestamp.physical, 1234567890);
            if let BlockData::Table { rows, cols, .. } = data {
                assert_eq!(rows, 3);
                assert_eq!(cols, 4);
            } else {
                panic!("Expected Table BlockData");
            }
        } else {
            panic!("Expected BlockUpdate");
        }
    }
}
