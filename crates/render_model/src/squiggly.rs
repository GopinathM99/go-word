//! Squiggly underline rendering for spellcheck errors
//!
//! This module provides types and utilities for rendering the red wavy
//! underlines that indicate spelling errors.

use crate::render_item::{Color, Rect, RenderItem, SquigglyRenderInfo};
use serde::{Deserialize, Serialize};

/// Style of squiggly underline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SquigglyStyle {
    /// Red wavy line for spelling errors
    Spelling,
    /// Blue wavy line for grammar errors
    Grammar,
    /// Green wavy line for style suggestions
    Style,
    /// Custom color wavy line
    Custom,
}

impl Default for SquigglyStyle {
    fn default() -> Self {
        Self::Spelling
    }
}

/// A squiggly underline for indicating errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquigglyUnderline {
    /// The bounding rectangle (baseline position and width)
    pub bounds: Rect,
    /// Style of the squiggly (determines color)
    pub style: SquigglyStyle,
    /// Custom color (used when style is Custom)
    pub color: Option<Color>,
    /// The node ID this underline belongs to
    pub node_id: String,
    /// Start offset in the text
    pub start_offset: usize,
    /// End offset in the text
    pub end_offset: usize,
    /// Optional error message/tooltip
    pub message: Option<String>,
}

impl SquigglyUnderline {
    /// Create a new spelling error squiggly
    pub fn spelling(
        x: f64,
        y: f64,
        width: f64,
        node_id: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        Self {
            bounds: Rect::new(x, y, width, 3.0), // Fixed height for squiggly
            style: SquigglyStyle::Spelling,
            color: None,
            node_id: node_id.into(),
            start_offset,
            end_offset,
            message: None,
        }
    }

    /// Create a new grammar error squiggly
    pub fn grammar(
        x: f64,
        y: f64,
        width: f64,
        node_id: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        Self {
            bounds: Rect::new(x, y, width, 3.0),
            style: SquigglyStyle::Grammar,
            color: None,
            node_id: node_id.into(),
            start_offset,
            end_offset,
            message: None,
        }
    }

    /// Create a squiggly with custom color
    pub fn custom(
        x: f64,
        y: f64,
        width: f64,
        color: Color,
        node_id: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        Self {
            bounds: Rect::new(x, y, width, 3.0),
            style: SquigglyStyle::Custom,
            color: Some(color),
            node_id: node_id.into(),
            start_offset,
            end_offset,
            message: None,
        }
    }

    /// Set an error message/tooltip
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Get the color for this squiggly
    pub fn get_color(&self) -> Color {
        match self.style {
            SquigglyStyle::Spelling => Color::rgb(255, 0, 0),   // Red
            SquigglyStyle::Grammar => Color::rgb(0, 0, 255),    // Blue
            SquigglyStyle::Style => Color::rgb(0, 128, 0),      // Green
            SquigglyStyle::Custom => self.color.unwrap_or(Color::rgb(255, 0, 0)),
        }
    }

    /// Convert to a render item for the canvas
    pub fn to_render_item(&self) -> RenderItem {
        RenderItem::Squiggly(SquigglyRenderInfo {
            bounds: self.bounds,
            color: self.get_color(),
            node_id: self.node_id.clone(),
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            message: self.message.clone(),
        })
    }
}

// Note: SquigglyRenderInfo is defined in render_item.rs

/// Helper functions for SquigglyRenderInfo
pub fn squiggly_wave_points(bounds: &Rect, amplitude: f64, wavelength: f64) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let baseline_y = bounds.y + bounds.height / 2.0;

    let mut x = bounds.x;
    let end_x = bounds.x + bounds.width;
    let mut going_up = true;

    while x <= end_x {
        let y = if going_up {
            baseline_y - amplitude
        } else {
            baseline_y + amplitude
        };

        points.push((x, y));

        x += wavelength / 2.0;
        going_up = !going_up;
    }

    // Ensure we end at the right edge
    if let Some(last) = points.last() {
        if last.0 < end_x {
            let y = if going_up {
                baseline_y - amplitude
            } else {
                baseline_y + amplitude
            };
            points.push((end_x, y));
        }
    }

    points
}

/// Get SVG path data for a squiggly line
pub fn squiggly_svg_path(bounds: &Rect) -> String {
    let amplitude = 1.5;
    let wavelength = 4.0;
    let points = squiggly_wave_points(bounds, amplitude, wavelength);

    if points.is_empty() {
        return String::new();
    }

    let mut path = format!("M {} {}", points[0].0, points[0].1);

    for point in points.iter().skip(1) {
        path.push_str(&format!(" L {} {}", point.0, point.1));
    }

    path
}

/// Collection of squiggly underlines for a page
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SquigglyLayer {
    /// All squiggly underlines on this layer
    pub underlines: Vec<SquigglyRenderInfo>,
}

impl SquigglyLayer {
    /// Create a new empty layer
    pub fn new() -> Self {
        Self {
            underlines: Vec::new(),
        }
    }

    /// Add a squiggly underline
    pub fn add(&mut self, squiggly: SquigglyRenderInfo) {
        self.underlines.push(squiggly);
    }

    /// Add from a SquigglyUnderline
    pub fn add_underline(&mut self, underline: SquigglyUnderline) {
        self.underlines.push(SquigglyRenderInfo {
            bounds: underline.bounds,
            color: underline.get_color(),
            node_id: underline.node_id,
            start_offset: underline.start_offset,
            end_offset: underline.end_offset,
            message: underline.message,
        });
    }

    /// Clear all underlines
    pub fn clear(&mut self) {
        self.underlines.clear();
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.underlines.is_empty()
    }

    /// Get underline count
    pub fn len(&self) -> usize {
        self.underlines.len()
    }

    /// Find squiggly at a position
    pub fn find_at_position(&self, x: f64, y: f64) -> Option<&SquigglyRenderInfo> {
        self.underlines.iter().find(|s| {
            x >= s.bounds.x
                && x <= s.bounds.x + s.bounds.width
                && y >= s.bounds.y
                && y <= s.bounds.y + s.bounds.height
        })
    }

    /// Remove underlines for a specific node
    pub fn remove_for_node(&mut self, node_id: &str) {
        self.underlines.retain(|s| s.node_id != node_id);
    }

    /// Update underlines for a node (replace all existing)
    pub fn update_for_node(&mut self, node_id: &str, new_underlines: Vec<SquigglyRenderInfo>) {
        self.remove_for_node(node_id);
        self.underlines.extend(new_underlines);
    }
}

/// Error marker type for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorMarkerType {
    /// Spelling error
    Spelling,
    /// Grammar error
    Grammar,
    /// Style suggestion
    Style,
    /// Search highlight
    SearchHighlight,
    /// Find/replace match
    FindMatch,
}

/// A generic error marker that can be rendered in various ways
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMarker {
    /// Type of marker
    pub marker_type: ErrorMarkerType,
    /// Paragraph/node ID
    pub node_id: String,
    /// Start offset in the text
    pub start_offset: usize,
    /// End offset in the text
    pub end_offset: usize,
    /// The text that is marked
    pub text: String,
    /// Optional message/tooltip
    pub message: Option<String>,
    /// Suggestions (for spelling/grammar)
    pub suggestions: Vec<String>,
}

impl ErrorMarker {
    /// Create a spelling error marker
    pub fn spelling(
        node_id: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
        text: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        Self {
            marker_type: ErrorMarkerType::Spelling,
            node_id: node_id.into(),
            start_offset,
            end_offset,
            text: text.into(),
            message: None,
            suggestions,
        }
    }

    /// Create a find match marker
    pub fn find_match(
        node_id: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
        text: impl Into<String>,
    ) -> Self {
        Self {
            marker_type: ErrorMarkerType::FindMatch,
            node_id: node_id.into(),
            start_offset,
            end_offset,
            text: text.into(),
            message: None,
            suggestions: Vec::new(),
        }
    }

    /// Get the appropriate color for this marker type
    pub fn get_color(&self) -> Color {
        match self.marker_type {
            ErrorMarkerType::Spelling => Color::rgb(255, 0, 0),      // Red
            ErrorMarkerType::Grammar => Color::rgb(0, 0, 255),       // Blue
            ErrorMarkerType::Style => Color::rgb(0, 128, 0),         // Green
            ErrorMarkerType::SearchHighlight => Color::rgba(255, 255, 0, 128), // Yellow semi-transparent
            ErrorMarkerType::FindMatch => Color::rgba(255, 200, 0, 128),       // Orange semi-transparent
        }
    }

    /// Check if this marker should show a squiggly underline
    pub fn uses_squiggly(&self) -> bool {
        matches!(
            self.marker_type,
            ErrorMarkerType::Spelling | ErrorMarkerType::Grammar | ErrorMarkerType::Style
        )
    }

    /// Check if this marker should show a highlight
    pub fn uses_highlight(&self) -> bool {
        matches!(
            self.marker_type,
            ErrorMarkerType::SearchHighlight | ErrorMarkerType::FindMatch
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squiggly_spelling() {
        let squiggly = SquigglyUnderline::spelling(10.0, 100.0, 50.0, "para1", 5, 10);

        assert_eq!(squiggly.style, SquigglyStyle::Spelling);
        assert_eq!(squiggly.bounds.x, 10.0);
        assert_eq!(squiggly.bounds.width, 50.0);
        assert_eq!(squiggly.start_offset, 5);
        assert_eq!(squiggly.end_offset, 10);

        let color = squiggly.get_color();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_squiggly_grammar() {
        let squiggly = SquigglyUnderline::grammar(10.0, 100.0, 50.0, "para1", 5, 10);

        assert_eq!(squiggly.style, SquigglyStyle::Grammar);
        let color = squiggly.get_color();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_squiggly_custom() {
        let custom_color = Color::rgb(128, 64, 32);
        let squiggly = SquigglyUnderline::custom(10.0, 100.0, 50.0, custom_color, "para1", 5, 10);

        assert_eq!(squiggly.style, SquigglyStyle::Custom);
        let color = squiggly.get_color();
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
    }

    #[test]
    fn test_squiggly_with_message() {
        let squiggly = SquigglyUnderline::spelling(10.0, 100.0, 50.0, "para1", 5, 10)
            .with_message("Unknown word");

        assert_eq!(squiggly.message, Some("Unknown word".to_string()));
    }

    #[test]
    fn test_wave_points() {
        let bounds = Rect::new(0.0, 10.0, 20.0, 3.0);

        let points = squiggly_wave_points(&bounds, 1.5, 4.0);
        assert!(!points.is_empty());

        // First point should be at start
        assert_eq!(points[0].0, 0.0);
    }

    #[test]
    fn test_svg_path() {
        let bounds = Rect::new(0.0, 10.0, 20.0, 3.0);

        let path = squiggly_svg_path(&bounds);
        assert!(path.starts_with("M"));
        assert!(path.contains("L"));
    }

    #[test]
    fn test_squiggly_layer() {
        let mut layer = SquigglyLayer::new();
        assert!(layer.is_empty());

        layer.add(SquigglyRenderInfo::new(
            Rect::new(0.0, 10.0, 20.0, 3.0),
            Color::rgb(255, 0, 0),
            "para1",
        ));

        assert_eq!(layer.len(), 1);
        assert!(!layer.is_empty());
    }

    #[test]
    fn test_squiggly_layer_find_at_position() {
        let mut layer = SquigglyLayer::new();

        layer.add(SquigglyRenderInfo::new(
            Rect::new(10.0, 100.0, 50.0, 3.0),
            Color::rgb(255, 0, 0),
            "para1",
        ));

        // Inside bounds
        assert!(layer.find_at_position(30.0, 101.0).is_some());

        // Outside bounds
        assert!(layer.find_at_position(5.0, 101.0).is_none());
        assert!(layer.find_at_position(30.0, 50.0).is_none());
    }

    #[test]
    fn test_squiggly_layer_remove_for_node() {
        let mut layer = SquigglyLayer::new();

        layer.add(SquigglyRenderInfo::new(
            Rect::new(0.0, 10.0, 20.0, 3.0),
            Color::rgb(255, 0, 0),
            "para1",
        ));
        layer.add(SquigglyRenderInfo::new(
            Rect::new(30.0, 10.0, 20.0, 3.0),
            Color::rgb(255, 0, 0),
            "para2",
        ));
        layer.add(SquigglyRenderInfo::new(
            Rect::new(60.0, 10.0, 20.0, 3.0),
            Color::rgb(255, 0, 0),
            "para1",
        ));

        assert_eq!(layer.len(), 3);

        layer.remove_for_node("para1");
        assert_eq!(layer.len(), 1);
        assert_eq!(layer.underlines[0].node_id, "para2");
    }

    #[test]
    fn test_error_marker_spelling() {
        let marker = ErrorMarker::spelling(
            "para1",
            5,
            10,
            "tset",
            vec!["test".to_string(), "set".to_string()],
        );

        assert_eq!(marker.marker_type, ErrorMarkerType::Spelling);
        assert!(marker.uses_squiggly());
        assert!(!marker.uses_highlight());
        assert_eq!(marker.suggestions.len(), 2);
    }

    #[test]
    fn test_error_marker_find_match() {
        let marker = ErrorMarker::find_match("para1", 5, 10, "hello");

        assert_eq!(marker.marker_type, ErrorMarkerType::FindMatch);
        assert!(!marker.uses_squiggly());
        assert!(marker.uses_highlight());
    }
}
