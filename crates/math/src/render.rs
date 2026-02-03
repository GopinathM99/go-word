//! Math Rendering - Convert layout to render primitives
//!
//! This module converts the layout tree into render primitives that can be
//! drawn by a rendering backend.

use crate::error::MathResult;
use crate::layout::{LayoutBox, LayoutContent, Point, Rect};
use crate::model::MathFontStyle;
use serde::{Deserialize, Serialize};

// =============================================================================
// Render Primitives
// =============================================================================

/// A color in RGBA format
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Normal
    }
}

/// Font style (italic/normal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

/// Text styling for rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: Color,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Cambria Math".to_string(),
            font_size: 11.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Italic,
            color: Color::BLACK,
        }
    }
}

impl TextStyle {
    pub fn from_math_style(math_style: &crate::model::MathStyle, base_size: f32) -> Self {
        let (font_weight, font_style) = match math_style.font_style {
            MathFontStyle::Normal => (FontWeight::Normal, FontStyle::Normal),
            MathFontStyle::Italic => (FontWeight::Normal, FontStyle::Italic),
            MathFontStyle::Bold => (FontWeight::Bold, FontStyle::Normal),
            MathFontStyle::BoldItalic => (FontWeight::Bold, FontStyle::Italic),
            MathFontStyle::Script | MathFontStyle::BoldScript => {
                // Script fonts would use a different font family in practice
                (FontWeight::Normal, FontStyle::Italic)
            }
            MathFontStyle::Fraktur | MathFontStyle::BoldFraktur => {
                // Fraktur would use a different font family
                (FontWeight::Normal, FontStyle::Normal)
            }
            MathFontStyle::DoubleStruck => (FontWeight::Bold, FontStyle::Normal),
            MathFontStyle::SansSerif => (FontWeight::Normal, FontStyle::Normal),
            MathFontStyle::SansSerifBold => (FontWeight::Bold, FontStyle::Normal),
            MathFontStyle::SansSerifItalic => (FontWeight::Normal, FontStyle::Italic),
            MathFontStyle::SansSerifBoldItalic => (FontWeight::Bold, FontStyle::Italic),
            MathFontStyle::Monospace => (FontWeight::Normal, FontStyle::Normal),
        };

        Self {
            font_family: "Cambria Math".to_string(),
            font_size: base_size * math_style.size_multiplier,
            font_weight,
            font_style,
            color: Color::BLACK,
        }
    }
}

/// A render primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderPrimitive {
    /// Draw text at a position
    Text {
        text: String,
        position: Point,
        style: TextStyle,
    },
    /// Draw a line (for fraction bars, etc.)
    Line {
        start: Point,
        end: Point,
        thickness: f32,
        color: Color,
    },
    /// Draw a rectangle (for borders, backgrounds)
    Rectangle {
        rect: Rect,
        fill: Option<Color>,
        stroke: Option<(Color, f32)>,
    },
    /// Draw a path (for radicals, braces, etc.)
    Path {
        commands: Vec<PathCommand>,
        fill: Option<Color>,
        stroke: Option<(Color, f32)>,
    },
    /// Draw a glyph (for special characters)
    Glyph {
        char: char,
        position: Point,
        size: f32,
        color: Color,
    },
    /// A group of primitives with a transform
    Group {
        transform: Transform,
        children: Vec<RenderPrimitive>,
    },
}

/// Path drawing commands (similar to SVG)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),       // control point, end point
    CubicTo(f32, f32, f32, f32, f32, f32), // two control points, end point
    ArcTo {
        rx: f32,
        ry: f32,
        rotation: f32,
        large_arc: bool,
        sweep: bool,
        x: f32,
        y: f32,
    },
    Close,
}

/// 2D transform
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub translate_x: f32,
    pub translate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotate: f32, // radians
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate: 0.0,
        }
    }
}

impl Transform {
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            translate_x: x,
            translate_y: y,
            ..Default::default()
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            scale_x: sx,
            scale_y: sy,
            ..Default::default()
        }
    }

    pub fn is_identity(&self) -> bool {
        self.translate_x == 0.0
            && self.translate_y == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
            && self.rotate == 0.0
    }
}

// =============================================================================
// Render Output
// =============================================================================

/// The complete render output for a math expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOutput {
    /// All render primitives
    pub primitives: Vec<RenderPrimitive>,
    /// Total bounding box
    pub bounds: Rect,
    /// Baseline position (y coordinate)
    pub baseline: f32,
}

impl RenderOutput {
    pub fn new(primitives: Vec<RenderPrimitive>, bounds: Rect, baseline: f32) -> Self {
        Self {
            primitives,
            bounds,
            baseline,
        }
    }
}

// =============================================================================
// Renderer
// =============================================================================

/// Configuration for the renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Base font size
    pub font_size: f32,
    /// Text color
    pub color: Color,
    /// Font family for math
    pub font_family: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            font_size: 11.0,
            color: Color::BLACK,
            font_family: "Cambria Math".to_string(),
        }
    }
}

/// Renderer for converting layout to render primitives
pub struct Renderer {
    config: RenderConfig,
}

impl Renderer {
    /// Create a new renderer with default config
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Render a layout tree to primitives
    pub fn render(&self, layout: &LayoutBox) -> MathResult<RenderOutput> {
        let mut primitives = Vec::new();
        self.render_box(layout, Point::origin(), &mut primitives)?;

        Ok(RenderOutput::new(
            primitives,
            layout.bounds,
            layout.baseline_offset,
        ))
    }

    /// Render a single layout box
    fn render_box(
        &self,
        layout: &LayoutBox,
        offset: Point,
        primitives: &mut Vec<RenderPrimitive>,
    ) -> MathResult<()> {
        let pos = Point::new(
            offset.x + layout.bounds.x(),
            offset.y + layout.bounds.y(),
        );

        match &layout.content {
            LayoutContent::Container => {
                // Render children
                for child in &layout.children {
                    self.render_box(child, pos, primitives)?;
                }
            }
            LayoutContent::Text { text, style } => {
                let text_style = TextStyle::from_math_style(style, self.config.font_size);
                primitives.push(RenderPrimitive::Text {
                    text: text.clone(),
                    position: Point::new(pos.x, pos.y + layout.baseline_offset),
                    style: text_style,
                });
            }
            LayoutContent::Rule { thickness } => {
                primitives.push(RenderPrimitive::Line {
                    start: pos,
                    end: Point::new(pos.x + layout.width(), pos.y),
                    thickness: *thickness,
                    color: self.config.color,
                });
            }
            LayoutContent::Radical { degree_present: _ } => {
                // Draw radical symbol using path
                let path = self.radical_path(pos, layout.width(), layout.height());
                primitives.push(RenderPrimitive::Path {
                    commands: path,
                    fill: None,
                    stroke: Some((self.config.color, 1.0)),
                });

                // Render children (degree if present, then base content)
                for child in &layout.children {
                    self.render_box(child, pos, primitives)?;
                }
            }
            LayoutContent::Delimiter { char, stretched } => {
                if *stretched {
                    // Draw stretched delimiter using path
                    let path = self.stretched_delimiter_path(*char, pos, layout.height());
                    primitives.push(RenderPrimitive::Path {
                        commands: path,
                        fill: None,
                        stroke: Some((self.config.color, 1.0)),
                    });
                } else {
                    // Draw as glyph
                    primitives.push(RenderPrimitive::Glyph {
                        char: *char,
                        position: Point::new(pos.x, pos.y + layout.baseline_offset),
                        size: self.config.font_size,
                        color: self.config.color,
                    });
                }
            }
            LayoutContent::NaryOp { char } => {
                // Draw large operator
                primitives.push(RenderPrimitive::Glyph {
                    char: *char,
                    position: Point::new(pos.x, pos.y + layout.baseline_offset),
                    size: self.config.font_size * 1.5,
                    color: self.config.color,
                });

                // Render children (limits)
                for child in &layout.children {
                    self.render_box(child, pos, primitives)?;
                }
            }
            LayoutContent::Accent { char } => {
                // Draw accent character
                let accent_pos = Point::new(
                    pos.x + layout.width() / 2.0,
                    pos.y + layout.height() / 2.0,
                );
                primitives.push(RenderPrimitive::Glyph {
                    char: *char,
                    position: accent_pos,
                    size: self.config.font_size * 0.8,
                    color: self.config.color,
                });
            }
            LayoutContent::Space => {
                // Nothing to render
            }
        }

        Ok(())
    }

    /// Generate path commands for a radical symbol
    fn radical_path(&self, pos: Point, width: f32, height: f32) -> Vec<PathCommand> {
        let x = pos.x;
        let y = pos.y;

        // Simplified radical shape
        vec![
            PathCommand::MoveTo(x, y + height * 0.6),
            PathCommand::LineTo(x + width * 0.1, y + height * 0.6),
            PathCommand::LineTo(x + width * 0.2, y + height),
            PathCommand::LineTo(x + width * 0.35, y),
            PathCommand::LineTo(x + width, y),
        ]
    }

    /// Generate path commands for a stretched delimiter
    fn stretched_delimiter_path(&self, char: char, pos: Point, height: f32) -> Vec<PathCommand> {
        let x = pos.x;
        let y = pos.y;
        let w = height * 0.15; // Width proportional to height

        match char {
            '(' => {
                // Left parenthesis
                vec![
                    PathCommand::MoveTo(x + w, y),
                    PathCommand::QuadTo(x, y + height * 0.5, x + w, y + height),
                ]
            }
            ')' => {
                // Right parenthesis
                vec![
                    PathCommand::MoveTo(x, y),
                    PathCommand::QuadTo(x + w, y + height * 0.5, x, y + height),
                ]
            }
            '[' => {
                // Left bracket
                vec![
                    PathCommand::MoveTo(x + w, y),
                    PathCommand::LineTo(x, y),
                    PathCommand::LineTo(x, y + height),
                    PathCommand::LineTo(x + w, y + height),
                ]
            }
            ']' => {
                // Right bracket
                vec![
                    PathCommand::MoveTo(x, y),
                    PathCommand::LineTo(x + w, y),
                    PathCommand::LineTo(x + w, y + height),
                    PathCommand::LineTo(x, y + height),
                ]
            }
            '{' => {
                // Left brace
                let mid = y + height * 0.5;
                vec![
                    PathCommand::MoveTo(x + w, y),
                    PathCommand::QuadTo(x + w * 0.5, y, x + w * 0.5, y + height * 0.25),
                    PathCommand::QuadTo(x + w * 0.5, mid, x, mid),
                    PathCommand::QuadTo(x + w * 0.5, mid, x + w * 0.5, y + height * 0.75),
                    PathCommand::QuadTo(x + w * 0.5, y + height, x + w, y + height),
                ]
            }
            '}' => {
                // Right brace
                let mid = y + height * 0.5;
                vec![
                    PathCommand::MoveTo(x, y),
                    PathCommand::QuadTo(x + w * 0.5, y, x + w * 0.5, y + height * 0.25),
                    PathCommand::QuadTo(x + w * 0.5, mid, x + w, mid),
                    PathCommand::QuadTo(x + w * 0.5, mid, x + w * 0.5, y + height * 0.75),
                    PathCommand::QuadTo(x + w * 0.5, y + height, x, y + height),
                ]
            }
            '|' => {
                // Vertical bar
                vec![
                    PathCommand::MoveTo(x + w * 0.5, y),
                    PathCommand::LineTo(x + w * 0.5, y + height),
                ]
            }
            _ => {
                // Default: simple vertical line
                vec![
                    PathCommand::MoveTo(x + w * 0.5, y),
                    PathCommand::LineTo(x + w * 0.5, y + height),
                ]
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::LayoutEngine;
    use crate::model::MathNode;

    #[test]
    fn test_color_creation() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.font_family, "Cambria Math");
        assert_eq!(style.font_size, 11.0);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::default();
        assert!(t.is_identity());

        let t2 = Transform::translate(1.0, 0.0);
        assert!(!t2.is_identity());
    }

    #[test]
    fn test_render_simple_text() {
        let layout_engine = LayoutEngine::new();
        let renderer = Renderer::new();

        let node = MathNode::run("x");
        let layout = layout_engine.layout(&node).unwrap();
        let output = renderer.render(&layout).unwrap();

        assert!(!output.primitives.is_empty());
    }

    #[test]
    fn test_render_fraction() {
        let layout_engine = LayoutEngine::new();
        let renderer = Renderer::new();

        let node = MathNode::fraction(MathNode::run("a"), MathNode::run("b"));
        let layout = layout_engine.layout(&node).unwrap();
        let output = renderer.render(&layout).unwrap();

        // Should have text primitives and a line for fraction bar
        assert!(output.primitives.len() >= 2);
    }

    #[test]
    fn test_render_sqrt() {
        let layout_engine = LayoutEngine::new();
        let renderer = Renderer::new();

        let node = MathNode::sqrt(MathNode::run("x"));
        let layout = layout_engine.layout(&node).unwrap();
        let output = renderer.render(&layout).unwrap();

        // Should have path for radical
        let has_path = output.primitives.iter().any(|p| {
            matches!(p, RenderPrimitive::Path { .. })
        });
        assert!(has_path);
    }

    #[test]
    fn test_render_delimiter() {
        let layout_engine = LayoutEngine::new();
        let renderer = Renderer::new();

        let node = MathNode::parens(vec![MathNode::run("x")]);
        let layout = layout_engine.layout(&node).unwrap();
        let output = renderer.render(&layout).unwrap();

        assert!(!output.primitives.is_empty());
    }

    #[test]
    fn test_render_output_bounds() {
        let layout_engine = LayoutEngine::new();
        let renderer = Renderer::new();

        let node = MathNode::omath(vec![MathNode::run("x"), MathNode::operator('+'), MathNode::run("y")]);
        let layout = layout_engine.layout(&node).unwrap();
        let output = renderer.render(&layout).unwrap();

        assert!(output.bounds.width() > 0.0);
        assert!(output.bounds.height() > 0.0);
    }

    #[test]
    fn test_render_config() {
        let config = RenderConfig {
            font_size: 14.0,
            color: Color::BLUE,
            font_family: "Arial".to_string(),
        };
        let renderer = Renderer::with_config(config);

        let layout_engine = LayoutEngine::new();
        let node = MathNode::run("x");
        let layout = layout_engine.layout(&node).unwrap();
        let _output = renderer.render(&layout).unwrap();
    }
}
