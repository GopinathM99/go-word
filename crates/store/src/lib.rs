//! Store - Persistence, autosave, and file I/O
//!
//! This crate handles document serialization, file operations,
//! autosave functionality, recovery, integrity checking, version tracking,
//! application settings, PDF export, DOCX import/export, RTF import/export,
//! ODT import, and templates.

mod format;
mod serializer;
mod file_io;
mod autosave;
mod recovery;
mod integrity;
mod versions;
mod error;
mod settings;
mod image_store;
pub mod pdf;
pub mod docx;
pub mod rtf;
pub mod odt;
pub mod templates;

pub use format::*;
pub use serializer::*;
pub use file_io::*;
pub use autosave::*;
pub use recovery::*;
pub use integrity::*;
pub use versions::*;
pub use error::*;
pub use settings::*;

// Re-export image store types explicitly to avoid Result conflict
pub use image_store::{
    ImageData, ImageFormat, ImageStore, ImageStoreConfig, ImageStoreError,
};

// Re-export DOCX functionality
pub use docx::{
    import_docx, export_docx, import_docx_bytes, export_docx_bytes,
    DocxError, DocxResult,
};

// Re-export RTF functionality
pub use rtf::{
    import_rtf, export_rtf, import_rtf_bytes, export_rtf_bytes,
    RtfError, RtfResult, ImportResult as RtfImportResult, ImportWarning as RtfImportWarning,
    WarningKind as RtfWarningKind,
};

// Re-export ODT functionality (read-only)
pub use odt::{
    import_odt, import_odt_bytes,
    OdtError, OdtResult, OdtImportResult, OdtWarning, OdtWarningKind,
};

// Re-export template functionality
pub use templates::{
    LockedRegion, LockedRegionManager, StylePack, StylePackApplyOptions,
    TemplateCategory, TemplateError, TemplateManager, TemplateMetadata,
    TemplatePackage, TemplateResult, TemplateSummary,
    TEMPLATE_EXTENSION, read_metadata as read_template_metadata,
    read_thumbnail as read_template_thumbnail,
};
