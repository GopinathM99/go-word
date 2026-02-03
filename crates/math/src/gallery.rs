//! Equation Gallery and Templates
//!
//! This module provides pre-built equation templates, commonly used formulas,
//! and categorized symbol palettes for the equation editor UI.

use crate::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Equation Templates
// =============================================================================

/// Categories for equation templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Algebra formulas
    Algebra,
    /// Calculus formulas
    Calculus,
    /// Trigonometry formulas
    Trigonometry,
    /// Geometry formulas
    Geometry,
    /// Statistics formulas
    Statistics,
    /// Physics formulas
    Physics,
    /// Matrix operations
    Matrices,
    /// Set theory
    SetTheory,
    /// Logic formulas
    Logic,
    /// Number theory
    NumberTheory,
}

/// An equation template
#[derive(Debug, Clone)]
pub struct EquationTemplate {
    /// Unique identifier
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Description
    pub description: &'static str,
    /// Category
    pub category: TemplateCategory,
    /// Linear notation for the equation
    pub linear_notation: &'static str,
    /// Tags for searching
    pub tags: &'static [&'static str],
}

impl EquationTemplate {
    /// Create the MathNode for this template
    pub fn to_math_node(&self) -> MathNode {
        // Parse the linear notation to create the node
        crate::linear::parse_linear(self.linear_notation).unwrap_or_else(|_| {
            MathNode::Text(self.linear_notation.to_string())
        })
    }
}

/// Built-in equation templates
pub fn builtin_templates() -> Vec<EquationTemplate> {
    vec![
        // Algebra
        EquationTemplate {
            id: "quadratic_formula",
            name: "Quadratic Formula",
            description: "Solution to ax^2 + bx + c = 0",
            category: TemplateCategory::Algebra,
            linear_notation: "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}",
            tags: &["quadratic", "roots", "polynomial"],
        },
        EquationTemplate {
            id: "binomial_theorem",
            name: "Binomial Theorem",
            description: "Expansion of (a + b)^n",
            category: TemplateCategory::Algebra,
            linear_notation: "(a + b)^n = \\sum_{k=0}^{n} \\frac{n!}{k!(n-k)!} a^{n-k} b^k",
            tags: &["binomial", "expansion", "combinatorics"],
        },
        EquationTemplate {
            id: "completing_square",
            name: "Completing the Square",
            description: "Rewriting quadratic expressions",
            category: TemplateCategory::Algebra,
            linear_notation: "ax^2 + bx + c = a(x + \\frac{b}{2a})^2 - \\frac{b^2 - 4ac}{4a}",
            tags: &["quadratic", "completing square"],
        },
        EquationTemplate {
            id: "difference_of_squares",
            name: "Difference of Squares",
            description: "Factoring a^2 - b^2",
            category: TemplateCategory::Algebra,
            linear_notation: "a^2 - b^2 = (a + b)(a - b)",
            tags: &["factoring", "squares"],
        },
        EquationTemplate {
            id: "sum_of_cubes",
            name: "Sum of Cubes",
            description: "Factoring a^3 + b^3",
            category: TemplateCategory::Algebra,
            linear_notation: "a^3 + b^3 = (a + b)(a^2 - ab + b^2)",
            tags: &["factoring", "cubes"],
        },

        // Calculus
        EquationTemplate {
            id: "derivative_definition",
            name: "Derivative Definition",
            description: "Limit definition of derivative",
            category: TemplateCategory::Calculus,
            linear_notation: "f'(x) = \\lim_{h \\to 0} \\frac{f(x+h) - f(x)}{h}",
            tags: &["derivative", "limit", "calculus"],
        },
        EquationTemplate {
            id: "chain_rule",
            name: "Chain Rule",
            description: "Derivative of composite functions",
            category: TemplateCategory::Calculus,
            linear_notation: "\\frac{d}{dx}[f(g(x))] = f'(g(x)) \\cdot g'(x)",
            tags: &["derivative", "chain rule"],
        },
        EquationTemplate {
            id: "product_rule",
            name: "Product Rule",
            description: "Derivative of product of functions",
            category: TemplateCategory::Calculus,
            linear_notation: "(fg)' = f'g + fg'",
            tags: &["derivative", "product rule"],
        },
        EquationTemplate {
            id: "quotient_rule",
            name: "Quotient Rule",
            description: "Derivative of quotient of functions",
            category: TemplateCategory::Calculus,
            linear_notation: "(\\frac{f}{g})' = \\frac{f'g - fg'}{g^2}",
            tags: &["derivative", "quotient rule"],
        },
        EquationTemplate {
            id: "fundamental_theorem",
            name: "Fundamental Theorem of Calculus",
            description: "Relationship between differentiation and integration",
            category: TemplateCategory::Calculus,
            linear_notation: "\\int_a^b f(x)dx = F(b) - F(a)",
            tags: &["integral", "fundamental theorem"],
        },
        EquationTemplate {
            id: "integration_by_parts",
            name: "Integration by Parts",
            description: "Integration technique",
            category: TemplateCategory::Calculus,
            linear_notation: "\\int u dv = uv - \\int v du",
            tags: &["integral", "integration by parts"],
        },
        EquationTemplate {
            id: "taylor_series",
            name: "Taylor Series",
            description: "Power series expansion of a function",
            category: TemplateCategory::Calculus,
            linear_notation: "f(x) = \\sum_{n=0}^{\\infty} \\frac{f^{(n)}(a)}{n!}(x-a)^n",
            tags: &["series", "taylor", "expansion"],
        },

        // Trigonometry
        EquationTemplate {
            id: "pythagorean_identity",
            name: "Pythagorean Identity",
            description: "Fundamental trig identity",
            category: TemplateCategory::Trigonometry,
            linear_notation: "sin^2(\\theta) + cos^2(\\theta) = 1",
            tags: &["trig", "identity", "pythagorean"],
        },
        EquationTemplate {
            id: "sum_angle_sin",
            name: "Sum of Angles (Sine)",
            description: "Sine of sum of angles",
            category: TemplateCategory::Trigonometry,
            linear_notation: "sin(\\alpha + \\beta) = sin(\\alpha)cos(\\beta) + cos(\\alpha)sin(\\beta)",
            tags: &["trig", "sum", "sine"],
        },
        EquationTemplate {
            id: "sum_angle_cos",
            name: "Sum of Angles (Cosine)",
            description: "Cosine of sum of angles",
            category: TemplateCategory::Trigonometry,
            linear_notation: "cos(\\alpha + \\beta) = cos(\\alpha)cos(\\beta) - sin(\\alpha)sin(\\beta)",
            tags: &["trig", "sum", "cosine"],
        },
        EquationTemplate {
            id: "double_angle_sin",
            name: "Double Angle (Sine)",
            description: "Sine of double angle",
            category: TemplateCategory::Trigonometry,
            linear_notation: "sin(2\\theta) = 2sin(\\theta)cos(\\theta)",
            tags: &["trig", "double angle", "sine"],
        },
        EquationTemplate {
            id: "double_angle_cos",
            name: "Double Angle (Cosine)",
            description: "Cosine of double angle",
            category: TemplateCategory::Trigonometry,
            linear_notation: "cos(2\\theta) = cos^2(\\theta) - sin^2(\\theta)",
            tags: &["trig", "double angle", "cosine"],
        },
        EquationTemplate {
            id: "eulers_formula",
            name: "Euler's Formula",
            description: "Complex exponential",
            category: TemplateCategory::Trigonometry,
            linear_notation: "e^{i\\theta} = cos(\\theta) + i sin(\\theta)",
            tags: &["euler", "complex", "exponential"],
        },

        // Geometry
        EquationTemplate {
            id: "pythagorean_theorem",
            name: "Pythagorean Theorem",
            description: "Right triangle relationship",
            category: TemplateCategory::Geometry,
            linear_notation: "a^2 + b^2 = c^2",
            tags: &["pythagorean", "triangle", "geometry"],
        },
        EquationTemplate {
            id: "circle_area",
            name: "Area of Circle",
            description: "Area formula for circle",
            category: TemplateCategory::Geometry,
            linear_notation: "A = \\pi r^2",
            tags: &["circle", "area", "geometry"],
        },
        EquationTemplate {
            id: "sphere_volume",
            name: "Volume of Sphere",
            description: "Volume formula for sphere",
            category: TemplateCategory::Geometry,
            linear_notation: "V = \\frac{4}{3}\\pi r^3",
            tags: &["sphere", "volume", "geometry"],
        },
        EquationTemplate {
            id: "distance_formula",
            name: "Distance Formula",
            description: "Distance between two points",
            category: TemplateCategory::Geometry,
            linear_notation: "d = \\sqrt{(x_2 - x_1)^2 + (y_2 - y_1)^2}",
            tags: &["distance", "coordinates", "geometry"],
        },
        EquationTemplate {
            id: "law_of_cosines",
            name: "Law of Cosines",
            description: "Generalized Pythagorean theorem",
            category: TemplateCategory::Geometry,
            linear_notation: "c^2 = a^2 + b^2 - 2ab cos(C)",
            tags: &["cosines", "triangle", "law"],
        },

        // Statistics
        EquationTemplate {
            id: "mean",
            name: "Arithmetic Mean",
            description: "Average of values",
            category: TemplateCategory::Statistics,
            linear_notation: "\\bar{x} = \\frac{1}{n}\\sum_{i=1}^{n} x_i",
            tags: &["mean", "average", "statistics"],
        },
        EquationTemplate {
            id: "variance",
            name: "Variance",
            description: "Measure of spread",
            category: TemplateCategory::Statistics,
            linear_notation: "\\sigma^2 = \\frac{1}{n}\\sum_{i=1}^{n}(x_i - \\bar{x})^2",
            tags: &["variance", "spread", "statistics"],
        },
        EquationTemplate {
            id: "standard_deviation",
            name: "Standard Deviation",
            description: "Square root of variance",
            category: TemplateCategory::Statistics,
            linear_notation: "\\sigma = \\sqrt{\\frac{1}{n}\\sum_{i=1}^{n}(x_i - \\bar{x})^2}",
            tags: &["standard deviation", "spread", "statistics"],
        },
        EquationTemplate {
            id: "normal_distribution",
            name: "Normal Distribution",
            description: "Gaussian probability density",
            category: TemplateCategory::Statistics,
            linear_notation: "f(x) = \\frac{1}{\\sigma\\sqrt{2\\pi}}e^{-\\frac{(x-\\mu)^2}{2\\sigma^2}}",
            tags: &["normal", "gaussian", "distribution"],
        },
        EquationTemplate {
            id: "bayes_theorem",
            name: "Bayes' Theorem",
            description: "Conditional probability",
            category: TemplateCategory::Statistics,
            linear_notation: "P(A|B) = \\frac{P(B|A)P(A)}{P(B)}",
            tags: &["bayes", "probability", "conditional"],
        },

        // Physics
        EquationTemplate {
            id: "newtons_second",
            name: "Newton's Second Law",
            description: "Force equals mass times acceleration",
            category: TemplateCategory::Physics,
            linear_notation: "F = ma",
            tags: &["newton", "force", "motion"],
        },
        EquationTemplate {
            id: "kinetic_energy",
            name: "Kinetic Energy",
            description: "Energy of motion",
            category: TemplateCategory::Physics,
            linear_notation: "KE = \\frac{1}{2}mv^2",
            tags: &["energy", "kinetic", "motion"],
        },
        EquationTemplate {
            id: "gravitational_force",
            name: "Gravitational Force",
            description: "Newton's law of gravitation",
            category: TemplateCategory::Physics,
            linear_notation: "F = G\\frac{m_1 m_2}{r^2}",
            tags: &["gravity", "force", "newton"],
        },
        EquationTemplate {
            id: "einstein_mass_energy",
            name: "Mass-Energy Equivalence",
            description: "Einstein's famous equation",
            category: TemplateCategory::Physics,
            linear_notation: "E = mc^2",
            tags: &["einstein", "energy", "relativity"],
        },
        EquationTemplate {
            id: "schrodinger",
            name: "Schrodinger Equation",
            description: "Quantum mechanics wave equation",
            category: TemplateCategory::Physics,
            linear_notation: "i\\hbar\\frac{\\partial}{\\partial t}\\Psi = \\hat{H}\\Psi",
            tags: &["quantum", "schrodinger", "wave"],
        },

        // Matrices
        EquationTemplate {
            id: "2x2_determinant",
            name: "2x2 Determinant",
            description: "Determinant of 2x2 matrix",
            category: TemplateCategory::Matrices,
            linear_notation: "det(A) = ad - bc",
            tags: &["determinant", "matrix", "2x2"],
        },
        EquationTemplate {
            id: "matrix_inverse",
            name: "Matrix Inverse",
            description: "Inverse of a 2x2 matrix",
            category: TemplateCategory::Matrices,
            linear_notation: "A^{-1} = \\frac{1}{ad-bc}\\pmatrix{d & -b \\\\ -c & a}",
            tags: &["inverse", "matrix", "2x2"],
        },
        EquationTemplate {
            id: "eigenvalue",
            name: "Eigenvalue Equation",
            description: "Definition of eigenvalue",
            category: TemplateCategory::Matrices,
            linear_notation: "Av = \\lambda v",
            tags: &["eigenvalue", "matrix", "linear algebra"],
        },

        // Set Theory
        EquationTemplate {
            id: "de_morgan_union",
            name: "De Morgan's Law (Union)",
            description: "Complement of union",
            category: TemplateCategory::SetTheory,
            linear_notation: "(A \\cup B)' = A' \\cap B'",
            tags: &["de morgan", "union", "set"],
        },
        EquationTemplate {
            id: "de_morgan_intersection",
            name: "De Morgan's Law (Intersection)",
            description: "Complement of intersection",
            category: TemplateCategory::SetTheory,
            linear_notation: "(A \\cap B)' = A' \\cup B'",
            tags: &["de morgan", "intersection", "set"],
        },

        // Logic
        EquationTemplate {
            id: "modus_ponens",
            name: "Modus Ponens",
            description: "If P then Q, P, therefore Q",
            category: TemplateCategory::Logic,
            linear_notation: "((P \\Rightarrow Q) \\land P) \\Rightarrow Q",
            tags: &["modus ponens", "logic", "implication"],
        },

        // Number Theory
        EquationTemplate {
            id: "eulers_totient",
            name: "Euler's Totient",
            description: "For prime p",
            category: TemplateCategory::NumberTheory,
            linear_notation: "\\phi(p) = p - 1",
            tags: &["euler", "totient", "prime"],
        },
    ]
}

/// Get templates by category
pub fn templates_by_category(category: TemplateCategory) -> Vec<&'static EquationTemplate> {
    static TEMPLATES: std::sync::OnceLock<Vec<EquationTemplate>> = std::sync::OnceLock::new();
    let templates = TEMPLATES.get_or_init(builtin_templates);
    templates.iter().filter(|t| t.category == category).collect()
}

/// Search templates by keyword
pub fn search_templates(query: &str) -> Vec<&'static EquationTemplate> {
    static TEMPLATES: std::sync::OnceLock<Vec<EquationTemplate>> = std::sync::OnceLock::new();
    let templates = TEMPLATES.get_or_init(builtin_templates);
    let query_lower = query.to_lowercase();
    templates
        .iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&query_lower)
                || t.description.to_lowercase().contains(&query_lower)
                || t.tags.iter().any(|tag| tag.contains(&query_lower))
        })
        .collect()
}

// =============================================================================
// Symbol Palettes
// =============================================================================

/// Categories for symbol palettes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolPaletteCategory {
    /// Greek lowercase letters
    GreekLowercase,
    /// Greek uppercase letters
    GreekUppercase,
    /// Binary operators
    Operators,
    /// Relations
    Relations,
    /// Arrows
    Arrows,
    /// Set notation
    SetNotation,
    /// Logic symbols
    Logic,
    /// Miscellaneous
    Miscellaneous,
    /// N-ary operators
    NaryOperators,
    /// Accents
    Accents,
}

/// A symbol entry in a palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    /// The Unicode character
    pub char: char,
    /// Display name
    pub name: &'static str,
    /// LaTeX-style command (without backslash)
    pub command: &'static str,
    /// Category
    pub category: SymbolPaletteCategory,
}

/// Get all symbols for a category
pub fn symbols_for_category(category: SymbolPaletteCategory) -> Vec<SymbolEntry> {
    match category {
        SymbolPaletteCategory::GreekLowercase => vec![
            SymbolEntry { char: symbols::ALPHA, name: "alpha", command: "alpha", category },
            SymbolEntry { char: symbols::BETA, name: "beta", command: "beta", category },
            SymbolEntry { char: symbols::GAMMA, name: "gamma", command: "gamma", category },
            SymbolEntry { char: symbols::DELTA, name: "delta", command: "delta", category },
            SymbolEntry { char: symbols::EPSILON, name: "epsilon", command: "epsilon", category },
            SymbolEntry { char: symbols::ZETA, name: "zeta", command: "zeta", category },
            SymbolEntry { char: symbols::ETA, name: "eta", command: "eta", category },
            SymbolEntry { char: symbols::THETA, name: "theta", command: "theta", category },
            SymbolEntry { char: symbols::IOTA, name: "iota", command: "iota", category },
            SymbolEntry { char: symbols::KAPPA, name: "kappa", command: "kappa", category },
            SymbolEntry { char: symbols::LAMBDA, name: "lambda", command: "lambda", category },
            SymbolEntry { char: symbols::MU, name: "mu", command: "mu", category },
            SymbolEntry { char: symbols::NU, name: "nu", command: "nu", category },
            SymbolEntry { char: symbols::XI, name: "xi", command: "xi", category },
            SymbolEntry { char: symbols::OMICRON, name: "omicron", command: "omicron", category },
            SymbolEntry { char: symbols::PI, name: "pi", command: "pi", category },
            SymbolEntry { char: symbols::RHO, name: "rho", command: "rho", category },
            SymbolEntry { char: symbols::SIGMA, name: "sigma", command: "sigma", category },
            SymbolEntry { char: symbols::TAU, name: "tau", command: "tau", category },
            SymbolEntry { char: symbols::UPSILON, name: "upsilon", command: "upsilon", category },
            SymbolEntry { char: symbols::PHI, name: "phi", command: "phi", category },
            SymbolEntry { char: symbols::CHI, name: "chi", command: "chi", category },
            SymbolEntry { char: symbols::PSI, name: "psi", command: "psi", category },
            SymbolEntry { char: symbols::OMEGA, name: "omega", command: "omega", category },
        ],
        SymbolPaletteCategory::GreekUppercase => vec![
            SymbolEntry { char: symbols::GAMMA_UPPER, name: "Gamma", command: "Gamma", category },
            SymbolEntry { char: symbols::DELTA_UPPER, name: "Delta", command: "Delta", category },
            SymbolEntry { char: symbols::THETA_UPPER, name: "Theta", command: "Theta", category },
            SymbolEntry { char: symbols::LAMBDA_UPPER, name: "Lambda", command: "Lambda", category },
            SymbolEntry { char: symbols::XI_UPPER, name: "Xi", command: "Xi", category },
            SymbolEntry { char: symbols::PI_UPPER, name: "Pi", command: "Pi", category },
            SymbolEntry { char: symbols::SIGMA_UPPER, name: "Sigma", command: "Sigma", category },
            SymbolEntry { char: symbols::PHI_UPPER, name: "Phi", command: "Phi", category },
            SymbolEntry { char: symbols::PSI_UPPER, name: "Psi", command: "Psi", category },
            SymbolEntry { char: symbols::OMEGA_UPPER, name: "Omega", command: "Omega", category },
        ],
        SymbolPaletteCategory::Operators => vec![
            SymbolEntry { char: symbols::PLUS, name: "plus", command: "+", category },
            SymbolEntry { char: symbols::MINUS, name: "minus", command: "-", category },
            SymbolEntry { char: symbols::TIMES, name: "times", command: "times", category },
            SymbolEntry { char: symbols::DIVIDE, name: "divide", command: "div", category },
            SymbolEntry { char: symbols::DOT, name: "dot", command: "cdot", category },
            SymbolEntry { char: symbols::PLUS_MINUS, name: "plus-minus", command: "pm", category },
            SymbolEntry { char: symbols::MINUS_PLUS, name: "minus-plus", command: "mp", category },
            SymbolEntry { char: '*', name: "asterisk", command: "*", category },
            SymbolEntry { char: '\u{2217}', name: "asterisk operator", command: "ast", category },
            SymbolEntry { char: '\u{2218}', name: "ring operator", command: "circ", category },
            SymbolEntry { char: '\u{2219}', name: "bullet operator", command: "bullet", category },
        ],
        SymbolPaletteCategory::Relations => vec![
            SymbolEntry { char: symbols::EQUALS, name: "equals", command: "=", category },
            SymbolEntry { char: symbols::NOT_EQUAL, name: "not equal", command: "ne", category },
            SymbolEntry { char: symbols::LESS_THAN, name: "less than", command: "<", category },
            SymbolEntry { char: symbols::GREATER_THAN, name: "greater than", command: ">", category },
            SymbolEntry { char: symbols::LESS_EQUAL, name: "less or equal", command: "le", category },
            SymbolEntry { char: symbols::GREATER_EQUAL, name: "greater or equal", command: "ge", category },
            SymbolEntry { char: symbols::APPROX, name: "approximately", command: "approx", category },
            SymbolEntry { char: symbols::EQUIV, name: "equivalent", command: "equiv", category },
            SymbolEntry { char: symbols::PROPORTIONAL, name: "proportional", command: "propto", category },
            SymbolEntry { char: '\u{227A}', name: "precedes", command: "prec", category },
            SymbolEntry { char: '\u{227B}', name: "succeeds", command: "succ", category },
            SymbolEntry { char: '\u{223C}', name: "similar", command: "sim", category },
            SymbolEntry { char: '\u{2245}', name: "congruent", command: "cong", category },
        ],
        SymbolPaletteCategory::Arrows => vec![
            SymbolEntry { char: symbols::RIGHT_ARROW, name: "right arrow", command: "rightarrow", category },
            SymbolEntry { char: symbols::LEFT_ARROW, name: "left arrow", command: "leftarrow", category },
            SymbolEntry { char: symbols::UP_ARROW, name: "up arrow", command: "uparrow", category },
            SymbolEntry { char: symbols::DOWN_ARROW, name: "down arrow", command: "downarrow", category },
            SymbolEntry { char: symbols::LEFT_RIGHT_ARROW, name: "left-right arrow", command: "leftrightarrow", category },
            SymbolEntry { char: symbols::DOUBLE_RIGHT_ARROW, name: "double right arrow", command: "Rightarrow", category },
            SymbolEntry { char: symbols::DOUBLE_LEFT_ARROW, name: "double left arrow", command: "Leftarrow", category },
            SymbolEntry { char: '\u{21D4}', name: "double left-right arrow", command: "Leftrightarrow", category },
            SymbolEntry { char: '\u{21A6}', name: "maps to", command: "mapsto", category },
            SymbolEntry { char: '\u{21AA}', name: "hookrightarrow", command: "hookrightarrow", category },
        ],
        SymbolPaletteCategory::SetNotation => vec![
            SymbolEntry { char: symbols::ELEMENT_OF, name: "element of", command: "in", category },
            SymbolEntry { char: symbols::NOT_ELEMENT_OF, name: "not element of", command: "notin", category },
            SymbolEntry { char: symbols::SUBSET, name: "subset", command: "subset", category },
            SymbolEntry { char: symbols::SUPERSET, name: "superset", command: "supset", category },
            SymbolEntry { char: symbols::SUBSET_EQUAL, name: "subset or equal", command: "subseteq", category },
            SymbolEntry { char: symbols::SUPERSET_EQUAL, name: "superset or equal", command: "supseteq", category },
            SymbolEntry { char: symbols::EMPTY_SET, name: "empty set", command: "emptyset", category },
            SymbolEntry { char: symbols::UNION, name: "union", command: "cup", category },
            SymbolEntry { char: symbols::INTERSECTION, name: "intersection", command: "cap", category },
            SymbolEntry { char: '\u{2216}', name: "set minus", command: "setminus", category },
        ],
        SymbolPaletteCategory::Logic => vec![
            SymbolEntry { char: symbols::FOR_ALL, name: "for all", command: "forall", category },
            SymbolEntry { char: symbols::EXISTS, name: "exists", command: "exists", category },
            SymbolEntry { char: symbols::NOT_EXISTS, name: "not exists", command: "nexists", category },
            SymbolEntry { char: symbols::LOGICAL_AND, name: "logical and", command: "land", category },
            SymbolEntry { char: symbols::LOGICAL_OR, name: "logical or", command: "lor", category },
            SymbolEntry { char: symbols::LOGICAL_NOT, name: "logical not", command: "lnot", category },
            SymbolEntry { char: symbols::IMPLIES, name: "implies", command: "Rightarrow", category },
            SymbolEntry { char: symbols::IFF, name: "if and only if", command: "Leftrightarrow", category },
            SymbolEntry { char: '\u{22A2}', name: "proves", command: "vdash", category },
            SymbolEntry { char: '\u{22A8}', name: "models", command: "models", category },
            SymbolEntry { char: '\u{22A5}', name: "bottom", command: "bot", category },
            SymbolEntry { char: '\u{22A4}', name: "top", command: "top", category },
        ],
        SymbolPaletteCategory::Miscellaneous => vec![
            SymbolEntry { char: symbols::INFINITY, name: "infinity", command: "infty", category },
            SymbolEntry { char: symbols::PARTIAL, name: "partial", command: "partial", category },
            SymbolEntry { char: symbols::NABLA, name: "nabla", command: "nabla", category },
            SymbolEntry { char: symbols::SQUARE_ROOT, name: "square root", command: "sqrt", category },
            SymbolEntry { char: symbols::PRIME, name: "prime", command: "'", category },
            SymbolEntry { char: symbols::DOUBLE_PRIME, name: "double prime", command: "''", category },
            SymbolEntry { char: symbols::DEGREE, name: "degree", command: "degree", category },
            SymbolEntry { char: '\u{210F}', name: "h-bar", command: "hbar", category },
            SymbolEntry { char: '\u{2113}', name: "script l", command: "ell", category },
            SymbolEntry { char: '\u{2111}', name: "imaginary part", command: "Im", category },
            SymbolEntry { char: '\u{211C}', name: "real part", command: "Re", category },
            SymbolEntry { char: '\u{2135}', name: "aleph", command: "aleph", category },
        ],
        SymbolPaletteCategory::NaryOperators => vec![
            SymbolEntry { char: symbols::SUM, name: "summation", command: "sum", category },
            SymbolEntry { char: symbols::PRODUCT, name: "product", command: "prod", category },
            SymbolEntry { char: symbols::COPRODUCT, name: "coproduct", command: "coprod", category },
            SymbolEntry { char: symbols::INTEGRAL, name: "integral", command: "int", category },
            SymbolEntry { char: symbols::DOUBLE_INTEGRAL, name: "double integral", command: "iint", category },
            SymbolEntry { char: symbols::TRIPLE_INTEGRAL, name: "triple integral", command: "iiint", category },
            SymbolEntry { char: symbols::CONTOUR_INTEGRAL, name: "contour integral", command: "oint", category },
            SymbolEntry { char: '\u{22C0}', name: "big wedge", command: "bigwedge", category },
            SymbolEntry { char: '\u{22C1}', name: "big vee", command: "bigvee", category },
            SymbolEntry { char: symbols::UNION, name: "big union", command: "bigcup", category },
            SymbolEntry { char: symbols::INTERSECTION, name: "big intersection", command: "bigcap", category },
        ],
        SymbolPaletteCategory::Accents => vec![
            SymbolEntry { char: '\u{0302}', name: "hat", command: "hat", category },
            SymbolEntry { char: '\u{0303}', name: "tilde", command: "tilde", category },
            SymbolEntry { char: '\u{0304}', name: "bar", command: "bar", category },
            SymbolEntry { char: '\u{20D7}', name: "vector", command: "vec", category },
            SymbolEntry { char: '\u{0307}', name: "dot", command: "dot", category },
            SymbolEntry { char: '\u{0308}', name: "double dot", command: "ddot", category },
            SymbolEntry { char: '\u{0306}', name: "breve", command: "breve", category },
            SymbolEntry { char: '\u{030C}', name: "check", command: "check", category },
        ],
    }
}

/// Get all symbol categories
pub fn all_symbol_categories() -> Vec<SymbolPaletteCategory> {
    vec![
        SymbolPaletteCategory::GreekLowercase,
        SymbolPaletteCategory::GreekUppercase,
        SymbolPaletteCategory::Operators,
        SymbolPaletteCategory::Relations,
        SymbolPaletteCategory::Arrows,
        SymbolPaletteCategory::SetNotation,
        SymbolPaletteCategory::Logic,
        SymbolPaletteCategory::NaryOperators,
        SymbolPaletteCategory::Accents,
        SymbolPaletteCategory::Miscellaneous,
    ]
}

/// Search symbols by name or command
pub fn search_symbols(query: &str) -> Vec<SymbolEntry> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for category in all_symbol_categories() {
        for symbol in symbols_for_category(category) {
            if symbol.name.to_lowercase().contains(&query_lower)
                || symbol.command.to_lowercase().contains(&query_lower)
            {
                results.push(symbol);
            }
        }
    }

    results
}

// =============================================================================
// Recently Used Tracking
// =============================================================================

/// Maximum number of recently used items to track
const MAX_RECENT_ITEMS: usize = 20;

/// Tracks recently used equations and symbols
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentlyUsed {
    /// Recently used equation template IDs
    equations: Vec<String>,
    /// Recently used symbol characters
    symbols: Vec<char>,
}

impl RecentlyUsed {
    /// Create a new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record use of an equation template
    pub fn use_equation(&mut self, template_id: &str) {
        // Remove if already present
        self.equations.retain(|id| id != template_id);
        // Add to front
        self.equations.insert(0, template_id.to_string());
        // Trim to max size
        self.equations.truncate(MAX_RECENT_ITEMS);
    }

    /// Record use of a symbol
    pub fn use_symbol(&mut self, symbol: char) {
        // Remove if already present
        self.symbols.retain(|&s| s != symbol);
        // Add to front
        self.symbols.insert(0, symbol);
        // Trim to max size
        self.symbols.truncate(MAX_RECENT_ITEMS);
    }

    /// Get recently used equations
    pub fn recent_equations(&self) -> &[String] {
        &self.equations
    }

    /// Get recently used symbols
    pub fn recent_symbols(&self) -> &[char] {
        &self.symbols
    }

    /// Clear all recent items
    pub fn clear(&mut self) {
        self.equations.clear();
        self.symbols.clear();
    }

    /// Get recent equation templates
    pub fn recent_equation_templates(&self) -> Vec<&'static EquationTemplate> {
        static TEMPLATES: std::sync::OnceLock<HashMap<String, &'static EquationTemplate>> =
            std::sync::OnceLock::new();

        let template_map = TEMPLATES.get_or_init(|| {
            let templates = Box::leak(Box::new(builtin_templates()));
            templates.iter().map(|t| (t.id.to_string(), t)).collect()
        });

        self.equations
            .iter()
            .filter_map(|id| template_map.get(id).copied())
            .collect()
    }
}

// =============================================================================
// Structure Gallery
// =============================================================================

/// Categories for mathematical structures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureCategory {
    /// Fractions
    Fractions,
    /// Scripts (superscript, subscript)
    Scripts,
    /// Radicals
    Radicals,
    /// Large operators
    LargeOperators,
    /// Brackets and delimiters
    Brackets,
    /// Functions
    Functions,
    /// Accents
    Accents,
    /// Limits
    Limits,
    /// Matrices
    Matrices,
}

/// A structure in the gallery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureEntry {
    /// Unique identifier
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Category
    pub category: StructureCategory,
    /// Linear notation preview
    pub preview: &'static str,
}

/// Get structures for a category
pub fn structures_for_category(category: StructureCategory) -> Vec<StructureEntry> {
    match category {
        StructureCategory::Fractions => vec![
            StructureEntry { id: "fraction", name: "Fraction", category, preview: "a/b" },
            StructureEntry { id: "stacked", name: "Stacked", category, preview: "a b" },
            StructureEntry { id: "slashed", name: "Slashed", category, preview: "a/b" },
            StructureEntry { id: "small_fraction", name: "Small Fraction", category, preview: "a/b" },
        ],
        StructureCategory::Scripts => vec![
            StructureEntry { id: "superscript", name: "Superscript", category, preview: "x^2" },
            StructureEntry { id: "subscript", name: "Subscript", category, preview: "x_i" },
            StructureEntry { id: "subsup", name: "Sub-Superscript", category, preview: "x_i^2" },
        ],
        StructureCategory::Radicals => vec![
            StructureEntry { id: "sqrt", name: "Square Root", category, preview: "sqrt(x)" },
            StructureEntry { id: "nthroot", name: "Nth Root", category, preview: "root(n,x)" },
        ],
        StructureCategory::LargeOperators => vec![
            StructureEntry { id: "sum", name: "Summation", category, preview: "sum" },
            StructureEntry { id: "product", name: "Product", category, preview: "prod" },
            StructureEntry { id: "integral", name: "Integral", category, preview: "int" },
            StructureEntry { id: "double_integral", name: "Double Integral", category, preview: "iint" },
            StructureEntry { id: "triple_integral", name: "Triple Integral", category, preview: "iiint" },
            StructureEntry { id: "contour_integral", name: "Contour Integral", category, preview: "oint" },
            StructureEntry { id: "union", name: "Union", category, preview: "union" },
            StructureEntry { id: "intersection", name: "Intersection", category, preview: "cap" },
        ],
        StructureCategory::Brackets => vec![
            StructureEntry { id: "parens", name: "Parentheses", category, preview: "(x)" },
            StructureEntry { id: "brackets", name: "Brackets", category, preview: "[x]" },
            StructureEntry { id: "braces", name: "Braces", category, preview: "{x}" },
            StructureEntry { id: "abs", name: "Absolute Value", category, preview: "|x|" },
            StructureEntry { id: "floor", name: "Floor", category, preview: "floor(x)" },
            StructureEntry { id: "ceil", name: "Ceiling", category, preview: "ceil(x)" },
        ],
        StructureCategory::Functions => vec![
            StructureEntry { id: "sin", name: "Sine", category, preview: "sin(x)" },
            StructureEntry { id: "cos", name: "Cosine", category, preview: "cos(x)" },
            StructureEntry { id: "tan", name: "Tangent", category, preview: "tan(x)" },
            StructureEntry { id: "log", name: "Logarithm", category, preview: "log(x)" },
            StructureEntry { id: "ln", name: "Natural Log", category, preview: "ln(x)" },
            StructureEntry { id: "exp", name: "Exponential", category, preview: "exp(x)" },
        ],
        StructureCategory::Accents => vec![
            StructureEntry { id: "hat", name: "Hat", category, preview: "x hat" },
            StructureEntry { id: "bar", name: "Overline", category, preview: "x bar" },
            StructureEntry { id: "tilde", name: "Tilde", category, preview: "x tilde" },
            StructureEntry { id: "vec", name: "Vector", category, preview: "x vec" },
            StructureEntry { id: "dot", name: "Dot", category, preview: "x dot" },
            StructureEntry { id: "ddot", name: "Double Dot", category, preview: "x ddot" },
            StructureEntry { id: "underline", name: "Underline", category, preview: "x_" },
        ],
        StructureCategory::Limits => vec![
            StructureEntry { id: "lim", name: "Limit", category, preview: "lim" },
            StructureEntry { id: "limsup", name: "Limit Superior", category, preview: "limsup" },
            StructureEntry { id: "liminf", name: "Limit Inferior", category, preview: "liminf" },
        ],
        StructureCategory::Matrices => vec![
            StructureEntry { id: "matrix_2x2", name: "2x2 Matrix", category, preview: "2x2" },
            StructureEntry { id: "matrix_3x3", name: "3x3 Matrix", category, preview: "3x3" },
            StructureEntry { id: "matrix_2x3", name: "2x3 Matrix", category, preview: "2x3" },
            StructureEntry { id: "column_vector", name: "Column Vector", category, preview: "col" },
            StructureEntry { id: "row_vector", name: "Row Vector", category, preview: "row" },
        ],
    }
}

/// Get all structure categories
pub fn all_structure_categories() -> Vec<StructureCategory> {
    vec![
        StructureCategory::Fractions,
        StructureCategory::Scripts,
        StructureCategory::Radicals,
        StructureCategory::LargeOperators,
        StructureCategory::Brackets,
        StructureCategory::Functions,
        StructureCategory::Accents,
        StructureCategory::Limits,
        StructureCategory::Matrices,
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates() {
        let templates = builtin_templates();
        assert!(!templates.is_empty());

        // Check that all templates have valid IDs
        for template in &templates {
            assert!(!template.id.is_empty());
            assert!(!template.name.is_empty());
        }
    }

    #[test]
    fn test_templates_by_category() {
        let algebra = templates_by_category(TemplateCategory::Algebra);
        assert!(!algebra.is_empty());

        for template in algebra {
            assert_eq!(template.category, TemplateCategory::Algebra);
        }
    }

    #[test]
    fn test_search_templates() {
        let results = search_templates("quadratic");
        assert!(!results.is_empty());

        let quadratic = results.iter().find(|t| t.id == "quadratic_formula");
        assert!(quadratic.is_some());
    }

    #[test]
    fn test_template_to_math_node() {
        let templates = builtin_templates();
        let template = templates.iter().find(|t| t.id == "pythagorean_theorem").unwrap();

        let node = template.to_math_node();
        // Should parse successfully
        assert!(!matches!(node, MathNode::Text(_)));
    }

    #[test]
    fn test_symbols_for_category() {
        let greek = symbols_for_category(SymbolPaletteCategory::GreekLowercase);
        assert!(!greek.is_empty());
        assert!(greek.iter().any(|s| s.char == symbols::ALPHA));
    }

    #[test]
    fn test_search_symbols() {
        let results = search_symbols("alpha");
        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.char == symbols::ALPHA));
    }

    #[test]
    fn test_recently_used() {
        let mut recent = RecentlyUsed::new();

        recent.use_equation("quadratic_formula");
        recent.use_symbol(symbols::ALPHA);

        assert_eq!(recent.recent_equations().len(), 1);
        assert_eq!(recent.recent_symbols().len(), 1);

        // Adding same item moves it to front
        recent.use_equation("pythagorean_theorem");
        recent.use_equation("quadratic_formula");
        assert_eq!(recent.recent_equations()[0], "quadratic_formula");
    }

    #[test]
    fn test_recently_used_max_items() {
        let mut recent = RecentlyUsed::new();

        for i in 0..30 {
            recent.use_equation(&format!("template_{}", i));
        }

        assert!(recent.recent_equations().len() <= MAX_RECENT_ITEMS);
    }

    #[test]
    fn test_recently_used_clear() {
        let mut recent = RecentlyUsed::new();
        recent.use_equation("test");
        recent.use_symbol('a');

        recent.clear();

        assert!(recent.recent_equations().is_empty());
        assert!(recent.recent_symbols().is_empty());
    }

    #[test]
    fn test_structures_for_category() {
        let fractions = structures_for_category(StructureCategory::Fractions);
        assert!(!fractions.is_empty());

        for structure in fractions {
            assert_eq!(structure.category, StructureCategory::Fractions);
        }
    }

    #[test]
    fn test_all_structure_categories() {
        let categories = all_structure_categories();
        assert!(!categories.is_empty());
    }

    #[test]
    fn test_all_symbol_categories() {
        let categories = all_symbol_categories();
        assert!(!categories.is_empty());
    }

    #[test]
    fn test_symbol_entry_fields() {
        let operators = symbols_for_category(SymbolPaletteCategory::Operators);
        for op in operators {
            assert!(!op.name.is_empty());
            assert!(!op.command.is_empty());
        }
    }

    #[test]
    fn test_template_categories_covered() {
        let templates = builtin_templates();
        let categories: std::collections::HashSet<_> = templates.iter().map(|t| t.category).collect();

        // Check that we have templates for multiple categories
        assert!(categories.len() > 5);
    }
}
