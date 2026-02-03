//! Image parsing for DOCX files
//!
//! Handles w:drawing and embedded image elements.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::parser::ImageData;
use crate::docx::reader::XmlParser;
use doc_model::{
    AnchorPosition, Dimension, DocumentTree, HorizontalAnchor, ImageNode, ImagePosition,
    ImageProperties, ResourceId, VerticalAnchor, WrapType,
};
use quick_xml::events::Event;

/// Parser for image elements
pub struct ImageParser;

impl ImageParser {
    /// Create a new image parser
    pub fn new() -> Self {
        Self
    }

    /// Process an image and add it to the tree if referenced
    pub fn process_image(
        &self,
        _rel_id: &str,
        image_data: &ImageData,
        tree: &mut DocumentTree,
    ) -> DocxResult<()> {
        // Images are processed when encountered in the document content
        // This method is for pre-processing image data
        // The actual insertion happens when parsing w:drawing elements
        Ok(())
    }

    /// Parse a w:drawing element and return image properties
    pub fn parse_drawing(&self, content: &str) -> DocxResult<Option<ParsedImage>> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut parsed = ParsedImage::default();
        let mut in_inline = false;
        let mut in_anchor = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "inline") {
                        in_inline = true;
                        parsed.position = ImagePosition::Inline;
                    } else if XmlParser::matches_element(name_ref, "anchor") {
                        in_anchor = true;
                        parsed.position = ImagePosition::Anchor(AnchorPosition::default());
                    } else if XmlParser::matches_element(name_ref, "extent") {
                        // Dimensions in EMUs
                        if let Some(cx) = XmlParser::get_attribute(e, b"cx") {
                            if let Some(width) = XmlParser::parse_emu(&cx) {
                                parsed.width = Some(width);
                            }
                        }
                        if let Some(cy) = XmlParser::get_attribute(e, b"cy") {
                            if let Some(height) = XmlParser::parse_emu(&cy) {
                                parsed.height = Some(height);
                            }
                        }
                    } else if XmlParser::matches_element(name_ref, "blip") {
                        // Image relationship reference
                        if let Some(embed) = XmlParser::get_r_attribute(e, "embed") {
                            parsed.rel_id = Some(embed);
                        }
                    } else if XmlParser::matches_element(name_ref, "docPr") {
                        // Document properties (alt text, title)
                        if let Some(descr) = XmlParser::get_attribute(e, b"descr") {
                            parsed.alt_text = Some(descr);
                        }
                        if let Some(name) = XmlParser::get_attribute(e, b"name") {
                            parsed.title = Some(name);
                        }
                    } else if in_anchor {
                        self.parse_anchor_properties(e, &mut parsed)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "inline") {
                        in_inline = false;
                    } else if XmlParser::matches_element(name_ref, "anchor") {
                        in_anchor = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        if parsed.rel_id.is_some() {
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Parse anchor-specific properties
    fn parse_anchor_properties(
        &self,
        e: &quick_xml::events::BytesStart,
        parsed: &mut ParsedImage,
    ) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "positionH") {
            if let Some(rel_from) = XmlParser::get_attribute(e, b"relativeFrom") {
                parsed.h_anchor = Some(parse_horizontal_anchor(&rel_from));
            }
        } else if XmlParser::matches_element(name_ref, "positionV") {
            if let Some(rel_from) = XmlParser::get_attribute(e, b"relativeFrom") {
                parsed.v_anchor = Some(parse_vertical_anchor(&rel_from));
            }
        } else if XmlParser::matches_element(name_ref, "posOffset") {
            // Position offset in EMUs - handled by parent context
        } else if XmlParser::matches_element(name_ref, "wrapSquare") {
            parsed.wrap_type = WrapType::Square;
        } else if XmlParser::matches_element(name_ref, "wrapTight") {
            parsed.wrap_type = WrapType::Tight;
        } else if XmlParser::matches_element(name_ref, "wrapNone") {
            parsed.wrap_type = WrapType::InFront;
        } else if XmlParser::matches_element(name_ref, "behindDoc") {
            let val = XmlParser::get_attribute(e, b"val")
                .map(|v| XmlParser::parse_bool(&v))
                .unwrap_or(true);
            if val {
                parsed.wrap_type = WrapType::Behind;
            }
        }

        Ok(())
    }

    /// Create an ImageNode from parsed data
    pub fn create_image_node(&self, parsed: &ParsedImage, image_data: &ImageData) -> ImageNode {
        let resource_id = ResourceId::new(&image_data.rel_id);

        // Try to determine original dimensions from image data
        let (orig_width, orig_height) = self.get_image_dimensions(&image_data.data, &image_data.content_type);

        let mut node = ImageNode::new(resource_id, orig_width, orig_height);

        // Set dimensions
        let mut props = ImageProperties::new();
        if let Some(width) = parsed.width {
            props.width = Dimension::points(width);
        }
        if let Some(height) = parsed.height {
            props.height = Dimension::points(height);
        }

        // Set wrap type and position
        props.wrap_type = parsed.wrap_type;
        props.position = parsed.position;

        node.set_properties(props);

        // Set metadata
        if let Some(ref alt_text) = parsed.alt_text {
            node.set_alt_text(alt_text);
        }
        if let Some(ref title) = parsed.title {
            node.set_title(title);
        }

        node
    }

    /// Get image dimensions from binary data
    fn get_image_dimensions(&self, data: &[u8], content_type: &str) -> (u32, u32) {
        // Simple dimension detection for common formats
        match content_type {
            "image/png" => self.get_png_dimensions(data),
            "image/jpeg" | "image/jpg" => self.get_jpeg_dimensions(data),
            "image/gif" => self.get_gif_dimensions(data),
            _ => (100, 100), // Default fallback
        }
    }

    /// Get PNG dimensions from header
    fn get_png_dimensions(&self, data: &[u8]) -> (u32, u32) {
        // PNG header: 8 bytes signature, then IHDR chunk
        // IHDR: 4 bytes length, 4 bytes "IHDR", 4 bytes width, 4 bytes height
        if data.len() >= 24 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
            let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
            let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            return (width, height);
        }
        (100, 100)
    }

    /// Get JPEG dimensions from header
    fn get_jpeg_dimensions(&self, data: &[u8]) -> (u32, u32) {
        // JPEG: Look for SOF0 marker (0xFF 0xC0) or SOF2 (0xFF 0xC2)
        let mut i = 0;
        while i + 9 < data.len() {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                if marker == 0xC0 || marker == 0xC2 {
                    // SOF0/SOF2: skip 2 bytes marker, 2 bytes length, 1 byte precision
                    let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                    let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                    return (width, height);
                } else if marker != 0xFF && marker != 0x00 && marker != 0xD8 && marker != 0xD9 {
                    // Skip to next marker
                    if i + 3 < data.len() {
                        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                        i += 2 + len;
                        continue;
                    }
                }
            }
            i += 1;
        }
        (100, 100)
    }

    /// Get GIF dimensions from header
    fn get_gif_dimensions(&self, data: &[u8]) -> (u32, u32) {
        // GIF: "GIF87a" or "GIF89a" followed by 2-byte width and 2-byte height (little-endian)
        if data.len() >= 10 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
            let width = u16::from_le_bytes([data[6], data[7]]) as u32;
            let height = u16::from_le_bytes([data[8], data[9]]) as u32;
            return (width, height);
        }
        (100, 100)
    }
}

/// Parsed image data
#[derive(Debug, Default)]
pub struct ParsedImage {
    pub rel_id: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub wrap_type: WrapType,
    pub position: ImagePosition,
    pub h_anchor: Option<HorizontalAnchor>,
    pub v_anchor: Option<VerticalAnchor>,
    pub offset_x: f32,
    pub offset_y: f32,
}

/// Parse horizontal anchor type
fn parse_horizontal_anchor(value: &str) -> HorizontalAnchor {
    match value {
        "page" => HorizontalAnchor::Page,
        "margin" => HorizontalAnchor::Margin,
        "character" => HorizontalAnchor::Character,
        _ => HorizontalAnchor::Column,
    }
}

/// Parse vertical anchor type
fn parse_vertical_anchor(value: &str) -> VerticalAnchor {
    match value {
        "page" => VerticalAnchor::Page,
        "margin" => VerticalAnchor::Margin,
        "line" => VerticalAnchor::Line,
        _ => VerticalAnchor::Paragraph,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_dimensions() {
        let parser = ImageParser::new();

        // Valid PNG header with 100x50 dimensions
        let mut png_data = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png_data.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
        png_data.extend_from_slice(b"IHDR");
        png_data.extend_from_slice(&100u32.to_be_bytes()); // width
        png_data.extend_from_slice(&50u32.to_be_bytes()); // height

        let (width, height) = parser.get_png_dimensions(&png_data);
        assert_eq!(width, 100);
        assert_eq!(height, 50);
    }

    #[test]
    fn test_gif_dimensions() {
        let parser = ImageParser::new();

        // Valid GIF89a header with 200x150 dimensions
        let mut gif_data = b"GIF89a".to_vec();
        gif_data.extend_from_slice(&200u16.to_le_bytes()); // width
        gif_data.extend_from_slice(&150u16.to_le_bytes()); // height

        let (width, height) = parser.get_gif_dimensions(&gif_data);
        assert_eq!(width, 200);
        assert_eq!(height, 150);
    }

    #[test]
    fn test_parse_horizontal_anchor() {
        assert_eq!(parse_horizontal_anchor("column"), HorizontalAnchor::Column);
        assert_eq!(parse_horizontal_anchor("page"), HorizontalAnchor::Page);
        assert_eq!(parse_horizontal_anchor("margin"), HorizontalAnchor::Margin);
        assert_eq!(parse_horizontal_anchor("character"), HorizontalAnchor::Character);
    }

    #[test]
    fn test_parse_vertical_anchor() {
        assert_eq!(parse_vertical_anchor("paragraph"), VerticalAnchor::Paragraph);
        assert_eq!(parse_vertical_anchor("page"), VerticalAnchor::Page);
        assert_eq!(parse_vertical_anchor("margin"), VerticalAnchor::Margin);
        assert_eq!(parse_vertical_anchor("line"), VerticalAnchor::Line);
    }
}
