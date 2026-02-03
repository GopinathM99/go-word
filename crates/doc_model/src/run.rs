//! Text run node - a contiguous span of text with consistent formatting

use crate::{CharacterProperties, Node, NodeId, NodeType, StyleId};
use serde::{Deserialize, Serialize};

/// Style reference for a run (kept for backwards compatibility)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunStyle {
    /// Style ID reference (e.g., "Normal", "Heading1")
    pub style_id: Option<String>,
    /// Bold override
    pub bold: Option<bool>,
    /// Italic override
    pub italic: Option<bool>,
    /// Underline override
    pub underline: Option<bool>,
    /// Font family override
    pub font_family: Option<String>,
    /// Font size in points override
    pub font_size: Option<f32>,
    /// Text color override (as CSS color string)
    pub color: Option<String>,
}

impl RunStyle {
    /// Convert to CharacterProperties for style cascade
    pub fn to_character_properties(&self) -> CharacterProperties {
        CharacterProperties {
            font_family: self.font_family.clone(),
            font_size: self.font_size,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            color: self.color.clone(),
            ..Default::default()
        }
    }

    /// Get the style ID reference
    pub fn style_id_ref(&self) -> Option<StyleId> {
        self.style_id.as_ref().map(|s| StyleId::new(s.clone()))
    }
}

/// A text run - contiguous text with consistent formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    id: NodeId,
    parent: Option<NodeId>,
    /// The text content of this run
    pub text: String,
    /// Style applied to this run (legacy field for backwards compatibility)
    pub style: RunStyle,
    /// Character style ID reference (new style system)
    #[serde(default)]
    pub character_style_id: Option<StyleId>,
    /// Direct formatting overrides (new style system)
    #[serde(default)]
    pub direct_formatting: CharacterProperties,
}

impl Run {
    /// Create a new run with text content
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            text: text.into(),
            style: RunStyle::default(),
            character_style_id: None,
            direct_formatting: CharacterProperties::default(),
        }
    }

    /// Create a new run with text and style (legacy)
    pub fn with_style(text: impl Into<String>, style: RunStyle) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            text: text.into(),
            style,
            character_style_id: None,
            direct_formatting: CharacterProperties::default(),
        }
    }

    /// Create a new run with text and character style ID
    pub fn with_character_style(text: impl Into<String>, style_id: impl Into<StyleId>) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            text: text.into(),
            style: RunStyle::default(),
            character_style_id: Some(style_id.into()),
            direct_formatting: CharacterProperties::default(),
        }
    }

    /// Create a new run with direct formatting
    pub fn with_direct_formatting(text: impl Into<String>, formatting: CharacterProperties) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            text: text.into(),
            style: RunStyle::default(),
            character_style_id: None,
            direct_formatting: formatting,
        }
    }

    /// Apply direct formatting to this run
    pub fn apply_direct_formatting(&mut self, formatting: CharacterProperties) {
        self.direct_formatting = self.direct_formatting.merge(&formatting);
    }

    /// Clear all direct formatting
    pub fn clear_direct_formatting(&mut self) {
        self.direct_formatting = CharacterProperties::default();
    }

    /// Set the character style ID
    pub fn set_character_style(&mut self, style_id: Option<StyleId>) {
        self.character_style_id = style_id;
    }

    /// Check if this run has any direct formatting
    pub fn has_direct_formatting(&self) -> bool {
        !self.direct_formatting.is_empty()
    }

    /// Get the length of the text in this run (in UTF-8 bytes)
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Check if this run is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get the number of grapheme clusters in this run
    pub fn grapheme_count(&self) -> usize {
        use unicode_segmentation::UnicodeSegmentation;
        self.text.graphemes(true).count()
    }
}

impl Node for Run {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Run
    }

    fn children(&self) -> &[NodeId] {
        // Runs have no children
        &[]
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        false
    }

    fn text_content(&self) -> Option<&str> {
        Some(&self.text)
    }
}
