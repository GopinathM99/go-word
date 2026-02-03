//! Drawings Import/Export for DOCX
//!
//! Handles DrawingML elements including:
//! - Text boxes (w:txbxContent in drawing ML)
//! - Shapes with geometry
//! - Shape groups (wpg:wgp)
//! - Connectors between shapes
//! - Shape effects (shadow, 3D, glow)
//! - Gradient and pattern fills

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::namespaces;
use crate::docx::reader::XmlParser;
use doc_model::{HorizontalAnchor, VerticalAnchor, WrapType};
use quick_xml::events::Event;
use std::collections::HashMap;

// Local color type
#[derive(Debug, Clone, Copy)]
pub struct ShapeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ShapeColor {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Self { r, g, b, a: 255 })
        } else {
            None
        }
    }
}

// Local shape type enum (simplified)
#[derive(Debug, Clone, Default)]
pub enum ShapeType {
    #[default]
    Rectangle,
    RoundedRectangle,
    Oval,
    Triangle,
    Diamond,
    Pentagon,
    Hexagon,
    Octagon,
    Star4,
    Star5,
    Star6,
    Star8,
    Star10,
    Star12,
    Line,
    RightArrowBlock,
    LeftArrowBlock,
    UpArrowBlock,
    DownArrowBlock,
    FlowchartProcess,
    FlowchartDecision,
    FlowchartTerminator,
    FlowchartDocument,
    RectangularCallout,
    CloudCallout,
    Heart,
    LightningBolt,
    Sun,
    Moon,
    Cloud,
    Custom(String),
}

// =============================================================================
// Drawing Parser
// =============================================================================

/// Parser for drawings in DOCX
pub struct DrawingParser {
    /// Parsed drawings
    drawings: Vec<ParsedDrawing>,
}

impl DrawingParser {
    /// Create a new drawing parser
    pub fn new() -> Self {
        Self {
            drawings: Vec::new(),
        }
    }

    /// Parse a wp:drawing element
    pub fn parse_drawing(&mut self, content: &str) -> DocxResult<ParsedDrawing> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        let mut drawing = ParsedDrawing::default();
        let mut in_position_h = false;
        let mut in_position_v = false;
        let mut in_txbx = false;
        let mut in_sp_pr = false;
        let mut in_text = false;
        let mut text_content = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    // Anchor element (floating drawing)
                    if XmlParser::matches_element(name_ref, "anchor") {
                        drawing.is_inline = false;
                        drawing.allow_overlap = XmlParser::get_attribute(e, b"allowOverlap")
                            .map(|s| s == "1")
                            .unwrap_or(false);
                        drawing.behind_doc = XmlParser::get_attribute(e, b"behindDoc")
                            .map(|s| s == "1")
                            .unwrap_or(false);
                        drawing.layout_in_cell = XmlParser::get_attribute(e, b"layoutInCell")
                            .map(|s| s == "1")
                            .unwrap_or(true);
                        drawing.relative_height = XmlParser::get_attribute(e, b"relativeHeight")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    }
                    // Inline element
                    else if XmlParser::matches_element(name_ref, "inline") {
                        drawing.is_inline = true;
                    }
                    // Extent (size)
                    else if XmlParser::matches_element(name_ref, "extent") {
                        if let Some(cx) = XmlParser::get_attribute(e, b"cx") {
                            drawing.width = XmlParser::parse_emu(&cx);
                        }
                        if let Some(cy) = XmlParser::get_attribute(e, b"cy") {
                            drawing.height = XmlParser::parse_emu(&cy);
                        }
                    }
                    // Horizontal position
                    else if XmlParser::matches_element(name_ref, "positionH") {
                        in_position_h = true;
                        drawing.horizontal_anchor = XmlParser::get_attribute(e, b"relativeFrom")
                            .map(|s| parse_horizontal_anchor(&s));
                    }
                    // Vertical position
                    else if XmlParser::matches_element(name_ref, "positionV") {
                        in_position_v = true;
                        drawing.vertical_anchor = XmlParser::get_attribute(e, b"relativeFrom")
                            .map(|s| parse_vertical_anchor(&s));
                    }
                    // Wrap type
                    else if XmlParser::matches_element(name_ref, "wrapNone") {
                        drawing.wrap_type = Some(WrapType::InFront);
                    } else if XmlParser::matches_element(name_ref, "wrapSquare") {
                        drawing.wrap_type = Some(WrapType::Square);
                        drawing.wrap_text = XmlParser::get_attribute(e, b"wrapText");
                    } else if XmlParser::matches_element(name_ref, "wrapTight") {
                        drawing.wrap_type = Some(WrapType::Tight);
                    } else if XmlParser::matches_element(name_ref, "wrapThrough") {
                        drawing.wrap_type = Some(WrapType::Tight);
                    } else if XmlParser::matches_element(name_ref, "wrapTopAndBottom") {
                        drawing.wrap_type = Some(WrapType::Square);
                    }
                    // Text box content
                    else if XmlParser::matches_element(name_ref, "txbxContent") || XmlParser::matches_element(name_ref, "txbx") {
                        in_txbx = true;
                        drawing.drawing_type = DrawingType::TextBox;
                    }
                    // Shape properties
                    else if XmlParser::matches_element(name_ref, "spPr") {
                        in_sp_pr = true;
                    }
                    // Preset geometry
                    else if in_sp_pr && XmlParser::matches_element(name_ref, "prstGeom") {
                        if let Some(prst) = XmlParser::get_attribute(e, b"prst") {
                            drawing.shape_type = Some(parse_preset_shape(&prst));
                        }
                    }
                    // Custom geometry
                    else if in_sp_pr && XmlParser::matches_element(name_ref, "custGeom") {
                        drawing.drawing_type = DrawingType::CustomShape;
                    }
                    // Fill types
                    else if in_sp_pr && XmlParser::matches_element(name_ref, "solidFill") {
                        drawing.fill_type = Some(FillType::Solid);
                    } else if in_sp_pr && XmlParser::matches_element(name_ref, "gradFill") {
                        drawing.fill_type = Some(FillType::Gradient);
                    } else if in_sp_pr && XmlParser::matches_element(name_ref, "pattFill") {
                        drawing.fill_type = Some(FillType::Pattern);
                    } else if in_sp_pr && XmlParser::matches_element(name_ref, "noFill") {
                        drawing.fill_type = Some(FillType::None);
                    }
                    // Color
                    else if XmlParser::matches_element(name_ref, "srgbClr") {
                        if let Some(val) = XmlParser::get_attribute(e, b"val") {
                            drawing.fill_color = ShapeColor::from_hex(&val);
                        }
                    }
                    // Shape group
                    else if XmlParser::matches_element(name_ref, "wgp") {
                        drawing.drawing_type = DrawingType::Group;
                    }
                    // Connection shape
                    else if XmlParser::matches_element(name_ref, "cxnSp") {
                        drawing.drawing_type = DrawingType::Connector;
                    }
                    // Text
                    else if in_txbx && XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "extent") {
                        if let Some(cx) = XmlParser::get_attribute(e, b"cx") {
                            drawing.width = XmlParser::parse_emu(&cx);
                        }
                        if let Some(cy) = XmlParser::get_attribute(e, b"cy") {
                            drawing.height = XmlParser::parse_emu(&cy);
                        }
                    } else if XmlParser::matches_element(name_ref, "prstGeom") {
                        if let Some(prst) = XmlParser::get_attribute(e, b"prst") {
                            drawing.shape_type = Some(parse_preset_shape(&prst));
                        }
                    } else if XmlParser::matches_element(name_ref, "wrapNone") {
                        drawing.wrap_type = Some(WrapType::InFront);
                    } else if XmlParser::matches_element(name_ref, "solidFill") {
                        drawing.fill_type = Some(FillType::Solid);
                    } else if XmlParser::matches_element(name_ref, "noFill") {
                        drawing.fill_type = Some(FillType::None);
                    } else if XmlParser::matches_element(name_ref, "srgbClr") {
                        if let Some(val) = XmlParser::get_attribute(e, b"val") {
                            drawing.fill_color = ShapeColor::from_hex(&val);
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape()
                        .map_err(|e| DocxError::XmlParse(e.to_string()))?;

                    if in_position_h {
                        if let Some(emu) = XmlParser::parse_emu(&text) {
                            drawing.offset_x = Some(emu);
                        }
                    } else if in_position_v {
                        if let Some(emu) = XmlParser::parse_emu(&text) {
                            drawing.offset_y = Some(emu);
                        }
                    } else if in_text {
                        text_content.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "positionH") {
                        in_position_h = false;
                    } else if XmlParser::matches_element(name_ref, "positionV") {
                        in_position_v = false;
                    } else if XmlParser::matches_element(name_ref, "txbxContent") || XmlParser::matches_element(name_ref, "txbx") {
                        in_txbx = false;
                    } else if XmlParser::matches_element(name_ref, "spPr") {
                        in_sp_pr = false;
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        drawing.text_content = if text_content.is_empty() { None } else { Some(text_content) };
        self.drawings.push(drawing.clone());

        Ok(drawing)
    }

    /// Get all parsed drawings
    pub fn get_drawings(&self) -> &[ParsedDrawing] {
        &self.drawings
    }

    /// Clear parsed drawings
    pub fn clear(&mut self) {
        self.drawings.clear();
    }
}

// =============================================================================
// Drawing Writer
// =============================================================================

/// Writer for drawings in DOCX export
pub struct DrawingWriter;

impl DrawingWriter {
    /// Write a simple text box drawing
    pub fn write_simple_text_box(
        xml: &mut String,
        width: f32,
        height: f32,
        text: &str,
        is_inline: bool,
    ) {
        let width_emu = (width * 12700.0) as i64;
        let height_emu = (height * 12700.0) as i64;

        xml.push_str("<w:drawing>");

        if is_inline {
            xml.push_str(&format!(
                "<wp:inline xmlns:wp=\"{}\" xmlns:a=\"{}\">",
                namespaces::WP,
                namespaces::A
            ));
        } else {
            xml.push_str(&format!(
                "<wp:anchor xmlns:wp=\"{}\" xmlns:a=\"{}\" allowOverlap=\"1\" behindDoc=\"0\" distB=\"0\" distL=\"0\" distR=\"0\" distT=\"0\" layoutInCell=\"1\" locked=\"0\" relativeHeight=\"0\" simplePos=\"0\">",
                namespaces::WP,
                namespaces::A
            ));
            xml.push_str("<wp:simplePos x=\"0\" y=\"0\"/>");
            xml.push_str("<wp:positionH relativeFrom=\"column\"><wp:posOffset>0</wp:posOffset></wp:positionH>");
            xml.push_str("<wp:positionV relativeFrom=\"paragraph\"><wp:posOffset>0</wp:posOffset></wp:positionV>");
        }

        xml.push_str(&format!(
            "<wp:extent cx=\"{}\" cy=\"{}\"/>",
            width_emu, height_emu
        ));
        xml.push_str("<wp:effectExtent b=\"0\" l=\"0\" r=\"0\" t=\"0\"/>");

        if !is_inline {
            xml.push_str("<wp:wrapSquare wrapText=\"bothSides\"/>");
        }

        xml.push_str("<wp:docPr id=\"1\" name=\"Text Box\"/>");
        xml.push_str("<a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">");
        xml.push_str("<a:graphicData uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">");
        xml.push_str("<wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">");
        xml.push_str("<wps:cNvSpPr txBox=\"1\"/>");
        xml.push_str("<wps:spPr>");
        xml.push_str(&format!(
            "<a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>",
            width_emu, height_emu
        ));
        xml.push_str("<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>");
        xml.push_str("<a:solidFill><a:srgbClr val=\"FFFFFF\"/></a:solidFill>");
        xml.push_str("<a:ln><a:solidFill><a:srgbClr val=\"000000\"/></a:solidFill></a:ln>");
        xml.push_str("</wps:spPr>");
        xml.push_str("<wps:txbx><w:txbxContent xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">");
        xml.push_str("<w:p><w:r><w:t>");
        xml.push_str(&escape_xml(text));
        xml.push_str("</w:t></w:r></w:p>");
        xml.push_str("</w:txbxContent></wps:txbx>");
        xml.push_str("<wps:bodyPr anchor=\"t\" lIns=\"91440\" tIns=\"45720\" rIns=\"91440\" bIns=\"45720\"/>");
        xml.push_str("</wps:wsp>");
        xml.push_str("</a:graphicData>");
        xml.push_str("</a:graphic>");

        if is_inline {
            xml.push_str("</wp:inline>");
        } else {
            xml.push_str("</wp:anchor>");
        }

        xml.push_str("</w:drawing>");
    }
}

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed drawing from DOCX
#[derive(Debug, Clone, Default)]
pub struct ParsedDrawing {
    /// Drawing type
    pub drawing_type: DrawingType,
    /// Whether this is an inline drawing
    pub is_inline: bool,
    /// Width in points
    pub width: Option<f32>,
    /// Height in points
    pub height: Option<f32>,
    /// Horizontal position offset in points
    pub offset_x: Option<f32>,
    /// Vertical position offset in points
    pub offset_y: Option<f32>,
    /// Horizontal anchor
    pub horizontal_anchor: Option<HorizontalAnchor>,
    /// Vertical anchor
    pub vertical_anchor: Option<VerticalAnchor>,
    /// Wrap type
    pub wrap_type: Option<WrapType>,
    /// Wrap text mode
    pub wrap_text: Option<String>,
    /// Allow overlap
    pub allow_overlap: bool,
    /// Behind document
    pub behind_doc: bool,
    /// Layout in cell
    pub layout_in_cell: bool,
    /// Relative height (z-order)
    pub relative_height: u32,
    /// Shape type (for shapes)
    pub shape_type: Option<ShapeType>,
    /// Fill type
    pub fill_type: Option<FillType>,
    /// Fill color
    pub fill_color: Option<ShapeColor>,
    /// Text content (for text boxes)
    pub text_content: Option<String>,
    /// Group members (for groups)
    pub group_members: Vec<ParsedDrawing>,
    /// Connector start (for connectors)
    pub connector_start: Option<ConnectorEnd>,
    /// Connector end (for connectors)
    pub connector_end: Option<ConnectorEnd>,
}

/// Drawing type
#[derive(Debug, Clone, Default)]
pub enum DrawingType {
    #[default]
    Picture,
    TextBox,
    Shape,
    CustomShape,
    Group,
    Connector,
    Chart,
    Canvas,
}

/// Fill type
#[derive(Debug, Clone)]
pub enum FillType {
    None,
    Solid,
    Gradient,
    Pattern,
    Picture,
}

/// Connector end point
#[derive(Debug, Clone)]
pub struct ConnectorEnd {
    /// ID of the shape this connects to
    pub shape_id: Option<String>,
    /// Connection site index
    pub connection_site: Option<u32>,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse horizontal anchor from string
fn parse_horizontal_anchor(s: &str) -> HorizontalAnchor {
    match s {
        "character" => HorizontalAnchor::Character,
        "column" => HorizontalAnchor::Column,
        "margin" => HorizontalAnchor::Margin,
        "page" => HorizontalAnchor::Page,
        _ => HorizontalAnchor::Column,
    }
}

/// Parse vertical anchor from string
fn parse_vertical_anchor(s: &str) -> VerticalAnchor {
    match s {
        "line" => VerticalAnchor::Line,
        "margin" => VerticalAnchor::Margin,
        "page" => VerticalAnchor::Page,
        "paragraph" => VerticalAnchor::Paragraph,
        _ => VerticalAnchor::Paragraph,
    }
}

/// Parse preset shape type from string
fn parse_preset_shape(s: &str) -> ShapeType {
    match s {
        "rect" => ShapeType::Rectangle,
        "roundRect" => ShapeType::RoundedRectangle,
        "ellipse" => ShapeType::Oval,
        "triangle" => ShapeType::Triangle,
        "diamond" => ShapeType::Diamond,
        "pentagon" => ShapeType::Pentagon,
        "hexagon" => ShapeType::Hexagon,
        "octagon" => ShapeType::Octagon,
        "star4" => ShapeType::Star4,
        "star5" => ShapeType::Star5,
        "star6" => ShapeType::Star6,
        "star8" => ShapeType::Star8,
        "star10" => ShapeType::Star10,
        "star12" => ShapeType::Star12,
        "line" => ShapeType::Line,
        "rightArrow" => ShapeType::RightArrowBlock,
        "leftArrow" => ShapeType::LeftArrowBlock,
        "upArrow" => ShapeType::UpArrowBlock,
        "downArrow" => ShapeType::DownArrowBlock,
        "flowChartProcess" => ShapeType::FlowchartProcess,
        "flowChartDecision" => ShapeType::FlowchartDecision,
        "flowChartTerminator" => ShapeType::FlowchartTerminator,
        "flowChartDocument" => ShapeType::FlowchartDocument,
        "callout1" | "callout2" | "callout3" => ShapeType::RectangularCallout,
        "cloudCallout" => ShapeType::CloudCallout,
        "heart" => ShapeType::Heart,
        "lightningBolt" => ShapeType::LightningBolt,
        "sun" => ShapeType::Sun,
        "moon" => ShapeType::Moon,
        "cloud" => ShapeType::Cloud,
        other => ShapeType::Custom(other.to_string()),
    }
}

/// Escape XML text content
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawing_parser_new() {
        let parser = DrawingParser::new();
        assert!(parser.drawings.is_empty());
    }

    #[test]
    fn test_parse_horizontal_anchor() {
        assert!(matches!(parse_horizontal_anchor("page"), HorizontalAnchor::Page));
        assert!(matches!(parse_horizontal_anchor("column"), HorizontalAnchor::Column));
    }

    #[test]
    fn test_parse_vertical_anchor() {
        assert!(matches!(parse_vertical_anchor("page"), VerticalAnchor::Page));
        assert!(matches!(parse_vertical_anchor("paragraph"), VerticalAnchor::Paragraph));
    }

    #[test]
    fn test_parse_preset_shape() {
        assert!(matches!(parse_preset_shape("rect"), ShapeType::Rectangle));
        assert!(matches!(parse_preset_shape("ellipse"), ShapeType::Oval));
        assert!(matches!(parse_preset_shape("star5"), ShapeType::Star5));
    }
}
