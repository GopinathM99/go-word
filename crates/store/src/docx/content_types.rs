//! [Content_Types].xml parsing and generation
//!
//! This file defines the content types for all parts in the DOCX package.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use quick_xml::events::Event;
use std::collections::HashMap;

/// Represents the content types in a DOCX package
#[derive(Debug, Clone, Default)]
pub struct ContentTypes {
    /// Default content types by extension (e.g., "xml" -> "application/xml")
    pub defaults: HashMap<String, String>,
    /// Override content types by part name (e.g., "/word/document.xml" -> "...")
    pub overrides: HashMap<String, String>,
}

impl ContentTypes {
    /// Create a new ContentTypes with default DOCX settings
    pub fn new() -> Self {
        let mut ct = Self::default();

        // Add standard defaults
        ct.defaults.insert("rels".to_string(),
            "application/vnd.openxmlformats-package.relationships+xml".to_string());
        ct.defaults.insert("xml".to_string(),
            "application/xml".to_string());

        ct
    }

    /// Parse [Content_Types].xml from its content
    pub fn parse(content: &str) -> DocxResult<Self> {
        let mut result = Self::default();
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if XmlParser::matches_element(name.as_ref(), "Default") {
                        if let (Some(ext), Some(ct)) = (
                            XmlParser::get_attribute(e, b"Extension"),
                            XmlParser::get_attribute(e, b"ContentType"),
                        ) {
                            result.defaults.insert(ext, ct);
                        }
                    } else if XmlParser::matches_element(name.as_ref(), "Override") {
                        if let (Some(part), Some(ct)) = (
                            XmlParser::get_attribute(e, b"PartName"),
                            XmlParser::get_attribute(e, b"ContentType"),
                        ) {
                            result.overrides.insert(part, ct);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(result)
    }

    /// Get the content type for a given path
    pub fn get_content_type(&self, path: &str) -> Option<&String> {
        // First check overrides
        let normalized_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };

        if let Some(ct) = self.overrides.get(&normalized_path) {
            return Some(ct);
        }

        // Then check defaults by extension
        if let Some(ext) = path.rsplit('.').next() {
            return self.defaults.get(ext);
        }

        None
    }

    /// Add an override for a specific part
    pub fn add_override(&mut self, part_name: &str, content_type: &str) {
        let normalized = if part_name.starts_with('/') {
            part_name.to_string()
        } else {
            format!("/{}", part_name)
        };
        self.overrides.insert(normalized, content_type.to_string());
    }

    /// Generate XML content for [Content_Types].xml
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');
        xml.push_str(r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#);

        // Write defaults
        for (ext, ct) in &self.defaults {
            xml.push_str(&format!(
                r#"<Default Extension="{}" ContentType="{}"/>"#,
                ext, ct
            ));
        }

        // Write overrides
        for (part, ct) in &self.overrides {
            xml.push_str(&format!(
                r#"<Override PartName="{}" ContentType="{}"/>"#,
                part, ct
            ));
        }

        xml.push_str("</Types>");
        xml
    }
}

/// Create default content types for a new DOCX file
pub fn create_default_content_types() -> ContentTypes {
    let mut ct = ContentTypes::new();

    // Add standard overrides for DOCX
    ct.add_override(
        "/word/document.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
    );
    ct.add_override(
        "/word/styles.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"
    );
    ct.add_override(
        "/word/numbering.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"
    );
    ct.add_override(
        "/word/settings.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"
    );

    // Add image types
    ct.defaults.insert("png".to_string(), "image/png".to_string());
    ct.defaults.insert("jpg".to_string(), "image/jpeg".to_string());
    ct.defaults.insert("jpeg".to_string(), "image/jpeg".to_string());
    ct.defaults.insert("gif".to_string(), "image/gif".to_string());

    ct
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_types_creation() {
        let ct = ContentTypes::new();
        assert!(ct.defaults.contains_key("rels"));
        assert!(ct.defaults.contains_key("xml"));
    }

    #[test]
    fn test_content_types_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="xml" ContentType="application/xml"/>
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

        let ct = ContentTypes::parse(xml).unwrap();
        assert_eq!(ct.defaults.get("xml"), Some(&"application/xml".to_string()));
        assert!(ct.overrides.contains_key("/word/document.xml"));
    }

    #[test]
    fn test_get_content_type() {
        let ct = create_default_content_types();

        // Test override lookup
        assert!(ct.get_content_type("/word/document.xml").is_some());
        assert!(ct.get_content_type("word/document.xml").is_some());

        // Test default lookup
        assert!(ct.get_content_type("test.xml").is_some());
        assert!(ct.get_content_type("image.png").is_some());
    }

    #[test]
    fn test_to_xml_roundtrip() {
        let original = create_default_content_types();
        let xml = original.to_xml();
        let parsed = ContentTypes::parse(&xml).unwrap();

        assert_eq!(original.defaults.len(), parsed.defaults.len());
        assert_eq!(original.overrides.len(), parsed.overrides.len());
    }
}
