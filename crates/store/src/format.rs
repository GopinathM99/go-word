//! Internal document format specification

use serde::{Deserialize, Serialize};

/// File format version
pub const FORMAT_VERSION: u32 = 1;

/// File extension for the internal format
pub const FILE_EXTENSION: &str = "wdj";

/// File header for format identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHeader {
    /// Magic string for format identification
    pub magic: String,
    /// Format version
    pub version: u32,
    /// Document ID
    pub document_id: String,
    /// Creation timestamp (ISO 8601)
    pub created: String,
    /// Last modified timestamp (ISO 8601)
    pub modified: String,
}

impl FileHeader {
    pub const MAGIC: &'static str = "MSWORD-DOC";

    pub fn new(document_id: impl Into<String>) -> Self {
        let now = chrono_lite::now_iso8601();
        Self {
            magic: Self::MAGIC.to_string(),
            version: FORMAT_VERSION,
            document_id: document_id.into(),
            created: now.clone(),
            modified: now,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC && self.version <= FORMAT_VERSION
    }
}

/// Simple timestamp helper (avoiding chrono dependency)
mod chrono_lite {
    pub fn now_iso8601() -> String {
        // In production, this would use proper time libraries
        "2024-01-01T00:00:00Z".to_string()
    }
}

/// Complete file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFile {
    pub header: FileHeader,
    pub document: doc_model::DocumentTree,
}

impl DocumentFile {
    pub fn new(document: doc_model::DocumentTree) -> Self {
        Self {
            header: FileHeader::new(document.root_id().to_string()),
            document,
        }
    }
}
