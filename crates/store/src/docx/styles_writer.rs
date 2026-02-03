//! Styles.xml writer
//!
//! Generates styles.xml from the document's style registry.

use crate::docx::error::DocxResult;
use crate::docx::namespaces;
use doc_model::{
    Alignment, CharacterProperties, DocumentTree, LineSpacing, ParagraphProperties, Style,
    StyleType,
};

/// Writer for styles.xml
pub struct StylesWriter;

impl StylesWriter {
    /// Create a new styles writer
    pub fn new() -> Self {
        Self
    }

    /// Generate styles.xml content
    pub fn write(&self, tree: &DocumentTree) -> DocxResult<String> {
        let mut xml = String::new();

        // XML declaration
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');

        // Styles element with namespace
        xml.push_str(&format!(
            r#"<w:styles xmlns:w="{}" xmlns:r="{}">"#,
            namespaces::W,
            namespaces::R,
        ));

        // Write default styles (docDefaults)
        self.write_doc_defaults(&mut xml)?;

        // Write all registered styles
        for style in tree.style_registry().all_styles() {
            self.write_style(&mut xml, style)?;
        }

        xml.push_str("</w:styles>");
        Ok(xml)
    }

    /// Write document defaults
    fn write_doc_defaults(&self, xml: &mut String) -> DocxResult<()> {
        xml.push_str("<w:docDefaults>");

        // Run properties defaults
        xml.push_str("<w:rPrDefault>");
        xml.push_str("<w:rPr>");
        xml.push_str(r#"<w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/>"#);
        xml.push_str(r#"<w:sz w:val="22"/>"#);
        xml.push_str(r#"<w:szCs w:val="22"/>"#);
        xml.push_str("</w:rPr>");
        xml.push_str("</w:rPrDefault>");

        // Paragraph properties defaults
        xml.push_str("<w:pPrDefault>");
        xml.push_str("<w:pPr>");
        xml.push_str(r#"<w:spacing w:after="160" w:line="259" w:lineRule="auto"/>"#);
        xml.push_str("</w:pPr>");
        xml.push_str("</w:pPrDefault>");

        xml.push_str("</w:docDefaults>");
        Ok(())
    }

    /// Write a single style definition
    fn write_style(&self, xml: &mut String, style: &Style) -> DocxResult<()> {
        // Style element with type and ID
        let type_str = match style.style_type {
            StyleType::Paragraph => "paragraph",
            StyleType::Character => "character",
            StyleType::Table => "table",
            StyleType::Numbering => "numbering",
        };

        xml.push_str(&format!(
            r#"<w:style w:type="{}" w:styleId="{}""#,
            type_str,
            escape_xml(&style.id.as_str())
        ));

        if style.built_in {
            xml.push_str(r#" w:default="1""#);
        }

        xml.push('>');

        // Style name
        xml.push_str(&format!(
            r#"<w:name w:val="{}"/>"#,
            escape_xml(&style.name)
        ));

        // Based on
        if let Some(ref based_on) = style.based_on {
            xml.push_str(&format!(
                r#"<w:basedOn w:val="{}"/>"#,
                escape_xml(based_on.as_str())
            ));
        }

        // Next style
        if let Some(ref next) = style.next_style {
            xml.push_str(&format!(
                r#"<w:next w:val="{}"/>"#,
                escape_xml(next.as_str())
            ));
        }

        // UI Priority
        xml.push_str(&format!(r#"<w:uiPriority w:val="{}"/>"#, style.priority));

        // Quick format (show in gallery)
        if !style.hidden {
            xml.push_str("<w:qFormat/>");
        }

        // Hidden
        if style.hidden {
            xml.push_str("<w:hidden/>");
        }

        // Paragraph properties
        if style.style_type == StyleType::Paragraph && !style.paragraph_props.is_empty() {
            self.write_paragraph_properties(xml, &style.paragraph_props)?;
        }

        // Character properties
        if !style.character_props.is_empty() {
            self.write_character_properties(xml, &style.character_props)?;
        }

        xml.push_str("</w:style>");
        Ok(())
    }

    /// Write paragraph properties
    fn write_paragraph_properties(
        &self,
        xml: &mut String,
        props: &ParagraphProperties,
    ) -> DocxResult<()> {
        xml.push_str("<w:pPr>");

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

        // Outline level
        if let Some(level) = props.outline_level {
            xml.push_str(&format!(r#"<w:outlineLvl w:val="{}"/>"#, level));
        }

        xml.push_str("</w:pPr>");
        Ok(())
    }

    /// Write character properties
    fn write_character_properties(
        &self,
        xml: &mut String,
        props: &CharacterProperties,
    ) -> DocxResult<()> {
        xml.push_str("<w:rPr>");

        // Font family
        if let Some(ref font) = props.font_family {
            xml.push_str(&format!(
                r#"<w:rFonts w:ascii="{}" w:hAnsi="{}" w:cs="{}"/>"#,
                escape_xml(font),
                escape_xml(font),
                escape_xml(font)
            ));
        }

        // Font size
        if let Some(size) = props.font_size {
            let half_pts = (size * 2.0) as i32;
            xml.push_str(&format!(r#"<w:sz w:val="{}"/>"#, half_pts));
            xml.push_str(&format!(r#"<w:szCs w:val="{}"/>"#, half_pts));
        }

        // Bold
        if let Some(bold) = props.bold {
            if bold {
                xml.push_str("<w:b/>");
                xml.push_str("<w:bCs/>");
            }
        }

        // Italic
        if let Some(italic) = props.italic {
            if italic {
                xml.push_str("<w:i/>");
                xml.push_str("<w:iCs/>");
            }
        }

        // Underline
        if let Some(underline) = props.underline {
            if underline {
                xml.push_str(r#"<w:u w:val="single"/>"#);
            }
        }

        // Strikethrough
        if let Some(strike) = props.strikethrough {
            if strike {
                xml.push_str("<w:strike/>");
            }
        }

        // Color
        if let Some(ref color) = props.color {
            let color_val = color.trim_start_matches('#');
            xml.push_str(&format!(r#"<w:color w:val="{}"/>"#, color_val));
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

        // Character spacing
        if let Some(spacing) = props.spacing {
            xml.push_str(&format!(r#"<w:spacing w:val="{}"/>"#, (spacing * 20.0) as i32));
        }

        xml.push_str("</w:rPr>");
        Ok(())
    }
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styles_writer_basic() {
        let tree = DocumentTree::new();
        let writer = StylesWriter::new();
        let xml = writer.write(&tree).unwrap();

        assert!(xml.contains("w:styles"));
        assert!(xml.contains("w:docDefaults"));
    }

    #[test]
    fn test_doc_defaults() {
        let writer = StylesWriter::new();
        let mut xml = String::new();
        writer.write_doc_defaults(&mut xml).unwrap();

        assert!(xml.contains("w:rPrDefault"));
        assert!(xml.contains("w:pPrDefault"));
        assert!(xml.contains("Calibri"));
    }
}
