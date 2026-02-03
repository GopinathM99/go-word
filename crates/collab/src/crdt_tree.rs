//! CRDT Tree for document block structure
//!
//! This module implements a CRDT-based tree structure for representing
//! document blocks (paragraphs, tables, images, etc.) in a way that
//! supports concurrent editing with automatic conflict resolution.
//!
//! # Key Concepts
//!
//! - **Children Ordering**: Children are ordered by (after_sibling, OpId) for deterministic ordering
//! - **Tombstones**: Deleted nodes stay in tree but are filtered from traversal
//! - **Move = Delete + Insert**: Moving is equivalent to deleting from old parent and inserting at new
//! - **NodeId Mapping**: Maintains bidirectional mapping between doc NodeId and CRDT OpId

use crate::op_id::{ClientId, OpId};
use doc_model::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block data for different block types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BlockData {
    /// Root document node
    Document,

    /// Paragraph block
    Paragraph {
        /// Style name or ID
        style: Option<String>,
    },

    /// Section block
    Section {
        /// Section properties
        properties: serde_json::Value,
    },

    /// Table block
    Table {
        /// Number of rows
        rows: usize,
        /// Number of columns
        cols: usize,
        /// Table properties
        properties: serde_json::Value,
    },

    /// Table row
    TableRow {
        /// Row index
        index: usize,
        /// Row properties
        properties: serde_json::Value,
    },

    /// Table cell
    TableCell {
        /// Row index
        row: usize,
        /// Column index
        col: usize,
        /// Cell properties
        properties: serde_json::Value,
    },

    /// Image block
    Image {
        /// Image source (URL or data URI)
        src: String,
        /// Alt text
        alt: Option<String>,
        /// Width in EMUs (English Metric Units)
        width: Option<i64>,
        /// Height in EMUs
        height: Option<i64>,
    },

    /// List item
    ListItem {
        /// List ID
        list_id: String,
        /// Nesting level (0-based)
        level: u8,
        /// List style
        style: Option<String>,
    },

    /// Header/Footer
    HeaderFooter {
        /// Type (header or footer)
        hf_type: HeaderFooterType,
        /// Section reference
        section_id: Option<String>,
    },

    /// Text box
    TextBox {
        /// Position and size
        bounds: serde_json::Value,
    },

    /// Shape
    Shape {
        /// Shape type
        shape_type: String,
        /// Shape properties
        properties: serde_json::Value,
    },

    /// Heading block
    Heading {
        /// Level 1-6
        level: u8,
        /// Style name or ID
        style: Option<String>,
    },

    /// Block quote
    BlockQuote,

    /// Code block
    CodeBlock {
        /// Programming language
        language: Option<String>,
    },

    /// Horizontal rule
    HorizontalRule,

    /// Generic block with custom data
    Custom {
        /// Block type identifier
        block_type: String,
        /// Custom data
        data: serde_json::Value,
    },
}

/// Header/Footer type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeaderFooterType {
    /// Default header
    DefaultHeader,
    /// First page header
    FirstPageHeader,
    /// Even page header
    EvenPageHeader,
    /// Default footer
    DefaultFooter,
    /// First page footer
    FirstPageFooter,
    /// Even page footer
    EvenPageFooter,
}

impl Default for BlockData {
    fn default() -> Self {
        BlockData::Paragraph { style: None }
    }
}

impl BlockData {
    /// Get a descriptive name for the block type
    pub fn type_name(&self) -> &'static str {
        match self {
            BlockData::Document => "Document",
            BlockData::Paragraph { .. } => "Paragraph",
            BlockData::Section { .. } => "Section",
            BlockData::Table { .. } => "Table",
            BlockData::TableRow { .. } => "TableRow",
            BlockData::TableCell { .. } => "TableCell",
            BlockData::Image { .. } => "Image",
            BlockData::ListItem { .. } => "ListItem",
            BlockData::HeaderFooter { .. } => "HeaderFooter",
            BlockData::TextBox { .. } => "TextBox",
            BlockData::Shape { .. } => "Shape",
            BlockData::Heading { .. } => "Heading",
            BlockData::BlockQuote => "BlockQuote",
            BlockData::CodeBlock { .. } => "CodeBlock",
            BlockData::HorizontalRule => "HorizontalRule",
            BlockData::Custom { .. } => "Custom",
        }
    }
}

/// A node in the CRDT tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrdtTreeNode {
    /// Unique operation ID for this node
    pub id: OpId,
    /// Corresponding document NodeId
    pub node_id: NodeId,
    /// Parent node (None for root)
    pub parent_id: Option<OpId>,
    /// Position within parent's children - OpId of sibling this node is after (None = first)
    pub position_in_parent: Option<OpId>,
    /// The block data
    pub data: BlockData,
    /// Whether this node is deleted (tombstone)
    pub tombstone: bool,
}

impl CrdtTreeNode {
    /// Create a new tree node
    pub fn new(
        id: OpId,
        node_id: NodeId,
        parent_id: Option<OpId>,
        position_in_parent: Option<OpId>,
        data: BlockData,
    ) -> Self {
        Self {
            id,
            node_id,
            parent_id,
            position_in_parent,
            data,
            tombstone: false,
        }
    }

    /// Check if this node is a tombstone
    pub fn is_tombstone(&self) -> bool {
        self.tombstone
    }
}

/// Internal structure for tracking children ordering
/// Children are stored with their (after_sibling, id) for CRDT ordering
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ChildrenList {
    /// List of (child_id, after_sibling) pairs
    children: Vec<(OpId, Option<OpId>)>,
}

impl ChildrenList {
    fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Insert a child with the given position (after_sibling)
    fn insert(&mut self, child_id: OpId, after_sibling: Option<OpId>) {
        // Remove if already present (for idempotency)
        self.children.retain(|(id, _)| *id != child_id);

        // Find insertion position
        let pos = self.find_insert_position(after_sibling, child_id);
        self.children.insert(pos, (child_id, after_sibling));
    }

    /// Find the correct position to insert a new child
    /// Uses the RGA-style ordering: items after the same sibling are ordered by OpId (descending)
    fn find_insert_position(&self, after_sibling: Option<OpId>, new_id: OpId) -> usize {
        if after_sibling.is_none() {
            // Inserting at the beginning - skip children with higher IDs that are also at the beginning
            let mut pos = 0;
            while pos < self.children.len() {
                let (child_id, child_after) = &self.children[pos];
                if child_after.is_none() && *child_id > new_id {
                    pos += 1;
                } else {
                    break;
                }
            }
            return pos;
        }

        // Find the sibling we're inserting after
        let after_pos = self
            .children
            .iter()
            .position(|(id, _)| Some(*id) == after_sibling);

        let start_pos = match after_pos {
            Some(p) => p + 1,
            None => 0, // If sibling not found, insert at beginning
        };

        // Skip children that are also after the same sibling with higher IDs
        let mut pos = start_pos;
        while pos < self.children.len() {
            let (child_id, child_after) = &self.children[pos];
            if *child_after == after_sibling && *child_id > new_id {
                pos += 1;
            } else {
                break;
            }
        }

        pos
    }

    /// Remove a child (mark as deleted in RGA style - but for tree we just remove)
    fn remove(&mut self, child_id: OpId) -> bool {
        let len_before = self.children.len();
        self.children.retain(|(id, _)| *id != child_id);
        self.children.len() < len_before
    }

    /// Get ordered list of child IDs
    fn iter_ids(&self) -> impl Iterator<Item = OpId> + '_ {
        self.children.iter().map(|(id, _)| *id)
    }
}

/// CRDT Tree for document structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrdtTree {
    /// All nodes by OpId
    nodes: HashMap<OpId, CrdtTreeNode>,
    /// Child ordering per node
    children: HashMap<OpId, ChildrenList>,
    /// Root node OpId
    root: OpId,
    /// Current sequence for operations
    seq: u64,
    /// This client's ID
    client_id: ClientId,
    /// Mapping from NodeId to OpId for lookups
    node_id_to_op_id: HashMap<NodeId, OpId>,
}

impl CrdtTree {
    /// Create a new tree with a root document node
    ///
    /// The root uses a sentinel OpId (OpId::root()) that is shared by all clients.
    /// This ensures that all clients can find the root and add children to it.
    pub fn new(client_id: ClientId) -> Self {
        // Use the sentinel root OpId so all clients share the same root
        let root_id = OpId::root();
        let root_node_id = NodeId::new();

        let root_node = CrdtTreeNode::new(root_id, root_node_id, None, None, BlockData::Document);

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        let mut children = HashMap::new();
        children.insert(root_id, ChildrenList::new());

        let mut node_id_to_op_id = HashMap::new();
        node_id_to_op_id.insert(root_node_id, root_id);

        Self {
            nodes,
            children,
            root: root_id,
            seq: 0, // Start at 0 since root uses seq 0
            client_id,
            node_id_to_op_id,
        }
    }

    /// Get the next OpId for a new operation
    fn next_op_id(&mut self) -> OpId {
        self.seq += 1;
        OpId::new(self.client_id, self.seq)
    }

    /// Insert a new block as a child of parent at the given position
    ///
    /// Returns the OpId of the inserted block.
    pub fn insert_block(
        &mut self,
        parent_op_id: OpId,
        after_sibling: Option<OpId>,
        node_id: NodeId,
        data: BlockData,
    ) -> OpId {
        let id = self.next_op_id();
        self.apply_insert_block(id, parent_op_id, after_sibling, node_id, data);
        id
    }

    /// Apply a remote block insert
    ///
    /// This is used both for local inserts (after generating an OpId) and
    /// for applying remote operations.
    pub fn apply_insert_block(
        &mut self,
        id: OpId,
        parent_op_id: OpId,
        after_sibling: Option<OpId>,
        node_id: NodeId,
        data: BlockData,
    ) {
        // Update sequence counter if this is a remote op with higher seq
        if id.seq > self.seq {
            self.seq = id.seq;
        }

        // Create the node
        let node = CrdtTreeNode::new(id, node_id, Some(parent_op_id), after_sibling, data);

        // Add to nodes map
        self.nodes.insert(id, node);

        // Add to node_id mapping
        self.node_id_to_op_id.insert(node_id, id);

        // Add to parent's children list
        let children = self
            .children
            .entry(parent_op_id)
            .or_insert_with(ChildrenList::new);
        children.insert(id, after_sibling);

        // Initialize children list for this node
        self.children.entry(id).or_insert_with(ChildrenList::new);
    }

    /// Delete a block (marks as tombstone)
    ///
    /// Returns true if the block was found and deleted.
    pub fn delete_block(&mut self, op_id: OpId) -> bool {
        self.apply_delete_block(op_id)
    }

    /// Apply a remote delete
    ///
    /// Returns true if the block was found and deleted.
    pub fn apply_delete_block(&mut self, op_id: OpId) -> bool {
        // Don't delete the root
        if op_id == self.root {
            return false;
        }

        if let Some(node) = self.nodes.get_mut(&op_id) {
            if node.tombstone {
                return false; // Already deleted
            }
            node.tombstone = true;

            // Remove from parent's children list
            if let Some(parent_id) = node.parent_id {
                if let Some(children) = self.children.get_mut(&parent_id) {
                    children.remove(op_id);
                }
            }

            true
        } else {
            false
        }
    }

    /// Move a block to a new parent/position
    ///
    /// Returns the OpId of the move operation if successful, None otherwise.
    /// Note: Moving is implemented as updating the node's parent and position.
    pub fn move_block(
        &mut self,
        op_id: OpId,
        new_parent: OpId,
        after_sibling: Option<OpId>,
    ) -> Option<OpId> {
        // Can't move root
        if op_id == self.root {
            return None;
        }

        // Check if node exists and is not tombstoned
        let node = self.nodes.get(&op_id)?;
        if node.tombstone {
            return None;
        }

        // Check that new parent exists and is not tombstoned
        let parent_node = self.nodes.get(&new_parent)?;
        if parent_node.tombstone {
            return None;
        }

        // Prevent moving a node under itself (cycle detection)
        if self.is_ancestor_of(op_id, new_parent) {
            return None;
        }

        let old_parent_id = node.parent_id?;
        let move_op_id = self.next_op_id();

        // Apply the move
        self.apply_move_block(op_id, old_parent_id, new_parent, after_sibling);

        Some(move_op_id)
    }

    /// Apply a move operation
    fn apply_move_block(
        &mut self,
        op_id: OpId,
        old_parent: OpId,
        new_parent: OpId,
        after_sibling: Option<OpId>,
    ) {
        // Remove from old parent's children
        if let Some(children) = self.children.get_mut(&old_parent) {
            children.remove(op_id);
        }

        // Add to new parent's children
        let new_children = self
            .children
            .entry(new_parent)
            .or_insert_with(ChildrenList::new);
        new_children.insert(op_id, after_sibling);

        // Update the node's parent reference
        if let Some(node) = self.nodes.get_mut(&op_id) {
            node.parent_id = Some(new_parent);
            node.position_in_parent = after_sibling;
        }
    }

    /// Check if potential_ancestor is an ancestor of node
    fn is_ancestor_of(&self, potential_ancestor: OpId, mut node: OpId) -> bool {
        while let Some(parent_id) = self.nodes.get(&node).and_then(|n| n.parent_id) {
            if parent_id == potential_ancestor {
                return true;
            }
            node = parent_id;
        }
        false
    }

    /// Get a node by OpId
    pub fn get_node(&self, op_id: OpId) -> Option<&CrdtTreeNode> {
        self.nodes.get(&op_id)
    }

    /// Get a mutable reference to a node by OpId
    pub fn get_node_mut(&mut self, op_id: OpId) -> Option<&mut CrdtTreeNode> {
        self.nodes.get_mut(&op_id)
    }

    /// Get node by document NodeId
    pub fn get_by_node_id(&self, node_id: &NodeId) -> Option<&CrdtTreeNode> {
        self.node_id_to_op_id
            .get(node_id)
            .and_then(|op_id| self.nodes.get(op_id))
    }

    /// Get children of a node (excluding tombstones)
    pub fn children(&self, op_id: OpId) -> Vec<OpId> {
        self.children
            .get(&op_id)
            .map(|list| {
                list.iter_ids()
                    .filter(|id| {
                        self.nodes
                            .get(id)
                            .map(|n| !n.tombstone)
                            .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all children including tombstones
    pub fn children_with_tombstones(&self, op_id: OpId) -> Vec<OpId> {
        self.children
            .get(&op_id)
            .map(|list| list.iter_ids().collect())
            .unwrap_or_default()
    }

    /// Get the root OpId
    pub fn root(&self) -> OpId {
        self.root
    }

    /// Get the root node
    pub fn root_node(&self) -> Option<&CrdtTreeNode> {
        self.nodes.get(&self.root)
    }

    /// Get all operations for sync
    pub fn all_ops(&self) -> Vec<TreeOperation> {
        let mut ops = Vec::new();

        // Collect all insert operations (excluding root)
        for (_, node) in &self.nodes {
            if node.id != self.root {
                ops.push(TreeOperation::InsertBlock {
                    id: node.id,
                    parent_op_id: node.parent_id.unwrap_or(self.root),
                    after_sibling: node.position_in_parent,
                    node_id: node.node_id,
                    data: node.data.clone(),
                });

                if node.tombstone {
                    ops.push(TreeOperation::DeleteBlock { id: node.id });
                }
            }
        }

        // Sort by OpId to ensure deterministic order
        ops.sort_by_key(|op| op.op_id());

        ops
    }

    /// Get the total number of nodes (including tombstones)
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of visible nodes (excluding tombstones)
    pub fn visible_nodes(&self) -> usize {
        self.nodes.values().filter(|n| !n.tombstone).count()
    }

    /// Traverse the tree in depth-first order, calling the visitor for each node
    pub fn traverse<F>(&self, mut visitor: F)
    where
        F: FnMut(&CrdtTreeNode, usize),
    {
        self.traverse_recursive(self.root, 0, &mut visitor);
    }

    fn traverse_recursive<F>(&self, op_id: OpId, depth: usize, visitor: &mut F)
    where
        F: FnMut(&CrdtTreeNode, usize),
    {
        if let Some(node) = self.nodes.get(&op_id) {
            if !node.tombstone {
                visitor(node, depth);
                for child_id in self.children(op_id) {
                    self.traverse_recursive(child_id, depth + 1, visitor);
                }
            }
        }
    }

    /// Get the path from root to a node
    pub fn path_to_node(&self, op_id: OpId) -> Vec<OpId> {
        let mut path = Vec::new();
        let mut current = Some(op_id);

        while let Some(id) = current {
            path.push(id);
            current = self.nodes.get(&id).and_then(|n| n.parent_id);
        }

        path.reverse();
        path
    }

    /// Update block data for a node
    pub fn update_block_data(&mut self, op_id: OpId, data: BlockData) -> bool {
        if let Some(node) = self.nodes.get_mut(&op_id) {
            if !node.tombstone {
                node.data = data;
                return true;
            }
        }
        false
    }

    /// Get the client ID for this tree
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Get the current sequence number
    pub fn current_seq(&self) -> u64 {
        self.seq
    }

    /// Get the OpId for a given NodeId
    pub fn get_op_id_for_node_id(&self, node_id: &NodeId) -> Option<OpId> {
        self.node_id_to_op_id.get(node_id).copied()
    }

    /// Get the parent of a node
    pub fn parent(&self, op_id: OpId) -> Option<OpId> {
        self.nodes.get(&op_id).and_then(|n| n.parent_id)
    }

    /// Get siblings of a node (children of the same parent, excluding the node itself)
    pub fn siblings(&self, op_id: OpId) -> Vec<OpId> {
        if let Some(parent_id) = self.parent(op_id) {
            self.children(parent_id)
                .into_iter()
                .filter(|&id| id != op_id)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get the depth of a node (distance from root)
    pub fn depth(&self, op_id: OpId) -> usize {
        let mut depth = 0;
        let mut current = self.nodes.get(&op_id);

        while let Some(node) = current {
            if let Some(parent_id) = node.parent_id {
                depth += 1;
                current = self.nodes.get(&parent_id);
            } else {
                break;
            }
        }

        depth
    }
}

/// Operations on the tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreeOperation {
    /// Insert a new block
    InsertBlock {
        id: OpId,
        parent_op_id: OpId,
        after_sibling: Option<OpId>,
        node_id: NodeId,
        data: BlockData,
    },
    /// Delete a block (tombstone)
    DeleteBlock { id: OpId },
    /// Move a block to a new parent/position
    MoveBlock {
        id: OpId,
        new_parent: OpId,
        after_sibling: Option<OpId>,
    },
    /// Update block data
    UpdateBlockData { id: OpId, data: BlockData },
}

impl TreeOperation {
    /// Get the OpId associated with this operation
    pub fn op_id(&self) -> OpId {
        match self {
            TreeOperation::InsertBlock { id, .. } => *id,
            TreeOperation::DeleteBlock { id } => *id,
            TreeOperation::MoveBlock { id, .. } => *id,
            TreeOperation::UpdateBlockData { id, .. } => *id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client_id(id: u64) -> ClientId {
        ClientId::new(id)
    }

    #[test]
    fn test_new_tree() {
        let tree = CrdtTree::new(make_client_id(1));
        assert_eq!(tree.visible_nodes(), 1);
        assert_eq!(tree.total_nodes(), 1);

        let root = tree.root_node().unwrap();
        assert!(matches!(root.data, BlockData::Document));
        assert!(root.parent_id.is_none());
    }

    #[test]
    fn test_insert_blocks() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        // Insert a paragraph
        let para_node_id = NodeId::new();
        let para_op_id = tree.insert_block(
            root,
            None,
            para_node_id,
            BlockData::Paragraph { style: None },
        );

        assert_eq!(tree.visible_nodes(), 2);
        assert_eq!(tree.children(root), vec![para_op_id]);

        // Insert another paragraph after the first
        let para2_node_id = NodeId::new();
        let para2_op_id = tree.insert_block(
            root,
            Some(para_op_id),
            para2_node_id,
            BlockData::Paragraph { style: None },
        );

        assert_eq!(tree.visible_nodes(), 3);
        assert_eq!(tree.children(root), vec![para_op_id, para2_op_id]);

        // Verify lookup by NodeId
        let node = tree.get_by_node_id(&para_node_id).unwrap();
        assert_eq!(node.id, para_op_id);
    }

    #[test]
    fn test_insert_at_beginning() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        // Insert first paragraph
        let para1_id = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        // Insert second paragraph at the beginning (before first)
        let para2_id = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        // para2 should come after para1 in RGA ordering (higher seq wins)
        let children = tree.children(root);
        assert_eq!(children.len(), 2);
        // The one with higher OpId comes first when both are inserted at root
        assert_eq!(children[0], para2_id);
        assert_eq!(children[1], para1_id);
    }

    #[test]
    fn test_delete_blocks() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let para_op_id = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        assert_eq!(tree.visible_nodes(), 2);
        assert_eq!(tree.children(root).len(), 1);

        // Delete the paragraph
        assert!(tree.delete_block(para_op_id));
        assert_eq!(tree.visible_nodes(), 1);
        assert_eq!(tree.children(root).len(), 0);

        // Node should still exist as tombstone
        assert_eq!(tree.total_nodes(), 2);
        let node = tree.get_node(para_op_id).unwrap();
        assert!(node.tombstone);

        // Can't delete again
        assert!(!tree.delete_block(para_op_id));

        // Can't delete root
        assert!(!tree.delete_block(root));
    }

    #[test]
    fn test_move_blocks() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        // Create a table structure
        let table_id = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Table {
                rows: 2,
                cols: 2,
                properties: serde_json::Value::Null,
            },
        );

        let row1_id = tree.insert_block(
            table_id,
            None,
            NodeId::new(),
            BlockData::TableRow {
                index: 0,
                properties: serde_json::Value::Null,
            },
        );
        let row2_id = tree.insert_block(
            table_id,
            Some(row1_id),
            NodeId::new(),
            BlockData::TableRow {
                index: 1,
                properties: serde_json::Value::Null,
            },
        );

        let cell1_id = tree.insert_block(
            row1_id,
            None,
            NodeId::new(),
            BlockData::TableCell {
                row: 0,
                col: 0,
                properties: serde_json::Value::Null,
            },
        );

        // Verify initial structure
        assert_eq!(tree.children(table_id), vec![row1_id, row2_id]);
        assert_eq!(tree.children(row1_id), vec![cell1_id]);
        assert_eq!(tree.children(row2_id).len(), 0);

        // Move cell from row1 to row2
        let move_result = tree.move_block(cell1_id, row2_id, None);
        assert!(move_result.is_some());

        // Verify new structure
        assert_eq!(tree.children(row1_id).len(), 0);
        assert_eq!(tree.children(row2_id), vec![cell1_id]);
    }

    #[test]
    fn test_move_prevents_cycles() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let parent_id = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let child_id = tree.insert_block(
            parent_id,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        // Try to move parent under child (would create cycle)
        let result = tree.move_block(parent_id, child_id, None);
        assert!(result.is_none());

        // Original structure should be preserved
        assert_eq!(tree.children(root), vec![parent_id]);
        assert_eq!(tree.children(parent_id), vec![child_id]);
    }

    #[test]
    fn test_concurrent_inserts() {
        // Simulate two clients inserting at the same position
        let mut tree1 = CrdtTree::new(make_client_id(1));
        let mut tree2 = CrdtTree::new(make_client_id(2));

        let root1 = tree1.root();
        let root2 = tree2.root();

        // Client 1 inserts a paragraph
        let node_id_1 = NodeId::new();
        let op_id_1 = tree1.insert_block(
            root1,
            None,
            node_id_1,
            BlockData::Paragraph { style: None },
        );

        // Client 2 inserts a paragraph
        let node_id_2 = NodeId::new();
        let op_id_2 = tree2.insert_block(
            root2,
            None,
            node_id_2,
            BlockData::Paragraph { style: None },
        );

        // Apply client 2's operation to tree1
        tree1.apply_insert_block(
            op_id_2,
            root1,
            None,
            node_id_2,
            BlockData::Paragraph { style: None },
        );

        // Apply client 1's operation to tree2
        tree2.apply_insert_block(
            op_id_1,
            root2,
            None,
            node_id_1,
            BlockData::Paragraph { style: None },
        );

        // Both trees should have the same children order
        let children1 = tree1.children(root1);
        let children2 = tree2.children(root2);

        assert_eq!(children1.len(), 2);
        assert_eq!(children2.len(), 2);
        assert_eq!(children1, children2);
    }

    #[test]
    fn test_tree_traversal() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let para1 = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let para2 = tree.insert_block(
            root,
            Some(para1),
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let _nested = tree.insert_block(
            para1,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let mut visited = Vec::new();
        tree.traverse(|node, depth| {
            visited.push((node.id, depth));
        });

        // Root at depth 0, two children at depth 1, one grandchild at depth 2
        assert_eq!(visited.len(), 4);
        assert_eq!(visited[0], (root, 0));
        assert_eq!(visited[1].1, 1); // para1 at depth 1
        assert_eq!(visited[2].1, 2); // nested at depth 2
        assert_eq!(visited[3], (para2, 1)); // para2 at depth 1
    }

    #[test]
    fn test_path_to_node() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let para = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let nested = tree.insert_block(
            para,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let path = tree.path_to_node(nested);
        assert_eq!(path, vec![root, para, nested]);
    }

    #[test]
    fn test_all_ops() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let para1 = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let _para2 = tree.insert_block(
            root,
            Some(para1),
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        tree.delete_block(para1);

        let ops = tree.all_ops();

        // Should have 2 inserts + 1 delete
        assert_eq!(ops.len(), 3);

        // Verify ops are sorted by OpId
        let op_ids: Vec<OpId> = ops.iter().map(|op| op.op_id()).collect();
        for i in 1..op_ids.len() {
            assert!(op_ids[i - 1] <= op_ids[i]);
        }

        // Verify we have the expected operations
        let insert_count = ops
            .iter()
            .filter(|op| matches!(op, TreeOperation::InsertBlock { .. }))
            .count();
        let delete_count = ops
            .iter()
            .filter(|op| matches!(op, TreeOperation::DeleteBlock { .. }))
            .count();

        assert_eq!(insert_count, 2);
        assert_eq!(delete_count, 1);
    }

    #[test]
    fn test_update_block_data() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let para = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        // Update to a different type
        let updated = tree.update_block_data(
            para,
            BlockData::Heading {
                level: 1,
                style: None,
            },
        );
        assert!(updated);

        let node = tree.get_node(para).unwrap();
        assert!(matches!(node.data, BlockData::Heading { level: 1, .. }));

        // Can't update tombstoned node
        tree.delete_block(para);
        let updated = tree.update_block_data(para, BlockData::Paragraph { style: None });
        assert!(!updated);
    }

    #[test]
    fn test_nested_table_structure() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        // Create a 2x2 table
        let table = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Table {
                rows: 2,
                cols: 2,
                properties: serde_json::Value::Null,
            },
        );

        let row1 = tree.insert_block(
            table,
            None,
            NodeId::new(),
            BlockData::TableRow {
                index: 0,
                properties: serde_json::Value::Null,
            },
        );
        let row2 = tree.insert_block(
            table,
            Some(row1),
            NodeId::new(),
            BlockData::TableRow {
                index: 1,
                properties: serde_json::Value::Null,
            },
        );

        let cell1_1 = tree.insert_block(
            row1,
            None,
            NodeId::new(),
            BlockData::TableCell {
                row: 0,
                col: 0,
                properties: serde_json::Value::Null,
            },
        );
        let cell1_2 = tree.insert_block(
            row1,
            Some(cell1_1),
            NodeId::new(),
            BlockData::TableCell {
                row: 0,
                col: 1,
                properties: serde_json::Value::Null,
            },
        );
        let cell2_1 = tree.insert_block(
            row2,
            None,
            NodeId::new(),
            BlockData::TableCell {
                row: 1,
                col: 0,
                properties: serde_json::Value::Null,
            },
        );
        let cell2_2 = tree.insert_block(
            row2,
            Some(cell2_1),
            NodeId::new(),
            BlockData::TableCell {
                row: 1,
                col: 1,
                properties: serde_json::Value::Null,
            },
        );

        // Verify structure
        assert_eq!(tree.children(root), vec![table]);
        assert_eq!(tree.children(table), vec![row1, row2]);
        assert_eq!(tree.children(row1), vec![cell1_1, cell1_2]);
        assert_eq!(tree.children(row2), vec![cell2_1, cell2_2]);

        // Verify total visible nodes: root + table + 2 rows + 4 cells = 8
        assert_eq!(tree.visible_nodes(), 8);
    }

    #[test]
    fn test_concurrent_delete_and_insert() {
        // Test case: one client deletes a node while another inserts a child
        let mut tree1 = CrdtTree::new(make_client_id(1));
        let mut tree2 = CrdtTree::new(make_client_id(2));

        let root1 = tree1.root();
        let root2 = tree2.root();

        // Initial paragraph in both trees
        let para_node_id = NodeId::new();
        let para_id = tree1.insert_block(
            root1,
            None,
            para_node_id,
            BlockData::Paragraph { style: None },
        );

        // Apply to tree2
        tree2.apply_insert_block(
            para_id,
            root2,
            None,
            para_node_id,
            BlockData::Paragraph { style: None },
        );

        // Client 1 deletes the paragraph
        tree1.delete_block(para_id);

        // Client 2 inserts a child in the paragraph (before receiving delete)
        let child_node_id = NodeId::new();
        let child_id = tree2.insert_block(
            para_id,
            None,
            child_node_id,
            BlockData::Paragraph { style: None },
        );

        // Apply client 2's insert to tree1
        tree1.apply_insert_block(
            child_id,
            para_id,
            None,
            child_node_id,
            BlockData::Paragraph { style: None },
        );

        // Apply client 1's delete to tree2
        tree2.apply_delete_block(para_id);

        // Both trees should have:
        // - The paragraph deleted (tombstoned)
        // - The child exists but parent is tombstoned
        let para_node1 = tree1.get_node(para_id).unwrap();
        let para_node2 = tree2.get_node(para_id).unwrap();
        assert!(para_node1.tombstone);
        assert!(para_node2.tombstone);

        // Child exists in both trees
        assert!(tree1.get_node(child_id).is_some());
        assert!(tree2.get_node(child_id).is_some());
    }

    #[test]
    fn test_depth() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        assert_eq!(tree.depth(root), 0);

        let level1 = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );
        assert_eq!(tree.depth(level1), 1);

        let level2 = tree.insert_block(
            level1,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );
        assert_eq!(tree.depth(level2), 2);

        let level3 = tree.insert_block(
            level2,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );
        assert_eq!(tree.depth(level3), 3);
    }

    #[test]
    fn test_siblings() {
        let mut tree = CrdtTree::new(make_client_id(1));
        let root = tree.root();

        let para1 = tree.insert_block(
            root,
            None,
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let para2 = tree.insert_block(
            root,
            Some(para1),
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        let para3 = tree.insert_block(
            root,
            Some(para2),
            NodeId::new(),
            BlockData::Paragraph { style: None },
        );

        // para1's siblings should be para2 and para3
        let siblings = tree.siblings(para1);
        assert_eq!(siblings.len(), 2);
        assert!(siblings.contains(&para2));
        assert!(siblings.contains(&para3));

        // Root has no siblings
        assert!(tree.siblings(root).is_empty());
    }

    #[test]
    fn test_block_data_serialization() {
        let block = BlockData::Paragraph {
            style: Some("Normal".to_string()),
        };

        let json = serde_json::to_string(&block).unwrap();
        let deserialized: BlockData = serde_json::from_str(&json).unwrap();

        assert_eq!(block, deserialized);
    }

    #[test]
    fn test_image_block() {
        let block = BlockData::Image {
            src: "https://example.com/image.png".to_string(),
            alt: Some("Example image".to_string()),
            width: Some(1000000),
            height: Some(500000),
        };

        let json = serde_json::to_string(&block).unwrap();
        let deserialized: BlockData = serde_json::from_str(&json).unwrap();

        assert_eq!(block, deserialized);
    }
}
