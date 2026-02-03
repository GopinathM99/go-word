//! Math AST - Abstract Syntax Tree for mathematical expressions
//!
//! This module defines the core data structures for representing math equations
//! in a Word-compatible format based on Office Math Markup Language (OMML).

use serde::{Deserialize, Serialize};

// =============================================================================
// Math Node - Core AST
// =============================================================================

/// A node in the math expression tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathNode {
    // Root containers
    /// Inline math expression (embedded in text)
    OMath(Vec<MathNode>),
    /// Display math (centered, paragraph-level)
    OMathPara(Vec<MathNode>),

    // Structures
    /// Fraction: numerator over denominator
    Fraction {
        num: Box<MathNode>,
        den: Box<MathNode>,
        bar_visible: bool,
    },
    /// Radical (square root or nth root)
    Radical {
        degree: Option<Box<MathNode>>,
        base: Box<MathNode>,
    },
    /// Subscript
    Subscript {
        base: Box<MathNode>,
        sub: Box<MathNode>,
    },
    /// Superscript
    Superscript {
        base: Box<MathNode>,
        sup: Box<MathNode>,
    },
    /// Combined subscript and superscript
    SubSuperscript {
        base: Box<MathNode>,
        sub: Box<MathNode>,
        sup: Box<MathNode>,
    },
    /// N-ary operator (sum, product, integral, etc.)
    Nary {
        /// The operator character (e.g., '\u{2211}' for sum, '\u{222B}' for integral)
        op: char,
        /// Placement of subscript/superscript limits
        sub_sup_placement: SubSupPlacement,
        /// Lower limit (e.g., i=0)
        sub: Option<Box<MathNode>>,
        /// Upper limit (e.g., n)
        sup: Option<Box<MathNode>>,
        /// The expression being operated on
        base: Box<MathNode>,
    },
    /// Delimiter (parentheses, brackets, braces, etc.)
    Delimiter {
        open: char,
        close: char,
        separators: Vec<char>,
        content: Vec<MathNode>,
        /// Whether delimiters should grow to match content height
        grow: bool,
    },
    /// Matrix or array
    Matrix {
        rows: Vec<Vec<MathNode>>,
        row_spacing: f32,
        col_spacing: f32,
    },
    /// Equation array (aligned equations)
    EqArray(Vec<Vec<MathNode>>),
    /// Boxed expression (for highlighting)
    Box(Box<MathNode>),
    /// Bar over or under an expression
    Bar {
        base: Box<MathNode>,
        position: BarPosition,
    },
    /// Accent over a base (hat, tilde, arrow, etc.)
    Accent {
        base: Box<MathNode>,
        accent_char: char,
    },
    /// Limit function with lower/upper limits
    Limit {
        func: Box<MathNode>,
        limit: Box<MathNode>,
        position: LimitPosition,
    },
    /// Function name (sin, cos, log, etc.)
    Function {
        name: String,
        base: Box<MathNode>,
    },
    /// Group character (brace, bracket under/over expression)
    GroupChar {
        base: Box<MathNode>,
        chr: char,
        position: BarPosition,
    },
    /// Border box around expression
    BorderBox {
        base: Box<MathNode>,
        hide_top: bool,
        hide_bottom: bool,
        hide_left: bool,
        hide_right: bool,
    },
    /// Phantom - takes up space but is invisible
    Phantom {
        base: Box<MathNode>,
        zero_width: bool,
        zero_height: bool,
    },

    // Content nodes
    /// A run of text with math styling
    Run {
        text: String,
        style: MathStyle,
    },
    /// A math operator with specific form
    Operator {
        chr: char,
        form: OperatorForm,
    },
    /// Normal text embedded in math (not italicized)
    Text(String),
    /// A single number
    Number(String),
    /// Unknown/preserved XML for round-trip fidelity
    Unknown {
        tag: String,
        content: String,
    },
}

impl MathNode {
    /// Create a new OMath container
    pub fn omath(children: Vec<MathNode>) -> Self {
        MathNode::OMath(children)
    }

    /// Create a new OMathPara container
    pub fn omath_para(children: Vec<MathNode>) -> Self {
        MathNode::OMathPara(children)
    }

    /// Create a fraction
    pub fn fraction(num: MathNode, den: MathNode) -> Self {
        MathNode::Fraction {
            num: Box::new(num),
            den: Box::new(den),
            bar_visible: true,
        }
    }

    /// Create a fraction without visible bar (stacked)
    pub fn stacked(num: MathNode, den: MathNode) -> Self {
        MathNode::Fraction {
            num: Box::new(num),
            den: Box::new(den),
            bar_visible: false,
        }
    }

    /// Create a square root
    pub fn sqrt(base: MathNode) -> Self {
        MathNode::Radical {
            degree: None,
            base: Box::new(base),
        }
    }

    /// Create an nth root
    pub fn nthroot(degree: MathNode, base: MathNode) -> Self {
        MathNode::Radical {
            degree: Some(Box::new(degree)),
            base: Box::new(base),
        }
    }

    /// Create a subscript
    pub fn subscript(base: MathNode, sub: MathNode) -> Self {
        MathNode::Subscript {
            base: Box::new(base),
            sub: Box::new(sub),
        }
    }

    /// Create a superscript
    pub fn superscript(base: MathNode, sup: MathNode) -> Self {
        MathNode::Superscript {
            base: Box::new(base),
            sup: Box::new(sup),
        }
    }

    /// Create combined sub/superscript
    pub fn sub_superscript(base: MathNode, sub: MathNode, sup: MathNode) -> Self {
        MathNode::SubSuperscript {
            base: Box::new(base),
            sub: Box::new(sub),
            sup: Box::new(sup),
        }
    }

    /// Create a text run
    pub fn run(text: impl Into<String>) -> Self {
        MathNode::Run {
            text: text.into(),
            style: MathStyle::default(),
        }
    }

    /// Create a styled text run
    pub fn styled_run(text: impl Into<String>, style: MathStyle) -> Self {
        MathNode::Run {
            text: text.into(),
            style,
        }
    }

    /// Create an operator
    pub fn operator(chr: char) -> Self {
        MathNode::Operator {
            chr,
            form: OperatorForm::Infix,
        }
    }

    /// Create a number
    pub fn number(n: impl Into<String>) -> Self {
        MathNode::Number(n.into())
    }

    /// Create parentheses around content
    pub fn parens(content: Vec<MathNode>) -> Self {
        MathNode::Delimiter {
            open: '(',
            close: ')',
            separators: vec![],
            content,
            grow: true,
        }
    }

    /// Create brackets around content
    pub fn brackets(content: Vec<MathNode>) -> Self {
        MathNode::Delimiter {
            open: '[',
            close: ']',
            separators: vec![],
            content,
            grow: true,
        }
    }

    /// Create braces around content
    pub fn braces(content: Vec<MathNode>) -> Self {
        MathNode::Delimiter {
            open: '{',
            close: '}',
            separators: vec![],
            content,
            grow: true,
        }
    }

    /// Create a summation
    pub fn sum(lower: Option<MathNode>, upper: Option<MathNode>, base: MathNode) -> Self {
        MathNode::Nary {
            op: '\u{2211}', // ∑
            sub_sup_placement: SubSupPlacement::AboveBelow,
            sub: lower.map(Box::new),
            sup: upper.map(Box::new),
            base: Box::new(base),
        }
    }

    /// Create a product
    pub fn product(lower: Option<MathNode>, upper: Option<MathNode>, base: MathNode) -> Self {
        MathNode::Nary {
            op: '\u{220F}', // ∏
            sub_sup_placement: SubSupPlacement::AboveBelow,
            sub: lower.map(Box::new),
            sup: upper.map(Box::new),
            base: Box::new(base),
        }
    }

    /// Create an integral
    pub fn integral(lower: Option<MathNode>, upper: Option<MathNode>, base: MathNode) -> Self {
        MathNode::Nary {
            op: '\u{222B}', // ∫
            sub_sup_placement: SubSupPlacement::Inline,
            sub: lower.map(Box::new),
            sup: upper.map(Box::new),
            base: Box::new(base),
        }
    }

    /// Create a matrix
    pub fn matrix(rows: Vec<Vec<MathNode>>) -> Self {
        MathNode::Matrix {
            rows,
            row_spacing: 1.0,
            col_spacing: 1.0,
        }
    }

    /// Create an overline (bar on top)
    pub fn overline(base: MathNode) -> Self {
        MathNode::Bar {
            base: Box::new(base),
            position: BarPosition::Top,
        }
    }

    /// Create an underline (bar on bottom)
    pub fn underline(base: MathNode) -> Self {
        MathNode::Bar {
            base: Box::new(base),
            position: BarPosition::Bottom,
        }
    }

    /// Check if this is an empty node
    pub fn is_empty(&self) -> bool {
        match self {
            MathNode::OMath(children) | MathNode::OMathPara(children) => {
                children.is_empty() || children.iter().all(|c| c.is_empty())
            }
            MathNode::Run { text, .. } => text.is_empty(),
            MathNode::Text(t) => t.is_empty(),
            MathNode::Number(n) => n.is_empty(),
            _ => false,
        }
    }

    /// Get all children of this node
    pub fn children(&self) -> Vec<&MathNode> {
        match self {
            MathNode::OMath(children) | MathNode::OMathPara(children) => {
                children.iter().collect()
            }
            MathNode::Fraction { num, den, .. } => vec![num.as_ref(), den.as_ref()],
            MathNode::Radical { degree, base } => {
                let mut v = vec![base.as_ref()];
                if let Some(d) = degree {
                    v.insert(0, d.as_ref());
                }
                v
            }
            MathNode::Subscript { base, sub } => vec![base.as_ref(), sub.as_ref()],
            MathNode::Superscript { base, sup } => vec![base.as_ref(), sup.as_ref()],
            MathNode::SubSuperscript { base, sub, sup } => {
                vec![base.as_ref(), sub.as_ref(), sup.as_ref()]
            }
            MathNode::Nary { sub, sup, base, .. } => {
                let mut v = Vec::new();
                if let Some(s) = sub {
                    v.push(s.as_ref());
                }
                if let Some(s) = sup {
                    v.push(s.as_ref());
                }
                v.push(base.as_ref());
                v
            }
            MathNode::Delimiter { content, .. } => content.iter().collect(),
            MathNode::Matrix { rows, .. } => rows.iter().flatten().collect(),
            MathNode::EqArray(rows) => rows.iter().flatten().collect(),
            MathNode::Box(base)
            | MathNode::Bar { base, .. }
            | MathNode::Accent { base, .. }
            | MathNode::GroupChar { base, .. } => vec![base.as_ref()],
            MathNode::Limit { func, limit, .. } => vec![func.as_ref(), limit.as_ref()],
            MathNode::Function { base, .. } => vec![base.as_ref()],
            MathNode::BorderBox { base, .. } | MathNode::Phantom { base, .. } => {
                vec![base.as_ref()]
            }
            MathNode::Run { .. }
            | MathNode::Operator { .. }
            | MathNode::Text(_)
            | MathNode::Number(_)
            | MathNode::Unknown { .. } => vec![],
        }
    }
}

// =============================================================================
// Supporting Types
// =============================================================================

/// Placement of subscript/superscript for n-ary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SubSupPlacement {
    /// Limits beside the operator (e.g., ∫_0^1)
    #[default]
    Inline,
    /// Limits above and below the operator (e.g., ∑ with limits stacked)
    AboveBelow,
}

/// Position of a bar (overline/underline)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarPosition {
    /// Bar above the expression
    Top,
    /// Bar below the expression
    Bottom,
}

/// Position for limit expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitPosition {
    /// Limit below the function
    Lower,
    /// Limit above the function
    Upper,
}

/// Form of a math operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OperatorForm {
    /// Prefix operator (e.g., - for negation)
    Prefix,
    /// Infix operator (e.g., + for addition)
    #[default]
    Infix,
    /// Postfix operator (e.g., ! for factorial)
    Postfix,
}

/// Style for math text runs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MathStyle {
    /// Font style for the text
    pub font_style: MathFontStyle,
    /// Size multiplier relative to base size (1.0 = normal)
    pub size_multiplier: f32,
    /// Whether this is a "literal" run (no auto-formatting)
    pub literal: bool,
}

impl Default for MathStyle {
    fn default() -> Self {
        Self {
            font_style: MathFontStyle::Italic,
            size_multiplier: 1.0,
            literal: false,
        }
    }
}

impl MathStyle {
    /// Create a normal (non-italic) style
    pub fn normal() -> Self {
        Self {
            font_style: MathFontStyle::Normal,
            ..Default::default()
        }
    }

    /// Create an italic style
    pub fn italic() -> Self {
        Self {
            font_style: MathFontStyle::Italic,
            ..Default::default()
        }
    }

    /// Create a bold style
    pub fn bold() -> Self {
        Self {
            font_style: MathFontStyle::Bold,
            ..Default::default()
        }
    }

    /// Create a bold-italic style
    pub fn bold_italic() -> Self {
        Self {
            font_style: MathFontStyle::BoldItalic,
            ..Default::default()
        }
    }

    /// Create a script (calligraphic) style
    pub fn script() -> Self {
        Self {
            font_style: MathFontStyle::Script,
            ..Default::default()
        }
    }

    /// Set the size multiplier
    pub fn with_size(mut self, multiplier: f32) -> Self {
        self.size_multiplier = multiplier;
        self
    }
}

/// Math font style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MathFontStyle {
    /// Normal upright text
    Normal,
    /// Italic (default for variables)
    #[default]
    Italic,
    /// Bold
    Bold,
    /// Bold Italic
    BoldItalic,
    /// Script (calligraphic)
    Script,
    /// Bold Script
    BoldScript,
    /// Fraktur (German blackletter)
    Fraktur,
    /// Bold Fraktur
    BoldFraktur,
    /// Double-struck (blackboard bold)
    DoubleStruck,
    /// Sans-serif
    SansSerif,
    /// Sans-serif Bold
    SansSerifBold,
    /// Sans-serif Italic
    SansSerifItalic,
    /// Sans-serif Bold Italic
    SansSerifBoldItalic,
    /// Monospace
    Monospace,
}

impl MathFontStyle {
    /// Check if this style is italic
    pub fn is_italic(&self) -> bool {
        matches!(
            self,
            MathFontStyle::Italic
                | MathFontStyle::BoldItalic
                | MathFontStyle::SansSerifItalic
                | MathFontStyle::SansSerifBoldItalic
        )
    }

    /// Check if this style is bold
    pub fn is_bold(&self) -> bool {
        matches!(
            self,
            MathFontStyle::Bold
                | MathFontStyle::BoldItalic
                | MathFontStyle::BoldScript
                | MathFontStyle::BoldFraktur
                | MathFontStyle::SansSerifBold
                | MathFontStyle::SansSerifBoldItalic
        )
    }
}

// =============================================================================
// Common Math Symbols
// =============================================================================

/// Common math operator characters
pub mod symbols {
    // Greek letters (lowercase)
    pub const ALPHA: char = '\u{03B1}';
    pub const BETA: char = '\u{03B2}';
    pub const GAMMA: char = '\u{03B3}';
    pub const DELTA: char = '\u{03B4}';
    pub const EPSILON: char = '\u{03B5}';
    pub const ZETA: char = '\u{03B6}';
    pub const ETA: char = '\u{03B7}';
    pub const THETA: char = '\u{03B8}';
    pub const IOTA: char = '\u{03B9}';
    pub const KAPPA: char = '\u{03BA}';
    pub const LAMBDA: char = '\u{03BB}';
    pub const MU: char = '\u{03BC}';
    pub const NU: char = '\u{03BD}';
    pub const XI: char = '\u{03BE}';
    pub const OMICRON: char = '\u{03BF}';
    pub const PI: char = '\u{03C0}';
    pub const RHO: char = '\u{03C1}';
    pub const SIGMA: char = '\u{03C3}';
    pub const TAU: char = '\u{03C4}';
    pub const UPSILON: char = '\u{03C5}';
    pub const PHI: char = '\u{03C6}';
    pub const CHI: char = '\u{03C7}';
    pub const PSI: char = '\u{03C8}';
    pub const OMEGA: char = '\u{03C9}';

    // Greek letters (uppercase)
    pub const GAMMA_UPPER: char = '\u{0393}';
    pub const DELTA_UPPER: char = '\u{0394}';
    pub const THETA_UPPER: char = '\u{0398}';
    pub const LAMBDA_UPPER: char = '\u{039B}';
    pub const XI_UPPER: char = '\u{039E}';
    pub const PI_UPPER: char = '\u{03A0}';
    pub const SIGMA_UPPER: char = '\u{03A3}';
    pub const PHI_UPPER: char = '\u{03A6}';
    pub const PSI_UPPER: char = '\u{03A8}';
    pub const OMEGA_UPPER: char = '\u{03A9}';

    // N-ary operators
    pub const SUM: char = '\u{2211}';
    pub const PRODUCT: char = '\u{220F}';
    pub const COPRODUCT: char = '\u{2210}';
    pub const INTEGRAL: char = '\u{222B}';
    pub const DOUBLE_INTEGRAL: char = '\u{222C}';
    pub const TRIPLE_INTEGRAL: char = '\u{222D}';
    pub const CONTOUR_INTEGRAL: char = '\u{222E}';
    pub const UNION: char = '\u{22C3}';
    pub const INTERSECTION: char = '\u{22C2}';

    // Binary operators
    pub const PLUS: char = '+';
    pub const MINUS: char = '\u{2212}';
    pub const TIMES: char = '\u{00D7}';
    pub const DIVIDE: char = '\u{00F7}';
    pub const DOT: char = '\u{22C5}';
    pub const PLUS_MINUS: char = '\u{00B1}';
    pub const MINUS_PLUS: char = '\u{2213}';

    // Relations
    pub const EQUALS: char = '=';
    pub const NOT_EQUAL: char = '\u{2260}';
    pub const LESS_THAN: char = '<';
    pub const GREATER_THAN: char = '>';
    pub const LESS_EQUAL: char = '\u{2264}';
    pub const GREATER_EQUAL: char = '\u{2265}';
    pub const APPROX: char = '\u{2248}';
    pub const EQUIV: char = '\u{2261}';
    pub const PROPORTIONAL: char = '\u{221D}';

    // Set notation
    pub const ELEMENT_OF: char = '\u{2208}';
    pub const NOT_ELEMENT_OF: char = '\u{2209}';
    pub const SUBSET: char = '\u{2282}';
    pub const SUPERSET: char = '\u{2283}';
    pub const SUBSET_EQUAL: char = '\u{2286}';
    pub const SUPERSET_EQUAL: char = '\u{2287}';
    pub const EMPTY_SET: char = '\u{2205}';

    // Logic
    pub const FOR_ALL: char = '\u{2200}';
    pub const EXISTS: char = '\u{2203}';
    pub const NOT_EXISTS: char = '\u{2204}';
    pub const LOGICAL_AND: char = '\u{2227}';
    pub const LOGICAL_OR: char = '\u{2228}';
    pub const LOGICAL_NOT: char = '\u{00AC}';
    pub const IMPLIES: char = '\u{21D2}';
    pub const IFF: char = '\u{21D4}';

    // Arrows
    pub const RIGHT_ARROW: char = '\u{2192}';
    pub const LEFT_ARROW: char = '\u{2190}';
    pub const UP_ARROW: char = '\u{2191}';
    pub const DOWN_ARROW: char = '\u{2193}';
    pub const DOUBLE_RIGHT_ARROW: char = '\u{21D2}';
    pub const DOUBLE_LEFT_ARROW: char = '\u{21D0}';
    pub const LEFT_RIGHT_ARROW: char = '\u{2194}';

    // Miscellaneous
    pub const INFINITY: char = '\u{221E}';
    pub const PARTIAL: char = '\u{2202}';
    pub const NABLA: char = '\u{2207}';
    pub const SQUARE_ROOT: char = '\u{221A}';
    pub const PRIME: char = '\u{2032}';
    pub const DOUBLE_PRIME: char = '\u{2033}';
    pub const DEGREE: char = '\u{00B0}';
    pub const PERCENT: char = '%';
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_node_creation() {
        let run = MathNode::run("x");
        assert!(matches!(run, MathNode::Run { text, .. } if text == "x"));
    }

    #[test]
    fn test_fraction_creation() {
        let frac = MathNode::fraction(MathNode::run("a"), MathNode::run("b"));
        if let MathNode::Fraction {
            num,
            den,
            bar_visible,
        } = frac
        {
            assert!(bar_visible);
            assert!(matches!(*num, MathNode::Run { ref text, .. } if text == "a"));
            assert!(matches!(*den, MathNode::Run { ref text, .. } if text == "b"));
        } else {
            panic!("Expected Fraction");
        }
    }

    #[test]
    fn test_sqrt_creation() {
        let sqrt = MathNode::sqrt(MathNode::run("x"));
        if let MathNode::Radical { degree, base } = sqrt {
            assert!(degree.is_none());
            assert!(matches!(*base, MathNode::Run { ref text, .. } if text == "x"));
        } else {
            panic!("Expected Radical");
        }
    }

    #[test]
    fn test_nthroot_creation() {
        let nthroot = MathNode::nthroot(MathNode::number("3"), MathNode::run("x"));
        if let MathNode::Radical { degree, base } = nthroot {
            assert!(degree.is_some());
            assert!(matches!(*base, MathNode::Run { ref text, .. } if text == "x"));
        } else {
            panic!("Expected Radical");
        }
    }

    #[test]
    fn test_subscript_creation() {
        let sub = MathNode::subscript(MathNode::run("x"), MathNode::number("2"));
        assert!(matches!(sub, MathNode::Subscript { .. }));
    }

    #[test]
    fn test_superscript_creation() {
        let sup = MathNode::superscript(MathNode::run("x"), MathNode::number("2"));
        assert!(matches!(sup, MathNode::Superscript { .. }));
    }

    #[test]
    fn test_sum_creation() {
        let sum = MathNode::sum(
            Some(MathNode::run("i=0")),
            Some(MathNode::run("n")),
            MathNode::run("i"),
        );
        if let MathNode::Nary {
            op,
            sub_sup_placement,
            ..
        } = sum
        {
            assert_eq!(op, symbols::SUM);
            assert_eq!(sub_sup_placement, SubSupPlacement::AboveBelow);
        } else {
            panic!("Expected Nary");
        }
    }

    #[test]
    fn test_integral_creation() {
        let integral = MathNode::integral(
            Some(MathNode::number("0")),
            Some(MathNode::number("1")),
            MathNode::run("f(x)dx"),
        );
        if let MathNode::Nary {
            op,
            sub_sup_placement,
            ..
        } = integral
        {
            assert_eq!(op, symbols::INTEGRAL);
            assert_eq!(sub_sup_placement, SubSupPlacement::Inline);
        } else {
            panic!("Expected Nary");
        }
    }

    #[test]
    fn test_matrix_creation() {
        let matrix = MathNode::matrix(vec![
            vec![MathNode::number("1"), MathNode::number("2")],
            vec![MathNode::number("3"), MathNode::number("4")],
        ]);
        if let MathNode::Matrix { rows, .. } = matrix {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
        } else {
            panic!("Expected Matrix");
        }
    }

    #[test]
    fn test_parens_creation() {
        let parens = MathNode::parens(vec![MathNode::run("x")]);
        if let MathNode::Delimiter { open, close, grow, .. } = parens {
            assert_eq!(open, '(');
            assert_eq!(close, ')');
            assert!(grow);
        } else {
            panic!("Expected Delimiter");
        }
    }

    #[test]
    fn test_is_empty() {
        let empty_run = MathNode::run("");
        assert!(empty_run.is_empty());

        let non_empty_run = MathNode::run("x");
        assert!(!non_empty_run.is_empty());

        let empty_omath = MathNode::omath(vec![]);
        assert!(empty_omath.is_empty());
    }

    #[test]
    fn test_children() {
        let frac = MathNode::fraction(MathNode::run("a"), MathNode::run("b"));
        let children = frac.children();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_math_style_default() {
        let style = MathStyle::default();
        assert_eq!(style.font_style, MathFontStyle::Italic);
        assert_eq!(style.size_multiplier, 1.0);
    }

    #[test]
    fn test_math_style_constructors() {
        assert_eq!(MathStyle::normal().font_style, MathFontStyle::Normal);
        assert_eq!(MathStyle::italic().font_style, MathFontStyle::Italic);
        assert_eq!(MathStyle::bold().font_style, MathFontStyle::Bold);
        assert_eq!(MathStyle::bold_italic().font_style, MathFontStyle::BoldItalic);
        assert_eq!(MathStyle::script().font_style, MathFontStyle::Script);
    }

    #[test]
    fn test_font_style_is_italic() {
        assert!(MathFontStyle::Italic.is_italic());
        assert!(MathFontStyle::BoldItalic.is_italic());
        assert!(!MathFontStyle::Normal.is_italic());
        assert!(!MathFontStyle::Bold.is_italic());
    }

    #[test]
    fn test_font_style_is_bold() {
        assert!(MathFontStyle::Bold.is_bold());
        assert!(MathFontStyle::BoldItalic.is_bold());
        assert!(!MathFontStyle::Normal.is_bold());
        assert!(!MathFontStyle::Italic.is_bold());
    }

    #[test]
    fn test_serialization() {
        let node = MathNode::fraction(MathNode::run("1"), MathNode::run("2"));
        let json = serde_json::to_string(&node).unwrap();
        let deserialized: MathNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, deserialized);
    }
}
