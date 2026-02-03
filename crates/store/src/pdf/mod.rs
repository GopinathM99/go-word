//! PDF Export Module
//!
//! This module provides PDF generation functionality for the MS Word clone.
//! It converts the render model (RenderPage, RenderItem) into valid PDF files.
//!
//! # Architecture
//!
//! - `objects`: PDF object model (Dictionary, Array, Stream, Reference)
//! - `document`: PDF document structure (Catalog, Pages, Resources)
//! - `content`: Content stream generation (text, graphics operators)
//! - `fonts`: Font handling and embedding
//! - `images`: Image XObject generation
//! - `renderer`: Converts RenderPage to PDF
//! - `options`: PDF export configuration
//! - `pdfa`: PDF/A compliance support (PDF/A-1b, PDF/A-2b)
//! - `api`: Public API for PDF export

mod api;
mod content;
mod document;
mod fonts;
mod images;
mod objects;
mod options;
pub mod pdfa;
mod renderer;
mod writer;

pub use api::*;
pub use options::*;
pub use pdfa::{
    ComplianceIssue, ComplianceReport, IssueCategory, IssueSeverity,
    PdfAConformance, PdfAError, PdfAValidator, XmpMetadata,
    get_iso_date,
};

// Re-export error type
pub use writer::PdfError;

#[cfg(test)]
mod tests;
