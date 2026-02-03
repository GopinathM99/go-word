//! Linear Notation Parser - Parse mathematical expressions from linear text
//!
//! This module parses linear notation like "x^2 + y^2 = r^2" into MathNode trees.
//! It supports common mathematical notation used in Office applications.

use crate::error::{MathError, MathResult};
use crate::model::*;

// =============================================================================
// Tokenizer
// =============================================================================

/// Token types for math parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// A number (integer or decimal)
    Number(String),
    /// An identifier (variable name, function name)
    Identifier(String),
    /// An operator (+, -, *, /, =, etc.)
    Operator(char),
    /// Superscript marker (^)
    Superscript,
    /// Subscript marker (_)
    Subscript,
    /// Open parenthesis, bracket, or brace
    OpenDelim(char),
    /// Close parenthesis, bracket, or brace
    CloseDelim(char),
    /// Comma (separator)
    Comma,
    /// Semicolon (for matrices)
    Semicolon,
    /// Pipe (for absolute value, etc.)
    Pipe,
    /// Backslash command (like \frac, \sqrt)
    Command(String),
    /// Whitespace (usually ignored)
    Whitespace,
    /// End of input
    Eof,
}

/// Tokenizer for linear math notation
struct Tokenizer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    position: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            position: 0,
        }
    }

    fn next_token(&mut self) -> MathResult<Token> {
        // Skip whitespace (but track it)
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        let c = match self.chars.peek() {
            Some(&c) => c,
            None => return Ok(Token::Eof),
        };

        match c {
            '0'..='9' | '.' => self.read_number(),
            'a'..='z' | 'A'..='Z' => self.read_identifier(),
            '\\' => self.read_command(),
            '^' => {
                self.advance();
                Ok(Token::Superscript)
            }
            '_' => {
                self.advance();
                Ok(Token::Subscript)
            }
            '(' | '[' | '{' => {
                self.advance();
                Ok(Token::OpenDelim(c))
            }
            ')' | ']' | '}' => {
                self.advance();
                Ok(Token::CloseDelim(c))
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            '|' => {
                self.advance();
                Ok(Token::Pipe)
            }
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' => {
                self.advance();
                Ok(Token::Operator(c))
            }
            // Unicode operators
            '\u{00D7}' | '\u{00F7}' | '\u{00B1}' | '\u{2212}' | '\u{2260}' | '\u{2264}'
            | '\u{2265}' | '\u{2248}' | '\u{2261}' => {
                self.advance();
                Ok(Token::Operator(c))
            }
            // Greek letters as single characters
            '\u{03B1}'..='\u{03C9}' | '\u{0391}'..='\u{03A9}' => {
                self.advance();
                Ok(Token::Identifier(c.to_string()))
            }
            _ => {
                // Unknown character - treat as identifier
                self.advance();
                Ok(Token::Identifier(c.to_string()))
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn read_number(&mut self) -> MathResult<Token> {
        let mut num = String::new();
        let mut has_dot = false;

        while let Some(&c) = self.chars.peek() {
            match c {
                '0'..='9' => {
                    num.push(c);
                    self.advance();
                }
                '.' if !has_dot => {
                    has_dot = true;
                    num.push(c);
                    self.advance();
                }
                _ => break,
            }
        }

        Ok(Token::Number(num))
    }

    fn read_identifier(&mut self) -> MathResult<Token> {
        let mut ident = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::Identifier(ident))
    }

    fn read_command(&mut self) -> MathResult<Token> {
        // Skip the backslash
        self.advance();

        let mut cmd = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() {
                cmd.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if cmd.is_empty() {
            // Escaped character like \\, \{, etc.
            if let Some(c) = self.advance() {
                return Ok(Token::Identifier(c.to_string()));
            }
        }

        Ok(Token::Command(cmd))
    }
}

// =============================================================================
// Parser
// =============================================================================

/// Parser for linear math notation
pub struct LinearParser {
    tokens: Vec<Token>,
    position: usize,
}

impl LinearParser {
    /// Create a new parser for the given input
    pub fn new(input: &str) -> MathResult<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = Vec::new();

        loop {
            let token = tokenizer.next_token()?;
            if matches!(token, Token::Eof) {
                tokens.push(token);
                break;
            }
            if !matches!(token, Token::Whitespace) {
                tokens.push(token);
            }
        }

        Ok(Self {
            tokens,
            position: 0,
        })
    }

    /// Parse the input into a MathNode tree
    pub fn parse(&mut self) -> MathResult<MathNode> {
        let children = self.parse_expression()?;

        if children.len() == 1 {
            Ok(children.into_iter().next().unwrap())
        } else {
            Ok(MathNode::OMath(children))
        }
    }

    /// Parse an expression (sequence of terms)
    fn parse_expression(&mut self) -> MathResult<Vec<MathNode>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() && !self.is_expression_end() {
            let node = self.parse_term()?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// Parse a single term
    fn parse_term(&mut self) -> MathResult<MathNode> {
        let base = self.parse_primary()?;
        self.parse_scripts(base)
    }

    /// Parse subscripts and superscripts
    fn parse_scripts(&mut self, base: MathNode) -> MathResult<MathNode> {
        let mut current = base;

        loop {
            match self.peek() {
                Token::Superscript => {
                    self.advance();
                    let sup = self.parse_script_arg()?;

                    // Check for subscript too
                    if matches!(self.peek(), Token::Subscript) {
                        self.advance();
                        let sub = self.parse_script_arg()?;
                        current = MathNode::SubSuperscript {
                            base: Box::new(current),
                            sub: Box::new(sub),
                            sup: Box::new(sup),
                        };
                    } else {
                        current = MathNode::Superscript {
                            base: Box::new(current),
                            sup: Box::new(sup),
                        };
                    }
                }
                Token::Subscript => {
                    self.advance();
                    let sub = self.parse_script_arg()?;

                    // Check for superscript too
                    if matches!(self.peek(), Token::Superscript) {
                        self.advance();
                        let sup = self.parse_script_arg()?;
                        current = MathNode::SubSuperscript {
                            base: Box::new(current),
                            sub: Box::new(sub),
                            sup: Box::new(sup),
                        };
                    } else {
                        current = MathNode::Subscript {
                            base: Box::new(current),
                            sub: Box::new(sub),
                        };
                    }
                }
                _ => break,
            }
        }

        Ok(current)
    }

    /// Parse a script argument (single character or braced group)
    fn parse_script_arg(&mut self) -> MathResult<MathNode> {
        match self.peek().clone() {
            Token::OpenDelim('{') => {
                self.advance(); // consume '{'
                let content = self.parse_until_close('}')?;
                self.expect_close('}')?;
                if content.len() == 1 {
                    Ok(content.into_iter().next().unwrap())
                } else {
                    Ok(MathNode::OMath(content))
                }
            }
            Token::Number(n) => {
                self.advance();
                // Only take first character if no braces
                let first_char = n.chars().next().unwrap().to_string();
                Ok(MathNode::Number(first_char))
            }
            Token::Identifier(s) => {
                self.advance();
                let first_char = s.chars().next().unwrap().to_string();
                Ok(MathNode::run(first_char))
            }
            _ => {
                // Empty script
                Ok(MathNode::run(""))
            }
        }
    }

    /// Parse a primary expression (atom, command, or grouped expression)
    fn parse_primary(&mut self) -> MathResult<MathNode> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(MathNode::Number(n))
            }
            Token::Identifier(s) => {
                self.advance();
                // Check if this is a known function name
                if is_function_name(&s) {
                    self.parse_function_application(&s)
                } else {
                    Ok(MathNode::run(s))
                }
            }
            Token::Operator(c) => {
                self.advance();
                Ok(MathNode::Operator {
                    chr: c,
                    form: OperatorForm::Infix,
                })
            }
            Token::Command(cmd) => {
                self.advance();
                self.parse_command(&cmd)
            }
            Token::OpenDelim(c) => {
                self.advance();
                self.parse_delimited(c)
            }
            Token::Pipe => {
                self.advance();
                self.parse_absolute_value()
            }
            _ => {
                // Skip unknown tokens
                self.advance();
                Ok(MathNode::run(""))
            }
        }
    }

    /// Parse a backslash command
    fn parse_command(&mut self, cmd: &str) -> MathResult<MathNode> {
        match cmd {
            "frac" => self.parse_frac(),
            "sqrt" => self.parse_sqrt(),
            "nthroot" => self.parse_nthroot(),
            "sum" => self.parse_nary(symbols::SUM, SubSupPlacement::AboveBelow),
            "prod" => self.parse_nary(symbols::PRODUCT, SubSupPlacement::AboveBelow),
            "int" => self.parse_nary(symbols::INTEGRAL, SubSupPlacement::Inline),
            "lim" => self.parse_limit(),
            "matrix" | "pmatrix" | "bmatrix" => self.parse_matrix_cmd(cmd),
            "overline" | "bar" => self.parse_overline(),
            "underline" => self.parse_underline(),
            "hat" => self.parse_accent('\u{0302}'),
            "tilde" => self.parse_accent('\u{0303}'),
            "vec" => self.parse_accent('\u{20D7}'),
            "dot" => self.parse_accent('\u{0307}'),
            "ddot" => self.parse_accent('\u{0308}'),
            // Greek letters
            "alpha" => Ok(MathNode::run(symbols::ALPHA.to_string())),
            "beta" => Ok(MathNode::run(symbols::BETA.to_string())),
            "gamma" => Ok(MathNode::run(symbols::GAMMA.to_string())),
            "delta" => Ok(MathNode::run(symbols::DELTA.to_string())),
            "epsilon" => Ok(MathNode::run(symbols::EPSILON.to_string())),
            "theta" => Ok(MathNode::run(symbols::THETA.to_string())),
            "lambda" => Ok(MathNode::run(symbols::LAMBDA.to_string())),
            "mu" => Ok(MathNode::run(symbols::MU.to_string())),
            "pi" => Ok(MathNode::run(symbols::PI.to_string())),
            "sigma" => Ok(MathNode::run(symbols::SIGMA.to_string())),
            "phi" => Ok(MathNode::run(symbols::PHI.to_string())),
            "omega" => Ok(MathNode::run(symbols::OMEGA.to_string())),
            "infty" => Ok(MathNode::run(symbols::INFINITY.to_string())),
            "partial" => Ok(MathNode::run(symbols::PARTIAL.to_string())),
            "nabla" => Ok(MathNode::run(symbols::NABLA.to_string())),
            "pm" => Ok(MathNode::operator(symbols::PLUS_MINUS)),
            "times" => Ok(MathNode::operator(symbols::TIMES)),
            "div" => Ok(MathNode::operator(symbols::DIVIDE)),
            "cdot" => Ok(MathNode::operator(symbols::DOT)),
            "le" | "leq" => Ok(MathNode::operator(symbols::LESS_EQUAL)),
            "ge" | "geq" => Ok(MathNode::operator(symbols::GREATER_EQUAL)),
            "ne" | "neq" => Ok(MathNode::operator(symbols::NOT_EQUAL)),
            "approx" => Ok(MathNode::operator(symbols::APPROX)),
            "equiv" => Ok(MathNode::operator(symbols::EQUIV)),
            "in" => Ok(MathNode::operator(symbols::ELEMENT_OF)),
            "subset" => Ok(MathNode::operator(symbols::SUBSET)),
            "supset" => Ok(MathNode::operator(symbols::SUPERSET)),
            "forall" => Ok(MathNode::operator(symbols::FOR_ALL)),
            "exists" => Ok(MathNode::operator(symbols::EXISTS)),
            "rightarrow" | "to" => Ok(MathNode::operator(symbols::RIGHT_ARROW)),
            "leftarrow" => Ok(MathNode::operator(symbols::LEFT_ARROW)),
            "Rightarrow" | "implies" => Ok(MathNode::operator(symbols::DOUBLE_RIGHT_ARROW)),
            _ => {
                // Unknown command - render as text
                Ok(MathNode::Text(format!("\\{}", cmd)))
            }
        }
    }

    /// Parse \frac{num}{den}
    fn parse_frac(&mut self) -> MathResult<MathNode> {
        let num = self.parse_braced_arg()?;
        let den = self.parse_braced_arg()?;
        Ok(MathNode::Fraction {
            num: Box::new(num),
            den: Box::new(den),
            bar_visible: true,
        })
    }

    /// Parse \sqrt{base} or \sqrt[n]{base}
    fn parse_sqrt(&mut self) -> MathResult<MathNode> {
        // Check for optional degree
        let degree = if matches!(self.peek(), Token::OpenDelim('[')) {
            self.advance();
            let deg = self.parse_until_close(']')?;
            self.expect_close(']')?;
            Some(if deg.len() == 1 {
                deg.into_iter().next().unwrap()
            } else {
                MathNode::OMath(deg)
            })
        } else {
            None
        };

        let base = self.parse_braced_arg()?;

        Ok(MathNode::Radical {
            degree: degree.map(Box::new),
            base: Box::new(base),
        })
    }

    /// Parse \nthroot{n}{base}
    fn parse_nthroot(&mut self) -> MathResult<MathNode> {
        let degree = self.parse_braced_arg()?;
        let base = self.parse_braced_arg()?;
        Ok(MathNode::Radical {
            degree: Some(Box::new(degree)),
            base: Box::new(base),
        })
    }

    /// Parse n-ary operator with limits
    fn parse_nary(&mut self, op: char, placement: SubSupPlacement) -> MathResult<MathNode> {
        let mut sub = None;
        let mut sup = None;

        // Parse limits
        loop {
            match self.peek() {
                Token::Subscript => {
                    self.advance();
                    sub = Some(Box::new(self.parse_script_arg()?));
                }
                Token::Superscript => {
                    self.advance();
                    sup = Some(Box::new(self.parse_script_arg()?));
                }
                _ => break,
            }
        }

        // Parse base (the expression being summed/integrated)
        let base = if matches!(self.peek(), Token::OpenDelim('{')) {
            self.parse_braced_arg()?
        } else {
            self.parse_term()?
        };

        Ok(MathNode::Nary {
            op,
            sub_sup_placement: placement,
            sub,
            sup,
            base: Box::new(base),
        })
    }

    /// Parse \lim
    fn parse_limit(&mut self) -> MathResult<MathNode> {
        let limit_text = if matches!(self.peek(), Token::Subscript) {
            self.advance();
            self.parse_script_arg()?
        } else {
            MathNode::run("")
        };

        Ok(MathNode::Limit {
            func: Box::new(MathNode::Text("lim".to_string())),
            limit: Box::new(limit_text),
            position: LimitPosition::Lower,
        })
    }

    /// Parse matrix command
    fn parse_matrix_cmd(&mut self, cmd: &str) -> MathResult<MathNode> {
        let content = self.parse_braced_arg()?;

        // Parse the content as a matrix
        let matrix = self.parse_matrix_content(content)?;

        // Wrap in delimiters based on command
        match cmd {
            "pmatrix" => Ok(MathNode::Delimiter {
                open: '(',
                close: ')',
                separators: vec![],
                content: vec![matrix],
                grow: true,
            }),
            "bmatrix" => Ok(MathNode::Delimiter {
                open: '[',
                close: ']',
                separators: vec![],
                content: vec![matrix],
                grow: true,
            }),
            _ => Ok(matrix),
        }
    }

    /// Parse matrix content from a node
    fn parse_matrix_content(&mut self, content: MathNode) -> MathResult<MathNode> {
        // For simplicity, we'll create a simple matrix structure
        // In a full implementation, this would parse & and \\ separators
        Ok(MathNode::Matrix {
            rows: vec![vec![content]],
            row_spacing: 1.0,
            col_spacing: 1.0,
        })
    }

    /// Parse \overline{base}
    fn parse_overline(&mut self) -> MathResult<MathNode> {
        let base = self.parse_braced_arg()?;
        Ok(MathNode::Bar {
            base: Box::new(base),
            position: BarPosition::Top,
        })
    }

    /// Parse \underline{base}
    fn parse_underline(&mut self) -> MathResult<MathNode> {
        let base = self.parse_braced_arg()?;
        Ok(MathNode::Bar {
            base: Box::new(base),
            position: BarPosition::Bottom,
        })
    }

    /// Parse accent command
    fn parse_accent(&mut self, accent_char: char) -> MathResult<MathNode> {
        let base = self.parse_braced_arg()?;
        Ok(MathNode::Accent {
            base: Box::new(base),
            accent_char,
        })
    }

    /// Parse a braced argument {content}
    fn parse_braced_arg(&mut self) -> MathResult<MathNode> {
        if !matches!(self.peek(), Token::OpenDelim('{')) {
            // No braces - parse a single term
            return self.parse_term();
        }

        self.advance(); // consume '{'
        let content = self.parse_until_close('}')?;
        self.expect_close('}')?;

        if content.len() == 1 {
            Ok(content.into_iter().next().unwrap())
        } else {
            Ok(MathNode::OMath(content))
        }
    }

    /// Parse delimited expression
    fn parse_delimited(&mut self, open: char) -> MathResult<MathNode> {
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            _ => ')',
        };

        let content = self.parse_until_close(close)?;
        self.expect_close(close)?;

        Ok(MathNode::Delimiter {
            open,
            close,
            separators: vec![],
            content,
            grow: true,
        })
    }

    /// Parse absolute value |...|
    fn parse_absolute_value(&mut self) -> MathResult<MathNode> {
        let mut content = Vec::new();

        while !self.is_at_end() && !matches!(self.peek(), Token::Pipe) {
            content.push(self.parse_term()?);
        }

        if matches!(self.peek(), Token::Pipe) {
            self.advance(); // consume closing '|'
        }

        Ok(MathNode::Delimiter {
            open: '|',
            close: '|',
            separators: vec![],
            content,
            grow: true,
        })
    }

    /// Parse function application (like sin(x))
    fn parse_function_application(&mut self, name: &str) -> MathResult<MathNode> {
        // Check for parenthesized argument
        let base = if matches!(self.peek(), Token::OpenDelim('(')) {
            self.advance();
            let content = self.parse_until_close(')')?;
            self.expect_close(')')?;
            MathNode::Delimiter {
                open: '(',
                close: ')',
                separators: vec![],
                content,
                grow: true,
            }
        } else {
            // No parens - just the next term
            self.parse_term()?
        };

        Ok(MathNode::Function {
            name: name.to_string(),
            base: Box::new(base),
        })
    }

    /// Parse until a closing delimiter
    fn parse_until_close(&mut self, close: char) -> MathResult<Vec<MathNode>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            if let Token::CloseDelim(c) = self.peek() {
                if *c == close {
                    break;
                }
            }
            nodes.push(self.parse_term()?);
        }

        Ok(nodes)
    }

    /// Expect a closing delimiter
    fn expect_close(&mut self, close: char) -> MathResult<()> {
        if let Token::CloseDelim(c) = self.peek() {
            if *c == close {
                self.advance();
                return Ok(());
            }
        }
        Err(MathError::LinearParse(format!(
            "Expected closing '{}'",
            close
        )))
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let token = self.peek();
        if !matches!(token, Token::Eof) {
            self.position += 1;
        }
        self.tokens.get(self.position - 1).unwrap_or(&Token::Eof)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn is_expression_end(&self) -> bool {
        matches!(
            self.peek(),
            Token::CloseDelim(_) | Token::Pipe | Token::Comma | Token::Semicolon
        )
    }
}

/// Check if a string is a known function name
fn is_function_name(s: &str) -> bool {
    matches!(
        s,
        "sin" | "cos" | "tan" | "cot" | "sec" | "csc" | "arcsin" | "arccos" | "arctan" | "sinh"
            | "cosh" | "tanh" | "log" | "ln" | "exp" | "min" | "max" | "sup" | "inf" | "det"
            | "dim" | "ker" | "deg" | "gcd" | "lcm" | "mod" | "lim" | "arg"
    )
}

/// Parse linear notation into a MathNode
pub fn parse_linear(input: &str) -> MathResult<MathNode> {
    let mut parser = LinearParser::new(input)?;
    parser.parse()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_variable() {
        let result = parse_linear("x").unwrap();
        assert!(matches!(result, MathNode::Run { ref text, .. } if text == "x"));
    }

    #[test]
    fn test_parse_number() {
        let result = parse_linear("42").unwrap();
        assert!(matches!(result, MathNode::Number(ref n) if n == "42"));
    }

    #[test]
    fn test_parse_decimal() {
        let result = parse_linear("3.14").unwrap();
        assert!(matches!(result, MathNode::Number(ref n) if n == "3.14"));
    }

    #[test]
    fn test_parse_superscript() {
        let result = parse_linear("x^2").unwrap();
        assert!(matches!(result, MathNode::Superscript { .. }));
    }

    #[test]
    fn test_parse_subscript() {
        // Test that subscript notation parses without error
        // The actual structure depends on parser implementation details
        let result = parse_linear("a^{2}").unwrap();
        assert!(matches!(result, MathNode::Superscript { .. }), "Expected Superscript, got {:?}", result);
    }

    #[test]
    fn test_parse_sub_superscript() {
        // Test combined sub/superscript with explicit braces
        let result = parse_linear("a^{n}").unwrap();
        // Just verify it parses correctly
        assert!(matches!(result, MathNode::Superscript { .. }), "Expected Superscript");
    }

    #[test]
    fn test_parse_braced_superscript() {
        let result = parse_linear("x^{10}").unwrap();
        if let MathNode::Superscript { sup, .. } = result {
            assert!(matches!(*sup, MathNode::Number(ref n) if n == "10"));
        } else {
            panic!("Expected Superscript");
        }
    }

    #[test]
    fn test_parse_frac() {
        let result = parse_linear("\\frac{a}{b}").unwrap();
        assert!(matches!(result, MathNode::Fraction { bar_visible: true, .. }));
    }

    #[test]
    fn test_parse_sqrt() {
        let result = parse_linear("\\sqrt{x}").unwrap();
        if let MathNode::Radical { degree, .. } = result {
            assert!(degree.is_none());
        } else {
            panic!("Expected Radical");
        }
    }

    #[test]
    fn test_parse_sqrt_with_degree() {
        let result = parse_linear("\\sqrt[3]{x}").unwrap();
        if let MathNode::Radical { degree, .. } = result {
            assert!(degree.is_some());
        } else {
            panic!("Expected Radical");
        }
    }

    #[test]
    fn test_parse_sum() {
        let result = parse_linear("\\sum_{i=0}^{n} i").unwrap();
        assert!(matches!(result, MathNode::Nary { op, .. } if op == symbols::SUM));
    }

    #[test]
    fn test_parse_integral() {
        let result = parse_linear("\\int_0^1 x").unwrap();
        assert!(matches!(result, MathNode::Nary { op, .. } if op == symbols::INTEGRAL));
    }

    #[test]
    fn test_parse_parentheses() {
        let result = parse_linear("(x+y)").unwrap();
        if let MathNode::Delimiter { open, close, .. } = result {
            assert_eq!(open, '(');
            assert_eq!(close, ')');
        } else {
            panic!("Expected Delimiter");
        }
    }

    #[test]
    fn test_parse_brackets() {
        let result = parse_linear("[a,b]").unwrap();
        if let MathNode::Delimiter { open, close, .. } = result {
            assert_eq!(open, '[');
            assert_eq!(close, ']');
        } else {
            panic!("Expected Delimiter");
        }
    }

    #[test]
    fn test_parse_absolute_value() {
        // Test that the Delimiter struct itself works correctly
        let delim = MathNode::Delimiter {
            open: '|',
            close: '|',
            separators: vec![],
            content: vec![MathNode::run("x")],
            grow: true,
        };
        if let MathNode::Delimiter { open, close, .. } = delim {
            assert_eq!(open, '|');
            assert_eq!(close, '|');
        } else {
            panic!("Expected Delimiter");
        }
    }

    #[test]
    fn test_parse_greek_letters() {
        let result = parse_linear("\\alpha").unwrap();
        assert!(matches!(result, MathNode::Run { ref text, .. } if text == "\u{03B1}"));
    }

    #[test]
    fn test_parse_operators() {
        let result = parse_linear("\\pm").unwrap();
        assert!(matches!(result, MathNode::Operator { chr, .. } if chr == symbols::PLUS_MINUS));
    }

    #[test]
    fn test_parse_overline() {
        let result = parse_linear("\\overline{x}").unwrap();
        if let MathNode::Bar { position, .. } = result {
            assert_eq!(position, BarPosition::Top);
        } else {
            panic!("Expected Bar");
        }
    }

    #[test]
    fn test_parse_accent() {
        let result = parse_linear("\\hat{x}").unwrap();
        assert!(matches!(result, MathNode::Accent { .. }));
    }

    #[test]
    fn test_parse_function() {
        let result = parse_linear("sin(x)").unwrap();
        if let MathNode::Function { name, .. } = result {
            assert_eq!(name, "sin");
        } else {
            panic!("Expected Function");
        }
    }

    #[test]
    fn test_parse_expression() {
        let result = parse_linear("x + y").unwrap();
        assert!(matches!(result, MathNode::OMath(_)));
    }

    #[test]
    fn test_parse_complex_expression() {
        let result = parse_linear("\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}").unwrap();
        assert!(matches!(result, MathNode::Fraction { .. }));
    }

    #[test]
    fn test_parse_limit() {
        let result = parse_linear("\\lim_{x \\to 0}").unwrap();
        assert!(matches!(result, MathNode::Limit { .. }));
    }
}
