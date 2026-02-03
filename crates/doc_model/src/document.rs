//! Document root node and document-level operations

use crate::{Node, NodeId, NodeType, Paragraph, Run};
use crate::protection::DocumentProtection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Document metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Page setup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSetup {
    /// Page width in points
    pub width: f32,
    /// Page height in points
    pub height: f32,
    /// Top margin in points
    pub margin_top: f32,
    /// Bottom margin in points
    pub margin_bottom: f32,
    /// Left margin in points
    pub margin_left: f32,
    /// Right margin in points
    pub margin_right: f32,
}

impl Default for PageSetup {
    fn default() -> Self {
        // Default to US Letter size with 1-inch margins
        Self {
            width: 612.0,   // 8.5 inches
            height: 792.0,  // 11 inches
            margin_top: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
        }
    }
}

/// The root document node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    id: NodeId,
    /// IDs of top-level body children (paragraphs, tables, etc.)
    body_children: Vec<NodeId>,
    /// Metadata
    pub metadata: DocumentMetadata,
    /// Page setup
    pub page_setup: PageSetup,
    /// Version counter for tracking changes
    version: u64,
    /// Document protection settings
    pub protection: DocumentProtection,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            body_children: Vec::new(),
            metadata: DocumentMetadata::default(),
            page_setup: PageSetup::default(),
            version: 0,
            protection: DocumentProtection::default(),
        }
    }

    /// Get the document version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Increment version after a change
    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    /// Add a child to the body
    pub fn add_body_child(&mut self, child_id: NodeId) {
        self.body_children.push(child_id);
        self.increment_version();
    }

    /// Insert a child at a specific index
    pub fn insert_body_child(&mut self, index: usize, child_id: NodeId) {
        self.body_children.insert(index, child_id);
        self.increment_version();
    }

    /// Remove a child by ID
    pub fn remove_body_child(&mut self, child_id: NodeId) -> bool {
        if let Some(pos) = self.body_children.iter().position(|&id| id == child_id) {
            self.body_children.remove(pos);
            self.increment_version();
            true
        } else {
            false
        }
    }

    /// Get the content area width (page width minus margins)
    pub fn content_width(&self) -> f32 {
        self.page_setup.width - self.page_setup.margin_left - self.page_setup.margin_right
    }

    /// Get the content area height (page height minus margins)
    pub fn content_height(&self) -> f32 {
        self.page_setup.height - self.page_setup.margin_top - self.page_setup.margin_bottom
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Document {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Document
    }

    fn children(&self) -> &[NodeId] {
        &self.body_children
    }

    fn parent(&self) -> Option<NodeId> {
        None // Document is the root
    }

    fn set_parent(&mut self, _parent: Option<NodeId>) {
        // Document cannot have a parent
    }

    fn can_have_children(&self) -> bool {
        true
    }
}
