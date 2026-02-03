//! Comments Import/Export for DOCX
//!
//! Handles w:commentRangeStart, w:commentRangeEnd, w:commentReference elements
//! and the word/comments.xml file with threading support.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use doc_model::NodeId;
use quick_xml::events::Event;
use std::collections::HashMap;

// Internal ID type for comments
pub type CommentId = u64;

// =============================================================================
// Comments Parser
// =============================================================================

/// Parser for comments in DOCX
pub struct CommentsParser {
    /// Map of DOCX comment ID (integer) to our internal CommentId
    id_map: HashMap<i64, CommentId>,
    /// Comments waiting for anchor positions
    pending_comments: HashMap<i64, ParsedComment>,
    /// Comment range starts (comment_id -> node position)
    range_starts: HashMap<i64, (NodeId, usize)>,
    /// Comment range ends (comment_id -> node position)
    range_ends: HashMap<i64, (NodeId, usize)>,
    /// Next internal ID
    next_id: CommentId,
}

impl CommentsParser {
    /// Create a new comments parser
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            pending_comments: HashMap::new(),
            range_starts: HashMap::new(),
            range_ends: HashMap::new(),
            next_id: 1,
        }
    }

    /// Parse the comments.xml file
    pub fn parse_comments_xml(&mut self, content: &str) -> DocxResult<Vec<ParsedComment>> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut comments = Vec::new();
        let mut current_comment: Option<ParsedComment> = None;
        let mut in_comment = false;
        let mut in_para = false;
        let mut in_run = false;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "comment") {
                        let id = XmlParser::get_w_attribute(e, "id")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        let author = XmlParser::get_w_attribute(e, "author")
                            .unwrap_or_default();
                        let date = XmlParser::get_w_attribute(e, "date");
                        let initials = XmlParser::get_w_attribute(e, "initials");

                        current_comment = Some(ParsedComment {
                            id,
                            author,
                            date,
                            initials,
                            content: String::new(),
                            parent_id: None,
                            done: XmlParser::get_w_attribute(e, "done")
                                .map(|s| s == "1" || s.to_lowercase() == "true")
                                .unwrap_or(false),
                        });
                        in_comment = true;
                    } else if in_comment && XmlParser::matches_element(name_ref, "p") {
                        in_para = true;
                    } else if in_para && XmlParser::matches_element(name_ref, "r") {
                        in_run = true;
                    } else if in_run && XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "annotationRef") && in_comment {
                        if let Some(ref mut comment) = current_comment {
                            let parent_id = XmlParser::get_w_attribute(e, "id")
                                .and_then(|s| s.parse().ok());
                            comment.parent_id = parent_id;
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        if let Some(ref mut comment) = current_comment {
                            let text = e.unescape()
                                .map_err(|e| DocxError::XmlParse(e.to_string()))?;
                            if !comment.content.is_empty() {
                                comment.content.push(' ');
                            }
                            comment.content.push_str(&text);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "comment") {
                        if let Some(comment) = current_comment.take() {
                            self.pending_comments.insert(comment.id, comment.clone());
                            comments.push(comment);
                        }
                        in_comment = false;
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

        Ok(comments)
    }

    /// Record a comment range start
    pub fn record_range_start(&mut self, comment_id: i64, node_id: NodeId, offset: usize) {
        self.range_starts.insert(comment_id, (node_id, offset));
    }

    /// Record a comment range end
    pub fn record_range_end(&mut self, comment_id: i64, node_id: NodeId, offset: usize) {
        self.range_ends.insert(comment_id, (node_id, offset));
    }

    /// Get pending comments
    pub fn get_pending_comments(&self) -> &HashMap<i64, ParsedComment> {
        &self.pending_comments
    }

    /// Get range starts
    pub fn get_range_starts(&self) -> &HashMap<i64, (NodeId, usize)> {
        &self.range_starts
    }

    /// Get range ends
    pub fn get_range_ends(&self) -> &HashMap<i64, (NodeId, usize)> {
        &self.range_ends
    }

    /// Generate next internal ID
    pub fn next_internal_id(&mut self) -> CommentId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

// =============================================================================
// Comments Writer
// =============================================================================

/// Writer for comments in DOCX export
pub struct CommentsWriter {
    next_comment_id: i64,
    /// Map of internal CommentId to DOCX integer ID
    id_map: HashMap<CommentId, i64>,
}

impl CommentsWriter {
    /// Create a new comments writer
    pub fn new() -> Self {
        Self {
            next_comment_id: 0,
            id_map: HashMap::new(),
        }
    }

    /// Generate comments.xml content from parsed comments
    pub fn write_comments_xml(&mut self, comments: &[ParsedComment]) -> DocxResult<String> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');
        xml.push_str(r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#);
        xml.push_str(r#"xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml">"#);

        for comment in comments {
            self.write_comment(&mut xml, comment)?;
        }

        xml.push_str("</w:comments>");

        Ok(xml)
    }

    /// Write a single comment element
    fn write_comment(&mut self, xml: &mut String, comment: &ParsedComment) -> DocxResult<()> {
        let id = self.next_comment_id;
        self.next_comment_id += 1;

        xml.push_str(&format!(
            r#"<w:comment w:id="{}" w:author="{}""#,
            id,
            escape_xml_attr(&comment.author)
        ));

        if let Some(ref date) = comment.date {
            xml.push_str(&format!(r#" w:date="{}""#, date));
        }

        if comment.done {
            xml.push_str(r#" w:done="1""#);
        }

        xml.push_str(">");

        // Write comment content as paragraph
        xml.push_str("<w:p><w:pPr><w:pStyle w:val=\"CommentText\"/></w:pPr>");
        xml.push_str("<w:r><w:rPr><w:rStyle w:val=\"CommentReference\"/></w:rPr>");
        xml.push_str("<w:annotationRef/></w:r>");
        xml.push_str("<w:r><w:t>");
        xml.push_str(&escape_xml(&comment.content));
        xml.push_str("</w:t></w:r></w:p>");

        xml.push_str("</w:comment>");

        Ok(())
    }

    /// Write comment range start marker in document
    pub fn write_comment_range_start(xml: &mut String, comment_id: i64) {
        xml.push_str(&format!(r#"<w:commentRangeStart w:id="{}"/>"#, comment_id));
    }

    /// Write comment range end marker in document
    pub fn write_comment_range_end(xml: &mut String, comment_id: i64) {
        xml.push_str(&format!(r#"<w:commentRangeEnd w:id="{}"/>"#, comment_id));
    }

    /// Write comment reference marker in document (within a run)
    pub fn write_comment_reference(xml: &mut String, comment_id: i64) {
        xml.push_str("<w:r><w:rPr><w:rStyle w:val=\"CommentReference\"/></w:rPr>");
        xml.push_str(&format!(r#"<w:commentReference w:id="{}"/>"#, comment_id));
        xml.push_str("</w:r>");
    }
}

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed comment data from DOCX
#[derive(Debug, Clone)]
pub struct ParsedComment {
    /// DOCX comment ID (integer)
    pub id: i64,
    /// Comment author
    pub author: String,
    /// Comment date (ISO 8601 string)
    pub date: Option<String>,
    /// Author initials
    pub initials: Option<String>,
    /// Comment content text
    pub content: String,
    /// Parent comment ID (for replies)
    pub parent_id: Option<i64>,
    /// Whether the comment is marked as done/resolved
    pub done: bool,
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

/// Escape XML attribute value
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comments_parser_new() {
        let parser = CommentsParser::new();
        assert!(parser.pending_comments.is_empty());
    }

    #[test]
    fn test_parse_simple_comments_xml() {
        let mut parser = CommentsParser::new();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:comment w:id="0" w:author="Test Author" w:date="2024-01-15T10:30:00Z">
                    <w:p>
                        <w:r><w:t>This is a comment</w:t></w:r>
                    </w:p>
                </w:comment>
            </w:comments>"#;

        let comments = parser.parse_comments_xml(xml).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].author, "Test Author");
        assert!(comments[0].content.contains("This is a comment"));
    }

    #[test]
    fn test_comments_writer_new() {
        let writer = CommentsWriter::new();
        assert_eq!(writer.next_comment_id, 0);
    }

    #[test]
    fn test_write_comment_range_markers() {
        let mut xml = String::new();

        CommentsWriter::write_comment_range_start(&mut xml, 0);
        assert!(xml.contains("commentRangeStart"));
        assert!(xml.contains("w:id=\"0\""));

        xml.clear();
        CommentsWriter::write_comment_range_end(&mut xml, 0);
        assert!(xml.contains("commentRangeEnd"));

        xml.clear();
        CommentsWriter::write_comment_reference(&mut xml, 0);
        assert!(xml.contains("commentReference"));
    }
}
