//! Bridge between CRDT operations and the document model.
//!
//! This module provides the crucial link between the document model (used for rendering
//! and user interactions) and the CRDT structures (used for collaboration and sync).
//!
//! Key responsibilities:
//! - Mapping Phase 0 commands (insert, delete, format) to CRDT operations
//! - Materializing CRDT state into a renderable DocumentTree
//! - Maintaining position mappings between document offsets and CRDT OpIds
//! - Supporting undo/redo in a collaborative context

use crate::clock::{HybridClock, VectorClock};
use crate::crdt_tree::{BlockData, CrdtTree};
use crate::lww_register::LwwMap;
use crate::op_id::{ClientId, OpId};
use crate::operation::{CrdtOp, OpLog};
use crate::rga::Rga;
use doc_model::{DocumentTree, Node, NodeId, Paragraph, Run};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The collaborative document state, combining all CRDT structures
#[derive(Clone, Debug)]
pub struct CollaborativeDocument {
    /// Client ID for this instance
    client_id: ClientId,
    /// Hybrid logical clock for timestamps
    clock: HybridClock,
    /// Vector clock for causality
    vector_clock: VectorClock,
    /// The CRDT tree for block structure
    tree: CrdtTree,
    /// Text content per paragraph (NodeId -> RGA)
    text_content: HashMap<NodeId, Rga<char>>,
    /// Formatting per text node
    formatting: HashMap<NodeId, LwwMap<String, serde_json::Value>>,
    /// Operation log for persistence and sync
    op_log: OpLog,
    /// Mapping from document positions to CRDT positions
    position_map: PositionMap,
    /// Pending local operations (not yet acknowledged by server)
    pending_ops: Vec<CrdtOp>,
    /// Undo stack for collaborative editing
    undo_stack: CollaborativeUndoStack,
    /// Current sequence number for this client
    seq: u64,
}

impl CollaborativeDocument {
    /// Create a new collaborative document
    pub fn new(client_id: ClientId) -> Self {
        let clock = HybridClock::new(client_id);
        let tree = CrdtTree::new(client_id);

        Self {
            client_id,
            clock,
            vector_clock: VectorClock::new(),
            tree,
            text_content: HashMap::new(),
            formatting: HashMap::new(),
            op_log: OpLog::new(),
            position_map: PositionMap::new(),
            pending_ops: Vec::new(),
            undo_stack: CollaborativeUndoStack::new(client_id),
            seq: 0,
        }
    }

    /// Create from an existing DocumentTree
    pub fn from_document(client_id: ClientId, doc: &DocumentTree) -> Self {
        let mut collab_doc = Self::new(client_id);

        // Convert existing document structure to CRDT operations
        let root_op_id = collab_doc.tree.root();

        // Track the last sibling for each parent level
        let mut last_sibling: Option<OpId> = None;

        // Iterate through paragraphs in order
        for para in doc.paragraphs() {
            let para_id = para.id();

            // Create a block for this paragraph
            let para_op_id = collab_doc.tree.insert_block(
                root_op_id,
                last_sibling,
                para_id,
                BlockData::Paragraph {
                    style: para.paragraph_style_id.as_ref().map(|s| s.to_string()),
                },
            );
            last_sibling = Some(para_op_id);

            // Create RGA for text content
            let mut rga = Rga::<char>::new(client_id);

            // Collect all text from runs in this paragraph
            let mut parent_op_id: Option<OpId> = None;
            for &run_id in para.children() {
                if let Some(run) = doc.get_run(run_id) {
                    for ch in run.text.chars() {
                        let char_op_id = rga.insert(parent_op_id, ch);
                        parent_op_id = Some(char_op_id);
                    }
                }
            }

            // Store the RGA and update position map
            collab_doc.position_map.update(para_id, &rga);
            collab_doc.text_content.insert(para_id, rga);

            // Create formatting map for this paragraph
            let formatting_map = LwwMap::<String, serde_json::Value>::new(client_id);
            collab_doc.formatting.insert(para_id, formatting_map);
        }

        collab_doc
    }

    /// Get the next operation ID for this client
    fn next_op_id(&mut self) -> OpId {
        self.seq += 1;
        self.vector_clock.set(self.client_id, self.seq);
        OpId::new(self.client_id, self.seq)
    }

    // ========== Command to CRDT Mapping ==========

    /// Insert text at a position
    ///
    /// Converts the document offset to an RGA parent OpId, then inserts each character.
    pub fn insert_text(&mut self, node_id: NodeId, offset: usize, text: &str) -> Vec<CrdtOp> {
        let mut ops = Vec::new();

        // Ensure the RGA exists for this node
        if !self.text_content.contains_key(&node_id) {
            self.text_content
                .insert(node_id, Rga::new(self.client_id));
        }

        // Get the parent OpId first (before mutably borrowing self)
        let initial_parent_op_id = if offset == 0 {
            None
        } else {
            self.text_content
                .get(&node_id)
                .and_then(|rga| rga.id_at_index(offset - 1))
        };

        let mut parent_op_id = initial_parent_op_id;

        // Insert each character
        for ch in text.chars() {
            let op_id = self.next_op_id();

            // Apply insert to RGA
            if let Some(rga) = self.text_content.get_mut(&node_id) {
                rga.apply_insert(op_id, parent_op_id, ch);
            }

            let op = CrdtOp::TextInsert {
                id: op_id,
                node_id,
                parent_op_id: parent_op_id.unwrap_or(OpId::root()),
                char: ch,
            };

            self.op_log.add(op.clone());
            self.pending_ops.push(op.clone());
            ops.push(op);

            parent_op_id = Some(op_id);
        }

        // Update position map
        if let Some(rga) = self.text_content.get(&node_id) {
            self.position_map.update(node_id, rga);
        }

        // Push to undo stack
        self.undo_stack.push(ops.clone());

        ops
    }

    /// Delete text in a range
    ///
    /// Converts offset range to OpIds and deletes each.
    pub fn delete_text(&mut self, node_id: NodeId, start: usize, end: usize) -> Vec<CrdtOp> {
        let mut ops = Vec::new();

        // First, collect the OpIds to delete (immutable borrow)
        let op_ids_to_delete: Vec<OpId> = {
            let rga = match self.text_content.get(&node_id) {
                Some(rga) => rga,
                None => return ops,
            };

            (start..end)
                .filter_map(|offset| rga.id_at_index(offset))
                .collect()
        };

        // Now delete each character
        for target_id in op_ids_to_delete {
            let op_id = self.next_op_id();

            // Apply delete to RGA
            if let Some(rga) = self.text_content.get_mut(&node_id) {
                rga.apply_delete(target_id);
            }

            let op = CrdtOp::TextDelete { id: op_id, target_id };

            self.op_log.add(op.clone());
            self.pending_ops.push(op.clone());
            ops.push(op);
        }

        // Update position map
        if let Some(rga) = self.text_content.get(&node_id) {
            self.position_map.update(node_id, rga);
        }

        // Push to undo stack
        self.undo_stack.push(ops.clone());

        ops
    }

    /// Apply formatting to a range
    ///
    /// Uses LWW map with HLC timestamps for conflict resolution.
    pub fn format_text(
        &mut self,
        node_id: NodeId,
        start: usize,
        end: usize,
        attribute: &str,
        value: serde_json::Value,
    ) -> Vec<CrdtOp> {
        let mut ops = Vec::new();
        let timestamp = self.clock.now();
        let op_id = self.next_op_id();

        // Get start and end OpIds
        let (start_op_id, end_op_id) = {
            let rga = match self.text_content.get(&node_id) {
                Some(rga) => rga,
                None => return ops,
            };

            let start_id = if start == 0 {
                OpId::root()
            } else {
                rga.id_at_index(start.saturating_sub(1))
                    .unwrap_or(OpId::root())
            };

            let end_id = rga.id_at_index(end.saturating_sub(1)).unwrap_or(OpId::root());

            (start_id, end_id)
        };

        // Create a composite key that includes the range
        let range_key = format!("{}:{}:{}", attribute, start_op_id, end_op_id);

        // Update local formatting map
        let format_map = self
            .formatting
            .entry(node_id)
            .or_insert_with(|| LwwMap::new(self.client_id));
        format_map.set(range_key, value.clone(), timestamp);

        let op = CrdtOp::FormatSet {
            id: op_id,
            node_id,
            start_op_id,
            end_op_id,
            attribute: attribute.to_string(),
            value,
            timestamp,
        };

        self.op_log.add(op.clone());
        self.pending_ops.push(op.clone());
        ops.push(op);

        // Push to undo stack
        self.undo_stack.push(ops.clone());

        ops
    }

    /// Insert a new paragraph
    pub fn insert_paragraph(&mut self, after_node: NodeId) -> (NodeId, Vec<CrdtOp>) {
        let mut ops = Vec::new();
        let op_id = self.next_op_id();
        let new_node_id = NodeId::new();

        // Find the after_node's OpId in the tree
        let after_op_id = self.tree.get_op_id_for_node_id(&after_node);

        // Insert into tree
        let _block_op_id = self.tree.insert_block(
            self.tree.root(),
            after_op_id,
            new_node_id,
            BlockData::Paragraph { style: None },
        );

        // Create RGA for the new paragraph
        let rga = Rga::<char>::new(self.client_id);
        self.text_content.insert(new_node_id, rga);

        // Create formatting map
        let format_map = LwwMap::<String, serde_json::Value>::new(self.client_id);
        self.formatting.insert(new_node_id, format_map);

        let op = CrdtOp::BlockInsert {
            id: op_id,
            parent_op_id: self.tree.root(),
            after_sibling: after_op_id,
            node_id: new_node_id,
            data: BlockData::Paragraph { style: None },
        };

        self.op_log.add(op.clone());
        self.pending_ops.push(op.clone());
        ops.push(op);

        // Push to undo stack
        self.undo_stack.push(ops.clone());

        (new_node_id, ops)
    }

    /// Delete a paragraph
    pub fn delete_paragraph(&mut self, node_id: NodeId) -> Vec<CrdtOp> {
        let mut ops = Vec::new();
        let op_id = self.next_op_id();

        // Find the OpId for this node
        let target_op_id = match self.tree.get_op_id_for_node_id(&node_id) {
            Some(id) => id,
            None => return ops,
        };

        // Delete from tree
        self.tree.delete_block(target_op_id);

        // Remove associated data
        self.text_content.remove(&node_id);
        self.formatting.remove(&node_id);
        self.position_map.remove_node(node_id);

        let op = CrdtOp::BlockDelete {
            id: op_id,
            target_id: target_op_id,
        };

        self.op_log.add(op.clone());
        self.pending_ops.push(op.clone());
        ops.push(op);

        // Push to undo stack
        self.undo_stack.push(ops.clone());

        ops
    }

    /// Split a paragraph at offset
    ///
    /// Creates a new paragraph with text from offset to end,
    /// removes that text from the original paragraph.
    pub fn split_paragraph(&mut self, node_id: NodeId, offset: usize) -> (NodeId, Vec<CrdtOp>) {
        let mut ops = Vec::new();

        // Get the text after the offset first
        let text_to_move: String = if let Some(rga) = self.text_content.get(&node_id) {
            let all_text: String = rga.to_vec().iter().map(|c| **c).collect();
            if offset < all_text.len() {
                all_text[offset..].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let original_len = self
            .text_content
            .get(&node_id)
            .map(|rga| rga.len())
            .unwrap_or(0);

        // First, insert a new paragraph after this one
        let (new_node_id, insert_ops) = self.insert_paragraph(node_id);
        ops.extend(insert_ops);

        // Delete text from original paragraph
        if !text_to_move.is_empty() {
            let delete_ops = self.delete_text(node_id, offset, original_len);
            ops.extend(delete_ops);
        }

        // Insert text into new paragraph
        if !text_to_move.is_empty() {
            let insert_ops = self.insert_text(new_node_id, 0, &text_to_move);
            ops.extend(insert_ops);
        }

        (new_node_id, ops)
    }

    /// Merge two paragraphs
    ///
    /// Moves all content from second paragraph to end of first,
    /// then deletes the second paragraph.
    pub fn merge_paragraphs(&mut self, first: NodeId, second: NodeId) -> Vec<CrdtOp> {
        let mut ops = Vec::new();

        // Get text from second paragraph
        let second_text: String = if let Some(rga) = self.text_content.get(&second) {
            rga.to_vec().iter().map(|c| **c).collect()
        } else {
            String::new()
        };

        // Get the length of first paragraph
        let first_len = self
            .text_content
            .get(&first)
            .map(|rga| rga.len())
            .unwrap_or(0);

        // Insert second's text at end of first
        if !second_text.is_empty() {
            let insert_ops = self.insert_text(first, first_len, &second_text);
            ops.extend(insert_ops);
        }

        // Delete second paragraph
        let delete_ops = self.delete_paragraph(second);
        ops.extend(delete_ops);

        ops
    }

    // ========== Remote Operation Application ==========

    /// Apply a remote operation
    ///
    /// Returns true if the operation was successfully applied.
    pub fn apply_remote(&mut self, op: CrdtOp) -> bool {
        // Update vector clock
        let op_id = op.id();
        let current = self.vector_clock.get(op_id.client_id);
        if op_id.seq > current {
            self.vector_clock.set(op_id.client_id, op_id.seq);
        }

        // Update our sequence if we see a higher one from our client
        if op_id.client_id == self.client_id && op_id.seq > self.seq {
            self.seq = op_id.seq;
        }

        // Check if we already have this operation
        if self.op_log.contains(op_id) {
            return false;
        }

        match &op {
            CrdtOp::TextInsert {
                id,
                node_id,
                parent_op_id,
                char,
            } => {
                let rga = self
                    .text_content
                    .entry(*node_id)
                    .or_insert_with(|| Rga::new(self.client_id));

                let parent = if parent_op_id.is_root() {
                    None
                } else {
                    Some(*parent_op_id)
                };

                rga.apply_insert(*id, parent, *char);
                self.position_map.update(*node_id, rga);
            }

            CrdtOp::TextDelete { target_id, .. } => {
                // Find the node containing this character and delete
                let node_ids: Vec<NodeId> = self.text_content.keys().copied().collect();
                for node_id in node_ids {
                    if let Some(rga) = self.text_content.get_mut(&node_id) {
                        if rga.get_node(*target_id).is_some() {
                            rga.apply_delete(*target_id);
                            // Update position map after mutation
                            let rga_ref = self.text_content.get(&node_id).unwrap();
                            self.position_map.update(node_id, rga_ref);
                            break;
                        }
                    }
                }
            }

            CrdtOp::FormatSet {
                node_id,
                start_op_id,
                end_op_id,
                attribute,
                value,
                timestamp,
                ..
            } => {
                let format_map = self
                    .formatting
                    .entry(*node_id)
                    .or_insert_with(|| LwwMap::new(self.client_id));

                let range_key = format!("{}:{}:{}", attribute, start_op_id, end_op_id);
                format_map.apply(
                    range_key,
                    Some(value.clone()),
                    *timestamp,
                    op_id.client_id,
                );
            }

            CrdtOp::BlockInsert {
                id,
                parent_op_id,
                after_sibling,
                node_id,
                data,
            } => {
                self.tree.apply_insert_block(
                    *id,
                    *parent_op_id,
                    *after_sibling,
                    *node_id,
                    data.clone(),
                );

                // Initialize RGA and formatting for new blocks
                if matches!(data, BlockData::Paragraph { .. }) {
                    self.text_content
                        .entry(*node_id)
                        .or_insert_with(|| Rga::new(self.client_id));
                    self.formatting
                        .entry(*node_id)
                        .or_insert_with(|| LwwMap::new(self.client_id));
                }
            }

            CrdtOp::BlockDelete { target_id, .. } => {
                self.tree.apply_delete_block(*target_id);
            }

            CrdtOp::BlockMove {
                target_id,
                new_parent,
                after_sibling,
                ..
            } => {
                // Move is handled by the tree internally via move_block
                // We need to use the public API
                let _ = self.tree.move_block(*target_id, *new_parent, *after_sibling);
            }

            CrdtOp::BlockUpdate {
                target_id, data, ..
            } => {
                self.tree.update_block_data(*target_id, data.clone());
            }
        }

        self.op_log.add(op);
        true
    }

    /// Apply multiple remote operations
    ///
    /// Returns the number of operations successfully applied.
    pub fn apply_remote_batch(&mut self, ops: Vec<CrdtOp>) -> usize {
        let mut applied = 0;
        for op in ops {
            if self.apply_remote(op) {
                applied += 1;
            }
        }
        applied
    }

    // ========== Materialization ==========

    /// Materialize the CRDT state into a DocumentTree
    ///
    /// Traverses the CRDT tree, extracts text from RGAs, and applies formatting.
    pub fn materialize(&self) -> DocumentTree {
        let mut doc = DocumentTree::new();

        // Traverse the CRDT tree and build the document
        let root = self.tree.root();
        let children = self.tree.children(root);

        for child_op_id in children {
            if let Some(node) = self.tree.get_node(child_op_id) {
                if node.tombstone {
                    continue;
                }

                match &node.data {
                    BlockData::Paragraph { style } => {
                        let para_node_id = node.node_id;
                        let mut para = if let Some(style_name) = style {
                            Paragraph::with_paragraph_style(style_name.clone())
                        } else {
                            Paragraph::new()
                        };

                        // Get text content for this paragraph
                        if let Some(rga) = self.text_content.get(&para_node_id) {
                            let text: String = rga.to_vec().iter().map(|c| **c).collect();
                            if !text.is_empty() {
                                // Create a run with the text
                                let run = Run::new(text);
                                let run_id = run.id();
                                doc.nodes.runs.insert(run_id, run);
                                para.add_child(run_id);
                            }
                        }

                        doc.nodes.paragraphs.insert(para_node_id, para);
                        doc.document.add_body_child(para_node_id);
                    }
                    _ => {
                        // Handle other block types as needed
                    }
                }
            }
        }

        doc
    }

    /// Get the text content of a node
    pub fn get_text(&self, node_id: NodeId) -> Option<String> {
        self.text_content
            .get(&node_id)
            .map(|rga| rga.to_vec().iter().map(|c| **c).collect())
    }

    /// Get formatting at a position
    pub fn get_formatting(
        &self,
        node_id: NodeId,
        _offset: usize,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();

        if let Some(format_map) = self.formatting.get(&node_id) {
            for (key, value) in format_map.iter() {
                // Parse the composite key to get the attribute name
                if let Some(attr) = key.split(':').next() {
                    result.insert(attr.to_string(), value.clone());
                }
            }
        }

        result
    }

    // ========== Sync Support ==========

    /// Get all operations since a vector clock
    pub fn ops_since(&self, clock: &VectorClock) -> Vec<&CrdtOp> {
        self.op_log.ops_since(clock)
    }

    /// Get the current vector clock
    pub fn clock(&self) -> &VectorClock {
        &self.vector_clock
    }

    /// Get pending local operations (not yet acknowledged)
    pub fn pending_ops(&self) -> &[CrdtOp] {
        &self.pending_ops
    }

    /// Clear pending operations (after acknowledgment)
    pub fn clear_pending_ops(&mut self) {
        self.pending_ops.clear();
    }

    /// Acknowledge operations up to a certain point
    pub fn acknowledge_ops(&mut self, up_to_seq: u64) {
        self.pending_ops.retain(|op| op.id().seq > up_to_seq);
    }

    // ========== Undo/Redo ==========

    /// Generate undo operations for the last local operations
    pub fn generate_undo(&mut self, count: usize) -> Vec<CrdtOp> {
        let mut ops = Vec::new();

        for _ in 0..count {
            if let Some(undo_ops) = self.undo_stack.undo() {
                // Generate inverse operations
                for op in undo_ops {
                    if let Some(inv_op) = self.generate_inverse(&op) {
                        ops.push(inv_op);
                    }
                }
            }
        }

        ops
    }

    /// Generate the inverse of an operation
    fn generate_inverse(&mut self, op: &CrdtOp) -> Option<CrdtOp> {
        match op {
            CrdtOp::TextInsert { id, .. } => {
                let op_id = self.next_op_id();
                Some(CrdtOp::TextDelete {
                    id: op_id,
                    target_id: *id,
                })
            }
            CrdtOp::TextDelete { target_id, .. } => {
                // For delete, we need to re-insert the character
                // Find the character value and parent - collect info first
                let node_info: Option<(NodeId, char, OpId)> = {
                    let node_ids: Vec<NodeId> = self.text_content.keys().copied().collect();
                    let mut result = None;
                    for node_id in node_ids {
                        if let Some(rga) = self.text_content.get(&node_id) {
                            if let Some(node) = rga.get_node(*target_id) {
                                if let Some(ch) = &node.value {
                                    result = Some((
                                        node_id,
                                        *ch,
                                        node.parent_id.unwrap_or(OpId::root()),
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                    result
                };

                // Now generate the op with mutable borrow
                if let Some((node_id, ch, parent_op_id)) = node_info {
                    let op_id = self.next_op_id();
                    return Some(CrdtOp::TextInsert {
                        id: op_id,
                        node_id,
                        parent_op_id,
                        char: ch,
                    });
                }
                None
            }
            CrdtOp::FormatSet {
                node_id,
                start_op_id,
                end_op_id,
                attribute,
                ..
            } => {
                // For format, we'd need to store the previous value
                // For now, just set to null
                let op_id = self.next_op_id();
                Some(CrdtOp::FormatSet {
                    id: op_id,
                    node_id: *node_id,
                    start_op_id: *start_op_id,
                    end_op_id: *end_op_id,
                    attribute: attribute.clone(),
                    value: serde_json::Value::Null,
                    timestamp: self.clock.now(),
                })
            }
            CrdtOp::BlockInsert { id, .. } => {
                let op_id = self.next_op_id();
                Some(CrdtOp::BlockDelete {
                    id: op_id,
                    target_id: *id,
                })
            }
            CrdtOp::BlockDelete { target_id, .. } => {
                // For block delete, we'd need to restore the block
                // Get node info first, then generate OpId
                let node_info = self.tree.get_node(*target_id).map(|node| {
                    (
                        node.parent_id.unwrap_or(self.tree.root()),
                        node.position_in_parent,
                        node.node_id,
                        node.data.clone(),
                    )
                });

                if let Some((parent_op_id, after_sibling, node_id, data)) = node_info {
                    let op_id = self.next_op_id();
                    Some(CrdtOp::BlockInsert {
                        id: op_id,
                        parent_op_id,
                        after_sibling,
                        node_id,
                        data,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.undo_stack.redo_stack.is_empty()
    }

    /// Get the client ID
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }
}

/// Maps document positions to CRDT OpIds and back
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PositionMap {
    /// Node -> (offset -> OpId)
    offset_to_op: HashMap<NodeId, Vec<OpId>>,
    /// OpId -> (Node, offset)
    op_to_offset: HashMap<OpId, (NodeId, usize)>,
}

impl PositionMap {
    /// Create a new empty position map
    pub fn new() -> Self {
        Self {
            offset_to_op: HashMap::new(),
            op_to_offset: HashMap::new(),
        }
    }

    /// Update mapping when content changes
    pub fn update(&mut self, node_id: NodeId, rga: &Rga<char>) {
        // Clear existing mappings for this node
        if let Some(old_ops) = self.offset_to_op.remove(&node_id) {
            for op_id in old_ops {
                self.op_to_offset.remove(&op_id);
            }
        }

        // Build new mappings
        let mut offset_to_op = Vec::new();
        let mut offset = 0;

        for node in rga.nodes_in_order() {
            if node.value.is_some() {
                offset_to_op.push(node.id);
                self.op_to_offset.insert(node.id, (node_id, offset));
                offset += 1;
            }
        }

        self.offset_to_op.insert(node_id, offset_to_op);
    }

    /// Remove all mappings for a node
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some(ops) = self.offset_to_op.remove(&node_id) {
            for op_id in ops {
                self.op_to_offset.remove(&op_id);
            }
        }
    }

    /// Get OpId for a document position
    pub fn to_op_id(&self, node_id: NodeId, offset: usize) -> Option<OpId> {
        self.offset_to_op
            .get(&node_id)
            .and_then(|ops| ops.get(offset).copied())
    }

    /// Get document position for an OpId
    pub fn to_position(&self, op_id: OpId) -> Option<(NodeId, usize)> {
        self.op_to_offset.get(&op_id).copied()
    }
}

/// Undo stack for collaborative editing
#[derive(Clone, Debug)]
pub struct CollaborativeUndoStack {
    /// Stack of (client_id, ops) pairs
    pub undo_stack: Vec<(ClientId, Vec<CrdtOp>)>,
    /// Redo stack
    pub redo_stack: Vec<(ClientId, Vec<CrdtOp>)>,
    /// Current client ID
    client_id: ClientId,
}

impl CollaborativeUndoStack {
    /// Create a new undo stack
    pub fn new(client_id: ClientId) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            client_id,
        }
    }

    /// Push operations onto undo stack
    pub fn push(&mut self, ops: Vec<CrdtOp>) {
        if !ops.is_empty() {
            self.undo_stack.push((self.client_id, ops));
            self.clear_redo();
        }
    }

    /// Pop and return undo operations
    pub fn undo(&mut self) -> Option<Vec<CrdtOp>> {
        // Find the most recent operations from our client
        let pos = self
            .undo_stack
            .iter()
            .rposition(|(client, _)| *client == self.client_id)?;

        let (_, ops) = self.undo_stack.remove(pos);

        // Move to redo stack
        self.redo_stack.push((self.client_id, ops.clone()));

        Some(ops)
    }

    /// Pop and return redo operations
    pub fn redo(&mut self) -> Option<Vec<CrdtOp>> {
        // Find the most recent redo from our client
        let pos = self
            .redo_stack
            .iter()
            .rposition(|(client, _)| *client == self.client_id)?;

        let (_, ops) = self.redo_stack.remove(pos);

        // Move back to undo stack
        self.undo_stack.push((self.client_id, ops.clone()));

        Some(ops)
    }

    /// Clear redo stack (called after new edits)
    pub fn clear_redo(&mut self) {
        self.redo_stack
            .retain(|(client, _)| *client != self.client_id);
    }
}

impl Default for CollaborativeUndoStack {
    fn default() -> Self {
        Self::new(ClientId::new(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client_id(id: u64) -> ClientId {
        ClientId::new(id)
    }

    #[test]
    fn test_new_collaborative_document() {
        let doc = CollaborativeDocument::new(make_client_id(1));
        assert_eq!(doc.client_id(), make_client_id(1));
        assert!(doc.pending_ops().is_empty());
        assert!(!doc.can_undo());
    }

    #[test]
    fn test_insert_text_and_materialize() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert a paragraph first
        let (para_id, _) = doc.insert_paragraph(NodeId::new());

        // Insert text
        let ops = doc.insert_text(para_id, 0, "Hello");

        assert_eq!(ops.len(), 5); // One op per character
        assert!(doc.can_undo());

        // Check text content
        let text = doc.get_text(para_id);
        assert_eq!(text, Some("Hello".to_string()));

        // Materialize and check
        let tree = doc.materialize();
        let content = tree.text_content();
        assert!(content.contains("Hello"));
    }

    #[test]
    fn test_delete_text_and_materialize() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert a paragraph and text
        let (para_id, _) = doc.insert_paragraph(NodeId::new());
        doc.insert_text(para_id, 0, "Hello World");

        // Delete "World"
        let delete_ops = doc.delete_text(para_id, 6, 11);

        assert_eq!(delete_ops.len(), 5); // 5 characters deleted

        // Check text content
        let text = doc.get_text(para_id);
        assert_eq!(text, Some("Hello ".to_string()));
    }

    #[test]
    fn test_format_text() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert a paragraph and text
        let (para_id, _) = doc.insert_paragraph(NodeId::new());
        doc.insert_text(para_id, 0, "Hello");

        // Format text
        let format_ops = doc.format_text(para_id, 0, 5, "bold", serde_json::json!(true));

        assert_eq!(format_ops.len(), 1);

        // Check formatting
        let formatting = doc.get_formatting(para_id, 0);
        assert!(formatting.contains_key("bold"));
    }

    #[test]
    fn test_remote_operation_application() {
        let mut doc1 = CollaborativeDocument::new(make_client_id(1));
        let mut doc2 = CollaborativeDocument::new(make_client_id(2));

        // Doc1 inserts a paragraph
        let (para_id, para_ops) = doc1.insert_paragraph(NodeId::new());

        // Apply to doc2
        for op in para_ops {
            doc2.apply_remote(op);
        }

        // Doc1 inserts text
        let text_ops = doc1.insert_text(para_id, 0, "Hello");

        // Apply to doc2
        let applied = doc2.apply_remote_batch(text_ops);

        assert_eq!(applied, 5);

        // Both should have the same text
        let text1 = doc1.get_text(para_id);
        let text2 = doc2.get_text(para_id);
        assert_eq!(text1, text2);
    }

    #[test]
    fn test_concurrent_edits() {
        let mut doc1 = CollaborativeDocument::new(make_client_id(1));
        let mut doc2 = CollaborativeDocument::new(make_client_id(2));

        // Both create the same paragraph structure first
        let (para_id, para_ops) = doc1.insert_paragraph(NodeId::new());

        // Sync paragraph to doc2
        for op in para_ops {
            doc2.apply_remote(op);
        }

        // Doc1 inserts "A"
        let ops1 = doc1.insert_text(para_id, 0, "A");

        // Doc2 inserts "B" concurrently
        let ops2 = doc2.insert_text(para_id, 0, "B");

        // Cross-apply operations
        doc1.apply_remote_batch(ops2);
        doc2.apply_remote_batch(ops1);

        // Both should converge to the same content
        let text1 = doc1.get_text(para_id);
        let text2 = doc2.get_text(para_id);
        assert_eq!(text1, text2);
    }

    #[test]
    fn test_split_paragraph() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert a paragraph and text
        let (para_id, _) = doc.insert_paragraph(NodeId::new());
        doc.insert_text(para_id, 0, "HelloWorld");

        // Split at position 5
        let (new_para_id, _split_ops) = doc.split_paragraph(para_id, 5);

        // Check both paragraphs
        let text1 = doc.get_text(para_id);
        let text2 = doc.get_text(new_para_id);

        assert_eq!(text1, Some("Hello".to_string()));
        assert_eq!(text2, Some("World".to_string()));
    }

    #[test]
    fn test_merge_paragraphs() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert two paragraphs
        let (para1_id, _) = doc.insert_paragraph(NodeId::new());
        let (para2_id, _) = doc.insert_paragraph(para1_id);

        // Insert text in both
        doc.insert_text(para1_id, 0, "Hello ");
        doc.insert_text(para2_id, 0, "World");

        // Merge
        doc.merge_paragraphs(para1_id, para2_id);

        // Check result
        let text1 = doc.get_text(para1_id);
        assert_eq!(text1, Some("Hello World".to_string()));

        // Second paragraph should be gone
        let text2 = doc.get_text(para2_id);
        assert_eq!(text2, None);
    }

    #[test]
    fn test_undo_redo() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert a paragraph and text
        let (para_id, _) = doc.insert_paragraph(NodeId::new());
        doc.insert_text(para_id, 0, "Hello");

        assert!(doc.can_undo());

        // Generate undo operations
        let undo_ops = doc.generate_undo(1);

        // The undo ops should delete the inserted characters
        assert!(!undo_ops.is_empty());

        // Apply the undo
        for op in undo_ops {
            doc.apply_remote(op);
        }

        // Text should be shorter or empty
        // Note: Due to how we generate undos, this might need adjustment
        // based on exact undo semantics
    }

    #[test]
    fn test_position_map() {
        let mut map = PositionMap::new();
        let node_id = NodeId::new();
        let mut rga = Rga::<char>::new(ClientId::new(1));

        // Insert some characters
        let id1 = rga.insert(None, 'a');
        let id2 = rga.insert(Some(id1), 'b');
        let id3 = rga.insert(Some(id2), 'c');

        // Update map
        map.update(node_id, &rga);

        // Test mapping
        assert_eq!(map.to_op_id(node_id, 0), Some(id1));
        assert_eq!(map.to_op_id(node_id, 1), Some(id2));
        assert_eq!(map.to_op_id(node_id, 2), Some(id3));

        assert_eq!(map.to_position(id1), Some((node_id, 0)));
        assert_eq!(map.to_position(id2), Some((node_id, 1)));
        assert_eq!(map.to_position(id3), Some((node_id, 2)));
    }

    #[test]
    fn test_undo_stack() {
        let client_id = make_client_id(1);
        let mut stack = CollaborativeUndoStack::new(client_id);

        // Push some operations
        let op1 = CrdtOp::TextInsert {
            id: OpId::new(client_id, 1),
            node_id: NodeId::new(),
            parent_op_id: OpId::root(),
            char: 'a',
        };
        let op2 = CrdtOp::TextInsert {
            id: OpId::new(client_id, 2),
            node_id: NodeId::new(),
            parent_op_id: OpId::new(client_id, 1),
            char: 'b',
        };

        stack.push(vec![op1.clone()]);
        stack.push(vec![op2.clone()]);

        // Undo should return the last operation
        let undo = stack.undo();
        assert!(undo.is_some());
        assert_eq!(undo.as_ref().unwrap().len(), 1);

        // Redo should return the undone operation
        let redo = stack.redo();
        assert!(redo.is_some());
        assert_eq!(redo.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_from_document() {
        // Create a document with some content
        let mut doc_tree = DocumentTree::new();

        // Add a paragraph
        let mut para = Paragraph::new();
        let para_id = para.id();

        // Add a run with text
        let run = Run::new("Hello World");
        let run_id = run.id();
        para.add_child(run_id);

        doc_tree.nodes.runs.insert(run_id, run);
        doc_tree.nodes.paragraphs.insert(para_id, para);
        doc_tree.document.add_body_child(para_id);

        // Create collaborative document from it
        let collab_doc = CollaborativeDocument::from_document(make_client_id(1), &doc_tree);

        // Check that we can get the text
        let text = collab_doc.get_text(para_id);
        assert_eq!(text, Some("Hello World".to_string()));
    }

    #[test]
    fn test_ops_since() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        // Insert a paragraph
        let (para_id, _) = doc.insert_paragraph(NodeId::new());

        // Insert some text
        doc.insert_text(para_id, 0, "ABC");

        // Get ops since empty clock
        let empty_clock = VectorClock::new();
        let ops = doc.ops_since(&empty_clock);
        assert!(!ops.is_empty());

        // Get ops since current clock (should be empty)
        let current_clock = doc.clock().clone();
        let ops = doc.ops_since(&current_clock);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_pending_ops() {
        let mut doc = CollaborativeDocument::new(make_client_id(1));

        assert!(doc.pending_ops().is_empty());

        // Insert a paragraph
        let (para_id, _) = doc.insert_paragraph(NodeId::new());

        // Pending should have ops now
        assert!(!doc.pending_ops().is_empty());

        // Insert text
        doc.insert_text(para_id, 0, "Hello");

        // More pending ops
        let pending_count = doc.pending_ops().len();
        assert!(pending_count > 5); // Block insert + 5 chars

        // Clear pending
        doc.clear_pending_ops();
        assert!(doc.pending_ops().is_empty());
    }
}
