//! RTF Writer - Serializes DocumentTree to RTF format
//!
//! This module generates valid RTF output from a document tree,
//! including all formatting, tables, and images.

use crate::rtf::control_words::*;
use crate::rtf::error::{RtfError, RtfResult};
use doc_model::{
    Alignment, CharacterProperties, DocumentTree, ImageNode, LineSpacing,
    Node, Paragraph, ParagraphProperties, Run, Table, TableCell, TableRow,
};
use std::collections::HashMap;
use std::io::Write;

/// RTF writer configuration
#[derive(Debug, Clone)]
pub struct RtfWriterConfig {
    /// Default font name
    pub default_font: String,
    /// Default font size in points
    pub default_font_size: f32,
    /// Whether to include metadata
    pub include_metadata: bool,
}

impl Default for RtfWriterConfig {
    fn default() -> Self {
        Self {
            default_font: "Calibri".to_string(),
            default_font_size: 11.0,
            include_metadata: true,
        }
    }
}

/// RTF Writer
pub struct RtfWriter<W: Write> {
    writer: W,
    config: RtfWriterConfig,
    /// Font table: name -> index
    fonts: HashMap<String, u32>,
    /// Color table: color string -> index
    colors: HashMap<String, u32>,
    /// Current font index
    current_font: u32,
    /// Current color index
    current_color: u32,
    /// Fonts used in document (for building font table)
    used_fonts: Vec<String>,
    /// Colors used in document (for building color table)
    used_colors: Vec<String>,
}

impl<W: Write> RtfWriter<W> {
    /// Create a new RTF writer
    pub fn new(writer: W) -> Self {
        Self::with_config(writer, RtfWriterConfig::default())
    }

    /// Create a new RTF writer with custom configuration
    pub fn with_config(writer: W, config: RtfWriterConfig) -> Self {
        Self {
            writer,
            config,
            fonts: HashMap::new(),
            colors: HashMap::new(),
            current_font: 0,
            current_color: 0,
            used_fonts: Vec::new(),
            used_colors: vec!["#000000".to_string()], // Default black at index 0
        }
    }

    /// Write the document tree to RTF format
    pub fn write(mut self, tree: &DocumentTree) -> RtfResult<()> {
        // First pass: collect all fonts and colors
        self.collect_fonts_and_colors(tree);

        // Build font and color tables
        self.build_tables();

        // Write RTF header
        self.write_header()?;

        // Write font table
        self.write_font_table()?;

        // Write color table
        self.write_color_table()?;

        // Write document content
        self.write_document_content(tree)?;

        // Close document
        self.write_str("}")?;

        self.writer.flush()?;
        Ok(())
    }

    /// Collect all fonts and colors used in the document
    fn collect_fonts_and_colors(&mut self, tree: &DocumentTree) {
        // Add default font
        if !self.used_fonts.contains(&self.config.default_font) {
            self.used_fonts.push(self.config.default_font.clone());
        }

        // Collect from runs
        for run in tree.nodes.runs.values() {
            if let Some(ref font) = run.direct_formatting.font_family {
                if !self.used_fonts.contains(font) {
                    self.used_fonts.push(font.clone());
                }
            }
            if let Some(ref color) = run.direct_formatting.color {
                if !self.used_colors.contains(color) {
                    self.used_colors.push(color.clone());
                }
            }
            if let Some(ref color) = run.direct_formatting.highlight {
                if !self.used_colors.contains(color) {
                    self.used_colors.push(color.clone());
                }
            }
        }
    }

    /// Build font and color lookup tables
    fn build_tables(&mut self) {
        for (idx, font) in self.used_fonts.iter().enumerate() {
            self.fonts.insert(font.clone(), idx as u32);
        }
        for (idx, color) in self.used_colors.iter().enumerate() {
            self.colors.insert(color.clone(), idx as u32);
        }
    }

    /// Write the RTF header
    fn write_header(&mut self) -> RtfResult<()> {
        // RTF version and character set
        self.write_str("{\\rtf1\\ansi\\ansicpg1252\\deff0")?;

        // Unicode skip count
        self.write_str("\\uc1")?;

        Ok(())
    }

    /// Write the font table
    fn write_font_table(&mut self) -> RtfResult<()> {
        self.write_str("{\\fonttbl")?;

        for (idx, font) in self.used_fonts.iter().enumerate() {
            // Font family detection (simplified)
            let family = if font.to_lowercase().contains("times") || font.to_lowercase().contains("serif") {
                "\\froman"
            } else if font.to_lowercase().contains("courier") || font.to_lowercase().contains("mono") {
                "\\fmodern"
            } else {
                "\\fswiss"
            };

            write!(
                self.writer,
                "{{\\f{}{} {};}}",
                idx, family, font
            )?;
        }

        self.write_str("}")?;
        Ok(())
    }

    /// Write the color table
    fn write_color_table(&mut self) -> RtfResult<()> {
        self.write_str("{\\colortbl;")?;

        for color in &self.used_colors {
            let (r, g, b) = parse_color(color);
            write!(self.writer, "\\red{}\\green{}\\blue{};", r, g, b)?;
        }

        self.write_str("}")?;
        Ok(())
    }

    /// Write the document content
    fn write_document_content(&mut self, tree: &DocumentTree) -> RtfResult<()> {
        // Write body children
        for &child_id in tree.document.children() {
            if let Some(para) = tree.nodes.paragraphs.get(&child_id) {
                self.write_paragraph(tree, para)?;
            } else if let Some(table) = tree.nodes.tables.get(&child_id) {
                self.write_table(tree, table)?;
            }
        }

        Ok(())
    }

    /// Write a paragraph
    fn write_paragraph(&mut self, tree: &DocumentTree, para: &Paragraph) -> RtfResult<()> {
        // Start paragraph with reset
        self.write_str("\\pard")?;

        // Paragraph formatting
        self.write_paragraph_formatting(&para.direct_formatting)?;

        // Write runs
        for &child_id in para.children() {
            if let Some(run) = tree.nodes.runs.get(&child_id) {
                self.write_run(run)?;
            } else if let Some(image) = tree.nodes.images.get(&child_id) {
                self.write_image(image)?;
            }
        }

        // End paragraph
        self.write_str("\\par\n")?;

        Ok(())
    }

    /// Write paragraph formatting
    fn write_paragraph_formatting(&mut self, props: &ParagraphProperties) -> RtfResult<()> {
        // Alignment
        if let Some(align) = props.alignment {
            match align {
                Alignment::Left => self.write_str("\\ql")?,
                Alignment::Center => self.write_str("\\qc")?,
                Alignment::Right => self.write_str("\\qr")?,
                Alignment::Justify => self.write_str("\\qj")?,
            }
        }

        // Indentation (convert points to twips)
        if let Some(li) = props.indent_left {
            write!(self.writer, "\\li{}", (li * 20.0) as i32)?;
        }
        if let Some(ri) = props.indent_right {
            write!(self.writer, "\\ri{}", (ri * 20.0) as i32)?;
        }
        if let Some(fi) = props.indent_first_line {
            write!(self.writer, "\\fi{}", (fi * 20.0) as i32)?;
        }

        // Spacing (convert points to twips)
        if let Some(sb) = props.space_before {
            write!(self.writer, "\\sb{}", (sb * 20.0) as i32)?;
        }
        if let Some(sa) = props.space_after {
            write!(self.writer, "\\sa{}", (sa * 20.0) as i32)?;
        }

        // Line spacing
        if let Some(ls) = props.line_spacing {
            match ls {
                LineSpacing::Multiple(mult) => {
                    write!(self.writer, "\\sl{}\\slmult1", (mult * 240.0) as i32)?;
                }
                LineSpacing::Exact(pts) => {
                    write!(self.writer, "\\sl-{}\\slmult0", (pts * 20.0) as i32)?;
                }
                LineSpacing::AtLeast(pts) => {
                    write!(self.writer, "\\sl{}\\slmult0", (pts * 20.0) as i32)?;
                }
            }
        }

        // Keep with next
        if props.keep_with_next == Some(true) {
            self.write_str("\\keepn")?;
        }

        // Keep together
        if props.keep_together == Some(true) {
            self.write_str("\\keep")?;
        }

        // Page break before
        if props.page_break_before == Some(true) {
            self.write_str("\\pagebb")?;
        }

        self.write_str(" ")?;
        Ok(())
    }

    /// Write a text run
    fn write_run(&mut self, run: &Run) -> RtfResult<()> {
        // Character formatting
        self.write_character_formatting(&run.direct_formatting)?;

        // Write text content with escaping
        self.write_text(&run.text)?;

        // Reset to plain if we had formatting
        if !run.direct_formatting.is_empty() {
            self.write_str("\\plain ")?;
        }

        Ok(())
    }

    /// Write character formatting
    fn write_character_formatting(&mut self, props: &CharacterProperties) -> RtfResult<()> {
        // Font
        if let Some(ref font) = props.font_family {
            if let Some(&idx) = self.fonts.get(font) {
                if idx != self.current_font {
                    write!(self.writer, "\\f{}", idx)?;
                    self.current_font = idx;
                }
            }
        }

        // Font size (convert points to half-points)
        if let Some(size) = props.font_size {
            write!(self.writer, "\\fs{}", (size * 2.0) as i32)?;
        }

        // Bold
        if props.bold == Some(true) {
            self.write_str("\\b")?;
        } else if props.bold == Some(false) {
            self.write_str("\\b0")?;
        }

        // Italic
        if props.italic == Some(true) {
            self.write_str("\\i")?;
        } else if props.italic == Some(false) {
            self.write_str("\\i0")?;
        }

        // Underline
        if props.underline == Some(true) {
            self.write_str("\\ul")?;
        } else if props.underline == Some(false) {
            self.write_str("\\ulnone")?;
        }

        // Strikethrough
        if props.strikethrough == Some(true) {
            self.write_str("\\strike")?;
        } else if props.strikethrough == Some(false) {
            self.write_str("\\strike0")?;
        }

        // Color
        if let Some(ref color) = props.color {
            if let Some(&idx) = self.colors.get(color) {
                write!(self.writer, "\\cf{}", idx)?;
            }
        }

        // Highlight/background color
        if let Some(ref color) = props.highlight {
            if let Some(&idx) = self.colors.get(color) {
                write!(self.writer, "\\cb{}", idx)?;
            }
        }

        self.write_str(" ")?;
        Ok(())
    }

    /// Write text with proper escaping
    fn write_text(&mut self, text: &str) -> RtfResult<()> {
        for ch in text.chars() {
            match ch {
                '\\' => self.write_str("\\\\")?,
                '{' => self.write_str("\\{")?,
                '}' => self.write_str("\\}")?,
                '\n' => self.write_str("\\line ")?,
                '\t' => self.write_str("\\tab ")?,
                '\u{00A0}' => self.write_str("\\~")?,      // Non-breaking space
                '\u{00AD}' => self.write_str("\\-")?,      // Soft hyphen
                '\u{2011}' => self.write_str("\\_")?,      // Non-breaking hyphen
                c if c as u32 > 127 => {
                    // Unicode character
                    let code = c as i32;
                    if code > 32767 {
                        // High Unicode needs to be negative
                        write!(self.writer, "\\u{}?", code - 65536)?;
                    } else {
                        write!(self.writer, "\\u{}?", code)?;
                    }
                }
                c => {
                    write!(self.writer, "{}", c)?;
                }
            }
        }
        Ok(())
    }

    /// Write a table
    fn write_table(&mut self, tree: &DocumentTree, table: &Table) -> RtfResult<()> {
        // Calculate column widths
        let col_widths: Vec<i32> = table.grid.columns.iter().map(|col| {
            match col.width.width_type {
                doc_model::WidthType::Fixed => (col.width.value * 20.0) as i32,
                _ => 1440, // Default 1 inch
            }
        }).collect();

        // Write rows
        for &row_id in table.children() {
            if let Some(row) = tree.nodes.table_rows.get(&row_id) {
                self.write_table_row(tree, row, &col_widths)?;
            }
        }

        Ok(())
    }

    /// Write a table row
    fn write_table_row(&mut self, tree: &DocumentTree, row: &TableRow, col_widths: &[i32]) -> RtfResult<()> {
        // Row definition
        self.write_str("\\trowd")?;

        // Row height
        if let Some(height) = row.properties.height {
            write!(self.writer, "\\trrh{}", (height * 20.0) as i32)?;
        }

        // Cell definitions (accumulating right boundaries)
        let mut right_boundary = 0;
        for (idx, &cell_id) in row.children().iter().enumerate() {
            let width = col_widths.get(idx).copied().unwrap_or(1440);
            right_boundary += width;

            // Cell borders (default)
            self.write_str("\\clbrdrt\\brdrs\\brdrw10")?;
            self.write_str("\\clbrdrb\\brdrs\\brdrw10")?;
            self.write_str("\\clbrdrl\\brdrs\\brdrw10")?;
            self.write_str("\\clbrdrr\\brdrs\\brdrw10")?;

            // Cell shading
            if let Some(cell) = tree.nodes.table_cells.get(&cell_id) {
                if let Some(ref shading) = cell.properties.shading {
                    if let Some(&idx) = self.colors.get(shading) {
                        write!(self.writer, "\\clcbpat{}", idx)?;
                    }
                }
            }

            write!(self.writer, "\\cellx{}", right_boundary)?;
        }

        self.write_str("\n")?;

        // Cell contents
        for &cell_id in row.children() {
            if let Some(cell) = tree.nodes.table_cells.get(&cell_id) {
                self.write_table_cell(tree, cell)?;
            }
        }

        // End row
        self.write_str("\\row\n")?;

        Ok(())
    }

    /// Write a table cell
    fn write_table_cell(&mut self, tree: &DocumentTree, cell: &TableCell) -> RtfResult<()> {
        self.write_str("\\intbl ")?;

        // Write cell content (paragraphs)
        for (idx, &child_id) in cell.children().iter().enumerate() {
            if let Some(para) = tree.nodes.paragraphs.get(&child_id) {
                // Don't add \par after the last paragraph in cell
                if idx > 0 {
                    self.write_str("\\par ")?;
                }

                // Paragraph formatting
                self.write_str("\\pard\\intbl")?;
                self.write_paragraph_formatting(&para.direct_formatting)?;

                // Write runs
                for &run_id in para.children() {
                    if let Some(run) = tree.nodes.runs.get(&run_id) {
                        self.write_run(run)?;
                    }
                }
            }
        }

        self.write_str("\\cell ")?;
        Ok(())
    }

    /// Write an image
    fn write_image(&mut self, image: &ImageNode) -> RtfResult<()> {
        // Image support requires the actual image data from an image store
        // For now, we write a placeholder
        // In a full implementation, this would embed the image data

        let width_twips = (image.effective_width(612.0) * 20.0) as i32;
        let height_twips = (image.effective_height(792.0) * 20.0) as i32;

        write!(
            self.writer,
            "{{\\pict\\pngblip\\picw{}\\pich{}\\picwgoal{}\\pichgoal{} }}",
            image.original_width * 20,
            image.original_height * 20,
            width_twips,
            height_twips
        )?;

        Ok(())
    }

    /// Helper to write a string
    fn write_str(&mut self, s: &str) -> RtfResult<()> {
        self.writer.write_all(s.as_bytes())?;
        Ok(())
    }
}

/// Parse a CSS color string to RGB components
fn parse_color(color: &str) -> (u8, u8, u8) {
    if color.starts_with('#') && color.len() >= 7 {
        let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
        (r, g, b)
    } else if color.starts_with("rgb(") {
        // Parse rgb(r, g, b) format
        let inner = color.trim_start_matches("rgb(").trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<u8>().unwrap_or(0);
            let g = parts[1].trim().parse::<u8>().unwrap_or(0);
            let b = parts[2].trim().parse::<u8>().unwrap_or(0);
            return (r, g, b);
        }
        (0, 0, 0)
    } else {
        // Named colors (basic support)
        match color.to_lowercase().as_str() {
            "black" => (0, 0, 0),
            "white" => (255, 255, 255),
            "red" => (255, 0, 0),
            "green" => (0, 128, 0),
            "blue" => (0, 0, 255),
            "yellow" => (255, 255, 0),
            "cyan" | "aqua" => (0, 255, 255),
            "magenta" | "fuchsia" => (255, 0, 255),
            "gray" | "grey" => (128, 128, 128),
            _ => (0, 0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::Node;

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#FF0000"), (255, 0, 0));
        assert_eq!(parse_color("#00FF00"), (0, 255, 0));
        assert_eq!(parse_color("#0000FF"), (0, 0, 255));
        assert_eq!(parse_color("#123456"), (18, 52, 86));
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("black"), (0, 0, 0));
        assert_eq!(parse_color("white"), (255, 255, 255));
        assert_eq!(parse_color("RED"), (255, 0, 0));
    }

    #[test]
    fn test_write_simple_document() {
        let mut tree = DocumentTree::new();

        // Create a paragraph with text
        let mut para = Paragraph::new();
        let para_id = para.id();

        let mut run = Run::new("Hello, RTF!");
        let run_id = run.id();
        run.set_parent(Some(para_id));

        para.add_child(run_id);
        tree.nodes.runs.insert(run_id, run);
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        // Write to RTF
        let mut output = Vec::new();
        let writer = RtfWriter::new(&mut output);
        writer.write(&tree).unwrap();

        let rtf = String::from_utf8(output).unwrap();
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.contains("Hello, RTF!"));
        assert!(rtf.ends_with("}"));
    }

    #[test]
    fn test_write_formatting() {
        let mut tree = DocumentTree::new();

        let mut para = Paragraph::new();
        let para_id = para.id();

        let formatting = CharacterProperties {
            bold: Some(true),
            font_size: Some(14.0),
            ..Default::default()
        };
        let mut run = Run::with_direct_formatting("Bold text", formatting);
        let run_id = run.id();
        run.set_parent(Some(para_id));

        para.add_child(run_id);
        tree.nodes.runs.insert(run_id, run);
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        let mut output = Vec::new();
        let writer = RtfWriter::new(&mut output);
        writer.write(&tree).unwrap();

        let rtf = String::from_utf8(output).unwrap();
        assert!(rtf.contains("\\b"));
        assert!(rtf.contains("\\fs28")); // 14 * 2 = 28 half-points
    }
}
