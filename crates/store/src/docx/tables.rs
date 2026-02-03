//! Table parsing for DOCX files
//!
//! Handles w:tbl, w:tr, w:tc elements and their properties.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use doc_model::{
    CellBorders, CellPadding, CellProperties, CellVerticalAlign, DocumentTree, GridColumn,
    HeightRule, Node, Paragraph, RowProperties, Run, Table, TableAlignment, TableBorder,
    TableBorderStyle, TableBorders, TableCell, TableGrid, TableProperties, TableRow, TableWidth,
    WidthType,
};
use quick_xml::events::Event;

/// Parser for table elements in document.xml
pub struct TableParser;

impl TableParser {
    /// Create a new table parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a w:tbl element and return a Table with its content
    pub fn parse_table(&self, content: &str) -> DocxResult<ParsedTable> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut table = ParsedTable::new();
        let mut current_row: Option<ParsedRow> = None;
        let mut current_cell: Option<ParsedCell> = None;
        let mut current_para: Option<ParsedParagraph> = None;
        let mut current_run: Option<ParsedRun> = None;

        let mut in_tbl_pr = false;
        let mut in_tbl_grid = false;
        let mut in_tr_pr = false;
        let mut in_tc_pr = false;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "tblPr") {
                        in_tbl_pr = true;
                    } else if XmlParser::matches_element(name_ref, "tblGrid") {
                        in_tbl_grid = true;
                    } else if XmlParser::matches_element(name_ref, "tr") {
                        current_row = Some(ParsedRow::new());
                    } else if XmlParser::matches_element(name_ref, "trPr") {
                        in_tr_pr = true;
                    } else if XmlParser::matches_element(name_ref, "tc") {
                        current_cell = Some(ParsedCell::new());
                    } else if XmlParser::matches_element(name_ref, "tcPr") {
                        in_tc_pr = true;
                    } else if current_cell.is_some() && XmlParser::matches_element(name_ref, "p") {
                        current_para = Some(ParsedParagraph::new());
                    } else if current_para.is_some() && XmlParser::matches_element(name_ref, "r") {
                        current_run = Some(ParsedRun::new());
                    } else if current_run.is_some() && XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    } else if in_tbl_pr {
                        self.parse_table_property(e, &mut table)?;
                    } else if in_tr_pr && current_row.is_some() {
                        self.parse_row_property(e, current_row.as_mut().unwrap())?;
                    } else if in_tc_pr && current_cell.is_some() {
                        self.parse_cell_property(e, current_cell.as_mut().unwrap())?;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if in_tbl_grid && XmlParser::matches_element(name_ref, "gridCol") {
                        if let Some(width) = XmlParser::get_w_attribute(e, "w") {
                            let w = XmlParser::parse_twips(&width).unwrap_or(0.0);
                            table.grid.push(w);
                        }
                    } else if in_tbl_pr {
                        self.parse_table_property(e, &mut table)?;
                    } else if in_tr_pr && current_row.is_some() {
                        self.parse_row_property(e, current_row.as_mut().unwrap())?;
                    } else if in_tc_pr && current_cell.is_some() {
                        self.parse_cell_property(e, current_cell.as_mut().unwrap())?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "tblPr") {
                        in_tbl_pr = false;
                    } else if XmlParser::matches_element(name_ref, "tblGrid") {
                        in_tbl_grid = false;
                    } else if XmlParser::matches_element(name_ref, "tr") {
                        if let Some(row) = current_row.take() {
                            table.rows.push(row);
                        }
                    } else if XmlParser::matches_element(name_ref, "trPr") {
                        in_tr_pr = false;
                    } else if XmlParser::matches_element(name_ref, "tc") {
                        if let Some(cell) = current_cell.take() {
                            if let Some(ref mut row) = current_row {
                                row.cells.push(cell);
                            }
                        }
                    } else if XmlParser::matches_element(name_ref, "tcPr") {
                        in_tc_pr = false;
                    } else if XmlParser::matches_element(name_ref, "p") {
                        if let Some(para) = current_para.take() {
                            if let Some(ref mut cell) = current_cell {
                                cell.paragraphs.push(para);
                            }
                        }
                    } else if XmlParser::matches_element(name_ref, "r") {
                        if let Some(run) = current_run.take() {
                            if let Some(ref mut para) = current_para {
                                para.runs.push(run);
                            }
                        }
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        if let Some(ref mut run) = current_run {
                            let text = e.unescape().map_err(|e| DocxError::XmlParse(e.to_string()))?;
                            run.text.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(table)
    }

    /// Parse table properties
    fn parse_table_property(&self, e: &quick_xml::events::BytesStart, table: &mut ParsedTable) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "tblW") {
            if let Some(w) = XmlParser::get_w_attribute(e, "w") {
                let width_type = XmlParser::get_w_attribute(e, "type")
                    .unwrap_or_else(|| "dxa".to_string());
                table.width = Some(parse_table_width(&w, &width_type));
            }
        } else if XmlParser::matches_element(name_ref, "jc") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                table.alignment = Some(parse_table_alignment(&val));
            }
        } else if XmlParser::matches_element(name_ref, "tblInd") {
            if let Some(w) = XmlParser::get_w_attribute(e, "w") {
                table.indent = XmlParser::parse_twips(&w);
            }
        }

        Ok(())
    }

    /// Parse row properties
    fn parse_row_property(&self, e: &quick_xml::events::BytesStart, row: &mut ParsedRow) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "trHeight") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                row.height = XmlParser::parse_twips(&val);
            }
            if let Some(rule) = XmlParser::get_w_attribute(e, "hRule") {
                row.height_rule = parse_height_rule(&rule);
            }
        } else if XmlParser::matches_element(name_ref, "tblHeader") {
            row.is_header = true;
        } else if XmlParser::matches_element(name_ref, "cantSplit") {
            row.can_split = false;
        }

        Ok(())
    }

    /// Parse cell properties
    fn parse_cell_property(&self, e: &quick_xml::events::BytesStart, cell: &mut ParsedCell) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "tcW") {
            if let Some(w) = XmlParser::get_w_attribute(e, "w") {
                let width_type = XmlParser::get_w_attribute(e, "type")
                    .unwrap_or_else(|| "dxa".to_string());
                cell.width = Some(parse_table_width(&w, &width_type));
            }
        } else if XmlParser::matches_element(name_ref, "gridSpan") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                cell.grid_span = val.parse().unwrap_or(1);
            }
        } else if XmlParser::matches_element(name_ref, "vMerge") {
            let val = XmlParser::get_w_attribute(e, "val");
            cell.v_merge = if val.as_deref() == Some("restart") {
                VMerge::Restart
            } else {
                VMerge::Continue
            };
        } else if XmlParser::matches_element(name_ref, "vAlign") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                cell.vertical_align = Some(parse_vertical_align(&val));
            }
        } else if XmlParser::matches_element(name_ref, "shd") {
            if let Some(fill) = XmlParser::get_w_attribute(e, "fill") {
                if fill != "auto" {
                    cell.shading = Some(format!("#{}", fill));
                }
            }
        }

        Ok(())
    }

    /// Commit a parsed table to the document tree
    pub fn commit_table(&self, parsed: ParsedTable, tree: &mut DocumentTree) -> DocxResult<()> {
        // Create table grid
        let grid = if parsed.grid.is_empty() {
            // Infer grid from first row
            if let Some(first_row) = parsed.rows.first() {
                TableGrid::new(first_row.cells.len())
            } else {
                TableGrid::new(0)
            }
        } else {
            TableGrid {
                columns: parsed.grid.iter().map(|&w| GridColumn::fixed(w)).collect(),
            }
        };

        // Create table properties
        let mut props = TableProperties::new();
        if let Some(width) = parsed.width {
            props.width = Some(width);
        }
        if let Some(alignment) = parsed.alignment {
            props.alignment = Some(alignment);
        }
        if let Some(indent) = parsed.indent {
            props.indent_left = Some(indent);
        }

        // Create the table
        let table = Table::with_grid_and_properties(grid, props);
        let table_id = tree.insert_table(table, None)?;

        // Add rows
        for parsed_row in parsed.rows {
            let row_props = RowProperties {
                height: parsed_row.height,
                height_rule: parsed_row.height_rule,
                is_header: parsed_row.is_header,
                can_split: parsed_row.can_split,
                cant_split: !parsed_row.can_split,
                keep_with_next: false,
            };

            let row = TableRow::with_properties(row_props);
            let row_id = tree.insert_table_row(row, table_id, None)?;

            // Add cells
            for parsed_cell in parsed_row.cells {
                let cell_props = CellProperties {
                    width: parsed_cell.width,
                    vertical_align: parsed_cell.vertical_align,
                    shading: parsed_cell.shading,
                    ..Default::default()
                };

                let mut cell = TableCell::with_properties(cell_props);
                cell.grid_span = parsed_cell.grid_span;
                cell.v_merge_continue = matches!(parsed_cell.v_merge, VMerge::Continue);

                let cell_id = tree.insert_table_cell(cell, row_id, None)?;

                // Ensure cell has at least one paragraph
                let has_paragraphs = !parsed_cell.paragraphs.is_empty();

                // Add paragraphs to cell
                for parsed_para in parsed_cell.paragraphs {
                    let para = Paragraph::new();
                    let para_id = tree.insert_paragraph_into_cell(para, cell_id, None)?;

                    // Add runs to paragraph
                    for parsed_run in parsed_para.runs {
                        if !parsed_run.text.is_empty() {
                            let run = Run::new(&parsed_run.text);
                            tree.insert_run(run, para_id, None)?;
                        }
                    }
                }

                // Ensure cell has at least one paragraph
                if !has_paragraphs {
                    let para = Paragraph::new();
                    tree.insert_paragraph_into_cell(para, cell_id, None)?;
                }
            }
        }

        Ok(())
    }
}

/// Parsed table structure
#[derive(Debug, Default)]
pub struct ParsedTable {
    pub width: Option<TableWidth>,
    pub alignment: Option<TableAlignment>,
    pub indent: Option<f32>,
    pub grid: Vec<f32>,
    pub rows: Vec<ParsedRow>,
}

impl ParsedTable {
    fn new() -> Self {
        Self::default()
    }
}

/// Parsed row structure
#[derive(Debug, Default)]
pub struct ParsedRow {
    pub height: Option<f32>,
    pub height_rule: HeightRule,
    pub is_header: bool,
    pub can_split: bool,
    pub cells: Vec<ParsedCell>,
}

impl ParsedRow {
    fn new() -> Self {
        Self {
            can_split: true,
            ..Default::default()
        }
    }
}

/// Vertical merge state
#[derive(Debug, Default, Clone, Copy)]
pub enum VMerge {
    #[default]
    None,
    Restart,
    Continue,
}

/// Parsed cell structure
#[derive(Debug, Default)]
pub struct ParsedCell {
    pub width: Option<TableWidth>,
    pub grid_span: u32,
    pub v_merge: VMerge,
    pub vertical_align: Option<CellVerticalAlign>,
    pub shading: Option<String>,
    pub paragraphs: Vec<ParsedParagraph>,
}

impl ParsedCell {
    fn new() -> Self {
        Self {
            grid_span: 1,
            ..Default::default()
        }
    }
}

/// Parsed paragraph (simplified for table content)
#[derive(Debug, Default)]
pub struct ParsedParagraph {
    pub runs: Vec<ParsedRun>,
}

impl ParsedParagraph {
    fn new() -> Self {
        Self::default()
    }
}

/// Parsed run (simplified)
#[derive(Debug, Default)]
pub struct ParsedRun {
    pub text: String,
}

impl ParsedRun {
    fn new() -> Self {
        Self::default()
    }
}

/// Parse table width from value and type
fn parse_table_width(value: &str, width_type: &str) -> TableWidth {
    let val: f32 = value.parse().unwrap_or(0.0);

    match width_type {
        "pct" => TableWidth::percent(val / 50.0), // DOCX uses 50ths of a percent
        "auto" => TableWidth::auto(),
        _ => TableWidth::fixed(val / 20.0), // dxa = twips
    }
}

/// Parse table alignment
fn parse_table_alignment(value: &str) -> TableAlignment {
    match value {
        "center" => TableAlignment::Center,
        "right" => TableAlignment::Right,
        _ => TableAlignment::Left,
    }
}

/// Parse height rule
fn parse_height_rule(value: &str) -> HeightRule {
    match value {
        "exact" => HeightRule::Exact,
        "atLeast" => HeightRule::AtLeast,
        _ => HeightRule::Auto,
    }
}

/// Parse vertical alignment
fn parse_vertical_align(value: &str) -> CellVerticalAlign {
    match value {
        "center" => CellVerticalAlign::Center,
        "bottom" => CellVerticalAlign::Bottom,
        _ => CellVerticalAlign::Top,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table_width() {
        let fixed = parse_table_width("1440", "dxa");
        assert_eq!(fixed.width_type, WidthType::Fixed);
        assert!((fixed.value - 72.0).abs() < 0.1); // 1440 twips = 72 points

        let pct = parse_table_width("5000", "pct");
        assert_eq!(pct.width_type, WidthType::Percent);
        assert!((pct.value - 100.0).abs() < 0.1); // 5000/50 = 100%

        let auto = parse_table_width("0", "auto");
        assert_eq!(auto.width_type, WidthType::Auto);
    }

    #[test]
    fn test_parse_table_alignment() {
        assert_eq!(parse_table_alignment("left"), TableAlignment::Left);
        assert_eq!(parse_table_alignment("center"), TableAlignment::Center);
        assert_eq!(parse_table_alignment("right"), TableAlignment::Right);
    }

    #[test]
    fn test_parse_vertical_align() {
        assert_eq!(parse_vertical_align("top"), CellVerticalAlign::Top);
        assert_eq!(parse_vertical_align("center"), CellVerticalAlign::Center);
        assert_eq!(parse_vertical_align("bottom"), CellVerticalAlign::Bottom);
    }
}
