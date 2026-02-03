//! Table writer for DOCX files
//!
//! Generates w:tbl elements from doc_model tables.

use crate::docx::error::DocxResult;
use doc_model::{
    CellVerticalAlign, DocumentTree, HeightRule, Node, Paragraph, Run, Table, TableAlignment,
    TableCell, TableRow, TableWidth, WidthType,
};

/// Writer for table elements
pub struct TableWriter;

impl TableWriter {
    /// Create a new table writer
    pub fn new() -> Self {
        Self
    }

    /// Write a table element
    pub fn write_table(
        &self,
        xml: &mut String,
        tree: &DocumentTree,
        table: &Table,
    ) -> DocxResult<()> {
        xml.push_str("<w:tbl>");

        // Table properties
        self.write_table_properties(xml, table)?;

        // Table grid
        self.write_table_grid(xml, table)?;

        // Table rows
        for row_id in table.children() {
            if let Some(row) = tree.nodes.table_rows.get(row_id) {
                self.write_table_row(xml, tree, row)?;
            }
        }

        xml.push_str("</w:tbl>");
        Ok(())
    }

    /// Write table properties
    fn write_table_properties(&self, xml: &mut String, table: &Table) -> DocxResult<()> {
        let props = &table.properties;

        xml.push_str("<w:tblPr>");

        // Table style
        if let Some(ref style) = props.style_id {
            xml.push_str(&format!(r#"<w:tblStyle w:val="{}"/>"#, style.as_str()));
        }

        // Table width
        if let Some(ref width) = props.width {
            let (w, t) = format_table_width(width);
            xml.push_str(&format!(r#"<w:tblW w:w="{}" w:type="{}"/>"#, w, t));
        }

        // Alignment
        if let Some(alignment) = props.alignment {
            let val = match alignment {
                TableAlignment::Left => "left",
                TableAlignment::Center => "center",
                TableAlignment::Right => "right",
            };
            xml.push_str(&format!(r#"<w:jc w:val="{}"/>"#, val));
        }

        // Indent
        if let Some(indent) = props.indent_left {
            xml.push_str(&format!(r#"<w:tblInd w:w="{}" w:type="dxa"/>"#, (indent * 20.0) as i32));
        }

        // Cell margins (default)
        xml.push_str("<w:tblCellMar>");
        xml.push_str(r#"<w:top w:w="0" w:type="dxa"/>"#);
        xml.push_str(r#"<w:left w:w="108" w:type="dxa"/>"#);
        xml.push_str(r#"<w:bottom w:w="0" w:type="dxa"/>"#);
        xml.push_str(r#"<w:right w:w="108" w:type="dxa"/>"#);
        xml.push_str("</w:tblCellMar>");

        // Look (first row, last row, etc.)
        xml.push_str(r#"<w:tblLook w:val="04A0" w:firstRow="1" w:lastRow="0" w:firstColumn="1" w:lastColumn="0" w:noHBand="0" w:noVBand="1"/>"#);

        xml.push_str("</w:tblPr>");
        Ok(())
    }

    /// Write table grid
    fn write_table_grid(&self, xml: &mut String, table: &Table) -> DocxResult<()> {
        xml.push_str("<w:tblGrid>");

        for col in &table.grid.columns {
            let width_twips = (col.width.value * 20.0) as i32;
            xml.push_str(&format!(r#"<w:gridCol w:w="{}"/>"#, width_twips));
        }

        xml.push_str("</w:tblGrid>");
        Ok(())
    }

    /// Write a table row
    fn write_table_row(
        &self,
        xml: &mut String,
        tree: &DocumentTree,
        row: &TableRow,
    ) -> DocxResult<()> {
        xml.push_str("<w:tr>");

        // Row properties
        self.write_row_properties(xml, row)?;

        // Cells
        for cell_id in row.children() {
            if let Some(cell) = tree.nodes.table_cells.get(cell_id) {
                self.write_table_cell(xml, tree, cell)?;
            }
        }

        xml.push_str("</w:tr>");
        Ok(())
    }

    /// Write row properties
    fn write_row_properties(&self, xml: &mut String, row: &TableRow) -> DocxResult<()> {
        let props = &row.properties;

        // Only write if there are properties
        let has_height = props.height.is_some();
        let has_header = props.is_header;
        let has_cant_split = !props.can_split;

        if !has_height && !has_header && !has_cant_split {
            return Ok(());
        }

        xml.push_str("<w:trPr>");

        // Row height
        if let Some(height) = props.height {
            let twips = (height * 20.0) as i32;
            let rule = match props.height_rule {
                HeightRule::Auto => "auto",
                HeightRule::AtLeast => "atLeast",
                HeightRule::Exact => "exact",
            };
            xml.push_str(&format!(r#"<w:trHeight w:val="{}" w:hRule="{}"/>"#, twips, rule));
        }

        // Header row
        if props.is_header {
            xml.push_str("<w:tblHeader/>");
        }

        // Can't split
        if !props.can_split {
            xml.push_str("<w:cantSplit/>");
        }

        xml.push_str("</w:trPr>");
        Ok(())
    }

    /// Write a table cell
    fn write_table_cell(
        &self,
        xml: &mut String,
        tree: &DocumentTree,
        cell: &TableCell,
    ) -> DocxResult<()> {
        xml.push_str("<w:tc>");

        // Cell properties
        self.write_cell_properties(xml, cell)?;

        // Cell content (paragraphs)
        for content_id in cell.children() {
            if let Some(para) = tree.nodes.paragraphs.get(content_id) {
                self.write_paragraph(xml, tree, para)?;
            }
        }

        // Ensure at least one paragraph
        if cell.children().is_empty() {
            xml.push_str("<w:p/>");
        }

        xml.push_str("</w:tc>");
        Ok(())
    }

    /// Write cell properties
    fn write_cell_properties(&self, xml: &mut String, cell: &TableCell) -> DocxResult<()> {
        let props = &cell.properties;

        xml.push_str("<w:tcPr>");

        // Cell width
        if let Some(ref width) = props.width {
            let (w, t) = format_table_width(width);
            xml.push_str(&format!(r#"<w:tcW w:w="{}" w:type="{}"/>"#, w, t));
        }

        // Grid span (horizontal merge)
        if cell.grid_span > 1 {
            xml.push_str(&format!(r#"<w:gridSpan w:val="{}"/>"#, cell.grid_span));
        }

        // Vertical merge
        if cell.row_span > 1 {
            xml.push_str(r#"<w:vMerge w:val="restart"/>"#);
        } else if cell.v_merge_continue {
            xml.push_str("<w:vMerge/>");
        }

        // Vertical alignment
        if let Some(valign) = props.vertical_align {
            let val = match valign {
                CellVerticalAlign::Top => "top",
                CellVerticalAlign::Center => "center",
                CellVerticalAlign::Bottom => "bottom",
            };
            xml.push_str(&format!(r#"<w:vAlign w:val="{}"/>"#, val));
        }

        // Shading
        if let Some(ref shading) = props.shading {
            let color = shading.trim_start_matches('#');
            xml.push_str(&format!(
                r#"<w:shd w:val="clear" w:color="auto" w:fill="{}"/>"#,
                color
            ));
        }

        xml.push_str("</w:tcPr>");
        Ok(())
    }

    /// Write a paragraph within a cell (simplified version)
    fn write_paragraph(
        &self,
        xml: &mut String,
        tree: &DocumentTree,
        para: &Paragraph,
    ) -> DocxResult<()> {
        xml.push_str("<w:p>");

        // Simple paragraph - just write runs
        for child_id in para.children() {
            if let Some(run) = tree.nodes.runs.get(child_id) {
                self.write_run(xml, run)?;
            }
        }

        xml.push_str("</w:p>");
        Ok(())
    }

    /// Write a run within a cell paragraph
    fn write_run(&self, xml: &mut String, run: &Run) -> DocxResult<()> {
        xml.push_str("<w:r>");
        xml.push_str("<w:t>");
        xml.push_str(&escape_xml(&run.text));
        xml.push_str("</w:t>");
        xml.push_str("</w:r>");
        Ok(())
    }
}

/// Format table width for XML output
fn format_table_width(width: &TableWidth) -> (i32, &'static str) {
    match width.width_type {
        WidthType::Auto => (0, "auto"),
        WidthType::Fixed => ((width.value * 20.0) as i32, "dxa"),
        WidthType::Percent => ((width.value * 50.0) as i32, "pct"),
    }
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_table_width() {
        let fixed = TableWidth::fixed(72.0);
        let (w, t) = format_table_width(&fixed);
        assert_eq!(w, 1440);
        assert_eq!(t, "dxa");

        let pct = TableWidth::percent(50.0);
        let (w, t) = format_table_width(&pct);
        assert_eq!(w, 2500);
        assert_eq!(t, "pct");

        let auto = TableWidth::auto();
        let (_, t) = format_table_width(&auto);
        assert_eq!(t, "auto");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Hello & World"), "Hello &amp; World");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }
}
