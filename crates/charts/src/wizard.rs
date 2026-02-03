//! Chart insertion wizard
//!
//! This module provides a wizard-style interface for inserting new charts,
//! guiding users through chart type selection, data entry, and styling.

use crate::error::{ChartError, ChartResult};
use crate::layout::ChartLayoutCalculator;
use crate::model::*;
use crate::render::{ChartRenderer, RenderedChart};
use crate::styles::{ChartStylePreset, ColorScheme};
use serde::{Deserialize, Serialize};

/// Steps in the chart insertion wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WizardStep {
    /// Step 1: Select the chart type
    SelectType,
    /// Step 2: Enter or import initial data
    EnterData,
    /// Step 3: Configure chart options (title, legend, etc.)
    ConfigureOptions,
    /// Step 4: Select style/color scheme
    SelectStyle,
    /// Step 5: Preview and confirm
    Preview,
}

impl WizardStep {
    /// Get the next step
    pub fn next(&self) -> Option<WizardStep> {
        match self {
            WizardStep::SelectType => Some(WizardStep::EnterData),
            WizardStep::EnterData => Some(WizardStep::ConfigureOptions),
            WizardStep::ConfigureOptions => Some(WizardStep::SelectStyle),
            WizardStep::SelectStyle => Some(WizardStep::Preview),
            WizardStep::Preview => None,
        }
    }

    /// Get the previous step
    pub fn previous(&self) -> Option<WizardStep> {
        match self {
            WizardStep::SelectType => None,
            WizardStep::EnterData => Some(WizardStep::SelectType),
            WizardStep::ConfigureOptions => Some(WizardStep::EnterData),
            WizardStep::SelectStyle => Some(WizardStep::ConfigureOptions),
            WizardStep::Preview => Some(WizardStep::SelectStyle),
        }
    }

    /// Get the step number (1-based)
    pub fn number(&self) -> usize {
        match self {
            WizardStep::SelectType => 1,
            WizardStep::EnterData => 2,
            WizardStep::ConfigureOptions => 3,
            WizardStep::SelectStyle => 4,
            WizardStep::Preview => 5,
        }
    }

    /// Get the total number of steps
    pub fn total_steps() -> usize {
        5
    }

    /// Get the step name
    pub fn name(&self) -> &'static str {
        match self {
            WizardStep::SelectType => "Select Chart Type",
            WizardStep::EnterData => "Enter Data",
            WizardStep::ConfigureOptions => "Configure Options",
            WizardStep::SelectStyle => "Select Style",
            WizardStep::Preview => "Preview",
        }
    }

    /// Get a description of the step
    pub fn description(&self) -> &'static str {
        match self {
            WizardStep::SelectType => "Choose the type of chart you want to create",
            WizardStep::EnterData => "Enter your data or import from a spreadsheet",
            WizardStep::ConfigureOptions => "Set the chart title, legend, and axis options",
            WizardStep::SelectStyle => "Choose colors and visual style",
            WizardStep::Preview => "Review your chart before inserting",
        }
    }
}

/// Chart type category for grouping in the wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartCategory {
    /// Column and bar charts
    ColumnBar,
    /// Line and area charts
    LineArea,
    /// Pie and doughnut charts
    PieDoughnut,
    /// XY (scatter) and bubble charts
    XYScatter,
    /// Other chart types (radar, stock, etc.)
    Other,
}

impl ChartCategory {
    /// Get all categories
    pub fn all() -> Vec<ChartCategory> {
        vec![
            ChartCategory::ColumnBar,
            ChartCategory::LineArea,
            ChartCategory::PieDoughnut,
            ChartCategory::XYScatter,
            ChartCategory::Other,
        ]
    }

    /// Get the name of this category
    pub fn name(&self) -> &'static str {
        match self {
            ChartCategory::ColumnBar => "Column & Bar",
            ChartCategory::LineArea => "Line & Area",
            ChartCategory::PieDoughnut => "Pie & Doughnut",
            ChartCategory::XYScatter => "XY (Scatter) & Bubble",
            ChartCategory::Other => "Other",
        }
    }

    /// Get the chart types in this category
    pub fn chart_types(&self) -> Vec<ChartTypeOption> {
        match self {
            ChartCategory::ColumnBar => vec![
                ChartTypeOption::column(),
                ChartTypeOption::stacked_column(),
                ChartTypeOption::percent_stacked_column(),
                ChartTypeOption::bar(),
                ChartTypeOption::stacked_bar(),
            ],
            ChartCategory::LineArea => vec![
                ChartTypeOption::line(),
                ChartTypeOption::line_with_markers(),
                ChartTypeOption::smooth_line(),
                ChartTypeOption::area(),
                ChartTypeOption::stacked_area(),
            ],
            ChartCategory::PieDoughnut => vec![
                ChartTypeOption::pie(),
                ChartTypeOption::exploded_pie(),
                ChartTypeOption::doughnut(),
            ],
            ChartCategory::XYScatter => vec![
                ChartTypeOption::scatter(),
                ChartTypeOption::scatter_with_lines(),
                ChartTypeOption::bubble(),
            ],
            ChartCategory::Other => vec![
                ChartTypeOption::radar(),
                ChartTypeOption::filled_radar(),
                ChartTypeOption::stock(),
            ],
        }
    }
}

/// A chart type option for display in the wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartTypeOption {
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// The actual chart type
    pub chart_type: ChartType,
    /// Icon identifier (for UI)
    pub icon: String,
}

impl ChartTypeOption {
    /// Create a column chart option
    pub fn column() -> Self {
        Self {
            name: "Column".to_string(),
            description: "Compare values across categories".to_string(),
            chart_type: ChartType::Column {
                stacked: false,
                stacked_percent: false,
            },
            icon: "column".to_string(),
        }
    }

    /// Create a stacked column chart option
    pub fn stacked_column() -> Self {
        Self {
            name: "Stacked Column".to_string(),
            description: "Compare parts of a whole across categories".to_string(),
            chart_type: ChartType::Column {
                stacked: true,
                stacked_percent: false,
            },
            icon: "stacked-column".to_string(),
        }
    }

    /// Create a 100% stacked column chart option
    pub fn percent_stacked_column() -> Self {
        Self {
            name: "100% Stacked Column".to_string(),
            description: "Compare percentage contribution across categories".to_string(),
            chart_type: ChartType::Column {
                stacked: true,
                stacked_percent: true,
            },
            icon: "percent-stacked-column".to_string(),
        }
    }

    /// Create a bar chart option
    pub fn bar() -> Self {
        Self {
            name: "Bar".to_string(),
            description: "Compare values with horizontal bars".to_string(),
            chart_type: ChartType::Bar {
                horizontal: true,
                stacked: false,
                stacked_percent: false,
            },
            icon: "bar".to_string(),
        }
    }

    /// Create a stacked bar chart option
    pub fn stacked_bar() -> Self {
        Self {
            name: "Stacked Bar".to_string(),
            description: "Compare parts of a whole with horizontal bars".to_string(),
            chart_type: ChartType::Bar {
                horizontal: true,
                stacked: true,
                stacked_percent: false,
            },
            icon: "stacked-bar".to_string(),
        }
    }

    /// Create a line chart option
    pub fn line() -> Self {
        Self {
            name: "Line".to_string(),
            description: "Show trends over time or categories".to_string(),
            chart_type: ChartType::Line {
                smooth: false,
                markers: false,
            },
            icon: "line".to_string(),
        }
    }

    /// Create a line with markers chart option
    pub fn line_with_markers() -> Self {
        Self {
            name: "Line with Markers".to_string(),
            description: "Show trends with data point markers".to_string(),
            chart_type: ChartType::Line {
                smooth: false,
                markers: true,
            },
            icon: "line-markers".to_string(),
        }
    }

    /// Create a smooth line chart option
    pub fn smooth_line() -> Self {
        Self {
            name: "Smooth Line".to_string(),
            description: "Show trends with smooth curves".to_string(),
            chart_type: ChartType::Line {
                smooth: true,
                markers: true,
            },
            icon: "smooth-line".to_string(),
        }
    }

    /// Create an area chart option
    pub fn area() -> Self {
        Self {
            name: "Area".to_string(),
            description: "Show trends with filled areas".to_string(),
            chart_type: ChartType::Area { stacked: false },
            icon: "area".to_string(),
        }
    }

    /// Create a stacked area chart option
    pub fn stacked_area() -> Self {
        Self {
            name: "Stacked Area".to_string(),
            description: "Show cumulative trends over time".to_string(),
            chart_type: ChartType::Area { stacked: true },
            icon: "stacked-area".to_string(),
        }
    }

    /// Create a pie chart option
    pub fn pie() -> Self {
        Self {
            name: "Pie".to_string(),
            description: "Show proportions of a whole".to_string(),
            chart_type: ChartType::Pie {
                doughnut: false,
                explosion: 0.0,
            },
            icon: "pie".to_string(),
        }
    }

    /// Create an exploded pie chart option
    pub fn exploded_pie() -> Self {
        Self {
            name: "Exploded Pie".to_string(),
            description: "Show proportions with separated slices".to_string(),
            chart_type: ChartType::Pie {
                doughnut: false,
                explosion: 10.0,
            },
            icon: "exploded-pie".to_string(),
        }
    }

    /// Create a doughnut chart option
    pub fn doughnut() -> Self {
        Self {
            name: "Doughnut".to_string(),
            description: "Show proportions with a hole in the center".to_string(),
            chart_type: ChartType::Pie {
                doughnut: true,
                explosion: 0.0,
            },
            icon: "doughnut".to_string(),
        }
    }

    /// Create a scatter chart option
    pub fn scatter() -> Self {
        Self {
            name: "Scatter".to_string(),
            description: "Show relationships between two variables".to_string(),
            chart_type: ChartType::Scatter { with_lines: false },
            icon: "scatter".to_string(),
        }
    }

    /// Create a scatter with lines chart option
    pub fn scatter_with_lines() -> Self {
        Self {
            name: "Scatter with Lines".to_string(),
            description: "Show relationships with connecting lines".to_string(),
            chart_type: ChartType::Scatter { with_lines: true },
            icon: "scatter-lines".to_string(),
        }
    }

    /// Create a bubble chart option
    pub fn bubble() -> Self {
        Self {
            name: "Bubble".to_string(),
            description: "Show three dimensions of data".to_string(),
            chart_type: ChartType::Bubble,
            icon: "bubble".to_string(),
        }
    }

    /// Create a radar chart option
    pub fn radar() -> Self {
        Self {
            name: "Radar".to_string(),
            description: "Compare multiple variables".to_string(),
            chart_type: ChartType::Radar { filled: false },
            icon: "radar".to_string(),
        }
    }

    /// Create a filled radar chart option
    pub fn filled_radar() -> Self {
        Self {
            name: "Filled Radar".to_string(),
            description: "Compare multiple variables with filled areas".to_string(),
            chart_type: ChartType::Radar { filled: true },
            icon: "filled-radar".to_string(),
        }
    }

    /// Create a stock chart option
    pub fn stock() -> Self {
        Self {
            name: "Stock".to_string(),
            description: "Show stock price movements".to_string(),
            chart_type: ChartType::Stock,
            icon: "stock".to_string(),
        }
    }
}

/// Configuration options for the wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardOptions {
    /// Chart title
    pub title: Option<String>,
    /// Whether to show legend
    pub show_legend: bool,
    /// Legend position
    pub legend_position: LegendPosition,
    /// Whether to show data labels
    pub show_data_labels: bool,
    /// Category axis title
    pub category_axis_title: Option<String>,
    /// Value axis title
    pub value_axis_title: Option<String>,
    /// Show gridlines
    pub show_gridlines: bool,
}

impl Default for WizardOptions {
    fn default() -> Self {
        Self {
            title: None,
            show_legend: true,
            legend_position: LegendPosition::Right,
            show_data_labels: false,
            category_axis_title: None,
            value_axis_title: None,
            show_gridlines: true,
        }
    }
}

/// The chart insertion wizard state
#[derive(Debug, Clone)]
pub struct ChartWizard {
    /// Current step
    current_step: WizardStep,
    /// Selected chart type category
    selected_category: Option<ChartCategory>,
    /// Selected chart type
    selected_type: Option<ChartType>,
    /// Chart data
    data: ChartData,
    /// Configuration options
    options: WizardOptions,
    /// Selected style preset
    style_preset: ChartStylePreset,
    /// Selected color scheme (overrides preset if set)
    color_scheme: Option<ColorScheme>,
    /// Generated chart ID
    chart_id: String,
    /// Preview dimensions
    preview_width: f64,
    preview_height: f64,
}

impl Default for ChartWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartWizard {
    /// Create a new chart wizard
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::SelectType,
            selected_category: None,
            selected_type: None,
            data: ChartData::default(),
            options: WizardOptions::default(),
            style_preset: ChartStylePreset::default(),
            color_scheme: None,
            chart_id: format!("chart_{}", uuid_simple()),
            preview_width: 400.0,
            preview_height: 300.0,
        }
    }

    /// Create a wizard with a specific chart ID
    pub fn with_id(id: impl Into<String>) -> Self {
        let mut wizard = Self::new();
        wizard.chart_id = id.into();
        wizard
    }

    /// Get the current step
    pub fn current_step(&self) -> WizardStep {
        self.current_step
    }

    /// Go to the next step
    pub fn next_step(&mut self) -> ChartResult<bool> {
        // Validate current step before proceeding
        self.validate_current_step()?;

        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Go to the previous step
    pub fn previous_step(&mut self) -> bool {
        if let Some(prev) = self.current_step.previous() {
            self.current_step = prev;
            true
        } else {
            false
        }
    }

    /// Go to a specific step (if valid)
    pub fn go_to_step(&mut self, step: WizardStep) -> ChartResult<()> {
        // Can only go to steps we've already passed or the current step
        if step.number() <= self.current_step.number() {
            self.current_step = step;
            Ok(())
        } else {
            // Need to validate all steps up to the target
            let mut temp_step = self.current_step;
            while temp_step.number() < step.number() {
                self.current_step = temp_step;
                self.validate_current_step()?;
                if let Some(next) = temp_step.next() {
                    temp_step = next;
                } else {
                    break;
                }
            }
            self.current_step = step;
            Ok(())
        }
    }

    /// Validate the current step
    fn validate_current_step(&self) -> ChartResult<()> {
        match self.current_step {
            WizardStep::SelectType => {
                if self.selected_type.is_none() {
                    return Err(ChartError::InvalidData(
                        "Please select a chart type".to_string(),
                    ));
                }
            }
            WizardStep::EnterData => {
                if self.data.series.is_empty() {
                    return Err(ChartError::InvalidData(
                        "Please enter at least one data series".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Check if the wizard can be finished
    pub fn can_finish(&self) -> bool {
        self.selected_type.is_some() && !self.data.series.is_empty()
    }

    /// Get the progress percentage (0-100)
    pub fn progress_percent(&self) -> u8 {
        ((self.current_step.number() as f64 / WizardStep::total_steps() as f64) * 100.0) as u8
    }

    // === Type Selection ===

    /// Set the selected category
    pub fn select_category(&mut self, category: ChartCategory) {
        self.selected_category = Some(category);
    }

    /// Get the selected category
    pub fn selected_category(&self) -> Option<ChartCategory> {
        self.selected_category
    }

    /// Set the selected chart type
    pub fn select_chart_type(&mut self, chart_type: ChartType) {
        self.selected_type = Some(chart_type);
    }

    /// Get the selected chart type
    pub fn selected_chart_type(&self) -> Option<&ChartType> {
        self.selected_type.as_ref()
    }

    // === Data Entry ===

    /// Set the chart data
    pub fn set_data(&mut self, data: ChartData) {
        self.data = data;
    }

    /// Get the chart data
    pub fn data(&self) -> &ChartData {
        &self.data
    }

    /// Get mutable access to chart data
    pub fn data_mut(&mut self) -> &mut ChartData {
        &mut self.data
    }

    /// Add a data series
    pub fn add_series(&mut self, series: DataSeries) {
        self.data.series.push(series);
    }

    /// Set categories
    pub fn set_categories(&mut self, categories: Vec<String>) {
        self.data.categories = categories;
    }

    /// Set sample data for the selected chart type
    pub fn set_sample_data(&mut self) {
        self.data = Self::sample_data_for_type(
            self.selected_type.as_ref().unwrap_or(&ChartType::default()),
        );
    }

    /// Generate sample data appropriate for a chart type
    pub fn sample_data_for_type(chart_type: &ChartType) -> ChartData {
        match chart_type {
            ChartType::Pie { .. } => {
                let mut data = ChartData::new(vec![
                    "Product A".to_string(),
                    "Product B".to_string(),
                    "Product C".to_string(),
                    "Product D".to_string(),
                ]);
                data.series
                    .push(DataSeries::new("Sales", vec![30.0, 25.0, 20.0, 25.0]));
                data
            }
            ChartType::Line { .. } | ChartType::Area { .. } => {
                let mut data = ChartData::new(vec![
                    "Jan".to_string(),
                    "Feb".to_string(),
                    "Mar".to_string(),
                    "Apr".to_string(),
                    "May".to_string(),
                    "Jun".to_string(),
                ]);
                data.series.push(DataSeries::new(
                    "Series 1",
                    vec![10.0, 15.0, 13.0, 17.0, 20.0, 25.0],
                ));
                data.series.push(DataSeries::new(
                    "Series 2",
                    vec![8.0, 12.0, 15.0, 14.0, 18.0, 22.0],
                ));
                data
            }
            ChartType::Scatter { .. } | ChartType::Bubble => {
                let mut data = ChartData::new(vec![]);
                data.series.push(DataSeries::new(
                    "Data Points",
                    vec![5.0, 10.0, 15.0, 20.0, 25.0, 30.0],
                ));
                data
            }
            ChartType::Radar { .. } => {
                let mut data = ChartData::new(vec![
                    "Speed".to_string(),
                    "Power".to_string(),
                    "Accuracy".to_string(),
                    "Endurance".to_string(),
                    "Technique".to_string(),
                ]);
                data.series
                    .push(DataSeries::new("Player A", vec![80.0, 70.0, 90.0, 60.0, 85.0]));
                data.series
                    .push(DataSeries::new("Player B", vec![75.0, 85.0, 70.0, 80.0, 75.0]));
                data
            }
            _ => {
                // Default sample data for column/bar charts
                let mut data = ChartData::new(vec![
                    "Q1".to_string(),
                    "Q2".to_string(),
                    "Q3".to_string(),
                    "Q4".to_string(),
                ]);
                data.series.push(DataSeries::new(
                    "Product A",
                    vec![100.0, 120.0, 140.0, 130.0],
                ));
                data.series
                    .push(DataSeries::new("Product B", vec![80.0, 95.0, 110.0, 105.0]));
                data.series
                    .push(DataSeries::new("Product C", vec![60.0, 70.0, 85.0, 90.0]));
                data
            }
        }
    }

    // === Options ===

    /// Set wizard options
    pub fn set_options(&mut self, options: WizardOptions) {
        self.options = options;
    }

    /// Get wizard options
    pub fn options(&self) -> &WizardOptions {
        &self.options
    }

    /// Get mutable access to options
    pub fn options_mut(&mut self) -> &mut WizardOptions {
        &mut self.options
    }

    /// Set the chart title
    pub fn set_title(&mut self, title: Option<String>) {
        self.options.title = title;
    }

    /// Set legend visibility
    pub fn set_show_legend(&mut self, show: bool) {
        self.options.show_legend = show;
    }

    /// Set legend position
    pub fn set_legend_position(&mut self, position: LegendPosition) {
        self.options.legend_position = position;
    }

    // === Styling ===

    /// Set the style preset
    pub fn set_style_preset(&mut self, preset: ChartStylePreset) {
        self.style_preset = preset;
    }

    /// Get the style preset
    pub fn style_preset(&self) -> ChartStylePreset {
        self.style_preset
    }

    /// Set the color scheme (overrides preset colors)
    pub fn set_color_scheme(&mut self, scheme: Option<ColorScheme>) {
        self.color_scheme = scheme;
    }

    /// Get the color scheme
    pub fn color_scheme(&self) -> Option<ColorScheme> {
        self.color_scheme
    }

    // === Preview ===

    /// Set preview dimensions
    pub fn set_preview_size(&mut self, width: f64, height: f64) {
        self.preview_width = width;
        self.preview_height = height;
    }

    /// Generate a preview chart
    pub fn generate_preview(&self) -> Option<Chart> {
        let chart_type = self.selected_type.as_ref()?;
        Some(self.build_chart_internal(chart_type.clone()))
    }

    /// Render a preview
    pub fn render_preview(&self) -> Option<RenderedChart> {
        let chart = self.generate_preview()?;
        let calculator = ChartLayoutCalculator::default();
        let layout = calculator.calculate(&chart, self.preview_width, self.preview_height);
        let renderer = ChartRenderer::default();
        Some(renderer.render(&chart, &layout))
    }

    /// Render preview to SVG
    pub fn render_preview_svg(&self) -> Option<String> {
        let chart = self.generate_preview()?;
        let calculator = ChartLayoutCalculator::default();
        let layout = calculator.calculate(&chart, self.preview_width, self.preview_height);
        let renderer = ChartRenderer::default();
        Some(renderer.render_svg(&chart, &layout))
    }

    // === Finish ===

    /// Build the final chart
    pub fn finish(&self) -> ChartResult<Chart> {
        let chart_type = self
            .selected_type
            .clone()
            .ok_or_else(|| ChartError::InvalidData("No chart type selected".to_string()))?;

        if self.data.series.is_empty() {
            return Err(ChartError::InvalidData("No data provided".to_string()));
        }

        Ok(self.build_chart_internal(chart_type))
    }

    /// Build the chart from current settings
    fn build_chart_internal(&self, chart_type: ChartType) -> Chart {
        let mut chart = Chart::new(&self.chart_id, chart_type);

        // Set data
        chart.data = self.data.clone();

        // Set style
        let mut style = self.style_preset.to_style();
        if let Some(scheme) = self.color_scheme {
            style.colors = scheme.colors();
        }
        chart.style = style;

        // Set title
        if let Some(ref title) = self.options.title {
            chart.title = Some(ChartTitle {
                text: title.clone(),
                position: TitlePosition::Top,
            });
        }

        // Set legend
        if self.options.show_legend {
            chart.legend = Some(Legend {
                position: self.options.legend_position,
                visible: true,
            });
        }

        // Set axes
        let mut category_axis = Axis::default();
        category_axis.major_gridlines = self.options.show_gridlines;
        if let Some(ref title) = self.options.category_axis_title {
            category_axis.title = Some(title.clone());
        }
        chart.axes.category_axis = Some(category_axis);

        let mut value_axis = Axis::default();
        value_axis.major_gridlines = self.options.show_gridlines;
        if let Some(ref title) = self.options.value_axis_title {
            value_axis.title = Some(title.clone());
        }
        chart.axes.value_axis = Some(value_axis);

        chart
    }

    /// Reset the wizard to its initial state
    pub fn reset(&mut self) {
        *self = Self::with_id(&self.chart_id);
    }
}

/// Generate a simple unique ID
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_step_navigation() {
        assert_eq!(WizardStep::SelectType.next(), Some(WizardStep::EnterData));
        assert_eq!(WizardStep::Preview.next(), None);
        assert_eq!(WizardStep::SelectType.previous(), None);
        assert_eq!(
            WizardStep::EnterData.previous(),
            Some(WizardStep::SelectType)
        );
    }

    #[test]
    fn test_wizard_step_number() {
        assert_eq!(WizardStep::SelectType.number(), 1);
        assert_eq!(WizardStep::Preview.number(), 5);
        assert_eq!(WizardStep::total_steps(), 5);
    }

    #[test]
    fn test_chart_category_types() {
        let category = ChartCategory::ColumnBar;
        let types = category.chart_types();

        assert!(!types.is_empty());
        assert!(types.iter().any(|t| t.name == "Column"));
    }

    #[test]
    fn test_wizard_new() {
        let wizard = ChartWizard::new();

        assert_eq!(wizard.current_step(), WizardStep::SelectType);
        assert!(wizard.selected_type.is_none());
        assert!(wizard.data.series.is_empty());
    }

    #[test]
    fn test_wizard_select_type() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());

        assert!(wizard.selected_chart_type().is_some());
    }

    #[test]
    fn test_wizard_next_step_validation() {
        let mut wizard = ChartWizard::new();

        // Can't proceed without selecting a type
        assert!(wizard.next_step().is_err());

        // Select a type and try again
        wizard.select_chart_type(ChartType::default());
        assert!(wizard.next_step().is_ok());
        assert_eq!(wizard.current_step(), WizardStep::EnterData);
    }

    #[test]
    fn test_wizard_data_entry() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());

        wizard.set_categories(vec!["A".to_string(), "B".to_string()]);
        wizard.add_series(DataSeries::new("Test", vec![1.0, 2.0]));

        assert_eq!(wizard.data().categories.len(), 2);
        assert_eq!(wizard.data().series.len(), 1);
    }

    #[test]
    fn test_wizard_sample_data() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::Pie {
            doughnut: false,
            explosion: 0.0,
        });
        wizard.set_sample_data();

        assert!(!wizard.data().series.is_empty());
        assert!(!wizard.data().categories.is_empty());
    }

    #[test]
    fn test_wizard_options() {
        let mut wizard = ChartWizard::new();
        wizard.set_title(Some("My Chart".to_string()));
        wizard.set_show_legend(false);
        wizard.set_legend_position(LegendPosition::Bottom);

        assert_eq!(wizard.options().title, Some("My Chart".to_string()));
        assert!(!wizard.options().show_legend);
        assert_eq!(wizard.options().legend_position, LegendPosition::Bottom);
    }

    #[test]
    fn test_wizard_styling() {
        let mut wizard = ChartWizard::new();
        wizard.set_style_preset(ChartStylePreset::Dark);
        wizard.set_color_scheme(Some(ColorScheme::Ocean));

        assert_eq!(wizard.style_preset(), ChartStylePreset::Dark);
        assert_eq!(wizard.color_scheme(), Some(ColorScheme::Ocean));
    }

    #[test]
    fn test_wizard_finish() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());
        wizard.add_series(DataSeries::new("Test", vec![1.0, 2.0, 3.0]));
        wizard.set_title(Some("Test Chart".to_string()));

        let chart = wizard.finish().unwrap();

        assert!(!chart.data.series.is_empty());
        assert!(chart.title.is_some());
    }

    #[test]
    fn test_wizard_finish_without_type() {
        let wizard = ChartWizard::new();
        assert!(wizard.finish().is_err());
    }

    #[test]
    fn test_wizard_finish_without_data() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());

        assert!(wizard.finish().is_err());
    }

    #[test]
    fn test_wizard_preview() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());
        wizard.add_series(DataSeries::new("Test", vec![1.0, 2.0, 3.0]));

        let preview = wizard.generate_preview();
        assert!(preview.is_some());

        let svg = wizard.render_preview_svg();
        assert!(svg.is_some());
        assert!(svg.unwrap().contains("<svg"));
    }

    #[test]
    fn test_wizard_can_finish() {
        let mut wizard = ChartWizard::new();
        assert!(!wizard.can_finish());

        wizard.select_chart_type(ChartType::default());
        assert!(!wizard.can_finish());

        wizard.add_series(DataSeries::new("Test", vec![1.0]));
        assert!(wizard.can_finish());
    }

    #[test]
    fn test_wizard_progress() {
        let mut wizard = ChartWizard::new();
        assert_eq!(wizard.progress_percent(), 20); // Step 1 of 5

        wizard.select_chart_type(ChartType::default());
        wizard.next_step().unwrap();
        assert_eq!(wizard.progress_percent(), 40); // Step 2 of 5
    }

    #[test]
    fn test_wizard_reset() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());
        wizard.add_series(DataSeries::new("Test", vec![1.0]));

        let original_id = wizard.chart_id.clone();
        wizard.reset();

        assert_eq!(wizard.chart_id, original_id);
        assert!(wizard.selected_type.is_none());
        assert!(wizard.data.series.is_empty());
    }

    #[test]
    fn test_chart_type_options() {
        let column = ChartTypeOption::column();
        assert_eq!(column.name, "Column");
        assert!(matches!(column.chart_type, ChartType::Column { .. }));

        let pie = ChartTypeOption::pie();
        assert_eq!(pie.name, "Pie");
        assert!(matches!(pie.chart_type, ChartType::Pie { .. }));
    }

    #[test]
    fn test_sample_data_for_different_types() {
        let pie_data = ChartWizard::sample_data_for_type(&ChartType::Pie {
            doughnut: false,
            explosion: 0.0,
        });
        assert!(!pie_data.series.is_empty());

        let line_data = ChartWizard::sample_data_for_type(&ChartType::Line {
            smooth: false,
            markers: true,
        });
        assert!(line_data.series.len() >= 2);

        let radar_data = ChartWizard::sample_data_for_type(&ChartType::Radar { filled: false });
        assert!(radar_data.categories.len() >= 5);
    }

    #[test]
    fn test_wizard_with_id() {
        let wizard = ChartWizard::with_id("custom_id");
        assert_eq!(wizard.chart_id, "custom_id");
    }

    #[test]
    fn test_previous_step() {
        let mut wizard = ChartWizard::new();
        wizard.select_chart_type(ChartType::default());
        wizard.next_step().unwrap();

        assert_eq!(wizard.current_step(), WizardStep::EnterData);

        wizard.previous_step();
        assert_eq!(wizard.current_step(), WizardStep::SelectType);

        // Can't go before first step
        assert!(!wizard.previous_step());
    }
}
