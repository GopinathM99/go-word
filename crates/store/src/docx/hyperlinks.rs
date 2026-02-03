//! Hyperlink parsing for DOCX files
//!
//! Handles w:hyperlink elements and their associated relationships.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use crate::docx::relationships::Relationships;
use doc_model::{DocumentTree, Hyperlink, HyperlinkTarget, Node, Run};
use quick_xml::events::Event;

/// Parser for hyperlink elements
pub struct HyperlinkParser<'a> {
    /// Document relationships for resolving external links
    doc_rels: &'a Relationships,
}

impl<'a> HyperlinkParser<'a> {
    /// Create a new hyperlink parser
    pub fn new(doc_rels: &'a Relationships) -> Self {
        Self { doc_rels }
    }

    /// Parse a w:hyperlink element and return parsed data
    pub fn parse_hyperlink(&self, content: &str) -> DocxResult<ParsedHyperlink> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut parsed = ParsedHyperlink::default();
        let mut current_run: Option<ParsedRun> = None;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "hyperlink") {
                        // Get the relationship ID or anchor
                        parsed.rel_id = XmlParser::get_r_attribute(e, "id");
                        parsed.anchor = XmlParser::get_w_attribute(e, "anchor");
                        parsed.tooltip = XmlParser::get_w_attribute(e, "tooltip");
                    } else if XmlParser::matches_element(name_ref, "r") {
                        current_run = Some(ParsedRun::default());
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "r") {
                        if let Some(run) = current_run.take() {
                            parsed.runs.push(run);
                        }
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        if let Some(ref mut run) = current_run {
                            let text = e.unescape().map_err(|e| DocxError::XmlParse(e.to_string()))?;
                            run.text.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(parsed)
    }

    /// Resolve a hyperlink target from its relationship ID or anchor
    pub fn resolve_target(&self, parsed: &ParsedHyperlink) -> Option<HyperlinkTarget> {
        // First, check for an anchor (internal bookmark link)
        if let Some(ref anchor) = parsed.anchor {
            return Some(HyperlinkTarget::internal(anchor));
        }

        // Then, check for a relationship ID (external link)
        if let Some(ref rel_id) = parsed.rel_id {
            if let Some(rel) = self.doc_rels.get(rel_id) {
                let url = &rel.target;

                // Check if it's an email link
                if url.starts_with("mailto:") {
                    return Some(parse_mailto_url(url));
                }

                // External URL
                return Some(HyperlinkTarget::external(url));
            }
        }

        None
    }

    /// Create a Hyperlink node from parsed data
    pub fn create_hyperlink(&self, parsed: &ParsedHyperlink) -> Option<Hyperlink> {
        let target = self.resolve_target(parsed)?;

        let mut hyperlink = if let Some(ref tooltip) = parsed.tooltip {
            Hyperlink::with_tooltip(target, tooltip)
        } else {
            Hyperlink::new(target)
        };

        Some(hyperlink)
    }
}

/// Parsed hyperlink data
#[derive(Debug, Default)]
pub struct ParsedHyperlink {
    /// Relationship ID for external links
    pub rel_id: Option<String>,
    /// Anchor name for internal links
    pub anchor: Option<String>,
    /// Tooltip text
    pub tooltip: Option<String>,
    /// Runs containing the hyperlink text
    pub runs: Vec<ParsedRun>,
}

/// Parsed run data (simplified for hyperlinks)
#[derive(Debug, Default)]
pub struct ParsedRun {
    pub text: String,
}

/// Parse a mailto: URL into an email target
fn parse_mailto_url(url: &str) -> HyperlinkTarget {
    let email_part = url.trim_start_matches("mailto:");

    // Parse out address and optional subject
    if let Some(question_pos) = email_part.find('?') {
        let address = &email_part[..question_pos];
        let query = &email_part[question_pos + 1..];

        // Parse query parameters
        let mut subject = None;
        for param in query.split('&') {
            if let Some(eq_pos) = param.find('=') {
                let key = &param[..eq_pos];
                let value = &param[eq_pos + 1..];

                if key.eq_ignore_ascii_case("subject") {
                    // URL decode the subject
                    subject = Some(url_decode(value));
                }
            }
        }

        HyperlinkTarget::email(address, subject)
    } else {
        HyperlinkTarget::email(email_part, None)
    }
}

/// Simple URL decoding for common escape sequences
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to read two hex digits
            let h1 = chars.next();
            let h2 = chars.next();

            if let (Some(h1), Some(h2)) = (h1, h2) {
                let hex = format!("{}{}", h1, h2);
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }

            // If decoding fails, keep the original
            result.push('%');
            if let Some(h1) = h1 {
                result.push(h1);
            }
            if let Some(h2) = h2 {
                result.push(h2);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mailto_simple() {
        let target = parse_mailto_url("mailto:test@example.com");
        if let HyperlinkTarget::Email { address, subject } = target {
            assert_eq!(address, "test@example.com");
            assert!(subject.is_none());
        } else {
            panic!("Expected Email target");
        }
    }

    #[test]
    fn test_parse_mailto_with_subject() {
        let target = parse_mailto_url("mailto:test@example.com?subject=Hello%20World");
        if let HyperlinkTarget::Email { address, subject } = target {
            assert_eq!(address, "test@example.com");
            assert_eq!(subject, Some("Hello World".to_string()));
        } else {
            panic!("Expected Email target");
        }
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("Hello%20World"), "Hello World");
        assert_eq!(url_decode("Hello+World"), "Hello World");
        assert_eq!(url_decode("%3A%2F%2F"), "://");
        assert_eq!(url_decode("no%encoding"), "no%encoding");
    }

    #[test]
    fn test_parsed_hyperlink_structure() {
        let parsed = ParsedHyperlink {
            rel_id: Some("rId1".to_string()),
            anchor: None,
            tooltip: Some("Click here".to_string()),
            runs: vec![
                ParsedRun { text: "Link ".to_string() },
                ParsedRun { text: "text".to_string() },
            ],
        };

        assert!(parsed.rel_id.is_some());
        assert!(parsed.anchor.is_none());
        assert_eq!(parsed.runs.len(), 2);
    }
}
