//! Main DOCX parsing logic
//!
//! This module coordinates parsing of all DOCX parts and builds the DocumentTree.

use crate::docx::content_types::ContentTypes;
use crate::docx::document::DocumentParser;
use crate::docx::error::{DocxError, DocxResult};
use crate::docx::images::ImageParser;
use crate::docx::lists::NumberingParser;
use crate::docx::reader::DocxReader;
use crate::docx::relationships::Relationships;
use crate::docx::relationship_types;
use crate::docx::styles::StylesParser;
use doc_model::DocumentTree;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Parsed DOCX data before conversion to DocumentTree
#[derive(Debug)]
pub struct ParsedDocx {
    /// Content types
    pub content_types: ContentTypes,
    /// Root relationships
    pub root_rels: Relationships,
    /// Document relationships
    pub doc_rels: Relationships,
    /// Raw document.xml content
    pub document_xml: String,
    /// Raw styles.xml content (if present)
    pub styles_xml: Option<String>,
    /// Raw numbering.xml content (if present)
    pub numbering_xml: Option<String>,
    /// Image data keyed by relationship ID
    pub images: HashMap<String, ImageData>,
    /// External hyperlink targets keyed by relationship ID
    pub hyperlinks: HashMap<String, String>,
}

/// Image data from the DOCX
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Relationship ID
    pub rel_id: String,
    /// Image file path within the archive
    pub path: String,
    /// Image data bytes
    pub data: Vec<u8>,
    /// Content type (e.g., "image/png")
    pub content_type: String,
}

/// Main parser for DOCX files
pub struct DocxParser;

impl DocxParser {
    /// Parse a DOCX file from a reader and build a DocumentTree
    pub fn parse<R: Read + Seek>(reader: R) -> DocxResult<DocumentTree> {
        // First, read and parse all parts
        let parsed = Self::read_parts(reader)?;

        // Then, convert to DocumentTree
        Self::build_tree(parsed)
    }

    /// Read all parts from the DOCX archive
    fn read_parts<R: Read + Seek>(reader: R) -> DocxResult<ParsedDocx> {
        let mut docx = DocxReader::new(reader)?;

        // Validate it's a proper DOCX
        if !docx.is_valid_docx() {
            return Err(DocxError::InvalidStructure(
                "Missing required DOCX files".to_string()
            ));
        }

        // Parse [Content_Types].xml
        let ct_content = docx.read_file_as_string("[Content_Types].xml")?;
        let content_types = ContentTypes::parse(&ct_content)?;

        // Parse _rels/.rels
        let root_rels_content = docx.read_file_as_string("_rels/.rels")?;
        let root_rels = Relationships::parse(&root_rels_content)?;

        // Find the main document relationship
        let doc_rel = root_rels.get_by_type(relationship_types::DOCUMENT)
            .ok_or_else(|| DocxError::MissingPart("Main document relationship".into()))?;

        // Parse document.xml
        let document_xml = docx.read_file_as_string(&doc_rel.target)?;

        // Parse word/_rels/document.xml.rels (if exists)
        let doc_rels = if docx.file_exists("word/_rels/document.xml.rels") {
            let rels_content = docx.read_file_as_string("word/_rels/document.xml.rels")?;
            Relationships::parse(&rels_content)?
        } else {
            Relationships::new()
        };

        // Parse styles.xml (if exists)
        let styles_xml = if let Some(style_rel) = doc_rels.get_by_type(relationship_types::STYLES) {
            let path = format!("word/{}", style_rel.target);
            if docx.file_exists(&path) {
                Some(docx.read_file_as_string(&path)?)
            } else {
                None
            }
        } else if docx.file_exists("word/styles.xml") {
            Some(docx.read_file_as_string("word/styles.xml")?)
        } else {
            None
        };

        // Parse numbering.xml (if exists)
        let numbering_xml = if let Some(num_rel) = doc_rels.get_by_type(relationship_types::NUMBERING) {
            let path = format!("word/{}", num_rel.target);
            if docx.file_exists(&path) {
                Some(docx.read_file_as_string(&path)?)
            } else {
                None
            }
        } else if docx.file_exists("word/numbering.xml") {
            Some(docx.read_file_as_string("word/numbering.xml")?)
        } else {
            None
        };

        // Load images
        let mut images = HashMap::new();
        for rel in doc_rels.get_all_by_type(relationship_types::IMAGE) {
            let path = if rel.target.starts_with("media/") {
                format!("word/{}", rel.target)
            } else {
                rel.target.clone()
            };

            if docx.file_exists(&path) {
                let data = docx.read_file_as_bytes(&path)?;
                let content_type = content_types.get_content_type(&path)
                    .cloned()
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                images.insert(rel.id.clone(), ImageData {
                    rel_id: rel.id.clone(),
                    path,
                    data,
                    content_type,
                });
            }
        }

        // Collect hyperlink targets
        let mut hyperlinks = HashMap::new();
        for rel in doc_rels.get_all_by_type(relationship_types::HYPERLINK) {
            hyperlinks.insert(rel.id.clone(), rel.target.clone());
        }

        Ok(ParsedDocx {
            content_types,
            root_rels,
            doc_rels,
            document_xml,
            styles_xml,
            numbering_xml,
            images,
            hyperlinks,
        })
    }

    /// Build a DocumentTree from parsed DOCX data
    fn build_tree(parsed: ParsedDocx) -> DocxResult<DocumentTree> {
        let mut tree = DocumentTree::new();

        // Parse styles first (needed for document parsing)
        if let Some(ref styles_xml) = parsed.styles_xml {
            let styles_parser = StylesParser::new();
            let styles = styles_parser.parse(styles_xml)?;
            for style in styles {
                tree.style_registry_mut().register(style);
            }
        }

        // Parse numbering definitions
        if let Some(ref numbering_xml) = parsed.numbering_xml {
            let numbering_parser = NumberingParser::new();
            let (abstract_nums, instances) = numbering_parser.parse(numbering_xml)?;

            for abstract_num in abstract_nums {
                tree.numbering_registry_mut().create_abstract_num(abstract_num);
            }
            for instance in instances {
                tree.numbering_registry_mut().create_instance(instance);
            }
        }

        // Parse the main document
        let doc_parser = DocumentParser::new(&parsed.doc_rels, &parsed.hyperlinks);
        doc_parser.parse(&parsed.document_xml, &mut tree)?;

        // Process images
        let image_parser = ImageParser::new();
        for (rel_id, image_data) in &parsed.images {
            image_parser.process_image(rel_id, image_data, &mut tree)?;
        }

        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_structure() {
        // Basic structural test
        let parsed = ParsedDocx {
            content_types: ContentTypes::new(),
            root_rels: Relationships::new(),
            doc_rels: Relationships::new(),
            document_xml: String::new(),
            styles_xml: None,
            numbering_xml: None,
            images: HashMap::new(),
            hyperlinks: HashMap::new(),
        };

        assert!(parsed.styles_xml.is_none());
        assert!(parsed.images.is_empty());
    }
}
