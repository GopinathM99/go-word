//! Styles.xml parser
//!
//! Parses style definitions and maps them to doc_model styles.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use doc_model::{
    Alignment, CharacterProperties, LineSpacing, ParagraphProperties, Style, StyleId, StyleType,
};
use quick_xml::events::Event;

/// Parser for styles.xml
pub struct StylesParser;

impl StylesParser {
    /// Create a new styles parser
    pub fn new() -> Self {
        Self
    }

    /// Parse styles.xml and return a vector of Style objects
    pub fn parse(&self, content: &str) -> DocxResult<Vec<Style>> {
        let mut styles = Vec::new();
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        // Parse state
        let mut current_style: Option<ParsedStyle> = None;
        let mut in_style = false;
        let mut in_para_props = false;
        let mut in_run_props = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "style") {
                        // Start of a style definition
                        let style_type = XmlParser::get_w_attribute(e, "type")
                            .unwrap_or_else(|| "paragraph".to_string());
                        let style_id = XmlParser::get_w_attribute(e, "styleId")
                            .unwrap_or_else(|| "Unknown".to_string());
                        let is_default = XmlParser::get_w_attribute(e, "default")
                            .map(|v| XmlParser::parse_bool(&v))
                            .unwrap_or(false);

                        current_style = Some(ParsedStyle::new(&style_id, &style_type, is_default));
                        in_style = true;
                    } else if in_style && XmlParser::matches_element(name_ref, "pPr") {
                        in_para_props = true;
                    } else if in_style && XmlParser::matches_element(name_ref, "rPr") {
                        in_run_props = true;
                    } else if in_style {
                        self.parse_style_element(e, current_style.as_mut().unwrap())?;
                    } else if in_para_props && current_style.is_some() {
                        self.parse_para_property(e, current_style.as_mut().unwrap())?;
                    } else if in_run_props && current_style.is_some() {
                        self.parse_run_property(e, current_style.as_mut().unwrap())?;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if in_style && !in_para_props && !in_run_props {
                        self.parse_style_element(e, current_style.as_mut().unwrap())?;
                    } else if in_para_props && current_style.is_some() {
                        self.parse_para_property(e, current_style.as_mut().unwrap())?;
                    } else if in_run_props && current_style.is_some() {
                        self.parse_run_property(e, current_style.as_mut().unwrap())?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "style") {
                        // End of style - convert and add to results
                        if let Some(parsed) = current_style.take() {
                            if let Some(style) = parsed.to_style() {
                                styles.push(style);
                            }
                        }
                        in_style = false;
                    } else if XmlParser::matches_element(name_ref, "pPr") {
                        in_para_props = false;
                    } else if XmlParser::matches_element(name_ref, "rPr") {
                        in_run_props = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(styles)
    }

    /// Parse a style element (name, basedOn, next, etc.)
    fn parse_style_element(&self, e: &quick_xml::events::BytesStart, style: &mut ParsedStyle) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "name") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.name = Some(val);
            }
        } else if XmlParser::matches_element(name_ref, "basedOn") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.based_on = Some(val);
            }
        } else if XmlParser::matches_element(name_ref, "next") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.next_style = Some(val);
            }
        } else if XmlParser::matches_element(name_ref, "qFormat") {
            style.quick_format = true;
        } else if XmlParser::matches_element(name_ref, "hidden") {
            style.hidden = true;
        } else if XmlParser::matches_element(name_ref, "uiPriority") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.priority = val.parse().unwrap_or(99);
            }
        }

        Ok(())
    }

    /// Parse a paragraph property element
    fn parse_para_property(&self, e: &quick_xml::events::BytesStart, style: &mut ParsedStyle) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "jc") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.para_props.alignment = Some(parse_alignment(&val));
            }
        } else if XmlParser::matches_element(name_ref, "ind") {
            if let Some(val) = XmlParser::get_w_attribute(e, "left") {
                style.para_props.indent_left = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "right") {
                style.para_props.indent_right = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "firstLine") {
                style.para_props.indent_first_line = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "hanging") {
                if let Some(twips) = XmlParser::parse_twips(&val) {
                    style.para_props.indent_first_line = Some(-twips);
                }
            }
        } else if XmlParser::matches_element(name_ref, "spacing") {
            if let Some(val) = XmlParser::get_w_attribute(e, "before") {
                style.para_props.space_before = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "after") {
                style.para_props.space_after = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "line") {
                let line_rule = XmlParser::get_w_attribute(e, "lineRule")
                    .unwrap_or_else(|| "auto".to_string());
                style.para_props.line_spacing = Some(parse_line_spacing(&val, &line_rule));
            }
        } else if XmlParser::matches_element(name_ref, "keepNext") {
            style.para_props.keep_with_next = Some(true);
        } else if XmlParser::matches_element(name_ref, "keepLines") {
            style.para_props.keep_together = Some(true);
        } else if XmlParser::matches_element(name_ref, "pageBreakBefore") {
            style.para_props.page_break_before = Some(true);
        } else if XmlParser::matches_element(name_ref, "outlineLvl") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.para_props.outline_level = val.parse().ok();
            }
        }

        Ok(())
    }

    /// Parse a run property element
    fn parse_run_property(&self, e: &quick_xml::events::BytesStart, style: &mut ParsedStyle) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "b") {
            let val = XmlParser::get_w_attribute(e, "val");
            style.char_props.bold = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "i") {
            let val = XmlParser::get_w_attribute(e, "val");
            style.char_props.italic = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "u") {
            let val = XmlParser::get_w_attribute(e, "val");
            style.char_props.underline = Some(val.map(|v| v != "none").unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "strike") {
            let val = XmlParser::get_w_attribute(e, "val");
            style.char_props.strikethrough = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "sz") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.char_props.font_size = XmlParser::parse_half_points(&val);
            }
        } else if XmlParser::matches_element(name_ref, "rFonts") {
            if let Some(font) = XmlParser::get_w_attribute(e, "ascii")
                .or_else(|| XmlParser::get_w_attribute(e, "hAnsi"))
                .or_else(|| XmlParser::get_w_attribute(e, "cs"))
            {
                style.char_props.font_family = Some(font);
            }
        } else if XmlParser::matches_element(name_ref, "color") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                if val != "auto" {
                    style.char_props.color = Some(format!("#{}", val));
                }
            }
        } else if XmlParser::matches_element(name_ref, "caps") {
            let val = XmlParser::get_w_attribute(e, "val");
            style.char_props.all_caps = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "smallCaps") {
            let val = XmlParser::get_w_attribute(e, "val");
            style.char_props.small_caps = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "spacing") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                style.char_props.spacing = XmlParser::parse_twips(&val);
            }
        }

        Ok(())
    }
}

/// Intermediate parsed style structure
#[derive(Debug)]
struct ParsedStyle {
    id: String,
    name: Option<String>,
    style_type: String,
    is_default: bool,
    based_on: Option<String>,
    next_style: Option<String>,
    quick_format: bool,
    hidden: bool,
    priority: u32,
    para_props: ParagraphProperties,
    char_props: CharacterProperties,
}

impl ParsedStyle {
    fn new(id: &str, style_type: &str, is_default: bool) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            style_type: style_type.to_string(),
            is_default,
            based_on: None,
            next_style: None,
            quick_format: false,
            hidden: false,
            priority: 99,
            para_props: ParagraphProperties::default(),
            char_props: CharacterProperties::default(),
        }
    }

    fn to_style(self) -> Option<Style> {
        let style_type = match self.style_type.as_str() {
            "paragraph" => StyleType::Paragraph,
            "character" => StyleType::Character,
            "table" => StyleType::Table,
            "numbering" => StyleType::Numbering,
            _ => return None, // Skip unknown style types
        };

        let name = self.name.unwrap_or_else(|| self.id.clone());

        let mut style = match style_type {
            StyleType::Paragraph => Style::paragraph(self.id.as_str(), &name),
            StyleType::Character => Style::character(self.id.as_str(), &name),
            _ => return None, // Skip table/numbering for now
        };

        // Apply base style
        if let Some(base) = self.based_on {
            style = style.with_based_on(base);
        }

        // Apply next style
        if let Some(next) = self.next_style {
            style = style.with_next_style(next);
        }

        // Apply priority
        style = style.with_priority(self.priority);

        // Apply properties
        style = style.with_paragraph_props(self.para_props);
        style = style.with_character_props(self.char_props);

        // Mark as built-in if it's a default style
        if self.is_default {
            style = style.as_built_in();
        }

        // Set hidden
        style.hidden = self.hidden;

        Some(style)
    }
}

/// Parse alignment value
fn parse_alignment(value: &str) -> Alignment {
    match value {
        "center" => Alignment::Center,
        "right" => Alignment::Right,
        "both" | "justify" => Alignment::Justify,
        _ => Alignment::Left,
    }
}

/// Parse line spacing value
fn parse_line_spacing(value: &str, line_rule: &str) -> LineSpacing {
    let val: f32 = value.parse().unwrap_or(240.0);

    match line_rule {
        "exact" => LineSpacing::Exact(val / 20.0),
        "atLeast" => LineSpacing::AtLeast(val / 20.0),
        _ => LineSpacing::Multiple(val / 240.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_style() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:style w:type="paragraph" w:styleId="Normal" w:default="1">
        <w:name w:val="Normal"/>
        <w:qFormat/>
        <w:pPr>
            <w:spacing w:after="160" w:line="259" w:lineRule="auto"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/>
            <w:sz w:val="22"/>
        </w:rPr>
    </w:style>
</w:styles>"#;

        let parser = StylesParser::new();
        let styles = parser.parse(xml).unwrap();

        assert_eq!(styles.len(), 1);
        let style = &styles[0];
        assert_eq!(style.id.as_str(), "Normal");
        assert_eq!(style.name, "Normal");
        assert_eq!(style.style_type, StyleType::Paragraph);
        assert_eq!(style.character_props.font_family, Some("Calibri".to_string()));
        assert_eq!(style.character_props.font_size, Some(11.0)); // 22 half-points
    }

    #[test]
    fn test_parse_heading_style() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:style w:type="paragraph" w:styleId="Heading1">
        <w:name w:val="Heading 1"/>
        <w:basedOn w:val="Normal"/>
        <w:next w:val="Normal"/>
        <w:uiPriority w:val="9"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="240" w:after="0"/>
            <w:outlineLvl w:val="0"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:ascii="Calibri Light"/>
            <w:b/>
            <w:sz w:val="32"/>
            <w:color w:val="2F5496"/>
        </w:rPr>
    </w:style>
</w:styles>"#;

        let parser = StylesParser::new();
        let styles = parser.parse(xml).unwrap();

        assert_eq!(styles.len(), 1);
        let style = &styles[0];
        assert_eq!(style.id.as_str(), "Heading1");
        assert_eq!(style.based_on, Some(StyleId::new("Normal")));
        assert_eq!(style.next_style, Some(StyleId::new("Normal")));
        assert_eq!(style.priority, 9);
        assert_eq!(style.paragraph_props.keep_with_next, Some(true));
        assert_eq!(style.paragraph_props.keep_together, Some(true));
        assert_eq!(style.paragraph_props.outline_level, Some(0));
        assert_eq!(style.character_props.bold, Some(true));
        assert_eq!(style.character_props.font_size, Some(16.0)); // 32 half-points
        assert_eq!(style.character_props.color, Some("#2F5496".to_string()));
    }

    #[test]
    fn test_parse_character_style() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:style w:type="character" w:styleId="Strong">
        <w:name w:val="Strong"/>
        <w:uiPriority w:val="22"/>
        <w:rPr>
            <w:b/>
        </w:rPr>
    </w:style>
</w:styles>"#;

        let parser = StylesParser::new();
        let styles = parser.parse(xml).unwrap();

        assert_eq!(styles.len(), 1);
        let style = &styles[0];
        assert_eq!(style.id.as_str(), "Strong");
        assert_eq!(style.style_type, StyleType::Character);
        assert_eq!(style.character_props.bold, Some(true));
    }
}
