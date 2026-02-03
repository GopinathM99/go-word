//! Chart rendering
//!
//! This module renders charts to SVG or render primitives
//! that can be displayed by the frontend.

use crate::layout::*;
use crate::model::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// A render primitive for chart elements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChartRenderPrimitive {
    /// A filled rectangle
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    /// A line
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: String,
        stroke_width: f64,
    },
    /// A polyline (multiple connected line segments)
    Polyline {
        points: Vec<(f64, f64)>,
        stroke: String,
        stroke_width: f64,
        fill: Option<String>,
    },
    /// A polygon (closed shape)
    Polygon {
        points: Vec<(f64, f64)>,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    /// A circle
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    /// An arc/pie slice
    Arc {
        cx: f64,
        cy: f64,
        inner_radius: f64,
        outer_radius: f64,
        start_angle: f64,
        end_angle: f64,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    /// Text
    Text {
        x: f64,
        y: f64,
        text: String,
        font_size: f64,
        font_family: String,
        fill: String,
        anchor: TextAnchor,
        baseline: TextBaseline,
    },
    /// A path (SVG path data)
    Path {
        d: String,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
}

/// Text anchor position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

/// Text baseline position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextBaseline {
    Top,
    Middle,
    Bottom,
    Alphabetic,
}

/// Rendered chart output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedChart {
    /// Width of the chart
    pub width: f64,
    /// Height of the chart
    pub height: f64,
    /// Render primitives
    pub primitives: Vec<ChartRenderPrimitive>,
}

/// Chart renderer
pub struct ChartRenderer {
    /// Background color
    pub background_color: Option<Color>,
    /// Gridline color
    pub gridline_color: Color,
    /// Gridline width
    pub gridline_width: f64,
    /// Axis line color
    pub axis_color: Color,
    /// Axis line width
    pub axis_width: f64,
    /// Font family for text
    pub font_family: String,
    /// Title font size
    pub title_font_size: f64,
    /// Label font size
    pub label_font_size: f64,
    /// Legend font size
    pub legend_font_size: f64,
    /// Text color
    pub text_color: Color,
    /// Line width for chart lines
    pub line_width: f64,
    /// Bar stroke width
    pub bar_stroke_width: f64,
}

impl Default for ChartRenderer {
    fn default() -> Self {
        Self {
            background_color: Some(Color::WHITE),
            gridline_color: Color::rgb(230, 230, 230),
            gridline_width: 1.0,
            axis_color: Color::rgb(100, 100, 100),
            axis_width: 1.0,
            font_family: "sans-serif".to_string(),
            title_font_size: 18.0,
            label_font_size: 12.0,
            legend_font_size: 11.0,
            text_color: Color::rgb(50, 50, 50),
            line_width: 2.0,
            bar_stroke_width: 1.0,
        }
    }
}

impl ChartRenderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a chart to primitives
    pub fn render(&self, _chart: &Chart, layout: &ChartLayout) -> RenderedChart {
        let mut primitives = Vec::new();

        // Render background
        if let Some(bg) = self.background_color {
            primitives.push(ChartRenderPrimitive::Rect {
                x: 0.0,
                y: 0.0,
                width: layout.total_bounds.width,
                height: layout.total_bounds.height,
                fill: bg.to_css(),
                stroke: None,
                stroke_width: None,
            });
        }

        // Render title
        if let Some((bounds, ref text)) = layout.title {
            primitives.push(ChartRenderPrimitive::Text {
                x: bounds.center_x(),
                y: bounds.center_y(),
                text: text.clone(),
                font_size: self.title_font_size,
                font_family: self.font_family.clone(),
                fill: self.text_color.to_css(),
                anchor: TextAnchor::Middle,
                baseline: TextBaseline::Middle,
            });
        }

        // Render gridlines
        self.render_gridlines(&mut primitives, layout);

        // Render axes
        self.render_axes(&mut primitives, layout);

        // Render chart-specific elements
        self.render_bars(&mut primitives, layout);
        self.render_lines(&mut primitives, layout);
        self.render_markers(&mut primitives, layout);
        self.render_pie_slices(&mut primitives, layout);
        self.render_areas(&mut primitives, layout);

        // Render legend
        if let Some(ref legend) = layout.legend {
            self.render_legend(&mut primitives, legend);
        }

        // Render data labels
        self.render_data_labels(&mut primitives, layout);

        RenderedChart {
            width: layout.total_bounds.width,
            height: layout.total_bounds.height,
            primitives,
        }
    }

    /// Render chart to SVG string
    pub fn render_svg(&self, chart: &Chart, layout: &ChartLayout) -> String {
        let rendered = self.render(chart, layout);
        self.to_svg(&rendered)
    }

    /// Convert rendered chart to SVG string
    pub fn to_svg(&self, rendered: &RenderedChart) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            rendered.width, rendered.height, rendered.width, rendered.height
        );
        svg.push('\n');

        for primitive in &rendered.primitives {
            svg.push_str(&self.primitive_to_svg(primitive));
            svg.push('\n');
        }

        svg.push_str("</svg>");
        svg
    }

    fn primitive_to_svg(&self, primitive: &ChartRenderPrimitive) -> String {
        match primitive {
            ChartRenderPrimitive::Rect {
                x,
                y,
                width,
                height,
                fill,
                stroke,
                stroke_width,
            } => {
                let mut attrs = format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}""#,
                    x, y, width, height, fill
                );
                if let Some(s) = stroke {
                    attrs.push_str(&format!(r#" stroke="{}""#, s));
                }
                if let Some(sw) = stroke_width {
                    attrs.push_str(&format!(r#" stroke-width="{}""#, sw));
                }
                attrs.push_str("/>");
                attrs
            }
            ChartRenderPrimitive::Line {
                x1,
                y1,
                x2,
                y2,
                stroke,
                stroke_width,
            } => {
                format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
                    x1, y1, x2, y2, stroke, stroke_width
                )
            }
            ChartRenderPrimitive::Polyline {
                points,
                stroke,
                stroke_width,
                fill,
            } => {
                let points_str: String = points
                    .iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(" ");
                let fill_attr = fill.as_deref().unwrap_or("none");
                format!(
                    r#"<polyline points="{}" stroke="{}" stroke-width="{}" fill="{}"/>"#,
                    points_str, stroke, stroke_width, fill_attr
                )
            }
            ChartRenderPrimitive::Polygon {
                points,
                fill,
                stroke,
                stroke_width,
            } => {
                let points_str: String = points
                    .iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut attrs = format!(r#"<polygon points="{}" fill="{}""#, points_str, fill);
                if let Some(s) = stroke {
                    attrs.push_str(&format!(r#" stroke="{}""#, s));
                }
                if let Some(sw) = stroke_width {
                    attrs.push_str(&format!(r#" stroke-width="{}""#, sw));
                }
                attrs.push_str("/>");
                attrs
            }
            ChartRenderPrimitive::Circle {
                cx,
                cy,
                r,
                fill,
                stroke,
                stroke_width,
            } => {
                let mut attrs = format!(r#"<circle cx="{}" cy="{}" r="{}" fill="{}""#, cx, cy, r, fill);
                if let Some(s) = stroke {
                    attrs.push_str(&format!(r#" stroke="{}""#, s));
                }
                if let Some(sw) = stroke_width {
                    attrs.push_str(&format!(r#" stroke-width="{}""#, sw));
                }
                attrs.push_str("/>");
                attrs
            }
            ChartRenderPrimitive::Arc {
                cx,
                cy,
                inner_radius,
                outer_radius,
                start_angle,
                end_angle,
                fill,
                stroke,
                stroke_width,
            } => {
                let path = self.arc_to_path(*cx, *cy, *inner_radius, *outer_radius, *start_angle, *end_angle);
                let mut attrs = format!(r#"<path d="{}" fill="{}""#, path, fill);
                if let Some(s) = stroke {
                    attrs.push_str(&format!(r#" stroke="{}""#, s));
                }
                if let Some(sw) = stroke_width {
                    attrs.push_str(&format!(r#" stroke-width="{}""#, sw));
                }
                attrs.push_str("/>");
                attrs
            }
            ChartRenderPrimitive::Text {
                x,
                y,
                text,
                font_size,
                font_family,
                fill,
                anchor,
                baseline,
            } => {
                let anchor_str = match anchor {
                    TextAnchor::Start => "start",
                    TextAnchor::Middle => "middle",
                    TextAnchor::End => "end",
                };
                let baseline_str = match baseline {
                    TextBaseline::Top => "hanging",
                    TextBaseline::Middle => "middle",
                    TextBaseline::Bottom => "text-bottom",
                    TextBaseline::Alphabetic => "alphabetic",
                };
                format!(
                    r#"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}" text-anchor="{}" dominant-baseline="{}">{}</text>"#,
                    x, y, font_size, font_family, fill, anchor_str, baseline_str,
                    self.escape_xml(text)
                )
            }
            ChartRenderPrimitive::Path {
                d,
                fill,
                stroke,
                stroke_width,
            } => {
                let fill_attr = fill.as_deref().unwrap_or("none");
                let mut attrs = format!(r#"<path d="{}" fill="{}""#, d, fill_attr);
                if let Some(s) = stroke {
                    attrs.push_str(&format!(r#" stroke="{}""#, s));
                }
                if let Some(sw) = stroke_width {
                    attrs.push_str(&format!(r#" stroke-width="{}""#, sw));
                }
                attrs.push_str("/>");
                attrs
            }
        }
    }

    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn arc_to_path(
        &self,
        cx: f64,
        cy: f64,
        inner_radius: f64,
        outer_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> String {
        let large_arc = if (end_angle - start_angle).abs() > PI { 1 } else { 0 };

        let outer_start_x = cx + outer_radius * start_angle.cos();
        let outer_start_y = cy + outer_radius * start_angle.sin();
        let outer_end_x = cx + outer_radius * end_angle.cos();
        let outer_end_y = cy + outer_radius * end_angle.sin();

        if inner_radius > 0.0 {
            // Doughnut slice
            let inner_start_x = cx + inner_radius * start_angle.cos();
            let inner_start_y = cy + inner_radius * start_angle.sin();
            let inner_end_x = cx + inner_radius * end_angle.cos();
            let inner_end_y = cy + inner_radius * end_angle.sin();

            format!(
                "M {} {} A {} {} 0 {} 1 {} {} L {} {} A {} {} 0 {} 0 {} {} Z",
                outer_start_x, outer_start_y,
                outer_radius, outer_radius, large_arc, outer_end_x, outer_end_y,
                inner_end_x, inner_end_y,
                inner_radius, inner_radius, large_arc, inner_start_x, inner_start_y
            )
        } else {
            // Pie slice
            format!(
                "M {} {} L {} {} A {} {} 0 {} 1 {} {} Z",
                cx, cy,
                outer_start_x, outer_start_y,
                outer_radius, outer_radius, large_arc, outer_end_x, outer_end_y
            )
        }
    }

    fn render_gridlines(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        let plot = &layout.plot_area;
        let stroke = self.gridline_color.to_css();

        // Horizontal gridlines
        for &y in &layout.horizontal_gridlines {
            primitives.push(ChartRenderPrimitive::Line {
                x1: plot.x,
                y1: y,
                x2: plot.right(),
                y2: y,
                stroke: stroke.clone(),
                stroke_width: self.gridline_width,
            });
        }

        // Vertical gridlines
        for &x in &layout.vertical_gridlines {
            primitives.push(ChartRenderPrimitive::Line {
                x1: x,
                y1: plot.y,
                x2: x,
                y2: plot.bottom(),
                stroke: stroke.clone(),
                stroke_width: self.gridline_width,
            });
        }
    }

    fn render_axes(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        let stroke = self.axis_color.to_css();

        // Category axis
        if let Some(ref axis) = layout.category_axis {
            // Axis line
            primitives.push(ChartRenderPrimitive::Line {
                x1: axis.line_start.x,
                y1: axis.line_start.y,
                x2: axis.line_end.x,
                y2: axis.line_end.y,
                stroke: stroke.clone(),
                stroke_width: self.axis_width,
            });

            // Tick labels
            for tick in &axis.ticks {
                primitives.push(ChartRenderPrimitive::Text {
                    x: tick.position,
                    y: axis.line_start.y + 15.0,
                    text: tick.label.clone(),
                    font_size: self.label_font_size,
                    font_family: self.font_family.clone(),
                    fill: self.text_color.to_css(),
                    anchor: TextAnchor::Middle,
                    baseline: TextBaseline::Top,
                });
            }

            // Axis title
            if let (Some(ref bounds), Some(ref text)) = (&axis.title, &axis.title_text) {
                primitives.push(ChartRenderPrimitive::Text {
                    x: bounds.center_x(),
                    y: bounds.center_y(),
                    text: text.clone(),
                    font_size: self.label_font_size,
                    font_family: self.font_family.clone(),
                    fill: self.text_color.to_css(),
                    anchor: TextAnchor::Middle,
                    baseline: TextBaseline::Middle,
                });
            }
        }

        // Value axis
        if let Some(ref axis) = layout.value_axis {
            // Axis line
            primitives.push(ChartRenderPrimitive::Line {
                x1: axis.line_start.x,
                y1: axis.line_start.y,
                x2: axis.line_end.x,
                y2: axis.line_end.y,
                stroke: stroke.clone(),
                stroke_width: self.axis_width,
            });

            // Tick labels
            for tick in &axis.ticks {
                primitives.push(ChartRenderPrimitive::Text {
                    x: axis.line_start.x - 8.0,
                    y: tick.position,
                    text: tick.label.clone(),
                    font_size: self.label_font_size,
                    font_family: self.font_family.clone(),
                    fill: self.text_color.to_css(),
                    anchor: TextAnchor::End,
                    baseline: TextBaseline::Middle,
                });
            }

            // Axis title (rotated for vertical axis)
            if let (Some(ref _bounds), Some(ref text)) = (&axis.title, &axis.title_text) {
                // For vertical axis, we'd need rotation which is complex in SVG
                // For now, just place it at the top
                primitives.push(ChartRenderPrimitive::Text {
                    x: axis.line_start.x - 30.0,
                    y: axis.line_start.y,
                    text: text.clone(),
                    font_size: self.label_font_size,
                    font_family: self.font_family.clone(),
                    fill: self.text_color.to_css(),
                    anchor: TextAnchor::Middle,
                    baseline: TextBaseline::Bottom,
                });
            }
        }
    }

    fn render_bars(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        for bar in &layout.bars {
            primitives.push(ChartRenderPrimitive::Rect {
                x: bar.bounds.x,
                y: bar.bounds.y,
                width: bar.bounds.width,
                height: bar.bounds.height,
                fill: bar.color.to_css(),
                stroke: Some(self.darken_color(&bar.color).to_css()),
                stroke_width: Some(self.bar_stroke_width),
            });
        }
    }

    fn render_lines(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        // Group lines by series for smooth rendering
        if layout.lines.is_empty() {
            return;
        }

        // Build polylines by series
        let mut series_points: std::collections::HashMap<usize, Vec<(f64, f64)>> =
            std::collections::HashMap::new();
        let mut series_colors: std::collections::HashMap<usize, Color> =
            std::collections::HashMap::new();

        for line in &layout.lines {
            let points = series_points.entry(line.series_index).or_default();
            if points.is_empty() {
                points.push((line.start.x, line.start.y));
            }
            points.push((line.end.x, line.end.y));
            series_colors.insert(line.series_index, line.color);
        }

        for (series_idx, points) in series_points {
            if let Some(color) = series_colors.get(&series_idx) {
                primitives.push(ChartRenderPrimitive::Polyline {
                    points,
                    stroke: color.to_css(),
                    stroke_width: self.line_width,
                    fill: None,
                });
            }
        }
    }

    fn render_markers(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        for marker in &layout.markers {
            primitives.push(ChartRenderPrimitive::Circle {
                cx: marker.center.x,
                cy: marker.center.y,
                r: marker.radius,
                fill: marker.color.to_css(),
                stroke: Some(self.darken_color(&marker.color).to_css()),
                stroke_width: Some(1.0),
            });
        }
    }

    fn render_pie_slices(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        for slice in &layout.pie_slices {
            // Calculate explosion offset
            let mid_angle = (slice.start_angle + slice.end_angle) / 2.0;
            let offset_x = slice.explosion_offset * mid_angle.cos();
            let offset_y = slice.explosion_offset * mid_angle.sin();

            primitives.push(ChartRenderPrimitive::Arc {
                cx: slice.center.x + offset_x,
                cy: slice.center.y + offset_y,
                inner_radius: slice.inner_radius,
                outer_radius: slice.outer_radius,
                start_angle: slice.start_angle,
                end_angle: slice.end_angle,
                fill: slice.color.to_css(),
                stroke: Some(Color::WHITE.to_css()),
                stroke_width: Some(1.0),
            });
        }
    }

    fn render_areas(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        for area in &layout.areas {
            if area.top_points.is_empty() {
                continue;
            }

            // Build polygon from top points (forward) and bottom points (reverse)
            let mut points: Vec<(f64, f64)> = area
                .top_points
                .iter()
                .map(|p| (p.x, p.y))
                .collect();

            for p in area.bottom_points.iter().rev() {
                points.push((p.x, p.y));
            }

            primitives.push(ChartRenderPrimitive::Polygon {
                points,
                fill: self.color_with_alpha(&area.color, 0.5).to_css(),
                stroke: Some(area.color.to_css()),
                stroke_width: Some(1.0),
            });
        }
    }

    fn render_legend(&self, primitives: &mut Vec<ChartRenderPrimitive>, legend: &LegendLayout) {
        // Legend background
        primitives.push(ChartRenderPrimitive::Rect {
            x: legend.bounds.x,
            y: legend.bounds.y,
            width: legend.bounds.width,
            height: legend.bounds.height,
            fill: Color::rgba(255, 255, 255, 230).to_css(),
            stroke: Some(Color::rgb(200, 200, 200).to_css()),
            stroke_width: Some(1.0),
        });

        for entry in &legend.entries {
            // Color box
            primitives.push(ChartRenderPrimitive::Rect {
                x: entry.color_box.x,
                y: entry.color_box.y,
                width: entry.color_box.width,
                height: entry.color_box.height,
                fill: self.get_series_color(entry.series_index).to_css(),
                stroke: None,
                stroke_width: None,
            });

            // Text
            primitives.push(ChartRenderPrimitive::Text {
                x: entry.text_position.x,
                y: entry.text_position.y,
                text: entry.text.clone(),
                font_size: self.legend_font_size,
                font_family: self.font_family.clone(),
                fill: self.text_color.to_css(),
                anchor: TextAnchor::Start,
                baseline: TextBaseline::Middle,
            });
        }
    }

    fn render_data_labels(&self, primitives: &mut Vec<ChartRenderPrimitive>, layout: &ChartLayout) {
        for label in &layout.data_labels {
            primitives.push(ChartRenderPrimitive::Text {
                x: label.position.x,
                y: label.position.y,
                text: label.text.clone(),
                font_size: self.label_font_size,
                font_family: self.font_family.clone(),
                fill: self.text_color.to_css(),
                anchor: TextAnchor::Middle,
                baseline: TextBaseline::Middle,
            });
        }
    }

    fn darken_color(&self, color: &Color) -> Color {
        Color::rgba(
            (color.r as f64 * 0.7) as u8,
            (color.g as f64 * 0.7) as u8,
            (color.b as f64 * 0.7) as u8,
            color.a,
        )
    }

    fn color_with_alpha(&self, color: &Color, alpha: f64) -> Color {
        Color::rgba(color.r, color.g, color.b, (alpha * 255.0) as u8)
    }

    fn get_series_color(&self, index: usize) -> Color {
        let colors = vec![
            Color::rgb(79, 129, 189),
            Color::rgb(192, 80, 77),
            Color::rgb(155, 187, 89),
            Color::rgb(128, 100, 162),
            Color::rgb(75, 172, 198),
            Color::rgb(247, 150, 70),
        ];
        colors[index % colors.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chart() -> Chart {
        let mut chart = Chart::new(
            "test",
            ChartType::Bar {
                horizontal: false,
                stacked: false,
                stacked_percent: false,
            },
        );
        chart.set_categories(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        chart.add_series(DataSeries::new("Series 1", vec![10.0, 20.0, 30.0]));
        chart
    }

    #[test]
    fn test_render_bar_chart() {
        let chart = create_test_chart();
        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let rendered = renderer.render(&chart, &layout);

        assert!(rendered.primitives.len() > 0);
        assert_eq!(rendered.width, 400.0);
        assert_eq!(rendered.height, 300.0);
    }

    #[test]
    fn test_render_to_svg() {
        let chart = create_test_chart();
        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("rect"));
    }

    #[test]
    fn test_render_line_chart() {
        let mut chart = Chart::new("test", ChartType::Line { smooth: false, markers: true });
        chart.add_series(DataSeries::new("Line", vec![5.0, 15.0, 10.0, 20.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        assert!(svg.contains("polyline") || svg.contains("line"));
        assert!(svg.contains("circle"));
    }

    #[test]
    fn test_render_pie_chart() {
        let mut chart = Chart::new("test", ChartType::Pie { doughnut: false, explosion: 0.0 });
        chart.add_series(DataSeries::new("Pie", vec![25.0, 50.0, 25.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        assert!(svg.contains("path"));
    }

    #[test]
    fn test_render_with_title() {
        let mut chart = create_test_chart();
        chart.title = Some(ChartTitle {
            text: "Test Title".to_string(),
            position: TitlePosition::Top,
        });

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        assert!(svg.contains("Test Title"));
    }

    #[test]
    fn test_render_with_legend() {
        let mut chart = create_test_chart();
        chart.legend = Some(Legend {
            position: LegendPosition::Right,
            visible: true,
        });

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        assert!(svg.contains("Series 1"));
    }

    #[test]
    fn test_arc_to_path() {
        let renderer = ChartRenderer::new();

        // Test pie slice path
        let path = renderer.arc_to_path(100.0, 100.0, 0.0, 50.0, 0.0, PI / 2.0);
        assert!(path.contains("M"));
        assert!(path.contains("A"));
        assert!(path.contains("Z"));
    }

    #[test]
    fn test_arc_to_path_doughnut() {
        let renderer = ChartRenderer::new();

        // Test doughnut slice path
        let path = renderer.arc_to_path(100.0, 100.0, 25.0, 50.0, 0.0, PI / 2.0);
        assert!(path.contains("M"));
        assert!(path.contains("A"));
        assert!(path.contains("L"));
        assert!(path.contains("Z"));
    }

    #[test]
    fn test_escape_xml() {
        let renderer = ChartRenderer::new();

        let escaped = renderer.escape_xml("<test & \"value\">");
        assert_eq!(escaped, "&lt;test &amp; &quot;value&quot;&gt;");
    }

    #[test]
    fn test_darken_color() {
        let renderer = ChartRenderer::new();
        let color = Color::rgb(100, 100, 100);
        let darkened = renderer.darken_color(&color);

        assert!(darkened.r < color.r);
        assert!(darkened.g < color.g);
        assert!(darkened.b < color.b);
    }

    #[test]
    fn test_color_with_alpha() {
        let renderer = ChartRenderer::new();
        let color = Color::rgb(100, 100, 100);
        let with_alpha = renderer.color_with_alpha(&color, 0.5);

        assert_eq!(with_alpha.r, 100);
        assert_eq!(with_alpha.g, 100);
        assert_eq!(with_alpha.b, 100);
        assert_eq!(with_alpha.a, 127);
    }

    #[test]
    fn test_render_gridlines() {
        let chart = create_test_chart();
        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        // Should contain gridlines (lines)
        assert!(svg.contains("line"));
    }

    #[test]
    fn test_render_axes() {
        let chart = create_test_chart();
        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        // Should contain axis labels
        assert!(svg.contains("text"));
    }

    #[test]
    fn test_render_area_chart() {
        let mut chart = Chart::new("test", ChartType::Area { stacked: false });
        chart.add_series(DataSeries::new("Area", vec![10.0, 20.0, 15.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        let renderer = ChartRenderer::new();
        let svg = renderer.render_svg(&chart, &layout);

        assert!(svg.contains("polygon"));
    }

    #[test]
    fn test_primitive_types() {
        let renderer = ChartRenderer::new();

        // Test rect
        let rect = ChartRenderPrimitive::Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            fill: "red".to_string(),
            stroke: Some("black".to_string()),
            stroke_width: Some(1.0),
        };
        let svg = renderer.primitive_to_svg(&rect);
        assert!(svg.contains("rect"));

        // Test circle
        let circle = ChartRenderPrimitive::Circle {
            cx: 50.0,
            cy: 50.0,
            r: 25.0,
            fill: "blue".to_string(),
            stroke: None,
            stroke_width: None,
        };
        let svg = renderer.primitive_to_svg(&circle);
        assert!(svg.contains("circle"));

        // Test text
        let text = ChartRenderPrimitive::Text {
            x: 10.0,
            y: 20.0,
            text: "Hello".to_string(),
            font_size: 12.0,
            font_family: "Arial".to_string(),
            fill: "black".to_string(),
            anchor: TextAnchor::Start,
            baseline: TextBaseline::Alphabetic,
        };
        let svg = renderer.primitive_to_svg(&text);
        assert!(svg.contains("text"));
        assert!(svg.contains("Hello"));
    }
}
