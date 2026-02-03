//! DrawingML writer for chart XML
//!
//! Serializes Chart structures back to DrawingML XML format
//! for saving in DOCX packages.

use crate::error::{ChartError, ChartResult};
use crate::model::*;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Cursor;

/// Writer for DrawingML chart XML
pub struct DrawingMLWriter {
    /// XML namespace prefix for chart elements
    chart_prefix: String,
    /// XML namespace prefix for drawing elements
    draw_prefix: String,
}

impl Default for DrawingMLWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawingMLWriter {
    /// Create a new writer with default namespace prefixes
    pub fn new() -> Self {
        Self {
            chart_prefix: "c".to_string(),
            draw_prefix: "a".to_string(),
        }
    }

    /// Set the chart namespace prefix
    pub fn chart_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.chart_prefix = prefix.into();
        self
    }

    /// Set the drawing namespace prefix
    pub fn draw_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.draw_prefix = prefix.into();
        self
    }

    /// Write a chart to XML string
    pub fn write(&self, chart: &Chart) -> ChartResult<String> {
        // If we have preserved original XML, return that for round-trip fidelity
        if let Some(ref original) = chart.original_xml {
            return Ok(original.clone());
        }

        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // Write XML declaration
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), Some("yes"))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write chartSpace root element
        self.write_chart_space(&mut writer, chart)?;

        let result = writer.into_inner().into_inner();
        String::from_utf8(result).map_err(|e| ChartError::Serialization(e.to_string()))
    }

    fn write_chart_space<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
    ) -> ChartResult<()> {
        let mut chart_space = BytesStart::new(format!("{}:chartSpace", self.chart_prefix));
        chart_space.push_attribute((
            format!("xmlns:{}", self.chart_prefix).as_str(),
            "http://schemas.openxmlformats.org/drawingml/2006/chart",
        ));
        chart_space.push_attribute((
            format!("xmlns:{}", self.draw_prefix).as_str(),
            "http://schemas.openxmlformats.org/drawingml/2006/main",
        ));
        chart_space.push_attribute((
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ));

        writer
            .write_event(Event::Start(chart_space))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write date1904 element (optional)
        self.write_empty_element(writer, "date1904", &[("val", "0")])?;

        // Write lang element
        self.write_empty_element(writer, "lang", &[("val", "en-US")])?;

        // Write chart element
        self.write_chart(writer, chart)?;

        // Close chartSpace
        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:chartSpace",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:chart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write title if present
        if let Some(ref title) = chart.title {
            self.write_title(writer, title)?;
        }

        // Write auto title deleted (if no title)
        if chart.title.is_none() {
            self.write_empty_element(writer, "autoTitleDeleted", &[("val", "1")])?;
        }

        // Write plot area
        self.write_plot_area(writer, chart)?;

        // Write legend if present
        if let Some(ref legend) = chart.legend {
            if legend.visible {
                self.write_legend(writer, legend)?;
            }
        }

        // Write plotVisOnly
        self.write_empty_element(writer, "plotVisOnly", &[("val", "1")])?;

        // Write dispBlanksAs
        self.write_empty_element(writer, "dispBlanksAs", &[("val", "gap")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:chart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_title<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        title: &ChartTitle,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:title",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write tx element
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:tx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write rich text
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:rich",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Body properties
        writer
            .write_event(Event::Empty(BytesStart::new(format!(
                "{}:bodyPr",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // List style
        writer
            .write_event(Event::Empty(BytesStart::new(format!(
                "{}:lstStyle",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Paragraph
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:p",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Run
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:r",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Text
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:t",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Text(BytesText::new(&title.text)))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:t",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:r",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:p",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:rich",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:tx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Overlay
        self.write_empty_element(
            writer,
            "overlay",
            &[("val", if title.position == TitlePosition::Overlay { "1" } else { "0" })],
        )?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:title",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_plot_area<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:plotArea",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write layout (empty for auto)
        writer
            .write_event(Event::Empty(BytesStart::new(format!(
                "{}:layout",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write the specific chart type
        self.write_chart_type(writer, chart)?;

        // Write category axis if present
        if let Some(ref axis) = chart.axes.category_axis {
            self.write_category_axis(writer, axis)?;
        }

        // Write value axis if present
        if let Some(ref axis) = chart.axes.value_axis {
            self.write_value_axis(writer, axis)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:plotArea",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_chart_type<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
    ) -> ChartResult<()> {
        match &chart.chart_type {
            ChartType::Bar {
                horizontal,
                stacked,
                stacked_percent,
            } => self.write_bar_chart(writer, chart, *horizontal, *stacked, *stacked_percent),
            ChartType::Column {
                stacked,
                stacked_percent,
            } => self.write_bar_chart(writer, chart, false, *stacked, *stacked_percent),
            ChartType::Line { smooth, markers } => {
                self.write_line_chart(writer, chart, *smooth, *markers)
            }
            ChartType::Pie { doughnut, explosion } => {
                self.write_pie_chart(writer, chart, *doughnut, *explosion)
            }
            ChartType::Area { stacked } => self.write_area_chart(writer, chart, *stacked),
            ChartType::Scatter { with_lines } => {
                self.write_scatter_chart(writer, chart, *with_lines)
            }
            ChartType::Bubble => self.write_bubble_chart(writer, chart),
            ChartType::Radar { filled } => self.write_radar_chart(writer, chart, *filled),
            ChartType::Stock => self.write_stock_chart(writer, chart),
        }
    }

    fn write_bar_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
        horizontal: bool,
        stacked: bool,
        stacked_percent: bool,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:barChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write bar direction
        self.write_empty_element(
            writer,
            "barDir",
            &[("val", if horizontal { "bar" } else { "col" })],
        )?;

        // Write grouping
        let grouping = if stacked_percent {
            "percentStacked"
        } else if stacked {
            "stacked"
        } else {
            "clustered"
        };
        self.write_empty_element(writer, "grouping", &[("val", grouping)])?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "0")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write gap width
        self.write_empty_element(writer, "gapWidth", &[("val", "150")])?;

        // Write overlap for stacked
        if stacked || stacked_percent {
            self.write_empty_element(writer, "overlap", &[("val", "100")])?;
        }

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:barChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_line_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
        smooth: bool,
        _markers: bool,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:lineChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write grouping
        self.write_empty_element(writer, "grouping", &[("val", "standard")])?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "0")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write smooth
        self.write_empty_element(writer, "smooth", &[("val", if smooth { "1" } else { "0" })])?;

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:lineChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_pie_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
        doughnut: bool,
        explosion: f32,
    ) -> ChartResult<()> {
        let element_name = if doughnut { "doughnutChart" } else { "pieChart" };

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:{}",
                self.chart_prefix, element_name
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "1")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write explosion if non-zero
        if explosion > 0.0 {
            self.write_empty_element(writer, "explosion", &[("val", &explosion.to_string())])?;
        }

        // Write first slice angle
        self.write_empty_element(writer, "firstSliceAng", &[("val", "0")])?;

        // Write hole size for doughnut
        if doughnut {
            self.write_empty_element(writer, "holeSize", &[("val", "50")])?;
        }

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:{}",
                self.chart_prefix, element_name
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_area_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
        stacked: bool,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:areaChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write grouping
        self.write_empty_element(
            writer,
            "grouping",
            &[("val", if stacked { "stacked" } else { "standard" })],
        )?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "0")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:areaChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_scatter_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
        with_lines: bool,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:scatterChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write scatter style
        self.write_empty_element(
            writer,
            "scatterStyle",
            &[("val", if with_lines { "lineMarker" } else { "marker" })],
        )?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "0")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:scatterChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_bubble_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:bubbleChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "0")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write bubble scale
        self.write_empty_element(writer, "bubbleScale", &[("val", "100")])?;

        // Write show negative bubbles
        self.write_empty_element(writer, "showNegBubbles", &[("val", "0")])?;

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:bubbleChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_radar_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
        filled: bool,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:radarChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write radar style
        self.write_empty_element(
            writer,
            "radarStyle",
            &[("val", if filled { "filled" } else { "marker" })],
        )?;

        // Write vary colors
        self.write_empty_element(writer, "varyColors", &[("val", "0")])?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:radarChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_stock_chart<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        chart: &Chart,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:stockChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write series
        for (idx, series) in chart.data.series.iter().enumerate() {
            self.write_series(writer, series, idx, &chart.data.categories, &chart.style)?;
        }

        // Write axis IDs
        self.write_empty_element(writer, "axId", &[("val", "1")])?;
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:stockChart",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_series<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        series: &DataSeries,
        idx: usize,
        categories: &[String],
        style: &ChartStyle,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:ser",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write index
        self.write_empty_element(writer, "idx", &[("val", &idx.to_string())])?;

        // Write order
        self.write_empty_element(writer, "order", &[("val", &idx.to_string())])?;

        // Write series text (name)
        self.write_series_text(writer, &series.name)?;

        // Write shape properties (color)
        let color = series
            .color
            .unwrap_or_else(|| style.colors.get(idx % style.colors.len()).copied().unwrap_or(Color::BLUE));
        self.write_shape_properties(writer, &color)?;

        // Write categories if present
        if !categories.is_empty() {
            self.write_categories(writer, categories)?;
        }

        // Write values
        self.write_values(writer, &series.values)?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:ser",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_series_text<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:tx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:v",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Text(BytesText::new(name)))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:v",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:tx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_shape_properties<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        color: &Color,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:spPr",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write solid fill
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:solidFill",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        let mut srgb = BytesStart::new(format!("{}:srgbClr", self.draw_prefix));
        srgb.push_attribute(("val", color.to_hex().as_str()));
        writer
            .write_event(Event::Empty(srgb))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:solidFill",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:spPr",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_categories<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        categories: &[String],
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:cat",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:strLit",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write point count
        self.write_empty_element(writer, "ptCount", &[("val", &categories.len().to_string())])?;

        // Write each category
        for (idx, cat) in categories.iter().enumerate() {
            let mut pt = BytesStart::new(format!("{}:pt", self.chart_prefix));
            pt.push_attribute(("idx", idx.to_string().as_str()));
            writer
                .write_event(Event::Start(pt))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::Start(BytesStart::new(format!(
                    "{}:v",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::Text(BytesText::new(cat)))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::End(BytesEnd::new(format!(
                    "{}:v",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::End(BytesEnd::new(format!(
                    "{}:pt",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:strLit",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:cat",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_values<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        values: &[f64],
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:val",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:numLit",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write format code
        self.write_element_with_text(writer, "formatCode", "General")?;

        // Write point count
        self.write_empty_element(writer, "ptCount", &[("val", &values.len().to_string())])?;

        // Write each value
        for (idx, val) in values.iter().enumerate() {
            let mut pt = BytesStart::new(format!("{}:pt", self.chart_prefix));
            pt.push_attribute(("idx", idx.to_string().as_str()));
            writer
                .write_event(Event::Start(pt))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::Start(BytesStart::new(format!(
                    "{}:v",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::Text(BytesText::new(&val.to_string())))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::End(BytesEnd::new(format!(
                    "{}:v",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;

            writer
                .write_event(Event::End(BytesEnd::new(format!(
                    "{}:pt",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:numLit",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:val",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_legend<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        legend: &Legend,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:legend",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write legend position
        let pos = match legend.position {
            LegendPosition::Top => "t",
            LegendPosition::Bottom => "b",
            LegendPosition::Left => "l",
            LegendPosition::Right => "r",
            LegendPosition::TopRight => "tr",
            LegendPosition::None => "r",
        };
        self.write_empty_element(writer, "legendPos", &[("val", pos)])?;

        // Write overlay
        self.write_empty_element(writer, "overlay", &[("val", "0")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:legend",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_category_axis<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        axis: &Axis,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:catAx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write axis ID
        self.write_empty_element(writer, "axId", &[("val", "1")])?;

        // Write scaling
        self.write_axis_scaling(writer, axis)?;

        // Write position
        self.write_empty_element(writer, "axPos", &[("val", "b")])?;

        // Write major gridlines if enabled
        if axis.major_gridlines {
            writer
                .write_event(Event::Empty(BytesStart::new(format!(
                    "{}:majorGridlines",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;
        }

        // Write title if present
        if let Some(ref title) = axis.title {
            self.write_axis_title(writer, title)?;
        }

        // Write cross axis ID
        self.write_empty_element(writer, "crossAx", &[("val", "2")])?;

        // Write crosses
        self.write_empty_element(writer, "crosses", &[("val", "autoZero")])?;

        // Write auto
        self.write_empty_element(writer, "auto", &[("val", "1")])?;

        // Write label alignment
        self.write_empty_element(writer, "lblAlgn", &[("val", "ctr")])?;

        // Write label offset
        self.write_empty_element(writer, "lblOffset", &[("val", "100")])?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:catAx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_value_axis<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        axis: &Axis,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:valAx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write axis ID
        self.write_empty_element(writer, "axId", &[("val", "2")])?;

        // Write scaling
        self.write_axis_scaling(writer, axis)?;

        // Write position
        self.write_empty_element(writer, "axPos", &[("val", "l")])?;

        // Write major gridlines if enabled
        if axis.major_gridlines {
            writer
                .write_event(Event::Empty(BytesStart::new(format!(
                    "{}:majorGridlines",
                    self.chart_prefix
                ))))
                .map_err(|e| ChartError::Serialization(e.to_string()))?;
        }

        // Write title if present
        if let Some(ref title) = axis.title {
            self.write_axis_title(writer, title)?;
        }

        // Write number format
        if let Some(ref format) = axis.number_format {
            self.write_empty_element(
                writer,
                "numFmt",
                &[("formatCode", format.as_str()), ("sourceLinked", "0")],
            )?;
        }

        // Write cross axis ID
        self.write_empty_element(writer, "crossAx", &[("val", "1")])?;

        // Write crosses
        self.write_empty_element(writer, "crosses", &[("val", "autoZero")])?;

        // Write major unit if specified
        if let Some(unit) = axis.major_unit {
            self.write_empty_element(writer, "majorUnit", &[("val", &unit.to_string())])?;
        }

        // Write minor unit if specified
        if let Some(unit) = axis.minor_unit {
            self.write_empty_element(writer, "minorUnit", &[("val", &unit.to_string())])?;
        }

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:valAx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_axis_scaling<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        axis: &Axis,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:scaling",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        // Write orientation
        self.write_empty_element(
            writer,
            "orientation",
            &[("val", if axis.reversed { "maxMin" } else { "minMax" })],
        )?;

        // Write min if specified
        if let Some(min) = axis.min {
            self.write_empty_element(writer, "min", &[("val", &min.to_string())])?;
        }

        // Write max if specified
        if let Some(max) = axis.max {
            self.write_empty_element(writer, "max", &[("val", &max.to_string())])?;
        }

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:scaling",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_axis_title<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        title: &str,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:title",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:tx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:rich",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Empty(BytesStart::new(format!(
                "{}:bodyPr",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Empty(BytesStart::new(format!(
                "{}:lstStyle",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:p",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:r",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:t",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Text(BytesText::new(title)))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:t",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:r",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:p",
                self.draw_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:rich",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:tx",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:title",
                self.chart_prefix
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_empty_element<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> ChartResult<()> {
        let mut element = BytesStart::new(format!("{}:{}", self.chart_prefix, name));
        for (key, val) in attrs {
            element.push_attribute((*key, *val));
        }
        writer
            .write_event(Event::Empty(element))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;
        Ok(())
    }

    fn write_element_with_text<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        name: &str,
        text: &str,
    ) -> ChartResult<()> {
        writer
            .write_event(Event::Start(BytesStart::new(format!(
                "{}:{}",
                self.chart_prefix, name
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new(format!(
                "{}:{}",
                self.chart_prefix, name
            ))))
            .map_err(|e| ChartError::Serialization(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_simple_bar_chart() {
        let mut chart = Chart::new(
            "test",
            ChartType::Bar {
                horizontal: false,
                stacked: false,
                stacked_percent: false,
            },
        );
        chart.add_series(DataSeries::new("Series 1", vec![10.0, 20.0, 30.0]));

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("barChart"));
        assert!(xml.contains("barDir"));
        assert!(xml.contains("Series 1"));
    }

    #[test]
    fn test_write_chart_with_title() {
        let chart = Chart::new("test", ChartType::Line { smooth: false, markers: true })
            .with_title("My Chart");

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("title"));
        assert!(xml.contains("My Chart"));
    }

    #[test]
    fn test_write_chart_with_legend() {
        let chart = Chart::new("test", ChartType::Pie { doughnut: false, explosion: 0.0 })
            .with_legend(LegendPosition::Bottom);

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("legend"));
        assert!(xml.contains("legendPos"));
    }

    #[test]
    fn test_write_line_chart() {
        let mut chart = Chart::new("test", ChartType::Line { smooth: true, markers: true });
        chart.add_series(DataSeries::new("Line", vec![1.0, 2.0, 3.0]));

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("lineChart"));
        assert!(xml.contains("smooth"));
    }

    #[test]
    fn test_write_pie_chart() {
        let mut chart = Chart::new("test", ChartType::Pie { doughnut: false, explosion: 10.0 });
        chart.add_series(DataSeries::new("Pie", vec![25.0, 50.0, 25.0]));

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("pieChart"));
    }

    #[test]
    fn test_write_doughnut_chart() {
        let chart = Chart::new("test", ChartType::Pie { doughnut: true, explosion: 0.0 });

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("doughnutChart"));
        assert!(xml.contains("holeSize"));
    }

    #[test]
    fn test_round_trip_with_original_xml() {
        let original_xml = r#"<c:chart><c:plotArea/></c:chart>"#;
        let mut chart = Chart::new("test", ChartType::default());
        chart.original_xml = Some(original_xml.to_string());

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert_eq!(xml, original_xml);
    }

    #[test]
    fn test_write_chart_with_categories() {
        let mut chart = Chart::new(
            "test",
            ChartType::Column {
                stacked: false,
                stacked_percent: false,
            },
        );
        chart.set_categories(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        chart.add_series(DataSeries::new("Data", vec![1.0, 2.0, 3.0]));

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("cat"));
        assert!(xml.contains("strLit"));
    }

    #[test]
    fn test_write_chart_with_axes() {
        let mut chart = Chart::new(
            "test",
            ChartType::Bar {
                horizontal: false,
                stacked: false,
                stacked_percent: false,
            },
        );
        chart.axes.category_axis = Some(Axis {
            title: Some("Categories".to_string()),
            ..Axis::default()
        });
        chart.axes.value_axis = Some(Axis {
            title: Some("Values".to_string()),
            min: Some(0.0),
            max: Some(100.0),
            ..Axis::default()
        });

        let writer = DrawingMLWriter::new();
        let xml = writer.write(&chart).unwrap();

        assert!(xml.contains("catAx"));
        assert!(xml.contains("valAx"));
    }
}
