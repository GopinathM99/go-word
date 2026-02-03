//! Math Layout - Calculate positions and sizes for math expressions
//!
//! This module computes the layout (bounding boxes, positions) for math nodes,
//! preparing them for rendering.

use crate::error::MathResult;
use crate::model::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Layout Types
// =============================================================================

/// A position in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self::default()
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// A size with width and height
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self::default()
    }
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn x(&self) -> f32 {
        self.origin.x
    }

    pub fn y(&self) -> f32 {
        self.origin.y
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn right(&self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn bottom(&self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn center_x(&self) -> f32 {
        self.origin.x + self.size.width / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.origin.y + self.size.height / 2.0
    }
}

/// Math font metrics used for layout calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathFontMetrics {
    /// Base font size in points
    pub font_size: f32,
    /// x-height (height of lowercase x)
    pub x_height: f32,
    /// Ascent above baseline
    pub ascent: f32,
    /// Descent below baseline
    pub descent: f32,
    /// Width of a typical character
    pub char_width: f32,
    /// Fraction bar thickness
    pub fraction_rule_thickness: f32,
    /// Gap above fraction bar
    pub fraction_num_gap: f32,
    /// Gap below fraction bar
    pub fraction_den_gap: f32,
    /// Radical rule thickness
    pub radical_rule_thickness: f32,
    /// Vertical gap for radical
    pub radical_vertical_gap: f32,
    /// Subscript shift down
    pub subscript_shift_down: f32,
    /// Superscript shift up
    pub superscript_shift_up: f32,
    /// Scale factor for sub/superscript
    pub script_scale: f32,
    /// Delimiter extension size
    pub delimiter_factor: f32,
}

impl Default for MathFontMetrics {
    fn default() -> Self {
        Self::for_size(11.0)
    }
}

impl MathFontMetrics {
    /// Create metrics for a given font size
    pub fn for_size(font_size: f32) -> Self {
        // These are approximate values based on typical math fonts
        let em = font_size;
        Self {
            font_size,
            x_height: em * 0.45,
            ascent: em * 0.8,
            descent: em * 0.2,
            char_width: em * 0.5,
            fraction_rule_thickness: em * 0.04,
            fraction_num_gap: em * 0.15,
            fraction_den_gap: em * 0.15,
            radical_rule_thickness: em * 0.04,
            radical_vertical_gap: em * 0.1,
            subscript_shift_down: em * 0.25,
            superscript_shift_up: em * 0.4,
            script_scale: 0.7,
            delimiter_factor: 0.9,
        }
    }

    /// Scale metrics for script (sub/superscript) size
    pub fn script_metrics(&self) -> Self {
        Self::for_size(self.font_size * self.script_scale)
    }

    /// Scale metrics for scriptscript size (nested scripts)
    pub fn scriptscript_metrics(&self) -> Self {
        let script = self.script_metrics();
        Self::for_size(script.font_size * self.script_scale)
    }
}

/// A laid out math node with position and size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox {
    /// The bounding box
    pub bounds: Rect,
    /// Distance from top of box to baseline
    pub baseline_offset: f32,
    /// The content of this layout box
    pub content: LayoutContent,
    /// Child boxes
    pub children: Vec<LayoutBox>,
}

impl LayoutBox {
    /// Create a new layout box
    pub fn new(bounds: Rect, baseline_offset: f32, content: LayoutContent) -> Self {
        Self {
            bounds,
            baseline_offset,
            content,
            children: Vec::new(),
        }
    }

    /// Create with children
    pub fn with_children(
        bounds: Rect,
        baseline_offset: f32,
        content: LayoutContent,
        children: Vec<LayoutBox>,
    ) -> Self {
        Self {
            bounds,
            baseline_offset,
            content,
            children,
        }
    }

    /// Get the width
    pub fn width(&self) -> f32 {
        self.bounds.width()
    }

    /// Get the height
    pub fn height(&self) -> f32 {
        self.bounds.height()
    }

    /// Get the baseline y position (in absolute coordinates)
    pub fn baseline_y(&self) -> f32 {
        self.bounds.y() + self.baseline_offset
    }
}

/// The content type of a layout box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutContent {
    /// Container (oMath, oMathPara)
    Container,
    /// Text run
    Text {
        text: String,
        style: MathStyle,
    },
    /// Horizontal rule (fraction bar, overline, etc.)
    Rule {
        thickness: f32,
    },
    /// Radical symbol
    Radical {
        degree_present: bool,
    },
    /// Delimiter (parenthesis, bracket, etc.)
    Delimiter {
        char: char,
        stretched: bool,
    },
    /// N-ary operator
    NaryOp {
        char: char,
    },
    /// Accent
    Accent {
        char: char,
    },
    /// Empty space
    Space,
}

// =============================================================================
// Layout Engine
// =============================================================================

/// Engine for computing math layout
pub struct LayoutEngine {
    metrics: MathFontMetrics,
}

impl LayoutEngine {
    /// Create a new layout engine with default metrics
    pub fn new() -> Self {
        Self {
            metrics: MathFontMetrics::default(),
        }
    }

    /// Create with specific metrics
    pub fn with_metrics(metrics: MathFontMetrics) -> Self {
        Self { metrics }
    }

    /// Layout a math node and return the layout tree
    pub fn layout(&self, node: &MathNode) -> MathResult<LayoutBox> {
        self.layout_node(node, &self.metrics)
    }

    /// Layout a node with given metrics
    fn layout_node(&self, node: &MathNode, metrics: &MathFontMetrics) -> MathResult<LayoutBox> {
        match node {
            MathNode::OMath(children) | MathNode::OMathPara(children) => {
                self.layout_container(children, metrics)
            }
            MathNode::Fraction { num, den, bar_visible } => {
                self.layout_fraction(num, den, *bar_visible, metrics)
            }
            MathNode::Radical { degree, base } => {
                self.layout_radical(degree.as_deref(), base, metrics)
            }
            MathNode::Subscript { base, sub } => self.layout_subscript(base, sub, metrics),
            MathNode::Superscript { base, sup } => self.layout_superscript(base, sup, metrics),
            MathNode::SubSuperscript { base, sub, sup } => {
                self.layout_sub_superscript(base, sub, sup, metrics)
            }
            MathNode::Nary {
                op,
                sub_sup_placement,
                sub,
                sup,
                base,
            } => self.layout_nary(
                *op,
                *sub_sup_placement,
                sub.as_deref(),
                sup.as_deref(),
                base,
                metrics,
            ),
            MathNode::Delimiter {
                open,
                close,
                content,
                grow,
                ..
            } => self.layout_delimiter(*open, *close, content, *grow, metrics),
            MathNode::Matrix { rows, .. } => self.layout_matrix(rows, metrics),
            MathNode::EqArray(rows) => self.layout_eq_array(rows, metrics),
            MathNode::Box(base) => self.layout_node(base, metrics),
            MathNode::Bar { base, position } => self.layout_bar(base, *position, metrics),
            MathNode::Accent { base, accent_char } => {
                self.layout_accent(base, *accent_char, metrics)
            }
            MathNode::Limit { func, limit, position } => {
                self.layout_limit(func, limit, *position, metrics)
            }
            MathNode::Function { name, base } => self.layout_function(name, base, metrics),
            MathNode::GroupChar { base, chr, position } => {
                self.layout_group_char(base, *chr, *position, metrics)
            }
            MathNode::BorderBox { base, .. } => {
                // Layout the base, border is added during rendering
                self.layout_node(base, metrics)
            }
            MathNode::Phantom { base, zero_width, zero_height } => {
                self.layout_phantom(base, *zero_width, *zero_height, metrics)
            }
            MathNode::Run { text, style } => self.layout_run(text, style, metrics),
            MathNode::Operator { chr, .. } => self.layout_operator(*chr, metrics),
            MathNode::Text(text) => self.layout_text(text, metrics),
            MathNode::Number(num) => self.layout_number(num, metrics),
            MathNode::Unknown { .. } => {
                // Unknown elements get empty boxes
                Ok(LayoutBox::new(
                    Rect::default(),
                    0.0,
                    LayoutContent::Space,
                ))
            }
        }
    }

    /// Layout a container (horizontal sequence of children)
    fn layout_container(
        &self,
        children: &[MathNode],
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let mut child_boxes = Vec::new();
        let mut x = 0.0f32;
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;

        for child in children {
            let mut child_box = self.layout_node(child, metrics)?;
            child_box.bounds.origin.x = x;

            let ascent = child_box.baseline_offset;
            let descent = child_box.height() - child_box.baseline_offset;

            max_ascent = max_ascent.max(ascent);
            max_descent = max_descent.max(descent);

            x += child_box.width();
            child_boxes.push(child_box);
        }

        // Align all children to common baseline
        for child_box in &mut child_boxes {
            child_box.bounds.origin.y = max_ascent - child_box.baseline_offset;
        }

        let total_height = max_ascent + max_descent;
        let bounds = Rect::new(0.0, 0.0, x, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            max_ascent,
            LayoutContent::Container,
            child_boxes,
        ))
    }

    /// Layout a fraction
    fn layout_fraction(
        &self,
        num: &MathNode,
        den: &MathNode,
        bar_visible: bool,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let num_box = self.layout_node(num, metrics)?;
        let den_box = self.layout_node(den, metrics)?;

        let width = num_box.width().max(den_box.width());
        let bar_y = metrics.x_height / 2.0; // Axis height

        // Position numerator above bar
        let num_bottom = bar_y - metrics.fraction_num_gap;
        let mut num_layout = num_box;
        num_layout.bounds.origin.x = (width - num_layout.width()) / 2.0;
        num_layout.bounds.origin.y = num_bottom - num_layout.height();

        // Position denominator below bar
        let den_top = bar_y + metrics.fraction_rule_thickness + metrics.fraction_den_gap;
        let mut den_layout = den_box;
        den_layout.bounds.origin.x = (width - den_layout.width()) / 2.0;
        den_layout.bounds.origin.y = den_top;

        let top = num_layout.bounds.y();
        let bottom = den_layout.bounds.bottom();
        let height = bottom - top;

        // Shift everything so top is at y=0
        num_layout.bounds.origin.y -= top;
        den_layout.bounds.origin.y -= top;
        let baseline_offset = bar_y - top + metrics.x_height / 2.0;

        let bounds = Rect::new(0.0, 0.0, width, height);

        let mut children = vec![num_layout, den_layout];

        // Add fraction bar if visible
        if bar_visible {
            let bar_box = LayoutBox::new(
                Rect::new(
                    0.0,
                    bar_y - top,
                    width,
                    metrics.fraction_rule_thickness,
                ),
                0.0,
                LayoutContent::Rule {
                    thickness: metrics.fraction_rule_thickness,
                },
            );
            children.push(bar_box);
        }

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout a radical (square root or nth root)
    fn layout_radical(
        &self,
        degree: Option<&MathNode>,
        base: &MathNode,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;

        // Radical symbol size
        let radical_height = base_box.height() + metrics.radical_vertical_gap;
        let radical_width = metrics.char_width * 0.8;

        let mut children = Vec::new();
        let mut total_width = radical_width + base_box.width();
        let mut total_height = radical_height + metrics.radical_rule_thickness;
        let mut baseline_offset = base_box.baseline_offset + metrics.radical_vertical_gap;

        // Handle degree if present
        let degree_width = if let Some(deg) = degree {
            let script_metrics = metrics.script_metrics();
            let mut deg_box = self.layout_node(deg, &script_metrics)?;
            deg_box.bounds.origin.x = 0.0;
            deg_box.bounds.origin.y = 0.0;
            let w = deg_box.width();
            children.push(deg_box);
            w + 2.0
        } else {
            0.0
        };

        total_width += degree_width;

        // Radical symbol
        let radical_box = LayoutBox::new(
            Rect::new(
                degree_width,
                0.0,
                radical_width,
                radical_height,
            ),
            radical_height,
            LayoutContent::Radical {
                degree_present: degree.is_some(),
            },
        );
        children.push(radical_box);

        // Base content
        let mut base_layout = base_box;
        base_layout.bounds.origin.x = degree_width + radical_width;
        base_layout.bounds.origin.y = metrics.radical_vertical_gap + metrics.radical_rule_thickness;
        children.push(base_layout);

        // Overline
        let rule_box = LayoutBox::new(
            Rect::new(
                degree_width + radical_width,
                0.0,
                total_width - degree_width - radical_width,
                metrics.radical_rule_thickness,
            ),
            0.0,
            LayoutContent::Rule {
                thickness: metrics.radical_rule_thickness,
            },
        );
        children.push(rule_box);

        let bounds = Rect::new(0.0, 0.0, total_width, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout subscript
    fn layout_subscript(
        &self,
        base: &MathNode,
        sub: &MathNode,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let script_metrics = metrics.script_metrics();
        let sub_box = self.layout_node(sub, &script_metrics)?;

        let width = base_box.width() + sub_box.width();

        let mut base_layout = base_box;
        base_layout.bounds.origin = Point::origin();

        let mut sub_layout = sub_box;
        sub_layout.bounds.origin.x = base_layout.width();
        sub_layout.bounds.origin.y =
            base_layout.baseline_offset + metrics.subscript_shift_down - sub_layout.baseline_offset;

        let height = (base_layout.bounds.bottom())
            .max(sub_layout.bounds.bottom());
        let baseline_offset = base_layout.baseline_offset;

        let bounds = Rect::new(0.0, 0.0, width, height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            vec![base_layout, sub_layout],
        ))
    }

    /// Layout superscript
    fn layout_superscript(
        &self,
        base: &MathNode,
        sup: &MathNode,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let script_metrics = metrics.script_metrics();
        let sup_box = self.layout_node(sup, &script_metrics)?;

        let width = base_box.width() + sup_box.width();

        // Superscript is raised
        let sup_bottom = base_box.baseline_offset - metrics.superscript_shift_up;
        let sup_top = sup_bottom - sup_box.height();

        let top_offset = if sup_top < 0.0 { -sup_top } else { 0.0 };

        let mut base_layout = base_box;
        base_layout.bounds.origin.y = top_offset;

        let mut sup_layout = sup_box;
        sup_layout.bounds.origin.x = base_layout.width();
        sup_layout.bounds.origin.y = top_offset + sup_top;

        let height = (base_layout.bounds.bottom()).max(sup_layout.bounds.bottom());
        let baseline_offset = base_layout.baseline_offset + top_offset;

        let bounds = Rect::new(0.0, 0.0, width, height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            vec![base_layout, sup_layout],
        ))
    }

    /// Layout combined sub/superscript
    fn layout_sub_superscript(
        &self,
        base: &MathNode,
        sub: &MathNode,
        sup: &MathNode,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let script_metrics = metrics.script_metrics();
        let sub_box = self.layout_node(sub, &script_metrics)?;
        let sup_box = self.layout_node(sup, &script_metrics)?;

        let script_width = sub_box.width().max(sup_box.width());
        let width = base_box.width() + script_width;

        // Position superscript
        let sup_bottom = base_box.baseline_offset - metrics.superscript_shift_up;
        let sup_top = sup_bottom - sup_box.height();

        let top_offset = if sup_top < 0.0 { -sup_top } else { 0.0 };

        let mut base_layout = base_box;
        base_layout.bounds.origin.y = top_offset;

        let mut sup_layout = sup_box;
        sup_layout.bounds.origin.x = base_layout.width();
        sup_layout.bounds.origin.y = top_offset + sup_top;

        let mut sub_layout = sub_box;
        sub_layout.bounds.origin.x = base_layout.width();
        sub_layout.bounds.origin.y = top_offset + base_layout.baseline_offset
            + metrics.subscript_shift_down
            - sub_layout.baseline_offset;

        let height = (base_layout.bounds.bottom())
            .max(sub_layout.bounds.bottom())
            .max(sup_layout.bounds.bottom());
        let baseline_offset = base_layout.baseline_offset + top_offset;

        let bounds = Rect::new(0.0, 0.0, width, height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            vec![base_layout, sub_layout, sup_layout],
        ))
    }

    /// Layout n-ary operator
    fn layout_nary(
        &self,
        op: char,
        placement: SubSupPlacement,
        sub: Option<&MathNode>,
        sup: Option<&MathNode>,
        base: &MathNode,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let script_metrics = metrics.script_metrics();

        // Operator size
        let op_size = metrics.font_size * 1.5;
        let op_box = LayoutBox::new(
            Rect::new(0.0, 0.0, op_size * 0.8, op_size),
            op_size * 0.6,
            LayoutContent::NaryOp { char: op },
        );

        let mut children = vec![op_box.clone()];
        let mut total_width = op_box.width();
        let mut total_height = op_box.height();
        let mut baseline_offset = op_box.baseline_offset;

        match placement {
            SubSupPlacement::AboveBelow => {
                // Limits above and below operator
                if let Some(sup_node) = sup {
                    let mut sup_box = self.layout_node(sup_node, &script_metrics)?;
                    sup_box.bounds.origin.x = (op_box.width() - sup_box.width()) / 2.0;
                    sup_box.bounds.origin.y = 0.0;
                    total_height += sup_box.height();
                    baseline_offset += sup_box.height();
                    children.push(sup_box);
                }

                if let Some(sub_node) = sub {
                    let mut sub_box = self.layout_node(sub_node, &script_metrics)?;
                    sub_box.bounds.origin.x = (op_box.width() - sub_box.width()) / 2.0;
                    sub_box.bounds.origin.y = total_height;
                    total_height += sub_box.height();
                    children.push(sub_box);
                }
            }
            SubSupPlacement::Inline => {
                // Limits as sub/superscript
                if let Some(sup_node) = sup {
                    let mut sup_box = self.layout_node(sup_node, &script_metrics)?;
                    sup_box.bounds.origin.x = op_box.width();
                    sup_box.bounds.origin.y = 0.0;
                    children.push(sup_box);
                }

                if let Some(sub_node) = sub {
                    let mut sub_box = self.layout_node(sub_node, &script_metrics)?;
                    sub_box.bounds.origin.x = op_box.width();
                    sub_box.bounds.origin.y = op_box.height() - sub_box.height();
                    children.push(sub_box);
                }
            }
        }

        // Add base
        let mut base_layout = base_box;
        base_layout.bounds.origin.x = total_width + metrics.char_width * 0.3;
        base_layout.bounds.origin.y = baseline_offset - base_layout.baseline_offset;
        total_width = base_layout.bounds.right();
        total_height = total_height.max(base_layout.bounds.bottom());
        children.push(base_layout);

        let bounds = Rect::new(0.0, 0.0, total_width, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout delimiter (parentheses, brackets, etc.)
    fn layout_delimiter(
        &self,
        open: char,
        close: char,
        content: &[MathNode],
        grow: bool,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        // Layout content
        let content_box = self.layout_container(content, metrics)?;

        let delim_height = if grow {
            content_box.height() * metrics.delimiter_factor
        } else {
            metrics.font_size
        };
        let delim_width = metrics.char_width * 0.5;

        let mut children = Vec::new();

        // Open delimiter
        let open_box = LayoutBox::new(
            Rect::new(0.0, 0.0, delim_width, delim_height),
            delim_height * 0.6,
            LayoutContent::Delimiter {
                char: open,
                stretched: grow,
            },
        );
        children.push(open_box);

        // Content
        let mut content_layout = content_box;
        content_layout.bounds.origin.x = delim_width;
        content_layout.bounds.origin.y =
            (delim_height - content_layout.height()) / 2.0;
        let content_right = content_layout.bounds.right();
        children.push(content_layout);

        // Close delimiter
        let close_box = LayoutBox::new(
            Rect::new(content_right, 0.0, delim_width, delim_height),
            delim_height * 0.6,
            LayoutContent::Delimiter {
                char: close,
                stretched: grow,
            },
        );
        children.push(close_box);

        let total_width = content_right + delim_width;
        let bounds = Rect::new(0.0, 0.0, total_width, delim_height);

        Ok(LayoutBox::with_children(
            bounds,
            delim_height * 0.6,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout matrix
    fn layout_matrix(
        &self,
        rows: &[Vec<MathNode>],
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        if rows.is_empty() {
            return Ok(LayoutBox::new(Rect::default(), 0.0, LayoutContent::Container));
        }

        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if num_cols == 0 {
            return Ok(LayoutBox::new(Rect::default(), 0.0, LayoutContent::Container));
        }

        // Layout all cells
        let mut cell_boxes: Vec<Vec<LayoutBox>> = Vec::new();
        let mut col_widths = vec![0.0f32; num_cols];
        let mut row_heights = vec![0.0f32; rows.len()];

        for (row_idx, row) in rows.iter().enumerate() {
            let mut row_boxes = Vec::new();
            for (col_idx, cell) in row.iter().enumerate() {
                let cell_box = self.layout_node(cell, metrics)?;
                col_widths[col_idx] = col_widths[col_idx].max(cell_box.width());
                row_heights[row_idx] = row_heights[row_idx].max(cell_box.height());
                row_boxes.push(cell_box);
            }
            cell_boxes.push(row_boxes);
        }

        // Calculate positions
        let col_gap = metrics.char_width * 0.5;
        let row_gap = metrics.font_size * 0.3;

        let mut children = Vec::new();
        let mut y = 0.0;

        for (row_idx, row_boxes) in cell_boxes.into_iter().enumerate() {
            let mut x = 0.0;
            for (col_idx, mut cell_box) in row_boxes.into_iter().enumerate() {
                // Center in cell
                cell_box.bounds.origin.x = x + (col_widths[col_idx] - cell_box.width()) / 2.0;
                cell_box.bounds.origin.y = y + (row_heights[row_idx] - cell_box.height()) / 2.0;
                children.push(cell_box);
                x += col_widths[col_idx] + col_gap;
            }
            y += row_heights[row_idx] + row_gap;
        }

        let total_width: f32 = col_widths.iter().sum::<f32>() + col_gap * (num_cols as f32 - 1.0);
        let total_height: f32 = row_heights.iter().sum::<f32>() + row_gap * (rows.len() as f32 - 1.0);
        let baseline_offset = total_height / 2.0 + metrics.x_height / 2.0;

        let bounds = Rect::new(0.0, 0.0, total_width, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout equation array
    fn layout_eq_array(
        &self,
        rows: &[Vec<MathNode>],
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        // Similar to matrix but for aligned equations
        let mut row_boxes = Vec::new();
        let mut max_width = 0.0f32;
        let row_gap = metrics.font_size * 0.5;

        for row in rows {
            let row_box = self.layout_container(row, metrics)?;
            max_width = max_width.max(row_box.width());
            row_boxes.push(row_box);
        }

        let mut children = Vec::new();
        let mut y = 0.0;

        for mut row_box in row_boxes {
            row_box.bounds.origin.y = y;
            y += row_box.height() + row_gap;
            children.push(row_box);
        }

        let total_height = y - row_gap;
        let baseline_offset = total_height / 2.0 + metrics.x_height / 2.0;
        let bounds = Rect::new(0.0, 0.0, max_width, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout bar (overline/underline)
    fn layout_bar(
        &self,
        base: &MathNode,
        position: BarPosition,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let bar_thickness = metrics.fraction_rule_thickness;
        let gap = metrics.font_size * 0.1;

        let mut children = Vec::new();

        let (bar_y, base_y, total_height, baseline_offset) = match position {
            BarPosition::Top => {
                let bar_y = 0.0;
                let base_y = bar_thickness + gap;
                let total_height = base_box.height() + bar_thickness + gap;
                let baseline = base_y + base_box.baseline_offset;
                (bar_y, base_y, total_height, baseline)
            }
            BarPosition::Bottom => {
                let base_y = 0.0;
                let bar_y = base_box.height() + gap;
                let total_height = base_box.height() + bar_thickness + gap;
                let baseline = base_box.baseline_offset;
                (bar_y, base_y, total_height, baseline)
            }
        };

        // Bar
        let bar_box = LayoutBox::new(
            Rect::new(0.0, bar_y, base_box.width(), bar_thickness),
            0.0,
            LayoutContent::Rule {
                thickness: bar_thickness,
            },
        );
        children.push(bar_box);

        // Base
        let mut base_layout = base_box;
        base_layout.bounds.origin.y = base_y;
        children.push(base_layout);

        let bounds = Rect::new(0.0, 0.0, children[1].width(), total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            children,
        ))
    }

    /// Layout accent
    fn layout_accent(
        &self,
        base: &MathNode,
        accent_char: char,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let accent_height = metrics.font_size * 0.3;
        let gap = metrics.font_size * 0.05;

        let accent_box = LayoutBox::new(
            Rect::new(0.0, 0.0, base_box.width(), accent_height),
            accent_height,
            LayoutContent::Accent { char: accent_char },
        );

        let mut base_layout = base_box;
        base_layout.bounds.origin.y = accent_height + gap;

        let total_height = accent_height + gap + base_layout.height();
        let baseline_offset = accent_height + gap + base_layout.baseline_offset;

        let bounds = Rect::new(0.0, 0.0, base_layout.width(), total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            vec![accent_box, base_layout],
        ))
    }

    /// Layout limit expression
    fn layout_limit(
        &self,
        func: &MathNode,
        limit: &MathNode,
        position: LimitPosition,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let func_box = self.layout_node(func, metrics)?;
        let script_metrics = metrics.script_metrics();
        let limit_box = self.layout_node(limit, &script_metrics)?;

        let width = func_box.width().max(limit_box.width());
        let gap = metrics.font_size * 0.1;

        let (func_y, limit_y, total_height, baseline_offset) = match position {
            LimitPosition::Lower => {
                let func_y = 0.0;
                let limit_y = func_box.height() + gap;
                let total = func_box.height() + gap + limit_box.height();
                (func_y, limit_y, total, func_box.baseline_offset)
            }
            LimitPosition::Upper => {
                let limit_y = 0.0;
                let func_y = limit_box.height() + gap;
                let total = func_box.height() + gap + limit_box.height();
                (func_y, limit_y, total, func_y + func_box.baseline_offset)
            }
        };

        let mut func_layout = func_box;
        func_layout.bounds.origin.x = (width - func_layout.width()) / 2.0;
        func_layout.bounds.origin.y = func_y;

        let mut limit_layout = limit_box;
        limit_layout.bounds.origin.x = (width - limit_layout.width()) / 2.0;
        limit_layout.bounds.origin.y = limit_y;

        let bounds = Rect::new(0.0, 0.0, width, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            vec![func_layout, limit_layout],
        ))
    }

    /// Layout function (like sin, cos, log)
    fn layout_function(
        &self,
        name: &str,
        base: &MathNode,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        // Layout function name as text
        let name_box = self.layout_text(name, metrics)?;
        let base_box = self.layout_node(base, metrics)?;

        let gap = metrics.char_width * 0.2;
        let width = name_box.width() + gap + base_box.width();

        let max_ascent = name_box.baseline_offset.max(base_box.baseline_offset);
        let max_descent = (name_box.height() - name_box.baseline_offset)
            .max(base_box.height() - base_box.baseline_offset);
        let total_height = max_ascent + max_descent;

        let mut name_layout = name_box;
        name_layout.bounds.origin.y = max_ascent - name_layout.baseline_offset;

        let mut base_layout = base_box;
        base_layout.bounds.origin.x = name_layout.width() + gap;
        base_layout.bounds.origin.y = max_ascent - base_layout.baseline_offset;

        let bounds = Rect::new(0.0, 0.0, width, total_height);

        Ok(LayoutBox::with_children(
            bounds,
            max_ascent,
            LayoutContent::Container,
            vec![name_layout, base_layout],
        ))
    }

    /// Layout group character (brace over/under)
    fn layout_group_char(
        &self,
        base: &MathNode,
        chr: char,
        position: BarPosition,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;
        let char_height = metrics.font_size * 0.3;
        let gap = metrics.font_size * 0.1;

        let char_box = LayoutBox::new(
            Rect::new(0.0, 0.0, base_box.width(), char_height),
            char_height / 2.0,
            LayoutContent::Delimiter {
                char: chr,
                stretched: true,
            },
        );

        let (char_y, base_y, total_height, baseline_offset) = match position {
            BarPosition::Top => {
                let char_y = 0.0;
                let base_y = char_height + gap;
                let total = base_box.height() + char_height + gap;
                (char_y, base_y, total, base_y + base_box.baseline_offset)
            }
            BarPosition::Bottom => {
                let base_y = 0.0;
                let char_y = base_box.height() + gap;
                let total = base_box.height() + char_height + gap;
                (char_y, base_y, total, base_box.baseline_offset)
            }
        };

        let mut char_layout = char_box;
        char_layout.bounds.origin.y = char_y;

        let mut base_layout = base_box;
        base_layout.bounds.origin.y = base_y;

        let bounds = Rect::new(0.0, 0.0, base_layout.width(), total_height);

        Ok(LayoutBox::with_children(
            bounds,
            baseline_offset,
            LayoutContent::Container,
            vec![char_layout, base_layout],
        ))
    }

    /// Layout phantom (invisible content)
    fn layout_phantom(
        &self,
        base: &MathNode,
        zero_width: bool,
        zero_height: bool,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let base_box = self.layout_node(base, metrics)?;

        let width = if zero_width { 0.0 } else { base_box.width() };
        let height = if zero_height { 0.0 } else { base_box.height() };
        let baseline = if zero_height {
            0.0
        } else {
            base_box.baseline_offset
        };

        let bounds = Rect::new(0.0, 0.0, width, height);

        Ok(LayoutBox::new(bounds, baseline, LayoutContent::Space))
    }

    /// Layout a text run
    fn layout_run(
        &self,
        text: &str,
        style: &MathStyle,
        metrics: &MathFontMetrics,
    ) -> MathResult<LayoutBox> {
        let scaled_metrics = if style.size_multiplier != 1.0 {
            MathFontMetrics::for_size(metrics.font_size * style.size_multiplier)
        } else {
            metrics.clone()
        };

        let width = text.len() as f32 * scaled_metrics.char_width;
        let height = scaled_metrics.ascent + scaled_metrics.descent;

        let bounds = Rect::new(0.0, 0.0, width, height);

        Ok(LayoutBox::new(
            bounds,
            scaled_metrics.ascent,
            LayoutContent::Text {
                text: text.to_string(),
                style: style.clone(),
            },
        ))
    }

    /// Layout an operator
    fn layout_operator(&self, chr: char, metrics: &MathFontMetrics) -> MathResult<LayoutBox> {
        let width = metrics.char_width;
        let height = metrics.ascent + metrics.descent;

        let bounds = Rect::new(0.0, 0.0, width, height);

        Ok(LayoutBox::new(
            bounds,
            metrics.ascent,
            LayoutContent::Text {
                text: chr.to_string(),
                style: MathStyle::normal(),
            },
        ))
    }

    /// Layout plain text
    fn layout_text(&self, text: &str, metrics: &MathFontMetrics) -> MathResult<LayoutBox> {
        self.layout_run(text, &MathStyle::normal(), metrics)
    }

    /// Layout a number
    fn layout_number(&self, num: &str, metrics: &MathFontMetrics) -> MathResult<LayoutBox> {
        self.layout_run(num, &MathStyle::normal(), metrics)
    }
}

impl Default for LayoutEngine {
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

    #[test]
    fn test_point_operations() {
        let p = Point::new(10.0, 20.0);
        let offset = p.offset(5.0, -3.0);
        assert_eq!(offset.x, 15.0);
        assert_eq!(offset.y, 17.0);
    }

    #[test]
    fn test_rect_operations() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.right(), 40.0);
        assert_eq!(r.bottom(), 60.0);
        assert_eq!(r.center_x(), 25.0);
        assert_eq!(r.center_y(), 40.0);
    }

    #[test]
    fn test_layout_simple_run() {
        let engine = LayoutEngine::new();
        let node = MathNode::run("x");
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
        assert!(layout.height() > 0.0);
    }

    #[test]
    fn test_layout_fraction() {
        let engine = LayoutEngine::new();
        let node = MathNode::fraction(MathNode::run("a"), MathNode::run("b"));
        let layout = engine.layout(&node).unwrap();
        assert!(layout.height() > 0.0);
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn test_layout_sqrt() {
        let engine = LayoutEngine::new();
        let node = MathNode::sqrt(MathNode::run("x"));
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_layout_subscript() {
        let engine = LayoutEngine::new();
        let node = MathNode::subscript(MathNode::run("x"), MathNode::number("2"));
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_layout_superscript() {
        let engine = LayoutEngine::new();
        let node = MathNode::superscript(MathNode::run("x"), MathNode::number("2"));
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_layout_parens() {
        let engine = LayoutEngine::new();
        let node = MathNode::parens(vec![MathNode::run("x")]);
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_layout_matrix() {
        let engine = LayoutEngine::new();
        let node = MathNode::matrix(vec![
            vec![MathNode::number("1"), MathNode::number("2")],
            vec![MathNode::number("3"), MathNode::number("4")],
        ]);
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
        assert!(layout.height() > 0.0);
    }

    #[test]
    fn test_layout_sum() {
        let engine = LayoutEngine::new();
        let node = MathNode::sum(
            Some(MathNode::run("i=0")),
            Some(MathNode::run("n")),
            MathNode::run("i"),
        );
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_layout_omath() {
        let engine = LayoutEngine::new();
        let node = MathNode::omath(vec![
            MathNode::run("x"),
            MathNode::operator('+'),
            MathNode::run("y"),
        ]);
        let layout = engine.layout(&node).unwrap();
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_metrics_scaling() {
        let metrics = MathFontMetrics::for_size(12.0);
        let script = metrics.script_metrics();
        assert!(script.font_size < metrics.font_size);
    }
}
