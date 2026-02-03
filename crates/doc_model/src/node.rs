//! Core node trait and types

use crate::{NodeId, Result};
use serde::{Deserialize, Serialize};

/// Enumeration of all node types in the document tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Document,
    Section,
    Paragraph,
    Run,
    Table,
    TableRow,
    TableCell,
    Image,
    Shape,
    TextBox,
    Hyperlink,
    Field,
    Bookmark,
    ContentControl,
}

/// Common interface for all document nodes
pub trait Node: std::fmt::Debug {
    /// Get the unique ID of this node
    fn id(&self) -> NodeId;

    /// Get the type of this node
    fn node_type(&self) -> NodeType;

    /// Get the IDs of child nodes
    fn children(&self) -> &[NodeId];

    /// Get the ID of the parent node (None for root)
    fn parent(&self) -> Option<NodeId>;

    /// Set the parent node ID
    fn set_parent(&mut self, parent: Option<NodeId>);

    /// Check if this node can have children
    fn can_have_children(&self) -> bool;

    /// Get the text content of this node (if any)
    fn text_content(&self) -> Option<&str> {
        None
    }
}

/// A boxed node trait object
pub type BoxedNode = Box<dyn Node + Send + Sync>;
