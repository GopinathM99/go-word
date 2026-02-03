//! Selection model - cursor position and text selection

use crate::NodeId;
use serde::{Deserialize, Serialize};

/// A position in the document tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// The node containing this position
    pub node_id: NodeId,
    /// Character offset within the node (in grapheme clusters)
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(node_id: NodeId, offset: usize) -> Self {
        Self { node_id, offset }
    }

    /// Create a position at the start of a node
    pub fn start_of(node_id: NodeId) -> Self {
        Self { node_id, offset: 0 }
    }
}

/// A selection in the document
///
/// A selection has an anchor (where the selection started) and a focus
/// (where the selection ends / where the caret is). When anchor == focus,
/// the selection is collapsed (just a caret).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Where the selection started
    pub anchor: Position,
    /// Where the selection ends (caret position)
    pub focus: Position,
}

impl Selection {
    /// Create a new selection
    pub fn new(anchor: Position, focus: Position) -> Self {
        Self { anchor, focus }
    }

    /// Create a collapsed selection (caret only)
    pub fn collapsed(position: Position) -> Self {
        Self {
            anchor: position,
            focus: position,
        }
    }

    /// Create a selection at the start of a node
    pub fn at_start_of(node_id: NodeId) -> Self {
        let pos = Position::start_of(node_id);
        Self::collapsed(pos)
    }

    /// Check if this selection is collapsed (just a caret)
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    /// Check if the selection goes forward (anchor before focus)
    pub fn is_forward(&self) -> bool {
        // This is a simplified check - full implementation needs document structure
        self.anchor.node_id == self.focus.node_id && self.anchor.offset <= self.focus.offset
    }

    /// Get the start position of the selection (regardless of direction)
    pub fn start(&self) -> Position {
        if self.is_forward() {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Get the end position of the selection (regardless of direction)
    pub fn end(&self) -> Position {
        if self.is_forward() {
            self.focus
        } else {
            self.anchor
        }
    }

    /// Move the focus, extending the selection
    pub fn extend_to(&self, focus: Position) -> Self {
        Self {
            anchor: self.anchor,
            focus,
        }
    }

    /// Collapse the selection to the focus position
    pub fn collapse_to_focus(&self) -> Self {
        Self::collapsed(self.focus)
    }

    /// Collapse the selection to the anchor position
    pub fn collapse_to_anchor(&self) -> Self {
        Self::collapsed(self.anchor)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            anchor: Position::new(NodeId::new(), 0),
            focus: Position::new(NodeId::new(), 0),
        }
    }
}
