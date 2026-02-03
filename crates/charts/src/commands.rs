//! Chart editing commands
//!
//! This module provides command types for manipulating charts,
//! following the command pattern for undo/redo support.

use crate::error::{ChartError, ChartResult};
use crate::model::*;
use crate::styles::{ChartStylePreset, ColorScheme};
use serde::{Deserialize, Serialize};

/// A command that can be applied to a chart
pub trait ChartCommand {
    /// Execute the command on the given chart
    fn execute(&self, chart: &mut Chart) -> ChartResult<()>;

    /// Get a description of the command for display
    fn description(&self) -> String;
}

/// Command to insert a new chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertChart {
    /// The chart ID to assign
    pub chart_id: String,
    /// The type of chart to create
    pub chart_type: ChartType,
    /// Optional initial title
    pub title: Option<String>,
    /// Optional initial data
    pub initial_data: Option<ChartData>,
    /// Optional initial style preset
    pub style_preset: Option<ChartStylePreset>,
}

impl InsertChart {
    /// Create a new InsertChart command
    pub fn new(chart_id: impl Into<String>, chart_type: ChartType) -> Self {
        Self {
            chart_id: chart_id.into(),
            chart_type,
            title: None,
            initial_data: None,
            style_preset: None,
        }
    }

    /// Set the initial title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set initial data
    pub fn with_data(mut self, data: ChartData) -> Self {
        self.initial_data = Some(data);
        self
    }

    /// Set initial style preset
    pub fn with_style_preset(mut self, preset: ChartStylePreset) -> Self {
        self.style_preset = Some(preset);
        self
    }

    /// Create the chart from this command
    pub fn create_chart(&self) -> Chart {
        let mut chart = Chart::new(&self.chart_id, self.chart_type.clone());

        if let Some(ref title) = self.title {
            chart = chart.with_title(title.clone());
        }

        if let Some(ref data) = self.initial_data {
            chart.data = data.clone();
        }

        if let Some(ref preset) = self.style_preset {
            chart.style = preset.to_style();
        }

        chart
    }
}

impl ChartCommand for InsertChart {
    fn execute(&self, chart: &mut Chart) -> ChartResult<()> {
        // When executed on an existing chart, we replace it with the new configuration
        *chart = self.create_chart();
        Ok(())
    }

    fn description(&self) -> String {
        format!("Insert {:?} chart", self.chart_type)
    }
}

/// Command to update chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChartData {
    /// The type of data update
    pub update_type: DataUpdateType,
}

/// Types of data updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataUpdateType {
    /// Replace all data
    ReplaceAll { data: ChartData },
    /// Add a new series
    AddSeries { series: DataSeries },
    /// Remove a series by index
    RemoveSeries { index: usize },
    /// Update a specific series
    UpdateSeries { index: usize, series: DataSeries },
    /// Update a single value
    UpdateValue {
        series_index: usize,
        value_index: usize,
        value: f64,
    },
    /// Add a category
    AddCategory { name: String },
    /// Remove a category by index
    RemoveCategory { index: usize },
    /// Rename a category
    RenameCategory { index: usize, name: String },
    /// Set all categories
    SetCategories { categories: Vec<String> },
    /// Update series name
    UpdateSeriesName { index: usize, name: String },
    /// Append values to a series
    AppendValues { series_index: usize, values: Vec<f64> },
}

impl UpdateChartData {
    /// Create a command to replace all data
    pub fn replace_all(data: ChartData) -> Self {
        Self {
            update_type: DataUpdateType::ReplaceAll { data },
        }
    }

    /// Create a command to add a series
    pub fn add_series(series: DataSeries) -> Self {
        Self {
            update_type: DataUpdateType::AddSeries { series },
        }
    }

    /// Create a command to remove a series
    pub fn remove_series(index: usize) -> Self {
        Self {
            update_type: DataUpdateType::RemoveSeries { index },
        }
    }

    /// Create a command to update a series
    pub fn update_series(index: usize, series: DataSeries) -> Self {
        Self {
            update_type: DataUpdateType::UpdateSeries { index, series },
        }
    }

    /// Create a command to update a single value
    pub fn update_value(series_index: usize, value_index: usize, value: f64) -> Self {
        Self {
            update_type: DataUpdateType::UpdateValue {
                series_index,
                value_index,
                value,
            },
        }
    }

    /// Create a command to add a category
    pub fn add_category(name: impl Into<String>) -> Self {
        Self {
            update_type: DataUpdateType::AddCategory { name: name.into() },
        }
    }

    /// Create a command to remove a category
    pub fn remove_category(index: usize) -> Self {
        Self {
            update_type: DataUpdateType::RemoveCategory { index },
        }
    }

    /// Create a command to set all categories
    pub fn set_categories(categories: Vec<String>) -> Self {
        Self {
            update_type: DataUpdateType::SetCategories { categories },
        }
    }
}

impl ChartCommand for UpdateChartData {
    fn execute(&self, chart: &mut Chart) -> ChartResult<()> {
        match &self.update_type {
            DataUpdateType::ReplaceAll { data } => {
                chart.data = data.clone();
            }
            DataUpdateType::AddSeries { series } => {
                chart.data.series.push(series.clone());
            }
            DataUpdateType::RemoveSeries { index } => {
                if *index >= chart.data.series.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Series index {} out of bounds",
                        index
                    )));
                }
                chart.data.series.remove(*index);
            }
            DataUpdateType::UpdateSeries { index, series } => {
                if *index >= chart.data.series.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Series index {} out of bounds",
                        index
                    )));
                }
                chart.data.series[*index] = series.clone();
            }
            DataUpdateType::UpdateValue {
                series_index,
                value_index,
                value,
            } => {
                if *series_index >= chart.data.series.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Series index {} out of bounds",
                        series_index
                    )));
                }
                let series = &mut chart.data.series[*series_index];
                if *value_index >= series.values.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Value index {} out of bounds",
                        value_index
                    )));
                }
                series.values[*value_index] = *value;
            }
            DataUpdateType::AddCategory { name } => {
                chart.data.categories.push(name.clone());
            }
            DataUpdateType::RemoveCategory { index } => {
                if *index >= chart.data.categories.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Category index {} out of bounds",
                        index
                    )));
                }
                chart.data.categories.remove(*index);
                // Also remove the corresponding value from each series
                for series in &mut chart.data.series {
                    if *index < series.values.len() {
                        series.values.remove(*index);
                    }
                }
            }
            DataUpdateType::RenameCategory { index, name } => {
                if *index >= chart.data.categories.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Category index {} out of bounds",
                        index
                    )));
                }
                chart.data.categories[*index] = name.clone();
            }
            DataUpdateType::SetCategories { categories } => {
                chart.data.categories = categories.clone();
            }
            DataUpdateType::UpdateSeriesName { index, name } => {
                if *index >= chart.data.series.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Series index {} out of bounds",
                        index
                    )));
                }
                chart.data.series[*index].name = name.clone();
            }
            DataUpdateType::AppendValues {
                series_index,
                values,
            } => {
                if *series_index >= chart.data.series.len() {
                    return Err(ChartError::InvalidData(format!(
                        "Series index {} out of bounds",
                        series_index
                    )));
                }
                chart.data.series[*series_index].values.extend(values);
            }
        }
        Ok(())
    }

    fn description(&self) -> String {
        match &self.update_type {
            DataUpdateType::ReplaceAll { .. } => "Replace chart data".to_string(),
            DataUpdateType::AddSeries { series } => {
                format!("Add series '{}'", series.name)
            }
            DataUpdateType::RemoveSeries { index } => format!("Remove series {}", index),
            DataUpdateType::UpdateSeries { index, series } => {
                format!("Update series {} to '{}'", index, series.name)
            }
            DataUpdateType::UpdateValue {
                series_index,
                value_index,
                value,
            } => format!(
                "Update value at [{}, {}] to {}",
                series_index, value_index, value
            ),
            DataUpdateType::AddCategory { name } => format!("Add category '{}'", name),
            DataUpdateType::RemoveCategory { index } => format!("Remove category {}", index),
            DataUpdateType::RenameCategory { index, name } => {
                format!("Rename category {} to '{}'", index, name)
            }
            DataUpdateType::SetCategories { .. } => "Set categories".to_string(),
            DataUpdateType::UpdateSeriesName { index, name } => {
                format!("Rename series {} to '{}'", index, name)
            }
            DataUpdateType::AppendValues { series_index, .. } => {
                format!("Append values to series {}", series_index)
            }
        }
    }
}

/// Command to change the chart type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeChartType {
    /// The new chart type
    pub new_type: ChartType,
    /// Whether to preserve compatible settings
    pub preserve_settings: bool,
}

impl ChangeChartType {
    /// Create a new ChangeChartType command
    pub fn new(new_type: ChartType) -> Self {
        Self {
            new_type,
            preserve_settings: true,
        }
    }

    /// Set whether to preserve compatible settings
    pub fn preserve_settings(mut self, preserve: bool) -> Self {
        self.preserve_settings = preserve;
        self
    }
}

impl ChartCommand for ChangeChartType {
    fn execute(&self, chart: &mut Chart) -> ChartResult<()> {
        chart.chart_type = self.new_type.clone();

        // Clear original XML since we're changing the type
        chart.original_xml = None;

        // Adjust legend position for pie charts
        if matches!(self.new_type, ChartType::Pie { .. }) {
            if let Some(ref mut legend) = chart.legend {
                // Pie charts often have legend on the right
                if legend.position == LegendPosition::Bottom {
                    legend.position = LegendPosition::Right;
                }
            }
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!("Change chart type to {:?}", self.new_type)
    }
}

/// Command to update chart styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChartStyle {
    /// The type of style update
    pub update_type: StyleUpdateType,
}

/// Types of style updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleUpdateType {
    /// Apply a style preset
    ApplyPreset { preset: ChartStylePreset },
    /// Apply a color scheme
    ApplyColorScheme { scheme: ColorScheme },
    /// Set the background color
    SetBackground { color: Option<Color> },
    /// Set the plot area background
    SetPlotAreaBackground { color: Option<Color> },
    /// Set a specific series color
    SetSeriesColor { index: usize, color: Color },
    /// Set the border style
    SetBorder { border: Option<BorderStyle> },
    /// Update the title
    UpdateTitle { title: Option<ChartTitle> },
    /// Update the legend
    UpdateLegend { legend: Option<Legend> },
    /// Update axis settings
    UpdateCategoryAxis { axis: Option<Axis> },
    /// Update value axis settings
    UpdateValueAxis { axis: Option<Axis> },
    /// Set all colors at once
    SetColors { colors: Vec<Color> },
}

impl UpdateChartStyle {
    /// Create a command to apply a style preset
    pub fn apply_preset(preset: ChartStylePreset) -> Self {
        Self {
            update_type: StyleUpdateType::ApplyPreset { preset },
        }
    }

    /// Create a command to apply a color scheme
    pub fn apply_color_scheme(scheme: ColorScheme) -> Self {
        Self {
            update_type: StyleUpdateType::ApplyColorScheme { scheme },
        }
    }

    /// Create a command to set the background color
    pub fn set_background(color: Option<Color>) -> Self {
        Self {
            update_type: StyleUpdateType::SetBackground { color },
        }
    }

    /// Create a command to set a series color
    pub fn set_series_color(index: usize, color: Color) -> Self {
        Self {
            update_type: StyleUpdateType::SetSeriesColor { index, color },
        }
    }

    /// Create a command to update the title
    pub fn update_title(title: Option<ChartTitle>) -> Self {
        Self {
            update_type: StyleUpdateType::UpdateTitle { title },
        }
    }

    /// Create a command to update the legend
    pub fn update_legend(legend: Option<Legend>) -> Self {
        Self {
            update_type: StyleUpdateType::UpdateLegend { legend },
        }
    }

    /// Create a command to set the border
    pub fn set_border(border: Option<BorderStyle>) -> Self {
        Self {
            update_type: StyleUpdateType::SetBorder { border },
        }
    }
}

impl ChartCommand for UpdateChartStyle {
    fn execute(&self, chart: &mut Chart) -> ChartResult<()> {
        match &self.update_type {
            StyleUpdateType::ApplyPreset { preset } => {
                chart.style = preset.to_style();
            }
            StyleUpdateType::ApplyColorScheme { scheme } => {
                chart.style.colors = scheme.colors();
            }
            StyleUpdateType::SetBackground { color } => {
                chart.style.background = *color;
            }
            StyleUpdateType::SetPlotAreaBackground { color } => {
                chart.style.plot_area_background = *color;
            }
            StyleUpdateType::SetSeriesColor { index, color } => {
                // Extend colors array if needed
                while chart.style.colors.len() <= *index {
                    chart
                        .style
                        .colors
                        .push(chart.style.colors.last().copied().unwrap_or(Color::GRAY));
                }
                chart.style.colors[*index] = *color;

                // Also update the series-specific color if it exists
                if *index < chart.data.series.len() {
                    chart.data.series[*index].color = Some(*color);
                }
            }
            StyleUpdateType::SetBorder { border } => {
                chart.style.border = border.clone();
            }
            StyleUpdateType::UpdateTitle { title } => {
                chart.title = title.clone();
            }
            StyleUpdateType::UpdateLegend { legend } => {
                chart.legend = legend.clone();
            }
            StyleUpdateType::UpdateCategoryAxis { axis } => {
                chart.axes.category_axis = axis.clone();
            }
            StyleUpdateType::UpdateValueAxis { axis } => {
                chart.axes.value_axis = axis.clone();
            }
            StyleUpdateType::SetColors { colors } => {
                chart.style.colors = colors.clone();
            }
        }
        Ok(())
    }

    fn description(&self) -> String {
        match &self.update_type {
            StyleUpdateType::ApplyPreset { preset } => {
                format!("Apply style preset {:?}", preset)
            }
            StyleUpdateType::ApplyColorScheme { scheme } => {
                format!("Apply color scheme {:?}", scheme)
            }
            StyleUpdateType::SetBackground { .. } => "Set background color".to_string(),
            StyleUpdateType::SetPlotAreaBackground { .. } => {
                "Set plot area background".to_string()
            }
            StyleUpdateType::SetSeriesColor { index, .. } => {
                format!("Set series {} color", index)
            }
            StyleUpdateType::SetBorder { .. } => "Set border style".to_string(),
            StyleUpdateType::UpdateTitle { .. } => "Update chart title".to_string(),
            StyleUpdateType::UpdateLegend { .. } => "Update legend".to_string(),
            StyleUpdateType::UpdateCategoryAxis { .. } => "Update category axis".to_string(),
            StyleUpdateType::UpdateValueAxis { .. } => "Update value axis".to_string(),
            StyleUpdateType::SetColors { .. } => "Set color palette".to_string(),
        }
    }
}

/// A composite command that groups multiple commands
pub struct CompositeCommand {
    /// The commands to execute
    pub commands: Vec<Box<dyn ChartCommandClone>>,
    /// Description for the composite command
    pub description: String,
}

impl std::fmt::Debug for CompositeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeCommand")
            .field("commands_count", &self.commands.len())
            .field("description", &self.description)
            .finish()
    }
}

/// Helper trait for cloning boxed commands
pub trait ChartCommandClone: ChartCommand {
    fn clone_box(&self) -> Box<dyn ChartCommandClone>;
}

impl<T: ChartCommand + Clone + 'static> ChartCommandClone for T {
    fn clone_box(&self) -> Box<dyn ChartCommandClone> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ChartCommandClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl CompositeCommand {
    /// Create a new composite command
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            commands: Vec::new(),
            description: description.into(),
        }
    }

    /// Add a command to the composite
    pub fn add<C: ChartCommand + Clone + 'static>(mut self, command: C) -> Self {
        self.commands.push(Box::new(command));
        self
    }
}

impl ChartCommand for CompositeCommand {
    fn execute(&self, chart: &mut Chart) -> ChartResult<()> {
        for command in &self.commands {
            command.execute(chart)?;
        }
        Ok(())
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_chart_command() {
        let cmd = InsertChart::new("chart1", ChartType::default()).with_title("My Chart");

        let chart = cmd.create_chart();

        assert_eq!(chart.id, "chart1");
        assert!(chart.title.is_some());
        assert_eq!(chart.title.as_ref().unwrap().text, "My Chart");
    }

    #[test]
    fn test_insert_chart_with_data() {
        let mut data = ChartData::new(vec!["A".to_string(), "B".to_string()]);
        data.series
            .push(DataSeries::new("Series 1", vec![10.0, 20.0]));

        let cmd = InsertChart::new("chart1", ChartType::default()).with_data(data);

        let chart = cmd.create_chart();

        assert_eq!(chart.data.categories.len(), 2);
        assert_eq!(chart.data.series.len(), 1);
    }

    #[test]
    fn test_update_chart_data_add_series() {
        let mut chart = Chart::new("test", ChartType::default());
        let cmd = UpdateChartData::add_series(DataSeries::new("New Series", vec![1.0, 2.0, 3.0]));

        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.series.len(), 1);
        assert_eq!(chart.data.series[0].name, "New Series");
    }

    #[test]
    fn test_update_chart_data_remove_series() {
        let mut chart = Chart::new("test", ChartType::default());
        chart
            .data
            .series
            .push(DataSeries::new("Series 1", vec![1.0]));
        chart
            .data
            .series
            .push(DataSeries::new("Series 2", vec![2.0]));

        let cmd = UpdateChartData::remove_series(0);
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.series.len(), 1);
        assert_eq!(chart.data.series[0].name, "Series 2");
    }

    #[test]
    fn test_update_chart_data_remove_series_out_of_bounds() {
        let mut chart = Chart::new("test", ChartType::default());

        let cmd = UpdateChartData::remove_series(0);
        let result = cmd.execute(&mut chart);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_chart_data_update_value() {
        let mut chart = Chart::new("test", ChartType::default());
        chart
            .data
            .series
            .push(DataSeries::new("Series 1", vec![1.0, 2.0, 3.0]));

        let cmd = UpdateChartData::update_value(0, 1, 99.0);
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.series[0].values[1], 99.0);
    }

    #[test]
    fn test_update_chart_data_add_category() {
        let mut chart = Chart::new("test", ChartType::default());
        chart.data.categories = vec!["A".to_string()];

        let cmd = UpdateChartData::add_category("B");
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.categories.len(), 2);
        assert_eq!(chart.data.categories[1], "B");
    }

    #[test]
    fn test_update_chart_data_remove_category() {
        let mut chart = Chart::new("test", ChartType::default());
        chart.data.categories = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        chart
            .data
            .series
            .push(DataSeries::new("Series 1", vec![1.0, 2.0, 3.0]));

        let cmd = UpdateChartData::remove_category(1);
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.categories, vec!["A", "C"]);
        assert_eq!(chart.data.series[0].values, vec![1.0, 3.0]);
    }

    #[test]
    fn test_change_chart_type() {
        let mut chart = Chart::new("test", ChartType::default());

        let cmd = ChangeChartType::new(ChartType::Line {
            smooth: true,
            markers: true,
        });
        cmd.execute(&mut chart).unwrap();

        assert!(matches!(
            chart.chart_type,
            ChartType::Line {
                smooth: true,
                markers: true
            }
        ));
    }

    #[test]
    fn test_update_chart_style_background() {
        let mut chart = Chart::new("test", ChartType::default());

        let cmd = UpdateChartStyle::set_background(Some(Color::rgb(240, 240, 240)));
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.style.background, Some(Color::rgb(240, 240, 240)));
    }

    #[test]
    fn test_update_chart_style_series_color() {
        let mut chart = Chart::new("test", ChartType::default());
        chart
            .data
            .series
            .push(DataSeries::new("Series 1", vec![1.0]));

        let cmd = UpdateChartStyle::set_series_color(0, Color::RED);
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.style.colors[0], Color::RED);
        assert_eq!(chart.data.series[0].color, Some(Color::RED));
    }

    #[test]
    fn test_update_chart_style_title() {
        let mut chart = Chart::new("test", ChartType::default());

        let cmd = UpdateChartStyle::update_title(Some(ChartTitle {
            text: "New Title".to_string(),
            position: TitlePosition::Top,
        }));
        cmd.execute(&mut chart).unwrap();

        assert!(chart.title.is_some());
        assert_eq!(chart.title.as_ref().unwrap().text, "New Title");
    }

    #[test]
    fn test_composite_command() {
        let mut chart = Chart::new("test", ChartType::default());

        let composite = CompositeCommand::new("Setup chart")
            .add(UpdateChartData::add_series(DataSeries::new(
                "Series 1",
                vec![1.0, 2.0],
            )))
            .add(UpdateChartStyle::update_title(Some(ChartTitle {
                text: "My Chart".to_string(),
                position: TitlePosition::Top,
            })));

        composite.execute(&mut chart).unwrap();

        assert_eq!(chart.data.series.len(), 1);
        assert!(chart.title.is_some());
    }

    #[test]
    fn test_command_descriptions() {
        let cmd1 = InsertChart::new("test", ChartType::default());
        assert!(cmd1.description().contains("Insert"));

        let cmd2 = UpdateChartData::add_series(DataSeries::new("Test", vec![]));
        assert!(cmd2.description().contains("Add series"));

        let cmd3 = ChangeChartType::new(ChartType::Pie {
            doughnut: false,
            explosion: 0.0,
        });
        assert!(cmd3.description().contains("Change chart type"));

        let cmd4 = UpdateChartStyle::set_background(None);
        assert!(cmd4.description().contains("background"));
    }

    #[test]
    fn test_replace_all_data() {
        let mut chart = Chart::new("test", ChartType::default());
        chart
            .data
            .series
            .push(DataSeries::new("Old", vec![1.0, 2.0]));

        let mut new_data = ChartData::new(vec!["X".to_string(), "Y".to_string()]);
        new_data.series.push(DataSeries::new("New", vec![3.0, 4.0]));

        let cmd = UpdateChartData::replace_all(new_data);
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.series.len(), 1);
        assert_eq!(chart.data.series[0].name, "New");
        assert_eq!(chart.data.categories, vec!["X", "Y"]);
    }

    #[test]
    fn test_set_categories() {
        let mut chart = Chart::new("test", ChartType::default());

        let cmd = UpdateChartData::set_categories(vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ]);
        cmd.execute(&mut chart).unwrap();

        assert_eq!(chart.data.categories.len(), 4);
        assert_eq!(chart.data.categories[0], "Q1");
    }
}
