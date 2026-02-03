//! DrawingML parser for chart XML
//!
//! Parses chart*.xml files from DOCX packages according to the
//! Office Open XML (OOXML) DrawingML specification.

use crate::error::{ChartError, ChartResult};
use crate::model::*;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::io::BufRead;

/// Parser for DrawingML chart XML
pub struct DrawingMLParser {
    /// Whether to preserve original XML for round-trip
    preserve_original: bool,
}

impl Default for DrawingMLParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawingMLParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            preserve_original: true,
        }
    }

    /// Set whether to preserve original XML
    pub fn preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }

    /// Parse chart XML from a string
    pub fn parse(&self, xml: &str) -> ChartResult<Chart> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut chart = Chart::new("chart", ChartType::default());
        let mut buf = Vec::new();

        if self.preserve_original {
            chart.original_xml = Some(xml.to_string());
        }

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    self.handle_start_element(&mut reader, e, &mut chart)?;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(chart)
    }

    /// Parse chart XML from a reader
    pub fn parse_reader<R: BufRead>(&self, reader: R) -> ChartResult<Chart> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.trim_text(true);

        let mut chart = Chart::new("chart", ChartType::default());
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    self.handle_start_element_reader(&mut xml_reader, e, &mut chart)?;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(chart)
    }

    fn handle_start_element(
        &self,
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
        chart: &mut Chart,
    ) -> ChartResult<()> {
        let local_name = element.local_name();
        let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

        match name {
            "barChart" => self.parse_bar_chart(reader, chart)?,
            "bar3DChart" => self.parse_bar_chart(reader, chart)?,
            "lineChart" => self.parse_line_chart(reader, chart)?,
            "line3DChart" => self.parse_line_chart(reader, chart)?,
            "pieChart" => self.parse_pie_chart(reader, chart, false)?,
            "pie3DChart" => self.parse_pie_chart(reader, chart, false)?,
            "doughnutChart" => self.parse_pie_chart(reader, chart, true)?,
            "areaChart" => self.parse_area_chart(reader, chart)?,
            "area3DChart" => self.parse_area_chart(reader, chart)?,
            "scatterChart" => self.parse_scatter_chart(reader, chart)?,
            "bubbleChart" => self.parse_bubble_chart(reader, chart)?,
            "radarChart" => self.parse_radar_chart(reader, chart)?,
            "stockChart" => self.parse_stock_chart(reader, chart)?,
            "title" => self.parse_title(reader, chart)?,
            "legend" => self.parse_legend(reader, chart)?,
            "catAx" => self.parse_category_axis(reader, chart)?,
            "valAx" => self.parse_value_axis(reader, chart)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_start_element_reader<R: BufRead>(
        &self,
        _reader: &mut Reader<R>,
        element: &BytesStart,
        chart: &mut Chart,
    ) -> ChartResult<()> {
        let local_name = element.local_name();
        let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

        match name {
            "barChart" | "bar3DChart" => {
                chart.chart_type = ChartType::Bar {
                    horizontal: false,
                    stacked: false,
                    stacked_percent: false,
                };
            }
            "lineChart" | "line3DChart" => {
                chart.chart_type = ChartType::Line {
                    smooth: false,
                    markers: true,
                };
            }
            "pieChart" | "pie3DChart" => {
                chart.chart_type = ChartType::Pie {
                    doughnut: false,
                    explosion: 0.0,
                };
            }
            "doughnutChart" => {
                chart.chart_type = ChartType::Pie {
                    doughnut: true,
                    explosion: 0.0,
                };
            }
            "areaChart" | "area3DChart" => {
                chart.chart_type = ChartType::Area { stacked: false };
            }
            "scatterChart" => {
                chart.chart_type = ChartType::Scatter { with_lines: false };
            }
            "bubbleChart" => {
                chart.chart_type = ChartType::Bubble;
            }
            "radarChart" => {
                chart.chart_type = ChartType::Radar { filled: false };
            }
            "stockChart" => {
                chart.chart_type = ChartType::Stock;
            }
            _ => {}
        }

        Ok(())
    }

    fn parse_bar_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut horizontal = false;
        let mut stacked = false;
        let mut stacked_percent = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "barDir" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                horizontal = val == "bar";
                            }
                        }
                        "grouping" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                stacked = val == "stacked" || val == "percentStacked";
                                stacked_percent = val == "percentStacked";
                            }
                        }
                        "ser" => {
                            if let Some(series) = self.parse_series(reader)? {
                                chart.data.series.push(series);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "barChart" || name == "bar3DChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Bar {
            horizontal,
            stacked,
            stacked_percent,
        };

        Ok(())
    }

    fn parse_line_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut smooth = false;
        let markers = true;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "smooth" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                smooth = val == "1" || val == "true";
                            }
                        }
                        "marker" => {
                            // Check for marker settings
                        }
                        "ser" => {
                            if let Some(series) = self.parse_series(reader)? {
                                chart.data.series.push(series);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "lineChart" || name == "line3DChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Line { smooth, markers };

        Ok(())
    }

    fn parse_pie_chart(
        &self,
        reader: &mut Reader<&[u8]>,
        chart: &mut Chart,
        is_doughnut: bool,
    ) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut explosion = 0.0f32;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "explosion" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                explosion = val.parse().unwrap_or(0.0);
                            }
                        }
                        "ser" => {
                            if let Some(series) = self.parse_series(reader)? {
                                chart.data.series.push(series);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "pieChart" || name == "pie3DChart" || name == "doughnutChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Pie {
            doughnut: is_doughnut,
            explosion,
        };

        Ok(())
    }

    fn parse_area_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut stacked = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "grouping" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                stacked = val == "stacked" || val == "percentStacked";
                            }
                        }
                        "ser" => {
                            if let Some(series) = self.parse_series(reader)? {
                                chart.data.series.push(series);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "areaChart" || name == "area3DChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Area { stacked };

        Ok(())
    }

    fn parse_scatter_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut with_lines = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "scatterStyle" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                // Check for both "Line" and "line" variants
                                with_lines = val.to_lowercase().contains("line");
                            }
                        }
                        "ser" => {
                            if let Some(series) = self.parse_series(reader)? {
                                chart.data.series.push(series);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "scatterChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Scatter { with_lines };

        Ok(())
    }

    fn parse_bubble_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "ser" {
                        if let Some(series) = self.parse_series(reader)? {
                            chart.data.series.push(series);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "bubbleChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Bubble;

        Ok(())
    }

    fn parse_radar_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut filled = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "radarStyle" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                filled = val == "filled";
                            }
                        }
                        "ser" => {
                            if let Some(series) = self.parse_series(reader)? {
                                chart.data.series.push(series);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "radarChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Radar { filled };

        Ok(())
    }

    fn parse_stock_chart(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "ser" {
                        if let Some(series) = self.parse_series(reader)? {
                            chart.data.series.push(series);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "stockChart" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.chart_type = ChartType::Stock;

        Ok(())
    }

    fn parse_series(&self, reader: &mut Reader<&[u8]>) -> ChartResult<Option<DataSeries>> {
        let mut buf = Vec::new();
        let mut name = String::new();
        let mut values: Vec<f64> = Vec::new();
        let mut color: Option<Color> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let elem_name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match elem_name {
                        "tx" => {
                            name = self.parse_series_text(reader)?;
                        }
                        "val" => {
                            values = self.parse_numeric_data(reader)?;
                        }
                        "yVal" => {
                            values = self.parse_numeric_data(reader)?;
                        }
                        "spPr" => {
                            color = self.parse_shape_properties(reader)?;
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let elem_name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if elem_name == "ser" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        if name.is_empty() && values.is_empty() {
            return Ok(None);
        }

        let mut series = DataSeries::new(name, values);
        series.color = color;

        Ok(Some(series))
    }

    fn parse_series_text(&self, reader: &mut Reader<&[u8]>) -> ChartResult<String> {
        let mut buf = Vec::new();
        let mut text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "v" {
                        text = self.read_text_content(reader)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "tx" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(text)
    }

    fn parse_numeric_data(&self, reader: &mut Reader<&[u8]>) -> ChartResult<Vec<f64>> {
        let mut buf = Vec::new();
        let mut values = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "v" {
                        let text = self.read_text_content(reader)?;
                        if let Ok(val) = text.parse::<f64>() {
                            values.push(val);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "val" || name == "yVal" || name == "numCache" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(values)
    }

    fn parse_shape_properties(&self, reader: &mut Reader<&[u8]>) -> ChartResult<Option<Color>> {
        let mut buf = Vec::new();
        let mut color: Option<Color> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "srgbClr" {
                        if let Some(val) = get_attribute(e, "val")? {
                            color = Color::from_hex(&val);
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "spPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(color)
    }

    fn read_text_content(&self, reader: &mut Reader<&[u8]>) -> ChartResult<String> {
        let mut buf = Vec::new();
        let mut text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) => {
                    text = e.unescape().unwrap_or_default().to_string();
                }
                Ok(Event::End(_)) => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(text)
    }

    fn parse_title(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut title_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "t" {
                        title_text = self.read_text_content(reader)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "title" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        if !title_text.is_empty() {
            chart.title = Some(ChartTitle {
                text: title_text,
                position: TitlePosition::Top,
            });
        }

        Ok(())
    }

    fn parse_legend(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let mut buf = Vec::new();
        let mut position = LegendPosition::Right;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    if name == "legendPos" {
                        if let Some(val) = get_attribute(e, "val")? {
                            position = match val.as_str() {
                                "t" => LegendPosition::Top,
                                "b" => LegendPosition::Bottom,
                                "l" => LegendPosition::Left,
                                "r" => LegendPosition::Right,
                                "tr" => LegendPosition::TopRight,
                                _ => LegendPosition::Right,
                            };
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == "legend" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        chart.legend = Some(Legend {
            position,
            visible: true,
        });

        Ok(())
    }

    fn parse_category_axis(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let axis = self.parse_axis(reader, "catAx")?;
        chart.axes.category_axis = Some(axis);
        Ok(())
    }

    fn parse_value_axis(&self, reader: &mut Reader<&[u8]>, chart: &mut Chart) -> ChartResult<()> {
        let axis = self.parse_axis(reader, "valAx")?;
        chart.axes.value_axis = Some(axis);
        Ok(())
    }

    fn parse_axis(&self, reader: &mut Reader<&[u8]>, end_tag: &str) -> ChartResult<Axis> {
        let mut buf = Vec::new();
        let mut axis = Axis::default();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                    match name {
                        "scaling" => {
                            // Parse min/max inside scaling element
                        }
                        "orientation" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                axis.reversed = val == "maxMin";
                            }
                        }
                        "min" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                axis.min = val.parse().ok();
                            }
                        }
                        "max" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                axis.max = val.parse().ok();
                            }
                        }
                        "majorGridlines" => {
                            axis.major_gridlines = true;
                        }
                        "minorGridlines" => {
                            axis.minor_gridlines = true;
                        }
                        "majorUnit" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                axis.major_unit = val.parse().ok();
                            }
                        }
                        "minorUnit" => {
                            if let Some(val) = get_attribute(e, "val")? {
                                axis.minor_unit = val.parse().ok();
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                    if name == end_tag {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ChartError::XmlParse(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(axis)
    }
}

/// Helper function to get an attribute value from an element
fn get_attribute(element: &BytesStart, attr_name: &str) -> ChartResult<Option<String>> {
    for attr_result in element.attributes() {
        let attr = attr_result?;
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        if key == attr_name || key.ends_with(&format!(":{}", attr_name)) {
            let value = std::str::from_utf8(&attr.value).unwrap_or("");
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_bar_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:barChart>
                        <c:barDir val="col"/>
                        <c:grouping val="clustered"/>
                        <c:ser>
                            <c:tx><c:v>Series 1</c:v></c:tx>
                            <c:val>
                                <c:numCache>
                                    <c:pt idx="0"><c:v>10</c:v></c:pt>
                                    <c:pt idx="1"><c:v>20</c:v></c:pt>
                                    <c:pt idx="2"><c:v>30</c:v></c:pt>
                                </c:numCache>
                            </c:val>
                        </c:ser>
                    </c:barChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Bar { horizontal: false, .. }));
        assert_eq!(chart.data.series.len(), 1);
        assert_eq!(chart.data.series[0].name, "Series 1");
        assert_eq!(chart.data.series[0].values, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_parse_horizontal_bar_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:barChart>
                        <c:barDir val="bar"/>
                        <c:grouping val="clustered"/>
                    </c:barChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Bar { horizontal: true, .. }));
    }

    #[test]
    fn test_parse_stacked_bar_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:barChart>
                        <c:barDir val="col"/>
                        <c:grouping val="stacked"/>
                    </c:barChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Bar { stacked: true, stacked_percent: false, .. }));
    }

    #[test]
    fn test_parse_line_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:lineChart>
                        <c:smooth val="1"/>
                        <c:ser>
                            <c:tx><c:v>Line Series</c:v></c:tx>
                            <c:val>
                                <c:numCache>
                                    <c:pt idx="0"><c:v>5</c:v></c:pt>
                                    <c:pt idx="1"><c:v>15</c:v></c:pt>
                                </c:numCache>
                            </c:val>
                        </c:ser>
                    </c:lineChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Line { smooth: true, .. }));
        assert_eq!(chart.data.series.len(), 1);
    }

    #[test]
    fn test_parse_pie_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:pieChart>
                        <c:ser>
                            <c:tx><c:v>Pie Data</c:v></c:tx>
                            <c:val>
                                <c:numCache>
                                    <c:pt idx="0"><c:v>25</c:v></c:pt>
                                    <c:pt idx="1"><c:v>50</c:v></c:pt>
                                    <c:pt idx="2"><c:v>25</c:v></c:pt>
                                </c:numCache>
                            </c:val>
                        </c:ser>
                    </c:pieChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Pie { doughnut: false, .. }));
    }

    #[test]
    fn test_parse_doughnut_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:doughnutChart>
                        <c:ser>
                            <c:tx><c:v>Doughnut</c:v></c:tx>
                        </c:ser>
                    </c:doughnutChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Pie { doughnut: true, .. }));
    }

    #[test]
    fn test_parse_area_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:areaChart>
                        <c:grouping val="standard"/>
                    </c:areaChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Area { stacked: false }));
    }

    #[test]
    fn test_parse_scatter_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:scatterChart>
                        <c:scatterStyle val="lineMarker"/>
                    </c:scatterChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Scatter { with_lines: true }));
    }

    #[test]
    fn test_parse_radar_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:radarChart>
                        <c:radarStyle val="filled"/>
                    </c:radarChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(matches!(chart.chart_type, ChartType::Radar { filled: true }));
    }

    #[test]
    fn test_parse_chart_with_title() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:title>
                    <c:tx>
                        <c:rich>
                            <a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
                                <a:r>
                                    <a:t>My Chart Title</a:t>
                                </a:r>
                            </a:p>
                        </c:rich>
                    </c:tx>
                </c:title>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(chart.title.is_some());
        assert_eq!(chart.title.as_ref().unwrap().text, "My Chart Title");
    }

    #[test]
    fn test_parse_chart_with_legend() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:legend>
                    <c:legendPos val="b"/>
                </c:legend>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(chart.legend.is_some());
        assert_eq!(chart.legend.as_ref().unwrap().position, LegendPosition::Bottom);
    }

    #[test]
    fn test_parse_multiple_series() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea>
                    <c:barChart>
                        <c:barDir val="col"/>
                        <c:ser>
                            <c:tx><c:v>Series A</c:v></c:tx>
                            <c:val><c:numCache><c:pt idx="0"><c:v>1</c:v></c:pt></c:numCache></c:val>
                        </c:ser>
                        <c:ser>
                            <c:tx><c:v>Series B</c:v></c:tx>
                            <c:val><c:numCache><c:pt idx="0"><c:v>2</c:v></c:pt></c:numCache></c:val>
                        </c:ser>
                        <c:ser>
                            <c:tx><c:v>Series C</c:v></c:tx>
                            <c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val>
                        </c:ser>
                    </c:barChart>
                </c:plotArea>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert_eq!(chart.data.series.len(), 3);
        assert_eq!(chart.data.series[0].name, "Series A");
        assert_eq!(chart.data.series[1].name, "Series B");
        assert_eq!(chart.data.series[2].name, "Series C");
    }

    #[test]
    fn test_preserve_original_xml() {
        let xml = r#"<?xml version="1.0"?><chart/>"#;

        let parser = DrawingMLParser::new().preserve_original(true);
        let chart = parser.parse(xml).unwrap();

        assert!(chart.original_xml.is_some());
        assert_eq!(chart.original_xml.as_ref().unwrap(), xml);
    }

    #[test]
    fn test_parse_empty_chart() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart>
                <c:plotArea/>
            </c:chart>
        </c:chartSpace>"#;

        let parser = DrawingMLParser::new();
        let chart = parser.parse(xml).unwrap();

        assert!(chart.data.series.is_empty());
    }
}
