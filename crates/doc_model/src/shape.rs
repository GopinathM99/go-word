//! Shape node types for inline and floating shapes
//!
//! This module provides the data structures for representing shapes in the document,
//! including basic shapes (rectangles, ovals, lines), callouts, flowchart shapes,
//! block arrows, stars, banners, and connectors. Shapes reuse the floating/anchor
//! system from images and support text content, grouping, and advanced styling.

use crate::{Dimension, ImagePosition, Node, NodeId, NodeType, WrapType};
use serde::{Deserialize, Serialize};

// =============================================================================
// Color Types
// =============================================================================

/// Color representation for shape fills and strokes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShapeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ShapeColor {
    /// Create an opaque RGB color
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create an RGBA color
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Common colors
    pub const BLACK: ShapeColor = ShapeColor::rgb(0, 0, 0);
    pub const WHITE: ShapeColor = ShapeColor::rgb(255, 255, 255);
    pub const TRANSPARENT: ShapeColor = ShapeColor::rgba(0, 0, 0, 0);
    pub const BLUE: ShapeColor = ShapeColor::rgb(68, 114, 196);
    pub const RED: ShapeColor = ShapeColor::rgb(192, 0, 0);
    pub const GREEN: ShapeColor = ShapeColor::rgb(84, 130, 53);
    pub const YELLOW: ShapeColor = ShapeColor::rgb(255, 192, 0);
    pub const ORANGE: ShapeColor = ShapeColor::rgb(237, 125, 49);
    pub const PURPLE: ShapeColor = ShapeColor::rgb(112, 48, 160);
    pub const GRAY: ShapeColor = ShapeColor::rgb(128, 128, 128);
    pub const LIGHT_GRAY: ShapeColor = ShapeColor::rgb(192, 192, 192);
    pub const DARK_GRAY: ShapeColor = ShapeColor::rgb(64, 64, 64);

    /// Convert to hex string (e.g., "#RRGGBB" or "#RRGGBBAA")
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }
}

impl Default for ShapeColor {
    fn default() -> Self {
        Self::BLACK
    }
}

// =============================================================================
// Shape Types - Expanded Library
// =============================================================================

/// Type of shape - comprehensive library of shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    // -------------------------------------------------------------------------
    // Basic Shapes (from Phase 1)
    // -------------------------------------------------------------------------
    /// Basic rectangle
    Rectangle,
    /// Rectangle with rounded corners
    RoundedRectangle {
        /// Corner radius in points
        corner_radius: f32,
    },
    /// Ellipse/oval
    Oval,
    /// Straight line (from corner to corner)
    Line,
    /// Line with arrow head
    Arrow,
    /// Line with arrow heads on both ends
    DoubleArrow,
    /// Triangle (equilateral)
    Triangle,
    /// Diamond (rotated square)
    Diamond,
    /// Pentagon (5 sides)
    Pentagon,
    /// Hexagon (6 sides)
    Hexagon,
    /// Star with configurable points
    Star {
        /// Number of star points (5-12)
        points: u8,
        /// Inner radius ratio (0.0 to 1.0)
        inner_radius_ratio: f32,
    },
    /// Callout/speech bubble (legacy)
    Callout {
        /// Tail position relative to shape (0.0-1.0 for x and y)
        tail_position: (f32, f32),
        /// Tail width in points
        tail_width: f32,
    },
    /// Text box (rectangular shape optimized for text)
    TextBox,

    // -------------------------------------------------------------------------
    // Block Arrows
    // -------------------------------------------------------------------------
    /// Right arrow block
    RightArrow {
        /// Arrow head width ratio (0.0 to 1.0)
        head_width: f32,
        /// Arrow head length ratio (0.0 to 1.0)
        head_length: f32,
    },
    /// Left arrow block
    LeftArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Up arrow block
    UpArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Down arrow block
    DownArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Left-right arrow (double-headed horizontal)
    LeftRightArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Up-down arrow (double-headed vertical)
    UpDownArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Quad arrow (4-way)
    QuadArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Bent arrow (90-degree turn)
    BentArrow {
        /// Turn position (0.0 to 1.0)
        bend_position: f32,
        head_width: f32,
        head_length: f32,
    },
    /// U-turn arrow
    UTurnArrow {
        head_width: f32,
        head_length: f32,
    },
    /// Chevron arrow (no tail)
    ChevronArrow {
        /// Chevron thickness ratio
        thickness: f32,
    },
    /// Striped right arrow
    StripedRightArrow {
        head_width: f32,
        head_length: f32,
        stripe_count: u8,
    },
    /// Notched right arrow
    NotchedRightArrow {
        head_width: f32,
        head_length: f32,
        notch_depth: f32,
    },
    /// Curved right arrow
    CurvedRightArrow {
        head_width: f32,
        head_length: f32,
        curve: f32,
    },
    /// Curved left arrow
    CurvedLeftArrow {
        head_width: f32,
        head_length: f32,
        curve: f32,
    },

    // Legacy block arrows (for backwards compatibility)
    /// Right arrow block (legacy)
    RightArrowBlock,
    /// Left arrow block (legacy)
    LeftArrowBlock,
    /// Up arrow block (legacy)
    UpArrowBlock,
    /// Down arrow block (legacy)
    DownArrowBlock,

    // -------------------------------------------------------------------------
    // Flowchart Shapes
    // -------------------------------------------------------------------------
    /// Process (rectangle) - standard flowchart process
    FlowchartProcess,
    /// Decision (diamond) - conditional branch
    FlowchartDecision,
    /// Data (parallelogram) - input/output
    FlowchartData,
    /// Terminator (rounded rectangle) - start/end
    FlowchartTerminator,
    /// Document shape - wavy bottom
    FlowchartDocument,
    /// Multi-document
    FlowchartMultiDocument,
    /// Predefined process - with side bars
    FlowchartPredefined,
    /// Manual input - keyboard input
    FlowchartManualInput,
    /// Preparation - hexagonal preparation step
    FlowchartPreparation,
    /// Internal storage
    FlowchartInternalStorage,
    /// Manual operation - trapezoid
    FlowchartManualOperation,
    /// Connector (circle) - on-page reference
    FlowchartConnector,
    /// Off-page connector (home plate shape)
    FlowchartOffPageConnector,
    /// Delay - D shape
    FlowchartDelay,
    /// Alternate process - rounded rectangle
    FlowchartAlternateProcess,
    /// Or - circle with cross
    FlowchartOr,
    /// Summing junction - circle with X
    FlowchartSummingJunction,
    /// Sort - two triangles
    FlowchartSort,
    /// Collate - hourglass
    FlowchartCollate,
    /// Extract - upward triangle
    FlowchartExtract,
    /// Merge - downward triangle
    FlowchartMerge,
    /// Stored data - cylinder on side
    FlowchartStoredData,
    /// Magnetic disk - cylinder
    FlowchartMagneticDisk,
    /// Direct access storage - cylinder
    FlowchartDirectAccessStorage,
    /// Sequential access storage - tape reel
    FlowchartSequentialAccess,
    /// Display - CRT shape
    FlowchartDisplay,
    /// Card - punched card
    FlowchartCard,
    /// Paper tape
    FlowchartPaperTape,

    // -------------------------------------------------------------------------
    // Callouts
    // -------------------------------------------------------------------------
    /// Rectangular callout
    RectangularCallout {
        /// Tail anchor point (0.0-1.0 for x, 0.0-1.0 for y relative to shape)
        tail_anchor: (f32, f32),
        /// Tail tip position (x, y offset from shape center in points)
        tail_tip: (f32, f32),
        /// Tail width at base in points
        tail_width: f32,
    },
    /// Rounded rectangular callout
    RoundedCallout {
        corner_radius: f32,
        tail_anchor: (f32, f32),
        tail_tip: (f32, f32),
        tail_width: f32,
    },
    /// Oval/ellipse callout
    OvalCallout {
        tail_anchor: (f32, f32),
        tail_tip: (f32, f32),
        tail_width: f32,
    },
    /// Cloud callout
    CloudCallout {
        tail_tip: (f32, f32),
        /// Number of bubbles in the tail
        bubble_count: u8,
    },
    /// Line callout (with leader line)
    LineCallout {
        line_type: CalloutLineType,
        /// Callout line start point relative to shape
        line_start: (f32, f32),
        /// Callout line end point (where the text is)
        line_end: (f32, f32),
        /// Whether to show accent bar
        accent_bar: bool,
        /// Callout text position
        text_position: CalloutTextPosition,
    },
    /// Thought bubble callout
    ThoughtBubbleCallout {
        tail_tip: (f32, f32),
        bubble_count: u8,
    },

    // -------------------------------------------------------------------------
    // Stars and Banners
    // -------------------------------------------------------------------------
    /// 4-pointed star
    Star4,
    /// 5-pointed star
    Star5,
    /// 6-pointed star (Star of David)
    Star6,
    /// 8-pointed star
    Star8,
    /// 10-pointed star
    Star10,
    /// 12-pointed star
    Star12,
    /// 16-pointed star (seal)
    Star16,
    /// 24-pointed star (starburst)
    Star24,
    /// 32-pointed star
    Star32,
    /// Explosion/burst shape
    Explosion1,
    /// Explosion variation
    Explosion2,
    /// Ribbon facing down
    Ribbon {
        /// Ribbon tail length ratio
        tail_length: f32,
        /// Whether tails point up or down
        tails_up: bool,
    },
    /// Ribbon 2 (curved)
    CurvedRibbon {
        curve: f32,
        tail_length: f32,
    },
    /// Wave shape
    Wave {
        /// Wave amplitude (0.0 to 1.0)
        amplitude: f32,
        /// Number of wave periods
        periods: f32,
    },
    /// Double wave shape
    DoubleWave {
        amplitude: f32,
        periods: f32,
    },
    /// Horizontal scroll
    HorizontalScroll {
        /// Scroll roll size ratio
        roll_size: f32,
    },
    /// Vertical scroll
    VerticalScroll {
        roll_size: f32,
    },

    // -------------------------------------------------------------------------
    // Basic Shapes - Additional
    // -------------------------------------------------------------------------
    /// Parallelogram
    Parallelogram {
        /// Slant ratio (0.0 to 1.0)
        slant: f32,
    },
    /// Trapezoid
    Trapezoid {
        /// Top width ratio compared to bottom (0.0 to 1.0)
        top_ratio: f32,
    },
    /// Octagon
    Octagon,
    /// Decagon (10 sides)
    Decagon,
    /// Dodecagon (12 sides)
    Dodecagon,
    /// Regular polygon with custom sides
    RegularPolygon {
        sides: u8,
    },
    /// Cross/plus shape
    Cross {
        /// Arm thickness ratio (0.0 to 0.5)
        thickness: f32,
    },
    /// Frame (hollow rectangle)
    Frame {
        /// Frame border thickness ratio
        thickness: f32,
    },
    /// L-shape
    LShape {
        /// Arm width ratio
        arm_width: f32,
    },
    /// Donut/ring shape
    Donut {
        /// Inner radius ratio (0.0 to 1.0)
        inner_radius: f32,
    },
    /// Arc shape
    Arc {
        /// Start angle in degrees
        start_angle: f32,
        /// End angle in degrees
        end_angle: f32,
        /// Arc thickness
        thickness: f32,
    },
    /// Pie/wedge shape
    Pie {
        start_angle: f32,
        end_angle: f32,
    },
    /// Chord (arc with straight line connecting ends)
    Chord {
        start_angle: f32,
        end_angle: f32,
    },
    /// Heart shape
    Heart,
    /// Lightning bolt
    LightningBolt,
    /// Sun shape
    Sun {
        /// Number of rays
        rays: u8,
    },
    /// Moon shape (crescent)
    Moon {
        /// Crescent size (0.0 to 1.0)
        crescent: f32,
    },
    /// Cloud shape
    Cloud,
    /// Smiley face
    SmileyFace,
    /// No symbol (circle with slash)
    NoSymbol,
    /// Block arc
    BlockArc {
        start_angle: f32,
        end_angle: f32,
        thickness: f32,
    },
    /// Folded corner (rectangle with folded corner)
    FoldedCorner {
        /// Fold size ratio
        fold_size: f32,
    },
    /// Bevel (3D looking rectangle)
    Bevel {
        /// Bevel depth
        depth: f32,
    },
    /// Cube (3D box)
    Cube {
        /// Depth perspective ratio
        depth: f32,
    },

    // -------------------------------------------------------------------------
    // Equation Shapes
    // -------------------------------------------------------------------------
    /// Plus sign
    MathPlus,
    /// Minus sign
    MathMinus,
    /// Multiply sign (X)
    MathMultiply,
    /// Divide sign
    MathDivide,
    /// Equal sign
    MathEqual,
    /// Not equal sign
    MathNotEqual,

    // -------------------------------------------------------------------------
    // Action Buttons
    // -------------------------------------------------------------------------
    /// Action button: blank
    ActionButtonBlank,
    /// Action button: home
    ActionButtonHome,
    /// Action button: help
    ActionButtonHelp,
    /// Action button: information
    ActionButtonInformation,
    /// Action button: back/previous
    ActionButtonBack,
    /// Action button: forward/next
    ActionButtonForward,
    /// Action button: beginning
    ActionButtonBeginning,
    /// Action button: end
    ActionButtonEnd,
    /// Action button: return
    ActionButtonReturn,
    /// Action button: document
    ActionButtonDocument,
    /// Action button: sound
    ActionButtonSound,
    /// Action button: movie
    ActionButtonMovie,

    // -------------------------------------------------------------------------
    // Custom/Freeform
    // -------------------------------------------------------------------------
    /// Custom shape defined by path
    CustomPath {
        /// SVG-like path data
        path_data: String,
    },
    /// Freeform shape with explicit points
    Freeform {
        /// Points defining the shape (closed polygon)
        points: Vec<Point>,
        /// Whether the shape is closed
        closed: bool,
    },
}

/// Point structure for freeform shapes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };
}

/// Callout line type for line callouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalloutLineType {
    /// Single straight line
    Single,
    /// Two-segment line (one bend)
    Double,
    /// Three-segment line (two bends)
    Triple,
    /// No line
    None,
}

impl Default for CalloutLineType {
    fn default() -> Self {
        Self::Single
    }
}

/// Text position for line callouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalloutTextPosition {
    /// Text at the end of the line
    End,
    /// Text beside the line
    Beside,
    /// Text above the line
    Above,
    /// Text below the line
    Below,
}

impl Default for CalloutTextPosition {
    fn default() -> Self {
        Self::End
    }
}

/// Shape category for organizing shapes in UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeCategory {
    Basic,
    BlockArrows,
    Flowchart,
    Callouts,
    StarsAndBanners,
    Equation,
    ActionButtons,
    Custom,
}

impl Default for ShapeType {
    fn default() -> Self {
        Self::Rectangle
    }
}

impl ShapeType {
    /// Get a user-friendly name for the shape type
    pub fn display_name(&self) -> &str {
        match self {
            // Basic shapes
            ShapeType::Rectangle => "Rectangle",
            ShapeType::RoundedRectangle { .. } => "Rounded Rectangle",
            ShapeType::Oval => "Oval",
            ShapeType::Line => "Line",
            ShapeType::Arrow => "Arrow",
            ShapeType::DoubleArrow => "Double Arrow",
            ShapeType::Triangle => "Triangle",
            ShapeType::Diamond => "Diamond",
            ShapeType::Pentagon => "Pentagon",
            ShapeType::Hexagon => "Hexagon",
            ShapeType::Star { .. } => "Star",
            ShapeType::Callout { .. } => "Callout",
            ShapeType::TextBox => "Text Box",

            // Block arrows
            ShapeType::RightArrow { .. } => "Right Arrow",
            ShapeType::LeftArrow { .. } => "Left Arrow",
            ShapeType::UpArrow { .. } => "Up Arrow",
            ShapeType::DownArrow { .. } => "Down Arrow",
            ShapeType::LeftRightArrow { .. } => "Left-Right Arrow",
            ShapeType::UpDownArrow { .. } => "Up-Down Arrow",
            ShapeType::QuadArrow { .. } => "Quad Arrow",
            ShapeType::BentArrow { .. } => "Bent Arrow",
            ShapeType::UTurnArrow { .. } => "U-Turn Arrow",
            ShapeType::ChevronArrow { .. } => "Chevron",
            ShapeType::StripedRightArrow { .. } => "Striped Right Arrow",
            ShapeType::NotchedRightArrow { .. } => "Notched Right Arrow",
            ShapeType::CurvedRightArrow { .. } => "Curved Right Arrow",
            ShapeType::CurvedLeftArrow { .. } => "Curved Left Arrow",
            ShapeType::RightArrowBlock => "Right Arrow Block",
            ShapeType::LeftArrowBlock => "Left Arrow Block",
            ShapeType::UpArrowBlock => "Up Arrow Block",
            ShapeType::DownArrowBlock => "Down Arrow Block",

            // Flowchart shapes
            ShapeType::FlowchartProcess => "Process",
            ShapeType::FlowchartDecision => "Decision",
            ShapeType::FlowchartData => "Data",
            ShapeType::FlowchartTerminator => "Terminator",
            ShapeType::FlowchartDocument => "Document",
            ShapeType::FlowchartMultiDocument => "Multi-Document",
            ShapeType::FlowchartPredefined => "Predefined Process",
            ShapeType::FlowchartManualInput => "Manual Input",
            ShapeType::FlowchartPreparation => "Preparation",
            ShapeType::FlowchartInternalStorage => "Internal Storage",
            ShapeType::FlowchartManualOperation => "Manual Operation",
            ShapeType::FlowchartConnector => "Connector",
            ShapeType::FlowchartOffPageConnector => "Off-Page Connector",
            ShapeType::FlowchartDelay => "Delay",
            ShapeType::FlowchartAlternateProcess => "Alternate Process",
            ShapeType::FlowchartOr => "Or",
            ShapeType::FlowchartSummingJunction => "Summing Junction",
            ShapeType::FlowchartSort => "Sort",
            ShapeType::FlowchartCollate => "Collate",
            ShapeType::FlowchartExtract => "Extract",
            ShapeType::FlowchartMerge => "Merge",
            ShapeType::FlowchartStoredData => "Stored Data",
            ShapeType::FlowchartMagneticDisk => "Magnetic Disk",
            ShapeType::FlowchartDirectAccessStorage => "Direct Access Storage",
            ShapeType::FlowchartSequentialAccess => "Sequential Access",
            ShapeType::FlowchartDisplay => "Display",
            ShapeType::FlowchartCard => "Card",
            ShapeType::FlowchartPaperTape => "Paper Tape",

            // Callouts
            ShapeType::RectangularCallout { .. } => "Rectangular Callout",
            ShapeType::RoundedCallout { .. } => "Rounded Callout",
            ShapeType::OvalCallout { .. } => "Oval Callout",
            ShapeType::CloudCallout { .. } => "Cloud Callout",
            ShapeType::LineCallout { .. } => "Line Callout",
            ShapeType::ThoughtBubbleCallout { .. } => "Thought Bubble",

            // Stars and banners
            ShapeType::Star4 => "4-Point Star",
            ShapeType::Star5 => "5-Point Star",
            ShapeType::Star6 => "6-Point Star",
            ShapeType::Star8 => "8-Point Star",
            ShapeType::Star10 => "10-Point Star",
            ShapeType::Star12 => "12-Point Star",
            ShapeType::Star16 => "16-Point Star",
            ShapeType::Star24 => "24-Point Star",
            ShapeType::Star32 => "32-Point Star",
            ShapeType::Explosion1 => "Explosion 1",
            ShapeType::Explosion2 => "Explosion 2",
            ShapeType::Ribbon { .. } => "Ribbon",
            ShapeType::CurvedRibbon { .. } => "Curved Ribbon",
            ShapeType::Wave { .. } => "Wave",
            ShapeType::DoubleWave { .. } => "Double Wave",
            ShapeType::HorizontalScroll { .. } => "Horizontal Scroll",
            ShapeType::VerticalScroll { .. } => "Vertical Scroll",

            // Additional basic shapes
            ShapeType::Parallelogram { .. } => "Parallelogram",
            ShapeType::Trapezoid { .. } => "Trapezoid",
            ShapeType::Octagon => "Octagon",
            ShapeType::Decagon => "Decagon",
            ShapeType::Dodecagon => "Dodecagon",
            ShapeType::RegularPolygon { .. } => "Regular Polygon",
            ShapeType::Cross { .. } => "Cross",
            ShapeType::Frame { .. } => "Frame",
            ShapeType::LShape { .. } => "L-Shape",
            ShapeType::Donut { .. } => "Donut",
            ShapeType::Arc { .. } => "Arc",
            ShapeType::Pie { .. } => "Pie",
            ShapeType::Chord { .. } => "Chord",
            ShapeType::Heart => "Heart",
            ShapeType::LightningBolt => "Lightning Bolt",
            ShapeType::Sun { .. } => "Sun",
            ShapeType::Moon { .. } => "Moon",
            ShapeType::Cloud => "Cloud",
            ShapeType::SmileyFace => "Smiley Face",
            ShapeType::NoSymbol => "No Symbol",
            ShapeType::BlockArc { .. } => "Block Arc",
            ShapeType::FoldedCorner { .. } => "Folded Corner",
            ShapeType::Bevel { .. } => "Bevel",
            ShapeType::Cube { .. } => "Cube",

            // Equation shapes
            ShapeType::MathPlus => "Plus",
            ShapeType::MathMinus => "Minus",
            ShapeType::MathMultiply => "Multiply",
            ShapeType::MathDivide => "Divide",
            ShapeType::MathEqual => "Equal",
            ShapeType::MathNotEqual => "Not Equal",

            // Action buttons
            ShapeType::ActionButtonBlank => "Blank Button",
            ShapeType::ActionButtonHome => "Home Button",
            ShapeType::ActionButtonHelp => "Help Button",
            ShapeType::ActionButtonInformation => "Information Button",
            ShapeType::ActionButtonBack => "Back Button",
            ShapeType::ActionButtonForward => "Forward Button",
            ShapeType::ActionButtonBeginning => "Beginning Button",
            ShapeType::ActionButtonEnd => "End Button",
            ShapeType::ActionButtonReturn => "Return Button",
            ShapeType::ActionButtonDocument => "Document Button",
            ShapeType::ActionButtonSound => "Sound Button",
            ShapeType::ActionButtonMovie => "Movie Button",

            // Custom
            ShapeType::CustomPath { .. } => "Custom Shape",
            ShapeType::Freeform { .. } => "Freeform",
        }
    }

    /// Check if this shape type is a line-based shape
    pub fn is_line(&self) -> bool {
        matches!(self, ShapeType::Line | ShapeType::Arrow | ShapeType::DoubleArrow)
    }

    /// Check if this shape type can contain text
    pub fn can_contain_text(&self) -> bool {
        !self.is_line()
    }

    /// Get the category of this shape
    pub fn category(&self) -> ShapeCategory {
        match self {
            // Basic shapes
            ShapeType::Rectangle | ShapeType::RoundedRectangle { .. } |
            ShapeType::Oval | ShapeType::Line | ShapeType::Arrow |
            ShapeType::DoubleArrow | ShapeType::Triangle | ShapeType::Diamond |
            ShapeType::Pentagon | ShapeType::Hexagon | ShapeType::Star { .. } |
            ShapeType::TextBox | ShapeType::Parallelogram { .. } |
            ShapeType::Trapezoid { .. } | ShapeType::Octagon | ShapeType::Decagon |
            ShapeType::Dodecagon | ShapeType::RegularPolygon { .. } |
            ShapeType::Cross { .. } | ShapeType::Frame { .. } | ShapeType::LShape { .. } |
            ShapeType::Donut { .. } | ShapeType::Arc { .. } | ShapeType::Pie { .. } |
            ShapeType::Chord { .. } | ShapeType::Heart | ShapeType::LightningBolt |
            ShapeType::Sun { .. } | ShapeType::Moon { .. } | ShapeType::Cloud |
            ShapeType::SmileyFace | ShapeType::NoSymbol | ShapeType::BlockArc { .. } |
            ShapeType::FoldedCorner { .. } | ShapeType::Bevel { .. } | ShapeType::Cube { .. } => {
                ShapeCategory::Basic
            }

            // Block arrows
            ShapeType::RightArrow { .. } | ShapeType::LeftArrow { .. } |
            ShapeType::UpArrow { .. } | ShapeType::DownArrow { .. } |
            ShapeType::LeftRightArrow { .. } | ShapeType::UpDownArrow { .. } |
            ShapeType::QuadArrow { .. } | ShapeType::BentArrow { .. } |
            ShapeType::UTurnArrow { .. } | ShapeType::ChevronArrow { .. } |
            ShapeType::StripedRightArrow { .. } | ShapeType::NotchedRightArrow { .. } |
            ShapeType::CurvedRightArrow { .. } | ShapeType::CurvedLeftArrow { .. } |
            ShapeType::RightArrowBlock | ShapeType::LeftArrowBlock |
            ShapeType::UpArrowBlock | ShapeType::DownArrowBlock => {
                ShapeCategory::BlockArrows
            }

            // Flowchart shapes
            ShapeType::FlowchartProcess | ShapeType::FlowchartDecision |
            ShapeType::FlowchartData | ShapeType::FlowchartTerminator |
            ShapeType::FlowchartDocument | ShapeType::FlowchartMultiDocument |
            ShapeType::FlowchartPredefined | ShapeType::FlowchartManualInput |
            ShapeType::FlowchartPreparation | ShapeType::FlowchartInternalStorage |
            ShapeType::FlowchartManualOperation | ShapeType::FlowchartConnector |
            ShapeType::FlowchartOffPageConnector | ShapeType::FlowchartDelay |
            ShapeType::FlowchartAlternateProcess | ShapeType::FlowchartOr |
            ShapeType::FlowchartSummingJunction | ShapeType::FlowchartSort |
            ShapeType::FlowchartCollate | ShapeType::FlowchartExtract |
            ShapeType::FlowchartMerge | ShapeType::FlowchartStoredData |
            ShapeType::FlowchartMagneticDisk | ShapeType::FlowchartDirectAccessStorage |
            ShapeType::FlowchartSequentialAccess | ShapeType::FlowchartDisplay |
            ShapeType::FlowchartCard | ShapeType::FlowchartPaperTape => {
                ShapeCategory::Flowchart
            }

            // Callouts
            ShapeType::Callout { .. } | ShapeType::RectangularCallout { .. } |
            ShapeType::RoundedCallout { .. } | ShapeType::OvalCallout { .. } |
            ShapeType::CloudCallout { .. } | ShapeType::LineCallout { .. } |
            ShapeType::ThoughtBubbleCallout { .. } => {
                ShapeCategory::Callouts
            }

            // Stars and banners
            ShapeType::Star4 | ShapeType::Star5 | ShapeType::Star6 |
            ShapeType::Star8 | ShapeType::Star10 | ShapeType::Star12 |
            ShapeType::Star16 | ShapeType::Star24 | ShapeType::Star32 |
            ShapeType::Explosion1 | ShapeType::Explosion2 |
            ShapeType::Ribbon { .. } | ShapeType::CurvedRibbon { .. } |
            ShapeType::Wave { .. } | ShapeType::DoubleWave { .. } |
            ShapeType::HorizontalScroll { .. } | ShapeType::VerticalScroll { .. } => {
                ShapeCategory::StarsAndBanners
            }

            // Equation shapes
            ShapeType::MathPlus | ShapeType::MathMinus | ShapeType::MathMultiply |
            ShapeType::MathDivide | ShapeType::MathEqual | ShapeType::MathNotEqual => {
                ShapeCategory::Equation
            }

            // Action buttons
            ShapeType::ActionButtonBlank | ShapeType::ActionButtonHome |
            ShapeType::ActionButtonHelp | ShapeType::ActionButtonInformation |
            ShapeType::ActionButtonBack | ShapeType::ActionButtonForward |
            ShapeType::ActionButtonBeginning | ShapeType::ActionButtonEnd |
            ShapeType::ActionButtonReturn | ShapeType::ActionButtonDocument |
            ShapeType::ActionButtonSound | ShapeType::ActionButtonMovie => {
                ShapeCategory::ActionButtons
            }

            // Custom
            ShapeType::CustomPath { .. } | ShapeType::Freeform { .. } => {
                ShapeCategory::Custom
            }
        }
    }

    /// Create a default right arrow shape
    pub fn right_arrow() -> Self {
        Self::RightArrow {
            head_width: 0.6,
            head_length: 0.35,
        }
    }

    /// Create a default left arrow shape
    pub fn left_arrow() -> Self {
        Self::LeftArrow {
            head_width: 0.6,
            head_length: 0.35,
        }
    }

    /// Create a default up arrow shape
    pub fn up_arrow() -> Self {
        Self::UpArrow {
            head_width: 0.6,
            head_length: 0.35,
        }
    }

    /// Create a default down arrow shape
    pub fn down_arrow() -> Self {
        Self::DownArrow {
            head_width: 0.6,
            head_length: 0.35,
        }
    }

    /// Create a rectangular callout with default settings
    pub fn rectangular_callout() -> Self {
        Self::RectangularCallout {
            tail_anchor: (0.5, 1.0),
            tail_tip: (0.0, 30.0),
            tail_width: 20.0,
        }
    }

    /// Create a cloud callout with default settings
    pub fn cloud_callout() -> Self {
        Self::CloudCallout {
            tail_tip: (0.0, 30.0),
            bubble_count: 3,
        }
    }
}

// =============================================================================
// Advanced Fill Types
// =============================================================================

/// Gradient stop for gradient fills
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Color at this stop
    pub color: ShapeColor,
    /// Position (0.0 to 1.0)
    pub position: f32,
    /// Transparency at this stop (0.0 = opaque, 1.0 = transparent)
    pub transparency: f32,
}

impl GradientStop {
    pub fn new(color: ShapeColor, position: f32) -> Self {
        Self {
            color,
            position,
            transparency: 0.0,
        }
    }

    pub fn with_transparency(color: ShapeColor, position: f32, transparency: f32) -> Self {
        Self {
            color,
            position,
            transparency: transparency.clamp(0.0, 1.0),
        }
    }
}

/// Pattern types for pattern fills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Solid (no pattern)
    Solid,
    /// Horizontal lines
    Horizontal,
    /// Vertical lines
    Vertical,
    /// Diagonal lines (bottom-left to top-right)
    DiagonalUp,
    /// Diagonal lines (top-left to bottom-right)
    DiagonalDown,
    /// Cross hatch
    Cross,
    /// Diagonal cross hatch
    DiagonalCross,
    /// Dotted pattern
    Dotted,
    /// Dense dotted pattern
    DottedDense,
    /// Light dotted pattern
    DottedLight,
    /// Small grid
    SmallGrid,
    /// Large grid
    LargeGrid,
    /// Small checker
    SmallChecker,
    /// Large checker
    LargeChecker,
    /// Outlined diamond
    OutlinedDiamond,
    /// Solid diamond
    SolidDiamond,
    /// Light horizontal
    LightHorizontal,
    /// Light vertical
    LightVertical,
    /// Dark horizontal
    DarkHorizontal,
    /// Dark vertical
    DarkVertical,
    /// Narrow horizontal
    NarrowHorizontal,
    /// Narrow vertical
    NarrowVertical,
    /// Dashed horizontal
    DashedHorizontal,
    /// Dashed vertical
    DashedVertical,
    /// Wave pattern
    Wave,
    /// Zigzag pattern
    Zigzag,
    /// Sphere pattern
    Sphere,
    /// Weave pattern
    Weave,
    /// Plaid pattern
    Plaid,
    /// Divot pattern
    Divot,
    /// Shingle pattern
    Shingle,
    /// Trellis pattern
    Trellis,
}

impl Default for PatternType {
    fn default() -> Self {
        Self::Solid
    }
}

/// Picture fill stretch mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PictureStretchMode {
    /// Stretch to fill (may distort)
    Stretch,
    /// Tile the image
    Tile,
    /// Fit within bounds maintaining aspect ratio
    Fit,
    /// Fill bounds maintaining aspect ratio (may crop)
    Fill,
}

impl Default for PictureStretchMode {
    fn default() -> Self {
        Self::Stretch
    }
}

/// Fill style for shapes - comprehensive fill options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeFill {
    /// No fill (transparent)
    None,
    /// Solid color fill
    Solid(ShapeColor),
    /// Linear gradient fill
    LinearGradient {
        /// Angle in degrees (0 = horizontal left-to-right)
        angle: f32,
        /// Gradient stops
        stops: Vec<GradientStop>,
        /// Whether to rotate with shape
        rotate_with_shape: bool,
    },
    /// Radial gradient fill
    RadialGradient {
        /// Center point (0.0-1.0 for x and y)
        center: Point,
        /// Gradient stops
        stops: Vec<GradientStop>,
    },
    /// Rectangular gradient fill
    RectangularGradient {
        /// Center point (0.0-1.0 for x and y)
        center: Point,
        /// Gradient stops
        stops: Vec<GradientStop>,
    },
    /// Path gradient fill (follows shape path)
    PathGradient {
        /// Gradient stops
        stops: Vec<GradientStop>,
    },
    /// Pattern fill
    Pattern {
        /// Pattern type
        pattern: PatternType,
        /// Foreground color
        foreground: ShapeColor,
        /// Background color
        background: ShapeColor,
    },
    /// Picture fill
    Picture {
        /// Reference to the image resource
        image_id: NodeId,
        /// How to stretch/tile the image
        stretch_mode: PictureStretchMode,
        /// Source rectangle for tiling (0.0-1.0)
        source_rect: Option<Rect>,
        /// Tile offset (for tiled fills)
        tile_offset: Option<Point>,
        /// Tile scale
        tile_scale: Option<Point>,
        /// Transparency (0.0-1.0)
        transparency: f32,
    },
    /// Legacy gradient format (for backwards compatibility)
    Gradient {
        /// Gradient stops (color, position 0.0-1.0)
        colors: Vec<(ShapeColor, f32)>,
        /// Gradient angle in degrees (0 = horizontal left-to-right)
        angle: f32,
    },
}

impl Default for ShapeFill {
    fn default() -> Self {
        Self::Solid(ShapeColor::BLUE)
    }
}

impl ShapeFill {
    /// Create a solid fill
    pub fn solid(color: ShapeColor) -> Self {
        Self::Solid(color)
    }

    /// Create a horizontal gradient
    pub fn horizontal_gradient(start: ShapeColor, end: ShapeColor) -> Self {
        Self::LinearGradient {
            angle: 0.0,
            stops: vec![
                GradientStop::new(start, 0.0),
                GradientStop::new(end, 1.0),
            ],
            rotate_with_shape: true,
        }
    }

    /// Create a vertical gradient
    pub fn vertical_gradient(top: ShapeColor, bottom: ShapeColor) -> Self {
        Self::LinearGradient {
            angle: 90.0,
            stops: vec![
                GradientStop::new(top, 0.0),
                GradientStop::new(bottom, 1.0),
            ],
            rotate_with_shape: true,
        }
    }

    /// Create a diagonal gradient (top-left to bottom-right)
    pub fn diagonal_gradient(start: ShapeColor, end: ShapeColor) -> Self {
        Self::LinearGradient {
            angle: 45.0,
            stops: vec![
                GradientStop::new(start, 0.0),
                GradientStop::new(end, 1.0),
            ],
            rotate_with_shape: true,
        }
    }

    /// Create a radial gradient from center
    pub fn radial_gradient(center: ShapeColor, outer: ShapeColor) -> Self {
        Self::RadialGradient {
            center: Point::new(0.5, 0.5),
            stops: vec![
                GradientStop::new(center, 0.0),
                GradientStop::new(outer, 1.0),
            ],
        }
    }

    /// Create a pattern fill
    pub fn pattern(pattern_type: PatternType, foreground: ShapeColor, background: ShapeColor) -> Self {
        Self::Pattern {
            pattern: pattern_type,
            foreground,
            background,
        }
    }

    /// Create a picture fill
    pub fn picture(image_id: NodeId) -> Self {
        Self::Picture {
            image_id,
            stretch_mode: PictureStretchMode::Stretch,
            source_rect: None,
            tile_offset: None,
            tile_scale: None,
            transparency: 0.0,
        }
    }

    /// Check if the fill is none/transparent
    pub fn is_none(&self) -> bool {
        matches!(self, ShapeFill::None)
    }
}

/// Rectangle structure for various uses
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p2.x - p1.x).abs();
        let height = (p2.y - p1.y).abs();
        Self { x, y, width, height }
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);
        Rect::new(x, y, x2 - x, y2 - y)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 100.0, 100.0)
    }
}

/// Dash style for strokes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashStyle {
    /// Solid line
    Solid,
    /// Dashed line
    Dash,
    /// Dotted line
    Dot,
    /// Dash-dot pattern
    DashDot,
    /// Dash-dot-dot pattern
    DashDotDot,
}

impl Default for DashStyle {
    fn default() -> Self {
        Self::Solid
    }
}

/// Line cap style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineCap {
    /// Flat cap
    Flat,
    /// Round cap
    Round,
    /// Square cap (extends past endpoint)
    Square,
}

impl Default for LineCap {
    fn default() -> Self {
        Self::Flat
    }
}

/// Line join style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineJoin {
    /// Miter join (sharp corners)
    Miter,
    /// Round join
    Round,
    /// Bevel join (flattened corners)
    Bevel,
}

impl Default for LineJoin {
    fn default() -> Self {
        Self::Round
    }
}

/// Stroke style for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeStroke {
    /// Stroke color
    pub color: ShapeColor,
    /// Stroke width in points
    pub width: f32,
    /// Dash style
    pub dash_style: DashStyle,
    /// Line cap style
    pub cap: LineCap,
    /// Line join style
    pub join: LineJoin,
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self {
            color: ShapeColor::BLACK,
            width: 1.0,
            dash_style: DashStyle::Solid,
            cap: LineCap::Flat,
            join: LineJoin::Round,
        }
    }
}

impl ShapeStroke {
    /// Create a simple solid stroke
    pub fn solid(color: ShapeColor, width: f32) -> Self {
        Self {
            color,
            width,
            dash_style: DashStyle::Solid,
            cap: LineCap::Flat,
            join: LineJoin::Round,
        }
    }

    /// Create a dashed stroke
    pub fn dashed(color: ShapeColor, width: f32) -> Self {
        Self {
            color,
            width,
            dash_style: DashStyle::Dash,
            cap: LineCap::Flat,
            join: LineJoin::Round,
        }
    }

    /// Create a dotted stroke
    pub fn dotted(color: ShapeColor, width: f32) -> Self {
        Self {
            color,
            width,
            dash_style: DashStyle::Dot,
            cap: LineCap::Round,
            join: LineJoin::Round,
        }
    }
}

// =============================================================================
// Advanced Effects
// =============================================================================

/// Shadow effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowType {
    /// Outer shadow (shadow outside shape)
    Outer,
    /// Inner shadow (shadow inside shape)
    Inner,
    /// Perspective shadow (3D-like shadow)
    Perspective,
}

impl Default for ShadowType {
    fn default() -> Self {
        Self::Outer
    }
}

/// Shadow preset types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowPreset {
    /// No shadow
    None,
    /// Shadow offset down-right
    OffsetDiagonal,
    /// Shadow offset down
    OffsetBottom,
    /// Shadow offset down-left
    OffsetDiagonalLeft,
    /// Shadow offset right
    OffsetRight,
    /// Shadow centered
    OffsetCenter,
    /// Perspective diagonal upper right
    PerspectiveDiagonalUpperRight,
    /// Perspective diagonal upper left
    PerspectiveDiagonalUpperLeft,
    /// Perspective below
    PerspectiveBelow,
    /// Perspective diagonal lower right
    PerspectiveDiagonalLowerRight,
    /// Perspective diagonal lower left
    PerspectiveDiagonalLowerLeft,
    /// Inner shadow
    Inner,
    /// Custom
    Custom,
}

impl Default for ShadowPreset {
    fn default() -> Self {
        Self::None
    }
}

/// Shadow effect for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowEffect {
    /// Whether shadow is enabled
    pub enabled: bool,
    /// Shadow color
    pub color: ShapeColor,
    /// Shadow type
    pub shadow_type: ShadowType,
    /// Blur radius in points
    pub blur: f32,
    /// Horizontal offset in points
    pub offset_x: f32,
    /// Vertical offset in points
    pub offset_y: f32,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Distance from shape (for perspective shadows)
    pub distance: f32,
    /// Angle in degrees (for perspective shadows)
    pub angle: f32,
    /// Scale X factor (for perspective shadows)
    pub scale_x: f32,
    /// Scale Y factor (for perspective shadows)
    pub scale_y: f32,
    /// Preset type
    pub preset: ShadowPreset,
}

impl Default for ShadowEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            color: ShapeColor::rgba(0, 0, 0, 128),
            shadow_type: ShadowType::Outer,
            blur: 4.0,
            offset_x: 2.0,
            offset_y: 2.0,
            opacity: 0.5,
            distance: 3.0,
            angle: 45.0,
            scale_x: 1.0,
            scale_y: 1.0,
            preset: ShadowPreset::OffsetDiagonal,
        }
    }
}

impl ShadowEffect {
    /// Create a simple outer shadow
    pub fn outer(offset_x: f32, offset_y: f32, blur: f32, color: ShapeColor) -> Self {
        Self {
            enabled: true,
            color,
            shadow_type: ShadowType::Outer,
            blur,
            offset_x,
            offset_y,
            opacity: 0.5,
            distance: 0.0,
            angle: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            preset: ShadowPreset::Custom,
        }
    }

    /// Create an inner shadow
    pub fn inner(offset_x: f32, offset_y: f32, blur: f32, color: ShapeColor) -> Self {
        Self {
            enabled: true,
            color,
            shadow_type: ShadowType::Inner,
            blur,
            offset_x,
            offset_y,
            opacity: 0.5,
            distance: 0.0,
            angle: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            preset: ShadowPreset::Inner,
        }
    }

    /// Create a perspective shadow
    pub fn perspective(distance: f32, angle: f32, blur: f32) -> Self {
        Self {
            enabled: true,
            color: ShapeColor::rgba(0, 0, 0, 128),
            shadow_type: ShadowType::Perspective,
            blur,
            offset_x: 0.0,
            offset_y: 0.0,
            opacity: 0.5,
            distance,
            angle,
            scale_x: 1.0,
            scale_y: 0.5,
            preset: ShadowPreset::PerspectiveBelow,
        }
    }
}

/// Lighting type for 3D effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingType {
    /// No lighting
    None,
    /// Flat lighting
    Flat,
    /// Soft lighting
    Soft,
    /// Harsh lighting
    Harsh,
    /// Sunrise (warm light from the right)
    Sunrise,
    /// Sunset (warm light from the left)
    Sunset,
    /// Morning (cool light)
    Morning,
    /// Evening (warm light)
    Evening,
    /// Chilly (blue-tinted)
    Chilly,
    /// Freezing (blue-tinted, harsh)
    Freezing,
    /// Flood (bright uniform)
    Flood,
    /// Contrasting (dramatic)
    Contrasting,
    /// Three-point lighting
    ThreePoint,
    /// Balanced lighting
    Balanced,
    /// Bright room
    BrightRoom,
    /// Soft box
    SoftBox,
}

impl Default for LightingType {
    fn default() -> Self {
        Self::None
    }
}

/// Light rig direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightDirection {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Default for LightDirection {
    fn default() -> Self {
        Self::TopLeft
    }
}

/// Bevel type for 3D effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BevelType {
    /// No bevel
    None,
    /// Circle bevel
    Circle,
    /// Relaxed inset
    RelaxedInset,
    /// Cross bevel
    Cross,
    /// Cool slant
    CoolSlant,
    /// Angle bevel
    Angle,
    /// Soft round
    SoftRound,
    /// Convex
    Convex,
    /// Slope
    Slope,
    /// Divot
    Divot,
    /// Riblet
    Riblet,
    /// Hard edge
    HardEdge,
    /// Art deco
    ArtDeco,
}

impl Default for BevelType {
    fn default() -> Self {
        Self::None
    }
}

/// Bevel effect settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BevelEffect {
    /// Bevel type
    pub bevel_type: BevelType,
    /// Width in points
    pub width: f32,
    /// Height in points
    pub height: f32,
}

impl Default for BevelEffect {
    fn default() -> Self {
        Self {
            bevel_type: BevelType::None,
            width: 6.0,
            height: 6.0,
        }
    }
}

/// 3D effect for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect3D {
    /// Whether 3D effect is enabled
    pub enabled: bool,
    /// Depth/extrusion in points
    pub depth: f32,
    /// Extrusion color
    pub extrusion_color: ShapeColor,
    /// Lighting type
    pub lighting: LightingType,
    /// Light direction
    pub light_direction: LightDirection,
    /// Rotation angle X (degrees)
    pub rotation_x: f32,
    /// Rotation angle Y (degrees)
    pub rotation_y: f32,
    /// Rotation angle Z (degrees)
    pub rotation_z: f32,
    /// Perspective/field of view (degrees)
    pub perspective: f32,
    /// Top bevel
    pub bevel_top: BevelEffect,
    /// Bottom bevel
    pub bevel_bottom: BevelEffect,
    /// Material type for surface
    pub material: MaterialType,
    /// Contour color (if different from fill)
    pub contour_color: Option<ShapeColor>,
    /// Contour width
    pub contour_width: f32,
}

impl Default for Effect3D {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 0.0,
            extrusion_color: ShapeColor::GRAY,
            lighting: LightingType::None,
            light_direction: LightDirection::TopLeft,
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            perspective: 0.0,
            bevel_top: BevelEffect::default(),
            bevel_bottom: BevelEffect::default(),
            material: MaterialType::default(),
            contour_color: None,
            contour_width: 0.0,
        }
    }
}

impl Effect3D {
    /// Create a simple extrusion effect
    pub fn extrusion(depth: f32, color: ShapeColor) -> Self {
        Self {
            enabled: true,
            depth,
            extrusion_color: color,
            lighting: LightingType::ThreePoint,
            light_direction: LightDirection::TopLeft,
            ..Default::default()
        }
    }

    /// Create a bevel effect
    pub fn bevel(bevel_type: BevelType, width: f32, height: f32) -> Self {
        Self {
            enabled: true,
            bevel_top: BevelEffect {
                bevel_type,
                width,
                height,
            },
            lighting: LightingType::ThreePoint,
            ..Default::default()
        }
    }
}

/// Material type for 3D surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    /// Matte (diffuse)
    Matte,
    /// Warm matte
    WarmMatte,
    /// Plastic
    Plastic,
    /// Metal
    Metal,
    /// Dark edge
    DarkEdge,
    /// Soft edge
    SoftEdge,
    /// Flat
    Flat,
    /// Wire frame
    WireFrame,
    /// Powder
    Powder,
    /// Translucent powder
    TranslucentPowder,
    /// Clear
    Clear,
}

impl Default for MaterialType {
    fn default() -> Self {
        Self::Matte
    }
}

/// Glow effect for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlowEffect {
    /// Whether glow is enabled
    pub enabled: bool,
    /// Glow color
    pub color: ShapeColor,
    /// Glow radius in points
    pub radius: f32,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
}

impl Default for GlowEffect {
    fn default() -> Self {
        Self {
            enabled: false,
            color: ShapeColor::YELLOW,
            radius: 10.0,
            opacity: 0.5,
        }
    }
}

/// Soft edge effect for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftEdgeEffect {
    /// Whether soft edge is enabled
    pub enabled: bool,
    /// Radius in points
    pub radius: f32,
}

impl Default for SoftEdgeEffect {
    fn default() -> Self {
        Self {
            enabled: false,
            radius: 5.0,
        }
    }
}

/// Reflection effect for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionEffect {
    /// Whether reflection is enabled
    pub enabled: bool,
    /// Blur radius
    pub blur: f32,
    /// Starting opacity (0.0 to 1.0)
    pub start_opacity: f32,
    /// Ending opacity (0.0 to 1.0)
    pub end_opacity: f32,
    /// Starting position (0.0 to 1.0)
    pub start_position: f32,
    /// Ending position (0.0 to 1.0)
    pub end_position: f32,
    /// Vertical offset from shape
    pub offset: f32,
    /// Scale Y factor
    pub scale_y: f32,
    /// Rotation (degrees)
    pub angle: f32,
}

impl Default for ReflectionEffect {
    fn default() -> Self {
        Self {
            enabled: false,
            blur: 0.0,
            start_opacity: 0.5,
            end_opacity: 0.0,
            start_position: 0.0,
            end_position: 0.5,
            offset: 0.0,
            scale_y: -1.0,
            angle: 0.0,
        }
    }
}

/// Visual effects for shapes - comprehensive effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeEffects {
    /// Shadow effect
    pub shadow: Option<ShadowEffect>,
    /// 3D effect
    pub effect_3d: Option<Effect3D>,
    /// Glow effect
    pub glow: Option<GlowEffect>,
    /// Soft edge effect
    pub soft_edge: Option<SoftEdgeEffect>,
    /// Reflection effect
    pub reflection: Option<ReflectionEffect>,
    /// Overall opacity (0.0 = fully transparent, 1.0 = fully opaque)
    pub opacity: f32,
}

impl Default for ShapeEffects {
    fn default() -> Self {
        Self {
            shadow: None,
            effect_3d: None,
            glow: None,
            soft_edge: None,
            reflection: None,
            opacity: 1.0,
        }
    }
}

impl ShapeEffects {
    /// Create effects with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create effects with shadow
    pub fn with_shadow() -> Self {
        Self {
            shadow: Some(ShadowEffect::default()),
            ..Default::default()
        }
    }

    /// Create effects with 3D
    pub fn with_3d(depth: f32) -> Self {
        Self {
            effect_3d: Some(Effect3D::extrusion(depth, ShapeColor::GRAY)),
            ..Default::default()
        }
    }

    /// Create effects with glow
    pub fn with_glow(color: ShapeColor, radius: f32) -> Self {
        Self {
            glow: Some(GlowEffect {
                enabled: true,
                color,
                radius,
                opacity: 0.5,
            }),
            ..Default::default()
        }
    }

    /// Create effects with reflection
    pub fn with_reflection() -> Self {
        Self {
            reflection: Some(ReflectionEffect {
                enabled: true,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

// =============================================================================
// Shape Text
// =============================================================================

/// Anchor point for text within a shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAnchor {
    /// Anchor at top
    Top,
    /// Anchor at middle (vertical center)
    Middle,
    /// Anchor at bottom
    Bottom,
}

impl Default for TextAnchor {
    fn default() -> Self {
        Self::Middle
    }
}

/// Auto-fit behavior for text in shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAutoFit {
    /// No auto-fit, text may overflow
    None,
    /// Shrink text size to fit within shape
    ShrinkText,
    /// Resize shape to fit text content
    ResizeShape,
}

impl Default for TextAutoFit {
    fn default() -> Self {
        Self::None
    }
}

/// Vertical alignment for text in shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeTextVerticalAlign {
    Top,
    Center,
    Bottom,
    /// Justify text vertically (distribute space)
    Justify,
    /// Justify with low priority
    JustifyLow,
}

impl Default for ShapeTextVerticalAlign {
    fn default() -> Self {
        Self::Center
    }
}

/// Text direction within a shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeTextDirection {
    /// Horizontal left-to-right
    Horizontal,
    /// Horizontal right-to-left
    HorizontalRtl,
    /// Vertical (top to bottom, right to left columns)
    Vertical,
    /// Vertical (top to bottom, left to right columns)
    VerticalLtr,
    /// Stacked (letters stacked vertically)
    Stacked,
}

impl Default for ShapeTextDirection {
    fn default() -> Self {
        Self::Horizontal
    }
}

/// Text wrapping mode within shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeTextWrap {
    /// No wrapping, text extends beyond shape
    None,
    /// Wrap text at shape boundary
    Square,
}

impl Default for ShapeTextWrap {
    fn default() -> Self {
        Self::Square
    }
}

/// Margins for text within a shape
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShapeTextMargins {
    /// Top margin in points
    pub top: f32,
    /// Right margin in points
    pub right: f32,
    /// Bottom margin in points
    pub bottom: f32,
    /// Left margin in points
    pub left: f32,
}

impl Default for ShapeTextMargins {
    fn default() -> Self {
        Self {
            top: 3.6,   // ~0.05 inch
            right: 7.2, // ~0.1 inch
            bottom: 3.6,
            left: 7.2,
        }
    }
}

impl ShapeTextMargins {
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn none() -> Self {
        Self::uniform(0.0)
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Text content configuration for shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeText {
    /// Content blocks (paragraph IDs)
    pub content: Vec<NodeId>,
    /// Text anchor position
    pub anchor: TextAnchor,
    /// Auto-fit behavior
    pub auto_fit: TextAutoFit,
    /// Internal margins
    pub margins: ShapeTextMargins,
    /// Vertical alignment
    pub vertical_align: ShapeTextVerticalAlign,
    /// Text direction
    pub direction: ShapeTextDirection,
    /// Text wrapping mode
    pub wrap: ShapeTextWrap,
    /// Rotation angle for text (independent of shape rotation)
    pub rotation: f32,
    /// Upright text when shape is vertical
    pub upright: bool,
    /// Number of columns for text
    pub columns: u8,
    /// Space between columns in points
    pub column_spacing: f32,
}

impl Default for ShapeText {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            anchor: TextAnchor::Middle,
            auto_fit: TextAutoFit::None,
            margins: ShapeTextMargins::default(),
            vertical_align: ShapeTextVerticalAlign::Center,
            direction: ShapeTextDirection::Horizontal,
            wrap: ShapeTextWrap::Square,
            rotation: 0.0,
            upright: false,
            columns: 1,
            column_spacing: 0.0,
        }
    }
}

impl ShapeText {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create shape text with auto-shrink
    pub fn with_shrink_text() -> Self {
        Self {
            auto_fit: TextAutoFit::ShrinkText,
            ..Default::default()
        }
    }

    /// Create shape text with auto-resize
    pub fn with_resize_shape() -> Self {
        Self {
            auto_fit: TextAutoFit::ResizeShape,
            ..Default::default()
        }
    }

    /// Add a content paragraph
    pub fn add_content(&mut self, para_id: NodeId) {
        self.content.push(para_id);
    }

    /// Check if there is any content
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Clear all content
    pub fn clear(&mut self) {
        self.content.clear();
    }
}

// =============================================================================
// Shape Groups
// =============================================================================

/// A group of shapes that behave as a single unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeGroup {
    /// Unique ID for the group
    pub id: NodeId,
    /// IDs of shapes in this group (can include nested groups)
    pub shapes: Vec<NodeId>,
    /// Combined bounding box of all shapes
    pub bounds: Rect,
    /// Optional name for the group
    pub name: Option<String>,
    /// Whether the group is locked for editing
    pub locked: bool,
}

impl ShapeGroup {
    /// Create a new empty shape group
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            shapes: Vec::new(),
            bounds: Rect::default(),
            name: None,
            locked: false,
        }
    }

    /// Create a group from a list of shape IDs
    pub fn from_shapes(shapes: Vec<NodeId>) -> Self {
        Self {
            id: NodeId::new(),
            shapes,
            bounds: Rect::default(),
            name: None,
            locked: false,
        }
    }

    /// Add a shape to the group
    pub fn add_shape(&mut self, shape_id: NodeId) {
        if !self.shapes.contains(&shape_id) {
            self.shapes.push(shape_id);
        }
    }

    /// Remove a shape from the group
    pub fn remove_shape(&mut self, shape_id: NodeId) -> bool {
        if let Some(pos) = self.shapes.iter().position(|&id| id == shape_id) {
            self.shapes.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if the group contains a specific shape
    pub fn contains(&self, shape_id: NodeId) -> bool {
        self.shapes.contains(&shape_id)
    }

    /// Check if the group is empty
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Get the number of shapes in the group
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Update the bounding box based on shape bounds
    pub fn update_bounds(&mut self, shape_bounds: &[(NodeId, Rect)]) {
        if let Some((_, first)) = shape_bounds.first() {
            let mut bounds = *first;
            for (id, rect) in shape_bounds.iter().skip(1) {
                if self.shapes.contains(id) {
                    bounds = bounds.union(rect);
                }
            }
            self.bounds = bounds;
        }
    }
}

impl Default for ShapeGroup {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Connectors
// =============================================================================

/// Connection point on a shape for connectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionPoint {
    /// Top center
    Top,
    /// Bottom center
    Bottom,
    /// Left center
    Left,
    /// Right center
    Right,
    /// Top-left corner
    TopLeft,
    /// Top-right corner
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
    /// Center of shape
    Center,
}

impl Default for ConnectionPoint {
    fn default() -> Self {
        Self::Right
    }
}

impl ConnectionPoint {
    /// Get the normalized position (0.0-1.0) for this connection point
    pub fn normalized_position(&self) -> (f32, f32) {
        match self {
            ConnectionPoint::Top => (0.5, 0.0),
            ConnectionPoint::Bottom => (0.5, 1.0),
            ConnectionPoint::Left => (0.0, 0.5),
            ConnectionPoint::Right => (1.0, 0.5),
            ConnectionPoint::TopLeft => (0.0, 0.0),
            ConnectionPoint::TopRight => (1.0, 0.0),
            ConnectionPoint::BottomLeft => (0.0, 1.0),
            ConnectionPoint::BottomRight => (1.0, 1.0),
            ConnectionPoint::Center => (0.5, 0.5),
        }
    }
}

/// Custom connection point with explicit position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectorEndpoint {
    /// Connect to a predefined point on a shape
    ShapeConnection {
        shape_id: NodeId,
        point: ConnectionPoint,
    },
    /// Connect to a custom position on a shape
    ShapeCustom {
        shape_id: NodeId,
        /// Position relative to shape (0.0-1.0)
        position: (f32, f32),
    },
    /// Floating point (not connected to a shape)
    Floating(Point),
}

impl Default for ConnectorEndpoint {
    fn default() -> Self {
        Self::Floating(Point::ORIGIN)
    }
}

/// Routing style for connectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorRouting {
    /// Straight line
    Straight,
    /// Elbow/orthogonal routing (right angles only)
    Elbow,
    /// Curved/smooth routing
    Curved,
}

impl Default for ConnectorRouting {
    fn default() -> Self {
        Self::Straight
    }
}

/// Arrow head style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowHead {
    /// No arrow head
    None,
    /// Triangle arrow
    Triangle,
    /// Stealth arrow (narrow triangle)
    Stealth,
    /// Diamond arrow
    Diamond,
    /// Oval arrow
    Oval,
    /// Open arrow (unfilled triangle)
    Open,
}

impl Default for ArrowHead {
    fn default() -> Self {
        Self::None
    }
}

/// Arrow size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowSize {
    Small,
    Medium,
    Large,
}

impl Default for ArrowSize {
    fn default() -> Self {
        Self::Medium
    }
}

/// Arrow configuration for line/connector ends
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrowConfig {
    /// Start arrow head type
    pub start_type: ArrowHead,
    /// Start arrow size
    pub start_size: ArrowSize,
    /// End arrow head type
    pub end_type: ArrowHead,
    /// End arrow size
    pub end_size: ArrowSize,
}

impl Default for ArrowConfig {
    fn default() -> Self {
        Self {
            start_type: ArrowHead::None,
            start_size: ArrowSize::Medium,
            end_type: ArrowHead::Triangle,
            end_size: ArrowSize::Medium,
        }
    }
}

impl ArrowConfig {
    /// Create config with no arrows
    pub fn none() -> Self {
        Self {
            start_type: ArrowHead::None,
            start_size: ArrowSize::Medium,
            end_type: ArrowHead::None,
            end_size: ArrowSize::Medium,
        }
    }

    /// Create config with arrow at end only
    pub fn end_arrow(arrow_type: ArrowHead) -> Self {
        Self {
            start_type: ArrowHead::None,
            start_size: ArrowSize::Medium,
            end_type: arrow_type,
            end_size: ArrowSize::Medium,
        }
    }

    /// Create config with arrows at both ends
    pub fn both_arrows(arrow_type: ArrowHead) -> Self {
        Self {
            start_type: arrow_type,
            start_size: ArrowSize::Medium,
            end_type: arrow_type,
            end_size: ArrowSize::Medium,
        }
    }
}

/// A connector line between shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    /// Unique ID
    pub id: NodeId,
    /// Parent node ID
    parent: Option<NodeId>,
    /// Start endpoint
    pub start: ConnectorEndpoint,
    /// End endpoint
    pub end: ConnectorEndpoint,
    /// Routing style
    pub routing: ConnectorRouting,
    /// Line style
    pub line_style: ShapeStroke,
    /// Arrow configuration
    pub arrows: ArrowConfig,
    /// Adjustment handles for elbow/curved connectors
    pub adjustments: Vec<f32>,
    /// Optional name
    pub name: Option<String>,
}

impl Connector {
    /// Create a new connector
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            start: ConnectorEndpoint::default(),
            end: ConnectorEndpoint::default(),
            routing: ConnectorRouting::Straight,
            line_style: ShapeStroke::default(),
            arrows: ArrowConfig::default(),
            adjustments: Vec::new(),
            name: None,
        }
    }

    /// Create a straight connector between two shapes
    pub fn straight(
        start_shape: NodeId,
        start_point: ConnectionPoint,
        end_shape: NodeId,
        end_point: ConnectionPoint,
    ) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            start: ConnectorEndpoint::ShapeConnection {
                shape_id: start_shape,
                point: start_point,
            },
            end: ConnectorEndpoint::ShapeConnection {
                shape_id: end_shape,
                point: end_point,
            },
            routing: ConnectorRouting::Straight,
            line_style: ShapeStroke::default(),
            arrows: ArrowConfig::end_arrow(ArrowHead::Triangle),
            adjustments: Vec::new(),
            name: None,
        }
    }

    /// Create an elbow connector between two shapes
    pub fn elbow(
        start_shape: NodeId,
        start_point: ConnectionPoint,
        end_shape: NodeId,
        end_point: ConnectionPoint,
    ) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            start: ConnectorEndpoint::ShapeConnection {
                shape_id: start_shape,
                point: start_point,
            },
            end: ConnectorEndpoint::ShapeConnection {
                shape_id: end_shape,
                point: end_point,
            },
            routing: ConnectorRouting::Elbow,
            line_style: ShapeStroke::default(),
            arrows: ArrowConfig::end_arrow(ArrowHead::Triangle),
            adjustments: vec![0.5], // Default midpoint
            name: None,
        }
    }

    /// Create a curved connector between two shapes
    pub fn curved(
        start_shape: NodeId,
        start_point: ConnectionPoint,
        end_shape: NodeId,
        end_point: ConnectionPoint,
    ) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            start: ConnectorEndpoint::ShapeConnection {
                shape_id: start_shape,
                point: start_point,
            },
            end: ConnectorEndpoint::ShapeConnection {
                shape_id: end_shape,
                point: end_point,
            },
            routing: ConnectorRouting::Curved,
            line_style: ShapeStroke::default(),
            arrows: ArrowConfig::end_arrow(ArrowHead::Triangle),
            adjustments: vec![0.5, 0.5], // Control point factors
            name: None,
        }
    }

    /// Check if this connector is connected to a specific shape
    pub fn is_connected_to(&self, shape_id: NodeId) -> bool {
        let start_connected = match &self.start {
            ConnectorEndpoint::ShapeConnection { shape_id: id, .. } => *id == shape_id,
            ConnectorEndpoint::ShapeCustom { shape_id: id, .. } => *id == shape_id,
            ConnectorEndpoint::Floating(_) => false,
        };
        let end_connected = match &self.end {
            ConnectorEndpoint::ShapeConnection { shape_id: id, .. } => *id == shape_id,
            ConnectorEndpoint::ShapeCustom { shape_id: id, .. } => *id == shape_id,
            ConnectorEndpoint::Floating(_) => false,
        };
        start_connected || end_connected
    }
}

impl Default for Connector {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Connector {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Shape // Connectors are a type of shape
    }

    fn children(&self) -> &[NodeId] {
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

// =============================================================================
// Shape Alignment and Distribution
// =============================================================================

/// Horizontal alignment option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HorizontalAlignment {
    /// Align left edges
    Left,
    /// Align centers horizontally
    Center,
    /// Align right edges
    Right,
}

/// Vertical alignment option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlignment {
    /// Align top edges
    Top,
    /// Align centers vertically
    Middle,
    /// Align bottom edges
    Bottom,
}

/// Distribution option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributeDirection {
    /// Distribute horizontally (equal spacing between shapes)
    Horizontal,
    /// Distribute vertically (equal spacing between shapes)
    Vertical,
}

/// Reference for alignment/distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentReference {
    /// Align to the selected shapes
    Selection,
    /// Align to the page
    Page,
    /// Align to the margin
    Margin,
}

/// Z-order operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZOrderOperation {
    /// Bring to front (topmost)
    BringToFront,
    /// Send to back (bottommost)
    SendToBack,
    /// Bring forward one level
    BringForward,
    /// Send backward one level
    SendBackward,
}

// =============================================================================
// Shape Properties
// =============================================================================

/// Properties controlling shape appearance and layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeProperties {
    /// Width of the shape
    pub width: Dimension,
    /// Height of the shape
    pub height: Dimension,
    /// How text wraps around the shape (reuses image wrap types)
    pub wrap_type: WrapType,
    /// Position type and anchor settings (reuses image positioning)
    pub position: ImagePosition,
    /// Rotation in degrees (clockwise)
    pub rotation: f32,
    /// Fill style
    pub fill: Option<ShapeFill>,
    /// Stroke style
    pub stroke: Option<ShapeStroke>,
    /// Visual effects
    pub effects: ShapeEffects,
    /// Whether to maintain aspect ratio during resize
    pub lock_aspect_ratio: bool,
    /// Whether the shape is flipped horizontally
    pub flip_horizontal: bool,
    /// Whether the shape is flipped vertically
    pub flip_vertical: bool,
}

impl Default for ShapeProperties {
    fn default() -> Self {
        Self {
            width: Dimension::points(100.0),
            height: Dimension::points(100.0),
            wrap_type: WrapType::InFront,
            position: ImagePosition::Inline,
            rotation: 0.0,
            fill: Some(ShapeFill::default()),
            stroke: Some(ShapeStroke::default()),
            effects: ShapeEffects::new(),
            lock_aspect_ratio: false,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

impl ShapeProperties {
    /// Create new properties with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create properties for an inline shape
    pub fn inline(width: f32, height: f32) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
            wrap_type: WrapType::Inline,
            position: ImagePosition::Inline,
            ..Default::default()
        }
    }

    /// Create properties for a floating shape
    pub fn floating(width: f32, height: f32, wrap_type: WrapType) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
            wrap_type,
            position: ImagePosition::Anchor(Default::default()),
            ..Default::default()
        }
    }

    /// Create properties for a line shape
    pub fn line(width: f32, height: f32) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
            wrap_type: WrapType::InFront,
            position: ImagePosition::Anchor(Default::default()),
            fill: None,
            stroke: Some(ShapeStroke::solid(ShapeColor::BLACK, 2.0)),
            ..Default::default()
        }
    }

    /// Create properties for a text box
    pub fn text_box(width: f32, height: f32) -> Self {
        Self {
            width: Dimension::points(width),
            height: Dimension::points(height),
            wrap_type: WrapType::Square,
            position: ImagePosition::Anchor(Default::default()),
            fill: Some(ShapeFill::Solid(ShapeColor::WHITE)),
            stroke: Some(ShapeStroke::solid(ShapeColor::BLACK, 1.0)),
            ..Default::default()
        }
    }
}

/// A shape node in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeNode {
    /// Unique node ID
    id: NodeId,
    /// Parent node ID
    parent: Option<NodeId>,
    /// Type of shape
    pub shape_type: ShapeType,
    /// Shape properties
    pub properties: ShapeProperties,
    /// Optional text content inside the shape (NodeId of a paragraph) - legacy
    pub text_content: Option<NodeId>,
    /// Advanced text content configuration
    pub shape_text: Option<ShapeText>,
    /// Optional name/title for the shape
    pub name: Option<String>,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
    /// Group this shape belongs to (if any)
    pub group_id: Option<NodeId>,
    /// Z-order index (higher = more in front)
    pub z_order: i32,
    /// Whether the shape is locked for editing
    pub locked: bool,
    /// Whether the shape is hidden
    pub hidden: bool,
}

impl ShapeNode {
    /// Create a new shape node
    pub fn new(shape_type: ShapeType) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            shape_type,
            properties: ShapeProperties::default(),
            text_content: None,
            shape_text: None,
            name: None,
            alt_text: None,
            group_id: None,
            z_order: 0,
            locked: false,
            hidden: false,
        }
    }

    /// Create a new shape with specific size
    pub fn with_size(shape_type: ShapeType, width: f32, height: f32) -> Self {
        let mut node = Self::new(shape_type);
        node.properties.width = Dimension::points(width);
        node.properties.height = Dimension::points(height);
        node
    }

    /// Create a rectangle shape
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::with_size(ShapeType::Rectangle, width, height)
    }

    /// Create a rounded rectangle shape
    pub fn rounded_rectangle(width: f32, height: f32, corner_radius: f32) -> Self {
        Self::with_size(
            ShapeType::RoundedRectangle { corner_radius },
            width,
            height,
        )
    }

    /// Create an oval shape
    pub fn oval(width: f32, height: f32) -> Self {
        Self::with_size(ShapeType::Oval, width, height)
    }

    /// Create a line shape
    pub fn line(width: f32, height: f32) -> Self {
        let mut node = Self::with_size(ShapeType::Line, width, height);
        node.properties = ShapeProperties::line(width, height);
        node
    }

    /// Create an arrow shape
    pub fn arrow(width: f32, height: f32) -> Self {
        let mut node = Self::with_size(ShapeType::Arrow, width, height);
        node.properties = ShapeProperties::line(width, height);
        node
    }

    /// Create a triangle shape
    pub fn triangle(width: f32, height: f32) -> Self {
        Self::with_size(ShapeType::Triangle, width, height)
    }

    /// Create a star shape
    pub fn star(width: f32, height: f32, points: u8) -> Self {
        Self::with_size(
            ShapeType::Star {
                points: points.clamp(5, 12),
                inner_radius_ratio: 0.4,
            },
            width,
            height,
        )
    }

    /// Create a text box shape
    pub fn text_box(width: f32, height: f32) -> Self {
        let mut node = Self::with_size(ShapeType::TextBox, width, height);
        node.properties = ShapeProperties::text_box(width, height);
        node
    }

    /// Create a callout shape
    pub fn callout(width: f32, height: f32, tail_x: f32, tail_y: f32) -> Self {
        Self::with_size(
            ShapeType::Callout {
                tail_position: (tail_x, tail_y),
                tail_width: 20.0,
            },
            width,
            height,
        )
    }

    /// Set the shape properties
    pub fn set_properties(&mut self, properties: ShapeProperties) {
        self.properties = properties;
    }

    /// Set the fill
    pub fn set_fill(&mut self, fill: Option<ShapeFill>) {
        self.properties.fill = fill;
    }

    /// Set the stroke
    pub fn set_stroke(&mut self, stroke: Option<ShapeStroke>) {
        self.properties.stroke = stroke;
    }

    /// Set the rotation
    pub fn set_rotation(&mut self, rotation: f32) {
        self.properties.rotation = rotation;
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Set the alt text
    pub fn set_alt_text(&mut self, alt_text: impl Into<String>) {
        self.alt_text = Some(alt_text.into());
    }

    /// Set the text content (for shapes that can contain text)
    pub fn set_text_content(&mut self, para_id: NodeId) {
        if self.shape_type.can_contain_text() {
            self.text_content = Some(para_id);
        }
    }

    /// Get the effective width in points
    pub fn effective_width(&self, container_width: f32) -> f32 {
        self.properties.width.resolve(container_width).unwrap_or(100.0)
    }

    /// Get the effective height in points
    pub fn effective_height(&self, container_height: f32) -> f32 {
        self.properties.height.resolve(container_height).unwrap_or(100.0)
    }

    /// Check if this is an inline shape
    pub fn is_inline(&self) -> bool {
        matches!(self.properties.wrap_type, WrapType::Inline)
    }

    /// Check if this is a floating shape
    pub fn is_floating(&self) -> bool {
        !self.is_inline()
    }
}

impl Node for ShapeNode {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Shape
    }

    fn children(&self) -> &[NodeId] {
        // Shapes don't have children in the traditional sense
        // Text content is stored as a reference
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

    // =========================================================================
    // Basic Shape Tests
    // =========================================================================

    #[test]
    fn test_shape_creation() {
        let shape = ShapeNode::rectangle(100.0, 50.0);
        assert!(matches!(shape.shape_type, ShapeType::Rectangle));
        assert_eq!(shape.effective_width(500.0), 100.0);
        assert_eq!(shape.effective_height(500.0), 50.0);
    }

    #[test]
    fn test_shape_type_display_name() {
        assert_eq!(ShapeType::Rectangle.display_name(), "Rectangle");
        assert_eq!(ShapeType::Line.display_name(), "Line");
        assert_eq!(
            ShapeType::RoundedRectangle { corner_radius: 10.0 }.display_name(),
            "Rounded Rectangle"
        );
    }

    #[test]
    fn test_line_shapes() {
        let line = ShapeNode::line(100.0, 0.0);
        assert!(line.shape_type.is_line());
        assert!(!line.shape_type.can_contain_text());
        assert!(line.properties.fill.is_none());

        let arrow = ShapeNode::arrow(100.0, 0.0);
        assert!(arrow.shape_type.is_line());
    }

    #[test]
    fn test_text_box() {
        let text_box = ShapeNode::text_box(200.0, 100.0);
        assert!(matches!(text_box.shape_type, ShapeType::TextBox));
        assert!(text_box.shape_type.can_contain_text());
    }

    #[test]
    fn test_star_shape() {
        let star = ShapeNode::star(100.0, 100.0, 5);
        if let ShapeType::Star { points, .. } = star.shape_type {
            assert_eq!(points, 5);
        } else {
            panic!("Expected Star shape type");
        }
    }

    #[test]
    fn test_shape_properties() {
        let mut shape = ShapeNode::rectangle(100.0, 100.0);

        shape.set_fill(Some(ShapeFill::Solid(ShapeColor::RED)));
        assert!(matches!(shape.properties.fill, Some(ShapeFill::Solid(_))));

        shape.set_stroke(None);
        assert!(shape.properties.stroke.is_none());

        shape.set_rotation(45.0);
        assert_eq!(shape.properties.rotation, 45.0);
    }

    // =========================================================================
    // Color Tests
    // =========================================================================

    #[test]
    fn test_shape_color_hex() {
        let color = ShapeColor::rgb(255, 128, 64);
        assert_eq!(color.to_hex(), "#FF8040");

        let color_alpha = ShapeColor::rgba(255, 128, 64, 128);
        assert_eq!(color_alpha.to_hex(), "#FF804080");
    }

    #[test]
    fn test_shape_color_from_hex() {
        let color = ShapeColor::from_hex("#FF8040").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);

        let color_alpha = ShapeColor::from_hex("#FF804080").unwrap();
        assert_eq!(color_alpha.a, 128);

        assert!(ShapeColor::from_hex("invalid").is_none());
        assert!(ShapeColor::from_hex("#FFF").is_none());
    }

    // =========================================================================
    // Fill Tests
    // =========================================================================

    #[test]
    fn test_shape_fill() {
        let solid = ShapeFill::solid(ShapeColor::BLUE);
        assert!(!solid.is_none());

        let none = ShapeFill::None;
        assert!(none.is_none());

        let gradient = ShapeFill::horizontal_gradient(ShapeColor::WHITE, ShapeColor::BLUE);
        assert!(!gradient.is_none());
    }

    #[test]
    fn test_linear_gradient() {
        let gradient = ShapeFill::LinearGradient {
            angle: 45.0,
            stops: vec![
                GradientStop::new(ShapeColor::RED, 0.0),
                GradientStop::new(ShapeColor::YELLOW, 0.5),
                GradientStop::new(ShapeColor::GREEN, 1.0),
            ],
            rotate_with_shape: true,
        };

        if let ShapeFill::LinearGradient { angle, stops, .. } = gradient {
            assert_eq!(angle, 45.0);
            assert_eq!(stops.len(), 3);
        } else {
            panic!("Expected LinearGradient");
        }
    }

    #[test]
    fn test_radial_gradient() {
        let gradient = ShapeFill::radial_gradient(ShapeColor::WHITE, ShapeColor::BLUE);

        if let ShapeFill::RadialGradient { center, stops } = gradient {
            assert_eq!(center.x, 0.5);
            assert_eq!(center.y, 0.5);
            assert_eq!(stops.len(), 2);
        } else {
            panic!("Expected RadialGradient");
        }
    }

    #[test]
    fn test_pattern_fill() {
        let pattern = ShapeFill::pattern(
            PatternType::DiagonalCross,
            ShapeColor::BLACK,
            ShapeColor::WHITE,
        );

        if let ShapeFill::Pattern { pattern: pat, foreground, background } = pattern {
            assert_eq!(pat, PatternType::DiagonalCross);
            assert_eq!(foreground, ShapeColor::BLACK);
            assert_eq!(background, ShapeColor::WHITE);
        } else {
            panic!("Expected Pattern fill");
        }
    }

    // =========================================================================
    // Stroke Tests
    // =========================================================================

    #[test]
    fn test_shape_stroke() {
        let stroke = ShapeStroke::solid(ShapeColor::BLACK, 2.0);
        assert_eq!(stroke.width, 2.0);
        assert!(matches!(stroke.dash_style, DashStyle::Solid));

        let dashed = ShapeStroke::dashed(ShapeColor::RED, 1.5);
        assert!(matches!(dashed.dash_style, DashStyle::Dash));
    }

    // =========================================================================
    // Effect Tests
    // =========================================================================

    #[test]
    fn test_shadow_effect() {
        let shadow = ShadowEffect::default();
        assert!(shadow.enabled);
        assert_eq!(shadow.shadow_type, ShadowType::Outer);
        assert!(shadow.blur > 0.0);

        let inner = ShadowEffect::inner(2.0, 2.0, 4.0, ShapeColor::BLACK);
        assert_eq!(inner.shadow_type, ShadowType::Inner);

        let perspective = ShadowEffect::perspective(10.0, 45.0, 5.0);
        assert_eq!(perspective.shadow_type, ShadowType::Perspective);
    }

    #[test]
    fn test_3d_effect() {
        let effect = Effect3D::extrusion(10.0, ShapeColor::GRAY);
        assert!(effect.enabled);
        assert_eq!(effect.depth, 10.0);
        assert_eq!(effect.lighting, LightingType::ThreePoint);

        let bevel = Effect3D::bevel(BevelType::Circle, 5.0, 5.0);
        assert!(bevel.enabled);
        assert_eq!(bevel.bevel_top.bevel_type, BevelType::Circle);
    }

    #[test]
    fn test_shape_effects() {
        let effects = ShapeEffects::with_shadow();
        assert!(effects.shadow.is_some());
        assert!(effects.effect_3d.is_none());

        let effects_3d = ShapeEffects::with_3d(5.0);
        assert!(effects_3d.effect_3d.is_some());

        let effects_glow = ShapeEffects::with_glow(ShapeColor::YELLOW, 10.0);
        assert!(effects_glow.glow.is_some());
    }

    // =========================================================================
    // Shape Text Tests
    // =========================================================================

    #[test]
    fn test_shape_text() {
        let mut text = ShapeText::new();
        assert!(text.is_empty());

        let para_id = NodeId::new();
        text.add_content(para_id);
        assert!(!text.is_empty());
        assert_eq!(text.content.len(), 1);

        text.clear();
        assert!(text.is_empty());
    }

    #[test]
    fn test_shape_text_auto_fit() {
        let shrink = ShapeText::with_shrink_text();
        assert_eq!(shrink.auto_fit, TextAutoFit::ShrinkText);

        let resize = ShapeText::with_resize_shape();
        assert_eq!(resize.auto_fit, TextAutoFit::ResizeShape);
    }

    #[test]
    fn test_shape_text_margins() {
        let margins = ShapeTextMargins::uniform(10.0);
        assert_eq!(margins.horizontal(), 20.0);
        assert_eq!(margins.vertical(), 20.0);

        let none = ShapeTextMargins::none();
        assert_eq!(none.horizontal(), 0.0);
    }

    // =========================================================================
    // Shape Group Tests
    // =========================================================================

    #[test]
    fn test_shape_group() {
        let mut group = ShapeGroup::new();
        assert!(group.is_empty());

        let shape1 = NodeId::new();
        let shape2 = NodeId::new();

        group.add_shape(shape1);
        group.add_shape(shape2);
        assert_eq!(group.len(), 2);
        assert!(group.contains(shape1));

        group.remove_shape(shape1);
        assert_eq!(group.len(), 1);
        assert!(!group.contains(shape1));
    }

    #[test]
    fn test_shape_group_from_shapes() {
        let shapes = vec![NodeId::new(), NodeId::new(), NodeId::new()];
        let group = ShapeGroup::from_shapes(shapes.clone());
        assert_eq!(group.len(), 3);
        for id in shapes {
            assert!(group.contains(id));
        }
    }

    // =========================================================================
    // Connector Tests
    // =========================================================================

    #[test]
    fn test_connector_creation() {
        let connector = Connector::new();
        assert_eq!(connector.routing, ConnectorRouting::Straight);
    }

    #[test]
    fn test_straight_connector() {
        let shape1 = NodeId::new();
        let shape2 = NodeId::new();

        let connector = Connector::straight(
            shape1,
            ConnectionPoint::Right,
            shape2,
            ConnectionPoint::Left,
        );

        assert!(connector.is_connected_to(shape1));
        assert!(connector.is_connected_to(shape2));
        assert_eq!(connector.routing, ConnectorRouting::Straight);
    }

    #[test]
    fn test_elbow_connector() {
        let shape1 = NodeId::new();
        let shape2 = NodeId::new();

        let connector = Connector::elbow(
            shape1,
            ConnectionPoint::Bottom,
            shape2,
            ConnectionPoint::Top,
        );

        assert_eq!(connector.routing, ConnectorRouting::Elbow);
        assert!(!connector.adjustments.is_empty());
    }

    #[test]
    fn test_curved_connector() {
        let shape1 = NodeId::new();
        let shape2 = NodeId::new();

        let connector = Connector::curved(
            shape1,
            ConnectionPoint::Right,
            shape2,
            ConnectionPoint::Left,
        );

        assert_eq!(connector.routing, ConnectorRouting::Curved);
        assert!(connector.adjustments.len() >= 2);
    }

    #[test]
    fn test_connection_point_positions() {
        let (x, y) = ConnectionPoint::Top.normalized_position();
        assert_eq!((x, y), (0.5, 0.0));

        let (x, y) = ConnectionPoint::BottomRight.normalized_position();
        assert_eq!((x, y), (1.0, 1.0));

        let (x, y) = ConnectionPoint::Center.normalized_position();
        assert_eq!((x, y), (0.5, 0.5));
    }

    // =========================================================================
    // Arrow Config Tests
    // =========================================================================

    #[test]
    fn test_arrow_config() {
        let none = ArrowConfig::none();
        assert_eq!(none.start_type, ArrowHead::None);
        assert_eq!(none.end_type, ArrowHead::None);

        let end_only = ArrowConfig::end_arrow(ArrowHead::Triangle);
        assert_eq!(end_only.start_type, ArrowHead::None);
        assert_eq!(end_only.end_type, ArrowHead::Triangle);

        let both = ArrowConfig::both_arrows(ArrowHead::Stealth);
        assert_eq!(both.start_type, ArrowHead::Stealth);
        assert_eq!(both.end_type, ArrowHead::Stealth);
    }

    // =========================================================================
    // Rectangle/Point Tests
    // =========================================================================

    #[test]
    fn test_point() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_rect() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let center = rect.center();
        assert_eq!(center.x, 60.0);
        assert_eq!(center.y, 45.0);

        assert!(rect.contains(Point::new(50.0, 40.0)));
        assert!(!rect.contains(Point::new(0.0, 0.0)));
    }

    #[test]
    fn test_rect_union() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(25.0, 25.0, 50.0, 50.0);
        let union = r1.union(&r2);

        assert_eq!(union.x, 0.0);
        assert_eq!(union.y, 0.0);
        assert_eq!(union.width, 75.0);
        assert_eq!(union.height, 75.0);
    }

    // =========================================================================
    // Shape Category Tests
    // =========================================================================

    #[test]
    fn test_shape_categories() {
        assert_eq!(ShapeType::Rectangle.category(), ShapeCategory::Basic);
        assert_eq!(ShapeType::right_arrow().category(), ShapeCategory::BlockArrows);
        assert_eq!(ShapeType::FlowchartProcess.category(), ShapeCategory::Flowchart);
        assert_eq!(ShapeType::rectangular_callout().category(), ShapeCategory::Callouts);
        assert_eq!(ShapeType::Star5.category(), ShapeCategory::StarsAndBanners);
        assert_eq!(ShapeType::MathPlus.category(), ShapeCategory::Equation);
        assert_eq!(ShapeType::ActionButtonHome.category(), ShapeCategory::ActionButtons);
    }

    // =========================================================================
    // Advanced Shape Type Tests
    // =========================================================================

    #[test]
    fn test_block_arrow_shapes() {
        let right = ShapeType::right_arrow();
        assert_eq!(right.display_name(), "Right Arrow");

        let left = ShapeType::left_arrow();
        assert_eq!(left.display_name(), "Left Arrow");

        let up = ShapeType::up_arrow();
        assert_eq!(up.display_name(), "Up Arrow");

        let down = ShapeType::down_arrow();
        assert_eq!(down.display_name(), "Down Arrow");
    }

    #[test]
    fn test_flowchart_shapes() {
        assert_eq!(ShapeType::FlowchartProcess.display_name(), "Process");
        assert_eq!(ShapeType::FlowchartDecision.display_name(), "Decision");
        assert_eq!(ShapeType::FlowchartTerminator.display_name(), "Terminator");
        assert_eq!(ShapeType::FlowchartConnector.display_name(), "Connector");
    }

    #[test]
    fn test_callout_shapes() {
        let rect_callout = ShapeType::rectangular_callout();
        assert_eq!(rect_callout.display_name(), "Rectangular Callout");

        let cloud = ShapeType::cloud_callout();
        assert_eq!(cloud.display_name(), "Cloud Callout");
    }

    #[test]
    fn test_star_and_banner_shapes() {
        assert_eq!(ShapeType::Star4.display_name(), "4-Point Star");
        assert_eq!(ShapeType::Star5.display_name(), "5-Point Star");
        assert_eq!(ShapeType::Star8.display_name(), "8-Point Star");
        assert_eq!(ShapeType::Explosion1.display_name(), "Explosion 1");
    }

    // =========================================================================
    // Z-Order Tests
    // =========================================================================

    #[test]
    fn test_shape_z_order() {
        let mut shape1 = ShapeNode::rectangle(100.0, 100.0);
        let mut shape2 = ShapeNode::rectangle(100.0, 100.0);

        shape1.z_order = 1;
        shape2.z_order = 2;

        assert!(shape2.z_order > shape1.z_order);
    }

    // =========================================================================
    // Serialization Tests
    // =========================================================================

    #[test]
    fn test_shape_serialization() {
        let shape = ShapeNode::rectangle(100.0, 50.0);
        let json = serde_json::to_string(&shape).unwrap();
        let deserialized: ShapeNode = serde_json::from_str(&json).unwrap();

        assert_eq!(shape.id(), deserialized.id());
        assert_eq!(shape.shape_type, deserialized.shape_type);
    }

    #[test]
    fn test_connector_serialization() {
        let connector = Connector::straight(
            NodeId::new(),
            ConnectionPoint::Right,
            NodeId::new(),
            ConnectionPoint::Left,
        );

        let json = serde_json::to_string(&connector).unwrap();
        let deserialized: Connector = serde_json::from_str(&json).unwrap();

        assert_eq!(connector.id, deserialized.id);
        assert_eq!(connector.routing, deserialized.routing);
    }

    #[test]
    fn test_shape_group_serialization() {
        let group = ShapeGroup::from_shapes(vec![NodeId::new(), NodeId::new()]);

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: ShapeGroup = serde_json::from_str(&json).unwrap();

        assert_eq!(group.id, deserialized.id);
        assert_eq!(group.shapes.len(), deserialized.shapes.len());
    }
}
