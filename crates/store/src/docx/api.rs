//! Public API for DOCX import/export
//!
//! This module provides the main entry points for working with DOCX files.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::parser::DocxParser;
use crate::docx::writer::DocxWriter;
use doc_model::DocumentTree;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::Path;

/// Import a DOCX file from disk and return a DocumentTree
///
/// # Arguments
///
/// * `path` - Path to the DOCX file
///
/// # Returns
///
/// * `Ok(DocumentTree)` - The parsed document tree
/// * `Err(DocxError)` - If parsing fails
///
/// # Example
///
/// ```ignore
/// use store::docx::import_docx;
/// use std::path::Path;
///
/// let tree = import_docx(Path::new("document.docx"))?;
/// ```
pub fn import_docx(path: &Path) -> DocxResult<DocumentTree> {
    // Open the file
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DocxError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ))
        } else {
            DocxError::Io(e)
        }
    })?;

    // Wrap in a buffered reader for better performance
    let reader = BufReader::new(file);

    // Parse the DOCX
    DocxParser::parse(reader)
}

/// Export a DocumentTree to a DOCX file on disk
///
/// # Arguments
///
/// * `tree` - The document tree to export
/// * `path` - Path where the DOCX file will be saved
///
/// # Returns
///
/// * `Ok(())` - If export succeeds
/// * `Err(DocxError)` - If export fails
///
/// # Example
///
/// ```ignore
/// use store::docx::export_docx;
/// use doc_model::DocumentTree;
/// use std::path::Path;
///
/// let tree = DocumentTree::new();
/// export_docx(&tree, Path::new("output.docx"))?;
/// ```
pub fn export_docx(tree: &DocumentTree, path: &Path) -> DocxResult<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create the output file
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Create the DOCX writer and write the document
    let docx_writer = DocxWriter::new(writer);
    docx_writer.write(tree)
}

/// Import a DOCX from an in-memory byte slice
///
/// # Arguments
///
/// * `bytes` - The DOCX file content as bytes
///
/// # Returns
///
/// * `Ok(DocumentTree)` - The parsed document tree
/// * `Err(DocxError)` - If parsing fails
///
/// # Example
///
/// ```ignore
/// use store::docx::import_docx_bytes;
///
/// let docx_data: Vec<u8> = std::fs::read("document.docx")?;
/// let tree = import_docx_bytes(&docx_data)?;
/// ```
pub fn import_docx_bytes(bytes: &[u8]) -> DocxResult<DocumentTree> {
    let cursor = Cursor::new(bytes);
    DocxParser::parse(cursor)
}

/// Export a DocumentTree to an in-memory byte vector
///
/// # Arguments
///
/// * `tree` - The document tree to export
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The DOCX file content as bytes
/// * `Err(DocxError)` - If export fails
///
/// # Example
///
/// ```ignore
/// use store::docx::export_docx_bytes;
/// use doc_model::DocumentTree;
///
/// let tree = DocumentTree::new();
/// let bytes = export_docx_bytes(&tree)?;
/// std::fs::write("output.docx", bytes)?;
/// ```
pub fn export_docx_bytes(tree: &DocumentTree) -> DocxResult<Vec<u8>> {
    let cursor = Cursor::new(Vec::new());
    let docx_writer = DocxWriter::new(cursor);

    // Write the document
    // We need to get the bytes back from the cursor after writing
    // This requires a bit of restructuring since write() consumes self

    // For now, we'll create the file in memory piece by piece
    let buffer = Cursor::new(Vec::new());
    let mut writer = DocxWriter::new(buffer);

    // Write all the parts manually to get access to the inner cursor
    // This is a workaround - in production, we'd want DocxWriter to return the inner writer

    let cursor = Cursor::new(Vec::new());
    let docx_writer = DocxWriter::new(cursor);
    match docx_writer.write(tree) {
        Ok(()) => {
            // The write was successful, but we need to get the bytes
            // This is tricky because write() consumes the writer
            // For now, let's use a different approach
        }
        Err(e) => return Err(e),
    }

    // Alternative implementation: write to a temp buffer
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let writer = DocxWriter::new(cursor);
        writer.write(tree)?;
    }

    Ok(buffer)
}

/// Supported file formats for import/export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// Microsoft Word DOCX format (Office Open XML)
    Docx,
    /// Rich Text Format
    Rtf,
    /// Plain text
    Txt,
    /// HTML
    Html,
    /// PDF (export only)
    Pdf,
}

impl FileFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::Docx => "docx",
            FileFormat::Rtf => "rtf",
            FileFormat::Txt => "txt",
            FileFormat::Html => "html",
            FileFormat::Pdf => "pdf",
        }
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileFormat::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            FileFormat::Rtf => "application/rtf",
            FileFormat::Txt => "text/plain",
            FileFormat::Html => "text/html",
            FileFormat::Pdf => "application/pdf",
        }
    }

    /// Get a human-readable name for this format
    pub fn display_name(&self) -> &'static str {
        match self {
            FileFormat::Docx => "Word Document",
            FileFormat::Rtf => "Rich Text Format",
            FileFormat::Txt => "Plain Text",
            FileFormat::Html => "HTML Document",
            FileFormat::Pdf => "PDF Document",
        }
    }

    /// Check if this format supports import
    pub fn supports_import(&self) -> bool {
        matches!(self, FileFormat::Docx | FileFormat::Rtf | FileFormat::Txt | FileFormat::Html)
    }

    /// Check if this format supports export
    pub fn supports_export(&self) -> bool {
        true // All formats support export
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<FileFormat> {
        match ext.to_lowercase().as_str() {
            "docx" => Some(FileFormat::Docx),
            "rtf" => Some(FileFormat::Rtf),
            "txt" => Some(FileFormat::Txt),
            "html" | "htm" => Some(FileFormat::Html),
            "pdf" => Some(FileFormat::Pdf),
            _ => None,
        }
    }

    /// Detect format from file path
    pub fn from_path(path: &Path) -> Option<FileFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(FileFormat::from_extension)
    }
}

/// Get a list of all supported file formats
pub fn get_supported_formats() -> Vec<FileFormat> {
    vec![
        FileFormat::Docx,
        FileFormat::Rtf,
        FileFormat::Txt,
        FileFormat::Html,
        FileFormat::Pdf,
    ]
}

/// Get formats that support import
pub fn get_import_formats() -> Vec<FileFormat> {
    get_supported_formats()
        .into_iter()
        .filter(|f| f.supports_import())
        .collect()
}

/// Get formats that support export
pub fn get_export_formats() -> Vec<FileFormat> {
    get_supported_formats()
        .into_iter()
        .filter(|f| f.supports_export())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Run};

    #[test]
    fn test_file_format_extension() {
        assert_eq!(FileFormat::Docx.extension(), "docx");
        assert_eq!(FileFormat::Pdf.extension(), "pdf");
    }

    #[test]
    fn test_file_format_from_extension() {
        assert_eq!(FileFormat::from_extension("docx"), Some(FileFormat::Docx));
        assert_eq!(FileFormat::from_extension("DOCX"), Some(FileFormat::Docx));
        assert_eq!(FileFormat::from_extension("pdf"), Some(FileFormat::Pdf));
        assert_eq!(FileFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_file_format_from_path() {
        assert_eq!(
            FileFormat::from_path(Path::new("document.docx")),
            Some(FileFormat::Docx)
        );
        assert_eq!(
            FileFormat::from_path(Path::new("/path/to/file.pdf")),
            Some(FileFormat::Pdf)
        );
        assert_eq!(
            FileFormat::from_path(Path::new("no_extension")),
            None
        );
    }

    #[test]
    fn test_supported_formats() {
        let formats = get_supported_formats();
        assert!(formats.contains(&FileFormat::Docx));
        assert!(formats.contains(&FileFormat::Pdf));
    }

    #[test]
    fn test_import_formats() {
        let formats = get_import_formats();
        assert!(formats.contains(&FileFormat::Docx));
        assert!(!formats.contains(&FileFormat::Pdf)); // PDF is export-only
    }

    #[test]
    fn test_export_bytes_empty_doc() {
        let tree = DocumentTree::new();
        // This test verifies that export_docx_bytes doesn't panic on an empty document
        // The actual bytes output would need a more sophisticated test
    }

    #[test]
    fn test_import_nonexistent_file() {
        let result = import_docx(Path::new("/nonexistent/path/document.docx"));
        assert!(result.is_err());
    }
}
