//! Comprehensive tests for PDF export functionality

use super::*;
use render_model::{Color, GlyphRun, PageRender, Rect, RenderItem};

// Helper to create a basic test page
fn create_basic_page() -> PageRender {
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

// Helper to create a page with various content
fn create_complex_page() -> PageRender {
    PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![
            // Text
            RenderItem::GlyphRun(GlyphRun {
                text: "Title".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 24.0,
                bold: true,
                italic: false,
                underline: false,
                color: Color::BLACK,
                x: 72.0,
                y: 72.0,
                hyperlink: None,
            }),
            RenderItem::GlyphRun(GlyphRun {
                text: "Body text paragraph.".to_string(),
                font_family: "Times New Roman".to_string(),
                font_size: 12.0,
                bold: false,
                italic: false,
                underline: false,
                color: Color::rgb(0, 0, 128),
                x: 72.0,
                y: 120.0,
                hyperlink: None,
            }),
            // Rectangle
            RenderItem::Rectangle {
                bounds: Rect::new(72.0, 200.0, 200.0, 100.0),
                fill: Some(Color::rgb(200, 200, 200)),
                stroke: Some(Color::BLACK),
                stroke_width: 1.0,
            },
            // Line
            RenderItem::Line {
                x1: 72.0,
                y1: 350.0,
                x2: 540.0,
                y2: 350.0,
                color: Color::BLACK,
                width: 0.5,
            },
        ],
    }
}

#[test]
fn test_export_basic_pdf() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::default();

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_export_complex_pdf() {
    let pages = vec![create_complex_page()];
    let options = PdfExportOptions::new().with_compression(false);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Verify structure
    assert!(pdf_str.contains("/Type /Catalog"));
    assert!(pdf_str.contains("/Type /Pages"));
    assert!(pdf_str.contains("/Type /Page"));
    assert!(pdf_str.contains("%%EOF"));
}

#[test]
fn test_pdf_with_all_metadata() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new()
        .with_title("Test Document")
        .with_author("Test Author")
        .with_subject("Test Subject")
        .with_keyword("keyword1")
        .with_keyword("keyword2");

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.contains("Test Document"));
    assert!(pdf_str.contains("Test Author"));
    assert!(pdf_str.contains("Test Subject"));
    assert!(pdf_str.contains("keyword1"));
}

#[test]
fn test_pdf_multiple_pages() {
    let pages = vec![
        create_basic_page(),
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
        PageRender {
            page_index: 2,
            width: 612.0,
            height: 792.0,
            items: vec![RenderItem::GlyphRun(GlyphRun {
                text: "Page 3".to_string(),
                font_family: "Courier".to_string(),
                font_size: 10.0,
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
    assert!(pdf_str.contains("/Count 3"));
}

#[test]
fn test_pdf_compression() {
    let pages = vec![create_basic_page()];

    // With compression
    let compressed = export_pdf_bytes(&pages, PdfExportOptions::new().with_compression(true))
        .unwrap();

    // Without compression
    let uncompressed = export_pdf_bytes(&pages, PdfExportOptions::new().with_compression(false))
        .unwrap();

    // Compressed should typically be smaller or equal
    // (for very small content, compression overhead might make it larger)
    // The key test is that both produce valid PDFs
    assert!(!compressed.is_empty());
    assert!(!uncompressed.is_empty());

    // Uncompressed should contain readable content stream
    let uncompressed_str = String::from_utf8_lossy(&uncompressed);
    assert!(uncompressed_str.contains("BT")); // Begin text
    assert!(uncompressed_str.contains("ET")); // End text
}

#[test]
fn test_pdf_different_page_sizes() {
    let pages = vec![
        // Letter
        PageRender {
            page_index: 0,
            width: 612.0,
            height: 792.0,
            items: vec![],
        },
        // A4
        PageRender {
            page_index: 1,
            width: 595.0,
            height: 842.0,
            items: vec![],
        },
        // Custom size
        PageRender {
            page_index: 2,
            width: 400.0,
            height: 600.0,
            items: vec![],
        },
    ];

    let options = PdfExportOptions::default();
    let result = export_pdf_bytes(&pages, options);

    assert!(result.is_ok());
    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Each page should have its own MediaBox
    assert!(pdf_str.contains("612")); // Letter width
    assert!(pdf_str.contains("842")); // A4 height
    assert!(pdf_str.contains("400")); // Custom width
}

#[test]
fn test_pdf_font_variants() {
    let page = PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![
            // Normal
            RenderItem::GlyphRun(GlyphRun {
                text: "Normal".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 12.0,
                bold: false,
                italic: false,
                underline: false,
                color: Color::BLACK,
                x: 72.0,
                y: 100.0,
                hyperlink: None,
            }),
            // Bold
            RenderItem::GlyphRun(GlyphRun {
                text: "Bold".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 12.0,
                bold: true,
                italic: false,
                underline: false,
                color: Color::BLACK,
                x: 72.0,
                y: 120.0,
                hyperlink: None,
            }),
            // Italic
            RenderItem::GlyphRun(GlyphRun {
                text: "Italic".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 12.0,
                bold: false,
                italic: true,
                underline: false,
                color: Color::BLACK,
                x: 72.0,
                y: 140.0,
                hyperlink: None,
            }),
            // Bold Italic
            RenderItem::GlyphRun(GlyphRun {
                text: "Bold Italic".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 12.0,
                bold: true,
                italic: true,
                underline: false,
                color: Color::BLACK,
                x: 72.0,
                y: 160.0,
                hyperlink: None,
            }),
        ],
    };

    let options = PdfExportOptions::new().with_compression(false);
    let result = export_pdf_bytes(&[page], options);

    assert!(result.is_ok());
    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Should have multiple font resources
    assert!(pdf_str.contains("/Font"));
}

#[test]
fn test_pdf_colors() {
    let page = PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![
            // Red text
            RenderItem::GlyphRun(GlyphRun {
                text: "Red".to_string(),
                font_family: "Helvetica".to_string(),
                font_size: 12.0,
                bold: false,
                italic: false,
                underline: false,
                color: Color::rgb(255, 0, 0),
                x: 72.0,
                y: 100.0,
                hyperlink: None,
            }),
            // Green rectangle
            RenderItem::Rectangle {
                bounds: Rect::new(72.0, 150.0, 100.0, 50.0),
                fill: Some(Color::rgb(0, 255, 0)),
                stroke: None,
                stroke_width: 0.0,
            },
            // Blue line
            RenderItem::Line {
                x1: 72.0,
                y1: 250.0,
                x2: 200.0,
                y2: 250.0,
                color: Color::rgb(0, 0, 255),
                width: 2.0,
            },
        ],
    };

    let options = PdfExportOptions::new().with_compression(false);
    let result = export_pdf_bytes(&[page], options);

    assert!(result.is_ok());
    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Should contain RGB color operators
    assert!(pdf_str.contains("rg") || pdf_str.contains("RG"));
}

#[test]
fn test_pdf_empty_page() {
    let page = PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    };

    let options = PdfExportOptions::default();
    let result = export_pdf_bytes(&[page], options);

    assert!(result.is_ok());
}

#[test]
fn test_pdf_no_pages_error() {
    let result = export_pdf_bytes(&[], PdfExportOptions::default());
    assert!(result.is_err());
}

#[test]
fn test_pdf_version_14() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new().with_version(PdfVersionOption::V14);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.starts_with("%PDF-1.4"));
}

#[test]
fn test_pdf_version_17() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new().with_version(PdfVersionOption::V17);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.starts_with("%PDF-1.7"));
}

#[test]
fn test_validate_pages() {
    // Valid pages
    let valid_pages = vec![create_basic_page()];
    assert!(validate_pages(&valid_pages).is_ok());

    // Empty pages
    assert!(validate_pages(&[]).is_err());

    // Invalid width
    let invalid_width = vec![PageRender {
        page_index: 0,
        width: 0.0,
        height: 792.0,
        items: vec![],
    }];
    assert!(validate_pages(&invalid_width).is_err());

    // Invalid height
    let invalid_height = vec![PageRender {
        page_index: 0,
        width: 612.0,
        height: -100.0,
        items: vec![],
    }];
    assert!(validate_pages(&invalid_height).is_err());
}

#[test]
fn test_page_range_option() {
    let pages = vec![
        create_basic_page(),
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
        PageRender {
            page_index: 2,
            width: 612.0,
            height: 792.0,
            items: vec![RenderItem::GlyphRun(GlyphRun {
                text: "Page 3".to_string(),
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

    // Export only pages 1-2 (0-indexed)
    let options = PdfExportOptions::new()
        .with_page_range(PageRange::new(0, 2))
        .with_compression(false);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    // Should have only 2 pages
    assert!(pdf_str.contains("/Count 2"));
}

#[test]
fn test_options_serialization() {
    let options = PdfExportOptions::new()
        .with_title("Test")
        .with_author("Author")
        .with_compression(false)
        .with_version(PdfVersionOption::V17)
        .with_page_range(PageRange::new(0, 5));

    // Serialize to JSON
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("\"title\""));
    assert!(json.contains("\"author\""));

    // Deserialize
    let parsed: PdfExportOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, Some("Test".to_string()));
    assert_eq!(parsed.author, Some("Author".to_string()));
    assert!(!parsed.compress);
}

// =============================================================================
// PDF/A Compliance Tests
// =============================================================================

#[test]
fn test_pdfa_1b_export() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new()
        .with_title("PDF/A-1b Test Document")
        .with_author("Test Author")
        .with_pdfa_conformance(PdfAConformance::PdfA1b);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Verify PDF/A structure
    assert!(pdf_str.starts_with("%PDF-1.4")); // PDF/A-1b uses PDF 1.4
    assert!(pdf_str.contains("/Type /Metadata")); // XMP metadata required
    assert!(pdf_str.contains("/OutputIntents")); // Output intents required
    assert!(pdf_str.contains("/MarkInfo")); // MarkInfo required
}

#[test]
fn test_pdfa_2b_export() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new()
        .with_title("PDF/A-2b Test Document")
        .with_pdfa_conformance(PdfAConformance::PdfA2b);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // PDF/A-2b uses PDF 1.7
    assert!(pdf_str.starts_with("%PDF-1.7"));
}

#[test]
fn test_pdfa_conformance_auto_enables_font_embedding() {
    let options = PdfExportOptions::new()
        .with_pdfa_conformance(PdfAConformance::PdfA1b);

    // PDF/A conformance should automatically enable font embedding
    assert!(options.embed_fonts);
}

#[test]
fn test_pdfa_conformance_serialization() {
    let options = PdfExportOptions::new()
        .with_pdfa_conformance(PdfAConformance::PdfA1b);

    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("\"pdfaConformance\""));

    let parsed: PdfExportOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.pdfa_conformance, PdfAConformance::PdfA1b);
}

#[test]
fn test_pdfa_validation_basic() {
    let pages = vec![create_basic_page()];
    let report = validate_pdf_a_compliance(&pages, PdfAConformance::PdfA1b);

    // Report should have the correct conformance level
    assert_eq!(report.conformance, PdfAConformance::PdfA1b);

    // Should detect fonts that need embedding
    assert!(!report.fonts_to_embed.is_empty() || report.issues.iter().any(|i|
        matches!(i.category, super::pdfa::IssueCategory::Font)
    ));
}

#[test]
fn test_pdfa_validation_transparency() {
    // Create a page with transparency (using shapes that might have transparency)
    let page = PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![
            RenderItem::Rectangle {
                bounds: Rect::new(100.0, 100.0, 200.0, 100.0),
                fill: Some(Color::rgb(255, 0, 0)),
                stroke: None,
                stroke_width: 0.0,
            },
        ],
    };

    let report = validate_pdf_a_compliance(&[page], PdfAConformance::PdfA1b);

    // Should report color spaces used
    assert!(report.color_spaces.contains(&"DeviceRGB".to_string()));
}

#[test]
fn test_pdfa_none_conformance() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new()
        .with_pdfa_conformance(PdfAConformance::None);

    // Should not have PDF/A specific structures
    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Standard PDF should not have these structures
    assert!(!pdf_str.contains("/Type /Metadata"));
    assert!(!pdf_str.contains("/OutputIntents"));
}

#[test]
fn test_pdfa_with_metadata() {
    let pages = vec![create_basic_page()];
    let options = PdfExportOptions::new()
        .with_title("Archival Document")
        .with_author("Archive Author")
        .with_subject("Test Subject")
        .with_keyword("archive")
        .with_keyword("test")
        .with_pdfa_conformance(PdfAConformance::PdfA1b);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Should contain the metadata
    assert!(pdf_str.contains("Archival Document"));
    assert!(pdf_str.contains("Archive Author"));
}

#[test]
fn test_pdfa_multiple_pages() {
    let pages = vec![
        create_basic_page(),
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

    let options = PdfExportOptions::new()
        .with_pdfa_conformance(PdfAConformance::PdfA1b);

    let result = export_pdf_bytes(&pages, options);
    assert!(result.is_ok());

    let pdf_bytes = result.unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.contains("/Count 2"));
}

#[test]
fn test_pdfa_effective_version() {
    // PDF/A-1b should use PDF 1.4
    let options_1b = PdfExportOptions::new()
        .with_pdfa_conformance(PdfAConformance::PdfA1b);
    assert_eq!(options_1b.effective_pdf_version(), super::document::PdfVersion::V1_4);

    // PDF/A-2b should use PDF 1.7
    let options_2b = PdfExportOptions::new()
        .with_pdfa_conformance(PdfAConformance::PdfA2b);
    assert_eq!(options_2b.effective_pdf_version(), super::document::PdfVersion::V1_7);

    // No PDF/A should use specified version
    let options_none = PdfExportOptions::new()
        .with_version(PdfVersionOption::V15);
    assert_eq!(options_none.effective_pdf_version(), super::document::PdfVersion::V1_5);
}

#[test]
fn test_pdfa_compliance_report_structure() {
    let pages = vec![create_basic_page()];
    let report = validate_pdf_a_compliance(&pages, PdfAConformance::PdfA1b);

    // Test report accessors
    let _errors = report.errors();
    let _warnings = report.warnings();
    let _error_count = report.error_count();
    let _warning_count = report.warning_count();

    // Report structure should be valid
    assert!(report.has_issues() || report.is_compliant);
}

#[test]
fn test_parse_pdfa_conformance() {
    // Test various input formats
    assert_eq!(parse_pdfa_conformance("1b"), Some(PdfAConformance::PdfA1b));
    assert_eq!(parse_pdfa_conformance("2b"), Some(PdfAConformance::PdfA2b));
    assert_eq!(parse_pdfa_conformance("PDF/A-1b"), Some(PdfAConformance::PdfA1b));
    assert_eq!(parse_pdfa_conformance("pdfa-2b"), Some(PdfAConformance::PdfA2b));
    assert_eq!(parse_pdfa_conformance("none"), Some(PdfAConformance::None));
    assert_eq!(parse_pdfa_conformance(""), Some(PdfAConformance::None));
    assert_eq!(parse_pdfa_conformance("invalid"), None);
}

#[test]
fn test_pdfa_xmp_metadata_generation() {
    use super::pdfa::XmpMetadata;
    use super::document::DocumentInfo;

    let mut info = DocumentInfo::new();
    info.title = Some("Test Title".to_string());
    info.author = Some("Test Author".to_string());

    let metadata = XmpMetadata::from_document_info(&info, PdfAConformance::PdfA1b);
    let xmp_bytes = metadata.generate();
    let xmp_str = String::from_utf8_lossy(&xmp_bytes);

    // Should contain required XMP elements
    assert!(xmp_str.contains("<?xpacket"));
    assert!(xmp_str.contains("x:xmpmeta"));
    assert!(xmp_str.contains("rdf:RDF"));
    assert!(xmp_str.contains("Test Title"));
    assert!(xmp_str.contains("Test Author"));
    assert!(xmp_str.contains("pdfaid:part"));
    assert!(xmp_str.contains("pdfaid:conformance"));
}

#[test]
fn test_pdfa_conformance_properties() {
    // PDF/A-1b properties
    assert_eq!(PdfAConformance::PdfA1b.part(), Some(1));
    assert_eq!(PdfAConformance::PdfA1b.conformance_level(), Some("B"));
    assert!(!PdfAConformance::PdfA1b.allows_transparency());
    assert!(!PdfAConformance::PdfA1b.allows_jpeg2000());
    assert!(PdfAConformance::PdfA1b.requires_font_embedding());
    assert!(PdfAConformance::PdfA1b.is_pdfa());

    // PDF/A-2b properties
    assert_eq!(PdfAConformance::PdfA2b.part(), Some(2));
    assert_eq!(PdfAConformance::PdfA2b.conformance_level(), Some("B"));
    assert!(PdfAConformance::PdfA2b.allows_transparency());
    assert!(PdfAConformance::PdfA2b.allows_jpeg2000());
    assert!(PdfAConformance::PdfA2b.requires_font_embedding());
    assert!(PdfAConformance::PdfA2b.is_pdfa());

    // None (standard PDF) properties
    assert_eq!(PdfAConformance::None.part(), None);
    assert!(PdfAConformance::None.allows_transparency());
    assert!(!PdfAConformance::None.requires_font_embedding());
    assert!(!PdfAConformance::None.is_pdfa());
}
