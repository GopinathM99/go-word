//! Public API for RTF import/export
//!
//! This module provides the main entry points for working with RTF files.

use crate::rtf::error::{RtfError, RtfResult};
use crate::rtf::parser::RtfParser;
use crate::rtf::writer::RtfWriter;
use doc_model::DocumentTree;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read};
use std::path::Path;

/// Warning about an unsupported or partially supported feature
#[derive(Debug, Clone)]
pub struct ImportWarning {
    /// Kind of warning
    pub kind: WarningKind,
    /// Description of the issue
    pub message: String,
}

/// Types of import warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    /// Feature is not supported and was skipped
    UnsupportedFeature,
    /// Feature is partially supported, may not render correctly
    PartialSupport,
    /// Data was lost during conversion
    DataLoss,
    /// Unknown or invalid element encountered
    UnknownElement,
    /// Encoding issue
    EncodingIssue,
}

impl std::fmt::Display for WarningKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarningKind::UnsupportedFeature => write!(f, "Unsupported feature"),
            WarningKind::PartialSupport => write!(f, "Partial support"),
            WarningKind::DataLoss => write!(f, "Data loss"),
            WarningKind::UnknownElement => write!(f, "Unknown element"),
            WarningKind::EncodingIssue => write!(f, "Encoding issue"),
        }
    }
}

impl std::fmt::Display for ImportWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

/// Result of importing an RTF file
#[derive(Debug)]
pub struct ImportResult {
    /// The imported document tree
    pub tree: DocumentTree,
    /// Warnings encountered during import
    pub warnings: Vec<ImportWarning>,
}

impl ImportResult {
    /// Check if there were any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get warnings of a specific kind
    pub fn warnings_of_kind(&self, kind: WarningKind) -> Vec<&ImportWarning> {
        self.warnings.iter().filter(|w| w.kind == kind).collect()
    }
}

/// Import an RTF file from disk
///
/// # Arguments
///
/// * `path` - Path to the RTF file
///
/// # Returns
///
/// * `Ok(ImportResult)` - The imported document with any warnings
/// * `Err(RtfError)` - If parsing fails
///
/// # Example
///
/// ```ignore
/// use store::rtf::import_rtf;
/// use std::path::Path;
///
/// let result = import_rtf(Path::new("document.rtf"))?;
/// println!("Imported document with {} warnings", result.warning_count());
/// ```
pub fn import_rtf(path: &Path) -> RtfResult<ImportResult> {
    // Open the file
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RtfError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ))
        } else {
            RtfError::Io(e)
        }
    })?;

    // Read entire file into memory
    let mut reader = BufReader::new(file);
    let mut content = Vec::new();
    reader.read_to_end(&mut content)?;

    // Parse the RTF
    let mut parser = RtfParser::new();
    let (tree, warnings) = parser.parse(&content)?;

    Ok(ImportResult { tree, warnings })
}

/// Export a DocumentTree to an RTF file
///
/// # Arguments
///
/// * `tree` - The document tree to export
/// * `path` - Path where the RTF file will be saved
///
/// # Returns
///
/// * `Ok(())` - If export succeeds
/// * `Err(RtfError)` - If export fails
///
/// # Example
///
/// ```ignore
/// use store::rtf::export_rtf;
/// use doc_model::DocumentTree;
/// use std::path::Path;
///
/// let tree = DocumentTree::new();
/// export_rtf(&tree, Path::new("output.rtf"))?;
/// ```
pub fn export_rtf(tree: &DocumentTree, path: &Path) -> RtfResult<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create the output file
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Write the RTF
    let rtf_writer = RtfWriter::new(writer);
    rtf_writer.write(tree)
}

/// Import RTF from an in-memory byte slice
///
/// # Arguments
///
/// * `bytes` - The RTF file content as bytes
///
/// # Returns
///
/// * `Ok(ImportResult)` - The imported document with any warnings
/// * `Err(RtfError)` - If parsing fails
///
/// # Example
///
/// ```ignore
/// use store::rtf::import_rtf_bytes;
///
/// let rtf_data: Vec<u8> = std::fs::read("document.rtf")?;
/// let result = import_rtf_bytes(&rtf_data)?;
/// ```
pub fn import_rtf_bytes(bytes: &[u8]) -> RtfResult<ImportResult> {
    let mut parser = RtfParser::new();
    let (tree, warnings) = parser.parse(bytes)?;
    Ok(ImportResult { tree, warnings })
}

/// Export a DocumentTree to an in-memory byte vector
///
/// # Arguments
///
/// * `tree` - The document tree to export
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The RTF file content as bytes
/// * `Err(RtfError)` - If export fails
///
/// # Example
///
/// ```ignore
/// use store::rtf::export_rtf_bytes;
/// use doc_model::DocumentTree;
///
/// let tree = DocumentTree::new();
/// let bytes = export_rtf_bytes(&tree)?;
/// std::fs::write("output.rtf", bytes)?;
/// ```
pub fn export_rtf_bytes(tree: &DocumentTree) -> RtfResult<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let writer = Cursor::new(&mut buffer);
        let rtf_writer = RtfWriter::new(writer);
        rtf_writer.write(tree)?;
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Node, Paragraph, Run};

    #[test]
    fn test_import_result_warnings() {
        let result = ImportResult {
            tree: DocumentTree::new(),
            warnings: vec![
                ImportWarning {
                    kind: WarningKind::UnsupportedFeature,
                    message: "Test warning".to_string(),
                },
                ImportWarning {
                    kind: WarningKind::DataLoss,
                    message: "Another warning".to_string(),
                },
            ],
        };

        assert!(result.has_warnings());
        assert_eq!(result.warning_count(), 2);
        assert_eq!(result.warnings_of_kind(WarningKind::UnsupportedFeature).len(), 1);
    }

    #[test]
    fn test_roundtrip_simple() {
        let mut tree = DocumentTree::new();

        let mut para = Paragraph::new();
        let para_id = para.id();

        let mut run = Run::new("Test content");
        let run_id = run.id();
        run.set_parent(Some(para_id));

        para.add_child(run_id);
        tree.nodes.runs.insert(run_id, run);
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        // Export to bytes
        let bytes = export_rtf_bytes(&tree).unwrap();

        // Should be valid RTF
        let rtf_str = String::from_utf8_lossy(&bytes);
        assert!(rtf_str.starts_with("{\\rtf1"));
        assert!(rtf_str.contains("Test content"));

        // Import back
        let result = import_rtf_bytes(&bytes).unwrap();
        let text = result.tree.text_content();
        assert!(text.contains("Test content"));
    }

    #[test]
    fn test_import_nonexistent_file() {
        let result = import_rtf(Path::new("/nonexistent/path/document.rtf"));
        assert!(result.is_err());
    }
}
