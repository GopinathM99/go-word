//! Chart model types
//!
//! This module defines the data structures for representing charts,
//! including chart types, data series, styling, and axes.

use serde::{Deserialize, Serialize};

/// A complete chart representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Unique identifier for this chart
    pub id: String,
    /// The type of chart (bar, line, pie, etc.)
    pub chart_type: ChartType,
    /// The data displayed in the chart
    pub data: ChartData,
    /// Visual styling
    pub style: ChartStyle,
    /// Optional chart title
    pub title: Option<ChartTitle>,
    /// Optional legend configuration
    pub legend: Option<Legend>,
    /// Axis configurations
    pub axes: ChartAxes,
    /// Preserve original XML for round-trip fidelity
    pub original_xml: Option<String>,
}

impl Chart {
    /// Create a new chart with the given ID and type
    pub fn new(id: impl Into<String>, chart_type: ChartType) -> Self {
        Self {
            id: id.into(),
            chart_type,
            data: ChartData::default(),
            style: ChartStyle::default(),
            title: None,
            legend: None,
            axes: ChartAxes::default(),
            original_xml: None,
        }
    }

    /// Set the chart title
    pub fn with_title(mut self, text: impl Into<String>) -> Self {
        self.title = Some(ChartTitle {
            text: text.into(),
            position: TitlePosition::Top,
        });
        self
    }

    /// Set the legend
    pub fn with_legend(mut self, position: LegendPosition) -> Self {
        self.legend = Some(Legend {
            position,
            visible: true,
        });
        self
    }

    /// Add a data series
    pub fn add_series(&mut self, series: DataSeries) {
        self.data.series.push(series);
    }

    /// Set categories
    pub fn set_categories(&mut self, categories: Vec<String>) {
        self.data.categories = categories;
    }
}

/// Types of charts supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ChartType {
    /// Bar chart (horizontal bars)
    Bar {
        horizontal: bool,
        stacked: bool,
        stacked_percent: bool,
    },
    /// Column chart (vertical bars)
    Column {
        stacked: bool,
        stacked_percent: bool,
    },
    /// Line chart
    Line {
        smooth: bool,
        markers: bool,
    },
    /// Pie or doughnut chart
    Pie {
        doughnut: bool,
        explosion: f32,
    },
    /// Scatter/XY chart
    Scatter {
        with_lines: bool,
    },
    /// Area chart
    Area {
        stacked: bool,
    },
    /// Bubble chart
    Bubble,
    /// Radar/spider chart
    Radar {
        filled: bool,
    },
    /// Stock chart (OHLC)
    Stock,
}

impl Default for ChartType {
    fn default() -> Self {
        ChartType::Column {
            stacked: false,
            stacked_percent: false,
        }
    }
}

/// Chart data containing categories and series
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChartData {
    /// Category labels (X-axis labels for most chart types)
    pub categories: Vec<String>,
    /// Data series
    pub series: Vec<DataSeries>,
}

impl ChartData {
    /// Create new chart data with given categories
    pub fn new(categories: Vec<String>) -> Self {
        Self {
            categories,
            series: Vec::new(),
        }
    }

    /// Get the number of data points (max across all series)
    pub fn data_point_count(&self) -> usize {
        self.series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0)
    }

    /// Get the minimum value across all series
    pub fn min_value(&self) -> f64 {
        self.series
            .iter()
            .flat_map(|s| s.values.iter())
            .cloned()
            .fold(f64::INFINITY, f64::min)
    }

    /// Get the maximum value across all series
    pub fn max_value(&self) -> f64 {
        self.series
            .iter()
            .flat_map(|s| s.values.iter())
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Calculate the sum of values at each category index (for stacked charts)
    pub fn stacked_totals(&self) -> Vec<f64> {
        let count = self.data_point_count();
        let mut totals = vec![0.0; count];
        for series in &self.series {
            for (i, &value) in series.values.iter().enumerate() {
                if i < count {
                    totals[i] += value;
                }
            }
        }
        totals
    }
}

/// A single data series in a chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// Name of the series (shown in legend)
    pub name: String,
    /// Numeric values
    pub values: Vec<f64>,
    /// Optional custom color for this series
    pub color: Option<Color>,
    /// Optional data label configuration
    pub data_labels: Option<DataLabelOptions>,
}

impl DataSeries {
    /// Create a new data series with a name and values
    pub fn new(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            values,
            color: None,
            data_labels: None,
        }
    }

    /// Set the series color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Enable data labels
    pub fn with_data_labels(mut self, options: DataLabelOptions) -> Self {
        self.data_labels = Some(options);
        self
    }
}

/// RGBA color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color from RGB values (fully opaque)
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color from RGBA values
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Self::rgb(r, g, b))
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Self::rgba(r, g, b, a))
        } else {
            None
        }
    }

    /// Convert to hex string (without # prefix)
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Convert to CSS color string
    pub fn to_css(&self) -> String {
        if self.a == 255 {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            format!(
                "rgba({}, {}, {}, {:.3})",
                self.r,
                self.g,
                self.b,
                self.a as f64 / 255.0
            )
        }
    }

    // Predefined colors
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);
    pub const CYAN: Color = Color::rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::rgb(255, 0, 255);
    pub const GRAY: Color = Color::rgb(128, 128, 128);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Options for displaying data labels on chart elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLabelOptions {
    /// Show the value
    pub show_value: bool,
    /// Show the category name
    pub show_category: bool,
    /// Show the series name
    pub show_series_name: bool,
    /// Show as percentage (for pie charts)
    pub show_percent: bool,
    /// Number format string
    pub number_format: Option<String>,
    /// Label position
    pub position: DataLabelPosition,
}

impl Default for DataLabelOptions {
    fn default() -> Self {
        Self {
            show_value: true,
            show_category: false,
            show_series_name: false,
            show_percent: false,
            number_format: None,
            position: DataLabelPosition::OutsideEnd,
        }
    }
}

/// Position of data labels relative to data points
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataLabelPosition {
    Center,
    InsideEnd,
    InsideBase,
    OutsideEnd,
    BestFit,
}

/// Visual styling for the chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartStyle {
    /// Color palette for series
    pub colors: Vec<Color>,
    /// Background color
    pub background: Option<Color>,
    /// Border style
    pub border: Option<BorderStyle>,
    /// Plot area background
    pub plot_area_background: Option<Color>,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            colors: default_color_palette(),
            background: Some(Color::WHITE),
            border: None,
            plot_area_background: None,
        }
    }
}

/// Default color palette for charts (Office-like colors)
fn default_color_palette() -> Vec<Color> {
    vec![
        Color::rgb(79, 129, 189),   // Blue
        Color::rgb(192, 80, 77),    // Red
        Color::rgb(155, 187, 89),   // Green
        Color::rgb(128, 100, 162),  // Purple
        Color::rgb(75, 172, 198),   // Teal
        Color::rgb(247, 150, 70),   // Orange
        Color::rgb(119, 146, 60),   // Olive
        Color::rgb(166, 166, 166),  // Gray
    ]
}

/// Border style for chart elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderStyle {
    pub color: Color,
    pub width: f32,
    pub style: LineStyle,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            style: LineStyle::Solid,
        }
    }
}

/// Line style for borders and lines
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
    None,
}

/// Chart title configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartTitle {
    /// Title text
    pub text: String,
    /// Position of the title
    pub position: TitlePosition,
}

/// Position of the chart title
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TitlePosition {
    Top,
    Bottom,
    Left,
    Right,
    Overlay,
}

/// Legend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Legend {
    /// Position of the legend
    pub position: LegendPosition,
    /// Whether the legend is visible
    pub visible: bool,
}

/// Position of the legend
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LegendPosition {
    Top,
    Bottom,
    Left,
    Right,
    TopRight,
    None,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            position: LegendPosition::Right,
            visible: true,
        }
    }
}

/// Axis configurations for the chart
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChartAxes {
    /// Category axis (typically X-axis)
    pub category_axis: Option<Axis>,
    /// Value axis (typically Y-axis)
    pub value_axis: Option<Axis>,
    /// Secondary category axis (for combo charts)
    pub secondary_category_axis: Option<Axis>,
    /// Secondary value axis (for combo charts)
    pub secondary_value_axis: Option<Axis>,
}

/// Configuration for a single axis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axis {
    /// Axis title
    pub title: Option<String>,
    /// Minimum value (None = auto)
    pub min: Option<f64>,
    /// Maximum value (None = auto)
    pub max: Option<f64>,
    /// Show major gridlines
    pub major_gridlines: bool,
    /// Show minor gridlines
    pub minor_gridlines: bool,
    /// Major tick interval (None = auto)
    pub major_unit: Option<f64>,
    /// Minor tick interval (None = auto)
    pub minor_unit: Option<f64>,
    /// Number format for tick labels
    pub number_format: Option<String>,
    /// Axis position
    pub position: AxisPosition,
    /// Whether to show axis line
    pub show_axis_line: bool,
    /// Whether to show tick marks
    pub show_tick_marks: bool,
    /// Whether to show tick labels
    pub show_tick_labels: bool,
    /// Reverse axis direction
    pub reversed: bool,
    /// Logarithmic scale
    pub logarithmic: bool,
    /// Log base (if logarithmic)
    pub log_base: f64,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            title: None,
            min: None,
            max: None,
            major_gridlines: true,
            minor_gridlines: false,
            major_unit: None,
            minor_unit: None,
            number_format: None,
            position: AxisPosition::Bottom,
            show_axis_line: true,
            show_tick_marks: true,
            show_tick_labels: true,
            reversed: false,
            logarithmic: false,
            log_base: 10.0,
        }
    }
}

/// Position of an axis
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AxisPosition {
    Bottom,
    Top,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color, Color::rgb(255, 0, 0));

        let color = Color::from_hex("00FF00").unwrap();
        assert_eq!(color, Color::rgb(0, 255, 0));

        let color = Color::from_hex("#0000FF80").unwrap();
        assert_eq!(color, Color::rgba(0, 0, 255, 128));
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::rgb(255, 128, 0);
        assert_eq!(color.to_hex(), "FF8000");

        let color = Color::rgba(255, 128, 0, 128);
        assert_eq!(color.to_hex(), "FF800080");
    }

    #[test]
    fn test_color_to_css() {
        let color = Color::rgb(255, 128, 0);
        assert_eq!(color.to_css(), "rgb(255, 128, 0)");

        let color = Color::rgba(255, 128, 0, 128);
        assert!(color.to_css().starts_with("rgba(255, 128, 0,"));
    }

    #[test]
    fn test_chart_data_min_max() {
        let mut data = ChartData::new(vec!["A".to_string(), "B".to_string()]);
        data.series.push(DataSeries::new("Series 1", vec![10.0, 20.0]));
        data.series.push(DataSeries::new("Series 2", vec![5.0, 25.0]));

        assert_eq!(data.min_value(), 5.0);
        assert_eq!(data.max_value(), 25.0);
    }

    #[test]
    fn test_chart_data_stacked_totals() {
        let mut data = ChartData::new(vec!["A".to_string(), "B".to_string()]);
        data.series.push(DataSeries::new("Series 1", vec![10.0, 20.0]));
        data.series.push(DataSeries::new("Series 2", vec![5.0, 15.0]));

        let totals = data.stacked_totals();
        assert_eq!(totals, vec![15.0, 35.0]);
    }

    #[test]
    fn test_chart_builder() {
        let chart = Chart::new("chart1", ChartType::Column { stacked: false, stacked_percent: false })
            .with_title("Sales Data")
            .with_legend(LegendPosition::Bottom);

        assert_eq!(chart.id, "chart1");
        assert!(chart.title.is_some());
        assert_eq!(chart.title.as_ref().unwrap().text, "Sales Data");
        assert!(chart.legend.is_some());
        assert_eq!(chart.legend.as_ref().unwrap().position, LegendPosition::Bottom);
    }

    #[test]
    fn test_data_series_builder() {
        let series = DataSeries::new("Test", vec![1.0, 2.0, 3.0])
            .with_color(Color::RED)
            .with_data_labels(DataLabelOptions::default());

        assert_eq!(series.name, "Test");
        assert_eq!(series.color, Some(Color::RED));
        assert!(series.data_labels.is_some());
    }

    #[test]
    fn test_chart_type_default() {
        let chart_type = ChartType::default();
        assert!(matches!(chart_type, ChartType::Column { stacked: false, stacked_percent: false }));
    }

    #[test]
    fn test_axis_default() {
        let axis = Axis::default();
        assert!(axis.major_gridlines);
        assert!(!axis.minor_gridlines);
        assert!(axis.show_axis_line);
        assert!(!axis.reversed);
    }
}
