//! Media writer for DOCX files
//!
//! Handles embedding images and other media files.

use crate::docx::error::DocxResult;
use doc_model::DocumentTree;
use std::collections::HashMap;
use std::io::{Seek, Write};

/// Writer for media files (images)
pub struct MediaWriter;

impl MediaWriter {
    /// Create a new media writer
    pub fn new() -> Self {
        Self
    }

    /// Write all media files to the DOCX archive
    /// Returns a map of relationship IDs to paths
    pub fn write_media<W: Write + Seek>(
        &self,
        tree: &DocumentTree,
        _writer: &mut crate::docx::writer::DocxWriter<W>,
    ) -> DocxResult<HashMap<String, String>> {
        let mut relationships = HashMap::new();

        // Collect all images from the document
        // For now, we don't store image data in the tree, so this is a placeholder
        // In a full implementation, we would:
        // 1. Iterate through all ImageNode instances in the tree
        // 2. Look up the image data from the image store
        // 3. Write each image to word/media/
        // 4. Create relationships for each image

        // TODO: Implement when image store integration is complete
        // for (id, image) in tree.nodes.images.iter() {
        //     let image_data = image_store.get(&image.resource_id)?;
        //     let filename = generate_media_filename(&image.resource_id, &image_data.content_type);
        //     let path = format!("word/media/{}", filename);
        //     writer.write_binary(&path, &image_data.data)?;
        //
        //     let rel_id = writer.doc_rels_mut().add(
        //         relationship_types::IMAGE,
        //         &format!("media/{}", filename),
        //         TargetMode::Internal,
        //     );
        //     relationships.insert(image.resource_id.as_str().to_string(), rel_id);
        // }

        Ok(relationships)
    }

    /// Generate a filename for a media file based on content type
    pub fn generate_filename(resource_id: &str, content_type: &str) -> String {
        let extension = match content_type {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpeg",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/tiff" => "tiff",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => "bin",
        };

        // Create a safe filename from the resource ID
        let safe_id: String = resource_id
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(32)
            .collect();

        if safe_id.is_empty() {
            format!("image1.{}", extension)
        } else {
            format!("{}.{}", safe_id, extension)
        }
    }

    /// Determine content type from file extension
    pub fn content_type_from_extension(path: &str) -> &'static str {
        let ext = path.rsplit('.').next().unwrap_or("");
        match ext.to_lowercase().as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "tiff" | "tif" => "image/tiff",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        }
    }
}

/// Generate a w:drawing element for an inline image
pub fn generate_inline_drawing(
    rel_id: &str,
    width_emu: i64,
    height_emu: i64,
    name: &str,
    alt_text: Option<&str>,
) -> String {
    let alt = alt_text.unwrap_or(name);

    format!(
        r#"<w:drawing>
    <wp:inline distT="0" distB="0" distL="0" distR="0">
        <wp:extent cx="{}" cy="{}"/>
        <wp:effectExtent l="0" t="0" r="0" b="0"/>
        <wp:docPr id="1" name="{}" descr="{}"/>
        <wp:cNvGraphicFramePr>
            <a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/>
        </wp:cNvGraphicFramePr>
        <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
            <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
                <pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
                    <pic:nvPicPr>
                        <pic:cNvPr id="0" name="{}"/>
                        <pic:cNvPicPr/>
                    </pic:nvPicPr>
                    <pic:blipFill>
                        <a:blip r:embed="{}"/>
                        <a:stretch>
                            <a:fillRect/>
                        </a:stretch>
                    </pic:blipFill>
                    <pic:spPr>
                        <a:xfrm>
                            <a:off x="0" y="0"/>
                            <a:ext cx="{}" cy="{}"/>
                        </a:xfrm>
                        <a:prstGeom prst="rect">
                            <a:avLst/>
                        </a:prstGeom>
                    </pic:spPr>
                </pic:pic>
            </a:graphicData>
        </a:graphic>
    </wp:inline>
</w:drawing>"#,
        width_emu, height_emu,
        escape_xml(name), escape_xml(alt),
        escape_xml(name),
        rel_id,
        width_emu, height_emu
    )
}

/// Convert points to EMUs (English Metric Units)
/// 1 inch = 914400 EMUs, 1 point = 12700 EMUs
pub fn points_to_emu(points: f32) -> i64 {
    (points as f64 * 12700.0).round() as i64
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename() {
        assert_eq!(
            MediaWriter::generate_filename("img123", "image/png"),
            "img123.png"
        );
        assert_eq!(
            MediaWriter::generate_filename("photo", "image/jpeg"),
            "photo.jpeg"
        );
        assert_eq!(
            MediaWriter::generate_filename("", "image/gif"),
            "image1.gif"
        );
    }

    #[test]
    fn test_content_type_from_extension() {
        assert_eq!(MediaWriter::content_type_from_extension("image.png"), "image/png");
        assert_eq!(MediaWriter::content_type_from_extension("photo.JPG"), "image/jpeg");
        assert_eq!(MediaWriter::content_type_from_extension("image.gif"), "image/gif");
        assert_eq!(MediaWriter::content_type_from_extension("file.unknown"), "application/octet-stream");
    }

    #[test]
    fn test_points_to_emu() {
        // 72 points = 1 inch = 914400 EMU
        assert_eq!(points_to_emu(72.0), 914400);
        // 36 points = 0.5 inch = 457200 EMU
        assert_eq!(points_to_emu(36.0), 457200);
    }

    #[test]
    fn test_generate_inline_drawing() {
        let xml = generate_inline_drawing("rId1", 914400, 914400, "test.png", Some("Test image"));
        assert!(xml.contains("w:drawing"));
        assert!(xml.contains("wp:inline"));
        assert!(xml.contains("r:embed=\"rId1\""));
    }
}
