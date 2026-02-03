//! Render item types

use serde::{Deserialize, Serialize};

/// A rectangle in render coordinates
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }
}

impl From<layout_engine::Rect> for Rect {
    fn from(r: layout_engine::Rect) -> Self {
        Self {
            x: r.x as f64,
            y: r.y as f64,
            width: r.width as f64,
            height: r.height as f64,
        }
    }
}

/// Color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// A glyph run for rendering text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphRun {
    /// The text to render
    pub text: String,
    /// Font family
    pub font_family: String,
    /// Font size in points
    pub font_size: f64,
    /// Whether bold
    pub bold: bool,
    /// Whether italic
    pub italic: bool,
    /// Whether underlined
    pub underline: bool,
    /// Text color
    pub color: Color,
    /// Position (baseline start)
    pub x: f64,
    pub y: f64,
    /// Optional hyperlink info (target URL and tooltip)
    pub hyperlink: Option<HyperlinkRenderInfo>,
}

/// Hyperlink information for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperlinkRenderInfo {
    /// The hyperlink node ID (for click handling)
    pub node_id: String,
    /// The link target URL
    pub target: String,
    /// Optional tooltip text
    pub tooltip: Option<String>,
    /// Type of link (external, internal, email)
    pub link_type: HyperlinkType,
}

/// Type of hyperlink
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HyperlinkType {
    External,
    Internal,
    Email,
}

/// Image render information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRenderInfo {
    /// The image node ID (for selection handling)
    pub node_id: String,
    /// Resource ID for fetching image data
    pub resource_id: String,
    /// Bounds where the image should be rendered
    pub bounds: Rect,
    /// Rotation in degrees (clockwise)
    pub rotation: f64,
    /// Alternative text
    pub alt_text: Option<String>,
    /// Title (tooltip)
    pub title: Option<String>,
    /// Whether this image is selected
    pub selected: bool,
}

/// Shape type for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ShapeRenderType {
    Rectangle,
    RoundedRectangle { corner_radius: f64 },
    Oval,
    Line,
    Arrow,
    DoubleArrow,
    Triangle,
    Diamond,
    Pentagon,
    Hexagon,
    Star { points: u8, inner_radius_ratio: f64 },
    Callout { tail_position: (f64, f64), tail_width: f64 },
    TextBox,
    RightArrowBlock,
    LeftArrowBlock,
    UpArrowBlock,
    DownArrowBlock,
}

/// Fill style for shape rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShapeFillRender {
    Solid { color: Color },
    Gradient { colors: Vec<(Color, f64)>, angle: f64 },
    None,
}

/// Stroke dash style for rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DashStyleRender {
    Solid,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
}

/// Stroke style for shape rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeStrokeRender {
    pub color: Color,
    pub width: f64,
    pub dash_style: DashStyleRender,
}

/// Shadow effect for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowRender {
    pub color: Color,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur_radius: f64,
}

/// Shape render information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeRenderInfo {
    /// The shape node ID (for selection handling)
    pub node_id: String,
    /// Type of shape
    pub shape_type: ShapeRenderType,
    /// Bounds where the shape should be rendered
    pub bounds: Rect,
    /// Rotation in degrees (clockwise)
    pub rotation: f64,
    /// Fill style
    pub fill: Option<ShapeFillRender>,
    /// Stroke style
    pub stroke: Option<ShapeStrokeRender>,
    /// Shadow effect
    pub shadow: Option<ShadowRender>,
    /// Opacity (0.0 to 1.0)
    pub opacity: f64,
    /// Whether this shape is selected
    pub selected: bool,
    /// Whether flipped horizontally
    pub flip_horizontal: bool,
    /// Whether flipped vertically
    pub flip_vertical: bool,
}

impl ImageRenderInfo {
    pub fn new(
        node_id: impl Into<String>,
        resource_id: impl Into<String>,
        bounds: Rect,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            resource_id: resource_id.into(),
            bounds,
            rotation: 0.0,
            alt_text: None,
            title: None,
            selected: false,
        }
    }
}

/// Text box border edge render info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoxBorderEdgeRender {
    /// Border width
    pub width: f64,
    /// Border color
    pub color: Color,
    /// Line style: "solid", "dashed", "dotted", "double", "none"
    pub style: String,
}

impl Default for TextBoxBorderEdgeRender {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color::BLACK,
            style: "solid".to_string(),
        }
    }
}

/// Text box border render info (all four edges)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextBoxBorderRender {
    pub top: TextBoxBorderEdgeRender,
    pub right: TextBoxBorderEdgeRender,
    pub bottom: TextBoxBorderEdgeRender,
    pub left: TextBoxBorderEdgeRender,
}

/// Text box fill style
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TextBoxFillRender {
    Solid { color: Color },
    Gradient { colors: Vec<(Color, f64)>, angle: f64 },
    None,
}

impl Default for TextBoxFillRender {
    fn default() -> Self {
        Self::Solid { color: Color::WHITE }
    }
}

/// Text box render information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoxRenderInfo {
    /// The text box node ID (for selection/editing handling)
    pub node_id: String,
    /// Outer bounds where the text box should be rendered
    pub bounds: Rect,
    /// Inner content bounds (accounting for margins and borders)
    pub content_bounds: Rect,
    /// Rotation in degrees (clockwise)
    pub rotation: f64,
    /// Fill style
    pub fill: Option<TextBoxFillRender>,
    /// Border style
    pub border: Option<TextBoxBorderRender>,
    /// Opacity (0.0 to 1.0)
    pub opacity: f64,
    /// Alternative text
    pub alt_text: Option<String>,
    /// Name
    pub name: Option<String>,
    /// Whether this text box is selected
    pub selected: bool,
    /// Whether this text box is in edit mode
    pub is_editing: bool,
    /// Content render items (text inside the text box)
    pub content_items: Vec<RenderItem>,
}

impl TextBoxRenderInfo {
    pub fn new(node_id: impl Into<String>, bounds: Rect) -> Self {
        let content_bounds = Rect::new(
            bounds.x + 7.2,
            bounds.y + 7.2,
            (bounds.width - 14.4).max(0.0),
            (bounds.height - 14.4).max(0.0),
        );
        Self {
            node_id: node_id.into(),
            bounds,
            content_bounds,
            rotation: 0.0,
            fill: Some(TextBoxFillRender::Solid { color: Color::WHITE }),
            border: Some(TextBoxBorderRender::default()),
            opacity: 1.0,
            alt_text: None,
            name: None,
            selected: false,
            is_editing: false,
            content_items: Vec::new(),
        }
    }
}

/// Table cell render information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCellRenderInfo {
    /// The cell node ID
    pub cell_id: String,
    /// Cell bounds
    pub bounds: Rect,
    /// Background/shading color (if any)
    pub background: Option<Color>,
    /// Whether this cell is selected
    pub selected: bool,
}

/// Table border render information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableBorderRenderInfo {
    /// Start point
    pub x1: f64,
    pub y1: f64,
    /// End point
    pub x2: f64,
    pub y2: f64,
    /// Border color
    pub color: Color,
    /// Border width
    pub width: f64,
    /// Border style (for future use: single, double, dotted, etc.)
    pub style: String,
}

impl TableBorderRenderInfo {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, color: Color, width: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
            style: "single".to_string(),
        }
    }

    /// Create a horizontal border
    pub fn horizontal(x: f64, y: f64, length: f64, color: Color, width: f64) -> Self {
        Self::new(x, y, x + length, y, color, width)
    }

    /// Create a vertical border
    pub fn vertical(x: f64, y: f64, length: f64, color: Color, width: f64) -> Self {
        Self::new(x, y, x, y + length, color, width)
    }
}

/// Line number render info (for margin line numbers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineNumberRenderInfo {
    /// The line number value to display
    pub number: u32,
    /// X position (right edge of the number text)
    pub x: f64,
    /// Y position (baseline)
    pub y: f64,
    /// Font size
    pub font_size: f64,
    /// Font family
    pub font_family: String,
    /// Text color
    pub color: Color,
}

impl LineNumberRenderInfo {
    /// Create a new line number render info
    pub fn new(number: u32, x: f64, y: f64, font_size: f64) -> Self {
        Self {
            number,
            x,
            y,
            font_size,
            font_family: "sans-serif".to_string(),
            color: Color::rgb(128, 128, 128), // Gray by default
        }
    }

    /// Set the font family
    pub fn with_font_family(mut self, font_family: impl Into<String>) -> Self {
        self.font_family = font_family.into();
        self
    }

    /// Set the color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Squiggly underline render info (for spellcheck etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquigglyRenderInfo {
    /// Position and size
    pub bounds: Rect,
    /// Color of the squiggly
    pub color: Color,
    /// The node ID this underline belongs to
    pub node_id: String,
    /// Start offset in the text
    pub start_offset: usize,
    /// End offset in the text
    pub end_offset: usize,
    /// Optional error message/tooltip
    pub message: Option<String>,
}

impl SquigglyRenderInfo {
    /// Create a new squiggly render info
    pub fn new(bounds: Rect, color: Color, node_id: impl Into<String>) -> Self {
        Self {
            bounds,
            color,
            node_id: node_id.into(),
            start_offset: 0,
            end_offset: 0,
            message: None,
        }
    }
}

/// Render item types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RenderItem {
    /// A glyph run (text)
    GlyphRun(GlyphRun),
    /// A filled rectangle
    Rectangle {
        bounds: Rect,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: f64,
    },
    /// The caret (cursor)
    Caret {
        x: f64,
        y: f64,
        height: f64,
        color: Color,
    },
    /// Selection highlight
    Selection {
        rects: Vec<Rect>,
        color: Color,
    },
    /// A line
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        width: f64,
    },
    /// An image
    Image(ImageRenderInfo),
    /// A shape
    Shape(ShapeRenderInfo),
    /// A text box
    TextBox(TextBoxRenderInfo),
    /// A table cell (background)
    TableCell(TableCellRenderInfo),
    /// A table border
    TableBorder(TableBorderRenderInfo),
    /// A squiggly underline (spellcheck errors)
    Squiggly(SquigglyRenderInfo),
    /// Find/replace highlight
    FindHighlight {
        bounds: Rect,
        color: Color,
        is_current: bool,
    },
    /// Line number in the margin
    LineNumber(LineNumberRenderInfo),
}

/// A rendered page
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageRender {
    pub page_index: u32,
    pub width: f64,
    pub height: f64,
    pub items: Vec<RenderItem>,
}

/// The complete render model
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderModel {
    pub pages: Vec<PageRender>,
}

impl RenderModel {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn add_page(&mut self, page: PageRender) {
        self.pages.push(page);
    }
}
