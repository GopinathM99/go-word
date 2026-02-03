//! PDF Export Options
//!
//! This module defines configuration options for PDF export.

use super::document::PdfVersion;
use super::pdfa::PdfAConformance;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Options for PDF export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfExportOptions {
    /// Document title
    #[serde(default)]
    pub title: Option<String>,
    /// Document author
    #[serde(default)]
    pub author: Option<String>,
    /// Document subject
    #[serde(default)]
    pub subject: Option<String>,
    /// Document keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Whether to compress content streams
    #[serde(default = "default_compress")]
    pub compress: bool,
    /// Whether to embed fonts
    #[serde(default = "default_embed_fonts")]
    pub embed_fonts: bool,
    /// PDF version
    #[serde(default)]
    pub pdf_version: PdfVersionOption,
    /// Page range to export (None = all pages)
    #[serde(default)]
    pub page_range: Option<PageRange>,
    /// Image quality for JPEG compression (0-100)
    #[serde(default = "default_image_quality")]
    pub image_quality: u8,
    /// Whether to include document outline/bookmarks
    #[serde(default = "default_include_outline")]
    pub include_outline: bool,
    /// Whether to include hyperlinks
    #[serde(default = "default_include_links")]
    pub include_links: bool,
    /// PDF/A conformance level (None for standard PDF)
    #[serde(default)]
    pub pdfa_conformance: PdfAConformance,
}

fn default_compress() -> bool {
    true
}

fn default_embed_fonts() -> bool {
    false
}

fn default_image_quality() -> u8 {
    85
}

fn default_include_outline() -> bool {
    true
}

fn default_include_links() -> bool {
    true
}

/// PDF version option for serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PdfVersionOption {
    /// PDF 1.4
    #[default]
    V14,
    /// PDF 1.5
    V15,
    /// PDF 1.7
    V17,
}

impl From<PdfVersionOption> for PdfVersion {
    fn from(opt: PdfVersionOption) -> Self {
        match opt {
            PdfVersionOption::V14 => PdfVersion::V1_4,
            PdfVersionOption::V15 => PdfVersion::V1_5,
            PdfVersionOption::V17 => PdfVersion::V1_7,
        }
    }
}

impl From<PdfVersion> for PdfVersionOption {
    fn from(ver: PdfVersion) -> Self {
        match ver {
            PdfVersion::V1_4 => PdfVersionOption::V14,
            PdfVersion::V1_5 => PdfVersionOption::V15,
            PdfVersion::V1_7 => PdfVersionOption::V17,
        }
    }
}

/// Page range for export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRange {
    /// First page (0-indexed)
    pub start: usize,
    /// Last page (exclusive, 0-indexed)
    pub end: usize,
}

impl PageRange {
    /// Create a new page range
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a page range for a single page
    pub fn single(page: usize) -> Self {
        Self {
            start: page,
            end: page + 1,
        }
    }

    /// Convert to a Rust Range
    pub fn to_range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Check if a page index is in range
    pub fn contains(&self, page: usize) -> bool {
        page >= self.start && page < self.end
    }
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            subject: None,
            keywords: Vec::new(),
            compress: true,
            embed_fonts: false,
            pdf_version: PdfVersionOption::default(),
            page_range: None,
            image_quality: 85,
            include_outline: true,
            include_links: true,
            pdfa_conformance: PdfAConformance::default(),
        }
    }
}

impl PdfExportOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the document title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the document author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the document subject
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Add a keyword
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    /// Set compression enabled/disabled
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Set font embedding enabled/disabled
    pub fn with_font_embedding(mut self, embed: bool) -> Self {
        self.embed_fonts = embed;
        self
    }

    /// Set the PDF version
    pub fn with_version(mut self, version: PdfVersionOption) -> Self {
        self.pdf_version = version;
        self
    }

    /// Set the page range
    pub fn with_page_range(mut self, range: PageRange) -> Self {
        self.page_range = Some(range);
        self
    }

    /// Set image quality
    pub fn with_image_quality(mut self, quality: u8) -> Self {
        self.image_quality = quality.min(100);
        self
    }

    /// Set PDF/A conformance level
    pub fn with_pdfa_conformance(mut self, conformance: PdfAConformance) -> Self {
        self.pdfa_conformance = conformance;
        // PDF/A requires font embedding
        if conformance.requires_font_embedding() {
            self.embed_fonts = true;
        }
        // PDF/A-2b requires PDF 1.7
        if matches!(conformance, PdfAConformance::PdfA2b) {
            self.pdf_version = PdfVersionOption::V17;
        }
        self
    }

    /// Check if PDF/A compliance is enabled
    pub fn is_pdfa(&self) -> bool {
        self.pdfa_conformance.is_pdfa()
    }

    /// Get the effective PDF version (considering PDF/A requirements)
    pub fn effective_pdf_version(&self) -> PdfVersion {
        if self.pdfa_conformance.is_pdfa() {
            self.pdfa_conformance.required_pdf_version()
        } else {
            self.pdf_version.into()
        }
    }

    /// Check if a page should be included based on page range
    pub fn should_include_page(&self, page_index: usize) -> bool {
        match &self.page_range {
            Some(range) => range.contains(page_index),
            None => true,
        }
    }

    /// Get the effective page range for a given total page count
    pub fn effective_page_range(&self, total_pages: usize) -> Range<usize> {
        match &self.page_range {
            Some(range) => {
                let start = range.start.min(total_pages);
                let end = range.end.min(total_pages);
                start..end
            }
            None => 0..total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = PdfExportOptions::default();
        assert!(opts.compress);
        assert!(!opts.embed_fonts);
        assert!(opts.title.is_none());
        assert_eq!(opts.pdf_version, PdfVersionOption::V14);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = PdfExportOptions::new()
            .with_title("Test Document")
            .with_author("Test Author")
            .with_compression(false)
            .with_version(PdfVersionOption::V17);

        assert_eq!(opts.title, Some("Test Document".to_string()));
        assert_eq!(opts.author, Some("Test Author".to_string()));
        assert!(!opts.compress);
        assert_eq!(opts.pdf_version, PdfVersionOption::V17);
    }

    #[test]
    fn test_page_range() {
        let range = PageRange::new(0, 5);
        assert!(range.contains(0));
        assert!(range.contains(4));
        assert!(!range.contains(5));
    }

    #[test]
    fn test_single_page_range() {
        let range = PageRange::single(3);
        assert!(!range.contains(2));
        assert!(range.contains(3));
        assert!(!range.contains(4));
    }

    #[test]
    fn test_effective_page_range() {
        let opts = PdfExportOptions::default();
        assert_eq!(opts.effective_page_range(10), 0..10);

        let opts = opts.with_page_range(PageRange::new(2, 7));
        assert_eq!(opts.effective_page_range(10), 2..7);
        assert_eq!(opts.effective_page_range(5), 2..5); // Clamped
    }

    #[test]
    fn test_serialization() {
        let opts = PdfExportOptions::new()
            .with_title("Test")
            .with_page_range(PageRange::new(0, 5));

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: PdfExportOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.title, opts.title);
        assert!(parsed.page_range.is_some());
    }
}
