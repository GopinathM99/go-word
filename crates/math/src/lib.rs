//! Math Crate - Equation/Math support for the MS Word clone
//!
//! This crate provides comprehensive support for mathematical equations including:
//! - A math AST (Abstract Syntax Tree) for representing equations
//! - OMML (Office Math Markup Language) parsing and writing
//! - Layout calculation for positioning math elements
//! - Rendering to primitives for display
//! - Linear notation parsing for user input
//! - Equation editing commands and state management
//! - Equation templates and symbol galleries

pub mod commands;
pub mod editor;
pub mod error;
pub mod gallery;
pub mod layout;
pub mod linear;
pub mod model;
pub mod omml_parser;
pub mod omml_writer;
pub mod render;

pub use commands::{
    Command, CommandHandler, EquationDisplayMode, InsertEquation, InsertStructure, InsertSymbol,
    StructureParams, StructureType, SymbolCategory,
};
pub use editor::{EquationEditor, MathBox, MathBoxType, MathPath, MathSelection, NavDirection};
pub use error::*;
pub use gallery::{
    all_structure_categories, all_symbol_categories, builtin_templates, search_symbols,
    search_templates, structures_for_category, symbols_for_category, templates_by_category,
    EquationTemplate, RecentlyUsed, StructureCategory, StructureEntry, SymbolEntry,
    SymbolPaletteCategory, TemplateCategory,
};
pub use layout::{LayoutBox, LayoutContent, LayoutEngine, MathFontMetrics, Point, Rect, Size};
pub use linear::parse_linear;
pub use model::*;
pub use omml_parser::{parse_omml, OmmlParser};
pub use omml_writer::{to_omml, OmmlWriter};
pub use render::{Color, RenderConfig, RenderOutput, RenderPrimitive, Renderer, TextStyle};

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Integration Tests
    // =============================================================================

    #[test]
    fn test_parse_layout_render_pipeline() {
        // Create a simple fraction
        let node = MathNode::fraction(MathNode::run("a"), MathNode::run("b"));

        // Layout the node
        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&node).unwrap();

        // Verify layout has content
        assert!(layout.width() > 0.0);
        assert!(layout.height() > 0.0);

        // Render the layout
        let renderer = Renderer::new();
        let output = renderer.render(&layout).unwrap();

        // Verify render output
        assert!(!output.primitives.is_empty());
    }

    #[test]
    fn test_omml_roundtrip() {
        let original = MathNode::omath(vec![MathNode::fraction(
            MathNode::superscript(MathNode::run("x"), MathNode::number("2")),
            MathNode::run("y"),
        )]);

        // Write to OMML
        let xml = to_omml(&original).unwrap();

        // Parse back
        let parsed = parse_omml(&xml).unwrap();

        // Should have one node
        assert_eq!(parsed.len(), 1);

        // Should be oMath containing fraction
        if let MathNode::OMath(children) = &parsed[0] {
            assert!(matches!(children[0], MathNode::Fraction { .. }));
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_linear_to_layout() {
        // Parse linear notation
        let node = parse_linear("x^2 + y^2").unwrap();

        // Layout
        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&node).unwrap();

        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_complex_equation() {
        // Quadratic formula
        let node = parse_linear("\\frac{-b \\pm \\sqrt{b^2}}{2a}").unwrap();

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&node).unwrap();

        let renderer = Renderer::new();
        let output = renderer.render(&layout).unwrap();

        assert!(output.bounds.width() > 0.0);
    }

    #[test]
    fn test_matrix_creation() {
        let matrix = MathNode::matrix(vec![
            vec![MathNode::number("1"), MathNode::number("0")],
            vec![MathNode::number("0"), MathNode::number("1")],
        ]);

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&matrix).unwrap();

        assert!(layout.width() > 0.0);
        assert!(layout.height() > 0.0);
    }

    #[test]
    fn test_nary_operators() {
        let sum = MathNode::sum(
            Some(MathNode::run("i=1")),
            Some(MathNode::run("n")),
            MathNode::run("i"),
        );

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&sum).unwrap();

        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_nested_fractions() {
        let inner = MathNode::fraction(MathNode::number("1"), MathNode::number("2"));
        let outer = MathNode::fraction(inner, MathNode::number("3"));

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&outer).unwrap();

        // Nested fractions should have reasonable height
        assert!(layout.height() > 0.0);
    }

    #[test]
    fn test_subscript_superscript_combined() {
        let node = MathNode::sub_superscript(
            MathNode::run("x"),
            MathNode::run("i"),
            MathNode::number("2"),
        );

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&node).unwrap();

        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_delimiter_stretching() {
        let tall_content = MathNode::fraction(
            MathNode::fraction(MathNode::run("a"), MathNode::run("b")),
            MathNode::run("c"),
        );

        let delimited = MathNode::parens(vec![tall_content]);

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&delimited).unwrap();

        assert!(layout.height() > 0.0);
    }

    #[test]
    fn test_radical_with_degree() {
        let cube_root = MathNode::nthroot(MathNode::number("3"), MathNode::run("x"));

        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.layout(&cube_root).unwrap();

        assert!(layout.width() > 0.0);
    }

    #[test]
    fn test_overline_underline() {
        let overline = MathNode::overline(MathNode::run("AB"));
        let underline = MathNode::underline(MathNode::run("CD"));

        let layout_engine = LayoutEngine::new();

        let over_layout = layout_engine.layout(&overline).unwrap();
        let under_layout = layout_engine.layout(&underline).unwrap();

        assert!(over_layout.height() > 0.0);
        assert!(under_layout.height() > 0.0);
    }

    #[test]
    fn test_font_metrics_scaling() {
        let metrics = MathFontMetrics::for_size(12.0);
        let script = metrics.script_metrics();
        let scriptscript = metrics.scriptscript_metrics();

        assert!(script.font_size < metrics.font_size);
        assert!(scriptscript.font_size < script.font_size);
    }

    #[test]
    fn test_symbols_module() {
        // Verify symbol constants are accessible
        assert_eq!(symbols::SUM, '\u{2211}');
        assert_eq!(symbols::INTEGRAL, '\u{222B}');
        assert_eq!(symbols::PI, '\u{03C0}');
        assert_eq!(symbols::INFINITY, '\u{221E}');
    }

    #[test]
    fn test_math_style_variants() {
        let normal = MathStyle::normal();
        let italic = MathStyle::italic();
        let bold = MathStyle::bold();
        let script = MathStyle::script();

        assert_eq!(normal.font_style, MathFontStyle::Normal);
        assert_eq!(italic.font_style, MathFontStyle::Italic);
        assert_eq!(bold.font_style, MathFontStyle::Bold);
        assert_eq!(script.font_style, MathFontStyle::Script);
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
        let output = renderer.render(&layout).unwrap();

        assert!(!output.primitives.is_empty());
    }
}
