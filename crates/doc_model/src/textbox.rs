//! Text Box node types for floating text containers
//!
//! This module provides the data structures for representing text boxes in the document.
//! Text boxes are floating containers that hold block-level content (paragraphs) and can
//! be positioned independently of the main text flow with various text wrapping options.

use crate::{Dimension, ImagePosition, Node, NodeId, NodeType, WrapType, AnchorPosition, HorizontalAnchor, VerticalAnchor};
use serde::{Deserialize, Serialize};

/// Vertical alignment for content within a text box
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextBoxVerticalAlign {
    /// Align content to the top
    #[default]
    Top,
    /// Center content vertically
    Center,
    /// Align content to the bottom
    Bottom,
}

/// Border line style for text boxes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BorderLineStyle {
    /// No border
    None,
    /// Solid line
    #[default]
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
    /// Double line
    Double,
}

/// Border style for a single edge
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BorderEdge {
    /// Border width in points
    pub width: f32,
    /// Border color (hex string, e.g., "#000000")
    pub color: String,
    /// Line style
    pub style: BorderLineStyle,
}

impl Default for BorderEdge {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: "#000000".to_string(),
            style: BorderLineStyle::Solid,
        }
    }
}

impl BorderEdge {
    /// Create a new border edge
    pub fn new(width: f32, color: impl Into<String>, style: BorderLineStyle) -> Self {
        Self {
            width,
            color: color.into(),
            style,
        }
    }

    /// Create a simple solid border
    pub fn solid(width: f32, color: impl Into<String>) -> Self {
        Self::new(width, color, BorderLineStyle::Solid)
    }

    /// Check if the border is visible
    pub fn is_visible(&self) -> bool {
        self.width > 0.0 && self.style != BorderLineStyle::None
    }
}

/// Complete border style for all four edges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBoxBorderStyle {
    /// Top border
    pub top: BorderEdge,
    /// Right border
    pub right: BorderEdge,
    /// Bottom border
    pub bottom: BorderEdge,
    /// Left border
    pub left: BorderEdge,
}

impl Default for TextBoxBorderStyle {
    fn default() -> Self {
        let edge = BorderEdge::default();
        Self {
            top: edge.clone(),
            right: edge.clone(),
            bottom: edge.clone(),
            left: edge,
        }
    }
}

impl TextBoxBorderStyle {
    /// Create a uniform border on all sides
    pub fn uniform(width: f32, color: impl Into<String>, style: BorderLineStyle) -> Self {
        let color = color.into();
        let edge = BorderEdge::new(width, &color, style);
        Self {
            top: edge.clone(),
            right: edge.clone(),
            bottom: edge.clone(),
            left: edge,
        }
    }

    /// Create a border with no lines (invisible)
    pub fn none() -> Self {
        let edge = BorderEdge::new(0.0, "", BorderLineStyle::None);
        Self {
            top: edge.clone(),
            right: edge.clone(),
            bottom: edge.clone(),
            left: edge,
        }
    }

    /// Check if any border edge is visible
    pub fn has_visible_border(&self) -> bool {
        self.top.is_visible() || self.right.is_visible() ||
        self.bottom.is_visible() || self.left.is_visible()
    }
}

/// Fill style for text boxes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FillStyle {
    /// No fill (transparent)
    None,
    /// Solid color fill (hex string)
    Solid(String),
    /// Gradient fill
    Gradient {
        /// Gradient stops (color, position 0.0-1.0)
        colors: Vec<(String, f32)>,
        /// Gradient angle in degrees
        angle: f32,
    },
}

impl Default for FillStyle {
    fn default() -> Self {
        Self::Solid("#FFFFFF".to_string())
    }
}

impl FillStyle {
    /// Create a solid fill
    pub fn solid(color: impl Into<String>) -> Self {
        Self::Solid(color.into())
    }

    /// Check if the fill is transparent
    pub fn is_none(&self) -> bool {
        matches!(self, FillStyle::None)
    }
}

/// Internal margins (padding) for text box content
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    /// Top margin in points
    pub top: f32,
    /// Right margin in points
    pub right: f32,
    /// Bottom margin in points
    pub bottom: f32,
    /// Left margin in points
    pub left: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 7.2,    // ~0.1 inch
            right: 7.2,
            bottom: 7.2,
            left: 7.2,
        }
    }
}

impl Margins {
    /// Create margins with uniform value on all sides
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create margins with no padding
    pub fn none() -> Self {
        Self::uniform(0.0)
    }

    /// Create margins with specified values
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    /// Get total horizontal margin (left + right)
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical margin (top + bottom)
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Anchor type for text box positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnchorType {
    /// Inline with text (treated as a character)
    Inline,
    /// Anchored to a character position
    Character,
    /// Anchored to a paragraph
    #[default]
    Paragraph,
    /// Anchored to a page
    Page,
}

/// Horizontal position specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HorizontalPosition {
    /// Absolute offset from anchor in points
    Absolute(f32),
    /// Relative position (left, center, right, inside, outside)
    Relative(HorizontalRelative),
}

impl Default for HorizontalPosition {
    fn default() -> Self {
        Self::Absolute(0.0)
    }
}

/// Relative horizontal position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HorizontalRelative {
    #[default]
    Left,
    Center,
    Right,
    Inside,
    Outside,
}

/// Vertical position specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerticalPosition {
    /// Absolute offset from anchor in points
    Absolute(f32),
    /// Relative position (top, center, bottom, inside, outside)
    Relative(VerticalRelative),
}

impl Default for VerticalPosition {
    fn default() -> Self {
        Self::Absolute(0.0)
    }
}

/// Relative vertical position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VerticalRelative {
    #[default]
    Top,
    Center,
    Bottom,
    Inside,
    Outside,
}

/// Wrap mode for text around the text box
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WrapMode {
    /// No wrapping - text box floats independently
    None,
    /// Square wrapping - text wraps around bounding box
    #[default]
    Square,
    /// Tight wrapping - text wraps closely to content
    Tight,
    /// Through wrapping - text flows through transparent areas
    Through,
    /// Top and bottom - text only above and below
    TopAndBottom,
}

impl From<WrapMode> for WrapType {
    fn from(mode: WrapMode) -> Self {
        match mode {
            WrapMode::None => WrapType::InFront,
            WrapMode::Square => WrapType::Square,
            WrapMode::Tight => WrapType::Tight,
            WrapMode::Through => WrapType::Tight,
            WrapMode::TopAndBottom => WrapType::Square,
        }
    }
}

/// Anchor configuration for text box positioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    /// Type of anchor
    pub anchor_type: AnchorType,
    /// Horizontal position specification
    pub horizontal: HorizontalPosition,
    /// Vertical position specification
    pub vertical: VerticalPosition,
    /// How text wraps around the text box
    pub wrap_mode: WrapMode,
    /// Whether this text box can overlap with others
    pub allow_overlap: bool,
    /// Horizontal anchor reference
    pub horizontal_anchor: HorizontalAnchor,
    /// Vertical anchor reference
    pub vertical_anchor: VerticalAnchor,
}

impl Default for Anchor {
    fn default() -> Self {
        Self {
            anchor_type: AnchorType::Paragraph,
            horizontal: HorizontalPosition::Absolute(72.0),  // 1 inch from anchor
            vertical: VerticalPosition::Absolute(0.0),
            wrap_mode: WrapMode::Square,
            allow_overlap: false,
            horizontal_anchor: HorizontalAnchor::Column,
            vertical_anchor: VerticalAnchor::Paragraph,
        }
    }
}

impl Anchor {
    /// Create a new anchor with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an inline anchor
    pub fn inline() -> Self {
        Self {
            anchor_type: AnchorType::Inline,
            horizontal: HorizontalPosition::Absolute(0.0),
            vertical: VerticalPosition::Absolute(0.0),
            wrap_mode: WrapMode::None,
            allow_overlap: false,
            horizontal_anchor: HorizontalAnchor::Character,
            vertical_anchor: VerticalAnchor::Line,
        }
    }

    /// Create a page-anchored position
    pub fn page(x: f32, y: f32) -> Self {
        Self {
            anchor_type: AnchorType::Page,
            horizontal: HorizontalPosition::Absolute(x),
            vertical: VerticalPosition::Absolute(y),
            wrap_mode: WrapMode::Square,
            allow_overlap: true,
            horizontal_anchor: HorizontalAnchor::Page,
            vertical_anchor: VerticalAnchor::Page,
        }
    }

    /// Convert to ImagePosition for compatibility with existing layout
    pub fn to_image_position(&self) -> ImagePosition {
        match self.anchor_type {
            AnchorType::Inline => ImagePosition::Inline,
            _ => ImagePosition::Anchor(AnchorPosition {
                horizontal: self.horizontal_anchor,
                vertical: self.vertical_anchor,
                offset_x: match &self.horizontal {
                    HorizontalPosition::Absolute(x) => *x,
                    HorizontalPosition::Relative(_) => 0.0,
                },
                offset_y: match &self.vertical {
                    VerticalPosition::Absolute(y) => *y,
                    VerticalPosition::Relative(_) => 0.0,
                },
            }),
        }
    }

    /// Check if this is an inline anchor
    pub fn is_inline(&self) -> bool {
        matches!(self.anchor_type, AnchorType::Inline)
    }
}

/// Size specification for text boxes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    /// Width dimension
    pub width: Dimension,
    /// Height dimension
    pub height: Dimension,
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: Dimension::points(200.0),
            height: Dimension::points(100.0),
        }
    }
}

impl Size {
    /// Create a size with specific dimensions in points
    pub fn points(width: f32, height: f32) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
        }
    }

    /// Create a size with auto height (fit to content)
    pub fn auto_height(width: f32) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::auto(),
        }
    }

    /// Create a size with auto width (fit to content)
    pub fn auto_width(height: f32) -> Self {
        Self {
            width: Dimension::auto(),
            height: Dimension::points(height),
        }
    }

    /// Create a fully auto size (fit to content)
    pub fn auto() -> Self {
        Self {
            width: Dimension::auto(),
            height: Dimension::auto(),
        }
    }

    /// Check if width is auto
    pub fn is_width_auto(&self) -> bool {
        self.width.is_auto()
    }

    /// Check if height is auto
    pub fn is_height_auto(&self) -> bool {
        self.height.is_auto()
    }

    /// Resolve width to points given a reference size
    pub fn resolve_width(&self, reference: f32) -> Option<f32> {
        self.width.resolve(reference)
    }

    /// Resolve height to points given a reference size
    pub fn resolve_height(&self, reference: f32) -> Option<f32> {
        self.height.resolve(reference)
    }
}

/// Style properties for text boxes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBoxStyle {
    /// Border style
    pub border: Option<TextBoxBorderStyle>,
    /// Fill style
    pub fill: Option<FillStyle>,
    /// Internal margins (padding)
    pub internal_margins: Margins,
    /// Vertical alignment of content
    pub vertical_align: TextBoxVerticalAlign,
    /// Rotation in degrees (clockwise)
    pub rotation: f32,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
}

impl Default for TextBoxStyle {
    fn default() -> Self {
        Self {
            border: Some(TextBoxBorderStyle::uniform(1.0, "#000000", BorderLineStyle::Solid)),
            fill: Some(FillStyle::Solid("#FFFFFF".to_string())),
            internal_margins: Margins::default(),
            vertical_align: TextBoxVerticalAlign::Top,
            rotation: 0.0,
            opacity: 1.0,
        }
    }
}

impl TextBoxStyle {
    /// Create a new style with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a style with no border
    pub fn no_border() -> Self {
        Self {
            border: None,
            ..Default::default()
        }
    }

    /// Create a style with no fill (transparent)
    pub fn transparent() -> Self {
        Self {
            fill: None,
            ..Default::default()
        }
    }

    /// Create a style with no border and no fill
    pub fn invisible_container() -> Self {
        Self {
            border: None,
            fill: None,
            ..Default::default()
        }
    }
}

/// A text box node in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBox {
    /// Unique node ID
    id: NodeId,
    /// Parent node ID (usually a paragraph for anchoring)
    parent: Option<NodeId>,
    /// Anchor configuration
    pub anchor: Anchor,
    /// Size specification
    pub size: Size,
    /// Content (paragraph IDs inside the text box)
    pub content: Vec<NodeId>,
    /// Style properties
    pub style: TextBoxStyle,
    /// Optional name for the text box
    pub name: Option<String>,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
}

impl TextBox {
    /// Create a new text box with default settings
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            anchor: Anchor::default(),
            size: Size::default(),
            content: Vec::new(),
            style: TextBoxStyle::default(),
            name: None,
            alt_text: None,
        }
    }

    /// Create a text box with specific size
    pub fn with_size(width: f32, height: f32) -> Self {
        let mut tb = Self::new();
        tb.size = Size::points(width, height);
        tb
    }

    /// Create a text box with auto-fit height
    pub fn auto_fit(width: f32) -> Self {
        let mut tb = Self::new();
        tb.size = Size::auto_height(width);
        tb
    }

    /// Create an inline text box
    pub fn inline(width: f32, height: f32) -> Self {
        let mut tb = Self::with_size(width, height);
        tb.anchor = Anchor::inline();
        tb
    }

    /// Set the anchor
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    /// Set the size
    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    /// Set the style
    pub fn set_style(&mut self, style: TextBoxStyle) {
        self.style = style;
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Set the alt text
    pub fn set_alt_text(&mut self, alt_text: impl Into<String>) {
        self.alt_text = Some(alt_text.into());
    }

    /// Add a content paragraph ID
    pub fn add_content(&mut self, para_id: NodeId) {
        self.content.push(para_id);
    }

    /// Insert a content paragraph at a specific index
    pub fn insert_content(&mut self, index: usize, para_id: NodeId) {
        self.content.insert(index, para_id);
    }

    /// Remove a content paragraph by ID
    pub fn remove_content(&mut self, para_id: NodeId) -> bool {
        if let Some(pos) = self.content.iter().position(|&id| id == para_id) {
            self.content.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all content
    pub fn clear_content(&mut self) {
        self.content.clear();
    }

    /// Get the effective width in points
    pub fn effective_width(&self, container_width: f32) -> f32 {
        self.size.resolve_width(container_width).unwrap_or(200.0)
    }

    /// Get the effective height in points
    pub fn effective_height(&self, container_height: f32) -> f32 {
        self.size.resolve_height(container_height).unwrap_or(100.0)
    }

    /// Get the inner content width (accounting for margins and borders)
    pub fn inner_width(&self, container_width: f32) -> f32 {
        let outer = self.effective_width(container_width);
        let border_width = self.style.border.as_ref()
            .map(|b| b.left.width + b.right.width)
            .unwrap_or(0.0);
        (outer - self.style.internal_margins.horizontal() - border_width).max(0.0)
    }

    /// Get the inner content height (accounting for margins and borders)
    pub fn inner_height(&self, container_height: f32) -> f32 {
        let outer = self.effective_height(container_height);
        let border_height = self.style.border.as_ref()
            .map(|b| b.top.width + b.bottom.width)
            .unwrap_or(0.0);
        (outer - self.style.internal_margins.vertical() - border_height).max(0.0)
    }

    /// Check if this is an inline text box
    pub fn is_inline(&self) -> bool {
        self.anchor.is_inline()
    }

    /// Check if this is a floating text box
    pub fn is_floating(&self) -> bool {
        !self.is_inline()
    }

    /// Get the wrap type for layout compatibility
    pub fn wrap_type(&self) -> WrapType {
        self.anchor.wrap_mode.into()
    }
}

impl Default for TextBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for TextBox {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::TextBox
    }

    fn children(&self) -> &[NodeId] {
        &self.content
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        true // Text boxes can contain paragraphs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_box_creation() {
        let tb = TextBox::new();
        assert!(tb.content.is_empty());
        assert!(tb.is_floating());
    }

    #[test]
    fn test_text_box_with_size() {
        let tb = TextBox::with_size(200.0, 150.0);
        assert_eq!(tb.effective_width(500.0), 200.0);
        assert_eq!(tb.effective_height(500.0), 150.0);
    }

    #[test]
    fn test_inline_text_box() {
        let tb = TextBox::inline(100.0, 50.0);
        assert!(tb.is_inline());
        assert!(!tb.is_floating());
    }

    #[test]
    fn test_anchor_conversion() {
        let anchor = Anchor::page(100.0, 200.0);
        let img_pos = anchor.to_image_position();
        assert!(matches!(img_pos, ImagePosition::Anchor(_)));
    }

    #[test]
    fn test_margins() {
        let margins = Margins::uniform(10.0);
        assert_eq!(margins.horizontal(), 20.0);
        assert_eq!(margins.vertical(), 20.0);
    }

    #[test]
    fn test_border_style() {
        let border = TextBoxBorderStyle::uniform(2.0, "#FF0000", BorderLineStyle::Dashed);
        assert!(border.has_visible_border());

        let no_border = TextBoxBorderStyle::none();
        assert!(!no_border.has_visible_border());
    }

    #[test]
    fn test_inner_dimensions() {
        let tb = TextBox::with_size(200.0, 100.0);
        let inner_w = tb.inner_width(500.0);
        let inner_h = tb.inner_height(500.0);

        // Should be smaller than outer dimensions due to margins and borders
        assert!(inner_w < 200.0);
        assert!(inner_h < 100.0);
    }

    #[test]
    fn test_content_management() {
        let mut tb = TextBox::new();
        let para_id = NodeId::new();

        tb.add_content(para_id);
        assert_eq!(tb.content.len(), 1);
        assert_eq!(tb.content[0], para_id);

        assert!(tb.remove_content(para_id));
        assert!(tb.content.is_empty());
    }

    #[test]
    fn test_wrap_mode_conversion() {
        assert_eq!(WrapType::from(WrapMode::Square), WrapType::Square);
        assert_eq!(WrapType::from(WrapMode::None), WrapType::InFront);
    }
}
