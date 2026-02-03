//! Numbering.xml parser for list definitions
//!
//! Handles abstract numbering definitions and numbering instances.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use doc_model::{
    AbstractNum, AbstractNumId, ListLevel, ListLevelAlignment, ListLevelSuffix, MultiLevelType,
    NumberingInstance, NumId, list::NumberFormat,
};
use quick_xml::events::Event;

/// Parser for numbering.xml
pub struct NumberingParser;

impl NumberingParser {
    /// Create a new numbering parser
    pub fn new() -> Self {
        Self
    }

    /// Parse numbering.xml and return abstract numbering definitions and instances
    pub fn parse(&self, content: &str) -> DocxResult<(Vec<AbstractNum>, Vec<NumberingInstance>)> {
        let mut abstract_nums = Vec::new();
        let mut instances = Vec::new();

        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut current_abstract: Option<ParsedAbstractNum> = None;
        let mut current_level: Option<ParsedLevel> = None;
        let mut in_lvl = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "abstractNum") {
                        if let Some(id) = XmlParser::get_w_attribute(e, "abstractNumId") {
                            let abstract_id: u32 = id.parse().unwrap_or(0);
                            current_abstract = Some(ParsedAbstractNum::new(abstract_id));
                        }
                    } else if XmlParser::matches_element(name_ref, "lvl") {
                        if let Some(ilvl) = XmlParser::get_w_attribute(e, "ilvl") {
                            let level: u8 = ilvl.parse().unwrap_or(0);
                            current_level = Some(ParsedLevel::new(level));
                            in_lvl = true;
                        }
                    } else if XmlParser::matches_element(name_ref, "num") {
                        if let Some(num_id) = XmlParser::get_w_attribute(e, "numId") {
                            let id: u32 = num_id.parse().unwrap_or(0);
                            // Parse numId and abstractNumId association
                            // We'll need to look for the abstractNumId child
                            // For now, create a placeholder
                        }
                    } else if in_lvl && current_level.is_some() {
                        self.parse_level_property(e, current_level.as_mut().unwrap())?;
                    } else if current_abstract.is_some() {
                        self.parse_abstract_property(e, current_abstract.as_mut().unwrap())?;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "abstractNumId") {
                        // This is inside a <num> element - we need to track the association
                        // For simplicity, we'll handle this separately
                    } else if in_lvl && current_level.is_some() {
                        self.parse_level_property(e, current_level.as_mut().unwrap())?;
                    } else if current_abstract.is_some() {
                        self.parse_abstract_property(e, current_abstract.as_mut().unwrap())?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "abstractNum") {
                        if let Some(parsed) = current_abstract.take() {
                            abstract_nums.push(parsed.to_abstract_num());
                        }
                    } else if XmlParser::matches_element(name_ref, "lvl") {
                        if let Some(parsed) = current_level.take() {
                            if let Some(ref mut abs) = current_abstract {
                                abs.levels.push(parsed);
                            }
                        }
                        in_lvl = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        // Second pass to parse num -> abstractNum associations
        let mut reader = XmlParser::from_string(content);
        let mut current_num_id: Option<u32> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "num") {
                        if let Some(num_id) = XmlParser::get_w_attribute(e, "numId") {
                            current_num_id = num_id.parse().ok();
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "abstractNumId") {
                        if let (Some(num_id), Some(abs_id)) = (
                            current_num_id,
                            XmlParser::get_w_attribute(e, "val").and_then(|v| v.parse::<u32>().ok())
                        ) {
                            instances.push(NumberingInstance::new(
                                NumId::new(num_id),
                                AbstractNumId::new(abs_id),
                            ));
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "num") {
                        current_num_id = None;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok((abstract_nums, instances))
    }

    /// Parse abstract numbering properties
    fn parse_abstract_property(&self, e: &quick_xml::events::BytesStart, abs: &mut ParsedAbstractNum) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "multiLevelType") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                abs.multi_level_type = parse_multi_level_type(&val);
            }
        } else if XmlParser::matches_element(name_ref, "name") {
            abs.name = XmlParser::get_w_attribute(e, "val");
        }

        Ok(())
    }

    /// Parse level properties
    fn parse_level_property(&self, e: &quick_xml::events::BytesStart, level: &mut ParsedLevel) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "start") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                level.start = val.parse().unwrap_or(1);
            }
        } else if XmlParser::matches_element(name_ref, "numFmt") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                level.format = parse_number_format(&val);
            }
        } else if XmlParser::matches_element(name_ref, "lvlText") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                level.text = val;
            }
        } else if XmlParser::matches_element(name_ref, "lvlJc") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                level.alignment = parse_level_alignment(&val);
            }
        } else if XmlParser::matches_element(name_ref, "suff") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                level.suffix = parse_level_suffix(&val);
            }
        } else if XmlParser::matches_element(name_ref, "ind") {
            if let Some(left) = XmlParser::get_w_attribute(e, "left") {
                level.indent = XmlParser::parse_twips(&left);
            }
            if let Some(hanging) = XmlParser::get_w_attribute(e, "hanging") {
                level.hanging = XmlParser::parse_twips(&hanging);
            }
        } else if XmlParser::matches_element(name_ref, "rFonts") {
            level.font = XmlParser::get_w_attribute(e, "ascii")
                .or_else(|| XmlParser::get_w_attribute(e, "hAnsi"));
        }

        Ok(())
    }
}

/// Parsed abstract numbering definition
#[derive(Debug)]
struct ParsedAbstractNum {
    id: u32,
    name: Option<String>,
    multi_level_type: MultiLevelType,
    levels: Vec<ParsedLevel>,
}

impl ParsedAbstractNum {
    fn new(id: u32) -> Self {
        Self {
            id,
            name: None,
            multi_level_type: MultiLevelType::default(),
            levels: Vec::new(),
        }
    }

    fn to_abstract_num(self) -> AbstractNum {
        let mut abs = AbstractNum::new(AbstractNumId::new(self.id));
        abs.name = self.name;
        abs.multi_level_type = self.multi_level_type;
        abs.levels = self.levels.into_iter().map(|l| l.to_list_level()).collect();

        // Ensure we have at least one level
        if abs.levels.is_empty() {
            abs.levels.push(ListLevel::default());
        }

        abs
    }
}

/// Parsed level definition
#[derive(Debug)]
struct ParsedLevel {
    level: u8,
    start: u32,
    format: NumberFormat,
    text: String,
    alignment: ListLevelAlignment,
    suffix: ListLevelSuffix,
    indent: Option<f32>,
    hanging: Option<f32>,
    font: Option<String>,
}

impl ParsedLevel {
    fn new(level: u8) -> Self {
        Self {
            level,
            start: 1,
            format: NumberFormat::Decimal,
            text: format!("%{}.", level + 1),
            alignment: ListLevelAlignment::Left,
            suffix: ListLevelSuffix::Tab,
            indent: None,
            hanging: None,
            font: None,
        }
    }

    fn to_list_level(self) -> ListLevel {
        let indent = self.indent.unwrap_or(36.0 * (self.level as f32 + 1.0));
        let hanging = self.hanging.unwrap_or(18.0);

        // Extract bullet character from text if format is bullet
        let bullet_char = if self.format == NumberFormat::Bullet {
            self.text.chars().next()
        } else {
            None
        };

        ListLevel {
            level: self.level,
            format: self.format,
            text: self.text,
            start: self.start,
            indent,
            hanging,
            font: self.font,
            bullet_char,
            tab_stop: None,
            restart_after_level: None,
            alignment: self.alignment,
            suffix: self.suffix,
        }
    }
}

/// Parse number format value
fn parse_number_format(value: &str) -> NumberFormat {
    match value {
        "decimal" => NumberFormat::Decimal,
        "decimalZero" => NumberFormat::DecimalZero,
        "lowerLetter" => NumberFormat::LowerLetter,
        "upperLetter" => NumberFormat::UpperLetter,
        "lowerRoman" => NumberFormat::LowerRoman,
        "upperRoman" => NumberFormat::UpperRoman,
        "bullet" => NumberFormat::Bullet,
        "none" => NumberFormat::None,
        "ordinal" => NumberFormat::Ordinal,
        "cardinalText" => NumberFormat::CardinalText,
        "ordinalText" => NumberFormat::OrdinalText,
        _ => NumberFormat::Decimal,
    }
}

/// Parse multi-level type
fn parse_multi_level_type(value: &str) -> MultiLevelType {
    match value {
        "singleLevel" => MultiLevelType::SingleLevel,
        "multilevel" => MultiLevelType::MultiLevel,
        "hybridMultilevel" => MultiLevelType::HybridMultiLevel,
        _ => MultiLevelType::SingleLevel,
    }
}

/// Parse level alignment
fn parse_level_alignment(value: &str) -> ListLevelAlignment {
    match value {
        "center" => ListLevelAlignment::Center,
        "right" => ListLevelAlignment::Right,
        _ => ListLevelAlignment::Left,
    }
}

/// Parse level suffix
fn parse_level_suffix(value: &str) -> ListLevelSuffix {
    match value {
        "space" => ListLevelSuffix::Space,
        "nothing" => ListLevelSuffix::Nothing,
        _ => ListLevelSuffix::Tab,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_format() {
        assert_eq!(parse_number_format("decimal"), NumberFormat::Decimal);
        assert_eq!(parse_number_format("bullet"), NumberFormat::Bullet);
        assert_eq!(parse_number_format("lowerRoman"), NumberFormat::LowerRoman);
        assert_eq!(parse_number_format("upperLetter"), NumberFormat::UpperLetter);
        assert_eq!(parse_number_format("unknown"), NumberFormat::Decimal);
    }

    #[test]
    fn test_parse_multi_level_type() {
        assert_eq!(parse_multi_level_type("singleLevel"), MultiLevelType::SingleLevel);
        assert_eq!(parse_multi_level_type("multilevel"), MultiLevelType::MultiLevel);
        assert_eq!(parse_multi_level_type("hybridMultilevel"), MultiLevelType::HybridMultiLevel);
    }

    #[test]
    fn test_parse_simple_numbering() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:abstractNum w:abstractNumId="0">
        <w:multiLevelType w:val="hybridMultilevel"/>
        <w:lvl w:ilvl="0">
            <w:start w:val="1"/>
            <w:numFmt w:val="decimal"/>
            <w:lvlText w:val="%1."/>
            <w:lvlJc w:val="left"/>
            <w:pPr>
                <w:ind w:left="720" w:hanging="360"/>
            </w:pPr>
        </w:lvl>
    </w:abstractNum>
    <w:num w:numId="1">
        <w:abstractNumId w:val="0"/>
    </w:num>
</w:numbering>"#;

        let parser = NumberingParser::new();
        let (abstract_nums, instances) = parser.parse(xml).unwrap();

        assert_eq!(abstract_nums.len(), 1);
        assert_eq!(instances.len(), 1);

        let abs = &abstract_nums[0];
        assert_eq!(abs.id.0, 0);
        assert_eq!(abs.multi_level_type, MultiLevelType::HybridMultiLevel);

        let level = abs.get_level(0).unwrap();
        assert_eq!(level.format, NumberFormat::Decimal);
        assert_eq!(level.start, 1);
        assert_eq!(level.text, "%1.");

        let inst = &instances[0];
        assert_eq!(inst.id.0, 1);
        assert_eq!(inst.abstract_num_id.0, 0);
    }

    #[test]
    fn test_parse_bullet_numbering() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:abstractNum w:abstractNumId="1">
        <w:multiLevelType w:val="multilevel"/>
        <w:lvl w:ilvl="0">
            <w:numFmt w:val="bullet"/>
            <w:lvlText w:val=""/>
            <w:rPr>
                <w:rFonts w:ascii="Symbol" w:hAnsi="Symbol"/>
            </w:rPr>
        </w:lvl>
    </w:abstractNum>
</w:numbering>"#;

        let parser = NumberingParser::new();
        let (abstract_nums, _) = parser.parse(xml).unwrap();

        assert_eq!(abstract_nums.len(), 1);
        let level = abstract_nums[0].get_level(0).unwrap();
        assert_eq!(level.format, NumberFormat::Bullet);
        assert_eq!(level.font, Some("Symbol".to_string()));
    }
}
