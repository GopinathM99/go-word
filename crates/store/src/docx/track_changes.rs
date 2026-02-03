//! Track Changes Import/Export for DOCX
//!
//! Handles w:ins, w:del, w:moveFrom, w:moveTo elements and format change tracking.
//! DOCX stores tracked changes inline in the document content with metadata.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use doc_model::{CharacterProperties, NodeId, ParagraphProperties};
use quick_xml::events::Event;
use std::collections::HashMap;

// DateTime handling without chrono dependency
type DateTime = String;

// =============================================================================
// Track Change Types
// =============================================================================

/// Represents a tracked change from DOCX
#[derive(Debug, Clone)]
pub struct TrackedChange {
    /// Unique revision ID (w:id)
    pub id: i64,
    /// Author of the change (w:author)
    pub author: String,
    /// Timestamp of the change (w:date) - ISO 8601 string
    pub date: Option<String>,
    /// Type of change
    pub change_type: TrackedChangeType,
}

/// Type of tracked change
#[derive(Debug, Clone)]
pub enum TrackedChangeType {
    /// Insertion (w:ins)
    Insert {
        content: String,
    },
    /// Deletion (w:del)
    Delete {
        content: String,
    },
    /// Move from origin (w:moveFrom)
    MoveFrom {
        move_id: i64,
        content: String,
    },
    /// Move to destination (w:moveTo)
    MoveTo {
        move_id: i64,
        content: String,
    },
    /// Run property change (w:rPrChange)
    RunPropertyChange {
        old_properties: Option<CharacterProperties>,
    },
    /// Paragraph property change (w:pPrChange)
    ParagraphPropertyChange {
        old_properties: Option<ParagraphProperties>,
    },
    /// Table property change
    TablePropertyChange,
    /// Section property change
    SectionPropertyChange,
    /// Cell merge change
    CellMergeChange {
        vertical: bool,
    },
}

// =============================================================================
// Track Changes Parser
// =============================================================================

/// Parser for track changes in DOCX
pub struct TrackChangesParser {
    /// Map of revision ID to change
    changes: HashMap<i64, TrackedChange>,
    /// Map of move ID to (from, to) revision pairs
    move_pairs: HashMap<i64, (Option<i64>, Option<i64>)>,
}

impl TrackChangesParser {
    /// Create a new track changes parser
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            move_pairs: HashMap::new(),
        }
    }

    /// Parse an insertion element (w:ins)
    pub fn parse_insert(&mut self, content: &str) -> DocxResult<ParsedInsertion> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut id: Option<i64> = None;
        let mut author = String::new();
        let mut date: Option<String> = None;
        let mut inserted_text = String::new();
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "ins") {
                        id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                        author = XmlParser::get_w_attribute(e, "author")
                            .unwrap_or_default();
                        date = XmlParser::get_w_attribute(e, "date")
                            .clone();
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "ins") {
                        id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                        author = XmlParser::get_w_attribute(e, "author")
                            .unwrap_or_default();
                        date = XmlParser::get_w_attribute(e, "date")
                            .clone();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        let text = e.unescape()
                            .map_err(|e| DocxError::XmlParse(e.to_string()))?;
                        inserted_text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        let change = TrackedChange {
            id: id.unwrap_or(0),
            author,
            date,
            change_type: TrackedChangeType::Insert {
                content: inserted_text.clone(),
            },
        };

        if let Some(id) = id {
            self.changes.insert(id, change.clone());
        }

        Ok(ParsedInsertion {
            id: id.unwrap_or(0),
            author: change.author.clone(),
            date: change.date,
            content: inserted_text,
        })
    }

    /// Parse a deletion element (w:del)
    pub fn parse_delete(&mut self, content: &str) -> DocxResult<ParsedDeletion> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut id: Option<i64> = None;
        let mut author = String::new();
        let mut date: Option<String> = None;
        let mut deleted_text = String::new();
        let mut in_del_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "del") {
                        id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                        author = XmlParser::get_w_attribute(e, "author")
                            .unwrap_or_default();
                        date = XmlParser::get_w_attribute(e, "date")
                            .clone();
                    } else if XmlParser::matches_element(name_ref, "delText") {
                        in_del_text = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_del_text {
                        let text = e.unescape()
                            .map_err(|e| DocxError::XmlParse(e.to_string()))?;
                        deleted_text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "delText") {
                        in_del_text = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        let change = TrackedChange {
            id: id.unwrap_or(0),
            author,
            date,
            change_type: TrackedChangeType::Delete {
                content: deleted_text.clone(),
            },
        };

        if let Some(id) = id {
            self.changes.insert(id, change.clone());
        }

        Ok(ParsedDeletion {
            id: id.unwrap_or(0),
            author: change.author.clone(),
            date: change.date,
            content: deleted_text,
        })
    }

    /// Parse a move from element (w:moveFrom)
    pub fn parse_move_from(&mut self, content: &str) -> DocxResult<ParsedMove> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut id: Option<i64> = None;
        let mut author = String::new();
        let mut date: Option<String> = None;
        let mut move_id: Option<i64> = None;
        let mut moved_text = String::new();
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "moveFrom") {
                        id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                        author = XmlParser::get_w_attribute(e, "author")
                            .unwrap_or_default();
                        date = XmlParser::get_w_attribute(e, "date")
                            .clone();
                    } else if XmlParser::matches_element(name_ref, "moveFromRangeStart") {
                        move_id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        let text = e.unescape()
                            .map_err(|e| DocxError::XmlParse(e.to_string()))?;
                        moved_text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        // Track the move pair
        let actual_move_id = move_id.unwrap_or_else(|| id.unwrap_or(0));
        let entry = self.move_pairs.entry(actual_move_id).or_insert((None, None));
        entry.0 = id;

        Ok(ParsedMove {
            id: id.unwrap_or(0),
            move_id: actual_move_id,
            author,
            date,
            content: moved_text,
            is_from: true,
        })
    }

    /// Parse a move to element (w:moveTo)
    pub fn parse_move_to(&mut self, content: &str) -> DocxResult<ParsedMove> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut id: Option<i64> = None;
        let mut author = String::new();
        let mut date: Option<String> = None;
        let mut move_id: Option<i64> = None;
        let mut moved_text = String::new();
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "moveTo") {
                        id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                        author = XmlParser::get_w_attribute(e, "author")
                            .unwrap_or_default();
                        date = XmlParser::get_w_attribute(e, "date")
                            .clone();
                    } else if XmlParser::matches_element(name_ref, "moveToRangeStart") {
                        move_id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok());
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        let text = e.unescape()
                            .map_err(|e| DocxError::XmlParse(e.to_string()))?;
                        moved_text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        // Track the move pair
        let actual_move_id = move_id.unwrap_or_else(|| id.unwrap_or(0));
        let entry = self.move_pairs.entry(actual_move_id).or_insert((None, None));
        entry.1 = id;

        Ok(ParsedMove {
            id: id.unwrap_or(0),
            move_id: actual_move_id,
            author,
            date,
            content: moved_text,
            is_from: false,
        })
    }

    /// Parse run property change (w:rPrChange)
    pub fn parse_run_property_change(
        &mut self,
        e: &quick_xml::events::BytesStart,
    ) -> DocxResult<ParsedPropertyChange> {
        let id = XmlParser::get_w_attribute(e, "id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let author = XmlParser::get_w_attribute(e, "author").unwrap_or_default();
        let date = XmlParser::get_w_attribute(e, "date")
            .clone();

        Ok(ParsedPropertyChange {
            id,
            author,
            date,
            change_type: PropertyChangeType::RunProperty,
        })
    }

    /// Parse paragraph property change (w:pPrChange)
    pub fn parse_paragraph_property_change(
        &mut self,
        e: &quick_xml::events::BytesStart,
    ) -> DocxResult<ParsedPropertyChange> {
        let id = XmlParser::get_w_attribute(e, "id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let author = XmlParser::get_w_attribute(e, "author").unwrap_or_default();
        let date = XmlParser::get_w_attribute(e, "date")
            .clone();

        Ok(ParsedPropertyChange {
            id,
            author,
            date,
            change_type: PropertyChangeType::ParagraphProperty,
        })
    }

    /// Get all tracked changes
    pub fn get_changes(&self) -> &HashMap<i64, TrackedChange> {
        &self.changes
    }

    /// Get move pairs (from_id, to_id)
    pub fn get_move_pairs(&self) -> &HashMap<i64, (Option<i64>, Option<i64>)> {
        &self.move_pairs
    }
}

// =============================================================================
// Track Changes Writer
// =============================================================================

/// Writer for track changes in DOCX export
pub struct TrackChangesWriter {
    next_revision_id: i64,
}

impl TrackChangesWriter {
    /// Create a new track changes writer
    pub fn new() -> Self {
        Self {
            next_revision_id: 0,
        }
    }

    /// Write an insertion element
    pub fn write_insert(&mut self, xml: &mut String, author: &str, date: Option<String>, content: &str) {
        let id = self.next_revision_id;
        self.next_revision_id += 1;

        xml.push_str(&format!(
            r#"<w:ins w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(author)
        ));

        if let Some(dt) = date {
            xml.push_str(&format!(r#" w:date="{}""#, dt));
        }

        xml.push_str(">");
        xml.push_str("<w:r><w:t>");
        xml.push_str(&escape_xml(content));
        xml.push_str("</w:t></w:r>");
        xml.push_str("</w:ins>");
    }

    /// Write a deletion element
    pub fn write_delete(&mut self, xml: &mut String, author: &str, date: Option<String>, content: &str) {
        let id = self.next_revision_id;
        self.next_revision_id += 1;

        xml.push_str(&format!(
            r#"<w:del w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(author)
        ));

        if let Some(dt) = date {
            xml.push_str(&format!(r#" w:date="{}""#, dt));
        }

        xml.push_str(">");
        xml.push_str("<w:r><w:delText>");
        xml.push_str(&escape_xml(content));
        xml.push_str("</w:delText></w:r>");
        xml.push_str("</w:del>");
    }

    /// Write a move from element (start of move range)
    pub fn write_move_from_start(&mut self, xml: &mut String, move_id: i64, author: &str, date: Option<String>) {
        let id = self.next_revision_id;
        self.next_revision_id += 1;

        xml.push_str(&format!(
            r#"<w:moveFromRangeStart w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(author)
        ));

        if let Some(dt) = date {
            xml.push_str(&format!(r#" w:date="{}""#, dt));
        }

        xml.push_str(&format!(r#" w:name="move{}"/>"#, move_id));
    }

    /// Write move from end marker
    pub fn write_move_from_end(&mut self, xml: &mut String, move_id: i64) {
        xml.push_str(&format!(r#"<w:moveFromRangeEnd w:id="{}"/>"#, move_id));
    }

    /// Write a move to element (start of move range)
    pub fn write_move_to_start(&mut self, xml: &mut String, move_id: i64, author: &str, date: Option<String>) {
        let id = self.next_revision_id;
        self.next_revision_id += 1;

        xml.push_str(&format!(
            r#"<w:moveToRangeStart w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(author)
        ));

        if let Some(dt) = date {
            xml.push_str(&format!(r#" w:date="{}""#, dt));
        }

        xml.push_str(&format!(r#" w:name="move{}"/>"#, move_id));
    }

    /// Write move to end marker
    pub fn write_move_to_end(&mut self, xml: &mut String, move_id: i64) {
        xml.push_str(&format!(r#"<w:moveToRangeEnd w:id="{}"/>"#, move_id));
    }

    /// Write run property change
    pub fn write_run_property_change(
        &mut self,
        xml: &mut String,
        author: &str,
        date: Option<String>,
        old_props: &CharacterProperties,
    ) {
        let id = self.next_revision_id;
        self.next_revision_id += 1;

        xml.push_str(&format!(
            r#"<w:rPrChange w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(author)
        ));

        if let Some(dt) = date {
            xml.push_str(&format!(r#" w:date="{}""#, dt));
        }

        xml.push_str(">");
        xml.push_str("<w:rPr>");
        write_character_properties(xml, old_props);
        xml.push_str("</w:rPr>");
        xml.push_str("</w:rPrChange>");
    }

    /// Write paragraph property change
    pub fn write_paragraph_property_change(
        &mut self,
        xml: &mut String,
        author: &str,
        date: Option<String>,
        old_props: &ParagraphProperties,
    ) {
        let id = self.next_revision_id;
        self.next_revision_id += 1;

        xml.push_str(&format!(
            r#"<w:pPrChange w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(author)
        ));

        if let Some(dt) = date {
            xml.push_str(&format!(r#" w:date="{}""#, dt));
        }

        xml.push_str(">");
        xml.push_str("<w:pPr>");
        write_paragraph_properties(xml, old_props);
        xml.push_str("</w:pPr>");
        xml.push_str("</w:pPrChange>");
    }
}

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed insertion data
#[derive(Debug, Clone)]
pub struct ParsedInsertion {
    pub id: i64,
    pub author: String,
    pub date: Option<String>,
    pub content: String,
}

/// Parsed deletion data
#[derive(Debug, Clone)]
pub struct ParsedDeletion {
    pub id: i64,
    pub author: String,
    pub date: Option<String>,
    pub content: String,
}

/// Parsed move data
#[derive(Debug, Clone)]
pub struct ParsedMove {
    pub id: i64,
    pub move_id: i64,
    pub author: String,
    pub date: Option<String>,
    pub content: String,
    pub is_from: bool,
}

/// Parsed property change data
#[derive(Debug, Clone)]
pub struct ParsedPropertyChange {
    pub id: i64,
    pub author: String,
    pub date: Option<String>,
    pub change_type: PropertyChangeType,
}

/// Type of property change
#[derive(Debug, Clone)]
pub enum PropertyChangeType {
    RunProperty,
    ParagraphProperty,
    TableProperty,
    SectionProperty,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Validate and normalize ISO 8601 datetime string
/// Returns the string if it appears valid, None otherwise
fn parse_iso_datetime(s: &str) -> Option<String> {
    // Basic validation - check if it looks like ISO 8601
    if s.contains('T') && s.len() >= 19 {
        Some(s.to_string())
    } else {
        None
    }
}

/// Escape XML text content
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape XML attribute value
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Write character properties to XML
fn write_character_properties(xml: &mut String, props: &CharacterProperties) {
    if let Some(bold) = props.bold {
        if bold {
            xml.push_str("<w:b/>");
        } else {
            xml.push_str(r#"<w:b w:val="0"/>"#);
        }
    }

    if let Some(italic) = props.italic {
        if italic {
            xml.push_str("<w:i/>");
        } else {
            xml.push_str(r#"<w:i w:val="0"/>"#);
        }
    }

    if let Some(underline) = props.underline {
        if underline {
            xml.push_str(r#"<w:u w:val="single"/>"#);
        } else {
            xml.push_str(r#"<w:u w:val="none"/>"#);
        }
    }

    if let Some(size) = props.font_size {
        let half_pts = (size * 2.0) as i32;
        xml.push_str(&format!(r#"<w:sz w:val="{}"/>"#, half_pts));
    }

    if let Some(ref font) = props.font_family {
        xml.push_str(&format!(
            r#"<w:rFonts w:ascii="{}" w:hAnsi="{}"/>"#,
            escape_xml_attr(font),
            escape_xml_attr(font)
        ));
    }

    if let Some(ref color) = props.color {
        let color_val = color.trim_start_matches('#');
        xml.push_str(&format!(r#"<w:color w:val="{}"/>"#, color_val));
    }
}

/// Write paragraph properties to XML
fn write_paragraph_properties(xml: &mut String, props: &ParagraphProperties) {
    use doc_model::Alignment;

    if let Some(alignment) = props.alignment {
        let val = match alignment {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right",
            Alignment::Justify => "both",
        };
        xml.push_str(&format!(r#"<w:jc w:val="{}"/>"#, val));
    }

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
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso_datetime() {
        let dt = parse_iso_datetime("2024-01-15T10:30:00Z");
        assert!(dt.is_some());

        let dt = parse_iso_datetime("2024-01-15T10:30:00");
        assert!(dt.is_some());
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Hello & World"), "Hello &amp; World");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_escape_xml_attr() {
        assert_eq!(escape_xml_attr("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_track_changes_parser_new() {
        let parser = TrackChangesParser::new();
        assert!(parser.changes.is_empty());
    }

    #[test]
    fn test_track_changes_writer_insert() {
        let mut writer = TrackChangesWriter::new();
        let mut xml = String::new();

        writer.write_insert(&mut xml, "Test Author", None, "Hello");

        assert!(xml.contains("w:ins"));
        assert!(xml.contains("Test Author"));
        assert!(xml.contains("Hello"));
    }

    #[test]
    fn test_track_changes_writer_delete() {
        let mut writer = TrackChangesWriter::new();
        let mut xml = String::new();

        writer.write_delete(&mut xml, "Test Author", None, "Deleted");

        assert!(xml.contains("w:del"));
        assert!(xml.contains("w:delText"));
        assert!(xml.contains("Deleted"));
    }
}
