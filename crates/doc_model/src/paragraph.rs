//! Paragraph node - a block of content containing runs

use crate::{Node, NodeId, NodeType, ParagraphProperties, StyleId};
use serde::{Deserialize, Serialize};

/// Text alignment options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Line spacing configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LineSpacing {
    /// Multiple of line height (1.0 = single, 1.5 = 1.5 lines, 2.0 = double)
    Multiple(f32),
    /// Exact spacing in points
    Exact(f32),
    /// At least this many points
    AtLeast(f32),
}

impl Default for LineSpacing {
    fn default() -> Self {
        LineSpacing::Multiple(1.0)
    }
}

/// Paragraph style properties (legacy, for backwards compatibility)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParagraphStyle {
    /// Style ID reference
    pub style_id: Option<String>,
    /// Text alignment
    pub alignment: Option<Alignment>,
    /// Left indent in points
    pub indent_left: Option<f32>,
    /// Right indent in points
    pub indent_right: Option<f32>,
    /// First line indent in points (negative for hanging)
    pub indent_first_line: Option<f32>,
    /// Space before paragraph in points
    pub space_before: Option<f32>,
    /// Space after paragraph in points
    pub space_after: Option<f32>,
    /// Line spacing
    pub line_spacing: Option<LineSpacing>,
    /// Keep with next paragraph
    pub keep_with_next: Option<bool>,
    /// Keep lines together
    pub keep_together: Option<bool>,
    /// Page break before
    pub page_break_before: Option<bool>,
}

impl ParagraphStyle {
    /// Convert to ParagraphProperties for style cascade
    pub fn to_paragraph_properties(&self) -> ParagraphProperties {
        ParagraphProperties {
            alignment: self.alignment,
            indent_left: self.indent_left,
            indent_right: self.indent_right,
            indent_first_line: self.indent_first_line,
            space_before: self.space_before,
            space_after: self.space_after,
            line_spacing: self.line_spacing,
            keep_with_next: self.keep_with_next,
            keep_together: self.keep_together,
            page_break_before: self.page_break_before,
            ..Default::default()
        }
    }

    /// Get the style ID reference
    pub fn style_id_ref(&self) -> Option<StyleId> {
        self.style_id.as_ref().map(|s| StyleId::new(s.clone()))
    }
}

/// A paragraph containing text runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    id: NodeId,
    parent: Option<NodeId>,
    /// IDs of child runs
    children: Vec<NodeId>,
    /// Paragraph style (legacy, for backwards compatibility)
    pub style: ParagraphStyle,
    /// Paragraph style ID reference (new style system)
    #[serde(default)]
    pub paragraph_style_id: Option<StyleId>,
    /// Direct formatting overrides (new style system)
    #[serde(default)]
    pub direct_formatting: ParagraphProperties,
}

impl Paragraph {
    /// Create a new empty paragraph
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            style: ParagraphStyle::default(),
            paragraph_style_id: Some(StyleId::new("Normal")),
            direct_formatting: ParagraphProperties::default(),
        }
    }

    /// Create a paragraph with specific legacy style
    pub fn with_style(style: ParagraphStyle) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            style,
            paragraph_style_id: Some(StyleId::new("Normal")),
            direct_formatting: ParagraphProperties::default(),
        }
    }

    /// Create a paragraph with a paragraph style ID
    pub fn with_paragraph_style(style_id: impl Into<StyleId>) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            style: ParagraphStyle::default(),
            paragraph_style_id: Some(style_id.into()),
            direct_formatting: ParagraphProperties::default(),
        }
    }

    /// Create a paragraph with direct formatting
    pub fn with_direct_formatting(formatting: ParagraphProperties) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            style: ParagraphStyle::default(),
            paragraph_style_id: Some(StyleId::new("Normal")),
            direct_formatting: formatting,
        }
    }

    /// Set the paragraph style ID
    pub fn set_paragraph_style(&mut self, style_id: Option<StyleId>) {
        self.paragraph_style_id = style_id;
    }

    /// Apply direct formatting to this paragraph
    pub fn apply_direct_formatting(&mut self, formatting: ParagraphProperties) {
        self.direct_formatting = self.direct_formatting.merge(&formatting);
    }

    /// Clear all direct formatting
    pub fn clear_direct_formatting(&mut self) {
        self.direct_formatting = ParagraphProperties::default();
    }

    /// Check if this paragraph has any direct formatting
    pub fn has_direct_formatting(&self) -> bool {
        !self.direct_formatting.is_empty()
    }

    /// Add a child run ID
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Insert a child at a specific index
    pub fn insert_child(&mut self, index: usize, child_id: NodeId) {
        self.children.insert(index, child_id);
    }

    /// Remove a child by ID
    pub fn remove_child(&mut self, child_id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Paragraph {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Paragraph
    }

    fn children(&self) -> &[NodeId] {
        &self.children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        true
    }
}
