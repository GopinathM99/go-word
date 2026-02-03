//! PDF Export Public API
//!
//! This module provides the public API for PDF export functionality.

use super::options::PdfExportOptions;
use super::pdfa::{ComplianceReport, PdfAConformance, PdfAValidator};
use super::renderer::{convert, PageRenderInfo, PdfRenderer};
use super::writer::{PdfDocumentWriter, PdfError, Result};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Export render pages to a PDF file
///
/// # Arguments
///
/// * `pages` - The render pages to export
/// * `path` - The file path to write the PDF to
/// * `options` - Export options
///
/// # Example
///
/// ```ignore
/// use store::pdf::{export_pdf, PdfExportOptions};
///
/// let pages = vec![/* render pages */];
/// let options = PdfExportOptions::new()
///     .with_title("My Document")
///     .with_author("Author Name");
///
/// export_pdf(&pages, "output.pdf", options)?;
/// ```
pub fn export_pdf(
    pages: &[render_model::PageRender],
    path: impl AsRef<Path>,
    options: PdfExportOptions,
) -> Result<()> {
    // Convert render pages to PDF page info
    let page_infos: Vec<PageRenderInfo> = pages.iter().map(convert::convert_page).collect();

    // Open file for writing
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Write PDF
    let doc_writer = PdfDocumentWriter::new(options);
    doc_writer.write(&page_infos, writer)?;

    Ok(())
}

/// Export render pages to PDF bytes in memory
///
/// # Arguments
///
/// * `pages` - The render pages to export
/// * `options` - Export options
///
/// # Returns
///
/// The PDF file contents as a byte vector
///
/// # Example
///
/// ```ignore
/// use store::pdf::{export_pdf_bytes, PdfExportOptions};
///
/// let pages = vec![/* render pages */];
/// let options = PdfExportOptions::default();
///
/// let pdf_bytes = export_pdf_bytes(&pages, options)?;
/// ```
pub fn export_pdf_bytes(
    pages: &[render_model::PageRender],
    options: PdfExportOptions,
) -> Result<Vec<u8>> {
    // Convert render pages to PDF page info
    let page_infos: Vec<PageRenderInfo> = pages.iter().map(convert::convert_page).collect();

    // Write PDF to memory
    let doc_writer = PdfDocumentWriter::new(options);
    doc_writer.write_to_bytes(&page_infos)
}

/// Export a single page to PDF bytes
///
/// Convenience function for exporting a single page.
pub fn export_single_page(
    page: &render_model::PageRender,
    options: PdfExportOptions,
) -> Result<Vec<u8>> {
    export_pdf_bytes(&[page.clone()], options)
}

/// Get default PDF export options
pub fn default_options() -> PdfExportOptions {
    PdfExportOptions::default()
}

/// Validate that pages can be exported to PDF
///
/// Returns Ok if the pages are valid for export, or an error describing the problem.
pub fn validate_pages(pages: &[render_model::PageRender]) -> Result<()> {
    if pages.is_empty() {
        return Err(PdfError::InvalidDocument("No pages to export".to_string()));
    }

    for (i, page) in pages.iter().enumerate() {
        if page.width <= 0.0 {
            return Err(PdfError::InvalidDocument(format!(
                "Page {} has invalid width: {}",
                i, page.width
            )));
        }
        if page.height <= 0.0 {
            return Err(PdfError::InvalidDocument(format!(
                "Page {} has invalid height: {}",
                i, page.height
            )));
        }
    }

    Ok(())
}

// =============================================================================
// PDF/A Export Functions
// =============================================================================

/// Export render pages to a PDF/A compliant file
///
/// # Arguments
///
/// * `pages` - The render pages to export
/// * `path` - The file path to write the PDF to
/// * `conformance` - The PDF/A conformance level ("1b" or "2b")
/// * `options` - Additional export options (title, author, etc.)
///
/// # Example
///
/// ```ignore
/// use store::pdf::{export_pdf_a, PdfExportOptions, PdfAConformance};
///
/// let pages = vec![/* render pages */];
/// let options = PdfExportOptions::new()
///     .with_title("Archived Document")
///     .with_author("Author Name");
///
/// export_pdf_a(&pages, "output.pdf", PdfAConformance::PdfA1b, options)?;
/// ```
pub fn export_pdf_a(
    pages: &[render_model::PageRender],
    path: impl AsRef<Path>,
    conformance: PdfAConformance,
    mut options: PdfExportOptions,
) -> Result<()> {
    // Set the conformance level (this also enables font embedding)
    options = options.with_pdfa_conformance(conformance);

    // Export using standard function with updated options
    export_pdf(pages, path, options)
}

/// Export render pages to PDF/A bytes in memory
///
/// # Arguments
///
/// * `pages` - The render pages to export
/// * `conformance` - The PDF/A conformance level
/// * `options` - Additional export options
///
/// # Returns
///
/// The PDF/A file contents as a byte vector
pub fn export_pdf_a_bytes(
    pages: &[render_model::PageRender],
    conformance: PdfAConformance,
    mut options: PdfExportOptions,
) -> Result<Vec<u8>> {
    // Set the conformance level
    options = options.with_pdfa_conformance(conformance);

    // Export using standard function with updated options
    export_pdf_bytes(pages, options)
}

/// Validate pages for PDF/A compliance
///
/// Checks if the given pages can be exported as PDF/A compliant
/// and returns a detailed compliance report.
///
/// # Arguments
///
/// * `pages` - The render pages to validate
/// * `conformance` - The target PDF/A conformance level
///
/// # Returns
///
/// A compliance report with any issues found
///
/// # Example
///
/// ```ignore
/// use store::pdf::{validate_pdf_a_compliance, PdfAConformance};
///
/// let pages = vec![/* render pages */];
/// let report = validate_pdf_a_compliance(&pages, PdfAConformance::PdfA1b);
///
/// if !report.is_compliant {
///     for issue in report.issues {
///         println!("Issue: {}", issue.description);
///     }
/// }
/// ```
pub fn validate_pdf_a_compliance(
    pages: &[render_model::PageRender],
    conformance: PdfAConformance,
) -> ComplianceReport {
    let mut validator = PdfAValidator::new(conformance);

    // Analyze pages for compliance issues
    for page in pages {
        for item in &page.items {
            analyze_render_item_for_compliance(&mut validator, item);
        }
    }

    validator.validate()
}

/// Analyze a render item for PDF/A compliance issues
fn analyze_render_item_for_compliance(
    validator: &mut PdfAValidator,
    item: &render_model::RenderItem,
) {
    match item {
        render_model::RenderItem::GlyphRun(glyph) => {
            // Track font usage (standard fonts are not embedded by default)
            validator.add_font(&glyph.font_family, false);
            validator.add_color_space("DeviceRGB");
        }
        render_model::RenderItem::Rectangle { fill, stroke, .. } => {
            if fill.is_some() {
                validator.add_color_space("DeviceRGB");
            }
            if stroke.is_some() {
                validator.add_color_space("DeviceRGB");
            }
        }
        render_model::RenderItem::Line { .. } => {
            validator.add_color_space("DeviceRGB");
        }
        render_model::RenderItem::Image(_) => {
            validator.add_color_space("DeviceRGB");
        }
        render_model::RenderItem::Shape(shape) => {
            // Check for transparency
            if let Some(render_model::ShapeFillRender::Gradient { .. }) = &shape.fill {
                validator.set_has_transparency(true);
            }
            validator.add_color_space("DeviceRGB");
        }
        // UI elements are not exported to PDF
        render_model::RenderItem::Caret { .. }
        | render_model::RenderItem::Selection { .. }
        | render_model::RenderItem::Squiggly(_)
        | render_model::RenderItem::FindHighlight { .. } => {}
        render_model::RenderItem::TableBorder(_) | render_model::RenderItem::TableCell(_) => {
            validator.add_color_space("DeviceRGB");
        }
        render_model::RenderItem::TextBox(textbox) => {
            // Text boxes use RGB colors
            validator.add_color_space("DeviceRGB");
            // Check for transparency in fill
            if let Some(render_model::TextBoxFillRender::Gradient { .. }) = &textbox.fill {
                validator.set_has_transparency(true);
            }
        }
        render_model::RenderItem::LineNumber(info) => {
            // Line numbers are rendered as text with a font
            validator.add_font(&info.font_family, false);
            validator.add_color_space("DeviceRGB");
        }
    }
}

/// Get PDF/A conformance level from string
///
/// Parses strings like "1b", "2b", "PDF/A-1b", "PDF/A-2b"
pub fn parse_pdfa_conformance(s: &str) -> Option<PdfAConformance> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "1b" | "a-1b" | "pdfa-1b" | "pdf/a-1b" => Some(PdfAConformance::PdfA1b),
        "2b" | "a-2b" | "pdfa-2b" | "pdf/a-2b" => Some(PdfAConformance::PdfA2b),
        "none" | "" => Some(PdfAConformance::None),
        _ => None,
    }
}

/// Check if a PDF/A conformance level is supported
pub fn is_pdfa_conformance_supported(conformance: PdfAConformance) -> bool {
    matches!(
        conformance,
        PdfAConformance::None | PdfAConformance::PdfA1b | PdfAConformance::PdfA2b
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_model::{Color, GlyphRun, PageRender, RenderItem};

    fn create_test_page() -> PageRender {
        PageRender {
            page_index: 0,
            width: 612.0,
            height: 792.0,
            items: vec![RenderItem::GlyphRun(GlyphRun {
                text: "Hello, World!".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 12.0,
                bold: false,
                italic: false,
                underline: false,
                color: Color::BLACK,
                x: 72.0,
                y: 720.0,
                hyperlink: None,
            })],
        }
    }

    #[test]
    fn test_export_pdf_bytes() {
        let pages = vec![create_test_page()];
        let options = PdfExportOptions::default();

        let result = export_pdf_bytes(&pages, options);
        assert!(result.is_ok());

        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());

        // Verify it starts with PDF header
        assert!(pdf_bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_export_with_options() {
        let pages = vec![create_test_page()];
        let options = PdfExportOptions::new()
            .with_title("Test Document")
            .with_author("Test Author")
            .with_compression(false);

        let result = export_pdf_bytes(&pages, options);
        assert!(result.is_ok());

        let pdf_bytes = result.unwrap();
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.contains("Test Document"));
        assert!(pdf_str.contains("Test Author"));
    }

    #[test]
    fn test_export_single_page() {
        let page = create_test_page();
        let options = PdfExportOptions::default();

        let result = export_single_page(&page, options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pages_empty() {
        let result = validate_pages(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pages_invalid_dimensions() {
        let page = PageRender {
            page_index: 0,
            width: 0.0,
            height: 792.0,
            items: vec![],
        };

        let result = validate_pages(&[page]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pages_valid() {
        let pages = vec![create_test_page()];
        let result = validate_pages(&pages);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_options() {
        let opts = default_options();
        assert!(opts.compress);
        assert!(!opts.embed_fonts);
        assert!(opts.title.is_none());
    }

    #[test]
    fn test_export_multiple_pages() {
        let pages = vec![
            create_test_page(),
            PageRender {
                page_index: 1,
                width: 612.0,
                height: 792.0,
                items: vec![RenderItem::GlyphRun(GlyphRun {
                    text: "Page 2".to_string(),
                    font_family: "Helvetica".to_string(),
                    font_size: 12.0,
                    bold: false,
                    italic: false,
                    underline: false,
                    color: Color::BLACK,
                    x: 72.0,
                    y: 720.0,
                    hyperlink: None,
                })],
            },
        ];

        let options = PdfExportOptions::default();
        let result = export_pdf_bytes(&pages, options);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.contains("/Count 2"));
    }

    #[test]
    fn test_export_with_graphics() {
        let page = PageRender {
            page_index: 0,
            width: 612.0,
            height: 792.0,
            items: vec![
                RenderItem::Rectangle {
                    bounds: render_model::Rect::new(100.0, 100.0, 200.0, 50.0),
                    fill: Some(Color::rgb(255, 0, 0)),
                    stroke: Some(Color::BLACK),
                    stroke_width: 1.0,
                },
                RenderItem::Line {
                    x1: 50.0,
                    y1: 50.0,
                    x2: 200.0,
                    y2: 200.0,
                    color: Color::BLACK,
                    width: 2.0,
                },
            ],
        };

        let options = PdfExportOptions::new().with_compression(false);
        let result = export_pdf_bytes(&[page], options);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.contains("re")); // Rectangle operator
    }
}
