//! Footnotes and Endnotes Import/Export for DOCX
//!
//! Handles footnotes.xml, endnotes.xml, and their references in the document.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use quick_xml::events::Event;
use std::collections::HashMap;

// =============================================================================
// Note Types
// =============================================================================

/// Type of note
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Footnote,
    Endnote,
}

/// Internal ID type for notes
pub type NoteId = u64;

// =============================================================================
// Footnotes/Endnotes Parser
// =============================================================================

/// Parser for footnotes and endnotes in DOCX
pub struct NotesParser {
    /// Map of DOCX note ID (integer) to our internal NoteId
    footnote_id_map: HashMap<i64, NoteId>,
    endnote_id_map: HashMap<i64, NoteId>,
    /// Next internal ID
    next_id: NoteId,
}

impl NotesParser {
    /// Create a new notes parser
    pub fn new() -> Self {
        Self {
            footnote_id_map: HashMap::new(),
            endnote_id_map: HashMap::new(),
            next_id: 1,
        }
    }

    /// Parse footnotes.xml file
    pub fn parse_footnotes_xml(&mut self, content: &str) -> DocxResult<Vec<ParsedNote>> {
        self.parse_notes_xml(content, NoteType::Footnote)
    }

    /// Parse endnotes.xml file
    pub fn parse_endnotes_xml(&mut self, content: &str) -> DocxResult<Vec<ParsedNote>> {
        self.parse_notes_xml(content, NoteType::Endnote)
    }

    /// Generic notes XML parser
    fn parse_notes_xml(&mut self, content: &str, note_type: NoteType) -> DocxResult<Vec<ParsedNote>> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut notes = Vec::new();
        let mut current_note: Option<ParsedNote> = None;
        let mut in_note = false;
        let mut in_para = false;
        let mut in_run = false;
        let mut in_text = false;

        let note_element = match note_type {
            NoteType::Footnote => "footnote",
            NoteType::Endnote => "endnote",
        };

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, note_element) {
                        let id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);

                        let note_type_attr = XmlParser::get_w_attribute(e, "type");
                        let is_separator = note_type_attr.as_deref() == Some("separator")
                            || note_type_attr.as_deref() == Some("continuationSeparator");

                        current_note = Some(ParsedNote {
                            id,
                            note_type,
                            content: String::new(),
                            is_separator,
                            custom_mark: None,
                        });
                        in_note = true;
                    } else if in_note && XmlParser::matches_element(name_ref, "p") {
                        in_para = true;
                    } else if in_para && XmlParser::matches_element(name_ref, "r") {
                        in_run = true;
                    } else if in_run && XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        if let Some(ref mut note) = current_note {
                            let text = e.unescape()
                                .map_err(|e| DocxError::XmlParse(e.to_string()))?;
                            note.content.push_str(&text);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, note_element) {
                        if let Some(note) = current_note.take() {
                            // Skip separator notes
                            if !note.is_separator {
                                // Store ID mapping
                                match note_type {
                                    NoteType::Footnote => {
                                        self.footnote_id_map.insert(note.id, self.next_id);
                                    }
                                    NoteType::Endnote => {
                                        self.endnote_id_map.insert(note.id, self.next_id);
                                    }
                                }
                                self.next_id += 1;
                                notes.push(note);
                            }
                        }
                        in_note = false;
                    } else if XmlParser::matches_element(name_ref, "p") {
                        in_para = false;
                    } else if XmlParser::matches_element(name_ref, "r") {
                        in_run = false;
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(notes)
    }

    /// Get internal NoteId for a DOCX footnote ID
    pub fn get_footnote_id(&self, docx_id: i64) -> Option<NoteId> {
        self.footnote_id_map.get(&docx_id).copied()
    }

    /// Get internal NoteId for a DOCX endnote ID
    pub fn get_endnote_id(&self, docx_id: i64) -> Option<NoteId> {
        self.endnote_id_map.get(&docx_id).copied()
    }
}

// =============================================================================
// Notes Writer
// =============================================================================

/// Writer for footnotes and endnotes in DOCX export
pub struct NotesWriter {
    next_footnote_id: i64,
    next_endnote_id: i64,
    /// Map of internal NoteId to DOCX integer ID
    footnote_id_map: HashMap<NoteId, i64>,
    endnote_id_map: HashMap<NoteId, i64>,
}

impl NotesWriter {
    /// Create a new notes writer
    pub fn new() -> Self {
        Self {
            // Start from 1 (0 is reserved for separator, -1 for continuation separator)
            next_footnote_id: 1,
            next_endnote_id: 1,
            footnote_id_map: HashMap::new(),
            endnote_id_map: HashMap::new(),
        }
    }

    /// Generate footnotes.xml content
    pub fn write_footnotes_xml(&mut self, notes: &[ParsedNote]) -> DocxResult<String> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');
        xml.push_str(r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#);

        // Write separator (required)
        xml.push_str(r#"<w:footnote w:type="separator" w:id="-1">"#);
        xml.push_str("<w:p><w:r><w:separator/></w:r></w:p>");
        xml.push_str("</w:footnote>");

        // Write continuation separator
        xml.push_str(r#"<w:footnote w:type="continuationSeparator" w:id="0">"#);
        xml.push_str("<w:p><w:r><w:continuationSeparator/></w:r></w:p>");
        xml.push_str("</w:footnote>");

        // Write actual footnotes
        for note in notes.iter().filter(|n| n.note_type == NoteType::Footnote) {
            let docx_id = self.next_footnote_id;
            self.next_footnote_id += 1;

            xml.push_str(&format!(r#"<w:footnote w:id="{}">"#, docx_id));
            xml.push_str("<w:p><w:pPr><w:pStyle w:val=\"FootnoteText\"/></w:pPr>");
            xml.push_str("<w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr>");
            xml.push_str("<w:footnoteRef/></w:r>");
            xml.push_str("<w:r><w:t xml:space=\"preserve\"> </w:t></w:r>");
            xml.push_str("<w:r><w:t>");
            xml.push_str(&escape_xml(&note.content));
            xml.push_str("</w:t></w:r>");
            xml.push_str("</w:p></w:footnote>");
        }

        xml.push_str("</w:footnotes>");

        Ok(xml)
    }

    /// Generate endnotes.xml content
    pub fn write_endnotes_xml(&mut self, notes: &[ParsedNote]) -> DocxResult<String> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');
        xml.push_str(r#"<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#);

        // Write separator (required)
        xml.push_str(r#"<w:endnote w:type="separator" w:id="-1">"#);
        xml.push_str("<w:p><w:r><w:separator/></w:r></w:p>");
        xml.push_str("</w:endnote>");

        // Write continuation separator
        xml.push_str(r#"<w:endnote w:type="continuationSeparator" w:id="0">"#);
        xml.push_str("<w:p><w:r><w:continuationSeparator/></w:r></w:p>");
        xml.push_str("</w:endnote>");

        // Write actual endnotes
        for note in notes.iter().filter(|n| n.note_type == NoteType::Endnote) {
            let docx_id = self.next_endnote_id;
            self.next_endnote_id += 1;

            xml.push_str(&format!(r#"<w:endnote w:id="{}">"#, docx_id));
            xml.push_str("<w:p><w:pPr><w:pStyle w:val=\"EndnoteText\"/></w:pPr>");
            xml.push_str("<w:r><w:rPr><w:rStyle w:val=\"EndnoteReference\"/></w:rPr>");
            xml.push_str("<w:endnoteRef/></w:r>");
            xml.push_str("<w:r><w:t xml:space=\"preserve\"> </w:t></w:r>");
            xml.push_str("<w:r><w:t>");
            xml.push_str(&escape_xml(&note.content));
            xml.push_str("</w:t></w:r>");
            xml.push_str("</w:p></w:endnote>");
        }

        xml.push_str("</w:endnotes>");

        Ok(xml)
    }

    /// Write footnote reference in document
    pub fn write_footnote_reference(xml: &mut String, docx_id: i64) {
        xml.push_str("<w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr>");
        xml.push_str(&format!(r#"<w:footnoteReference w:id="{}"/>"#, docx_id));
        xml.push_str("</w:r>");
    }

    /// Write endnote reference in document
    pub fn write_endnote_reference(xml: &mut String, docx_id: i64) {
        xml.push_str("<w:r><w:rPr><w:rStyle w:val=\"EndnoteReference\"/></w:rPr>");
        xml.push_str(&format!(r#"<w:endnoteReference w:id="{}"/>"#, docx_id));
        xml.push_str("</w:r>");
    }
}

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed note data from DOCX
#[derive(Debug, Clone)]
pub struct ParsedNote {
    /// DOCX note ID (integer)
    pub id: i64,
    /// Note type
    pub note_type: NoteType,
    /// Note content text
    pub content: String,
    /// Whether this is a separator note
    pub is_separator: bool,
    /// Custom mark (if not auto-numbered)
    pub custom_mark: Option<String>,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Escape XML text content
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
    fn test_notes_parser_new() {
        let parser = NotesParser::new();
        assert!(parser.footnote_id_map.is_empty());
    }

    #[test]
    fn test_parse_footnotes_xml() {
        let mut parser = NotesParser::new();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:footnote w:type="separator" w:id="-1">
                    <w:p><w:r><w:separator/></w:r></w:p>
                </w:footnote>
                <w:footnote w:id="1">
                    <w:p><w:r><w:t>This is a footnote</w:t></w:r></w:p>
                </w:footnote>
            </w:footnotes>"#;

        let notes = parser.parse_footnotes_xml(xml).unwrap();
        // Should only include the actual footnote, not the separator
        assert_eq!(notes.len(), 1);
        assert!(notes[0].content.contains("This is a footnote"));
    }

    #[test]
    fn test_notes_writer_new() {
        let writer = NotesWriter::new();
        assert_eq!(writer.next_footnote_id, 1);
        assert_eq!(writer.next_endnote_id, 1);
    }
}
