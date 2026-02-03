//! Image resource storage and management
//!
//! This module handles storing, retrieving, and caching image data for the document.
//! Images are stored as binary blobs with unique resource IDs.

use doc_model::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Error types for image store operations
#[derive(Debug, Error)]
pub enum ImageStoreError {
    #[error("Image not found: {0}")]
    NotFound(String),

    #[error("Invalid image format: {0}")]
    InvalidFormat(String),

    #[error("Image decode error: {0}")]
    DecodeError(String),

    #[error("Image encode error: {0}")]
    EncodeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image too large: {size} bytes (max: {max} bytes)")]
    TooLarge { size: usize, max: usize },
}

pub type Result<T> = std::result::Result<T, ImageStoreError>;

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Svg,
    Bmp,
    Unknown,
}

impl ImageFormat {
    /// Detect format from magic bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() < 4 {
            return Self::Unknown;
        }

        // PNG: 89 50 4E 47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Self::Png;
        }

        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Self::Jpeg;
        }

        // GIF: 47 49 46 38
        if data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
            return Self::Gif;
        }

        // WebP: 52 49 46 46 ... 57 45 42 50
        if data.len() >= 12 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && &data[8..12] == b"WEBP"
        {
            return Self::WebP;
        }

        // BMP: 42 4D
        if data.starts_with(&[0x42, 0x4D]) {
            return Self::Bmp;
        }

        // SVG: check for XML-like content
        if let Ok(text) = std::str::from_utf8(&data[..data.len().min(256)]) {
            let text_lower = text.to_lowercase();
            if text_lower.contains("<svg") || text_lower.contains("<?xml") {
                return Self::Svg;
            }
        }

        Self::Unknown
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Svg => "image/svg+xml",
            Self::Bmp => "image/bmp",
            Self::Unknown => "application/octet-stream",
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Svg => "svg",
            Self::Bmp => "bmp",
            Self::Unknown => "bin",
        }
    }

    /// Check if this is a supported format
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Stored image data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Unique resource identifier
    pub resource_id: ResourceId,
    /// Raw image bytes
    pub data: Vec<u8>,
    /// Detected image format
    pub format: ImageFormat,
    /// Original width in pixels
    pub width: u32,
    /// Original height in pixels
    pub height: u32,
    /// File size in bytes
    pub size: usize,
    /// Original filename (if known)
    pub filename: Option<String>,
}

impl ImageData {
    /// Create a new ImageData with basic validation
    pub fn new(data: Vec<u8>, filename: Option<String>) -> Result<Self> {
        let format = ImageFormat::from_bytes(&data);

        if !format.is_supported() {
            return Err(ImageStoreError::InvalidFormat(
                "Unknown or unsupported image format".into(),
            ));
        }

        // Try to get dimensions based on format
        let (width, height) = Self::detect_dimensions(&data, format)?;

        let resource_id = ResourceId::new(uuid::Uuid::new_v4().to_string());
        let size = data.len();

        Ok(Self {
            resource_id,
            data,
            format,
            width,
            height,
            size,
            filename,
        })
    }

    /// Detect image dimensions from raw data
    fn detect_dimensions(data: &[u8], format: ImageFormat) -> Result<(u32, u32)> {
        match format {
            ImageFormat::Png => Self::png_dimensions(data),
            ImageFormat::Jpeg => Self::jpeg_dimensions(data),
            ImageFormat::Gif => Self::gif_dimensions(data),
            ImageFormat::Bmp => Self::bmp_dimensions(data),
            ImageFormat::WebP => Self::webp_dimensions(data),
            ImageFormat::Svg => Self::svg_dimensions(data),
            ImageFormat::Unknown => Ok((0, 0)),
        }
    }

    fn png_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        // PNG IHDR chunk starts at byte 16, width at 16, height at 20
        if data.len() < 24 {
            return Ok((0, 0));
        }
        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        Ok((width, height))
    }

    fn jpeg_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        // JPEG dimensions are in SOF0/SOF2 markers
        let mut i = 2;
        while i < data.len() - 9 {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                // SOF0, SOF1, SOF2 markers
                if marker == 0xC0 || marker == 0xC1 || marker == 0xC2 {
                    let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                    let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                    return Ok((width, height));
                }
                // Skip marker
                let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                i += 2 + length;
            } else {
                i += 1;
            }
        }
        Ok((0, 0))
    }

    fn gif_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        // GIF dimensions at bytes 6-7 (width) and 8-9 (height)
        if data.len() < 10 {
            return Ok((0, 0));
        }
        let width = u16::from_le_bytes([data[6], data[7]]) as u32;
        let height = u16::from_le_bytes([data[8], data[9]]) as u32;
        Ok((width, height))
    }

    fn bmp_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        // BMP dimensions at bytes 18-21 (width) and 22-25 (height)
        if data.len() < 26 {
            return Ok((0, 0));
        }
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        Ok((width, height.abs_diff(0))) // Height can be negative in BMP
    }

    fn webp_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        // WebP has complex format, simplified detection
        if data.len() < 30 {
            return Ok((0, 0));
        }
        // VP8 (lossy) format
        if data.len() > 23 && &data[12..16] == b"VP8 " {
            // Dimensions at bytes 26-27 (width) and 28-29 (height)
            if data.len() >= 30 {
                let width = u16::from_le_bytes([data[26], data[27]]) as u32 & 0x3FFF;
                let height = u16::from_le_bytes([data[28], data[29]]) as u32 & 0x3FFF;
                return Ok((width, height));
            }
        }
        Ok((0, 0))
    }

    fn svg_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        // SVG dimensions need XML parsing, return 0 for now
        // In a real implementation, we'd parse the SVG to get viewBox or width/height
        if let Ok(text) = std::str::from_utf8(data) {
            // Simple regex-free parsing for width and height attributes
            if let (Some(w), Some(h)) = (
                Self::extract_svg_attr(text, "width"),
                Self::extract_svg_attr(text, "height"),
            ) {
                return Ok((w, h));
            }
        }
        Ok((300, 150)) // Default SVG dimensions
    }

    fn extract_svg_attr(text: &str, attr: &str) -> Option<u32> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = text.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(end) = text[value_start..].find('"') {
                let value = &text[value_start..value_start + end];
                // Remove units like "px", "pt", etc.
                let numeric: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
                return numeric.parse().ok();
            }
        }
        None
    }

    /// Convert to data URL for frontend rendering
    pub fn to_data_url(&self) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let base64_data = STANDARD.encode(&self.data);
        format!("data:{};base64,{}", self.format.mime_type(), base64_data)
    }
}

/// Configuration for the image store
#[derive(Debug, Clone)]
pub struct ImageStoreConfig {
    /// Maximum image size in bytes (default: 10MB)
    pub max_size: usize,
    /// Maximum number of cached decoded images
    pub cache_size: usize,
    /// Whether to compress images on import
    pub compress_on_import: bool,
    /// Maximum dimension (width or height) for imported images
    pub max_dimension: u32,
}

impl Default for ImageStoreConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10MB
            cache_size: 50,
            compress_on_import: false,
            max_dimension: 4096,
        }
    }
}

/// Image resource manager
///
/// Stores and retrieves image data by resource ID. Thread-safe via RwLock.
#[derive(Debug)]
pub struct ImageStore {
    /// Stored images by resource ID
    images: RwLock<HashMap<String, Arc<ImageData>>>,
    /// Configuration
    config: ImageStoreConfig,
}

impl ImageStore {
    /// Create a new image store with default config
    pub fn new() -> Self {
        Self::with_config(ImageStoreConfig::default())
    }

    /// Create a new image store with custom config
    pub fn with_config(config: ImageStoreConfig) -> Self {
        Self {
            images: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Store image data and return its resource ID
    pub fn store_image(&self, data: Vec<u8>, filename: Option<String>) -> Result<ResourceId> {
        // Check size limit
        if data.len() > self.config.max_size {
            return Err(ImageStoreError::TooLarge {
                size: data.len(),
                max: self.config.max_size,
            });
        }

        let image_data = ImageData::new(data, filename)?;
        let resource_id = image_data.resource_id.clone();

        let mut images = self.images.write().unwrap();
        images.insert(resource_id.as_str().to_string(), Arc::new(image_data));

        Ok(resource_id)
    }

    /// Store image data with a specific resource ID
    pub fn store_image_with_id(
        &self,
        resource_id: ResourceId,
        data: Vec<u8>,
        filename: Option<String>,
    ) -> Result<()> {
        if data.len() > self.config.max_size {
            return Err(ImageStoreError::TooLarge {
                size: data.len(),
                max: self.config.max_size,
            });
        }

        let format = ImageFormat::from_bytes(&data);
        if !format.is_supported() {
            return Err(ImageStoreError::InvalidFormat(
                "Unknown or unsupported image format".into(),
            ));
        }

        let (width, height) = ImageData::detect_dimensions(&data, format)?;
        let size = data.len();

        let image_data = ImageData {
            resource_id: resource_id.clone(),
            data,
            format,
            width,
            height,
            size,
            filename,
        };

        let mut images = self.images.write().unwrap();
        images.insert(resource_id.as_str().to_string(), Arc::new(image_data));

        Ok(())
    }

    /// Get image data by resource ID
    pub fn get_image(&self, resource_id: &ResourceId) -> Result<Arc<ImageData>> {
        let images = self.images.read().unwrap();
        images
            .get(resource_id.as_str())
            .cloned()
            .ok_or_else(|| ImageStoreError::NotFound(resource_id.to_string()))
    }

    /// Check if an image exists
    pub fn contains(&self, resource_id: &ResourceId) -> bool {
        let images = self.images.read().unwrap();
        images.contains_key(resource_id.as_str())
    }

    /// Remove an image by resource ID
    pub fn remove_image(&self, resource_id: &ResourceId) -> Result<Arc<ImageData>> {
        let mut images = self.images.write().unwrap();
        images
            .remove(resource_id.as_str())
            .ok_or_else(|| ImageStoreError::NotFound(resource_id.to_string()))
    }

    /// Get the number of stored images
    pub fn len(&self) -> usize {
        let images = self.images.read().unwrap();
        images.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all resource IDs
    pub fn resource_ids(&self) -> Vec<ResourceId> {
        let images = self.images.read().unwrap();
        images.keys().map(|k| ResourceId::new(k.clone())).collect()
    }

    /// Clear all stored images
    pub fn clear(&self) {
        let mut images = self.images.write().unwrap();
        images.clear();
    }

    /// Get the total size of all stored images
    pub fn total_size(&self) -> usize {
        let images = self.images.read().unwrap();
        images.values().map(|img| img.size).sum()
    }

    /// Get image as data URL for frontend
    pub fn get_data_url(&self, resource_id: &ResourceId) -> Result<String> {
        let image = self.get_image(resource_id)?;
        Ok(image.to_data_url())
    }
}

impl Default for ImageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ImageStore {
    fn clone(&self) -> Self {
        let images = self.images.read().unwrap();
        Self {
            images: RwLock::new(images.clone()),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid PNG (1x1 pixel, transparent)
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn test_format_detection() {
        assert_eq!(ImageFormat::from_bytes(TINY_PNG), ImageFormat::Png);

        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(ImageFormat::from_bytes(&jpeg_header), ImageFormat::Jpeg);

        let gif_header = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61];
        assert_eq!(ImageFormat::from_bytes(&gif_header), ImageFormat::Gif);
    }

    #[test]
    fn test_store_and_retrieve() {
        let store = ImageStore::new();

        let resource_id = store.store_image(TINY_PNG.to_vec(), Some("test.png".into())).unwrap();

        let image = store.get_image(&resource_id).unwrap();
        assert_eq!(image.format, ImageFormat::Png);
        assert_eq!(image.width, 1);
        assert_eq!(image.height, 1);
        assert_eq!(image.filename, Some("test.png".into()));
    }

    #[test]
    fn test_png_dimensions() {
        let image_data = ImageData::new(TINY_PNG.to_vec(), None).unwrap();
        assert_eq!(image_data.width, 1);
        assert_eq!(image_data.height, 1);
    }

    #[test]
    fn test_size_limit() {
        let config = ImageStoreConfig {
            max_size: 100,
            ..Default::default()
        };
        let store = ImageStore::with_config(config);

        let large_data = vec![0u8; 200];
        let result = store.store_image(large_data, None);
        assert!(matches!(result, Err(ImageStoreError::TooLarge { .. })));
    }

    #[test]
    fn test_data_url() {
        let store = ImageStore::new();
        let resource_id = store.store_image(TINY_PNG.to_vec(), None).unwrap();

        let data_url = store.get_data_url(&resource_id).unwrap();
        assert!(data_url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_remove_image() {
        let store = ImageStore::new();
        let resource_id = store.store_image(TINY_PNG.to_vec(), None).unwrap();

        assert!(store.contains(&resource_id));
        store.remove_image(&resource_id).unwrap();
        assert!(!store.contains(&resource_id));
    }
}
