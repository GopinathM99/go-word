//! Chart layout calculations
//!
//! This module handles calculating the layout of chart elements
//! including the plot area, title, legend, axes, and data points.

use crate::model::*;
use serde::{Deserialize, Serialize};

/// A rectangle in layout coordinates
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutRect {
    /// Create a new rectangle
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Get the right edge
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Get the bottom edge
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Get the center X coordinate
    pub fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    /// Get the center Y coordinate
    pub fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    /// Shrink the rectangle by the given padding
    pub fn inset(&self, padding: f64) -> Self {
        Self {
            x: self.x + padding,
            y: self.y + padding,
            width: (self.width - 2.0 * padding).max(0.0),
            height: (self.height - 2.0 * padding).max(0.0),
        }
    }

    /// Shrink by different amounts on each side
    pub fn inset_sides(&self, top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            x: self.x + left,
            y: self.y + top,
            width: (self.width - left - right).max(0.0),
            height: (self.height - top - bottom).max(0.0),
        }
    }
}

/// A point in layout coordinates
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LayoutPoint {
    pub x: f64,
    pub y: f64,
}

impl LayoutPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Layout for a bar/column in a bar chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarLayout {
    pub bounds: LayoutRect,
    pub series_index: usize,
    pub category_index: usize,
    pub value: f64,
    pub color: Color,
}

/// Layout for a line segment in a line chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSegmentLayout {
    pub start: LayoutPoint,
    pub end: LayoutPoint,
    pub series_index: usize,
    pub color: Color,
}

/// Layout for a data point marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerLayout {
    pub center: LayoutPoint,
    pub radius: f64,
    pub series_index: usize,
    pub category_index: usize,
    pub value: f64,
    pub color: Color,
}

/// Layout for a pie slice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieSliceLayout {
    pub center: LayoutPoint,
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub value: f64,
    pub percentage: f64,
    pub category_index: usize,
    pub color: Color,
    pub explosion_offset: f64,
}

/// Layout for an area region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaLayout {
    /// Points defining the top of the area
    pub top_points: Vec<LayoutPoint>,
    /// Points defining the bottom of the area (for stacked)
    pub bottom_points: Vec<LayoutPoint>,
    pub series_index: usize,
    pub color: Color,
}

/// Layout for axis tick marks and labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisTickLayout {
    pub position: f64,
    pub label: String,
    pub is_major: bool,
}

/// Layout for a complete axis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisLayout {
    pub line_start: LayoutPoint,
    pub line_end: LayoutPoint,
    pub ticks: Vec<AxisTickLayout>,
    pub title: Option<LayoutRect>,
    pub title_text: Option<String>,
    pub orientation: AxisOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AxisOrientation {
    Horizontal,
    Vertical,
}

/// Layout for a legend entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendEntryLayout {
    pub bounds: LayoutRect,
    pub color_box: LayoutRect,
    pub text_position: LayoutPoint,
    pub text: String,
    pub series_index: usize,
    pub color: Color,
}

/// Layout for the legend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendLayout {
    pub bounds: LayoutRect,
    pub entries: Vec<LegendEntryLayout>,
}

/// Layout for a data label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLabelLayout {
    pub position: LayoutPoint,
    pub text: String,
    pub series_index: usize,
    pub category_index: usize,
}

/// Complete layout for a chart
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChartLayout {
    /// Total bounds of the chart
    pub total_bounds: LayoutRect,
    /// Title bounds and text
    pub title: Option<(LayoutRect, String)>,
    /// Plot area bounds
    pub plot_area: LayoutRect,
    /// Legend layout
    pub legend: Option<LegendLayout>,
    /// Category axis layout
    pub category_axis: Option<AxisLayout>,
    /// Value axis layout
    pub value_axis: Option<AxisLayout>,
    /// Bar layouts (for bar/column charts)
    pub bars: Vec<BarLayout>,
    /// Line segments (for line charts)
    pub lines: Vec<LineSegmentLayout>,
    /// Markers (for line/scatter charts)
    pub markers: Vec<MarkerLayout>,
    /// Pie slices (for pie/doughnut charts)
    pub pie_slices: Vec<PieSliceLayout>,
    /// Area regions (for area charts)
    pub areas: Vec<AreaLayout>,
    /// Data labels
    pub data_labels: Vec<DataLabelLayout>,
    /// Gridlines (horizontal)
    pub horizontal_gridlines: Vec<f64>,
    /// Gridlines (vertical)
    pub vertical_gridlines: Vec<f64>,
}

/// Layout calculator for charts
pub struct ChartLayoutCalculator {
    /// Font size for title
    pub title_font_size: f64,
    /// Font size for axis labels
    pub axis_label_font_size: f64,
    /// Font size for legend
    pub legend_font_size: f64,
    /// Padding around the chart
    pub padding: f64,
    /// Gap between bars in a group
    pub bar_gap: f64,
    /// Gap between bar groups
    pub bar_group_gap: f64,
    /// Marker radius for line charts
    pub marker_radius: f64,
    /// Legend entry height
    pub legend_entry_height: f64,
}

impl Default for ChartLayoutCalculator {
    fn default() -> Self {
        Self {
            title_font_size: 18.0,
            axis_label_font_size: 12.0,
            legend_font_size: 11.0,
            padding: 10.0,
            bar_gap: 2.0,
            bar_group_gap: 10.0,
            marker_radius: 4.0,
            legend_entry_height: 20.0,
        }
    }
}

impl ChartLayoutCalculator {
    /// Create a new layout calculator
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the complete layout for a chart
    pub fn calculate(&self, chart: &Chart, width: f64, height: f64) -> ChartLayout {
        let total_bounds = LayoutRect::new(0.0, 0.0, width, height);
        let mut layout = ChartLayout {
            total_bounds,
            ..Default::default()
        };

        // Start with the full area
        let mut available = total_bounds.inset(self.padding);

        // Calculate title
        if let Some(ref title) = chart.title {
            let title_height = self.title_font_size * 1.5;
            let title_bounds = LayoutRect::new(
                available.x,
                available.y,
                available.width,
                title_height,
            );
            layout.title = Some((title_bounds, title.text.clone()));
            available = available.inset_sides(title_height + 5.0, 0.0, 0.0, 0.0);
        }

        // Calculate legend
        if let Some(ref legend) = chart.legend {
            if legend.visible {
                let legend_layout = self.calculate_legend(chart, &available, legend);
                match legend.position {
                    LegendPosition::Right | LegendPosition::TopRight => {
                        available = available.inset_sides(0.0, legend_layout.bounds.width + 10.0, 0.0, 0.0);
                    }
                    LegendPosition::Left => {
                        available = available.inset_sides(0.0, 0.0, 0.0, legend_layout.bounds.width + 10.0);
                    }
                    LegendPosition::Top => {
                        available = available.inset_sides(legend_layout.bounds.height + 10.0, 0.0, 0.0, 0.0);
                    }
                    LegendPosition::Bottom => {
                        available = available.inset_sides(0.0, 0.0, legend_layout.bounds.height + 10.0, 0.0);
                    }
                    LegendPosition::None => {}
                }
                layout.legend = Some(legend_layout);
            }
        }

        // Reserve space for axes
        let axis_label_space = self.axis_label_font_size * 2.0;
        let plot_area = match &chart.chart_type {
            ChartType::Pie { .. } => available,
            _ => available.inset_sides(0.0, 0.0, axis_label_space, axis_label_space),
        };
        layout.plot_area = plot_area;

        // Calculate chart-specific layout
        match &chart.chart_type {
            ChartType::Bar { horizontal, stacked, stacked_percent } => {
                self.calculate_bar_layout(chart, &mut layout, *horizontal, *stacked, *stacked_percent);
            }
            ChartType::Column { stacked, stacked_percent } => {
                self.calculate_bar_layout(chart, &mut layout, false, *stacked, *stacked_percent);
            }
            ChartType::Line { smooth: _, markers } => {
                self.calculate_line_layout(chart, &mut layout, *markers);
            }
            ChartType::Pie { doughnut, explosion } => {
                self.calculate_pie_layout(chart, &mut layout, *doughnut, *explosion);
            }
            ChartType::Area { stacked } => {
                self.calculate_area_layout(chart, &mut layout, *stacked);
            }
            ChartType::Scatter { with_lines } => {
                self.calculate_scatter_layout(chart, &mut layout, *with_lines);
            }
            ChartType::Bubble => {
                self.calculate_bubble_layout(chart, &mut layout);
            }
            ChartType::Radar { filled: _ } => {
                self.calculate_radar_layout(chart, &mut layout);
            }
            ChartType::Stock => {
                self.calculate_stock_layout(chart, &mut layout);
            }
        }

        // Calculate axes for non-pie charts
        if !matches!(chart.chart_type, ChartType::Pie { .. }) {
            self.calculate_axes(chart, &mut layout, &available);
        }

        layout
    }

    fn calculate_legend(
        &self,
        chart: &Chart,
        available: &LayoutRect,
        legend: &Legend,
    ) -> LegendLayout {
        let series_count = chart.data.series.len();
        let entry_height = self.legend_entry_height;
        let color_box_size = entry_height * 0.6;
        let text_offset = color_box_size + 8.0;

        // Estimate text width (rough approximation)
        let max_text_width = chart
            .data
            .series
            .iter()
            .map(|s| s.name.len() as f64 * self.legend_font_size * 0.6)
            .fold(0.0, f64::max);

        let entry_width = text_offset + max_text_width;
        let total_height = series_count as f64 * entry_height;

        let bounds = match legend.position {
            LegendPosition::Right | LegendPosition::TopRight => LayoutRect::new(
                available.right() - entry_width - 10.0,
                available.y,
                entry_width,
                total_height,
            ),
            LegendPosition::Left => LayoutRect::new(
                available.x + 10.0,
                available.y,
                entry_width,
                total_height,
            ),
            LegendPosition::Top => LayoutRect::new(
                available.x,
                available.y,
                available.width,
                entry_height,
            ),
            LegendPosition::Bottom => LayoutRect::new(
                available.x,
                available.bottom() - entry_height,
                available.width,
                entry_height,
            ),
            LegendPosition::None => LayoutRect::default(),
        };

        let mut entries = Vec::new();
        for (idx, series) in chart.data.series.iter().enumerate() {
            let color = series
                .color
                .unwrap_or_else(|| chart.style.colors.get(idx % chart.style.colors.len()).copied().unwrap_or(Color::BLUE));

            let entry_y = bounds.y + idx as f64 * entry_height;
            let entry_bounds = LayoutRect::new(bounds.x, entry_y, bounds.width, entry_height);
            let color_box = LayoutRect::new(
                bounds.x + 4.0,
                entry_y + (entry_height - color_box_size) / 2.0,
                color_box_size,
                color_box_size,
            );
            let text_position = LayoutPoint::new(
                bounds.x + text_offset,
                entry_y + entry_height / 2.0 + self.legend_font_size / 3.0,
            );

            entries.push(LegendEntryLayout {
                bounds: entry_bounds,
                color_box,
                text_position,
                text: series.name.clone(),
                series_index: idx,
                color,
            });
        }

        LegendLayout { bounds, entries }
    }

    fn calculate_bar_layout(
        &self,
        chart: &Chart,
        layout: &mut ChartLayout,
        horizontal: bool,
        stacked: bool,
        _stacked_percent: bool,
    ) {
        let plot = &layout.plot_area;
        let data = &chart.data;

        if data.series.is_empty() {
            return;
        }

        let category_count = data.data_point_count().max(1);
        let series_count = data.series.len();

        // Calculate value range
        let (min_val, max_val) = if stacked {
            let totals = data.stacked_totals();
            let max = totals.iter().cloned().fold(0.0, f64::max);
            (0.0, max.max(1.0))
        } else {
            let min = data.min_value().min(0.0);
            let max = data.max_value().max(1.0);
            (min, max)
        };
        let value_range = max_val - min_val;

        // Calculate bar dimensions
        let (category_size, value_size) = if horizontal {
            (plot.height / category_count as f64, plot.width)
        } else {
            (plot.width / category_count as f64, plot.height)
        };

        let bar_area = category_size - self.bar_group_gap;
        let bar_width = if stacked {
            bar_area
        } else {
            (bar_area - self.bar_gap * (series_count - 1) as f64) / series_count as f64
        };

        let mut stacked_offsets = vec![0.0; category_count];

        for (series_idx, series) in data.series.iter().enumerate() {
            let color = series
                .color
                .unwrap_or_else(|| chart.style.colors.get(series_idx % chart.style.colors.len()).copied().unwrap_or(Color::BLUE));

            for (cat_idx, &value) in series.values.iter().enumerate() {
                let normalized = (value - min_val) / value_range;
                let bar_length = normalized * value_size;

                let (x, y, w, h) = if stacked {
                    if horizontal {
                        let x = plot.x + stacked_offsets[cat_idx];
                        let y = plot.y + cat_idx as f64 * category_size + self.bar_group_gap / 2.0;
                        stacked_offsets[cat_idx] += bar_length;
                        (x, y, bar_length, bar_width)
                    } else {
                        let x = plot.x + cat_idx as f64 * category_size + self.bar_group_gap / 2.0;
                        let y = plot.bottom() - stacked_offsets[cat_idx] - bar_length;
                        stacked_offsets[cat_idx] += bar_length;
                        (x, y, bar_width, bar_length)
                    }
                } else if horizontal {
                    let x = plot.x;
                    let y = plot.y
                        + cat_idx as f64 * category_size
                        + self.bar_group_gap / 2.0
                        + series_idx as f64 * (bar_width + self.bar_gap);
                    (x, y, bar_length, bar_width)
                } else {
                    let x = plot.x
                        + cat_idx as f64 * category_size
                        + self.bar_group_gap / 2.0
                        + series_idx as f64 * (bar_width + self.bar_gap);
                    let y = plot.bottom() - bar_length;
                    (x, y, bar_width, bar_length)
                };

                layout.bars.push(BarLayout {
                    bounds: LayoutRect::new(x, y, w, h),
                    series_index: series_idx,
                    category_index: cat_idx,
                    value,
                    color,
                });
            }
        }

        // Calculate gridlines
        self.calculate_gridlines(layout, min_val, max_val, horizontal);
    }

    fn calculate_line_layout(&self, chart: &Chart, layout: &mut ChartLayout, markers: bool) {
        let plot = &layout.plot_area;
        let data = &chart.data;

        if data.series.is_empty() {
            return;
        }

        let point_count = data.data_point_count();
        if point_count == 0 {
            return;
        }

        let min_val = data.min_value().min(0.0);
        let max_val = data.max_value().max(1.0);
        let value_range = max_val - min_val;

        let x_step = if point_count > 1 {
            plot.width / (point_count - 1) as f64
        } else {
            plot.width
        };

        for (series_idx, series) in data.series.iter().enumerate() {
            let color = series
                .color
                .unwrap_or_else(|| chart.style.colors.get(series_idx % chart.style.colors.len()).copied().unwrap_or(Color::BLUE));

            let mut prev_point: Option<LayoutPoint> = None;

            for (cat_idx, &value) in series.values.iter().enumerate() {
                let normalized = (value - min_val) / value_range;
                let x = plot.x + cat_idx as f64 * x_step;
                let y = plot.bottom() - normalized * plot.height;
                let point = LayoutPoint::new(x, y);

                if let Some(prev) = prev_point {
                    layout.lines.push(LineSegmentLayout {
                        start: prev,
                        end: point,
                        series_index: series_idx,
                        color,
                    });
                }

                if markers {
                    layout.markers.push(MarkerLayout {
                        center: point,
                        radius: self.marker_radius,
                        series_index: series_idx,
                        category_index: cat_idx,
                        value,
                        color,
                    });
                }

                prev_point = Some(point);
            }
        }

        self.calculate_gridlines(layout, min_val, max_val, false);
    }

    fn calculate_pie_layout(
        &self,
        chart: &Chart,
        layout: &mut ChartLayout,
        doughnut: bool,
        explosion: f32,
    ) {
        let plot = &layout.plot_area;

        if chart.data.series.is_empty() {
            return;
        }

        // Use first series for pie chart
        let series = &chart.data.series[0];
        let total: f64 = series.values.iter().sum();
        if total == 0.0 {
            return;
        }

        let center = LayoutPoint::new(plot.center_x(), plot.center_y());
        let max_radius = plot.width.min(plot.height) / 2.0 * 0.9;
        let outer_radius = max_radius;
        let inner_radius = if doughnut { outer_radius * 0.5 } else { 0.0 };

        let mut current_angle = -std::f64::consts::FRAC_PI_2; // Start at top

        for (idx, &value) in series.values.iter().enumerate() {
            let percentage = value / total;
            let sweep_angle = percentage * std::f64::consts::PI * 2.0;
            let end_angle = current_angle + sweep_angle;

            let color = chart
                .style
                .colors
                .get(idx % chart.style.colors.len())
                .copied()
                .unwrap_or(Color::BLUE);

            layout.pie_slices.push(PieSliceLayout {
                center,
                inner_radius,
                outer_radius,
                start_angle: current_angle,
                end_angle,
                value,
                percentage,
                category_index: idx,
                color,
                explosion_offset: explosion as f64 / 100.0 * max_radius,
            });

            current_angle = end_angle;
        }
    }

    fn calculate_area_layout(&self, chart: &Chart, layout: &mut ChartLayout, stacked: bool) {
        let plot = &layout.plot_area;
        let data = &chart.data;

        if data.series.is_empty() {
            return;
        }

        let point_count = data.data_point_count();
        if point_count == 0 {
            return;
        }

        let (min_val, max_val) = if stacked {
            let totals = data.stacked_totals();
            (0.0, totals.iter().cloned().fold(0.0, f64::max).max(1.0))
        } else {
            (data.min_value().min(0.0), data.max_value().max(1.0))
        };
        let value_range = max_val - min_val;

        let x_step = if point_count > 1 {
            plot.width / (point_count - 1) as f64
        } else {
            plot.width
        };

        let mut baseline = vec![plot.bottom(); point_count];

        for (series_idx, series) in data.series.iter().enumerate() {
            let color = series
                .color
                .unwrap_or_else(|| chart.style.colors.get(series_idx % chart.style.colors.len()).copied().unwrap_or(Color::BLUE));

            let mut top_points = Vec::new();
            let mut bottom_points = Vec::new();

            for (cat_idx, &value) in series.values.iter().enumerate() {
                let normalized = (value - min_val) / value_range;
                let x = plot.x + cat_idx as f64 * x_step;
                let height = normalized * plot.height;

                if stacked {
                    let top_y = baseline[cat_idx] - height;
                    top_points.push(LayoutPoint::new(x, top_y));
                    bottom_points.push(LayoutPoint::new(x, baseline[cat_idx]));
                    baseline[cat_idx] = top_y;
                } else {
                    let y = plot.bottom() - height;
                    top_points.push(LayoutPoint::new(x, y));
                    bottom_points.push(LayoutPoint::new(x, plot.bottom()));
                }
            }

            layout.areas.push(AreaLayout {
                top_points,
                bottom_points,
                series_index: series_idx,
                color,
            });
        }

        self.calculate_gridlines(layout, min_val, max_val, false);
    }

    fn calculate_scatter_layout(&self, chart: &Chart, layout: &mut ChartLayout, with_lines: bool) {
        // Scatter is similar to line but typically uses two value axes
        self.calculate_line_layout(chart, layout, true);

        if !with_lines {
            layout.lines.clear();
        }
    }

    fn calculate_bubble_layout(&self, chart: &Chart, layout: &mut ChartLayout) {
        // Bubble is similar to scatter with varying marker sizes
        self.calculate_scatter_layout(chart, layout, false);

        // Scale marker sizes based on values (simplified)
        for marker in &mut layout.markers {
            marker.radius = (marker.value.abs().sqrt() * 2.0).max(4.0).min(30.0);
        }
    }

    fn calculate_radar_layout(&self, chart: &Chart, layout: &mut ChartLayout) {
        let plot = &layout.plot_area;
        let data = &chart.data;

        if data.series.is_empty() {
            return;
        }

        let point_count = data.data_point_count();
        if point_count == 0 {
            return;
        }

        let center = LayoutPoint::new(plot.center_x(), plot.center_y());
        let radius = plot.width.min(plot.height) / 2.0 * 0.8;
        let angle_step = std::f64::consts::PI * 2.0 / point_count as f64;

        let max_val = data.max_value().max(1.0);

        for (series_idx, series) in data.series.iter().enumerate() {
            let color = series
                .color
                .unwrap_or_else(|| chart.style.colors.get(series_idx % chart.style.colors.len()).copied().unwrap_or(Color::BLUE));

            let mut prev_point: Option<LayoutPoint> = None;
            let mut first_point: Option<LayoutPoint> = None;

            for (cat_idx, &value) in series.values.iter().enumerate() {
                let normalized = value / max_val;
                let angle = cat_idx as f64 * angle_step - std::f64::consts::FRAC_PI_2;
                let r = normalized * radius;
                let point = LayoutPoint::new(
                    center.x + r * angle.cos(),
                    center.y + r * angle.sin(),
                );

                if first_point.is_none() {
                    first_point = Some(point);
                }

                if let Some(prev) = prev_point {
                    layout.lines.push(LineSegmentLayout {
                        start: prev,
                        end: point,
                        series_index: series_idx,
                        color,
                    });
                }

                layout.markers.push(MarkerLayout {
                    center: point,
                    radius: self.marker_radius,
                    series_index: series_idx,
                    category_index: cat_idx,
                    value,
                    color,
                });

                prev_point = Some(point);
            }

            // Close the polygon
            if let (Some(first), Some(last)) = (first_point, prev_point) {
                layout.lines.push(LineSegmentLayout {
                    start: last,
                    end: first,
                    series_index: series_idx,
                    color,
                });
            }
        }
    }

    fn calculate_stock_layout(&self, chart: &Chart, layout: &mut ChartLayout) {
        // Stock charts are similar to line charts but with special rendering
        self.calculate_line_layout(chart, layout, true);
    }

    fn calculate_gridlines(&self, layout: &mut ChartLayout, min_val: f64, max_val: f64, horizontal: bool) {
        let plot = &layout.plot_area;
        let range = max_val - min_val;

        // Calculate nice tick intervals
        let tick_count = 5;
        let raw_step = range / tick_count as f64;
        let magnitude = 10_f64.powf(raw_step.log10().floor());
        let step = (raw_step / magnitude).ceil() * magnitude;

        let start = (min_val / step).floor() * step;
        let mut tick = start;

        while tick <= max_val {
            let normalized = (tick - min_val) / range;
            if horizontal {
                let x = plot.x + normalized * plot.width;
                layout.vertical_gridlines.push(x);
            } else {
                let y = plot.bottom() - normalized * plot.height;
                layout.horizontal_gridlines.push(y);
            }
            tick += step;
        }
    }

    fn calculate_axes(&self, chart: &Chart, layout: &mut ChartLayout, available: &LayoutRect) {
        let plot = &layout.plot_area;

        // Category axis (bottom for most charts)
        if chart.axes.category_axis.is_some() || !matches!(chart.chart_type, ChartType::Pie { .. }) {
            let mut ticks = Vec::new();
            let category_count = chart.data.categories.len().max(chart.data.data_point_count());

            if category_count > 0 {
                let step = plot.width / category_count as f64;
                for (idx, category) in chart.data.categories.iter().enumerate() {
                    ticks.push(AxisTickLayout {
                        position: plot.x + (idx as f64 + 0.5) * step,
                        label: category.clone(),
                        is_major: true,
                    });
                }
            }

            layout.category_axis = Some(AxisLayout {
                line_start: LayoutPoint::new(plot.x, plot.bottom()),
                line_end: LayoutPoint::new(plot.right(), plot.bottom()),
                ticks,
                title: chart.axes.category_axis.as_ref().and_then(|a| {
                    a.title.as_ref().map(|_| LayoutRect::new(
                        plot.x,
                        available.bottom() - self.axis_label_font_size,
                        plot.width,
                        self.axis_label_font_size,
                    ))
                }),
                title_text: chart.axes.category_axis.as_ref().and_then(|a| a.title.clone()),
                orientation: AxisOrientation::Horizontal,
            });
        }

        // Value axis (left for most charts)
        if chart.axes.value_axis.is_some() || !matches!(chart.chart_type, ChartType::Pie { .. }) {
            let mut ticks = Vec::new();

            // Generate value ticks based on gridlines
            let min_val = chart.data.min_value().min(0.0);
            let max_val = chart.data.max_value().max(1.0);
            let range = max_val - min_val;

            for &y in &layout.horizontal_gridlines {
                let normalized = (plot.bottom() - y) / plot.height;
                let value = min_val + normalized * range;
                ticks.push(AxisTickLayout {
                    position: y,
                    label: format!("{:.0}", value),
                    is_major: true,
                });
            }

            layout.value_axis = Some(AxisLayout {
                line_start: LayoutPoint::new(plot.x, plot.y),
                line_end: LayoutPoint::new(plot.x, plot.bottom()),
                ticks,
                title: chart.axes.value_axis.as_ref().and_then(|a| {
                    a.title.as_ref().map(|_| LayoutRect::new(
                        available.x,
                        plot.y,
                        self.axis_label_font_size,
                        plot.height,
                    ))
                }),
                title_text: chart.axes.value_axis.as_ref().and_then(|a| a.title.clone()),
                orientation: AxisOrientation::Vertical,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_rect_inset() {
        let rect = LayoutRect::new(10.0, 20.0, 100.0, 80.0);
        let inset = rect.inset(5.0);

        assert_eq!(inset.x, 15.0);
        assert_eq!(inset.y, 25.0);
        assert_eq!(inset.width, 90.0);
        assert_eq!(inset.height, 70.0);
    }

    #[test]
    fn test_layout_rect_center() {
        let rect = LayoutRect::new(0.0, 0.0, 100.0, 80.0);

        assert_eq!(rect.center_x(), 50.0);
        assert_eq!(rect.center_y(), 40.0);
    }

    #[test]
    fn test_calculate_bar_chart_layout() {
        let mut chart = Chart::new(
            "test",
            ChartType::Bar {
                horizontal: false,
                stacked: false,
                stacked_percent: false,
            },
        );
        chart.add_series(DataSeries::new("Series 1", vec![10.0, 20.0, 30.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.bars.is_empty());
        assert_eq!(layout.bars.len(), 3);
    }

    #[test]
    fn test_calculate_line_chart_layout() {
        let mut chart = Chart::new("test", ChartType::Line { smooth: false, markers: true });
        chart.add_series(DataSeries::new("Line", vec![5.0, 15.0, 10.0, 20.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.lines.is_empty());
        assert!(!layout.markers.is_empty());
        assert_eq!(layout.lines.len(), 3); // 4 points = 3 segments
        assert_eq!(layout.markers.len(), 4);
    }

    #[test]
    fn test_calculate_pie_chart_layout() {
        let mut chart = Chart::new("test", ChartType::Pie { doughnut: false, explosion: 0.0 });
        chart.add_series(DataSeries::new("Pie", vec![25.0, 50.0, 25.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.pie_slices.is_empty());
        assert_eq!(layout.pie_slices.len(), 3);

        // Check percentages
        assert!((layout.pie_slices[0].percentage - 0.25).abs() < 0.001);
        assert!((layout.pie_slices[1].percentage - 0.50).abs() < 0.001);
        assert!((layout.pie_slices[2].percentage - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_calculate_doughnut_chart_layout() {
        let mut chart = Chart::new("test", ChartType::Pie { doughnut: true, explosion: 0.0 });
        chart.add_series(DataSeries::new("Doughnut", vec![30.0, 70.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.pie_slices.is_empty());
        assert!(layout.pie_slices[0].inner_radius > 0.0);
    }

    #[test]
    fn test_calculate_stacked_bar_layout() {
        let mut chart = Chart::new(
            "test",
            ChartType::Bar {
                horizontal: false,
                stacked: true,
                stacked_percent: false,
            },
        );
        chart.add_series(DataSeries::new("Series 1", vec![10.0, 20.0]));
        chart.add_series(DataSeries::new("Series 2", vec![15.0, 10.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.bars.is_empty());
        assert_eq!(layout.bars.len(), 4); // 2 series x 2 categories
    }

    #[test]
    fn test_calculate_area_chart_layout() {
        let mut chart = Chart::new("test", ChartType::Area { stacked: false });
        chart.add_series(DataSeries::new("Area", vec![10.0, 20.0, 15.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.areas.is_empty());
        assert_eq!(layout.areas[0].top_points.len(), 3);
    }

    #[test]
    fn test_calculate_layout_with_title() {
        let chart = Chart::new("test", ChartType::default()).with_title("My Chart");

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(layout.title.is_some());
        let (_, title_text) = layout.title.unwrap();
        assert_eq!(title_text, "My Chart");
    }

    #[test]
    fn test_calculate_layout_with_legend() {
        let mut chart = Chart::new("test", ChartType::default())
            .with_legend(LegendPosition::Right);
        chart.add_series(DataSeries::new("A", vec![1.0]));
        chart.add_series(DataSeries::new("B", vec![2.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(layout.legend.is_some());
        assert_eq!(layout.legend.as_ref().unwrap().entries.len(), 2);
    }

    #[test]
    fn test_gridlines_calculation() {
        let mut chart = Chart::new("test", ChartType::default());
        chart.add_series(DataSeries::new("Data", vec![0.0, 50.0, 100.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(!layout.horizontal_gridlines.is_empty());
    }

    #[test]
    fn test_axes_calculation() {
        let mut chart = Chart::new("test", ChartType::default());
        chart.set_categories(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        chart.add_series(DataSeries::new("Data", vec![10.0, 20.0, 30.0]));

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(layout.category_axis.is_some());
        assert!(layout.value_axis.is_some());
        assert_eq!(layout.category_axis.as_ref().unwrap().ticks.len(), 3);
    }

    #[test]
    fn test_empty_chart_layout() {
        let chart = Chart::new("test", ChartType::default());

        let calculator = ChartLayoutCalculator::new();
        let layout = calculator.calculate(&chart, 400.0, 300.0);

        assert!(layout.bars.is_empty());
        assert!(layout.lines.is_empty());
        assert!(layout.pie_slices.is_empty());
    }
}
