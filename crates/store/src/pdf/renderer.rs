//! Layout to PDF Conversion
//!
//! This module converts the render model (RenderPage, RenderItem) to PDF
//! content streams and page objects.

use super::content::ContentStream;
use super::document::{MediaBox, PdfPage};
use super::fonts::{FontKey, FontManager, StandardFont};
use super::images::ImageManager;
use super::options::PdfExportOptions;

/// A color in RGB format (0.0 to 1.0)
#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl RgbColor {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

/// Text rendering info
#[derive(Debug, Clone)]
pub struct TextRenderInfo {
    /// The text to render
    pub text: String,
    /// X position (in points, from left)
    pub x: f64,
    /// Y position (in points, from top of page in layout coordinates)
    pub y: f64,
    /// Font family name
    pub font_family: String,
    /// Font size in points
    pub font_size: f64,
    /// Is bold
    pub bold: bool,
    /// Is italic
    pub italic: bool,
    /// Text color
    pub color: RgbColor,
}

/// Line rendering info
#[derive(Debug, Clone, Copy)]
pub struct LineRenderInfo {
    /// Start X position
    pub x1: f64,
    /// Start Y position
    pub y1: f64,
    /// End X position
    pub x2: f64,
    /// End Y position
    pub y2: f64,
    /// Line color
    pub color: RgbColor,
    /// Line width
    pub width: f64,
}

/// Rectangle rendering info
#[derive(Debug, Clone, Copy)]
pub struct RectRenderInfo {
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
    /// Fill color (optional)
    pub fill: Option<RgbColor>,
    /// Stroke color (optional)
    pub stroke: Option<RgbColor>,
    /// Stroke width
    pub stroke_width: f64,
}

/// Image rendering info
#[derive(Debug, Clone)]
pub struct ImageRenderInfo {
    /// Image resource ID
    pub resource_id: String,
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Display width
    pub width: f64,
    /// Display height
    pub height: f64,
}

/// Abstract render item for PDF generation
#[derive(Debug, Clone)]
pub enum PdfRenderItem {
    /// Text element
    Text(TextRenderInfo),
    /// Line element
    Line(LineRenderInfo),
    /// Rectangle element
    Rectangle(RectRenderInfo),
    /// Image element
    Image(ImageRenderInfo),
}

/// Page rendering info
#[derive(Debug, Clone)]
pub struct PageRenderInfo {
    /// Page width in points
    pub width: f64,
    /// Page height in points
    pub height: f64,
    /// Render items on this page
    pub items: Vec<PdfRenderItem>,
}

impl PageRenderInfo {
    /// Create a new page with the given dimensions
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            items: Vec::new(),
        }
    }

    /// Add a render item
    pub fn add_item(&mut self, item: PdfRenderItem) {
        self.items.push(item);
    }
}

/// PDF page renderer
pub struct PdfRenderer {
    /// Font manager
    font_manager: FontManager,
    /// Image manager
    image_manager: ImageManager,
    /// Export options
    options: PdfExportOptions,
}

impl PdfRenderer {
    /// Create a new renderer
    pub fn new(options: PdfExportOptions) -> Self {
        Self {
            font_manager: FontManager::new(),
            image_manager: ImageManager::new(),
            options,
        }
    }

    /// Get the font manager
    pub fn font_manager(&self) -> &FontManager {
        &self.font_manager
    }

    /// Get mutable font manager
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// Get the image manager
    pub fn image_manager(&self) -> &ImageManager {
        &self.image_manager
    }

    /// Get mutable image manager
    pub fn image_manager_mut(&mut self) -> &mut ImageManager {
        &mut self.image_manager
    }

    /// Render a page to a content stream
    pub fn render_page(&mut self, page_info: &PageRenderInfo) -> ContentStream {
        let mut content = ContentStream::new();
        let page_height = page_info.height;

        // Group items by type for more efficient rendering
        let mut texts: Vec<&TextRenderInfo> = Vec::new();
        let mut graphics: Vec<&PdfRenderItem> = Vec::new();

        for item in &page_info.items {
            match item {
                PdfRenderItem::Text(text) => texts.push(text),
                _ => graphics.push(item),
            }
        }

        // Render graphics first (backgrounds, lines, etc.)
        for item in &graphics {
            match item {
                PdfRenderItem::Rectangle(rect) => {
                    self.render_rectangle(&mut content, rect, page_height);
                }
                PdfRenderItem::Line(line) => {
                    self.render_line(&mut content, line, page_height);
                }
                PdfRenderItem::Image(image) => {
                    self.render_image(&mut content, image, page_height);
                }
                _ => {}
            }
        }

        // Render text
        if !texts.is_empty() {
            content.begin_text();

            let mut current_font: Option<(String, f64)> = None;
            let mut current_color: Option<RgbColor> = None;

            for text in texts {
                // Update font if needed
                let font_key = FontKey::new(&text.font_family, text.bold, text.italic);
                let font_info = self.font_manager.get_or_create_font(&font_key);
                let font_name = font_info.name.clone();

                let should_update_font = match &current_font {
                    Some((name, size)) => name != &font_name || *size != text.font_size,
                    None => true,
                };

                if should_update_font {
                    content.set_font(&font_name, text.font_size);
                    current_font = Some((font_name, text.font_size));
                }

                // Update color if needed
                if current_color.map(|c| (c.r, c.g, c.b)) != Some((text.color.r, text.color.g, text.color.b)) {
                    content.set_fill_rgb(text.color.r, text.color.g, text.color.b);
                    current_color = Some(text.color);
                }

                // Convert Y coordinate (PDF origin is bottom-left)
                let pdf_y = page_height - text.y;

                // Position and show text
                content.set_text_matrix(1.0, 0.0, 0.0, 1.0, text.x, pdf_y);
                content.show_text(&text.text);
            }

            content.end_text();
        }

        content
    }

    /// Render a rectangle
    fn render_rectangle(&self, content: &mut ContentStream, rect: &RectRenderInfo, page_height: f64) {
        content.save_state();

        // Convert Y coordinate
        let pdf_y = page_height - rect.y - rect.height;

        // Set fill color if present
        if let Some(fill) = rect.fill {
            content.set_fill_rgb(fill.r, fill.g, fill.b);
        }

        // Set stroke color if present
        if let Some(stroke) = rect.stroke {
            content.set_stroke_rgb(stroke.r, stroke.g, stroke.b);
            content.set_line_width(rect.stroke_width);
        }

        // Draw rectangle
        content.rect(rect.x, pdf_y, rect.width, rect.height);

        // Fill and/or stroke
        match (rect.fill.is_some(), rect.stroke.is_some()) {
            (true, true) => { content.fill_and_stroke(); }
            (true, false) => { content.fill(); }
            (false, true) => { content.stroke(); }
            (false, false) => { content.end_path(); }
        }

        content.restore_state();
    }

    /// Render a line
    fn render_line(&self, content: &mut ContentStream, line: &LineRenderInfo, page_height: f64) {
        content.save_state();

        // Convert Y coordinates
        let pdf_y1 = page_height - line.y1;
        let pdf_y2 = page_height - line.y2;

        content.set_stroke_rgb(line.color.r, line.color.g, line.color.b);
        content.set_line_width(line.width);
        content.move_to(line.x1, pdf_y1);
        content.line_to(line.x2, pdf_y2);
        content.stroke();

        content.restore_state();
    }

    /// Render an image
    fn render_image(&self, content: &mut ContentStream, image: &ImageRenderInfo, page_height: f64) {
        content.save_state();

        // Convert Y coordinate
        let pdf_y = page_height - image.y - image.height;

        // Apply transformation to scale and position the image
        // Images are rendered at 1x1 unit size, so we need to scale
        content.transform(
            image.width, 0.0,
            0.0, image.height,
            image.x, pdf_y
        );

        // Draw the image XObject
        // The image name would be looked up from the image manager
        // For now, we use a placeholder
        content.draw_xobject(&format!("Im_{}", image.resource_id));

        content.restore_state();
    }

    /// Create a PDF page object from page info
    pub fn create_page_object(&self, page_info: &PageRenderInfo) -> PdfPage {
        let mut page = PdfPage::new(MediaBox::from_dimensions(page_info.width, page_info.height));

        // Add font resources
        for font in self.font_manager.fonts() {
            // The actual font object reference will be set by the writer
            // For now, we use a placeholder (0)
            page.add_font(&font.name, 0);
        }

        page
    }
}

/// Convert render_model types to PDF render items
pub mod convert {
    use super::*;

    /// Convert a render_model::Color to RgbColor
    pub fn convert_color(color: &render_model::Color) -> RgbColor {
        RgbColor::from_u8(color.r, color.g, color.b)
    }

    /// Convert a render_model::GlyphRun to TextRenderInfo
    pub fn convert_glyph_run(glyph: &render_model::GlyphRun) -> TextRenderInfo {
        TextRenderInfo {
            text: glyph.text.clone(),
            x: glyph.x,
            y: glyph.y,
            font_family: glyph.font_family.clone(),
            font_size: glyph.font_size,
            bold: glyph.bold,
            italic: glyph.italic,
            color: convert_color(&glyph.color),
        }
    }

    /// Convert a render_model::RenderItem to PdfRenderItem(s)
    pub fn convert_render_item(item: &render_model::RenderItem) -> Vec<PdfRenderItem> {
        match item {
            render_model::RenderItem::GlyphRun(glyph) => {
                vec![PdfRenderItem::Text(convert_glyph_run(glyph))]
            }
            render_model::RenderItem::Rectangle { bounds, fill, stroke, stroke_width } => {
                vec![PdfRenderItem::Rectangle(RectRenderInfo {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                    fill: fill.map(|c| convert_color(&c)),
                    stroke: stroke.map(|c| convert_color(&c)),
                    stroke_width: *stroke_width,
                })]
            }
            render_model::RenderItem::Line { x1, y1, x2, y2, color, width } => {
                vec![PdfRenderItem::Line(LineRenderInfo {
                    x1: *x1,
                    y1: *y1,
                    x2: *x2,
                    y2: *y2,
                    color: convert_color(color),
                    width: *width,
                })]
            }
            render_model::RenderItem::Image(img) => {
                vec![PdfRenderItem::Image(ImageRenderInfo {
                    resource_id: img.resource_id.clone(),
                    x: img.bounds.x,
                    y: img.bounds.y,
                    width: img.bounds.width,
                    height: img.bounds.height,
                })]
            }
            render_model::RenderItem::TableBorder(border) => {
                vec![PdfRenderItem::Line(LineRenderInfo {
                    x1: border.x1,
                    y1: border.y1,
                    x2: border.x2,
                    y2: border.y2,
                    color: convert_color(&border.color),
                    width: border.width,
                })]
            }
            render_model::RenderItem::TableCell(cell) => {
                if let Some(bg) = &cell.background {
                    vec![PdfRenderItem::Rectangle(RectRenderInfo {
                        x: cell.bounds.x,
                        y: cell.bounds.y,
                        width: cell.bounds.width,
                        height: cell.bounds.height,
                        fill: Some(convert_color(bg)),
                        stroke: None,
                        stroke_width: 0.0,
                    })]
                } else {
                    vec![]
                }
            }
            // Skip UI-only elements
            render_model::RenderItem::Caret { .. } |
            render_model::RenderItem::Selection { .. } |
            render_model::RenderItem::Squiggly(_) |
            render_model::RenderItem::FindHighlight { .. } => {
                vec![]
            }
            render_model::RenderItem::Shape(shape) => {
                // Basic shape rendering - just render the bounding box for now
                let fill = match &shape.fill {
                    Some(render_model::ShapeFillRender::Solid { color }) => Some(convert_color(color)),
                    _ => None,
                };
                let stroke = shape.stroke.as_ref().map(|s| convert_color(&s.color));
                let stroke_width = shape.stroke.as_ref().map(|s| s.width).unwrap_or(1.0);

                vec![PdfRenderItem::Rectangle(RectRenderInfo {
                    x: shape.bounds.x,
                    y: shape.bounds.y,
                    width: shape.bounds.width,
                    height: shape.bounds.height,
                    fill,
                    stroke,
                    stroke_width,
                })]
            }
            render_model::RenderItem::TextBox(textbox) => {
                // Render text box as a rectangle for now
                // TODO: Implement full text box content rendering
                let fill = match &textbox.fill {
                    Some(render_model::TextBoxFillRender::Solid { color }) => Some(convert_color(color)),
                    _ => None,
                };
                let (stroke, stroke_width) = if let Some(border) = &textbox.border {
                    (Some(convert_color(&border.top.color)), border.top.width)
                } else {
                    (None, 1.0)
                };

                vec![PdfRenderItem::Rectangle(RectRenderInfo {
                    x: textbox.bounds.x,
                    y: textbox.bounds.y,
                    width: textbox.bounds.width,
                    height: textbox.bounds.height,
                    fill,
                    stroke,
                    stroke_width,
                })]
            }
            render_model::RenderItem::LineNumber(info) => {
                // Render line number as text
                vec![PdfRenderItem::Text(TextRenderInfo {
                    text: info.number.to_string(),
                    x: info.x,
                    y: info.y,
                    font_family: info.font_family.clone(),
                    font_size: info.font_size,
                    bold: false,
                    italic: false,
                    color: convert_color(&info.color),
                })]
            }
        }
    }

    /// Convert a render_model::PageRender to PageRenderInfo
    pub fn convert_page(page: &render_model::PageRender) -> PageRenderInfo {
        let mut page_info = PageRenderInfo::new(page.width, page.height);

        for item in &page.items {
            for pdf_item in convert_render_item(item) {
                page_info.add_item(pdf_item);
            }
        }

        page_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_color() {
        let color = RgbColor::from_u8(255, 128, 0);
        assert_eq!(color.r, 1.0);
        assert!((color.g - 0.502).abs() < 0.01);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_page_render_info() {
        let mut page = PageRenderInfo::new(612.0, 792.0);
        page.add_item(PdfRenderItem::Text(TextRenderInfo {
            text: "Hello".to_string(),
            x: 72.0,
            y: 720.0,
            font_family: "Helvetica".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: RgbColor::black(),
        }));

        assert_eq!(page.width, 612.0);
        assert_eq!(page.height, 792.0);
        assert_eq!(page.items.len(), 1);
    }

    #[test]
    fn test_renderer_basic() {
        let options = PdfExportOptions::default();
        let mut renderer = PdfRenderer::new(options);

        let mut page = PageRenderInfo::new(612.0, 792.0);
        page.add_item(PdfRenderItem::Text(TextRenderInfo {
            text: "Test".to_string(),
            x: 72.0,
            y: 720.0,
            font_family: "Arial".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: RgbColor::black(),
        }));

        let content = renderer.render_page(&page);
        let content_str = String::from_utf8(content.into_bytes()).unwrap();

        assert!(content_str.contains("BT")); // Begin text
        assert!(content_str.contains("ET")); // End text
        assert!(content_str.contains("Tf")); // Set font
        assert!(content_str.contains("Tj")); // Show text
    }

    #[test]
    fn test_renderer_rectangle() {
        let options = PdfExportOptions::default();
        let mut renderer = PdfRenderer::new(options);

        let mut page = PageRenderInfo::new(612.0, 792.0);
        page.add_item(PdfRenderItem::Rectangle(RectRenderInfo {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 50.0,
            fill: Some(RgbColor::new(1.0, 0.0, 0.0)),
            stroke: None,
            stroke_width: 1.0,
        }));

        let content = renderer.render_page(&page);
        let content_str = String::from_utf8(content.into_bytes()).unwrap();

        assert!(content_str.contains("q")); // Save state
        assert!(content_str.contains("Q")); // Restore state
        assert!(content_str.contains("re")); // Rectangle
        assert!(content_str.contains("rg")); // Set fill color
    }
}
