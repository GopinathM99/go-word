//! RTF Parser - Parses RTF content into DocumentTree
//!
//! This module implements a streaming RTF parser that handles:
//! - Control words and their parameters
//! - Groups (nested braces)
//! - Character encoding (ANSI, Unicode escapes)
//! - Text content

use crate::rtf::control_words::*;
use crate::rtf::error::{RtfError, RtfResult};
use crate::rtf::api::{ImportWarning, WarningKind};
use doc_model::{
    Alignment, CharacterProperties, DocumentTree, ImageNode, ImageProperties,
    LineSpacing, Node, Paragraph, ParagraphProperties, ResourceId, Run, StyleId,
    Table, TableCell, TableGrid, TableRow, GridColumn, TableWidth, WidthType,
    CellProperties, RowProperties,
};
use std::collections::HashMap;

/// Token types in RTF
#[derive(Debug, Clone, PartialEq)]
pub enum RtfToken {
    /// Start of a group '{'
    GroupStart,
    /// End of a group '}'
    GroupEnd,
    /// Control word with optional numeric parameter
    ControlWord {
        name: String,
        param: Option<i32>,
    },
    /// Control symbol (e.g., \\ \{ \})
    ControlSymbol(char),
    /// Plain text content
    Text(String),
    /// Binary data (for images)
    BinaryData(Vec<u8>),
    /// Hex data (for images in RTF)
    HexData(String),
}

/// RTF tokenizer - converts RTF text into tokens
pub struct RtfTokenizer<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> RtfTokenizer<'a> {
    /// Create a new tokenizer for the given input
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }

    /// Get the current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if we've reached the end
    pub fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Peek at the current byte without advancing
    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    /// Get the current byte and advance
    fn advance(&mut self) -> Option<u8> {
        let byte = self.input.get(self.position).copied();
        if byte.is_some() {
            self.position += 1;
        }
        byte
    }

    /// Skip whitespace (single space after control word)
    fn skip_delimiter(&mut self) {
        if let Some(b' ') = self.peek() {
            self.advance();
        }
    }

    /// Read the next token
    pub fn next_token(&mut self) -> RtfResult<Option<RtfToken>> {
        // Skip line breaks (they're not significant in RTF)
        while matches!(self.peek(), Some(b'\r') | Some(b'\n')) {
            self.advance();
        }

        let byte = match self.peek() {
            Some(b) => b,
            None => return Ok(None),
        };

        match byte {
            b'{' => {
                self.advance();
                Ok(Some(RtfToken::GroupStart))
            }
            b'}' => {
                self.advance();
                Ok(Some(RtfToken::GroupEnd))
            }
            b'\\' => {
                self.advance();
                self.read_control()
            }
            _ => self.read_text(),
        }
    }

    /// Read a control word or control symbol
    fn read_control(&mut self) -> RtfResult<Option<RtfToken>> {
        let byte = match self.peek() {
            Some(b) => b,
            None => return Err(RtfError::UnexpectedEof),
        };

        // Check for control symbol
        if !byte.is_ascii_alphabetic() {
            self.advance();
            match byte {
                b'\'' => {
                    // Hex character escape \'XX
                    let hex = self.read_hex_byte()?;
                    let ch = char::from(hex);
                    return Ok(Some(RtfToken::Text(ch.to_string())));
                }
                b'*' => {
                    // Ignorable destination - skip to end of group
                    return Ok(Some(RtfToken::ControlSymbol('*')));
                }
                b'\\' | b'{' | b'}' => {
                    return Ok(Some(RtfToken::Text((byte as char).to_string())));
                }
                b'\r' | b'\n' => {
                    // Line break after backslash - treat as \par
                    return Ok(Some(RtfToken::ControlWord {
                        name: "par".to_string(),
                        param: None,
                    }));
                }
                _ => {
                    return Ok(Some(RtfToken::ControlSymbol(byte as char)));
                }
            }
        }

        // Read control word name
        let mut name = String::new();
        while let Some(b) = self.peek() {
            if b.is_ascii_alphabetic() {
                name.push(b as char);
                self.advance();
            } else {
                break;
            }
        }

        // Read optional numeric parameter
        let param = if let Some(b) = self.peek() {
            if b == b'-' || b.is_ascii_digit() {
                Some(self.read_number()?)
            } else {
                None
            }
        } else {
            None
        };

        // Skip delimiter (single space after control word)
        self.skip_delimiter();

        Ok(Some(RtfToken::ControlWord { name, param }))
    }

    /// Read a numeric parameter (possibly negative)
    fn read_number(&mut self) -> RtfResult<i32> {
        let mut num_str = String::new();

        // Check for negative sign
        if let Some(b'-') = self.peek() {
            num_str.push('-');
            self.advance();
        }

        // Read digits
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                num_str.push(b as char);
                self.advance();
            } else {
                break;
            }
        }

        num_str.parse::<i32>().map_err(|_| {
            RtfError::parse_error(self.position, format!("Invalid number: {}", num_str))
        })
    }

    /// Read a hex byte (\'XX format)
    fn read_hex_byte(&mut self) -> RtfResult<u8> {
        let mut hex = String::new();
        for _ in 0..2 {
            if let Some(b) = self.advance() {
                if b.is_ascii_hexdigit() {
                    hex.push(b as char);
                } else {
                    return Err(RtfError::parse_error(
                        self.position,
                        format!("Invalid hex character: {}", b as char),
                    ));
                }
            } else {
                return Err(RtfError::UnexpectedEof);
            }
        }
        u8::from_str_radix(&hex, 16).map_err(|_| {
            RtfError::parse_error(self.position, format!("Invalid hex byte: {}", hex))
        })
    }

    /// Read text content until a control character
    fn read_text(&mut self) -> RtfResult<Option<RtfToken>> {
        let mut text = String::new();

        while let Some(b) = self.peek() {
            match b {
                b'{' | b'}' | b'\\' | b'\r' | b'\n' => break,
                _ => {
                    text.push(b as char);
                    self.advance();
                }
            }
        }

        if text.is_empty() {
            // Shouldn't happen, but handle gracefully
            self.next_token()
        } else {
            Ok(Some(RtfToken::Text(text)))
        }
    }

    /// Read hex data (for images)
    pub fn read_hex_data(&mut self) -> RtfResult<String> {
        let mut hex = String::new();
        while let Some(b) = self.peek() {
            if b.is_ascii_hexdigit() {
                hex.push(b as char);
                self.advance();
            } else if b == b'}' {
                break;
            } else if b.is_ascii_whitespace() {
                self.advance();
                continue;
            } else {
                break;
            }
        }
        Ok(hex)
    }
}

/// Character formatting state
#[derive(Debug, Clone, Default)]
struct CharState {
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strikethrough: Option<bool>,
    font_size: Option<f32>,
    font_index: Option<u32>,
    color_index: Option<u32>,
    superscript: bool,
    subscript: bool,
}

impl CharState {
    fn to_properties(&self, fonts: &HashMap<u32, String>, colors: &[String]) -> CharacterProperties {
        CharacterProperties {
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            strikethrough: self.strikethrough,
            font_size: self.font_size,
            font_family: self.font_index.and_then(|i| fonts.get(&i).cloned()),
            color: self.color_index.and_then(|i| colors.get(i as usize).cloned()),
            ..Default::default()
        }
    }
}

/// Paragraph formatting state
#[derive(Debug, Clone, Default)]
struct ParaState {
    alignment: Option<Alignment>,
    left_indent: Option<f32>,
    right_indent: Option<f32>,
    first_line_indent: Option<f32>,
    space_before: Option<f32>,
    space_after: Option<f32>,
    line_spacing: Option<f32>,
    line_spacing_mult: bool,
    keep_with_next: bool,
    keep_together: bool,
    page_break_before: bool,
    in_table: bool,
}

impl ParaState {
    fn to_properties(&self) -> ParagraphProperties {
        let line_spacing = if let Some(sl) = self.line_spacing {
            if self.line_spacing_mult {
                Some(LineSpacing::Multiple(sl / 240.0)) // Twips to multiple
            } else if sl > 0.0 {
                Some(LineSpacing::AtLeast(sl / 20.0)) // Twips to points
            } else if sl < 0.0 {
                Some(LineSpacing::Exact(-sl / 20.0))
            } else {
                None
            }
        } else {
            None
        };

        ParagraphProperties {
            alignment: self.alignment,
            indent_left: self.left_indent,
            indent_right: self.right_indent,
            indent_first_line: self.first_line_indent,
            space_before: self.space_before,
            space_after: self.space_after,
            line_spacing,
            keep_with_next: if self.keep_with_next { Some(true) } else { None },
            keep_together: if self.keep_together { Some(true) } else { None },
            page_break_before: if self.page_break_before { Some(true) } else { None },
            ..Default::default()
        }
    }
}

/// Table cell definition from RTF
#[derive(Debug, Clone, Default)]
struct CellDef {
    right_boundary: i32, // In twips
    // Cell properties can be expanded
}

/// Table row state
#[derive(Debug, Clone, Default)]
struct TableState {
    cell_defs: Vec<CellDef>,
    current_cell: usize,
    row_height: Option<f32>,
}

/// Parser state
#[derive(Debug, Clone)]
struct ParseState {
    char_state: CharState,
    para_state: ParaState,
    table_state: Option<TableState>,
    dest: Option<String>,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            char_state: CharState::default(),
            para_state: ParaState::default(),
            table_state: None,
            dest: None,
        }
    }
}

/// Main RTF parser
pub struct RtfParser {
    /// Font table: index -> font name
    fonts: HashMap<u32, String>,
    /// Color table
    colors: Vec<String>,
    /// Stack of states for nested groups
    state_stack: Vec<ParseState>,
    /// Current state
    current_state: ParseState,
    /// Accumulated warnings
    warnings: Vec<ImportWarning>,
    /// Image counter for resource IDs
    image_counter: u32,
    /// Unicode skip count (for \uc)
    unicode_skip: u32,
}

impl RtfParser {
    /// Create a new RTF parser
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            colors: vec!["#000000".to_string()], // Default black
            state_stack: Vec::new(),
            current_state: ParseState::default(),
            warnings: Vec::new(),
            image_counter: 0,
            unicode_skip: 1,
        }
    }

    /// Parse RTF content and return a DocumentTree
    pub fn parse(&mut self, content: &[u8]) -> RtfResult<(DocumentTree, Vec<ImportWarning>)> {
        let mut tokenizer = RtfTokenizer::new(content);
        let mut tree = DocumentTree::new();

        // Current paragraph being built
        let mut current_para: Option<Paragraph> = None;
        let mut current_text = String::new();

        // Table state
        let mut table_rows: Vec<(TableRow, Vec<(TableCell, Vec<(Paragraph, Vec<Run>)>)>)> = Vec::new();
        let mut current_table_cells: Vec<(TableCell, Vec<(Paragraph, Vec<Run>)>)> = Vec::new();
        let mut current_cell_paras: Vec<(Paragraph, Vec<Run>)> = Vec::new();

        // Check for RTF header
        let first_token = tokenizer.next_token()?;
        if !matches!(first_token, Some(RtfToken::GroupStart)) {
            return Err(RtfError::invalid_structure("RTF must start with '{'"));
        }

        // Process tokens
        while let Some(token) = tokenizer.next_token()? {
            match token {
                RtfToken::GroupStart => {
                    self.state_stack.push(self.current_state.clone());
                }
                RtfToken::GroupEnd => {
                    // Check for special destinations ending
                    let was_dest = self.current_state.dest.clone();

                    if let Some(state) = self.state_stack.pop() {
                        self.current_state = state;
                    }

                    // Handle end of font table, color table, etc.
                    if matches!(was_dest.as_deref(), Some("fonttbl") | Some("colortbl") | Some("stylesheet")) {
                        // Destinations are fully processed
                    }
                }
                RtfToken::ControlWord { name, param } => {
                    match name.as_str() {
                        RTF => {
                            // RTF version - param is version number
                        }
                        ANSI | "mac" | "pc" | "pca" => {
                            // Character set - we default to UTF-8 output
                        }
                        DEFF => {
                            // Default font
                        }
                        FONTTBL => {
                            self.current_state.dest = Some("fonttbl".to_string());
                            self.parse_font_table(&mut tokenizer)?;
                            self.current_state.dest = None;
                        }
                        COLORTBL => {
                            self.current_state.dest = Some("colortbl".to_string());
                            self.parse_color_table(&mut tokenizer)?;
                            self.current_state.dest = None;
                        }
                        STYLESHEET => {
                            self.current_state.dest = Some("stylesheet".to_string());
                            self.skip_group(&mut tokenizer)?;
                            self.current_state.dest = None;
                        }
                        INFO => {
                            self.current_state.dest = Some("info".to_string());
                            self.skip_group(&mut tokenizer)?;
                            self.current_state.dest = None;
                        }
                        // Character formatting
                        B => {
                            self.current_state.char_state.bold = Some(param.unwrap_or(1) != 0);
                        }
                        I => {
                            self.current_state.char_state.italic = Some(param.unwrap_or(1) != 0);
                        }
                        UL => {
                            self.current_state.char_state.underline = Some(param.unwrap_or(1) != 0);
                        }
                        ULNONE => {
                            self.current_state.char_state.underline = Some(false);
                        }
                        STRIKE => {
                            self.current_state.char_state.strikethrough = Some(param.unwrap_or(1) != 0);
                        }
                        FS => {
                            // Font size in half-points
                            if let Some(size) = param {
                                self.current_state.char_state.font_size = Some(size as f32 / 2.0);
                            }
                        }
                        F => {
                            // Font index
                            if let Some(idx) = param {
                                self.current_state.char_state.font_index = Some(idx as u32);
                            }
                        }
                        CF => {
                            // Color foreground index
                            if let Some(idx) = param {
                                self.current_state.char_state.color_index = Some(idx as u32);
                            }
                        }
                        PLAIN => {
                            // Reset character formatting
                            self.current_state.char_state = CharState::default();
                        }
                        SUPER => {
                            self.current_state.char_state.superscript = true;
                            self.current_state.char_state.subscript = false;
                        }
                        SUB => {
                            self.current_state.char_state.subscript = true;
                            self.current_state.char_state.superscript = false;
                        }
                        NOSUPERSUB => {
                            self.current_state.char_state.superscript = false;
                            self.current_state.char_state.subscript = false;
                        }
                        // Paragraph formatting
                        PARD => {
                            // Reset paragraph formatting
                            self.current_state.para_state = ParaState::default();
                        }
                        PAR => {
                            // End of paragraph
                            if !current_text.is_empty() {
                                let run = self.create_run(&current_text);
                                current_text.clear();

                                if self.current_state.para_state.in_table {
                                    // Table cell content
                                    if current_cell_paras.is_empty() {
                                        let para = self.create_paragraph();
                                        current_cell_paras.push((para, vec![run]));
                                    } else {
                                        current_cell_paras.last_mut().unwrap().1.push(run);
                                    }
                                    // Start new paragraph in cell
                                    let para = self.create_paragraph();
                                    current_cell_paras.push((para, Vec::new()));
                                } else {
                                    // Regular paragraph
                                    let para = current_para.take().unwrap_or_else(|| self.create_paragraph());
                                    self.add_paragraph_to_tree(&mut tree, para, vec![run]);
                                }
                            } else if !self.current_state.para_state.in_table {
                                // Empty paragraph
                                let para = current_para.take().unwrap_or_else(|| self.create_paragraph());
                                self.add_paragraph_to_tree(&mut tree, para, Vec::new());
                            }
                            current_para = None;
                        }
                        QL => {
                            self.current_state.para_state.alignment = Some(Alignment::Left);
                        }
                        QC => {
                            self.current_state.para_state.alignment = Some(Alignment::Center);
                        }
                        QR => {
                            self.current_state.para_state.alignment = Some(Alignment::Right);
                        }
                        QJ => {
                            self.current_state.para_state.alignment = Some(Alignment::Justify);
                        }
                        LI => {
                            // Left indent in twips
                            if let Some(twips) = param {
                                self.current_state.para_state.left_indent = Some(twips as f32 / 20.0);
                            }
                        }
                        RI => {
                            // Right indent in twips
                            if let Some(twips) = param {
                                self.current_state.para_state.right_indent = Some(twips as f32 / 20.0);
                            }
                        }
                        FI => {
                            // First line indent in twips
                            if let Some(twips) = param {
                                self.current_state.para_state.first_line_indent = Some(twips as f32 / 20.0);
                            }
                        }
                        SB => {
                            // Space before in twips
                            if let Some(twips) = param {
                                self.current_state.para_state.space_before = Some(twips as f32 / 20.0);
                            }
                        }
                        SA => {
                            // Space after in twips
                            if let Some(twips) = param {
                                self.current_state.para_state.space_after = Some(twips as f32 / 20.0);
                            }
                        }
                        SL => {
                            // Line spacing
                            if let Some(val) = param {
                                self.current_state.para_state.line_spacing = Some(val as f32);
                            }
                        }
                        SLMULT => {
                            self.current_state.para_state.line_spacing_mult = param.unwrap_or(0) == 1;
                        }
                        KEEPN => {
                            self.current_state.para_state.keep_with_next = true;
                        }
                        KEEP => {
                            self.current_state.para_state.keep_together = true;
                        }
                        PAGEBB => {
                            self.current_state.para_state.page_break_before = true;
                        }
                        // Table formatting
                        TROWD => {
                            // Start table row definition
                            self.current_state.table_state = Some(TableState::default());
                            self.current_state.para_state.in_table = true;
                        }
                        TRRH => {
                            // Row height
                            if let Some(ref mut ts) = self.current_state.table_state {
                                if let Some(twips) = param {
                                    ts.row_height = Some((twips.abs() as f32) / 20.0);
                                }
                            }
                        }
                        CELLX => {
                            // Cell right boundary
                            if let Some(ref mut ts) = self.current_state.table_state {
                                if let Some(twips) = param {
                                    ts.cell_defs.push(CellDef { right_boundary: twips });
                                }
                            }
                        }
                        INTBL => {
                            self.current_state.para_state.in_table = true;
                        }
                        CELL => {
                            // End of cell content
                            if !current_text.is_empty() {
                                let run = self.create_run(&current_text);
                                current_text.clear();

                                if current_cell_paras.is_empty() {
                                    let para = self.create_paragraph();
                                    current_cell_paras.push((para, vec![run]));
                                } else {
                                    current_cell_paras.last_mut().unwrap().1.push(run);
                                }
                            } else if current_cell_paras.is_empty() {
                                // Empty cell - add empty paragraph
                                let para = self.create_paragraph();
                                current_cell_paras.push((para, Vec::new()));
                            }

                            // Create cell and store paragraphs
                            let cell = TableCell::new();
                            current_table_cells.push((cell, std::mem::take(&mut current_cell_paras)));
                        }
                        ROW => {
                            // End of row
                            let row = if let Some(ref ts) = self.current_state.table_state {
                                let mut row = TableRow::new();
                                if let Some(h) = ts.row_height {
                                    row.properties.height = Some(h);
                                }
                                row
                            } else {
                                TableRow::new()
                            };

                            table_rows.push((row, std::mem::take(&mut current_table_cells)));

                            // Check if this is the last row (no more \trowd follows)
                            // For simplicity, we'll build the table when we see content outside table
                            self.current_state.table_state = None;
                            self.current_state.para_state.in_table = false;
                        }
                        // Image handling
                        PICT => {
                            self.current_state.dest = Some("pict".to_string());
                            // Image parsing would be handled in a separate method
                            // For now, skip image groups
                            self.warnings.push(ImportWarning {
                                kind: WarningKind::UnsupportedFeature,
                                message: "Image import not fully implemented".to_string(),
                            });
                            self.skip_group(&mut tokenizer)?;
                        }
                        // Unicode
                        U => {
                            // Unicode character
                            if let Some(code) = param {
                                let ch = if code < 0 {
                                    // Negative codes for high Unicode
                                    char::from_u32((code as i64 + 65536) as u32)
                                } else {
                                    char::from_u32(code as u32)
                                };
                                if let Some(c) = ch {
                                    current_text.push(c);
                                    // Skip the following ANSI representation
                                    for _ in 0..self.unicode_skip {
                                        if let Some(RtfToken::Text(t)) = tokenizer.next_token()? {
                                            if t.len() <= 1 {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        UC => {
                            // Unicode skip count
                            if let Some(count) = param {
                                self.unicode_skip = count as u32;
                            }
                        }
                        // Special characters
                        LINE => {
                            current_text.push('\n');
                        }
                        TAB => {
                            current_text.push('\t');
                        }
                        // Ignorable destinations
                        "*" => {
                            // Skip ignorable destination
                            self.skip_group(&mut tokenizer)?;
                        }
                        _ => {
                            // Unknown control word - skip
                            // This is common for extended RTF features
                        }
                    }
                }
                RtfToken::ControlSymbol(ch) => {
                    match ch {
                        '*' => {
                            // Ignorable destination marker
                            // The next group should be skipped if we don't understand it
                        }
                        '~' => {
                            // Non-breaking space
                            current_text.push('\u{00A0}');
                        }
                        '-' => {
                            // Optional hyphen
                            current_text.push('\u{00AD}');
                        }
                        '_' => {
                            // Non-breaking hyphen
                            current_text.push('\u{2011}');
                        }
                        _ => {}
                    }
                }
                RtfToken::Text(text) => {
                    // Skip text in special destinations
                    if self.current_state.dest.is_some() {
                        continue;
                    }
                    current_text.push_str(&text);
                }
                RtfToken::BinaryData(_) | RtfToken::HexData(_) => {
                    // Binary/hex data is handled in specific destinations
                }
            }
        }

        // Build any pending table
        if !table_rows.is_empty() {
            self.build_table(&mut tree, std::mem::take(&mut table_rows))?;
        }

        // Add any remaining text as a paragraph
        if !current_text.is_empty() {
            let run = self.create_run(&current_text);
            let para = current_para.take().unwrap_or_else(|| self.create_paragraph());
            self.add_paragraph_to_tree(&mut tree, para, vec![run]);
        }

        // Ensure document has at least one paragraph
        if tree.paragraphs().count() == 0 {
            let para = Paragraph::new();
            let para_id = para.id();
            tree.nodes.paragraphs.insert(para_id, para);
            tree.document.add_body_child(para_id);
        }

        Ok((tree, std::mem::take(&mut self.warnings)))
    }

    /// Parse the font table
    fn parse_font_table(&mut self, tokenizer: &mut RtfTokenizer) -> RtfResult<()> {
        let mut depth = 1;
        let mut current_font_idx: Option<u32> = None;
        let mut current_font_name = String::new();

        while depth > 0 {
            let token = tokenizer.next_token()?;
            match token {
                Some(RtfToken::GroupStart) => {
                    depth += 1;
                }
                Some(RtfToken::GroupEnd) => {
                    // Save current font before closing
                    if let Some(idx) = current_font_idx.take() {
                        let name = current_font_name.trim().trim_end_matches(';').to_string();
                        if !name.is_empty() {
                            self.fonts.insert(idx, name);
                        }
                    }
                    current_font_name.clear();
                    depth -= 1;
                }
                Some(RtfToken::ControlWord { name, param }) => {
                    match name.as_str() {
                        F => {
                            if let Some(idx) = param {
                                current_font_idx = Some(idx as u32);
                            }
                        }
                        FCHARSET | FPRQ | FNIL | FROMAN | FSWISS | FMODERN | FSCRIPT | FDECOR | FTECH | FBIDI => {
                            // Font properties - we just need the name
                        }
                        _ => {}
                    }
                }
                Some(RtfToken::Text(text)) => {
                    current_font_name.push_str(&text);
                }
                None => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Parse the color table
    fn parse_color_table(&mut self, tokenizer: &mut RtfTokenizer) -> RtfResult<()> {
        let mut depth = 1;
        let mut r: u8 = 0;
        let mut g: u8 = 0;
        let mut b: u8 = 0;
        let mut has_color = false;

        // First entry is the "auto" color (index 0)
        // Don't overwrite it

        while depth > 0 {
            let token = tokenizer.next_token()?;
            match token {
                Some(RtfToken::GroupStart) => {
                    depth += 1;
                }
                Some(RtfToken::GroupEnd) => {
                    depth -= 1;
                }
                Some(RtfToken::ControlWord { name, param }) => {
                    match name.as_str() {
                        "red" => {
                            r = param.unwrap_or(0) as u8;
                            has_color = true;
                        }
                        "green" => {
                            g = param.unwrap_or(0) as u8;
                            has_color = true;
                        }
                        "blue" => {
                            b = param.unwrap_or(0) as u8;
                            has_color = true;
                        }
                        _ => {}
                    }
                }
                Some(RtfToken::Text(text)) => {
                    // Semicolon ends a color entry
                    if text.contains(';') {
                        if has_color {
                            let color = format!("#{:02X}{:02X}{:02X}", r, g, b);
                            self.colors.push(color);
                        } else {
                            // Auto color entry
                            self.colors.push("#000000".to_string());
                        }
                        r = 0;
                        g = 0;
                        b = 0;
                        has_color = false;
                    }
                }
                None => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Skip a group and all its contents
    fn skip_group(&mut self, tokenizer: &mut RtfTokenizer) -> RtfResult<()> {
        let mut depth = 1;

        while depth > 0 {
            match tokenizer.next_token()? {
                Some(RtfToken::GroupStart) => depth += 1,
                Some(RtfToken::GroupEnd) => depth -= 1,
                None => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Create a Run from the current text and character state
    fn create_run(&self, text: &str) -> Run {
        let props = self.current_state.char_state.to_properties(&self.fonts, &self.colors);
        Run::with_direct_formatting(text, props)
    }

    /// Create a Paragraph from the current paragraph state
    fn create_paragraph(&self) -> Paragraph {
        let props = self.current_state.para_state.to_properties();
        Paragraph::with_direct_formatting(props)
    }

    /// Add a paragraph with its runs to the tree
    fn add_paragraph_to_tree(&self, tree: &mut DocumentTree, para: Paragraph, runs: Vec<Run>) {
        let para_id = para.id();
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        for run in runs {
            let run_id = run.id();
            tree.nodes.runs.insert(run_id, run);
            if let Some(p) = tree.nodes.paragraphs.get_mut(&para_id) {
                p.add_child(run_id);
            }
        }
    }

    /// Build a table from collected rows
    fn build_table(
        &self,
        tree: &mut DocumentTree,
        rows: Vec<(TableRow, Vec<(TableCell, Vec<(Paragraph, Vec<Run>)>)>)>,
    ) -> RtfResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Determine number of columns from first row
        let col_count = rows.first().map(|(_, cells)| cells.len()).unwrap_or(0);
        if col_count == 0 {
            return Ok(());
        }

        // Create table with grid
        let grid = TableGrid::new(col_count);
        let mut table = Table::with_grid(grid);
        let table_id = table.id();

        // Add rows
        for (row, cells) in rows {
            let mut row = row;
            let row_id = row.id();
            row.set_parent(Some(table_id));

            // Add cells
            for (cell, paras) in cells {
                let mut cell = cell;
                let cell_id = cell.id();
                cell.set_parent(Some(row_id));

                // Add paragraphs to cell
                for (para, runs) in paras {
                    let mut para = para;
                    let para_id = para.id();
                    para.set_parent(Some(cell_id));

                    // Add runs to paragraph
                    for mut run in runs {
                        let run_id = run.id();
                        run.set_parent(Some(para_id));
                        tree.nodes.runs.insert(run_id, run);
                        para.add_child(run_id);
                    }

                    tree.nodes.paragraphs.insert(para_id, para);
                    cell.add_child(para_id);
                }

                tree.nodes.table_cells.insert(cell_id, cell);
                row.add_cell(cell_id);
            }

            tree.nodes.table_rows.insert(row_id, row);
            table.add_row(row_id);
        }

        tree.nodes.tables.insert(table_id, table);
        tree.document.add_body_child(table_id);

        Ok(())
    }
}

impl Default for RtfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_basic() {
        let rtf = b"{\\rtf1\\ansi Hello World}";
        let mut tokenizer = RtfTokenizer::new(rtf);

        assert!(matches!(tokenizer.next_token().unwrap(), Some(RtfToken::GroupStart)));
        assert!(matches!(tokenizer.next_token().unwrap(), Some(RtfToken::ControlWord { name, .. }) if name == "rtf"));
    }

    #[test]
    fn test_parse_simple_rtf() {
        let rtf = b"{\\rtf1\\ansi\\deff0 Hello World}";
        let mut parser = RtfParser::new();
        let (tree, warnings) = parser.parse(rtf).unwrap();

        let text = tree.text_content();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_parse_formatting() {
        let rtf = b"{\\rtf1\\ansi {\\b Bold} and {\\i Italic}}";
        let mut parser = RtfParser::new();
        let (tree, _) = parser.parse(rtf).unwrap();

        // Should have parsed text with formatting
        assert!(tree.paragraphs().count() >= 1);
    }

    #[test]
    fn test_parse_paragraph() {
        let rtf = b"{\\rtf1\\ansi First para\\par Second para}";
        let mut parser = RtfParser::new();
        let (tree, _) = parser.parse(rtf).unwrap();

        assert!(tree.paragraphs().count() >= 2);
    }

    #[test]
    fn test_unicode_escape() {
        let rtf = b"{\\rtf1\\ansi Test \\u8364? euro}"; // Euro sign
        let mut parser = RtfParser::new();
        let (tree, _) = parser.parse(rtf).unwrap();

        let text = tree.text_content();
        // The ? is the ANSI fallback, Unicode should be preferred
        assert!(text.contains("Test"));
    }
}
