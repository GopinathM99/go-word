//! Advanced Tables Import/Export for DOCX
//!
//! Handles complete table support including:
//! - Cell merging (w:gridSpan for horizontal, w:vMerge for vertical)
//! - Row breaking across pages (w:cantSplit)
//! - Header row repeat (w:tblHeader)
//! - Nested tables
//! - Table styles with conditional formatting

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use doc_model::Alignment;
use quick_xml::events::Event;
use std::collections::HashMap;

// Local type definitions for table width
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableWidthType {
    #[default]
    Dxa,
    Pct,
    Auto,
    Nil,
}

// =============================================================================
// Table Parser
// =============================================================================

/// Parser for tables in DOCX with advanced features
pub struct TableParser {
    /// Stack for tracking nested tables
    table_stack: Vec<TableParseState>,
}

/// State for parsing a single table
#[derive(Debug, Clone)]
struct TableParseState {
    /// Table properties
    properties: ParsedTableProperties,
    /// Grid column widths
    grid_cols: Vec<f32>,
    /// Rows being parsed
    rows: Vec<ParsedTableRow>,
    /// Current row being parsed
    current_row: Option<ParsedTableRow>,
    /// Current cell being parsed
    current_cell: Option<ParsedTableCell>,
    /// Current column index in row
    current_col_index: usize,
}

impl TableParser {
    /// Create a new table parser
    pub fn new() -> Self {
        Self {
            table_stack: Vec::new(),
        }
    }

    /// Start parsing a new table
    pub fn start_table(&mut self) {
        self.table_stack.push(TableParseState {
            properties: ParsedTableProperties::default(),
            grid_cols: Vec::new(),
            rows: Vec::new(),
            current_row: None,
            current_cell: None,
            current_col_index: 0,
        });
    }

    /// End parsing the current table
    pub fn end_table(&mut self) -> Option<ParsedTable> {
        if let Some(state) = self.table_stack.pop() {
            Some(ParsedTable {
                properties: state.properties,
                grid_cols: state.grid_cols,
                rows: state.rows,
            })
        } else {
            None
        }
    }

    /// Check if we're currently inside a table
    pub fn in_table(&self) -> bool {
        !self.table_stack.is_empty()
    }

    /// Get nesting level (0 = not in table)
    pub fn nesting_level(&self) -> usize {
        self.table_stack.len()
    }

    /// Parse table properties (w:tblPr)
    pub fn parse_table_properties(&mut self, content: &str) -> DocxResult<()> {
        if let Some(state) = self.table_stack.last_mut() {
            let mut reader = XmlParser::from_string(content);
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                        let name = e.name();
                        let name_ref = name.as_ref();

                        if XmlParser::matches_element(name_ref, "tblStyle") {
                            state.properties.style_id = XmlParser::get_w_attribute(e, "val");
                        } else if XmlParser::matches_element(name_ref, "tblW") {
                            state.properties.width = XmlParser::get_w_attribute(e, "w")
                                .and_then(|s| s.parse().ok());
                            state.properties.width_type = XmlParser::get_w_attribute(e, "type")
                                .map(|s| parse_width_type(&s));
                        } else if XmlParser::matches_element(name_ref, "jc") {
                            state.properties.alignment = XmlParser::get_w_attribute(e, "val")
                                .map(|s| parse_alignment(&s));
                        } else if XmlParser::matches_element(name_ref, "tblInd") {
                            state.properties.indent = XmlParser::get_w_attribute(e, "w")
                                .and_then(|s| s.parse().ok())
                                .map(|twips: i32| twips as f32 / 20.0);
                        } else if XmlParser::matches_element(name_ref, "tblLook") {
                            state.properties.look_first_row = XmlParser::get_w_attribute(e, "firstRow")
                                .map(|s| s == "1")
                                .unwrap_or(false);
                            state.properties.look_last_row = XmlParser::get_w_attribute(e, "lastRow")
                                .map(|s| s == "1")
                                .unwrap_or(false);
                            state.properties.look_first_col = XmlParser::get_w_attribute(e, "firstColumn")
                                .map(|s| s == "1")
                                .unwrap_or(false);
                            state.properties.look_last_col = XmlParser::get_w_attribute(e, "lastColumn")
                                .map(|s| s == "1")
                                .unwrap_or(false);
                            state.properties.look_no_h_band = XmlParser::get_w_attribute(e, "noHBand")
                                .map(|s| s == "1")
                                .unwrap_or(false);
                            state.properties.look_no_v_band = XmlParser::get_w_attribute(e, "noVBand")
                                .map(|s| s == "1")
                                .unwrap_or(false);
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(DocxError::from(e)),
                    _ => {}
                }
                buf.clear();
            }
        }

        Ok(())
    }

    /// Parse table grid (w:tblGrid)
    pub fn parse_table_grid(&mut self, content: &str) -> DocxResult<()> {
        if let Some(state) = self.table_stack.last_mut() {
            let mut reader = XmlParser::from_string(content);
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                        let name = e.name();
                        let name_ref = name.as_ref();

                        if XmlParser::matches_element(name_ref, "gridCol") {
                            let width = XmlParser::get_w_attribute(e, "w")
                                .and_then(|s| s.parse().ok())
                                .map(|twips: i32| twips as f32 / 20.0)
                                .unwrap_or(72.0);
                            state.grid_cols.push(width);
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(DocxError::from(e)),
                    _ => {}
                }
                buf.clear();
            }
        }

        Ok(())
    }

    /// Start a new row
    pub fn start_row(&mut self) {
        if let Some(state) = self.table_stack.last_mut() {
            state.current_row = Some(ParsedTableRow::default());
            state.current_col_index = 0;
        }
    }

    /// Parse row properties (w:trPr)
    pub fn parse_row_properties(&mut self, content: &str) -> DocxResult<()> {
        if let Some(state) = self.table_stack.last_mut() {
            if let Some(ref mut row) = state.current_row {
                let mut reader = XmlParser::from_string(content);
                let mut buf = Vec::new();

                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                            let name = e.name();
                            let name_ref = name.as_ref();

                            if XmlParser::matches_element(name_ref, "tblHeader") {
                                row.properties.is_header = XmlParser::get_w_attribute(e, "val")
                                    .map(|s| s != "0")
                                    .unwrap_or(true);
                            } else if XmlParser::matches_element(name_ref, "cantSplit") {
                                row.properties.cant_split = XmlParser::get_w_attribute(e, "val")
                                    .map(|s| s != "0")
                                    .unwrap_or(true);
                            } else if XmlParser::matches_element(name_ref, "trHeight") {
                                row.properties.height = XmlParser::get_w_attribute(e, "val")
                                    .and_then(|s| s.parse().ok())
                                    .map(|twips: i32| twips as f32 / 20.0);
                                row.properties.height_rule = XmlParser::get_w_attribute(e, "hRule")
                                    .map(|s| parse_height_rule(&s));
                            } else if XmlParser::matches_element(name_ref, "jc") {
                                row.properties.alignment = XmlParser::get_w_attribute(e, "val")
                                    .map(|s| parse_alignment(&s));
                            }
                        }
                        Ok(Event::Eof) => break,
                        Err(e) => return Err(DocxError::from(e)),
                        _ => {}
                    }
                    buf.clear();
                }
            }
        }

        Ok(())
    }

    /// End the current row
    pub fn end_row(&mut self) {
        if let Some(state) = self.table_stack.last_mut() {
            if let Some(row) = state.current_row.take() {
                state.rows.push(row);
            }
        }
    }

    /// Start a new cell
    pub fn start_cell(&mut self) {
        if let Some(state) = self.table_stack.last_mut() {
            state.current_cell = Some(ParsedTableCell {
                properties: ParsedCellProperties::default(),
                content: String::new(),
                nested_table: None,
            });
        }
    }

    /// Parse cell properties (w:tcPr)
    pub fn parse_cell_properties(&mut self, content: &str) -> DocxResult<()> {
        if let Some(state) = self.table_stack.last_mut() {
            if let Some(ref mut cell) = state.current_cell {
                let mut reader = XmlParser::from_string(content);
                let mut buf = Vec::new();

                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                            let name = e.name();
                            let name_ref = name.as_ref();

                            if XmlParser::matches_element(name_ref, "tcW") {
                                cell.properties.width = XmlParser::get_w_attribute(e, "w")
                                    .and_then(|s| s.parse().ok())
                                    .map(|twips: i32| twips as f32 / 20.0);
                                cell.properties.width_type = XmlParser::get_w_attribute(e, "type")
                                    .map(|s| parse_width_type(&s));
                            } else if XmlParser::matches_element(name_ref, "gridSpan") {
                                cell.properties.grid_span = XmlParser::get_w_attribute(e, "val")
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(1);
                            } else if XmlParser::matches_element(name_ref, "vMerge") {
                                let merge_val = XmlParser::get_w_attribute(e, "val");
                                if merge_val.as_deref() == Some("restart") {
                                    cell.properties.v_merge = Some(VerticalMerge::Restart);
                                } else {
                                    cell.properties.v_merge = Some(VerticalMerge::Continue);
                                }
                            } else if XmlParser::matches_element(name_ref, "vAlign") {
                                cell.properties.vertical_align = XmlParser::get_w_attribute(e, "val")
                                    .map(|s| parse_vertical_align(&s));
                            } else if XmlParser::matches_element(name_ref, "shd") {
                                cell.properties.shading_color = XmlParser::get_w_attribute(e, "fill");
                            } else if XmlParser::matches_element(name_ref, "noWrap") {
                                cell.properties.no_wrap = true;
                            } else if XmlParser::matches_element(name_ref, "textDirection") {
                                cell.properties.text_direction = XmlParser::get_w_attribute(e, "val");
                            }
                        }
                        Ok(Event::Eof) => break,
                        Err(e) => return Err(DocxError::from(e)),
                        _ => {}
                    }
                    buf.clear();
                }
            }
        }

        Ok(())
    }

    /// End the current cell
    pub fn end_cell(&mut self) {
        if let Some(state) = self.table_stack.last_mut() {
            if let Some(cell) = state.current_cell.take() {
                let span = cell.properties.grid_span.max(1);

                if let Some(ref mut row) = state.current_row {
                    row.cells.push(cell);
                }

                state.current_col_index += span as usize;
            }
        }
    }

    /// Add content to the current cell
    pub fn add_cell_content(&mut self, content: &str) {
        if let Some(state) = self.table_stack.last_mut() {
            if let Some(ref mut cell) = state.current_cell {
                cell.content.push_str(content);
            }
        }
    }
}

// =============================================================================
// Table Writer
// =============================================================================

/// Writer for tables in DOCX export
pub struct TableWriter;

impl TableWriter {
    /// Write a simple table to XML from parsed data
    pub fn write_parsed_table(xml: &mut String, table: &ParsedTable) {
        xml.push_str("<w:tbl>");

        // Write table properties
        xml.push_str("<w:tblPr>");
        if let Some(ref style_id) = table.properties.style_id {
            xml.push_str(&format!(r#"<w:tblStyle w:val="{}"/>"#, style_id));
        }
        xml.push_str("<w:tblW w:w=\"0\" w:type=\"auto\"/>");
        xml.push_str("</w:tblPr>");

        // Write table grid
        xml.push_str("<w:tblGrid>");
        for col_width in &table.grid_cols {
            let width_twips = (*col_width * 20.0) as i32;
            xml.push_str(&format!(r#"<w:gridCol w:w="{}"/>"#, width_twips));
        }
        xml.push_str("</w:tblGrid>");

        // Write rows
        for row in &table.rows {
            Self::write_parsed_row(xml, row);
        }

        xml.push_str("</w:tbl>");
    }

    fn write_parsed_row(xml: &mut String, row: &ParsedTableRow) {
        xml.push_str("<w:tr>");

        // Row properties
        xml.push_str("<w:trPr>");
        if row.properties.is_header {
            xml.push_str("<w:tblHeader/>");
        }
        if row.properties.cant_split {
            xml.push_str("<w:cantSplit/>");
        }
        xml.push_str("</w:trPr>");

        // Cells
        for cell in &row.cells {
            Self::write_parsed_cell(xml, cell);
        }

        xml.push_str("</w:tr>");
    }

    fn write_parsed_cell(xml: &mut String, cell: &ParsedTableCell) {
        xml.push_str("<w:tc>");

        // Cell properties
        xml.push_str("<w:tcPr>");
        if cell.properties.grid_span > 1 {
            xml.push_str(&format!(r#"<w:gridSpan w:val="{}"/>"#, cell.properties.grid_span));
        }
        if let Some(ref v_merge) = cell.properties.v_merge {
            match v_merge {
                VerticalMerge::Restart => xml.push_str(r#"<w:vMerge w:val="restart"/>"#),
                VerticalMerge::Continue => xml.push_str("<w:vMerge/>"),
            }
        }
        xml.push_str("</w:tcPr>");

        // Cell content
        xml.push_str("<w:p><w:r><w:t>");
        xml.push_str(&escape_xml(&cell.content));
        xml.push_str("</w:t></w:r></w:p>");

        xml.push_str("</w:tc>");
    }
}

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed table
#[derive(Debug, Clone)]
pub struct ParsedTable {
    pub properties: ParsedTableProperties,
    pub grid_cols: Vec<f32>,
    pub rows: Vec<ParsedTableRow>,
}

/// Parsed table properties
#[derive(Debug, Clone, Default)]
pub struct ParsedTableProperties {
    pub style_id: Option<String>,
    pub width: Option<i32>,
    pub width_type: Option<TableWidthType>,
    pub alignment: Option<Alignment>,
    pub indent: Option<f32>,
    pub look_first_row: bool,
    pub look_last_row: bool,
    pub look_first_col: bool,
    pub look_last_col: bool,
    pub look_no_h_band: bool,
    pub look_no_v_band: bool,
}

/// Parsed table row
#[derive(Debug, Clone, Default)]
pub struct ParsedTableRow {
    pub properties: ParsedRowProperties,
    pub cells: Vec<ParsedTableCell>,
}

/// Parsed row properties
#[derive(Debug, Clone, Default)]
pub struct ParsedRowProperties {
    pub is_header: bool,
    pub cant_split: bool,
    pub height: Option<f32>,
    pub height_rule: Option<HeightRule>,
    pub alignment: Option<Alignment>,
}

/// Height rule for table rows
#[derive(Debug, Clone, Copy)]
pub enum HeightRule {
    Auto,
    AtLeast,
    Exact,
}

/// Parsed table cell
#[derive(Debug, Clone)]
pub struct ParsedTableCell {
    pub properties: ParsedCellProperties,
    pub content: String,
    pub nested_table: Option<Box<ParsedTable>>,
}

/// Parsed cell properties
#[derive(Debug, Clone)]
pub struct ParsedCellProperties {
    pub width: Option<f32>,
    pub width_type: Option<TableWidthType>,
    pub grid_span: u32,
    pub v_merge: Option<VerticalMerge>,
    pub vertical_align: Option<VerticalCellAlign>,
    pub shading_color: Option<String>,
    pub no_wrap: bool,
    pub text_direction: Option<String>,
}

impl Default for ParsedCellProperties {
    fn default() -> Self {
        Self {
            width: None,
            width_type: None,
            grid_span: 1,
            v_merge: None,
            vertical_align: None,
            shading_color: None,
            no_wrap: false,
            text_direction: None,
        }
    }
}

/// Vertical merge state
#[derive(Debug, Clone)]
pub enum VerticalMerge {
    Restart,
    Continue,
}

/// Vertical cell alignment
#[derive(Debug, Clone, Copy)]
pub enum VerticalCellAlign {
    Top,
    Center,
    Bottom,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn parse_width_type(s: &str) -> TableWidthType {
    match s {
        "dxa" => TableWidthType::Dxa,
        "pct" => TableWidthType::Pct,
        "auto" => TableWidthType::Auto,
        "nil" => TableWidthType::Nil,
        _ => TableWidthType::Dxa,
    }
}

fn parse_alignment(s: &str) -> Alignment {
    match s {
        "left" | "start" => Alignment::Left,
        "center" => Alignment::Center,
        "right" | "end" => Alignment::Right,
        "both" | "justify" => Alignment::Justify,
        _ => Alignment::Left,
    }
}

fn parse_height_rule(s: &str) -> HeightRule {
    match s {
        "auto" => HeightRule::Auto,
        "atLeast" => HeightRule::AtLeast,
        "exact" => HeightRule::Exact,
        _ => HeightRule::Auto,
    }
}

fn parse_vertical_align(s: &str) -> VerticalCellAlign {
    match s {
        "top" => VerticalCellAlign::Top,
        "center" => VerticalCellAlign::Center,
        "bottom" => VerticalCellAlign::Bottom,
        _ => VerticalCellAlign::Top,
    }
}

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
    fn test_table_parser_new() {
        let parser = TableParser::new();
        assert!(!parser.in_table());
    }

    #[test]
    fn test_table_parser_nesting() {
        let mut parser = TableParser::new();

        assert_eq!(parser.nesting_level(), 0);

        parser.start_table();
        assert_eq!(parser.nesting_level(), 1);
        assert!(parser.in_table());

        parser.start_table();
        assert_eq!(parser.nesting_level(), 2);

        parser.end_table();
        assert_eq!(parser.nesting_level(), 1);

        parser.end_table();
        assert_eq!(parser.nesting_level(), 0);
        assert!(!parser.in_table());
    }

    #[test]
    fn test_parse_width_type() {
        assert!(matches!(parse_width_type("dxa"), TableWidthType::Dxa));
        assert!(matches!(parse_width_type("pct"), TableWidthType::Pct));
        assert!(matches!(parse_width_type("auto"), TableWidthType::Auto));
    }

    #[test]
    fn test_parse_alignment() {
        assert!(matches!(parse_alignment("center"), Alignment::Center));
        assert!(matches!(parse_alignment("both"), Alignment::Justify));
    }

    #[test]
    fn test_parse_vertical_align() {
        assert!(matches!(parse_vertical_align("center"), VerticalCellAlign::Center));
        assert!(matches!(parse_vertical_align("bottom"), VerticalCellAlign::Bottom));
    }
}
