//! PDF Document Structure
//!
//! This module defines the high-level PDF document structure including:
//! - Document catalog (root object)
//! - Page tree
//! - Page objects
//! - Resources dictionary
//! - Info dictionary

use super::objects::{PdfDictionary, PdfObject, PdfStream, PdfString};
use std::collections::HashMap;

/// PDF version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfVersion {
    /// PDF 1.4 (Acrobat 5)
    V1_4,
    /// PDF 1.5 (Acrobat 6)
    V1_5,
    /// PDF 1.7 (Acrobat 8)
    V1_7,
}

impl PdfVersion {
    /// Get the version string
    pub fn as_str(&self) -> &'static str {
        match self {
            PdfVersion::V1_4 => "1.4",
            PdfVersion::V1_5 => "1.5",
            PdfVersion::V1_7 => "1.7",
        }
    }
}

impl Default for PdfVersion {
    fn default() -> Self {
        PdfVersion::V1_4
    }
}

/// PDF document information
#[derive(Debug, Clone, Default)]
pub struct DocumentInfo {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document keywords
    pub keywords: Vec<String>,
    /// Creator application
    pub creator: Option<String>,
    /// PDF producer
    pub producer: Option<String>,
    /// Creation date (PDF date string format)
    pub creation_date: Option<String>,
    /// Modification date (PDF date string format)
    pub modification_date: Option<String>,
}

impl DocumentInfo {
    /// Create a new document info with default values
    pub fn new() -> Self {
        Self {
            creator: Some("MS Word Clone".to_string()),
            producer: Some("MS Word Clone PDF Export".to_string()),
            ..Default::default()
        }
    }

    /// Convert to PDF dictionary
    pub fn to_dictionary(&self) -> PdfDictionary {
        let mut dict = PdfDictionary::new();

        if let Some(ref title) = self.title {
            dict.insert("Title", PdfObject::String(PdfString::from_str(title)));
        }
        if let Some(ref author) = self.author {
            dict.insert("Author", PdfObject::String(PdfString::from_str(author)));
        }
        if let Some(ref subject) = self.subject {
            dict.insert("Subject", PdfObject::String(PdfString::from_str(subject)));
        }
        if !self.keywords.is_empty() {
            let keywords = self.keywords.join(", ");
            dict.insert("Keywords", PdfObject::String(PdfString::from_str(&keywords)));
        }
        if let Some(ref creator) = self.creator {
            dict.insert("Creator", PdfObject::String(PdfString::from_str(creator)));
        }
        if let Some(ref producer) = self.producer {
            dict.insert("Producer", PdfObject::String(PdfString::from_str(producer)));
        }
        if let Some(ref date) = self.creation_date {
            dict.insert("CreationDate", PdfObject::String(PdfString::from_str(date)));
        }
        if let Some(ref date) = self.modification_date {
            dict.insert("ModDate", PdfObject::String(PdfString::from_str(date)));
        }

        dict
    }
}

/// Page media box (page dimensions)
#[derive(Debug, Clone, Copy)]
pub struct MediaBox {
    /// Lower-left x coordinate
    pub llx: f64,
    /// Lower-left y coordinate
    pub lly: f64,
    /// Upper-right x coordinate
    pub urx: f64,
    /// Upper-right y coordinate
    pub ury: f64,
}

impl MediaBox {
    /// Create a media box from dimensions (origin at lower-left)
    pub fn from_dimensions(width: f64, height: f64) -> Self {
        Self {
            llx: 0.0,
            lly: 0.0,
            urx: width,
            ury: height,
        }
    }

    /// US Letter size (8.5 x 11 inches)
    pub fn letter() -> Self {
        Self::from_dimensions(612.0, 792.0)
    }

    /// A4 size (210 x 297 mm)
    pub fn a4() -> Self {
        Self::from_dimensions(595.0, 842.0)
    }

    /// Convert to PDF array
    pub fn to_array(&self) -> PdfObject {
        PdfObject::Array(vec![
            PdfObject::Real(self.llx),
            PdfObject::Real(self.lly),
            PdfObject::Real(self.urx),
            PdfObject::Real(self.ury),
        ])
    }

    /// Get page width
    pub fn width(&self) -> f64 {
        self.urx - self.llx
    }

    /// Get page height
    pub fn height(&self) -> f64 {
        self.ury - self.lly
    }
}

impl Default for MediaBox {
    fn default() -> Self {
        Self::letter()
    }
}

/// PDF page object
#[derive(Debug, Clone)]
pub struct PdfPage {
    /// Page media box
    pub media_box: MediaBox,
    /// Content stream reference (object number)
    pub content_ref: Option<u32>,
    /// Font resources (name -> object reference)
    pub fonts: HashMap<String, u32>,
    /// Image XObject resources (name -> object reference)
    pub images: HashMap<String, u32>,
    /// Additional page properties
    pub properties: PdfDictionary,
}

impl PdfPage {
    /// Create a new page with the given media box
    pub fn new(media_box: MediaBox) -> Self {
        Self {
            media_box,
            content_ref: None,
            fonts: HashMap::new(),
            images: HashMap::new(),
            properties: PdfDictionary::new(),
        }
    }

    /// Create a new letter-size page
    pub fn letter() -> Self {
        Self::new(MediaBox::letter())
    }

    /// Create a new A4-size page
    pub fn a4() -> Self {
        Self::new(MediaBox::a4())
    }

    /// Set the content stream reference
    pub fn with_content(mut self, content_ref: u32) -> Self {
        self.content_ref = Some(content_ref);
        self
    }

    /// Add a font resource
    pub fn add_font(&mut self, name: impl Into<String>, obj_ref: u32) {
        self.fonts.insert(name.into(), obj_ref);
    }

    /// Add an image resource
    pub fn add_image(&mut self, name: impl Into<String>, obj_ref: u32) {
        self.images.insert(name.into(), obj_ref);
    }

    /// Build the resources dictionary
    pub fn build_resources(&self) -> PdfDictionary {
        let mut resources = PdfDictionary::new();

        // Add font resources
        if !self.fonts.is_empty() {
            let mut font_dict = PdfDictionary::new();
            for (name, obj_ref) in &self.fonts {
                font_dict.insert(name.clone(), PdfObject::Reference(*obj_ref, 0));
            }
            resources.insert("Font", PdfObject::Dictionary(font_dict));
        }

        // Add image XObject resources
        if !self.images.is_empty() {
            let mut xobject_dict = PdfDictionary::new();
            for (name, obj_ref) in &self.images {
                xobject_dict.insert(name.clone(), PdfObject::Reference(*obj_ref, 0));
            }
            resources.insert("XObject", PdfObject::Dictionary(xobject_dict));
        }

        // Add ProcSet (required for PDF 1.4 compatibility)
        resources.insert(
            "ProcSet",
            PdfObject::Array(vec![
                PdfObject::Name("PDF".to_string()),
                PdfObject::Name("Text".to_string()),
                PdfObject::Name("ImageB".to_string()),
                PdfObject::Name("ImageC".to_string()),
                PdfObject::Name("ImageI".to_string()),
            ]),
        );

        resources
    }

    /// Build the page dictionary
    pub fn to_dictionary(&self, parent_ref: u32) -> PdfDictionary {
        let mut dict = PdfDictionary::new().with_type("Page");

        dict.insert("Parent", PdfObject::Reference(parent_ref, 0));
        dict.insert("MediaBox", self.media_box.to_array());
        dict.insert("Resources", PdfObject::Dictionary(self.build_resources()));

        if let Some(content_ref) = self.content_ref {
            dict.insert("Contents", PdfObject::Reference(content_ref, 0));
        }

        // Add any additional properties
        for (key, value) in self.properties.iter() {
            if !dict.contains_key(key) {
                dict.insert(key.clone(), value.clone());
            }
        }

        dict
    }
}

/// PDF document structure builder
#[derive(Debug)]
pub struct PdfDocumentBuilder {
    /// PDF version
    pub version: PdfVersion,
    /// Document info
    pub info: DocumentInfo,
    /// Pages
    pub pages: Vec<PdfPage>,
}

impl PdfDocumentBuilder {
    /// Create a new document builder
    pub fn new() -> Self {
        Self {
            version: PdfVersion::default(),
            info: DocumentInfo::new(),
            pages: Vec::new(),
        }
    }

    /// Set the PDF version
    pub fn with_version(mut self, version: PdfVersion) -> Self {
        self.version = version;
        self
    }

    /// Set the document info
    pub fn with_info(mut self, info: DocumentInfo) -> Self {
        self.info = info;
        self
    }

    /// Add a page
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    /// Get number of pages
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

impl Default for PdfDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a catalog dictionary
pub fn create_catalog(pages_ref: u32) -> PdfDictionary {
    let mut dict = PdfDictionary::new().with_type("Catalog");
    dict.insert("Pages", PdfObject::Reference(pages_ref, 0));
    dict
}

/// Create a pages dictionary (page tree root)
pub fn create_pages(page_refs: &[u32], count: usize) -> PdfDictionary {
    let mut dict = PdfDictionary::new().with_type("Pages");

    let kids: Vec<PdfObject> = page_refs
        .iter()
        .map(|&r| PdfObject::Reference(r, 0))
        .collect();

    dict.insert("Kids", PdfObject::Array(kids));
    dict.insert("Count", PdfObject::Integer(count as i64));

    dict
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_box_letter() {
        let mb = MediaBox::letter();
        assert_eq!(mb.width(), 612.0);
        assert_eq!(mb.height(), 792.0);
    }

    #[test]
    fn test_media_box_a4() {
        let mb = MediaBox::a4();
        assert_eq!(mb.width(), 595.0);
        assert_eq!(mb.height(), 842.0);
    }

    #[test]
    fn test_document_info() {
        let mut info = DocumentInfo::new();
        info.title = Some("Test Document".to_string());
        info.author = Some("Test Author".to_string());

        let dict = info.to_dictionary();
        assert!(dict.get("Title").is_some());
        assert!(dict.get("Author").is_some());
        assert!(dict.get("Creator").is_some());
    }

    #[test]
    fn test_page_resources() {
        let mut page = PdfPage::letter();
        page.add_font("F1", 10);
        page.add_image("Im1", 11);

        let resources = page.build_resources();
        assert!(resources.get("Font").is_some());
        assert!(resources.get("XObject").is_some());
        assert!(resources.get("ProcSet").is_some());
    }

    #[test]
    fn test_create_catalog() {
        let catalog = create_catalog(2);
        assert!(catalog.get("Type").is_some());
        assert!(catalog.get("Pages").is_some());
    }

    #[test]
    fn test_create_pages() {
        let pages = create_pages(&[3, 4, 5], 3);
        assert!(pages.get("Type").is_some());
        assert!(pages.get("Kids").is_some());
        assert!(pages.get("Count").is_some());
    }
}
