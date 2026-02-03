//! Convert layout tree to render model

use crate::{
    Color, DashStyleRender, GlyphRun, HyperlinkRenderInfo, HyperlinkType, ImageRenderInfo,
    LineNumberRenderInfo, PageRender, Rect, RenderItem, RenderModel, Result, ShapeFillRender,
    ShapeRenderInfo, ShapeRenderType, ShapeStrokeRender, ShadowRender, TextBoxBorderEdgeRender,
    TextBoxBorderRender, TextBoxFillRender, TextBoxRenderInfo,
};
use doc_model::{BorderLineStyle, DashStyle, DocumentTree, FillStyle, HyperlinkTarget, ShapeFill, ShapeType, TextBox};
use layout_engine::{InlineType, LayoutTree};

/// Configuration for render conversion
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Background color for pages
    pub page_background: Color,
    /// Default text color
    pub text_color: Color,
    /// Default font family
    pub font_family: String,
    /// Default font size
    pub font_size: f64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            page_background: Color::WHITE,
            text_color: Color::BLACK,
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
        }
    }
}

/// Converts layout tree to render model
pub struct RenderConverter {
    config: RenderConfig,
}

impl RenderConverter {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Convert a layout tree to a render model
    pub fn convert(&self, layout: &LayoutTree, tree: &DocumentTree) -> Result<RenderModel> {
        let mut model = RenderModel::new();

        for page in &layout.pages {
            let mut page_render = PageRender {
                page_index: page.index as u32,
                width: page.bounds.width as f64,
                height: page.bounds.height as f64,
                items: Vec::new(),
            };

            // Add page background
            page_render.items.push(RenderItem::Rectangle {
                bounds: Rect::from(page.bounds),
                fill: Some(self.config.page_background),
                stroke: Some(Color::rgb(200, 200, 200)),
                stroke_width: 1.0,
            });

            // Render each area
            for area in &page.areas {
                for column in &area.columns {
                    for block in &column.blocks {
                        // Render each line in the block
                        for line in &block.lines {
                            // Render each inline in the line
                            for inline in &line.inlines {
                                match inline.inline_type {
                                    InlineType::Text => {
                                        // Get the text from the run
                                        if let Some(run) = tree.get_run(inline.node_id) {
                                            let text = if inline.end_offset <= run.text.len() {
                                                &run.text[inline.start_offset..inline.end_offset]
                                            } else {
                                                &run.text
                                            };

                                            if !text.is_empty() {
                                                let baseline_y = page.content_area.y + line.bounds.y + line.baseline;

                                                // Check if this run is inside a hyperlink
                                                let hyperlink_info = self.get_hyperlink_info(tree, inline.node_id);
                                                let (text_color, is_underline) = if hyperlink_info.is_some() {
                                                    // Hyperlink styling: blue and underlined
                                                    (Color::rgb(0, 0, 255), true)
                                                } else {
                                                    // Use run style color or default
                                                    let color = run.style.color.as_ref()
                                                        .and_then(|c| parse_color(c))
                                                        .unwrap_or(self.config.text_color);
                                                    (color, run.style.underline.unwrap_or(false))
                                                };

                                                page_render.items.push(RenderItem::GlyphRun(GlyphRun {
                                                    text: text.to_string(),
                                                    font_family: run.style.font_family
                                                        .as_ref()
                                                        .unwrap_or(&self.config.font_family)
                                                        .clone(),
                                                    font_size: run.style.font_size
                                                        .map(|s| s as f64)
                                                        .unwrap_or(self.config.font_size),
                                                    bold: run.style.bold.unwrap_or(false),
                                                    italic: run.style.italic.unwrap_or(false),
                                                    underline: is_underline,
                                                    color: text_color,
                                                    x: (page.content_area.x + inline.bounds.x) as f64,
                                                    y: baseline_y as f64,
                                                    hyperlink: hyperlink_info,
                                                }));
                                            }
                                        }
                                    }
                                    InlineType::Image => {
                                        // Render inline image
                                        if let Some(image) = tree.get_image(inline.node_id) {
                                            let image_x = page.content_area.x + inline.bounds.x;
                                            let image_y = page.content_area.y + line.bounds.y + inline.bounds.y;

                                            page_render.items.push(RenderItem::Image(ImageRenderInfo {
                                                node_id: inline.node_id.to_string(),
                                                resource_id: image.resource_id.to_string(),
                                                bounds: Rect::new(
                                                    image_x as f64,
                                                    image_y as f64,
                                                    inline.bounds.width as f64,
                                                    inline.bounds.height as f64,
                                                ),
                                                rotation: image.properties.rotation as f64,
                                                alt_text: image.alt_text.clone(),
                                                title: image.title.clone(),
                                                selected: false,
                                            }));
                                        }
                                    }
                                    InlineType::ListMarker => {
                                        // Render list marker (bullet or number)
                                        if let Some(marker) = &inline.list_marker {
                                            let baseline_y = page.content_area.y + line.bounds.y + line.baseline;

                                            page_render.items.push(RenderItem::GlyphRun(GlyphRun {
                                                text: marker.text.clone(),
                                                font_family: marker.font.clone().unwrap_or_else(|| {
                                                    if marker.is_bullet {
                                                        "Symbol".to_string()
                                                    } else {
                                                        self.config.font_family.clone()
                                                    }
                                                }),
                                                font_size: self.config.font_size,
                                                bold: false,
                                                italic: false,
                                                underline: false,
                                                color: self.config.text_color,
                                                x: (page.content_area.x + inline.bounds.x) as f64,
                                                y: baseline_y as f64,
                                                hyperlink: None,
                                            }));
                                        }
                                    }
                                    InlineType::Shape => {
                                        // Render inline shape
                                        if let Some(shape) = tree.get_shape(inline.node_id) {
                                            let shape_x = page.content_area.x + inline.bounds.x;
                                            let shape_y = page.content_area.y + line.bounds.y + inline.bounds.y;

                                            page_render.items.push(RenderItem::Shape(self.convert_shape_to_render_info(
                                                inline.node_id,
                                                shape,
                                                shape_x as f64,
                                                shape_y as f64,
                                                inline.bounds.width as f64,
                                                inline.bounds.height as f64,
                                                false,
                                            )));
                                        }
                                    }
                                    InlineType::TextBox => {
                                        // Render inline text box
                                        if let Some(textbox) = tree.get_textbox(inline.node_id) {
                                            let tb_x = page.content_area.x + inline.bounds.x;
                                            let tb_y = page.content_area.y + line.bounds.y + inline.bounds.y;

                                            page_render.items.push(RenderItem::TextBox(self.convert_textbox_to_render_info(
                                                inline.node_id,
                                                textbox,
                                                tb_x as f64,
                                                tb_y as f64,
                                                inline.bounds.width as f64,
                                                inline.bounds.height as f64,
                                                false,
                                            )));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Render floating images for this page
            for floating_img in layout.floating_images_on_page(page.index) {
                if let Some(image) = tree.get_image(floating_img.node_id) {
                    page_render.items.push(RenderItem::Image(ImageRenderInfo {
                        node_id: floating_img.node_id.to_string(),
                        resource_id: image.resource_id.to_string(),
                        bounds: Rect::from(floating_img.bounds),
                        rotation: image.properties.rotation as f64,
                        alt_text: image.alt_text.clone(),
                        title: image.title.clone(),
                        selected: false,
                    }));
                }
            }

            // Render floating shapes for this page
            for floating_shape in layout.floating_shapes_on_page(page.index) {
                if let Some(shape) = tree.get_shape(floating_shape.node_id) {
                    page_render.items.push(RenderItem::Shape(self.convert_shape_to_render_info(
                        floating_shape.node_id,
                        shape,
                        floating_shape.bounds.x as f64,
                        floating_shape.bounds.y as f64,
                        floating_shape.bounds.width as f64,
                        floating_shape.bounds.height as f64,
                        false,
                    )));
                }
            }

            // Render line numbers for this page
            for line_num in layout.line_numbers_on_page(page.index) {
                page_render.items.push(RenderItem::LineNumber(LineNumberRenderInfo::new(
                    line_num.number,
                    line_num.x as f64,
                    line_num.y as f64,
                    line_num.font_size as f64,
                )));
            }

            model.add_page(page_render);
        }

        Ok(model)
    }
}

impl RenderConverter {
    /// Get hyperlink render info if the run is inside a hyperlink
    fn get_hyperlink_info(&self, tree: &DocumentTree, run_id: doc_model::NodeId) -> Option<HyperlinkRenderInfo> {
        // Find if this run's parent is a hyperlink
        if let Some(hyperlink_id) = tree.find_hyperlink_for_run(run_id) {
            if let Some(hyperlink) = tree.get_hyperlink(hyperlink_id) {
                let (target, link_type) = match &hyperlink.target {
                    HyperlinkTarget::External(url) => (url.clone(), HyperlinkType::External),
                    HyperlinkTarget::Internal(bookmark) => (format!("#{}", bookmark), HyperlinkType::Internal),
                    HyperlinkTarget::Email { address, subject } => {
                        let mut url = format!("mailto:{}", address);
                        if let Some(subj) = subject {
                            url.push_str("?subject=");
                            url.push_str(subj);
                        }
                        (url, HyperlinkType::Email)
                    }
                };

                return Some(HyperlinkRenderInfo {
                    node_id: hyperlink_id.to_string(),
                    target,
                    tooltip: hyperlink.tooltip.clone(),
                    link_type,
                });
            }
        }
        None
    }

    /// Convert a shape to render info
    fn convert_shape_to_render_info(
        &self,
        node_id: doc_model::NodeId,
        shape: &doc_model::ShapeNode,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        selected: bool,
    ) -> ShapeRenderInfo {
        // Convert shape type - map all shape types to render types
        let shape_type = match &shape.shape_type {
            ShapeType::Rectangle => ShapeRenderType::Rectangle,
            ShapeType::RoundedRectangle { corner_radius } => {
                ShapeRenderType::RoundedRectangle { corner_radius: *corner_radius as f64 }
            }
            ShapeType::Oval => ShapeRenderType::Oval,
            ShapeType::Line => ShapeRenderType::Line,
            ShapeType::Arrow => ShapeRenderType::Arrow,
            ShapeType::DoubleArrow => ShapeRenderType::DoubleArrow,
            ShapeType::Triangle => ShapeRenderType::Triangle,
            ShapeType::Diamond => ShapeRenderType::Diamond,
            ShapeType::Pentagon => ShapeRenderType::Pentagon,
            ShapeType::Hexagon => ShapeRenderType::Hexagon,
            ShapeType::Star { points, inner_radius_ratio } => {
                ShapeRenderType::Star { points: *points, inner_radius_ratio: *inner_radius_ratio as f64 }
            }
            ShapeType::Callout { tail_position, tail_width } => {
                ShapeRenderType::Callout {
                    tail_position: (tail_position.0 as f64, tail_position.1 as f64),
                    tail_width: *tail_width as f64,
                }
            }
            ShapeType::TextBox => ShapeRenderType::TextBox,
            ShapeType::RightArrowBlock => ShapeRenderType::RightArrowBlock,
            ShapeType::LeftArrowBlock => ShapeRenderType::LeftArrowBlock,
            ShapeType::UpArrowBlock => ShapeRenderType::UpArrowBlock,
            ShapeType::DownArrowBlock => ShapeRenderType::DownArrowBlock,
            // Block arrows - map to generic shapes for rendering
            ShapeType::RightArrow { .. } => ShapeRenderType::RightArrowBlock,
            ShapeType::LeftArrow { .. } => ShapeRenderType::LeftArrowBlock,
            ShapeType::UpArrow { .. } => ShapeRenderType::UpArrowBlock,
            ShapeType::DownArrow { .. } => ShapeRenderType::DownArrowBlock,
            ShapeType::LeftRightArrow { .. } => ShapeRenderType::Rectangle, // TODO: dedicated render type
            ShapeType::UpDownArrow { .. } => ShapeRenderType::Rectangle,
            ShapeType::QuadArrow { .. } => ShapeRenderType::Rectangle,
            ShapeType::BentArrow { .. } => ShapeRenderType::Rectangle,
            ShapeType::UTurnArrow { .. } => ShapeRenderType::Rectangle,
            ShapeType::ChevronArrow { .. } => ShapeRenderType::Triangle,
            ShapeType::StripedRightArrow { .. } => ShapeRenderType::RightArrowBlock,
            ShapeType::NotchedRightArrow { .. } => ShapeRenderType::RightArrowBlock,
            ShapeType::CurvedRightArrow { .. } => ShapeRenderType::RightArrowBlock,
            ShapeType::CurvedLeftArrow { .. } => ShapeRenderType::LeftArrowBlock,
            // Flowchart shapes - map to closest basic shape
            ShapeType::FlowchartProcess => ShapeRenderType::Rectangle,
            ShapeType::FlowchartDecision => ShapeRenderType::Diamond,
            ShapeType::FlowchartData => ShapeRenderType::Rectangle, // parallelogram
            ShapeType::FlowchartTerminator => ShapeRenderType::RoundedRectangle { corner_radius: 20.0 },
            ShapeType::FlowchartDocument => ShapeRenderType::Rectangle,
            ShapeType::FlowchartMultiDocument => ShapeRenderType::Rectangle,
            ShapeType::FlowchartPredefined => ShapeRenderType::Rectangle,
            ShapeType::FlowchartManualInput => ShapeRenderType::Rectangle,
            ShapeType::FlowchartPreparation => ShapeRenderType::Hexagon,
            ShapeType::FlowchartInternalStorage => ShapeRenderType::Rectangle,
            ShapeType::FlowchartManualOperation => ShapeRenderType::Rectangle,
            ShapeType::FlowchartConnector => ShapeRenderType::Oval,
            ShapeType::FlowchartOffPageConnector => ShapeRenderType::Pentagon,
            ShapeType::FlowchartDelay => ShapeRenderType::RoundedRectangle { corner_radius: 30.0 },
            ShapeType::FlowchartAlternateProcess => ShapeRenderType::RoundedRectangle { corner_radius: 10.0 },
            ShapeType::FlowchartOr => ShapeRenderType::Oval,
            ShapeType::FlowchartSummingJunction => ShapeRenderType::Oval,
            ShapeType::FlowchartSort => ShapeRenderType::Diamond,
            ShapeType::FlowchartCollate => ShapeRenderType::Rectangle,
            ShapeType::FlowchartExtract => ShapeRenderType::Triangle,
            ShapeType::FlowchartMerge => ShapeRenderType::Triangle,
            ShapeType::FlowchartStoredData => ShapeRenderType::Rectangle,
            ShapeType::FlowchartMagneticDisk => ShapeRenderType::Oval,
            ShapeType::FlowchartDirectAccessStorage => ShapeRenderType::Oval,
            ShapeType::FlowchartSequentialAccess => ShapeRenderType::Oval,
            ShapeType::FlowchartDisplay => ShapeRenderType::RoundedRectangle { corner_radius: 15.0 },
            ShapeType::FlowchartCard => ShapeRenderType::Rectangle,
            ShapeType::FlowchartPaperTape => ShapeRenderType::Rectangle,
            // Callouts - map to basic callout
            ShapeType::RectangularCallout { tail_anchor, tail_tip, tail_width } => {
                ShapeRenderType::Callout {
                    tail_position: (tail_anchor.0 as f64, tail_anchor.1 as f64),
                    tail_width: *tail_width as f64,
                }
            }
            ShapeType::RoundedCallout { tail_anchor, tail_width, .. } => {
                ShapeRenderType::Callout {
                    tail_position: (tail_anchor.0 as f64, tail_anchor.1 as f64),
                    tail_width: *tail_width as f64,
                }
            }
            ShapeType::OvalCallout { tail_anchor, tail_width, .. } => {
                ShapeRenderType::Callout {
                    tail_position: (tail_anchor.0 as f64, tail_anchor.1 as f64),
                    tail_width: *tail_width as f64,
                }
            }
            ShapeType::CloudCallout { .. } => ShapeRenderType::Oval,
            ShapeType::LineCallout { .. } => ShapeRenderType::Rectangle,
            ShapeType::ThoughtBubbleCallout { .. } => ShapeRenderType::Oval,
            // Stars and banners
            ShapeType::Star4 => ShapeRenderType::Star { points: 4, inner_radius_ratio: 0.4 },
            ShapeType::Star5 => ShapeRenderType::Star { points: 5, inner_radius_ratio: 0.4 },
            ShapeType::Star6 => ShapeRenderType::Star { points: 6, inner_radius_ratio: 0.4 },
            ShapeType::Star8 => ShapeRenderType::Star { points: 8, inner_radius_ratio: 0.4 },
            ShapeType::Star10 => ShapeRenderType::Star { points: 10, inner_radius_ratio: 0.4 },
            ShapeType::Star12 => ShapeRenderType::Star { points: 12, inner_radius_ratio: 0.4 },
            ShapeType::Star16 => ShapeRenderType::Star { points: 16, inner_radius_ratio: 0.4 },
            ShapeType::Star24 => ShapeRenderType::Star { points: 24, inner_radius_ratio: 0.4 },
            ShapeType::Star32 => ShapeRenderType::Star { points: 32, inner_radius_ratio: 0.4 },
            ShapeType::Explosion1 | ShapeType::Explosion2 => {
                ShapeRenderType::Star { points: 12, inner_radius_ratio: 0.5 }
            }
            ShapeType::Ribbon { .. } | ShapeType::CurvedRibbon { .. } => ShapeRenderType::Rectangle,
            ShapeType::Wave { .. } | ShapeType::DoubleWave { .. } => ShapeRenderType::Rectangle,
            ShapeType::HorizontalScroll { .. } | ShapeType::VerticalScroll { .. } => ShapeRenderType::Rectangle,
            // Additional shapes
            ShapeType::Parallelogram { .. } => ShapeRenderType::Rectangle,
            ShapeType::Trapezoid { .. } => ShapeRenderType::Rectangle,
            ShapeType::Octagon => ShapeRenderType::Hexagon, // approximate
            ShapeType::Decagon | ShapeType::Dodecagon => ShapeRenderType::Oval,
            ShapeType::RegularPolygon { sides } => {
                if *sides <= 6 { ShapeRenderType::Hexagon } else { ShapeRenderType::Oval }
            }
            ShapeType::Cross { .. } => ShapeRenderType::Rectangle,
            ShapeType::Frame { .. } => ShapeRenderType::Rectangle,
            ShapeType::LShape { .. } => ShapeRenderType::Rectangle,
            ShapeType::Donut { inner_radius } => ShapeRenderType::Oval,
            ShapeType::Arc { .. } | ShapeType::BlockArc { .. } => ShapeRenderType::Oval,
            ShapeType::Pie { .. } | ShapeType::Chord { .. } => ShapeRenderType::Oval,
            ShapeType::Heart => ShapeRenderType::Oval,
            ShapeType::LightningBolt => ShapeRenderType::Triangle,
            ShapeType::Sun { .. } => ShapeRenderType::Star { points: 12, inner_radius_ratio: 0.6 },
            ShapeType::Moon { .. } => ShapeRenderType::Oval,
            ShapeType::Cloud => ShapeRenderType::Oval,
            ShapeType::SmileyFace => ShapeRenderType::Oval,
            ShapeType::NoSymbol => ShapeRenderType::Oval,
            ShapeType::FoldedCorner { .. } => ShapeRenderType::Rectangle,
            ShapeType::Bevel { .. } => ShapeRenderType::Rectangle,
            ShapeType::Cube { .. } => ShapeRenderType::Rectangle,
            // Equation shapes
            ShapeType::MathPlus | ShapeType::MathMinus => ShapeRenderType::Rectangle,
            ShapeType::MathMultiply => ShapeRenderType::Rectangle,
            ShapeType::MathDivide => ShapeRenderType::Rectangle,
            ShapeType::MathEqual | ShapeType::MathNotEqual => ShapeRenderType::Rectangle,
            // Action buttons
            ShapeType::ActionButtonBlank | ShapeType::ActionButtonHome |
            ShapeType::ActionButtonHelp | ShapeType::ActionButtonInformation |
            ShapeType::ActionButtonBack | ShapeType::ActionButtonForward |
            ShapeType::ActionButtonBeginning | ShapeType::ActionButtonEnd |
            ShapeType::ActionButtonReturn | ShapeType::ActionButtonDocument |
            ShapeType::ActionButtonSound | ShapeType::ActionButtonMovie => {
                ShapeRenderType::RoundedRectangle { corner_radius: 5.0 }
            }
            // Custom shapes
            ShapeType::CustomPath { .. } => ShapeRenderType::Rectangle,
            ShapeType::Freeform { .. } => ShapeRenderType::Rectangle,
        };

        // Convert fill - support all fill types
        let fill = shape.properties.fill.as_ref().map(|f| match f {
            ShapeFill::Solid(color) => ShapeFillRender::Solid {
                color: Color::rgba(color.r, color.g, color.b, color.a),
            },
            ShapeFill::Gradient { colors, angle } => ShapeFillRender::Gradient {
                colors: colors
                    .iter()
                    .map(|(c, pos)| (Color::rgba(c.r, c.g, c.b, c.a), *pos as f64))
                    .collect(),
                angle: *angle as f64,
            },
            ShapeFill::LinearGradient { angle, stops, .. } => ShapeFillRender::Gradient {
                colors: stops
                    .iter()
                    .map(|s| (Color::rgba(s.color.r, s.color.g, s.color.b, s.color.a), s.position as f64))
                    .collect(),
                angle: *angle as f64,
            },
            ShapeFill::RadialGradient { stops, .. } => ShapeFillRender::Gradient {
                colors: stops
                    .iter()
                    .map(|s| (Color::rgba(s.color.r, s.color.g, s.color.b, s.color.a), s.position as f64))
                    .collect(),
                angle: 0.0, // Radial gradients don't have angle
            },
            ShapeFill::RectangularGradient { stops, .. } => ShapeFillRender::Gradient {
                colors: stops
                    .iter()
                    .map(|s| (Color::rgba(s.color.r, s.color.g, s.color.b, s.color.a), s.position as f64))
                    .collect(),
                angle: 0.0,
            },
            ShapeFill::PathGradient { stops } => ShapeFillRender::Gradient {
                colors: stops
                    .iter()
                    .map(|s| (Color::rgba(s.color.r, s.color.g, s.color.b, s.color.a), s.position as f64))
                    .collect(),
                angle: 0.0,
            },
            ShapeFill::Pattern { foreground, background, .. } => {
                // For patterns, render as solid with foreground color for now
                ShapeFillRender::Solid {
                    color: Color::rgba(foreground.r, foreground.g, foreground.b, foreground.a),
                }
            },
            ShapeFill::Picture { .. } => {
                // For picture fills, render as transparent for now
                ShapeFillRender::None
            },
            ShapeFill::None => ShapeFillRender::None,
        });

        // Convert stroke
        let stroke = shape.properties.stroke.as_ref().map(|s| {
            ShapeStrokeRender {
                color: Color::rgba(s.color.r, s.color.g, s.color.b, s.color.a),
                width: s.width as f64,
                dash_style: match s.dash_style {
                    DashStyle::Solid => DashStyleRender::Solid,
                    DashStyle::Dash => DashStyleRender::Dash,
                    DashStyle::Dot => DashStyleRender::Dot,
                    DashStyle::DashDot => DashStyleRender::DashDot,
                    DashStyle::DashDotDot => DashStyleRender::DashDotDot,
                },
            }
        });

        // Convert shadow
        let shadow = shape.properties.effects.shadow.as_ref().map(|s| {
            ShadowRender {
                color: Color::rgba(s.color.r, s.color.g, s.color.b, s.color.a),
                offset_x: s.offset_x as f64,
                offset_y: s.offset_y as f64,
                blur_radius: s.blur as f64,
            }
        });

        ShapeRenderInfo {
            node_id: node_id.to_string(),
            shape_type,
            bounds: Rect::new(x, y, width, height),
            rotation: shape.properties.rotation as f64,
            fill,
            stroke,
            shadow,
            opacity: shape.properties.effects.opacity as f64,
            selected,
            flip_horizontal: shape.properties.flip_horizontal,
            flip_vertical: shape.properties.flip_vertical,
        }
    }

    /// Convert a TextBox to TextBoxRenderInfo
    fn convert_textbox_to_render_info(
        &self,
        node_id: doc_model::NodeId,
        textbox: &TextBox,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        selected: bool,
    ) -> TextBoxRenderInfo {
        // Calculate content bounds accounting for margins and borders
        let margins = &textbox.style.internal_margins;
        let border_width = textbox.style.border.as_ref()
            .map(|b| (b.left.width + b.right.width, b.top.width + b.bottom.width))
            .unwrap_or((0.0, 0.0));

        let content_bounds = Rect::new(
            x + margins.left as f64 + border_width.0 as f64 / 2.0,
            y + margins.top as f64 + border_width.1 as f64 / 2.0,
            (width - margins.left as f64 - margins.right as f64 - border_width.0 as f64).max(0.0),
            (height - margins.top as f64 - margins.bottom as f64 - border_width.1 as f64).max(0.0),
        );

        // Convert fill
        let fill = textbox.style.fill.as_ref().map(|f| match f {
            FillStyle::None => TextBoxFillRender::None,
            FillStyle::Solid(color) => TextBoxFillRender::Solid {
                color: parse_color(color).unwrap_or(Color::WHITE),
            },
            FillStyle::Gradient { colors, angle } => TextBoxFillRender::Gradient {
                colors: colors
                    .iter()
                    .map(|(c, pos)| (parse_color(c).unwrap_or(Color::WHITE), *pos as f64))
                    .collect(),
                angle: *angle as f64,
            },
        });

        // Convert border
        let border = textbox.style.border.as_ref().map(|b| {
            TextBoxBorderRender {
                top: convert_border_edge(&b.top),
                right: convert_border_edge(&b.right),
                bottom: convert_border_edge(&b.bottom),
                left: convert_border_edge(&b.left),
            }
        });

        TextBoxRenderInfo {
            node_id: node_id.to_string(),
            bounds: Rect::new(x, y, width, height),
            content_bounds,
            rotation: textbox.style.rotation as f64,
            fill,
            border,
            opacity: textbox.style.opacity as f64,
            alt_text: textbox.alt_text.clone(),
            name: textbox.name.clone(),
            selected,
            is_editing: false,
            content_items: Vec::new(), // Content items would be filled during layout
        }
    }
}

/// Convert a border edge to render format
fn convert_border_edge(edge: &doc_model::BorderEdge) -> TextBoxBorderEdgeRender {
    TextBoxBorderEdgeRender {
        width: edge.width as f64,
        color: parse_color(&edge.color).unwrap_or(Color::BLACK),
        style: match edge.style {
            BorderLineStyle::None => "none".to_string(),
            BorderLineStyle::Solid => "solid".to_string(),
            BorderLineStyle::Dashed => "dashed".to_string(),
            BorderLineStyle::Dotted => "dotted".to_string(),
            BorderLineStyle::Double => "double".to_string(),
        },
    }
}

/// Parse a CSS color string to a Color
fn parse_color(color_str: &str) -> Option<Color> {
    if color_str.starts_with('#') {
        let hex = &color_str[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::rgb(r, g, b));
        }
    }
    None
}

impl Default for RenderConverter {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}
