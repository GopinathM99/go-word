//! DOCX Import/Export Module
//!
//! This module provides functionality to read and write Microsoft Word DOCX files.
//! DOCX is based on the Office Open XML (OOXML) format defined in ECMA-376.
//!
//! ## Structure
//!
//! A DOCX file is a ZIP archive containing XML files:
//! - `[Content_Types].xml` - Content type definitions
//! - `_rels/.rels` - Root relationships
//! - `word/document.xml` - Main document content
//! - `word/styles.xml` - Style definitions
//! - `word/numbering.xml` - List/numbering definitions
//! - `word/_rels/document.xml.rels` - Document relationships
//! - `word/media/` - Embedded images and media
//! - `word/footnotes.xml` - Footnotes content
//! - `word/endnotes.xml` - Endnotes content
//! - `word/comments.xml` - Comments content
//!
//! ## Phase 2 Features
//!
//! This module now supports advanced DOCX features:
//! - Track changes (w:ins, w:del, w:moveFrom, w:moveTo)
//! - Comments with threading
//! - Footnotes and endnotes
//! - Fields (PAGE, TOC, REF, SEQ)
//! - Advanced tables (cell merging, row breaks, nested tables)
//! - Text boxes and shapes
//! - Shape groups and connectors

mod error;
mod reader;
mod content_types;
mod relationships;
mod parser;
mod document;
mod styles;
mod tables;
mod lists;
mod images;
mod hyperlinks;
mod writer;
mod document_writer;
mod styles_writer;
mod tables_writer;
mod numbering_writer;
mod media_writer;
mod api;

// Phase 2 modules for advanced DOCX features
mod track_changes;
mod comments_io;
mod footnotes_io;
mod fields_io;
mod drawings_io;
mod tables_io;
mod fidelity;
mod content_controls;
mod content_controls_writer;

pub use error::{DocxError, DocxResult};
pub use api::{import_docx, export_docx, import_docx_bytes, export_docx_bytes};
pub use api::{FileFormat, get_supported_formats, get_import_formats, get_export_formats};

// Re-export Phase 2 types for external use
pub use fidelity::{
    FidelityTracker, FidelityWarning, FidelityReport, WarningSeverity, FeatureCategory,
    ImportOptions, ExportOptions, WordVersion,
};
pub use track_changes::{TrackChangesParser, TrackChangesWriter, ParsedInsertion, ParsedDeletion, ParsedMove};
pub use comments_io::{CommentsParser, CommentsWriter, ParsedComment};
pub use footnotes_io::{NotesParser, NotesWriter, ParsedNote, NoteType};
pub use fields_io::{FieldParser, FieldWriter, ParsedField, Field, FieldInstruction};
pub use drawings_io::{DrawingParser, DrawingWriter, ParsedDrawing, DrawingType};
pub use tables_io::{TableParser, TableWriter, ParsedTable, ParsedTableRow, ParsedTableCell, VerticalMerge};
pub use content_controls::{
    ContentControlParser, ParsedContentControl, ParsedControlType, ParsedTypeProperties,
    ParsedDataBinding, ParsedListItem, LockSettings, CheckboxState,
};
pub use content_controls_writer::ContentControlWriter;

/// XML namespaces used in DOCX files
pub mod namespaces {
    /// Main WordprocessingML namespace
    pub const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    /// Relationships namespace
    pub const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    /// Package relationships namespace
    pub const PKG_REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    /// Content types namespace
    pub const CT: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
    /// DrawingML namespace
    pub const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    /// WordprocessingML Drawing namespace
    pub const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
    /// Picture namespace
    pub const PIC: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";
    /// VML namespace
    pub const V: &str = "urn:schemas-microsoft-com:vml";
}

/// Relationship types used in DOCX
pub mod relationship_types {
    pub const DOCUMENT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
    pub const STYLES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
    pub const NUMBERING: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering";
    pub const IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
    pub const HYPERLINK: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
    pub const SETTINGS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";
    pub const HEADER: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
    pub const FOOTER: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
    pub const FOOTNOTES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes";
    pub const ENDNOTES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes";
    pub const COMMENTS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
    pub const THEME: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
    pub const FONT_TABLE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable";
    pub const WEB_SETTINGS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings";
}

/// Content types for DOCX parts
pub mod content_type_values {
    pub const DOCUMENT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
    pub const STYLES: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
    pub const NUMBERING: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml";
    pub const SETTINGS: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml";
    pub const RELATIONSHIPS: &str = "application/vnd.openxmlformats-package.relationships+xml";
    pub const FOOTNOTES: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml";
    pub const ENDNOTES: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml";
    pub const COMMENTS: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
    pub const THEME: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
    pub const FONT_TABLE: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml";
    pub const WEB_SETTINGS: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Basic test to verify module compiles
        assert!(namespaces::W.contains("wordprocessingml"));
    }
}
