//! DOCX Writer Infrastructure
//!
//! Creates ZIP archives with correct DOCX structure.

use crate::docx::content_types::{create_default_content_types, ContentTypes};
use crate::docx::document_writer::DocumentWriter;
use crate::docx::error::{DocxError, DocxResult};
use crate::docx::media_writer::MediaWriter;
use crate::docx::numbering_writer::NumberingWriter;
use crate::docx::relationships::{create_document_rels, create_root_rels, Relationships, TargetMode};
use crate::docx::relationship_types;
use crate::docx::styles_writer::StylesWriter;
use doc_model::DocumentTree;
use std::io::{Seek, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Main DOCX writer
pub struct DocxWriter<W: Write + Seek> {
    zip: ZipWriter<W>,
    content_types: ContentTypes,
    root_rels: Relationships,
    doc_rels: Relationships,
}

impl<W: Write + Seek> DocxWriter<W> {
    /// Create a new DOCX writer
    pub fn new(writer: W) -> Self {
        Self {
            zip: ZipWriter::new(writer),
            content_types: create_default_content_types(),
            root_rels: create_root_rels(),
            doc_rels: create_document_rels(),
        }
    }

    /// Write a complete DOCX file from a DocumentTree
    pub fn write(mut self, tree: &DocumentTree) -> DocxResult<()> {
        // Write document.xml
        let doc_xml = DocumentWriter::new().write(tree)?;
        self.write_file("word/document.xml", &doc_xml)?;

        // Write styles.xml
        let styles_xml = StylesWriter::new().write(tree)?;
        self.write_file("word/styles.xml", &styles_xml)?;

        // Write numbering.xml if there are list definitions
        if tree.numbering_registry().all_abstract_nums().next().is_some() {
            let numbering_xml = NumberingWriter::new().write(tree)?;
            self.write_file("word/numbering.xml", &numbering_xml)?;
        }

        // Write media files (images)
        let media_writer = MediaWriter::new();
        let media_rels = media_writer.write_media(tree, &mut self)?;

        // Add image relationships
        for (_rel_id, path) in media_rels {
            self.doc_rels.add(
                relationship_types::IMAGE,
                &path,
                TargetMode::Internal,
            );
        }

        // Add hyperlink relationships from the document
        // These are collected during document writing
        // For now we skip this as they're handled inline

        // Write relationships
        let root_rels_xml = self.root_rels.to_xml();
        self.write_file("_rels/.rels", &root_rels_xml)?;

        let doc_rels_xml = self.doc_rels.to_xml();
        self.write_file("word/_rels/document.xml.rels", &doc_rels_xml)?;

        // Write [Content_Types].xml last
        let content_types_xml = self.content_types.to_xml();
        self.write_file("[Content_Types].xml", &content_types_xml)?;

        // Finish the ZIP archive
        self.zip.finish()?;

        Ok(())
    }

    /// Write a file to the ZIP archive
    pub fn write_file(&mut self, path: &str, content: &str) -> DocxResult<()> {
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        self.zip.start_file(path, options)?;
        self.zip.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Write binary data to the ZIP archive
    pub fn write_binary(&mut self, path: &str, data: &[u8]) -> DocxResult<()> {
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored); // Don't compress binary

        self.zip.start_file(path, options)?;
        self.zip.write_all(data)?;

        Ok(())
    }

    /// Get mutable access to content types
    pub fn content_types_mut(&mut self) -> &mut ContentTypes {
        &mut self.content_types
    }

    /// Get mutable access to document relationships
    pub fn doc_rels_mut(&mut self) -> &mut Relationships {
        &mut self.doc_rels
    }

    /// Add a hyperlink relationship and return its ID
    pub fn add_hyperlink(&mut self, url: &str) -> String {
        self.doc_rels.add(
            relationship_types::HYPERLINK,
            url,
            TargetMode::External,
        )
    }
}

/// Generate a minimal settings.xml
pub fn generate_settings_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:compat>
        <w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/>
    </w:compat>
</w:settings>"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_writer_creation() {
        let buffer = Cursor::new(Vec::new());
        let writer = DocxWriter::new(buffer);

        // Verify initial state
        assert!(writer.root_rels.get_by_type(relationship_types::DOCUMENT).is_some());
        assert!(writer.doc_rels.get_by_type(relationship_types::STYLES).is_some());
    }

    #[test]
    fn test_add_hyperlink() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = DocxWriter::new(buffer);

        let rel_id = writer.add_hyperlink("https://example.com");
        assert!(rel_id.starts_with("rId"));

        let rel = writer.doc_rels.get(&rel_id).unwrap();
        assert_eq!(rel.target, "https://example.com");
        assert_eq!(rel.target_mode, TargetMode::External);
    }

    #[test]
    fn test_generate_settings() {
        let settings = generate_settings_xml();
        assert!(settings.contains("w:settings"));
        assert!(settings.contains("compatibilityMode"));
    }
}
