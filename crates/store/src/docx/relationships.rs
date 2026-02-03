//! Relationships (.rels) file parsing and generation
//!
//! DOCX uses relationships to connect parts of the document together.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use crate::docx::relationship_types;
use quick_xml::events::Event;
use std::collections::HashMap;

/// A single relationship in a .rels file
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Unique ID within the rels file (e.g., "rId1")
    pub id: String,
    /// Relationship type URI
    pub rel_type: String,
    /// Target path (relative to the source part)
    pub target: String,
    /// Target mode (Internal or External)
    pub target_mode: TargetMode,
}

/// Target mode for relationships
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetMode {
    /// Internal target within the package
    Internal,
    /// External target (URL)
    External,
}

impl Default for TargetMode {
    fn default() -> Self {
        TargetMode::Internal
    }
}

/// Collection of relationships from a .rels file
#[derive(Debug, Clone, Default)]
pub struct Relationships {
    /// Map of relationship ID to relationship
    relationships: HashMap<String, Relationship>,
    /// Counter for generating new IDs
    next_id: u32,
}

impl Relationships {
    /// Create a new empty relationships collection
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
            next_id: 1,
        }
    }

    /// Parse a .rels file from its XML content
    pub fn parse(content: &str) -> DocxResult<Self> {
        let mut result = Self::new();
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();
        let mut max_id = 0u32;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if XmlParser::matches_element(name.as_ref(), "Relationship") {
                        let id = XmlParser::get_attribute(e, b"Id")
                            .ok_or_else(|| DocxError::InvalidStructure("Relationship missing Id".into()))?;
                        let rel_type = XmlParser::get_attribute(e, b"Type")
                            .ok_or_else(|| DocxError::InvalidStructure("Relationship missing Type".into()))?;
                        let target = XmlParser::get_attribute(e, b"Target")
                            .ok_or_else(|| DocxError::InvalidStructure("Relationship missing Target".into()))?;
                        let target_mode = XmlParser::get_attribute(e, b"TargetMode")
                            .map(|m| if m == "External" { TargetMode::External } else { TargetMode::Internal })
                            .unwrap_or(TargetMode::Internal);

                        // Track max ID for generating new ones
                        if let Some(num) = id.strip_prefix("rId").and_then(|n| n.parse::<u32>().ok()) {
                            max_id = max_id.max(num);
                        }

                        result.relationships.insert(id.clone(), Relationship {
                            id,
                            rel_type,
                            target,
                            target_mode,
                        });
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        result.next_id = max_id + 1;
        Ok(result)
    }

    /// Add a relationship and return its ID
    pub fn add(&mut self, rel_type: &str, target: &str, target_mode: TargetMode) -> String {
        let id = format!("rId{}", self.next_id);
        self.next_id += 1;

        self.relationships.insert(id.clone(), Relationship {
            id: id.clone(),
            rel_type: rel_type.to_string(),
            target: target.to_string(),
            target_mode,
        });

        id
    }

    /// Get a relationship by ID
    pub fn get(&self, id: &str) -> Option<&Relationship> {
        self.relationships.get(id)
    }

    /// Get a relationship by type
    pub fn get_by_type(&self, rel_type: &str) -> Option<&Relationship> {
        self.relationships.values().find(|r| r.rel_type == rel_type)
    }

    /// Get all relationships of a given type
    pub fn get_all_by_type(&self, rel_type: &str) -> Vec<&Relationship> {
        self.relationships.values()
            .filter(|r| r.rel_type == rel_type)
            .collect()
    }

    /// Get the target path for a relationship ID
    pub fn get_target(&self, id: &str) -> Option<&str> {
        self.relationships.get(id).map(|r| r.target.as_str())
    }

    /// Check if a relationship exists
    pub fn contains(&self, id: &str) -> bool {
        self.relationships.contains_key(id)
    }

    /// Get all relationships
    pub fn all(&self) -> impl Iterator<Item = &Relationship> {
        self.relationships.values()
    }

    /// Generate XML content for the .rels file
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push('\n');
        xml.push_str(r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#);

        for rel in self.relationships.values() {
            xml.push_str(&format!(
                r#"<Relationship Id="{}" Type="{}" Target="{}""#,
                rel.id, rel.rel_type, rel.target
            ));
            if rel.target_mode == TargetMode::External {
                xml.push_str(r#" TargetMode="External""#);
            }
            xml.push_str("/>");
        }

        xml.push_str("</Relationships>");
        xml
    }
}

/// Create the root .rels file for a new DOCX
pub fn create_root_rels() -> Relationships {
    let mut rels = Relationships::new();
    rels.add(
        relationship_types::DOCUMENT,
        "word/document.xml",
        TargetMode::Internal,
    );
    rels
}

/// Create the document.xml.rels for a new DOCX
pub fn create_document_rels() -> Relationships {
    let mut rels = Relationships::new();
    rels.add(
        relationship_types::STYLES,
        "styles.xml",
        TargetMode::Internal,
    );
    rels.add(
        relationship_types::NUMBERING,
        "numbering.xml",
        TargetMode::Internal,
    );
    rels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationships_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External"/>
</Relationships>"#;

        let rels = Relationships::parse(xml).unwrap();
        assert_eq!(rels.relationships.len(), 2);

        let r1 = rels.get("rId1").unwrap();
        assert_eq!(r1.target, "word/document.xml");
        assert_eq!(r1.target_mode, TargetMode::Internal);

        let r2 = rels.get("rId2").unwrap();
        assert_eq!(r2.target, "https://example.com");
        assert_eq!(r2.target_mode, TargetMode::External);
    }

    #[test]
    fn test_add_relationship() {
        let mut rels = Relationships::new();
        let id1 = rels.add(
            relationship_types::DOCUMENT,
            "word/document.xml",
            TargetMode::Internal,
        );
        let id2 = rels.add(
            relationship_types::STYLES,
            "word/styles.xml",
            TargetMode::Internal,
        );

        assert_eq!(id1, "rId1");
        assert_eq!(id2, "rId2");
        assert!(rels.contains("rId1"));
        assert!(rels.contains("rId2"));
    }

    #[test]
    fn test_get_by_type() {
        let rels = create_root_rels();
        let doc_rel = rels.get_by_type(relationship_types::DOCUMENT);
        assert!(doc_rel.is_some());
        assert_eq!(doc_rel.unwrap().target, "word/document.xml");
    }

    #[test]
    fn test_to_xml_roundtrip() {
        let original = create_root_rels();
        let xml = original.to_xml();
        let parsed = Relationships::parse(&xml).unwrap();

        assert_eq!(original.relationships.len(), parsed.relationships.len());
        assert!(parsed.get_by_type(relationship_types::DOCUMENT).is_some());
    }
}
