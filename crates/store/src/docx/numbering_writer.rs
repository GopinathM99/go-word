//! Numbering.xml writer
//!
//! Generates numbering.xml from the document's numbering registry.

use crate::docx::error::DocxResult;
use crate::docx::namespaces;
use doc_model::{
    AbstractNum, DocumentTree, ListLevel, ListLevelAlignment, ListLevelSuffix, MultiLevelType,
    NumberingInstance, list::NumberFormat,
};

/// Writer for numbering.xml
pub struct NumberingWriter;

impl NumberingWriter {
    /// Create a new numbering writer
    pub fn new() -> Self {
        Self
    }

    /// Generate numbering.xml content
    pub fn write(&self, tree: &DocumentTree) -> DocxResult<String> {
        let mut xml = String::new();

        // XML declaration
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');

        // Numbering element with namespace
        xml.push_str(&format!(
            r#"<w:numbering xmlns:w="{}" xmlns:r="{}">"#,
            namespaces::W,
            namespaces::R,
        ));

        // Write abstract numbering definitions
        for abstract_num in tree.numbering_registry().all_abstract_nums() {
            self.write_abstract_num(&mut xml, abstract_num)?;
        }

        // Write numbering instances
        for instance in tree.numbering_registry().all_instances() {
            self.write_num_instance(&mut xml, instance)?;
        }

        xml.push_str("</w:numbering>");
        Ok(xml)
    }

    /// Write an abstract numbering definition
    fn write_abstract_num(&self, xml: &mut String, abs: &AbstractNum) -> DocxResult<()> {
        xml.push_str(&format!(
            r#"<w:abstractNum w:abstractNumId="{}">"#,
            abs.id.0
        ));

        // Multi-level type
        let mlt = match abs.multi_level_type {
            MultiLevelType::SingleLevel => "singleLevel",
            MultiLevelType::MultiLevel => "multilevel",
            MultiLevelType::HybridMultiLevel => "hybridMultilevel",
        };
        xml.push_str(&format!(r#"<w:multiLevelType w:val="{}"/>"#, mlt));

        // Name (if present)
        if let Some(ref name) = abs.name {
            xml.push_str(&format!(r#"<w:name w:val="{}"/>"#, escape_xml(name)));
        }

        // Levels
        for level in &abs.levels {
            self.write_level(xml, level)?;
        }

        xml.push_str("</w:abstractNum>");
        Ok(())
    }

    /// Write a list level definition
    fn write_level(&self, xml: &mut String, level: &ListLevel) -> DocxResult<()> {
        xml.push_str(&format!(r#"<w:lvl w:ilvl="{}">"#, level.level));

        // Start value
        xml.push_str(&format!(r#"<w:start w:val="{}"/>"#, level.start));

        // Number format
        let fmt = match level.format {
            NumberFormat::Decimal => "decimal",
            NumberFormat::DecimalZero => "decimalZero",
            NumberFormat::LowerLetter => "lowerLetter",
            NumberFormat::UpperLetter => "upperLetter",
            NumberFormat::LowerRoman => "lowerRoman",
            NumberFormat::UpperRoman => "upperRoman",
            NumberFormat::Bullet => "bullet",
            NumberFormat::None => "none",
            NumberFormat::Ordinal => "ordinal",
            NumberFormat::CardinalText => "cardinalText",
            NumberFormat::OrdinalText => "ordinalText",
        };
        xml.push_str(&format!(r#"<w:numFmt w:val="{}"/>"#, fmt));

        // Level text
        xml.push_str(&format!(r#"<w:lvlText w:val="{}"/>"#, escape_xml(&level.text)));

        // Level justification
        let jc = match level.alignment {
            ListLevelAlignment::Left => "left",
            ListLevelAlignment::Center => "center",
            ListLevelAlignment::Right => "right",
        };
        xml.push_str(&format!(r#"<w:lvlJc w:val="{}"/>"#, jc));

        // Suffix
        let suff = match level.suffix {
            ListLevelSuffix::Tab => "tab",
            ListLevelSuffix::Space => "space",
            ListLevelSuffix::Nothing => "nothing",
        };
        xml.push_str(&format!(r#"<w:suff w:val="{}"/>"#, suff));

        // Paragraph properties (indentation)
        xml.push_str("<w:pPr>");
        xml.push_str(&format!(
            r#"<w:ind w:left="{}" w:hanging="{}"/>"#,
            (level.indent * 20.0) as i32,
            (level.hanging * 20.0) as i32
        ));
        if let Some(tab_stop) = level.tab_stop {
            xml.push_str(&format!(
                r#"<w:tabs><w:tab w:val="num" w:pos="{}"/></w:tabs>"#,
                (tab_stop * 20.0) as i32
            ));
        }
        xml.push_str("</w:pPr>");

        // Run properties (for bullet font)
        if level.format == NumberFormat::Bullet {
            if let Some(ref font) = level.font {
                xml.push_str("<w:rPr>");
                xml.push_str(&format!(
                    r#"<w:rFonts w:ascii="{}" w:hAnsi="{}" w:hint="default"/>"#,
                    escape_xml(font),
                    escape_xml(font)
                ));
                xml.push_str("</w:rPr>");
            }
        }

        xml.push_str("</w:lvl>");
        Ok(())
    }

    /// Write a numbering instance
    fn write_num_instance(&self, xml: &mut String, inst: &NumberingInstance) -> DocxResult<()> {
        xml.push_str(&format!(r#"<w:num w:numId="{}">"#, inst.id.0));
        xml.push_str(&format!(r#"<w:abstractNumId w:val="{}"/>"#, inst.abstract_num_id.0));

        // Level overrides (if any)
        for (level, override_data) in &inst.level_overrides {
            xml.push_str(&format!(r#"<w:lvlOverride w:ilvl="{}">"#, level));
            if let Some(start) = override_data.start_override {
                xml.push_str(&format!(r#"<w:startOverride w:val="{}"/>"#, start));
            }
            xml.push_str("</w:lvlOverride>");
        }

        xml.push_str("</w:num>");
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
    fn test_numbering_writer_basic() {
        let tree = DocumentTree::new();
        let writer = NumberingWriter::new();
        let xml = writer.write(&tree).unwrap();

        assert!(xml.contains("w:numbering"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Test & <Value>"), "Test &amp; &lt;Value&gt;");
    }
}
