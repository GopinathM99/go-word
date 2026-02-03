//! PDF Writer
//!
//! This module handles the actual PDF file generation, including:
//! - Object numbering and cross-reference table
//! - File structure (header, body, xref, trailer)
//! - Compression support
//! - PDF/A compliance (XMP metadata, output intents, font embedding)

use super::content::ContentStream;
use super::document::{create_catalog, create_pages, DocumentInfo, PdfDocumentBuilder, PdfPage, PdfVersion};
use super::fonts::{create_standard_font_dict, FontManager};
use super::objects::{PdfDictionary, PdfObject, PdfSerializer, PdfStream};
use super::options::PdfExportOptions;
use super::pdfa::{
    create_mark_info, create_srgb_icc_profile, create_srgb_output_intent,
    get_iso_date, PdfAConformance, XmpMetadata,
};
use super::renderer::{PageRenderInfo, PdfRenderer};
use std::io::{self, Write};
use thiserror::Error;

/// Error type for PDF operations
#[derive(Debug, Error)]
pub enum PdfError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// Invalid document structure
    #[error("Invalid document: {0}")]
    InvalidDocument(String),
    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),
}

/// Result type for PDF operations
pub type Result<T> = std::result::Result<T, PdfError>;

/// An object in the PDF file with its byte offset
#[derive(Debug)]
struct ObjectEntry {
    /// Object number
    obj_num: u32,
    /// Generation number (always 0 for new objects)
    gen_num: u16,
    /// Byte offset in the file
    offset: u64,
    /// The object data
    object: PdfObject,
}

/// PDF file writer
pub struct PdfWriter<W: Write> {
    /// Output writer
    writer: W,
    /// Current byte position
    position: u64,
    /// Objects in the file
    objects: Vec<ObjectEntry>,
    /// Next object number
    next_obj_num: u32,
    /// PDF version
    version: PdfVersion,
    /// Whether to compress streams
    compress: bool,
}

impl<W: Write> PdfWriter<W> {
    /// Create a new PDF writer
    pub fn new(writer: W, version: PdfVersion) -> Self {
        Self {
            writer,
            position: 0,
            objects: Vec::new(),
            next_obj_num: 1,
            version,
            compress: true,
        }
    }

    /// Set whether to compress streams
    pub fn set_compression(&mut self, compress: bool) {
        self.compress = compress;
    }

    /// Write bytes and update position
    fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        self.position += data.len() as u64;
        Ok(())
    }

    /// Write a string and update position
    fn write_str(&mut self, s: &str) -> Result<()> {
        self.write_bytes(s.as_bytes())
    }

    /// Allocate a new object number
    pub fn allocate_object(&mut self) -> u32 {
        let num = self.next_obj_num;
        self.next_obj_num += 1;
        num
    }

    /// Write the PDF header
    pub fn write_header(&mut self) -> Result<()> {
        self.write_str(&format!("%PDF-{}\n", self.version.as_str()))?;
        // Write binary marker (recommended for binary content)
        self.write_bytes(&[b'%', 0xE2, 0xE3, 0xCF, 0xD3, b'\n'])?;
        Ok(())
    }

    /// Write an indirect object
    pub fn write_object(&mut self, obj_num: u32, object: PdfObject) -> Result<()> {
        let offset = self.position;

        // Write object header
        self.write_str(&format!("{} 0 obj\n", obj_num))?;

        // Serialize the object
        let mut serializer = PdfSerializer::new(Vec::new());
        serializer.write_object(&object)?;
        self.write_bytes(&serializer.into_inner())?;

        // Write object footer
        self.write_str("\nendobj\n")?;

        // Record the object
        self.objects.push(ObjectEntry {
            obj_num,
            gen_num: 0,
            offset,
            object,
        });

        Ok(())
    }

    /// Write a stream object with optional compression
    pub fn write_stream_object(&mut self, obj_num: u32, mut stream: PdfStream) -> Result<()> {
        // Compress if enabled and not already compressed
        if self.compress && !stream.compressed {
            stream = self.compress_stream(stream)?;
        }

        // Update length in dictionary
        stream.dict.insert("Length", PdfObject::Integer(stream.data.len() as i64));

        let offset = self.position;

        // Write object header
        self.write_str(&format!("{} 0 obj\n", obj_num))?;

        // Serialize the stream
        let mut serializer = PdfSerializer::new(Vec::new());
        serializer.write_object(&PdfObject::Stream(stream.clone()))?;
        self.write_bytes(&serializer.into_inner())?;

        // Write object footer
        self.write_str("\nendobj\n")?;

        // Record the object
        self.objects.push(ObjectEntry {
            obj_num,
            gen_num: 0,
            offset,
            object: PdfObject::Stream(stream),
        });

        Ok(())
    }

    /// Compress a stream using flate compression
    fn compress_stream(&self, mut stream: PdfStream) -> Result<PdfStream> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&stream.data)?;
        stream.data = encoder.finish()?;
        stream.compressed = true;
        stream.dict.insert("Filter", PdfObject::Name("FlateDecode".to_string()));

        Ok(stream)
    }

    /// Write the cross-reference table and trailer
    pub fn write_xref_and_trailer(&mut self, catalog_ref: u32, info_ref: Option<u32>) -> Result<()> {
        let xref_offset = self.position;

        // Sort objects by number
        self.objects.sort_by_key(|e| e.obj_num);

        // Collect object entries to avoid borrow issues
        let entries: Vec<_> = self.objects.iter().map(|e| (e.obj_num, e.offset, e.gen_num)).collect();
        let next_obj_num = self.next_obj_num;

        // Write xref table header
        self.write_str("xref\n")?;
        self.write_str(&format!("0 {}\n", next_obj_num))?;

        // Write free entry for object 0
        self.write_str("0000000000 65535 f \n")?;

        // Write entries for each object
        let mut expected_num = 1u32;
        for (obj_num, offset, gen_num) in entries {
            // Fill gaps with free entries if needed
            while expected_num < obj_num {
                self.write_str("0000000000 65535 f \n")?;
                expected_num += 1;
            }

            self.write_str(&format!("{:010} {:05} n \n", offset, gen_num))?;
            expected_num = obj_num + 1;
        }

        // Write trailer
        self.write_str("trailer\n")?;

        let mut trailer = PdfDictionary::new();
        trailer.insert("Size", PdfObject::Integer(self.next_obj_num as i64));
        trailer.insert("Root", PdfObject::Reference(catalog_ref, 0));

        if let Some(info) = info_ref {
            trailer.insert("Info", PdfObject::Reference(info, 0));
        }

        let mut serializer = PdfSerializer::new(Vec::new());
        serializer.write_object(&PdfObject::Dictionary(trailer))?;
        self.write_bytes(&serializer.into_inner())?;
        self.write_str("\n")?;

        // Write startxref
        self.write_str("startxref\n")?;
        self.write_str(&format!("{}\n", xref_offset))?;
        self.write_str("%%EOF\n")?;

        Ok(())
    }

    /// Flush and return the inner writer
    pub fn finish(mut self) -> Result<W> {
        self.writer.flush()?;
        Ok(self.writer)
    }
}

/// High-level PDF document writer
pub struct PdfDocumentWriter {
    /// Export options
    options: PdfExportOptions,
}

impl PdfDocumentWriter {
    /// Create a new document writer
    pub fn new(options: PdfExportOptions) -> Self {
        Self { options }
    }

    /// Write a complete PDF document to a writer
    pub fn write<W: Write>(&self, pages: &[PageRenderInfo], writer: W) -> Result<()> {
        if pages.is_empty() {
            return Err(PdfError::InvalidDocument("No pages to export".to_string()));
        }

        // Use effective PDF version (may be overridden by PDF/A requirements)
        let version = self.options.effective_pdf_version();
        let mut pdf = PdfWriter::new(writer, version);
        pdf.set_compression(self.options.compress);

        let is_pdfa = self.options.is_pdfa();
        let pdfa_conformance = self.options.pdfa_conformance;

        // Write header
        pdf.write_header()?;

        // Allocate object numbers
        let catalog_ref = pdf.allocate_object();
        let pages_ref = pdf.allocate_object();
        let info_ref = pdf.allocate_object();

        // PDF/A requires additional objects
        let metadata_ref = if is_pdfa { Some(pdf.allocate_object()) } else { None };
        let output_intent_ref = if is_pdfa { Some(pdf.allocate_object()) } else { None };
        let icc_profile_ref = if is_pdfa { Some(pdf.allocate_object()) } else { None };

        // Allocate page and content object numbers
        let mut page_refs = Vec::new();
        let mut content_refs = Vec::new();
        let mut font_refs = Vec::new();

        // Create renderer to track fonts
        let mut renderer = PdfRenderer::new(self.options.clone());

        // First pass: render all pages and collect fonts
        let mut content_streams = Vec::new();
        for page_info in pages {
            if !self.options.should_include_page(content_streams.len()) {
                continue;
            }
            let content = renderer.render_page(page_info);
            content_streams.push((page_info, content));
        }

        // Allocate font objects
        for font in renderer.font_manager().fonts() {
            let font_ref = pdf.allocate_object();
            font_refs.push((font.name.clone(), font.standard_font, font_ref));
        }

        // Allocate page objects
        for _ in 0..content_streams.len() {
            page_refs.push(pdf.allocate_object());
            content_refs.push(pdf.allocate_object());
        }

        // Build catalog with PDF/A extensions
        let mut catalog = create_catalog(pages_ref);

        if is_pdfa {
            // Add metadata reference
            if let Some(meta_ref) = metadata_ref {
                catalog.insert("Metadata", PdfObject::Reference(meta_ref, 0));
            }

            // Add output intents array
            if let Some(oi_ref) = output_intent_ref {
                catalog.insert(
                    "OutputIntents",
                    PdfObject::Array(vec![PdfObject::Reference(oi_ref, 0)]),
                );
            }

            // Add MarkInfo for PDF/A
            let mark_info = create_mark_info(false);
            catalog.insert("MarkInfo", PdfObject::Dictionary(mark_info));
        }

        // Write catalog
        pdf.write_object(catalog_ref, PdfObject::Dictionary(catalog))?;

        // Write pages object
        let pages_dict = create_pages(&page_refs, page_refs.len());
        pdf.write_object(pages_ref, PdfObject::Dictionary(pages_dict))?;

        // Write info dictionary
        let mut info = DocumentInfo::new();
        info.title = self.options.title.clone();
        info.author = self.options.author.clone();
        info.subject = self.options.subject.clone();
        info.keywords = self.options.keywords.clone();

        // Add dates for PDF/A
        if is_pdfa {
            let iso_date = get_iso_date();
            let pdf_date = format!("D:{}", iso_date.replace("-", "").replace(":", "").replace("T", "").replace("Z", "+00'00'"));
            info.creation_date = Some(pdf_date.clone());
            info.modification_date = Some(pdf_date);
        }

        pdf.write_object(info_ref, PdfObject::Dictionary(info.to_dictionary()))?;

        // Write PDF/A specific objects
        if is_pdfa {
            // Write ICC profile
            if let Some(icc_ref) = icc_profile_ref {
                let icc_profile = create_srgb_icc_profile();
                pdf.write_stream_object(icc_ref, icc_profile)?;
            }

            // Write output intent
            if let (Some(oi_ref), Some(icc_ref)) = (output_intent_ref, icc_profile_ref) {
                let output_intent = create_srgb_output_intent(icc_ref);
                pdf.write_object(oi_ref, PdfObject::Dictionary(output_intent))?;
            }

            // Write XMP metadata
            if let Some(meta_ref) = metadata_ref {
                let mut xmp_info = DocumentInfo::new();
                xmp_info.title = self.options.title.clone();
                xmp_info.author = self.options.author.clone();
                xmp_info.subject = self.options.subject.clone();
                xmp_info.creation_date = Some(get_iso_date());
                xmp_info.modification_date = Some(get_iso_date());

                let xmp_metadata = XmpMetadata::from_document_info(&xmp_info, pdfa_conformance);
                let metadata_stream = xmp_metadata.to_stream();
                pdf.write_stream_object(meta_ref, metadata_stream)?;
            }
        }

        // Write font objects
        for (_, standard_font, font_ref) in &font_refs {
            let font_dict = create_standard_font_dict(*standard_font);
            pdf.write_object(*font_ref, PdfObject::Dictionary(font_dict))?;
        }

        // Write page and content objects
        for (i, (page_info, content)) in content_streams.into_iter().enumerate() {
            let page_ref = page_refs[i];
            let content_ref = content_refs[i];

            // Write content stream
            let stream = PdfStream::new(content.into_bytes());
            pdf.write_stream_object(content_ref, stream)?;

            // Build page dictionary
            let mut page_dict = PdfDictionary::new().with_type("Page");
            page_dict.insert("Parent", PdfObject::Reference(pages_ref, 0));
            page_dict.insert(
                "MediaBox",
                PdfObject::Array(vec![
                    PdfObject::Real(0.0),
                    PdfObject::Real(0.0),
                    PdfObject::Real(page_info.width),
                    PdfObject::Real(page_info.height),
                ]),
            );
            page_dict.insert("Contents", PdfObject::Reference(content_ref, 0));

            // Build resources dictionary
            let mut resources = PdfDictionary::new();

            // Add fonts
            if !font_refs.is_empty() {
                let mut font_dict = PdfDictionary::new();
                for (name, _, ref_num) in &font_refs {
                    font_dict.insert(name.clone(), PdfObject::Reference(*ref_num, 0));
                }
                resources.insert("Font", PdfObject::Dictionary(font_dict));
            }

            // Add ProcSet
            resources.insert(
                "ProcSet",
                PdfObject::Array(vec![
                    PdfObject::Name("PDF".to_string()),
                    PdfObject::Name("Text".to_string()),
                ]),
            );

            page_dict.insert("Resources", PdfObject::Dictionary(resources));

            pdf.write_object(page_ref, PdfObject::Dictionary(page_dict))?;
        }

        // Write cross-reference table and trailer
        pdf.write_xref_and_trailer(catalog_ref, Some(info_ref))?;

        // Finish
        pdf.finish()?;

        Ok(())
    }

    /// Write a complete PDF document to bytes
    pub fn write_to_bytes(&self, pages: &[PageRenderInfo]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.write(pages, &mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::renderer::{TextRenderInfo, RgbColor, PdfRenderItem};

    fn create_test_page() -> PageRenderInfo {
        let mut page = PageRenderInfo::new(612.0, 792.0);
        page.add_item(PdfRenderItem::Text(TextRenderInfo {
            text: "Hello, PDF!".to_string(),
            x: 72.0,
            y: 720.0,
            font_family: "Helvetica".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: RgbColor::black(),
        }));
        page
    }

    #[test]
    fn test_pdf_writer_header() {
        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, PdfVersion::V1_4);
        writer.write_header().unwrap();

        let output = String::from_utf8_lossy(&buffer);
        assert!(output.starts_with("%PDF-1.4"));
    }

    #[test]
    fn test_pdf_writer_object() {
        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, PdfVersion::V1_4);

        let obj_num = writer.allocate_object();
        writer.write_object(obj_num, PdfObject::Integer(42)).unwrap();

        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("1 0 obj"));
        assert!(output.contains("42"));
        assert!(output.contains("endobj"));
    }

    #[test]
    fn test_pdf_document_writer() {
        let options = PdfExportOptions::default();
        let writer = PdfDocumentWriter::new(options);

        let pages = vec![create_test_page()];
        let result = writer.write_to_bytes(&pages);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();

        // Verify PDF structure
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.starts_with("%PDF-"));
        assert!(pdf_str.contains("/Type /Catalog"));
        assert!(pdf_str.contains("/Type /Pages"));
        assert!(pdf_str.contains("/Type /Page"));
        assert!(pdf_str.contains("xref"));
        assert!(pdf_str.contains("trailer"));
        assert!(pdf_str.contains("startxref"));
        assert!(pdf_str.ends_with("%%EOF\n"));
    }

    #[test]
    fn test_pdf_with_metadata() {
        let options = PdfExportOptions::new()
            .with_title("Test Document")
            .with_author("Test Author");

        let writer = PdfDocumentWriter::new(options);
        let pages = vec![create_test_page()];
        let pdf_bytes = writer.write_to_bytes(&pages).unwrap();

        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.contains("Test Document"));
        assert!(pdf_str.contains("Test Author"));
    }

    #[test]
    fn test_pdf_no_compression() {
        let options = PdfExportOptions::new().with_compression(false);
        let writer = PdfDocumentWriter::new(options);

        let pages = vec![create_test_page()];
        let pdf_bytes = writer.write_to_bytes(&pages).unwrap();

        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        // Without compression, we should see the raw content
        assert!(pdf_str.contains("BT")); // Begin text
        assert!(pdf_str.contains("ET")); // End text
    }

    #[test]
    fn test_empty_pages_error() {
        let options = PdfExportOptions::default();
        let writer = PdfDocumentWriter::new(options);

        let result = writer.write_to_bytes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_pages() {
        let options = PdfExportOptions::default();
        let writer = PdfDocumentWriter::new(options);

        let pages = vec![
            create_test_page(),
            create_test_page(),
            create_test_page(),
        ];

        let pdf_bytes = writer.write_to_bytes(&pages).unwrap();
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);

        // Should have 3 page objects
        assert!(pdf_str.contains("/Count 3"));
    }
}
