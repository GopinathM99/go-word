//! Fields Import/Export for DOCX
//!
//! Handles complex field codes (w:fldChar, w:instrText) including:
//! - PAGE, NUMPAGES, TOC, REF, SEQ fields
//! - Nested fields
//! - Field preservation for unsupported field types

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use quick_xml::events::Event;
use std::collections::HashMap;

// Local type definitions to avoid dependency on doc_model's field types
// These are simplified versions for DOCX parsing/writing

/// Number format for field values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberFormat {
    #[default]
    Arabic,
    UppercaseRoman,
    LowercaseRoman,
    UppercaseLetter,
    LowercaseLetter,
    Ordinal,
    CardinalText,
    OrdinalText,
}

/// Type of REF display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RefDisplayType {
    #[default]
    Content,
    PageNumber,
    ParagraphNumber,
    ParagraphNumberFullContext,
    RelativePosition,
}

/// Options for REF fields
#[derive(Debug, Clone, Default)]
pub struct RefOptions {
    pub bookmark: String,
    pub display: RefDisplayType,
    pub hyperlink: bool,
    pub include_position: bool,
}

/// Options for SEQ fields
#[derive(Debug, Clone, Default)]
pub struct SeqOptions {
    pub identifier: String,
    pub format: NumberFormat,
    pub reset_at_heading_level: Option<u8>,
    pub current_only: bool,
    pub reset_to: Option<u32>,
    pub repeat_previous: bool,
}

/// Switches for TOC fields
#[derive(Debug, Clone)]
pub struct TocSwitches {
    pub heading_levels: std::ops::Range<u8>,
    pub include_page_numbers: bool,
    pub hyperlinks: bool,
    pub tab_leader: TocTabLeader,
    pub include_tc_fields: bool,
}

impl Default for TocSwitches {
    fn default() -> Self {
        Self {
            heading_levels: 1..4,
            include_page_numbers: true,
            hyperlinks: true,
            tab_leader: TocTabLeader::Dots,
            include_tc_fields: false,
        }
    }
}

/// Tab leader style for TOC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TocTabLeader {
    None,
    #[default]
    Dots,
    Dashes,
    Underline,
}

/// Field instruction types
#[derive(Debug, Clone)]
pub enum FieldInstruction {
    Page { format: NumberFormat },
    NumPages { format: NumberFormat },
    Date { format: String },
    Time { format: String },
    Toc { switches: TocSwitches },
    Ref { options: RefOptions },
    Seq { options: SeqOptions },
    Author,
    Title,
    Subject,
    FileName { include_path: bool },
    Section,
    SectionPages,
    Hyperlink { url: String, display_text: Option<String> },
    IncludeText { file_path: String },
    If { condition: String, true_text: String, false_text: String },
    PrintDate { format: String },
    SaveDate { format: String },
    CreateDate { format: String },
    EditTime,
    NumWords,
    NumChars,
    Custom { code: String },
}

/// A field with instruction and cached result
#[derive(Debug, Clone)]
pub struct Field {
    pub instruction: FieldInstruction,
    pub cached_text: Option<String>,
}

impl Field {
    /// Create a new field with the given instruction
    pub fn new(instruction: FieldInstruction) -> Self {
        Self {
            instruction,
            cached_text: None,
        }
    }

    /// Create a PAGE field
    pub fn page() -> Self {
        Self::new(FieldInstruction::Page { format: NumberFormat::Arabic })
    }

    /// Create a TOC field
    pub fn toc() -> Self {
        Self::new(FieldInstruction::Toc { switches: TocSwitches::default() })
    }
}

// =============================================================================
// Field Parser
// =============================================================================

/// Parser for field codes in DOCX
pub struct FieldParser {
    /// Stack of nested field codes being parsed
    field_stack: Vec<FieldParseState>,
    /// Completed parsed fields
    parsed_fields: Vec<ParsedField>,
}

/// State for parsing a single field
#[derive(Debug, Clone)]
struct FieldParseState {
    /// Whether we're currently in the instruction part
    in_instruction: bool,
    /// Whether we're in the result part
    in_result: bool,
    /// Accumulated instruction text
    instruction_text: String,
    /// Accumulated result text
    result_text: String,
    /// Nesting level for nested fields
    nesting_level: u32,
}

impl FieldParser {
    /// Create a new field parser
    pub fn new() -> Self {
        Self {
            field_stack: Vec::new(),
            parsed_fields: Vec::new(),
        }
    }

    /// Handle a w:fldChar element
    pub fn handle_fld_char(&mut self, e: &quick_xml::events::BytesStart) {
        let fld_char_type = XmlParser::get_w_attribute(e, "fldCharType")
            .unwrap_or_default();

        match fld_char_type.as_str() {
            "begin" => {
                // Start of a new field
                self.field_stack.push(FieldParseState {
                    in_instruction: true,
                    in_result: false,
                    instruction_text: String::new(),
                    result_text: String::new(),
                    nesting_level: self.field_stack.len() as u32,
                });
            }
            "separate" => {
                // Transition from instruction to result
                if let Some(state) = self.field_stack.last_mut() {
                    state.in_instruction = false;
                    state.in_result = true;
                }
            }
            "end" => {
                // End of field - finalize and pop
                if let Some(state) = self.field_stack.pop() {
                    let parsed = ParsedField {
                        instruction_text: state.instruction_text.trim().to_string(),
                        result_text: state.result_text,
                        nesting_level: state.nesting_level,
                    };
                    self.parsed_fields.push(parsed);
                }
            }
            _ => {}
        }
    }

    /// Handle w:instrText content
    pub fn handle_instr_text(&mut self, text: &str) {
        if let Some(state) = self.field_stack.last_mut() {
            if state.in_instruction {
                state.instruction_text.push_str(text);
            }
        }
    }

    /// Handle result text
    pub fn handle_result_text(&mut self, text: &str) {
        if let Some(state) = self.field_stack.last_mut() {
            if state.in_result {
                state.result_text.push_str(text);
            }
        }
    }

    /// Check if we're currently inside a field
    pub fn in_field(&self) -> bool {
        !self.field_stack.is_empty()
    }

    /// Check if we're in the instruction part of a field
    pub fn in_instruction(&self) -> bool {
        self.field_stack.last().map(|s| s.in_instruction).unwrap_or(false)
    }

    /// Check if we're in the result part of a field
    pub fn in_result(&self) -> bool {
        self.field_stack.last().map(|s| s.in_result).unwrap_or(false)
    }

    /// Get all parsed fields
    pub fn get_parsed_fields(&self) -> &[ParsedField] {
        &self.parsed_fields
    }

    /// Clear parsed fields
    pub fn clear(&mut self) {
        self.field_stack.clear();
        self.parsed_fields.clear();
    }

    /// Parse an instruction string into a Field
    pub fn parse_instruction(instruction: &str) -> DocxResult<Field> {
        let instruction = instruction.trim();

        // Extract the field code (first word)
        let parts: Vec<&str> = instruction.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Field::new(FieldInstruction::Custom {
                code: instruction.to_string(),
            }));
        }

        let code = parts[0].to_uppercase();

        let field_instruction = match code.as_str() {
            "PAGE" => {
                let format = Self::parse_number_format_switch(instruction);
                FieldInstruction::Page { format }
            }
            "NUMPAGES" => {
                let format = Self::parse_number_format_switch(instruction);
                FieldInstruction::NumPages { format }
            }
            "DATE" => {
                let format = Self::parse_date_format_switch(instruction)
                    .unwrap_or_else(|| "M/d/yyyy".to_string());
                FieldInstruction::Date { format }
            }
            "TIME" => {
                let format = Self::parse_date_format_switch(instruction)
                    .unwrap_or_else(|| "h:mm:ss AM/PM".to_string());
                FieldInstruction::Time { format }
            }
            "TOC" => {
                let switches = Self::parse_toc_switches(instruction);
                FieldInstruction::Toc { switches }
            }
            "REF" => {
                let options = Self::parse_ref_options(instruction);
                FieldInstruction::Ref { options }
            }
            "PAGEREF" => {
                // PAGEREF is similar to REF with page number display
                let bookmark = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
                FieldInstruction::Ref {
                    options: RefOptions {
                        bookmark,
                        display: RefDisplayType::PageNumber,
                        hyperlink: instruction.contains("\\h"),
                        include_position: instruction.contains("\\p"),
                    },
                }
            }
            "SEQ" => {
                let options = Self::parse_seq_options(instruction);
                FieldInstruction::Seq { options }
            }
            "AUTHOR" => FieldInstruction::Author,
            "TITLE" => FieldInstruction::Title,
            "SUBJECT" => FieldInstruction::Subject,
            "FILENAME" => {
                let include_path = instruction.contains("\\p");
                FieldInstruction::FileName { include_path }
            }
            "SECTION" => FieldInstruction::Section,
            "SECTIONPAGES" => FieldInstruction::SectionPages,
            "HYPERLINK" => {
                let (url, display_text) = Self::parse_hyperlink_instruction(instruction);
                FieldInstruction::Hyperlink { url, display_text }
            }
            "CREATEDATE" => {
                let format = Self::parse_date_format_switch(instruction)
                    .unwrap_or_else(|| "M/d/yyyy".to_string());
                FieldInstruction::CreateDate { format }
            }
            "SAVEDATE" => {
                let format = Self::parse_date_format_switch(instruction)
                    .unwrap_or_else(|| "M/d/yyyy".to_string());
                FieldInstruction::SaveDate { format }
            }
            "PRINTDATE" => {
                let format = Self::parse_date_format_switch(instruction)
                    .unwrap_or_else(|| "M/d/yyyy".to_string());
                FieldInstruction::PrintDate { format }
            }
            "EDITTIME" => FieldInstruction::EditTime,
            "NUMWORDS" => FieldInstruction::NumWords,
            "NUMCHARS" => FieldInstruction::NumChars,
            "IF" => {
                let (condition, true_text, false_text) = Self::parse_if_instruction(instruction);
                FieldInstruction::If {
                    condition,
                    true_text,
                    false_text,
                }
            }
            _ => {
                // Preserve unknown fields as custom
                FieldInstruction::Custom {
                    code: instruction.to_string(),
                }
            }
        };

        Ok(Field::new(field_instruction))
    }

    /// Parse number format switch (\\* format)
    fn parse_number_format_switch(instruction: &str) -> NumberFormat {
        if let Some(pos) = instruction.find("\\*") {
            let after = &instruction[pos + 2..].trim_start();
            let format_word = after.split_whitespace().next().unwrap_or("");

            match format_word.to_lowercase().as_str() {
                "arabic" => NumberFormat::Arabic,
                "roman" | "romanuc" | "upper" => NumberFormat::UppercaseRoman,
                "romanl" | "romanlc" | "lower" => NumberFormat::LowercaseRoman,
                "alphabetic" | "alpha" => NumberFormat::UppercaseLetter,
                "alphal" | "alphabeticl" => NumberFormat::LowercaseLetter,
                "ordinal" => NumberFormat::Ordinal,
                "cardtext" => NumberFormat::CardinalText,
                "ordtext" => NumberFormat::OrdinalText,
                _ => NumberFormat::Arabic,
            }
        } else {
            NumberFormat::Arabic
        }
    }

    /// Parse date format switch (\\@ "format")
    fn parse_date_format_switch(instruction: &str) -> Option<String> {
        if let Some(pos) = instruction.find("\\@") {
            let after = &instruction[pos + 2..].trim_start();
            // Look for quoted format
            if let Some(quote_start) = after.find('"') {
                let rest = &after[quote_start + 1..];
                if let Some(quote_end) = rest.find('"') {
                    return Some(rest[..quote_end].to_string());
                }
            }
            // Unquoted single word
            let format_word = after.split_whitespace().next().unwrap_or("");
            if !format_word.is_empty() && !format_word.starts_with('\\') {
                return Some(format_word.to_string());
            }
        }
        None
    }

    /// Parse TOC switches
    fn parse_toc_switches(instruction: &str) -> TocSwitches {
        let mut switches = TocSwitches::default();

        // Parse \\o "1-3" (outline levels)
        if let Some(pos) = instruction.find("\\o") {
            let after = &instruction[pos + 2..].trim_start();
            if let Some(quote_start) = after.find('"') {
                let rest = &after[quote_start + 1..];
                if let Some(quote_end) = rest.find('"') {
                    let range_str = &rest[..quote_end];
                    if let Some(dash_pos) = range_str.find('-') {
                        let start: u8 = range_str[..dash_pos].parse().unwrap_or(1);
                        let end: u8 = range_str[dash_pos + 1..].parse().unwrap_or(3);
                        switches.heading_levels = start..(end + 1);
                    }
                }
            }
        }

        // Parse \\h (hyperlinks)
        switches.hyperlinks = instruction.contains("\\h");

        // Parse \\n (no page numbers)
        switches.include_page_numbers = !instruction.contains("\\n");

        // Parse \\p (tab leader)
        if instruction.contains("\\p") {
            // Check for specific tab leader character after \\p
            if instruction.contains("\\p \"") {
                // Custom tab leader
            }
        }

        // Parse \\t (custom styles)
        if let Some(pos) = instruction.find("\\t") {
            // Custom style mapping would be parsed here
        }

        // Parse \\f (TC fields)
        switches.include_tc_fields = instruction.contains("\\f");

        switches
    }

    /// Parse REF options
    fn parse_ref_options(instruction: &str) -> RefOptions {
        let parts: Vec<&str> = instruction.split_whitespace().collect();

        let bookmark = if parts.len() > 1 {
            parts[1].trim_matches('"').to_string()
        } else {
            String::new()
        };

        let display = if instruction.contains("\\p") {
            RefDisplayType::PageNumber
        } else if instruction.contains("\\r") {
            RefDisplayType::RelativePosition
        } else if instruction.contains("\\n") {
            RefDisplayType::ParagraphNumber
        } else if instruction.contains("\\w") {
            RefDisplayType::ParagraphNumberFullContext
        } else {
            RefDisplayType::Content
        };

        RefOptions {
            bookmark,
            display,
            hyperlink: instruction.contains("\\h"),
            include_position: instruction.contains("\\r"),
        }
    }

    /// Parse SEQ options
    fn parse_seq_options(instruction: &str) -> SeqOptions {
        let parts: Vec<&str> = instruction.split_whitespace().collect();

        let identifier = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        };

        let format = Self::parse_number_format_switch(instruction);

        let reset_at_heading_level = if let Some(pos) = instruction.find("\\s") {
            let after = &instruction[pos + 2..].trim_start();
            after.split_whitespace().next()
                .and_then(|s| s.parse().ok())
        } else {
            None
        };

        let reset_to = if let Some(pos) = instruction.find("\\r") {
            let after = &instruction[pos + 2..].trim_start();
            after.split_whitespace().next()
                .and_then(|s| s.parse().ok())
        } else {
            None
        };

        SeqOptions {
            identifier,
            format,
            reset_at_heading_level,
            current_only: instruction.contains("\\c"),
            reset_to,
            repeat_previous: instruction.contains("\\*"),
        }
    }

    /// Parse HYPERLINK instruction
    fn parse_hyperlink_instruction(instruction: &str) -> (String, Option<String>) {
        let mut url = String::new();
        let mut in_quote = false;
        let mut quote_content = String::new();

        let after_hyperlink = instruction.trim_start_matches("HYPERLINK").trim();

        for ch in after_hyperlink.chars() {
            if ch == '"' {
                if in_quote {
                    url = quote_content.clone();
                    quote_content.clear();
                    in_quote = false;
                } else {
                    in_quote = true;
                }
            } else if in_quote {
                quote_content.push(ch);
            }
        }

        (url, None)
    }

    /// Parse IF instruction
    fn parse_if_instruction(instruction: &str) -> (String, String, String) {
        // IF field format: IF expression "true text" "false text"
        // This is a simplified parser
        let after_if = instruction.trim_start_matches("IF").trim();

        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;

        for ch in after_if.chars() {
            if ch == '"' {
                if in_quote {
                    parts.push(current.clone());
                    current.clear();
                    in_quote = false;
                } else {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                        current.clear();
                    }
                    in_quote = true;
                }
            } else if in_quote {
                current.push(ch);
            } else {
                current.push(ch);
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        let condition = parts.first().cloned().unwrap_or_default();
        let true_text = parts.get(1).cloned().unwrap_or_default();
        let false_text = parts.get(2).cloned().unwrap_or_default();

        (condition, true_text, false_text)
    }
}

// =============================================================================
// Field Writer
// =============================================================================

/// Writer for fields in DOCX export
pub struct FieldWriter;

impl FieldWriter {
    /// Write a field to XML
    pub fn write_field(xml: &mut String, field: &Field) {
        // Write field begin
        xml.push_str("<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>");

        // Write instruction
        xml.push_str("<w:r><w:instrText xml:space=\"preserve\"> ");
        xml.push_str(&Self::field_to_instruction_string(&field.instruction));
        xml.push_str(" </w:instrText></w:r>");

        // Write separator
        xml.push_str("<w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>");

        // Write result (cached value)
        if let Some(ref text) = field.cached_text {
            xml.push_str("<w:r><w:t>");
            xml.push_str(&escape_xml(text));
            xml.push_str("</w:t></w:r>");
        }

        // Write field end
        xml.push_str("<w:r><w:fldChar w:fldCharType=\"end\"/></w:r>");
    }

    /// Convert field instruction to DOCX instruction string
    fn field_to_instruction_string(instruction: &FieldInstruction) -> String {
        match instruction {
            FieldInstruction::Page { format } => {
                let mut s = "PAGE".to_string();
                if *format != NumberFormat::Arabic {
                    s.push_str(&format!(" \\* {}", Self::number_format_to_string(format)));
                }
                s
            }
            FieldInstruction::NumPages { format } => {
                let mut s = "NUMPAGES".to_string();
                if *format != NumberFormat::Arabic {
                    s.push_str(&format!(" \\* {}", Self::number_format_to_string(format)));
                }
                s
            }
            FieldInstruction::Date { format } => {
                format!("DATE \\@ \"{}\"", format)
            }
            FieldInstruction::Time { format } => {
                format!("TIME \\@ \"{}\"", format)
            }
            FieldInstruction::Toc { switches } => {
                let mut s = "TOC".to_string();
                s.push_str(&format!(
                    " \\o \"{}-{}\"",
                    switches.heading_levels.start,
                    switches.heading_levels.end - 1
                ));
                if switches.hyperlinks {
                    s.push_str(" \\h");
                }
                if !switches.include_page_numbers {
                    s.push_str(" \\n");
                }
                s
            }
            FieldInstruction::Ref { options } => {
                let mut s = format!("REF {}", options.bookmark);
                if options.hyperlink {
                    s.push_str(" \\h");
                }
                match options.display {
                    RefDisplayType::PageNumber => s.push_str(" \\p"),
                    RefDisplayType::ParagraphNumber => s.push_str(" \\n"),
                    RefDisplayType::ParagraphNumberFullContext => s.push_str(" \\w"),
                    RefDisplayType::RelativePosition => s.push_str(" \\r"),
                    RefDisplayType::Content => {}
                }
                s
            }
            FieldInstruction::Seq { options } => {
                let mut s = format!("SEQ {}", options.identifier);
                if options.format != NumberFormat::Arabic {
                    s.push_str(&format!(" \\* {}", Self::number_format_to_string(&options.format)));
                }
                if let Some(level) = options.reset_at_heading_level {
                    s.push_str(&format!(" \\s {}", level));
                }
                if let Some(val) = options.reset_to {
                    s.push_str(&format!(" \\r {}", val));
                }
                if options.current_only {
                    s.push_str(" \\c");
                }
                s
            }
            FieldInstruction::Author => "AUTHOR".to_string(),
            FieldInstruction::Title => "TITLE".to_string(),
            FieldInstruction::Subject => "SUBJECT".to_string(),
            FieldInstruction::FileName { include_path } => {
                if *include_path {
                    "FILENAME \\p".to_string()
                } else {
                    "FILENAME".to_string()
                }
            }
            FieldInstruction::Section => "SECTION".to_string(),
            FieldInstruction::SectionPages => "SECTIONPAGES".to_string(),
            FieldInstruction::Hyperlink { url, display_text: _ } => {
                format!("HYPERLINK \"{}\"", url)
            }
            FieldInstruction::IncludeText { file_path } => {
                format!("INCLUDETEXT \"{}\"", file_path)
            }
            FieldInstruction::If { condition, true_text, false_text } => {
                format!("IF {} \"{}\" \"{}\"", condition, true_text, false_text)
            }
            FieldInstruction::PrintDate { format } => {
                format!("PRINTDATE \\@ \"{}\"", format)
            }
            FieldInstruction::SaveDate { format } => {
                format!("SAVEDATE \\@ \"{}\"", format)
            }
            FieldInstruction::CreateDate { format } => {
                format!("CREATEDATE \\@ \"{}\"", format)
            }
            FieldInstruction::EditTime => "EDITTIME".to_string(),
            FieldInstruction::NumWords => "NUMWORDS".to_string(),
            FieldInstruction::NumChars => "NUMCHARS".to_string(),
            FieldInstruction::Custom { code } => code.clone(),
        }
    }

    /// Convert number format to DOCX string
    fn number_format_to_string(format: &NumberFormat) -> &'static str {
        match format {
            NumberFormat::Arabic => "Arabic",
            NumberFormat::UppercaseRoman => "ROMAN",
            NumberFormat::LowercaseRoman => "roman",
            NumberFormat::UppercaseLetter => "ALPHABETIC",
            NumberFormat::LowercaseLetter => "alphabetic",
            NumberFormat::Ordinal => "Ordinal",
            NumberFormat::CardinalText => "CardText",
            NumberFormat::OrdinalText => "OrdText",
        }
    }
}

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed field from DOCX
#[derive(Debug, Clone)]
pub struct ParsedField {
    /// Raw instruction text
    pub instruction_text: String,
    /// Result/display text
    pub result_text: String,
    /// Nesting level (0 = top-level)
    pub nesting_level: u32,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Escape XML text content
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_parser_new() {
        let parser = FieldParser::new();
        assert!(!parser.in_field());
    }

    #[test]
    fn test_parse_page_instruction() {
        let field = FieldParser::parse_instruction("PAGE").unwrap();
        assert!(matches!(field.instruction, FieldInstruction::Page { .. }));
    }

    #[test]
    fn test_parse_page_with_format() {
        let field = FieldParser::parse_instruction("PAGE \\* ROMAN").unwrap();
        if let FieldInstruction::Page { format } = field.instruction {
            assert_eq!(format, NumberFormat::UppercaseRoman);
        } else {
            panic!("Expected PAGE instruction");
        }
    }

    #[test]
    fn test_parse_date_instruction() {
        let field = FieldParser::parse_instruction("DATE \\@ \"M/d/yyyy\"").unwrap();
        if let FieldInstruction::Date { format } = field.instruction {
            assert_eq!(format, "M/d/yyyy");
        } else {
            panic!("Expected DATE instruction");
        }
    }

    #[test]
    fn test_parse_toc_instruction() {
        let field = FieldParser::parse_instruction("TOC \\o \"1-3\" \\h").unwrap();
        if let FieldInstruction::Toc { switches } = field.instruction {
            assert_eq!(switches.heading_levels, 1..4);
            assert!(switches.hyperlinks);
        } else {
            panic!("Expected TOC instruction");
        }
    }

    #[test]
    fn test_parse_ref_instruction() {
        let field = FieldParser::parse_instruction("REF bookmark1 \\h \\p").unwrap();
        if let FieldInstruction::Ref { options } = field.instruction {
            assert_eq!(options.bookmark, "bookmark1");
            assert!(options.hyperlink);
            assert_eq!(options.display, RefDisplayType::PageNumber);
        } else {
            panic!("Expected REF instruction");
        }
    }

    #[test]
    fn test_parse_seq_instruction() {
        let field = FieldParser::parse_instruction("SEQ Figure \\s 1").unwrap();
        if let FieldInstruction::Seq { options } = field.instruction {
            assert_eq!(options.identifier, "Figure");
            assert_eq!(options.reset_at_heading_level, Some(1));
        } else {
            panic!("Expected SEQ instruction");
        }
    }

    #[test]
    fn test_parse_custom_instruction() {
        let field = FieldParser::parse_instruction("UNKNOWNFIELD arg1").unwrap();
        if let FieldInstruction::Custom { code } = field.instruction {
            assert_eq!(code, "UNKNOWNFIELD arg1");
        } else {
            panic!("Expected Custom instruction");
        }
    }

    #[test]
    fn test_field_writer_page() {
        let mut xml = String::new();
        let field = Field::page();
        FieldWriter::write_field(&mut xml, &field);

        assert!(xml.contains("fldCharType=\"begin\""));
        assert!(xml.contains("PAGE"));
        assert!(xml.contains("fldCharType=\"separate\""));
        assert!(xml.contains("fldCharType=\"end\""));
    }
}
