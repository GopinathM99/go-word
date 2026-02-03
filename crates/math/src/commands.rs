//! Equation Editing Commands
//!
//! This module provides command types for editing mathematical equations.
//! Commands can be used to insert equations, symbols, and structures into documents.

use crate::error::MathResult;
use crate::model::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Command Types
// =============================================================================

/// Display mode for equations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EquationDisplayMode {
    /// Inline equation (embedded in text flow)
    #[default]
    Inline,
    /// Display equation (centered on its own line)
    Display,
}

/// Command to insert a new equation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertEquation {
    /// The initial content of the equation (can be empty for placeholder)
    pub content: Option<MathNode>,
    /// Display mode for the equation
    pub display_mode: EquationDisplayMode,
    /// Position in document (paragraph index, character offset)
    pub position: Option<(usize, usize)>,
}

impl InsertEquation {
    /// Create an empty inline equation
    pub fn empty_inline() -> Self {
        Self {
            content: None,
            display_mode: EquationDisplayMode::Inline,
            position: None,
        }
    }

    /// Create an empty display equation
    pub fn empty_display() -> Self {
        Self {
            content: None,
            display_mode: EquationDisplayMode::Display,
            position: None,
        }
    }

    /// Create an inline equation with content
    pub fn inline(content: MathNode) -> Self {
        Self {
            content: Some(content),
            display_mode: EquationDisplayMode::Inline,
            position: None,
        }
    }

    /// Create a display equation with content
    pub fn display(content: MathNode) -> Self {
        Self {
            content: Some(content),
            display_mode: EquationDisplayMode::Display,
            position: None,
        }
    }

    /// Set the insertion position
    pub fn at_position(mut self, paragraph: usize, offset: usize) -> Self {
        self.position = Some((paragraph, offset));
        self
    }

    /// Execute the command, producing the equation node
    pub fn execute(&self) -> MathResult<MathNode> {
        let content = self.content.clone().unwrap_or_else(|| {
            // Create an empty placeholder
            MathNode::Run {
                text: String::new(),
                style: MathStyle::default(),
            }
        });

        match self.display_mode {
            EquationDisplayMode::Inline => Ok(MathNode::OMath(vec![content])),
            EquationDisplayMode::Display => Ok(MathNode::OMathPara(vec![MathNode::OMath(vec![content])])),
        }
    }
}

impl Default for InsertEquation {
    fn default() -> Self {
        Self::empty_inline()
    }
}

/// Categories of math symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolCategory {
    /// Greek letters (alpha, beta, etc.)
    GreekLetter,
    /// Mathematical operators (+, -, times, etc.)
    Operator,
    /// Relations (=, <, >, approx, etc.)
    Relation,
    /// Arrows
    Arrow,
    /// Set notation symbols
    SetNotation,
    /// Logic symbols
    Logic,
    /// Miscellaneous symbols
    Miscellaneous,
}

/// Command to insert a math symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSymbol {
    /// The symbol to insert
    pub symbol: char,
    /// Category of the symbol
    pub category: SymbolCategory,
    /// Whether to insert as an operator (with spacing) or as a run
    pub as_operator: bool,
}

impl InsertSymbol {
    /// Create a symbol insertion command
    pub fn new(symbol: char, category: SymbolCategory) -> Self {
        let as_operator = matches!(
            category,
            SymbolCategory::Operator | SymbolCategory::Relation | SymbolCategory::Arrow
        );
        Self {
            symbol,
            category,
            as_operator,
        }
    }

    /// Greek letter insertion
    pub fn greek(symbol: char) -> Self {
        Self::new(symbol, SymbolCategory::GreekLetter)
    }

    /// Operator insertion
    pub fn operator(symbol: char) -> Self {
        Self::new(symbol, SymbolCategory::Operator)
    }

    /// Relation insertion
    pub fn relation(symbol: char) -> Self {
        Self::new(symbol, SymbolCategory::Relation)
    }

    /// Arrow insertion
    pub fn arrow(symbol: char) -> Self {
        Self::new(symbol, SymbolCategory::Arrow)
    }

    /// Execute the command, producing the math node
    pub fn execute(&self) -> MathResult<MathNode> {
        if self.as_operator {
            Ok(MathNode::Operator {
                chr: self.symbol,
                form: OperatorForm::Infix,
            })
        } else {
            Ok(MathNode::Run {
                text: self.symbol.to_string(),
                style: MathStyle::default(),
            })
        }
    }
}

/// Types of mathematical structures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureType {
    /// Fraction (numerator over denominator)
    Fraction,
    /// Stacked fraction (no bar)
    StackedFraction,
    /// Square root
    SquareRoot,
    /// Nth root
    NthRoot,
    /// Subscript
    Subscript,
    /// Superscript
    Superscript,
    /// Combined subscript and superscript
    SubSuperscript,
    /// Parentheses
    Parentheses,
    /// Square brackets
    Brackets,
    /// Curly braces
    Braces,
    /// Absolute value bars
    AbsoluteValue,
    /// Matrix (specify rows x cols in params)
    Matrix,
    /// Summation
    Summation,
    /// Product
    Product,
    /// Integral
    Integral,
    /// Double integral
    DoubleIntegral,
    /// Triple integral
    TripleIntegral,
    /// Contour integral
    ContourIntegral,
    /// Limit
    Limit,
    /// Overline (bar)
    Overline,
    /// Underline
    Underline,
    /// Hat accent
    Hat,
    /// Tilde accent
    Tilde,
    /// Vector arrow accent
    Vector,
    /// Dot accent
    Dot,
    /// Double dot accent
    DoubleDot,
    /// Overbrace
    Overbrace,
    /// Underbrace
    Underbrace,
    /// Equation array
    EquationArray,
}

/// Parameters for structure insertion
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructureParams {
    /// Number of rows (for matrices and equation arrays)
    pub rows: Option<usize>,
    /// Number of columns (for matrices)
    pub cols: Option<usize>,
    /// Initial content for placeholders
    pub content: Vec<MathNode>,
    /// Sub/superscript placement for n-ary operators
    pub limits_placement: Option<SubSupPlacement>,
}

impl StructureParams {
    /// Create empty params
    pub fn new() -> Self {
        Self::default()
    }

    /// Set matrix dimensions
    pub fn with_dimensions(mut self, rows: usize, cols: usize) -> Self {
        self.rows = Some(rows);
        self.cols = Some(cols);
        self
    }

    /// Add content
    pub fn with_content(mut self, content: Vec<MathNode>) -> Self {
        self.content = content;
        self
    }

    /// Set limits placement
    pub fn with_limits(mut self, placement: SubSupPlacement) -> Self {
        self.limits_placement = Some(placement);
        self
    }
}

/// Command to insert a mathematical structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertStructure {
    /// Type of structure to insert
    pub structure_type: StructureType,
    /// Parameters for the structure
    pub params: StructureParams,
}

impl InsertStructure {
    /// Create a structure insertion command
    pub fn new(structure_type: StructureType) -> Self {
        Self {
            structure_type,
            params: StructureParams::default(),
        }
    }

    /// Create with parameters
    pub fn with_params(structure_type: StructureType, params: StructureParams) -> Self {
        Self {
            structure_type,
            params,
        }
    }

    /// Create a fraction
    pub fn fraction() -> Self {
        Self::new(StructureType::Fraction)
    }

    /// Create a square root
    pub fn sqrt() -> Self {
        Self::new(StructureType::SquareRoot)
    }

    /// Create an nth root
    pub fn nthroot() -> Self {
        Self::new(StructureType::NthRoot)
    }

    /// Create a matrix with specified dimensions
    pub fn matrix(rows: usize, cols: usize) -> Self {
        Self::with_params(
            StructureType::Matrix,
            StructureParams::new().with_dimensions(rows, cols),
        )
    }

    /// Create a summation
    pub fn sum() -> Self {
        Self::with_params(
            StructureType::Summation,
            StructureParams::new().with_limits(SubSupPlacement::AboveBelow),
        )
    }

    /// Create an integral
    pub fn integral() -> Self {
        Self::with_params(
            StructureType::Integral,
            StructureParams::new().with_limits(SubSupPlacement::Inline),
        )
    }

    /// Create parentheses
    pub fn parens() -> Self {
        Self::new(StructureType::Parentheses)
    }

    /// Create brackets
    pub fn brackets() -> Self {
        Self::new(StructureType::Brackets)
    }

    /// Execute the command, producing the math node
    pub fn execute(&self) -> MathResult<MathNode> {
        let placeholder = || MathNode::Run {
            text: String::new(),
            style: MathStyle::default(),
        };

        match self.structure_type {
            StructureType::Fraction => {
                let (num, den) = if self.params.content.len() >= 2 {
                    (
                        self.params.content[0].clone(),
                        self.params.content[1].clone(),
                    )
                } else {
                    (placeholder(), placeholder())
                };
                Ok(MathNode::Fraction {
                    num: Box::new(num),
                    den: Box::new(den),
                    bar_visible: true,
                })
            }
            StructureType::StackedFraction => {
                let (num, den) = if self.params.content.len() >= 2 {
                    (
                        self.params.content[0].clone(),
                        self.params.content[1].clone(),
                    )
                } else {
                    (placeholder(), placeholder())
                };
                Ok(MathNode::Fraction {
                    num: Box::new(num),
                    den: Box::new(den),
                    bar_visible: false,
                })
            }
            StructureType::SquareRoot => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Radical {
                    degree: None,
                    base: Box::new(base),
                })
            }
            StructureType::NthRoot => {
                let (degree, base) = if self.params.content.len() >= 2 {
                    (
                        self.params.content[0].clone(),
                        self.params.content[1].clone(),
                    )
                } else {
                    (placeholder(), placeholder())
                };
                Ok(MathNode::Radical {
                    degree: Some(Box::new(degree)),
                    base: Box::new(base),
                })
            }
            StructureType::Subscript => {
                let (base, sub) = if self.params.content.len() >= 2 {
                    (
                        self.params.content[0].clone(),
                        self.params.content[1].clone(),
                    )
                } else {
                    (placeholder(), placeholder())
                };
                Ok(MathNode::Subscript {
                    base: Box::new(base),
                    sub: Box::new(sub),
                })
            }
            StructureType::Superscript => {
                let (base, sup) = if self.params.content.len() >= 2 {
                    (
                        self.params.content[0].clone(),
                        self.params.content[1].clone(),
                    )
                } else {
                    (placeholder(), placeholder())
                };
                Ok(MathNode::Superscript {
                    base: Box::new(base),
                    sup: Box::new(sup),
                })
            }
            StructureType::SubSuperscript => {
                let (base, sub, sup) = if self.params.content.len() >= 3 {
                    (
                        self.params.content[0].clone(),
                        self.params.content[1].clone(),
                        self.params.content[2].clone(),
                    )
                } else {
                    (placeholder(), placeholder(), placeholder())
                };
                Ok(MathNode::SubSuperscript {
                    base: Box::new(base),
                    sub: Box::new(sub),
                    sup: Box::new(sup),
                })
            }
            StructureType::Parentheses => {
                let content = if self.params.content.is_empty() {
                    vec![placeholder()]
                } else {
                    self.params.content.clone()
                };
                Ok(MathNode::Delimiter {
                    open: '(',
                    close: ')',
                    separators: vec![],
                    content,
                    grow: true,
                })
            }
            StructureType::Brackets => {
                let content = if self.params.content.is_empty() {
                    vec![placeholder()]
                } else {
                    self.params.content.clone()
                };
                Ok(MathNode::Delimiter {
                    open: '[',
                    close: ']',
                    separators: vec![],
                    content,
                    grow: true,
                })
            }
            StructureType::Braces => {
                let content = if self.params.content.is_empty() {
                    vec![placeholder()]
                } else {
                    self.params.content.clone()
                };
                Ok(MathNode::Delimiter {
                    open: '{',
                    close: '}',
                    separators: vec![],
                    content,
                    grow: true,
                })
            }
            StructureType::AbsoluteValue => {
                let content = if self.params.content.is_empty() {
                    vec![placeholder()]
                } else {
                    self.params.content.clone()
                };
                Ok(MathNode::Delimiter {
                    open: '|',
                    close: '|',
                    separators: vec![],
                    content,
                    grow: true,
                })
            }
            StructureType::Matrix => {
                let rows = self.params.rows.unwrap_or(2);
                let cols = self.params.cols.unwrap_or(2);

                let mut matrix_rows = Vec::with_capacity(rows);
                let mut content_iter = self.params.content.iter();

                for _ in 0..rows {
                    let mut row = Vec::with_capacity(cols);
                    for _ in 0..cols {
                        let cell = content_iter.next().cloned().unwrap_or_else(placeholder);
                        row.push(cell);
                    }
                    matrix_rows.push(row);
                }

                Ok(MathNode::Matrix {
                    rows: matrix_rows,
                    row_spacing: 1.0,
                    col_spacing: 1.0,
                })
            }
            StructureType::Summation => self.create_nary(symbols::SUM),
            StructureType::Product => self.create_nary(symbols::PRODUCT),
            StructureType::Integral => self.create_nary(symbols::INTEGRAL),
            StructureType::DoubleIntegral => self.create_nary(symbols::DOUBLE_INTEGRAL),
            StructureType::TripleIntegral => self.create_nary(symbols::TRIPLE_INTEGRAL),
            StructureType::ContourIntegral => self.create_nary(symbols::CONTOUR_INTEGRAL),
            StructureType::Limit => {
                let func = MathNode::Text("lim".to_string());
                let limit = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Limit {
                    func: Box::new(func),
                    limit: Box::new(limit),
                    position: LimitPosition::Lower,
                })
            }
            StructureType::Overline => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Bar {
                    base: Box::new(base),
                    position: BarPosition::Top,
                })
            }
            StructureType::Underline => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Bar {
                    base: Box::new(base),
                    position: BarPosition::Bottom,
                })
            }
            StructureType::Hat => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Accent {
                    base: Box::new(base),
                    accent_char: '\u{0302}', // Combining circumflex
                })
            }
            StructureType::Tilde => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Accent {
                    base: Box::new(base),
                    accent_char: '\u{0303}', // Combining tilde
                })
            }
            StructureType::Vector => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Accent {
                    base: Box::new(base),
                    accent_char: '\u{20D7}', // Combining right arrow above
                })
            }
            StructureType::Dot => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Accent {
                    base: Box::new(base),
                    accent_char: '\u{0307}', // Combining dot above
                })
            }
            StructureType::DoubleDot => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::Accent {
                    base: Box::new(base),
                    accent_char: '\u{0308}', // Combining diaeresis
                })
            }
            StructureType::Overbrace => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::GroupChar {
                    base: Box::new(base),
                    chr: '\u{23DE}', // Top curly bracket
                    position: BarPosition::Top,
                })
            }
            StructureType::Underbrace => {
                let base = self.params.content.first().cloned().unwrap_or_else(placeholder);
                Ok(MathNode::GroupChar {
                    base: Box::new(base),
                    chr: '\u{23DF}', // Bottom curly bracket
                    position: BarPosition::Bottom,
                })
            }
            StructureType::EquationArray => {
                let rows = self.params.rows.unwrap_or(2);
                let mut eq_rows = Vec::with_capacity(rows);
                let mut content_iter = self.params.content.iter();

                for _ in 0..rows {
                    let row = content_iter.next().cloned().unwrap_or_else(placeholder);
                    eq_rows.push(vec![row]);
                }

                Ok(MathNode::EqArray(eq_rows))
            }
        }
    }

    /// Helper to create n-ary operators
    fn create_nary(&self, op: char) -> MathResult<MathNode> {
        let placeholder = || MathNode::Run {
            text: String::new(),
            style: MathStyle::default(),
        };

        let placement = self.params.limits_placement.unwrap_or_else(|| {
            if op == symbols::INTEGRAL
                || op == symbols::DOUBLE_INTEGRAL
                || op == symbols::TRIPLE_INTEGRAL
                || op == symbols::CONTOUR_INTEGRAL
            {
                SubSupPlacement::Inline
            } else {
                SubSupPlacement::AboveBelow
            }
        });

        let (sub, sup, base) = if self.params.content.len() >= 3 {
            (
                Some(Box::new(self.params.content[0].clone())),
                Some(Box::new(self.params.content[1].clone())),
                self.params.content[2].clone(),
            )
        } else {
            (
                Some(Box::new(placeholder())),
                Some(Box::new(placeholder())),
                placeholder(),
            )
        };

        Ok(MathNode::Nary {
            op,
            sub_sup_placement: placement,
            sub,
            sup,
            base: Box::new(base),
        })
    }
}

// =============================================================================
// Command Handler
// =============================================================================

/// Handler for executing equation editing commands
#[derive(Debug, Default)]
pub struct CommandHandler {
    /// History of executed commands for undo support
    history: Vec<ExecutedCommand>,
    /// Index in history for redo support
    history_index: usize,
}

/// Record of an executed command
#[derive(Debug, Clone)]
pub struct ExecutedCommand {
    /// The command that was executed
    pub command: Command,
    /// The result of execution
    pub result: MathNode,
}

/// Unified command enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Insert a new equation
    InsertEquation(InsertEquation),
    /// Insert a symbol
    InsertSymbol(InsertSymbol),
    /// Insert a structure
    InsertStructure(InsertStructure),
}

impl CommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Execute a command
    pub fn execute(&mut self, command: Command) -> MathResult<MathNode> {
        let result = match &command {
            Command::InsertEquation(cmd) => cmd.execute()?,
            Command::InsertSymbol(cmd) => cmd.execute()?,
            Command::InsertStructure(cmd) => cmd.execute()?,
        };

        // Truncate history if we're not at the end
        self.history.truncate(self.history_index);

        // Add to history
        self.history.push(ExecutedCommand {
            command,
            result: result.clone(),
        });
        self.history_index += 1;

        Ok(result)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history_index < self.history.len()
    }

    /// Get the last executed command (for undo)
    pub fn last_command(&self) -> Option<&ExecutedCommand> {
        if self.history_index > 0 {
            self.history.get(self.history_index - 1)
        } else {
            None
        }
    }

    /// Move history index back (after undo)
    pub fn record_undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
        }
    }

    /// Move history index forward (after redo)
    pub fn record_redo(&mut self) -> Option<&ExecutedCommand> {
        if self.history_index < self.history.len() {
            let cmd = self.history.get(self.history_index);
            self.history_index += 1;
            cmd
        } else {
            None
        }
    }

    /// Clear command history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_index = 0;
    }

    /// Get the number of commands in history
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_equation_empty_inline() {
        let cmd = InsertEquation::empty_inline();
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::OMath(_)));
    }

    #[test]
    fn test_insert_equation_empty_display() {
        let cmd = InsertEquation::empty_display();
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::OMathPara(_)));
    }

    #[test]
    fn test_insert_equation_with_content() {
        let content = MathNode::run("x");
        let cmd = InsertEquation::inline(content);
        let result = cmd.execute().unwrap();
        if let MathNode::OMath(children) = result {
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_insert_symbol_greek() {
        let cmd = InsertSymbol::greek(symbols::ALPHA);
        let result = cmd.execute().unwrap();
        if let MathNode::Run { text, .. } = result {
            assert_eq!(text, symbols::ALPHA.to_string());
        } else {
            panic!("Expected Run");
        }
    }

    #[test]
    fn test_insert_symbol_operator() {
        let cmd = InsertSymbol::operator(symbols::PLUS_MINUS);
        let result = cmd.execute().unwrap();
        if let MathNode::Operator { chr, .. } = result {
            assert_eq!(chr, symbols::PLUS_MINUS);
        } else {
            panic!("Expected Operator");
        }
    }

    #[test]
    fn test_insert_structure_fraction() {
        let cmd = InsertStructure::fraction();
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::Fraction { bar_visible: true, .. }));
    }

    #[test]
    fn test_insert_structure_sqrt() {
        let cmd = InsertStructure::sqrt();
        let result = cmd.execute().unwrap();
        if let MathNode::Radical { degree, .. } = result {
            assert!(degree.is_none());
        } else {
            panic!("Expected Radical");
        }
    }

    #[test]
    fn test_insert_structure_nthroot() {
        let cmd = InsertStructure::nthroot();
        let result = cmd.execute().unwrap();
        if let MathNode::Radical { degree, .. } = result {
            assert!(degree.is_some());
        } else {
            panic!("Expected Radical");
        }
    }

    #[test]
    fn test_insert_structure_matrix() {
        let cmd = InsertStructure::matrix(2, 3);
        let result = cmd.execute().unwrap();
        if let MathNode::Matrix { rows, .. } = result {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 3);
        } else {
            panic!("Expected Matrix");
        }
    }

    #[test]
    fn test_insert_structure_sum() {
        let cmd = InsertStructure::sum();
        let result = cmd.execute().unwrap();
        if let MathNode::Nary { op, sub_sup_placement, .. } = result {
            assert_eq!(op, symbols::SUM);
            assert_eq!(sub_sup_placement, SubSupPlacement::AboveBelow);
        } else {
            panic!("Expected Nary");
        }
    }

    #[test]
    fn test_insert_structure_integral() {
        let cmd = InsertStructure::integral();
        let result = cmd.execute().unwrap();
        if let MathNode::Nary { op, sub_sup_placement, .. } = result {
            assert_eq!(op, symbols::INTEGRAL);
            assert_eq!(sub_sup_placement, SubSupPlacement::Inline);
        } else {
            panic!("Expected Nary");
        }
    }

    #[test]
    fn test_insert_structure_parens() {
        let cmd = InsertStructure::parens();
        let result = cmd.execute().unwrap();
        if let MathNode::Delimiter { open, close, .. } = result {
            assert_eq!(open, '(');
            assert_eq!(close, ')');
        } else {
            panic!("Expected Delimiter");
        }
    }

    #[test]
    fn test_insert_structure_with_content() {
        let params = StructureParams::new()
            .with_content(vec![MathNode::run("a"), MathNode::run("b")]);
        let cmd = InsertStructure::with_params(StructureType::Fraction, params);
        let result = cmd.execute().unwrap();
        if let MathNode::Fraction { num, den, .. } = result {
            assert!(matches!(*num, MathNode::Run { ref text, .. } if text == "a"));
            assert!(matches!(*den, MathNode::Run { ref text, .. } if text == "b"));
        } else {
            panic!("Expected Fraction");
        }
    }

    #[test]
    fn test_command_handler_execute() {
        let mut handler = CommandHandler::new();
        let cmd = Command::InsertStructure(InsertStructure::fraction());
        let result = handler.execute(cmd).unwrap();
        assert!(matches!(result, MathNode::Fraction { .. }));
        assert_eq!(handler.history_len(), 1);
    }

    #[test]
    fn test_command_handler_undo_redo() {
        let mut handler = CommandHandler::new();

        // Execute two commands
        handler
            .execute(Command::InsertStructure(InsertStructure::fraction()))
            .unwrap();
        handler
            .execute(Command::InsertStructure(InsertStructure::sqrt()))
            .unwrap();

        assert_eq!(handler.history_len(), 2);
        assert!(handler.can_undo());
        assert!(!handler.can_redo());

        // Undo
        handler.record_undo();
        assert!(handler.can_undo());
        assert!(handler.can_redo());

        // Redo
        let redone = handler.record_redo();
        assert!(redone.is_some());
        assert!(!handler.can_redo());
    }

    #[test]
    fn test_insert_structure_accents() {
        // Test hat
        let cmd = InsertStructure::new(StructureType::Hat);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::Accent { accent_char: '\u{0302}', .. }));

        // Test tilde
        let cmd = InsertStructure::new(StructureType::Tilde);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::Accent { accent_char: '\u{0303}', .. }));

        // Test vector
        let cmd = InsertStructure::new(StructureType::Vector);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::Accent { accent_char: '\u{20D7}', .. }));
    }

    #[test]
    fn test_insert_structure_bars() {
        // Test overline
        let cmd = InsertStructure::new(StructureType::Overline);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::Bar { position: BarPosition::Top, .. }));

        // Test underline
        let cmd = InsertStructure::new(StructureType::Underline);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::Bar { position: BarPosition::Bottom, .. }));
    }

    #[test]
    fn test_insert_structure_braces() {
        // Test overbrace
        let cmd = InsertStructure::new(StructureType::Overbrace);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::GroupChar { position: BarPosition::Top, .. }));

        // Test underbrace
        let cmd = InsertStructure::new(StructureType::Underbrace);
        let result = cmd.execute().unwrap();
        assert!(matches!(result, MathNode::GroupChar { position: BarPosition::Bottom, .. }));
    }

    #[test]
    fn test_equation_display_mode() {
        assert_eq!(EquationDisplayMode::default(), EquationDisplayMode::Inline);
    }

    #[test]
    fn test_insert_equation_at_position() {
        let cmd = InsertEquation::empty_inline().at_position(5, 10);
        assert_eq!(cmd.position, Some((5, 10)));
    }
}
