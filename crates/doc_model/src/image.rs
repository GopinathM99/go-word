//! Image node types for inline and floating images
//!
//! This module provides the data structures for representing images in the document,
//! including inline images (treated as characters in text flow) and floating images
//! (with text wrap options).

use crate::{Node, NodeId, NodeType};
use serde::{Deserialize, Serialize};

/// Unique identifier for stored image resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(String);

impl ResourceId {
    /// Create a new resource ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ResourceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ResourceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unit for dimension values
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionUnit {
    /// Points (1/72 inch)
    Points,
    /// Percentage of container width/height
    Percent,
    /// Auto-calculate based on aspect ratio
    Auto,
}

impl Default for DimensionUnit {
    fn default() -> Self {
        Self::Points
    }
}

/// A dimension value with unit
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dimension {
    /// The numeric value
    pub value: f32,
    /// The unit type
    pub unit: DimensionUnit,
}

impl Dimension {
    /// Create a new dimension in points
    pub fn points(value: f32) -> Self {
        Self {
            value,
            unit: DimensionUnit::Points,
        }
    }

    /// Create a new percentage dimension
    pub fn percent(value: f32) -> Self {
        Self {
            value,
            unit: DimensionUnit::Percent,
        }
    }

    /// Create an auto dimension
    pub fn auto() -> Self {
        Self {
            value: 0.0,
            unit: DimensionUnit::Auto,
        }
    }

    /// Check if this dimension is auto
    pub fn is_auto(&self) -> bool {
        matches!(self.unit, DimensionUnit::Auto)
    }

    /// Resolve the dimension to points given a reference size
    pub fn resolve(&self, reference_size: f32) -> Option<f32> {
        match self.unit {
            DimensionUnit::Points => Some(self.value),
            DimensionUnit::Percent => Some(self.value / 100.0 * reference_size),
            DimensionUnit::Auto => None,
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::auto()
    }
}

/// How text wraps around an image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapType {
    /// Treated as a character in text flow
    Inline,
    /// Text wraps around bounding box
    Square,
    /// Text wraps close to image shape (future)
    Tight,
    /// Image behind text
    Behind,
    /// Image in front of text
    InFront,
}

impl Default for WrapType {
    fn default() -> Self {
        Self::Inline
    }
}

/// Horizontal anchor point for floating images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HorizontalAnchor {
    /// Relative to column
    Column,
    /// Relative to page
    Page,
    /// Relative to margin
    Margin,
    /// Relative to character position
    Character,
}

impl Default for HorizontalAnchor {
    fn default() -> Self {
        Self::Column
    }
}

/// Vertical anchor point for floating images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAnchor {
    /// Relative to paragraph
    Paragraph,
    /// Relative to page
    Page,
    /// Relative to margin
    Margin,
    /// Relative to line
    Line,
}

impl Default for VerticalAnchor {
    fn default() -> Self {
        Self::Paragraph
    }
}

/// Position information for floating images
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnchorPosition {
    /// Horizontal anchor type
    pub horizontal: HorizontalAnchor,
    /// Vertical anchor type
    pub vertical: VerticalAnchor,
    /// Horizontal offset from anchor in points
    pub offset_x: f32,
    /// Vertical offset from anchor in points
    pub offset_y: f32,
}

impl Default for AnchorPosition {
    fn default() -> Self {
        Self {
            horizontal: HorizontalAnchor::Column,
            vertical: VerticalAnchor::Paragraph,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

/// Image position type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ImagePosition {
    /// Inline with text flow
    Inline,
    /// Floating with anchor
    Anchor(AnchorPosition),
}

impl Default for ImagePosition {
    fn default() -> Self {
        Self::Inline
    }
}

/// Crop rectangle (all values as fractions 0.0 - 1.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CropRect {
    /// Left crop (fraction of width to remove from left)
    pub left: f32,
    /// Top crop (fraction of height to remove from top)
    pub top: f32,
    /// Right crop (fraction of width to remove from right)
    pub right: f32,
    /// Bottom crop (fraction of height to remove from bottom)
    pub bottom: f32,
}

impl CropRect {
    /// Create a new crop rect with no cropping
    pub fn none() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }

    /// Check if there is any cropping
    pub fn is_cropped(&self) -> bool {
        self.left > 0.0 || self.top > 0.0 || self.right > 0.0 || self.bottom > 0.0
    }

    /// Get the visible fraction of width
    pub fn visible_width_fraction(&self) -> f32 {
        1.0 - self.left - self.right
    }

    /// Get the visible fraction of height
    pub fn visible_height_fraction(&self) -> f32 {
        1.0 - self.top - self.bottom
    }
}

impl Default for CropRect {
    fn default() -> Self {
        Self::none()
    }
}

/// Properties controlling image appearance and layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageProperties {
    /// Width of the image
    pub width: Dimension,
    /// Height of the image
    pub height: Dimension,
    /// How text wraps around the image
    pub wrap_type: WrapType,
    /// Position type and anchor settings
    pub position: ImagePosition,
    /// Rotation in degrees (clockwise)
    pub rotation: f32,
    /// Crop rectangle
    pub crop: Option<CropRect>,
    /// Whether to lock aspect ratio during resize
    pub lock_aspect_ratio: bool,
}

impl ImageProperties {
    /// Create new properties with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create inline image properties with specific dimensions
    pub fn inline(width: f32, height: f32) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
            wrap_type: WrapType::Inline,
            position: ImagePosition::Inline,
            rotation: 0.0,
            crop: None,
            lock_aspect_ratio: true,
        }
    }

    /// Create floating image properties
    pub fn floating(width: f32, height: f32, wrap_type: WrapType) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
            wrap_type,
            position: ImagePosition::Anchor(AnchorPosition::default()),
            rotation: 0.0,
            crop: None,
            lock_aspect_ratio: true,
        }
    }
}

impl Default for ImageProperties {
    fn default() -> Self {
        Self {
            width: Dimension::auto(),
            height: Dimension::auto(),
            wrap_type: WrapType::Inline,
            position: ImagePosition::Inline,
            rotation: 0.0,
            crop: None,
            lock_aspect_ratio: true,
        }
    }
}

/// An image node in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageNode {
    /// Unique node ID
    id: NodeId,
    /// Parent node ID
    parent: Option<NodeId>,
    /// Reference to stored image data
    pub resource_id: ResourceId,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
    /// Image title (shown as tooltip)
    pub title: Option<String>,
    /// Image properties
    pub properties: ImageProperties,
    /// Original image width in pixels (from source)
    pub original_width: u32,
    /// Original image height in pixels (from source)
    pub original_height: u32,
}

impl ImageNode {
    /// Create a new image node
    pub fn new(resource_id: ResourceId, original_width: u32, original_height: u32) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            resource_id,
            alt_text: None,
            title: None,
            properties: ImageProperties::default(),
            original_width,
            original_height,
        }
    }

    /// Create an inline image with auto dimensions
    pub fn inline(resource_id: ResourceId, original_width: u32, original_height: u32) -> Self {
        let mut node = Self::new(resource_id, original_width, original_height);
        node.properties.wrap_type = WrapType::Inline;
        node.properties.position = ImagePosition::Inline;
        node
    }

    /// Create an image with specific size in points
    pub fn with_size(
        resource_id: ResourceId,
        original_width: u32,
        original_height: u32,
        width: f32,
        height: f32,
    ) -> Self {
        let mut node = Self::new(resource_id, original_width, original_height);
        node.properties.width = Dimension::points(width);
        node.properties.height = Dimension::points(height);
        node
    }

    /// Set the alt text
    pub fn set_alt_text(&mut self, alt_text: impl Into<String>) {
        self.alt_text = Some(alt_text.into());
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Set the properties
    pub fn set_properties(&mut self, properties: ImageProperties) {
        self.properties = properties;
    }

    /// Get aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        if self.original_height == 0 {
            1.0
        } else {
            self.original_width as f32 / self.original_height as f32
        }
    }

    /// Calculate the effective width in points, resolving auto dimensions
    pub fn effective_width(&self, container_width: f32) -> f32 {
        if let Some(w) = self.properties.width.resolve(container_width) {
            return w;
        }

        // Width is auto - calculate from height or use original
        if let Some(h) = self.properties.height.resolve(container_width) {
            // Calculate width from height maintaining aspect ratio
            h * self.aspect_ratio()
        } else {
            // Both auto - use original pixels as points (72 dpi assumption)
            self.original_width as f32
        }
    }

    /// Calculate the effective height in points, resolving auto dimensions
    pub fn effective_height(&self, container_height: f32) -> f32 {
        if let Some(h) = self.properties.height.resolve(container_height) {
            return h;
        }

        // Height is auto - calculate from width or use original
        if let Some(w) = self.properties.width.resolve(container_height) {
            // Calculate height from width maintaining aspect ratio
            w / self.aspect_ratio()
        } else {
            // Both auto - use original pixels as points (72 dpi assumption)
            self.original_height as f32
        }
    }

    /// Check if this is an inline image
    pub fn is_inline(&self) -> bool {
        matches!(self.properties.wrap_type, WrapType::Inline)
    }

    /// Check if this is a floating image
    pub fn is_floating(&self) -> bool {
        !self.is_inline()
    }
}

impl Node for ImageNode {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Image
    }

    fn children(&self) -> &[NodeId] {
        // Images have no children
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_resolve() {
        let points = Dimension::points(100.0);
        assert_eq!(points.resolve(500.0), Some(100.0));

        let percent = Dimension::percent(50.0);
        assert_eq!(percent.resolve(500.0), Some(250.0));

        let auto = Dimension::auto();
        assert_eq!(auto.resolve(500.0), None);
    }

    #[test]
    fn test_image_aspect_ratio() {
        let image = ImageNode::new(ResourceId::new("test"), 800, 600);
        assert!((image.aspect_ratio() - 1.333333).abs() < 0.001);
    }

    #[test]
    fn test_effective_dimensions() {
        let mut image = ImageNode::new(ResourceId::new("test"), 800, 600);

        // Auto dimensions use original size
        assert_eq!(image.effective_width(500.0), 800.0);
        assert_eq!(image.effective_height(500.0), 600.0);

        // Set explicit width, height auto-calculated
        image.properties.width = Dimension::points(400.0);
        assert_eq!(image.effective_width(500.0), 400.0);
        assert_eq!(image.effective_height(500.0), 300.0);
    }

    #[test]
    fn test_crop_rect() {
        let crop = CropRect {
            left: 0.1,
            top: 0.2,
            right: 0.1,
            bottom: 0.2,
        };
        assert!(crop.is_cropped());
        assert!((crop.visible_width_fraction() - 0.8).abs() < 0.001);
        assert!((crop.visible_height_fraction() - 0.6).abs() < 0.001);
    }
}
