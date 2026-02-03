//! PDF Image Handling
//!
//! This module handles image embedding in PDF documents.
//! It supports:
//! - JPEG images (DCTDecode - passed through without re-encoding)
//! - PNG images (FlateDecode with alpha handling)
//! - Image XObject generation

use super::objects::{PdfDictionary, PdfObject, PdfStream};
use std::io::{self, Read};

/// Image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG image
    Jpeg,
    /// PNG image
    Png,
    /// Raw RGB data
    RawRgb,
    /// Raw grayscale data
    RawGray,
}

/// Image data for embedding in PDF
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Bits per component (usually 8)
    pub bits_per_component: u8,
    /// Color space
    pub color_space: ColorSpace,
    /// Image data (compressed or raw depending on format)
    pub data: Vec<u8>,
    /// Filter to use for decoding
    pub filter: Option<ImageFilter>,
    /// Optional soft mask (alpha channel) object reference
    pub soft_mask_ref: Option<u32>,
}

/// Color space for images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// Grayscale (1 component)
    DeviceGray,
    /// RGB (3 components)
    DeviceRGB,
    /// CMYK (4 components)
    DeviceCMYK,
}

impl ColorSpace {
    /// Get the PDF name for this color space
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ColorSpace::DeviceGray => "DeviceGray",
            ColorSpace::DeviceRGB => "DeviceRGB",
            ColorSpace::DeviceCMYK => "DeviceCMYK",
        }
    }

    /// Get the number of components
    pub fn components(&self) -> u8 {
        match self {
            ColorSpace::DeviceGray => 1,
            ColorSpace::DeviceRGB => 3,
            ColorSpace::DeviceCMYK => 4,
        }
    }
}

/// Image compression filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFilter {
    /// DCT (JPEG) compression
    DCTDecode,
    /// Flate (zlib) compression
    FlateDecode,
    /// ASCII hex encoding
    ASCIIHexDecode,
}

impl ImageFilter {
    /// Get the PDF name for this filter
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ImageFilter::DCTDecode => "DCTDecode",
            ImageFilter::FlateDecode => "FlateDecode",
            ImageFilter::ASCIIHexDecode => "ASCIIHexDecode",
        }
    }
}

impl ImageData {
    /// Create image data from JPEG bytes
    pub fn from_jpeg(data: Vec<u8>) -> Result<Self, ImageError> {
        // Parse JPEG header to get dimensions
        let (width, height) = parse_jpeg_dimensions(&data)?;

        Ok(Self {
            width,
            height,
            bits_per_component: 8,
            color_space: ColorSpace::DeviceRGB, // Most JPEGs are RGB
            data,
            filter: Some(ImageFilter::DCTDecode),
            soft_mask_ref: None,
        })
    }

    /// Create image data from raw RGB bytes
    pub fn from_raw_rgb(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bits_per_component: 8,
            color_space: ColorSpace::DeviceRGB,
            data,
            filter: None,
            soft_mask_ref: None,
        }
    }

    /// Create image data from raw grayscale bytes
    pub fn from_raw_gray(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bits_per_component: 8,
            color_space: ColorSpace::DeviceGray,
            data,
            filter: None,
            soft_mask_ref: None,
        }
    }

    /// Compress the image data using flate compression
    #[cfg(feature = "compression")]
    pub fn compress(&mut self) -> Result<(), ImageError> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;

        if self.filter.is_some() {
            // Already compressed
            return Ok(());
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.data)?;
        self.data = encoder.finish()?;
        self.filter = Some(ImageFilter::FlateDecode);
        Ok(())
    }

    /// Set the soft mask reference
    pub fn with_soft_mask(mut self, mask_ref: u32) -> Self {
        self.soft_mask_ref = Some(mask_ref);
        self
    }

    /// Convert to PDF XObject stream
    pub fn to_xobject(&self) -> PdfStream {
        let mut dict = PdfDictionary::new()
            .with_type("XObject");

        dict.insert("Subtype", PdfObject::Name("Image".to_string()));
        dict.insert("Width", PdfObject::Integer(self.width as i64));
        dict.insert("Height", PdfObject::Integer(self.height as i64));
        dict.insert("BitsPerComponent", PdfObject::Integer(self.bits_per_component as i64));
        dict.insert("ColorSpace", PdfObject::Name(self.color_space.pdf_name().to_string()));
        dict.insert("Length", PdfObject::Integer(self.data.len() as i64));

        if let Some(filter) = self.filter {
            dict.insert("Filter", PdfObject::Name(filter.pdf_name().to_string()));
        }

        if let Some(mask_ref) = self.soft_mask_ref {
            dict.insert("SMask", PdfObject::Reference(mask_ref, 0));
        }

        PdfStream {
            dict,
            data: self.data.clone(),
            compressed: self.filter.is_some(),
        }
    }
}

/// Create a soft mask (alpha channel) XObject
pub fn create_soft_mask(data: Vec<u8>, width: u32, height: u32) -> PdfStream {
    let mut dict = PdfDictionary::new()
        .with_type("XObject");

    dict.insert("Subtype", PdfObject::Name("Image".to_string()));
    dict.insert("Width", PdfObject::Integer(width as i64));
    dict.insert("Height", PdfObject::Integer(height as i64));
    dict.insert("BitsPerComponent", PdfObject::Integer(8));
    dict.insert("ColorSpace", PdfObject::Name("DeviceGray".to_string()));
    dict.insert("Length", PdfObject::Integer(data.len() as i64));

    PdfStream {
        dict,
        data,
        compressed: false,
    }
}

/// Error type for image operations
#[derive(Debug)]
pub enum ImageError {
    /// Invalid image format
    InvalidFormat(String),
    /// IO error
    Io(io::Error),
    /// Unsupported feature
    Unsupported(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::InvalidFormat(msg) => write!(f, "Invalid image format: {}", msg),
            ImageError::Io(e) => write!(f, "IO error: {}", e),
            ImageError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}

impl std::error::Error for ImageError {}

impl From<io::Error> for ImageError {
    fn from(e: io::Error) -> Self {
        ImageError::Io(e)
    }
}

/// Parse JPEG header to extract dimensions
fn parse_jpeg_dimensions(data: &[u8]) -> Result<(u32, u32), ImageError> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(ImageError::InvalidFormat("Not a valid JPEG".to_string()));
    }

    let mut pos = 2;
    while pos < data.len() - 1 {
        if data[pos] != 0xFF {
            return Err(ImageError::InvalidFormat("Invalid JPEG marker".to_string()));
        }

        let marker = data[pos + 1];
        pos += 2;

        // Skip padding
        while pos < data.len() && data[pos] == 0xFF {
            pos += 1;
        }

        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            // Standalone markers, no length
            continue;
        }

        if pos + 2 > data.len() {
            break;
        }

        let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);

        // SOF markers contain image dimensions
        if (0xC0..=0xC3).contains(&marker) || (0xC5..=0xC7).contains(&marker) ||
           (0xC9..=0xCB).contains(&marker) || (0xCD..=0xCF).contains(&marker) {
            if pos + 7 > data.len() {
                break;
            }
            let height = ((data[pos + 3] as u32) << 8) | (data[pos + 4] as u32);
            let width = ((data[pos + 5] as u32) << 8) | (data[pos + 6] as u32);
            return Ok((width, height));
        }

        pos += length;
    }

    Err(ImageError::InvalidFormat("Could not find image dimensions in JPEG".to_string()))
}

/// Image reference in a PDF document
#[derive(Debug, Clone)]
pub struct ImageRef {
    /// Internal image name (e.g., "Im1", "Im2")
    pub name: String,
    /// Object reference number
    pub obj_ref: u32,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
}

/// Image manager for PDF export
#[derive(Debug, Default)]
pub struct ImageManager {
    /// Images that have been added (internal name -> image ref)
    images: Vec<ImageRef>,
    /// Next image number
    next_image_num: u32,
}

impl ImageManager {
    /// Create a new image manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new image and return its reference
    pub fn register_image(&mut self, obj_ref: u32, width: u32, height: u32) -> ImageRef {
        let name = format!("Im{}", self.next_image_num);
        self.next_image_num += 1;

        let image_ref = ImageRef {
            name: name.clone(),
            obj_ref,
            width,
            height,
        };

        self.images.push(image_ref.clone());
        image_ref
    }

    /// Get all registered images
    pub fn images(&self) -> &[ImageRef] {
        &self.images
    }

    /// Get the number of images
    pub fn image_count(&self) -> usize {
        self.images.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_space() {
        assert_eq!(ColorSpace::DeviceRGB.components(), 3);
        assert_eq!(ColorSpace::DeviceGray.components(), 1);
        assert_eq!(ColorSpace::DeviceCMYK.components(), 4);
    }

    #[test]
    fn test_image_filter_names() {
        assert_eq!(ImageFilter::DCTDecode.pdf_name(), "DCTDecode");
        assert_eq!(ImageFilter::FlateDecode.pdf_name(), "FlateDecode");
    }

    #[test]
    fn test_raw_rgb_image() {
        let data = vec![255u8; 3 * 10 * 10]; // 10x10 white image
        let image = ImageData::from_raw_rgb(data, 10, 10);

        assert_eq!(image.width, 10);
        assert_eq!(image.height, 10);
        assert_eq!(image.color_space, ColorSpace::DeviceRGB);
        assert!(image.filter.is_none());
    }

    #[test]
    fn test_raw_gray_image() {
        let data = vec![128u8; 10 * 10]; // 10x10 gray image
        let image = ImageData::from_raw_gray(data, 10, 10);

        assert_eq!(image.width, 10);
        assert_eq!(image.height, 10);
        assert_eq!(image.color_space, ColorSpace::DeviceGray);
    }

    #[test]
    fn test_image_manager() {
        let mut manager = ImageManager::new();

        let img1 = manager.register_image(10, 100, 200);
        assert_eq!(img1.name, "Im0");
        assert_eq!(img1.obj_ref, 10);

        let img2 = manager.register_image(11, 50, 50);
        assert_eq!(img2.name, "Im1");

        assert_eq!(manager.image_count(), 2);
    }

    #[test]
    fn test_xobject_creation() {
        let data = vec![0u8; 3 * 5 * 5]; // 5x5 black image
        let image = ImageData::from_raw_rgb(data, 5, 5);
        let xobject = image.to_xobject();

        assert!(xobject.dict.get("Width").is_some());
        assert!(xobject.dict.get("Height").is_some());
        assert!(xobject.dict.get("ColorSpace").is_some());
    }
}
