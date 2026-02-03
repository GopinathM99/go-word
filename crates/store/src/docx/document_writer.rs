//! Document.xml writer
//!
//! Converts the DocumentTree to DOCX document.xml format.

use crate::docx::error::DocxResult;
use crate::docx::namespaces;
use crate::docx::tables_writer::TableWriter;
use doc_model::{
    Alignment, CharacterProperties, DocumentTree, Hyperlink, HyperlinkTarget, LineSpacing,
    Node, NodeType, Paragraph, ParagraphProperties, Run,
};

/// Writer for document.xml
pub struct DocumentWriter {
    /// External hyperlinks to be added to relationships
    pub hyperlinks: Vec<(String, String)>,
    next_hyperlink_id: u32,
}

impl DocumentWriter {
    /// Create a new document writer
    pub fn new() -> Self {
        Self {
            hyperlinks: Vec::new(),
            next_hyperlink_id: 1,
        }
    }

    /// Generate document.xml content
    pub fn write(&mut self, tree: &DocumentTree) -> DocxResult<String> {
        let mut xml = String::new();

        // XML declaration
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');

        // Document element with namespaces
        xml.push_str(&format!(
            r#"<w:document xmlns:w="{}" xmlns:r="{}" xmlns:wp="{}" xmlns:a="{}">"#,
            namespaces::W,
            namespaces::R,
            namespaces::WP,
            namespaces::A,
        ));

        // Body
        xml.push_str("<w:body>");

        // Write body content
        for child_id in tree.document.children() {
            self.write_body_element(&mut xml, tree, *child_id)?;
        }

        // Close body and document
        xml.push_str("</w:body>");
        xml.push_str("</w:document>");

        Ok(xml)
    }

    /// Write a body-level element (paragraph, table, etc.)
    fn write_body_element(
        &mut self,
        xml: &mut String,
        tree: &DocumentTree,
        node_id: doc_model::NodeId,
    ) -> DocxResult<()> {
        // Determine node type and write accordingly
        if let Some(para) = tree.nodes.paragraphs.get(&node_id) {
            self.write_paragraph(xml, tree, para)?;
        } else if let Some(table) = tree.nodes.tables.get(&node_id) {
            TableWriter::new().write_table(xml, tree, table)?;
        }

        Ok(())
    }

    /// Write a paragraph element
    fn write_paragraph(
        &mut self,
        xml: &mut String,
        tree: &DocumentTree,
        para: &Paragraph,
    ) -> DocxResult<()> {
        xml.push_str("<w:p>");

        // Paragraph properties
        self.write_paragraph_properties(xml, para)?;

        // Paragraph content (runs and hyperlinks)
        for child_id in para.children() {
            if let Some(run) = tree.nodes.runs.get(child_id) {
                self.write_run(xml, run)?;
            } else if let Some(hyperlink) = tree.nodes.hyperlinks.get(child_id) {
                self.write_hyperlink(xml, tree, hyperlink)?;
            }
        }

        xml.push_str("</w:p>");
        Ok(())
    }

    /// Write paragraph properties
    fn write_paragraph_properties(&self, xml: &mut String, para: &Paragraph) -> DocxResult<()> {
        let props = &para.direct_formatting;
        let style_id = para.paragraph_style_id.as_ref();

        // Only write pPr if there's something to write
        let has_style = style_id.is_some();
        let has_props = !props.is_empty();

        if !has_style && !has_props {
            return Ok(());
        }

        xml.push_str("<w:pPr>");

        // Style reference
        if let Some(style) = style_id {
            xml.push_str(&format!(r#"<w:pStyle w:val="{}"/>"#, style.as_str()));
        }

        // Alignment
        if let Some(alignment) = props.alignment {
            let val = match alignment {
                Alignment::Left => "left",
                Alignment::Center => "center",
                Alignment::Right => "right",
                Alignment::Justify => "both",
            };
            xml.push_str(&format!(r#"<w:jc w:val="{}"/>"#, val));
        }

        // Indentation
        if props.indent_left.is_some() || props.indent_right.is_some() || props.indent_first_line.is_some() {
            xml.push_str("<w:ind");
            if let Some(left) = props.indent_left {
                xml.push_str(&format!(r#" w:left="{}""#, (left * 20.0) as i32));
            }
            if let Some(right) = props.indent_right {
                xml.push_str(&format!(r#" w:right="{}""#, (right * 20.0) as i32));
            }
            if let Some(first) = props.indent_first_line {
                if first >= 0.0 {
                    xml.push_str(&format!(r#" w:firstLine="{}""#, (first * 20.0) as i32));
                } else {
                    xml.push_str(&format!(r#" w:hanging="{}""#, (-first * 20.0) as i32));
                }
            }
            xml.push_str("/>");
        }

        // Spacing
        if props.space_before.is_some() || props.space_after.is_some() || props.line_spacing.is_some() {
            xml.push_str("<w:spacing");
            if let Some(before) = props.space_before {
                xml.push_str(&format!(r#" w:before="{}""#, (before * 20.0) as i32));
            }
            if let Some(after) = props.space_after {
                xml.push_str(&format!(r#" w:after="{}""#, (after * 20.0) as i32));
            }
            if let Some(line_spacing) = props.line_spacing {
                match line_spacing {
                    LineSpacing::Multiple(mult) => {
                        xml.push_str(&format!(r#" w:line="{}" w:lineRule="auto""#, (mult * 240.0) as i32));
                    }
                    LineSpacing::Exact(pts) => {
                        xml.push_str(&format!(r#" w:line="{}" w:lineRule="exact""#, (pts * 20.0) as i32));
                    }
                    LineSpacing::AtLeast(pts) => {
                        xml.push_str(&format!(r#" w:line="{}" w:lineRule="atLeast""#, (pts * 20.0) as i32));
                    }
                }
            }
            xml.push_str("/>");
        }

        // Keep with next
        if props.keep_with_next == Some(true) {
            xml.push_str("<w:keepNext/>");
        }

        // Keep together
        if props.keep_together == Some(true) {
            xml.push_str("<w:keepLines/>");
        }

        // Page break before
        if props.page_break_before == Some(true) {
            xml.push_str("<w:pageBreakBefore/>");
        }

        xml.push_str("</w:pPr>");
        Ok(())
    }

    /// Write a run element
    fn write_run(&self, xml: &mut String, run: &Run) -> DocxResult<()> {
        xml.push_str("<w:r>");

        // Run properties
        self.write_run_properties(xml, run)?;

        // Text content - handle special characters
        let text = &run.text;
        for part in text.split('\n') {
            // Handle tabs
            let parts: Vec<&str> = part.split('\t').collect();
            for (i, segment) in parts.iter().enumerate() {
                if !segment.is_empty() {
                    // Write text with xml:space="preserve" for leading/trailing spaces
                    let needs_preserve = segment.starts_with(' ') || segment.ends_with(' ');
                    if needs_preserve {
                        xml.push_str(r#"<w:t xml:space="preserve">"#);
                    } else {
                        xml.push_str("<w:t>");
                    }
                    xml.push_str(&escape_xml(segment));
                    xml.push_str("</w:t>");
                }
                if i < parts.len() - 1 {
                    xml.push_str("<w:tab/>");
                }
            }
            // Add line break if not the last part
            if part != text.split('\n').last().unwrap_or("") {
                xml.push_str("<w:br/>");
            }
        }

        xml.push_str("</w:r>");
        Ok(())
    }

    /// Write run properties
    fn write_run_properties(&self, xml: &mut String, run: &Run) -> DocxResult<()> {
        let props = &run.direct_formatting;
        let style_id = run.character_style_id.as_ref();

        let has_style = style_id.is_some();
        let has_props = !props.is_empty();

        if !has_style && !has_props {
            return Ok(());
        }

        xml.push_str("<w:rPr>");

        // Style reference
        if let Some(style) = style_id {
            xml.push_str(&format!(r#"<w:rStyle w:val="{}"/>"#, style.as_str()));
        }

        // Font family
        if let Some(ref font) = props.font_family {
            xml.push_str(&format!(
                r#"<w:rFonts w:ascii="{}" w:hAnsi="{}"/>"#,
                escape_xml(font),
                escape_xml(font)
            ));
        }

        // Font size (in half-points)
        if let Some(size) = props.font_size {
            let half_pts = (size * 2.0) as i32;
            xml.push_str(&format!(r#"<w:sz w:val="{}"/>"#, half_pts));
            xml.push_str(&format!(r#"<w:szCs w:val="{}"/>"#, half_pts));
        }

        // Bold
        if let Some(bold) = props.bold {
            if bold {
                xml.push_str("<w:b/>");
            } else {
                xml.push_str(r#"<w:b w:val="0"/>"#);
            }
        }

        // Italic
        if let Some(italic) = props.italic {
            if italic {
                xml.push_str("<w:i/>");
            } else {
                xml.push_str(r#"<w:i w:val="0"/>"#);
            }
        }

        // Underline
        if let Some(underline) = props.underline {
            if underline {
                xml.push_str(r#"<w:u w:val="single"/>"#);
            } else {
                xml.push_str(r#"<w:u w:val="none"/>"#);
            }
        }

        // Strikethrough
        if let Some(strike) = props.strikethrough {
            if strike {
                xml.push_str("<w:strike/>");
            } else {
                xml.push_str(r#"<w:strike w:val="0"/>"#);
            }
        }

        // Color
        if let Some(ref color) = props.color {
            let color_val = color.trim_start_matches('#');
            xml.push_str(&format!(r#"<w:color w:val="{}"/>"#, color_val));
        }

        // Highlight
        if let Some(ref highlight) = props.highlight {
            let highlight_name = color_to_highlight(highlight);
            xml.push_str(&format!(r#"<w:highlight w:val="{}"/>"#, highlight_name));
        }

        // All caps
        if let Some(caps) = props.all_caps {
            if caps {
                xml.push_str("<w:caps/>");
            }
        }

        // Small caps
        if let Some(small_caps) = props.small_caps {
            if small_caps {
                xml.push_str("<w:smallCaps/>");
            }
        }

        xml.push_str("</w:rPr>");
        Ok(())
    }

    /// Write a hyperlink element
    fn write_hyperlink(
        &mut self,
        xml: &mut String,
        tree: &DocumentTree,
        hyperlink: &Hyperlink,
    ) -> DocxResult<()> {
        // Determine how to reference the hyperlink
        match &hyperlink.target {
            HyperlinkTarget::Internal(bookmark) => {
                xml.push_str(&format!(r#"<w:hyperlink w:anchor="{}">"#, escape_xml(bookmark)));
            }
            HyperlinkTarget::External(url) => {
                // Create a relationship ID
                let rel_id = format!("rId{}", self.next_hyperlink_id);
                self.next_hyperlink_id += 1;
                self.hyperlinks.push((rel_id.clone(), url.clone()));
                xml.push_str(&format!(r#"<w:hyperlink r:id="{}">"#, rel_id));
            }
            HyperlinkTarget::Email { address, subject } => {
                let mut url = format!("mailto:{}", address);
                if let Some(subj) = subject {
                    url.push_str(&format!("?subject={}", urlencoding_encode(subj)));
                }
                let rel_id = format!("rId{}", self.next_hyperlink_id);
                self.next_hyperlink_id += 1;
                self.hyperlinks.push((rel_id.clone(), url));
                xml.push_str(&format!(r#"<w:hyperlink r:id="{}">"#, rel_id));
            }
        }

        // Write hyperlink content (runs)
        for child_id in hyperlink.children() {
            if let Some(run) = tree.nodes.runs.get(child_id) {
                self.write_run(xml, run)?;
            }
        }

        xml.push_str("</w:hyperlink>");
        Ok(())
    }
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Simple URL encoding for subject parameter
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            ' ' => {
                result.push_str("%20");
            }
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}

/// Convert CSS color to Word highlight color name
fn color_to_highlight(color: &str) -> &'static str {
    let color = color.to_lowercase();
    match color.as_str() {
        "#ffff00" | "yellow" => "yellow",
        "#00ff00" | "lime" | "green" => "green",
        "#00ffff" | "cyan" | "aqua" => "cyan",
        "#ff00ff" | "magenta" | "fuchsia" => "magenta",
        "#0000ff" | "blue" => "blue",
        "#ff0000" | "red" => "red",
        "#000080" | "navy" => "darkBlue",
        "#008080" | "teal" => "darkCyan",
        "#008000" => "darkGreen",
        "#800080" | "purple" => "darkMagenta",
        "#800000" | "maroon" => "darkRed",
        "#808000" | "olive" => "darkYellow",
        "#808080" | "gray" | "grey" => "darkGray",
        "#c0c0c0" | "silver" => "lightGray",
        "#000000" | "black" => "black",
        "#ffffff" | "white" => "white",
        _ => "yellow", // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Hello & World"), "Hello &amp; World");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_color_to_highlight() {
        assert_eq!(color_to_highlight("#FFFF00"), "yellow");
        assert_eq!(color_to_highlight("#0000FF"), "blue");
        assert_eq!(color_to_highlight("#FF0000"), "red");
        assert_eq!(color_to_highlight("yellow"), "yellow");
    }

    #[test]
    fn test_urlencoding_encode() {
        assert_eq!(urlencoding_encode("Hello World"), "Hello%20World");
        assert_eq!(urlencoding_encode("test@example.com"), "test%40example.com");
    }

    #[test]
    fn test_document_writer_basic() {
        let tree = DocumentTree::new();
        let mut writer = DocumentWriter::new();
        let xml = writer.write(&tree).unwrap();

        assert!(xml.contains("w:document"));
        assert!(xml.contains("w:body"));
    }
}
