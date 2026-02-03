//! Public API for ODT import (read-only)
//!
//! This module provides the main entry points for reading ODT files.
//! Note: ODT export is not supported.

use crate::odt::error::{OdtError, OdtResult};
use crate::odt::reader::OdtReader;
use doc_model::DocumentTree;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

/// Warning about an unsupported or partially supported feature
#[derive(Debug, Clone)]
pub struct OdtWarning {
    /// Kind of warning
    pub kind: OdtWarningKind,
    /// Description of the issue
    pub message: String,
}

/// Types of import warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdtWarningKind {
    /// Feature is not supported and was skipped
    UnsupportedFeature,
    /// Feature is partially supported, may not render correctly
    PartialSupport,
    /// Data was lost during conversion
    DataLoss,
    /// Unknown or invalid element encountered
    UnknownElement,
    /// Style could not be resolved
    StyleNotFound,
}

impl std::fmt::Display for OdtWarningKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OdtWarningKind::UnsupportedFeature => write!(f, "Unsupported feature"),
            OdtWarningKind::PartialSupport => write!(f, "Partial support"),
            OdtWarningKind::DataLoss => write!(f, "Data loss"),
            OdtWarningKind::UnknownElement => write!(f, "Unknown element"),
            OdtWarningKind::StyleNotFound => write!(f, "Style not found"),
        }
    }
}

impl std::fmt::Display for OdtWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

/// Result of importing an ODT file
#[derive(Debug)]
pub struct OdtImportResult {
    /// The imported document tree
    pub tree: DocumentTree,
    /// Warnings encountered during import
    pub warnings: Vec<OdtWarning>,
}

impl OdtImportResult {
    /// Check if there were any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get warnings of a specific kind
    pub fn warnings_of_kind(&self, kind: OdtWarningKind) -> Vec<&OdtWarning> {
        self.warnings.iter().filter(|w| w.kind == kind).collect()
    }
}

/// Import an ODT file from disk
///
/// # Arguments
///
/// * `path` - Path to the ODT file
///
/// # Returns
///
/// * `Ok(OdtImportResult)` - The imported document with any warnings
/// * `Err(OdtError)` - If parsing fails
///
/// # Example
///
/// ```ignore
/// use store::odt::import_odt;
/// use std::path::Path;
///
/// let result = import_odt(Path::new("document.odt"))?;
/// println!("Imported document with {} warnings", result.warning_count());
/// ```
///
/// # Note
///
/// ODT export is not supported. This is a read-only import function.
pub fn import_odt(path: &Path) -> OdtResult<OdtImportResult> {
    // Open the file
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            OdtError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ))
        } else {
            OdtError::Io(e)
        }
    })?;

    let reader = BufReader::new(file);

    // Create ODT reader and parse
    let odt_reader = OdtReader::new(reader)?;
    let (tree, warnings) = odt_reader.parse()?;

    Ok(OdtImportResult { tree, warnings })
}

/// Import ODT from an in-memory byte slice
///
/// # Arguments
///
/// * `bytes` - The ODT file content as bytes
///
/// # Returns
///
/// * `Ok(OdtImportResult)` - The imported document with any warnings
/// * `Err(OdtError)` - If parsing fails
///
/// # Example
///
/// ```ignore
/// use store::odt::import_odt_bytes;
///
/// let odt_data: Vec<u8> = std::fs::read("document.odt")?;
/// let result = import_odt_bytes(&odt_data)?;
/// ```
///
/// # Note
///
/// ODT export is not supported. This is a read-only import function.
pub fn import_odt_bytes(bytes: &[u8]) -> OdtResult<OdtImportResult> {
    let cursor = Cursor::new(bytes);
    let odt_reader = OdtReader::new(cursor)?;
    let (tree, warnings) = odt_reader.parse()?;

    Ok(OdtImportResult { tree, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_result_warnings() {
        let result = OdtImportResult {
            tree: DocumentTree::new(),
            warnings: vec![
                OdtWarning {
                    kind: OdtWarningKind::UnsupportedFeature,
                    message: "Test warning".to_string(),
                },
                OdtWarning {
                    kind: OdtWarningKind::DataLoss,
                    message: "Another warning".to_string(),
                },
            ],
        };

        assert!(result.has_warnings());
        assert_eq!(result.warning_count(), 2);
        assert_eq!(result.warnings_of_kind(OdtWarningKind::UnsupportedFeature).len(), 1);
    }

    #[test]
    fn test_import_nonexistent_file() {
        let result = import_odt(Path::new("/nonexistent/path/document.odt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_bytes() {
        // Invalid ZIP data
        let invalid_data = b"This is not an ODT file";
        let result = import_odt_bytes(invalid_data);
        assert!(result.is_err());
    }
}
