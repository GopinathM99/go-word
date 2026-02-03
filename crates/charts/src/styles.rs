//! Chart styling
//!
//! This module provides predefined color schemes, style presets,
//! and utilities for applying consistent styling to charts.

use crate::model::*;
use serde::{Deserialize, Serialize};

/// Predefined color schemes for charts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Microsoft Office default colors
    Office,
    /// Colorful scheme with vibrant colors
    Colorful,
    /// Monochrome blue scheme
    MonochromeBlue,
    /// Monochrome green scheme
    MonochromeGreen,
    /// Monochrome gray scheme
    MonochromeGray,
    /// Warm colors (reds, oranges, yellows)
    Warm,
    /// Cool colors (blues, greens, purples)
    Cool,
    /// High contrast for accessibility
    HighContrast,
    /// Pastel colors
    Pastel,
    /// Earthy/natural colors
    Earth,
    /// Professional/corporate colors
    Corporate,
    /// Ocean-inspired colors
    Ocean,
    /// Sunset-inspired colors
    Sunset,
    /// Forest-inspired colors
    Forest,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme::Office
    }
}

impl ColorScheme {
    /// Get the colors for this scheme
    pub fn colors(&self) -> Vec<Color> {
        match self {
            ColorScheme::Office => vec![
                Color::rgb(79, 129, 189),   // Blue
                Color::rgb(192, 80, 77),    // Red
                Color::rgb(155, 187, 89),   // Green
                Color::rgb(128, 100, 162),  // Purple
                Color::rgb(75, 172, 198),   // Teal
                Color::rgb(247, 150, 70),   // Orange
                Color::rgb(119, 146, 60),   // Olive
                Color::rgb(166, 166, 166),  // Gray
            ],
            ColorScheme::Colorful => vec![
                Color::rgb(255, 99, 132),   // Pink-Red
                Color::rgb(54, 162, 235),   // Blue
                Color::rgb(255, 206, 86),   // Yellow
                Color::rgb(75, 192, 192),   // Teal
                Color::rgb(153, 102, 255),  // Purple
                Color::rgb(255, 159, 64),   // Orange
                Color::rgb(199, 199, 199),  // Gray
                Color::rgb(83, 102, 255),   // Indigo
            ],
            ColorScheme::MonochromeBlue => vec![
                Color::rgb(8, 48, 107),     // Very dark blue
                Color::rgb(8, 81, 156),     // Dark blue
                Color::rgb(33, 113, 181),   // Medium-dark blue
                Color::rgb(66, 146, 198),   // Medium blue
                Color::rgb(107, 174, 214),  // Light-medium blue
                Color::rgb(158, 202, 225),  // Light blue
                Color::rgb(198, 219, 239),  // Very light blue
                Color::rgb(222, 235, 247),  // Pale blue
            ],
            ColorScheme::MonochromeGreen => vec![
                Color::rgb(0, 68, 27),      // Very dark green
                Color::rgb(0, 109, 44),     // Dark green
                Color::rgb(35, 139, 69),    // Medium-dark green
                Color::rgb(65, 171, 93),    // Medium green
                Color::rgb(116, 196, 118),  // Light-medium green
                Color::rgb(161, 217, 155),  // Light green
                Color::rgb(199, 233, 192),  // Very light green
                Color::rgb(229, 245, 224),  // Pale green
            ],
            ColorScheme::MonochromeGray => vec![
                Color::rgb(37, 37, 37),     // Very dark gray
                Color::rgb(82, 82, 82),     // Dark gray
                Color::rgb(115, 115, 115),  // Medium-dark gray
                Color::rgb(150, 150, 150),  // Medium gray
                Color::rgb(189, 189, 189),  // Light-medium gray
                Color::rgb(217, 217, 217),  // Light gray
                Color::rgb(240, 240, 240),  // Very light gray
                Color::rgb(250, 250, 250),  // Almost white
            ],
            ColorScheme::Warm => vec![
                Color::rgb(178, 24, 43),    // Dark red
                Color::rgb(214, 96, 77),    // Medium red
                Color::rgb(244, 165, 130),  // Light red/salmon
                Color::rgb(253, 219, 199),  // Pale orange
                Color::rgb(254, 224, 144),  // Light yellow
                Color::rgb(224, 130, 20),   // Orange
                Color::rgb(179, 88, 6),     // Dark orange
                Color::rgb(127, 59, 8),     // Brown
            ],
            ColorScheme::Cool => vec![
                Color::rgb(69, 117, 180),   // Blue
                Color::rgb(116, 173, 209),  // Light blue
                Color::rgb(171, 217, 233),  // Pale blue
                Color::rgb(145, 191, 219),  // Medium blue
                Color::rgb(69, 137, 163),   // Teal
                Color::rgb(48, 104, 141),   // Dark teal
                Color::rgb(116, 169, 207),  // Sky blue
                Color::rgb(84, 158, 175),   // Cyan
            ],
            ColorScheme::HighContrast => vec![
                Color::rgb(0, 0, 0),        // Black
                Color::rgb(255, 255, 0),    // Yellow
                Color::rgb(0, 255, 255),    // Cyan
                Color::rgb(0, 255, 0),      // Green
                Color::rgb(255, 0, 255),    // Magenta
                Color::rgb(255, 0, 0),      // Red
                Color::rgb(0, 0, 255),      // Blue
                Color::rgb(255, 255, 255),  // White
            ],
            ColorScheme::Pastel => vec![
                Color::rgb(174, 198, 207),  // Pastel blue
                Color::rgb(255, 179, 186),  // Pastel pink
                Color::rgb(255, 223, 186),  // Pastel orange
                Color::rgb(255, 255, 186),  // Pastel yellow
                Color::rgb(186, 255, 201),  // Pastel green
                Color::rgb(186, 225, 255),  // Pastel sky blue
                Color::rgb(218, 186, 255),  // Pastel lavender
                Color::rgb(255, 186, 255),  // Pastel magenta
            ],
            ColorScheme::Earth => vec![
                Color::rgb(139, 90, 43),    // Saddle brown
                Color::rgb(160, 82, 45),    // Sienna
                Color::rgb(188, 143, 143),  // Rosy brown
                Color::rgb(128, 128, 0),    // Olive
                Color::rgb(85, 107, 47),    // Dark olive green
                Color::rgb(107, 142, 35),   // Olive drab
                Color::rgb(154, 205, 50),   // Yellow green
                Color::rgb(218, 165, 32),   // Goldenrod
            ],
            ColorScheme::Corporate => vec![
                Color::rgb(0, 63, 92),      // Dark navy
                Color::rgb(47, 75, 124),    // Navy blue
                Color::rgb(102, 81, 145),   // Purple
                Color::rgb(160, 81, 149),   // Mauve
                Color::rgb(212, 80, 135),   // Pink
                Color::rgb(249, 93, 106),   // Coral
                Color::rgb(255, 124, 67),   // Orange
                Color::rgb(255, 166, 0),    // Amber
            ],
            ColorScheme::Ocean => vec![
                Color::rgb(2, 62, 138),     // Deep blue
                Color::rgb(3, 78, 162),     // Navy
                Color::rgb(0, 119, 182),    // Ocean blue
                Color::rgb(0, 150, 199),    // Teal
                Color::rgb(0, 180, 216),    // Light teal
                Color::rgb(72, 202, 228),   // Sky
                Color::rgb(144, 224, 239),  // Light sky
                Color::rgb(202, 240, 248),  // Pale blue
            ],
            ColorScheme::Sunset => vec![
                Color::rgb(255, 89, 94),    // Coral red
                Color::rgb(255, 146, 76),   // Orange
                Color::rgb(255, 186, 73),   // Golden
                Color::rgb(255, 217, 61),   // Yellow
                Color::rgb(198, 169, 75),   // Olive
                Color::rgb(138, 138, 97),   // Sage
                Color::rgb(106, 76, 147),   // Purple
                Color::rgb(89, 54, 135),    // Deep purple
            ],
            ColorScheme::Forest => vec![
                Color::rgb(27, 94, 32),     // Dark forest green
                Color::rgb(46, 125, 50),    // Forest green
                Color::rgb(67, 160, 71),    // Green
                Color::rgb(102, 187, 106),  // Light green
                Color::rgb(129, 199, 132),  // Pale green
                Color::rgb(139, 195, 74),   // Lime
                Color::rgb(104, 159, 56),   // Olive green
                Color::rgb(85, 139, 47),    // Dark olive
            ],
        }
    }

    /// Get a color at a specific index (cycles through colors)
    pub fn color_at(&self, index: usize) -> Color {
        let colors = self.colors();
        colors[index % colors.len()]
    }

    /// Get all available color schemes
    pub fn all() -> Vec<ColorScheme> {
        vec![
            ColorScheme::Office,
            ColorScheme::Colorful,
            ColorScheme::MonochromeBlue,
            ColorScheme::MonochromeGreen,
            ColorScheme::MonochromeGray,
            ColorScheme::Warm,
            ColorScheme::Cool,
            ColorScheme::HighContrast,
            ColorScheme::Pastel,
            ColorScheme::Earth,
            ColorScheme::Corporate,
            ColorScheme::Ocean,
            ColorScheme::Sunset,
            ColorScheme::Forest,
        ]
    }

    /// Get the name of this color scheme
    pub fn name(&self) -> &'static str {
        match self {
            ColorScheme::Office => "Office",
            ColorScheme::Colorful => "Colorful",
            ColorScheme::MonochromeBlue => "Monochrome Blue",
            ColorScheme::MonochromeGreen => "Monochrome Green",
            ColorScheme::MonochromeGray => "Monochrome Gray",
            ColorScheme::Warm => "Warm",
            ColorScheme::Cool => "Cool",
            ColorScheme::HighContrast => "High Contrast",
            ColorScheme::Pastel => "Pastel",
            ColorScheme::Earth => "Earth",
            ColorScheme::Corporate => "Corporate",
            ColorScheme::Ocean => "Ocean",
            ColorScheme::Sunset => "Sunset",
            ColorScheme::Forest => "Forest",
        }
    }
}

/// Predefined style presets for charts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartStylePreset {
    /// Modern flat design with clean lines
    Modern,
    /// Classic professional look
    Classic,
    /// Minimal design with reduced visual elements
    Minimal,
    /// Dark theme
    Dark,
    /// Light theme with subtle colors
    Light,
    /// Bold design with strong colors and shadows
    Bold,
    /// Vintage/retro style
    Vintage,
    /// Technical/blueprint style
    Technical,
}

impl Default for ChartStylePreset {
    fn default() -> Self {
        ChartStylePreset::Modern
    }
}

impl ChartStylePreset {
    /// Convert this preset to a ChartStyle
    pub fn to_style(&self) -> ChartStyle {
        match self {
            ChartStylePreset::Modern => ChartStyle {
                colors: ColorScheme::Colorful.colors(),
                background: Some(Color::WHITE),
                border: None,
                plot_area_background: None,
            },
            ChartStylePreset::Classic => ChartStyle {
                colors: ColorScheme::Office.colors(),
                background: Some(Color::WHITE),
                border: Some(BorderStyle {
                    color: Color::rgb(200, 200, 200),
                    width: 1.0,
                    style: LineStyle::Solid,
                }),
                plot_area_background: Some(Color::rgb(250, 250, 250)),
            },
            ChartStylePreset::Minimal => ChartStyle {
                colors: ColorScheme::MonochromeGray.colors(),
                background: Some(Color::WHITE),
                border: None,
                plot_area_background: None,
            },
            ChartStylePreset::Dark => ChartStyle {
                colors: vec![
                    Color::rgb(102, 187, 255),  // Light blue
                    Color::rgb(255, 107, 107),  // Light red
                    Color::rgb(107, 203, 119),  // Light green
                    Color::rgb(255, 193, 7),    // Yellow
                    Color::rgb(156, 106, 222),  // Light purple
                    Color::rgb(255, 138, 101),  // Light orange
                    Color::rgb(77, 182, 172),   // Teal
                    Color::rgb(189, 189, 189),  // Light gray
                ],
                background: Some(Color::rgb(33, 33, 33)),
                border: Some(BorderStyle {
                    color: Color::rgb(66, 66, 66),
                    width: 1.0,
                    style: LineStyle::Solid,
                }),
                plot_area_background: Some(Color::rgb(44, 44, 44)),
            },
            ChartStylePreset::Light => ChartStyle {
                colors: ColorScheme::Pastel.colors(),
                background: Some(Color::rgb(252, 252, 252)),
                border: Some(BorderStyle {
                    color: Color::rgb(230, 230, 230),
                    width: 1.0,
                    style: LineStyle::Solid,
                }),
                plot_area_background: Some(Color::WHITE),
            },
            ChartStylePreset::Bold => ChartStyle {
                colors: vec![
                    Color::rgb(255, 0, 0),      // Red
                    Color::rgb(0, 0, 255),      // Blue
                    Color::rgb(0, 200, 0),      // Green
                    Color::rgb(255, 200, 0),    // Gold
                    Color::rgb(150, 0, 150),    // Purple
                    Color::rgb(255, 100, 0),    // Orange
                    Color::rgb(0, 150, 150),    // Teal
                    Color::rgb(100, 100, 100),  // Gray
                ],
                background: Some(Color::WHITE),
                border: Some(BorderStyle {
                    color: Color::BLACK,
                    width: 2.0,
                    style: LineStyle::Solid,
                }),
                plot_area_background: Some(Color::rgb(245, 245, 245)),
            },
            ChartStylePreset::Vintage => ChartStyle {
                colors: vec![
                    Color::rgb(139, 69, 19),    // Saddle brown
                    Color::rgb(160, 82, 45),    // Sienna
                    Color::rgb(210, 180, 140),  // Tan
                    Color::rgb(139, 90, 43),    // Brown
                    Color::rgb(85, 107, 47),    // Dark olive green
                    Color::rgb(128, 128, 0),    // Olive
                    Color::rgb(165, 42, 42),    // Brown-red
                    Color::rgb(128, 128, 128),  // Gray
                ],
                background: Some(Color::rgb(253, 245, 230)),  // Old lace
                border: Some(BorderStyle {
                    color: Color::rgb(139, 69, 19),
                    width: 2.0,
                    style: LineStyle::Solid,
                }),
                plot_area_background: Some(Color::rgb(250, 240, 230)),
            },
            ChartStylePreset::Technical => ChartStyle {
                colors: vec![
                    Color::rgb(0, 120, 215),    // Blue
                    Color::rgb(16, 137, 62),    // Green
                    Color::rgb(232, 17, 35),    // Red
                    Color::rgb(255, 140, 0),    // Orange
                    Color::rgb(136, 23, 152),   // Purple
                    Color::rgb(0, 178, 238),    // Cyan
                    Color::rgb(118, 118, 118),  // Gray
                    Color::rgb(180, 0, 158),    // Magenta
                ],
                background: Some(Color::rgb(240, 248, 255)),  // Alice blue
                border: Some(BorderStyle {
                    color: Color::rgb(0, 120, 215),
                    width: 1.0,
                    style: LineStyle::Dash,
                }),
                plot_area_background: Some(Color::WHITE),
            },
        }
    }

    /// Get all available style presets
    pub fn all() -> Vec<ChartStylePreset> {
        vec![
            ChartStylePreset::Modern,
            ChartStylePreset::Classic,
            ChartStylePreset::Minimal,
            ChartStylePreset::Dark,
            ChartStylePreset::Light,
            ChartStylePreset::Bold,
            ChartStylePreset::Vintage,
            ChartStylePreset::Technical,
        ]
    }

    /// Get the name of this style preset
    pub fn name(&self) -> &'static str {
        match self {
            ChartStylePreset::Modern => "Modern",
            ChartStylePreset::Classic => "Classic",
            ChartStylePreset::Minimal => "Minimal",
            ChartStylePreset::Dark => "Dark",
            ChartStylePreset::Light => "Light",
            ChartStylePreset::Bold => "Bold",
            ChartStylePreset::Vintage => "Vintage",
            ChartStylePreset::Technical => "Technical",
        }
    }

    /// Get a description of this style preset
    pub fn description(&self) -> &'static str {
        match self {
            ChartStylePreset::Modern => "Clean, flat design with vibrant colors",
            ChartStylePreset::Classic => "Professional look with subtle borders",
            ChartStylePreset::Minimal => "Reduced visual elements, grayscale palette",
            ChartStylePreset::Dark => "Dark background with bright accent colors",
            ChartStylePreset::Light => "Soft pastel colors on light background",
            ChartStylePreset::Bold => "Strong colors with prominent borders",
            ChartStylePreset::Vintage => "Warm, earthy tones with classic feel",
            ChartStylePreset::Technical => "Blueprint-inspired with technical aesthetics",
        }
    }
}

/// Utilities for working with chart styles
pub struct StyleUtils;

impl StyleUtils {
    /// Apply a color scheme to a chart
    pub fn apply_color_scheme(chart: &mut Chart, scheme: ColorScheme) {
        chart.style.colors = scheme.colors();
    }

    /// Apply a style preset to a chart
    pub fn apply_preset(chart: &mut Chart, preset: ChartStylePreset) {
        chart.style = preset.to_style();
    }

    /// Get a contrasting text color for a background
    pub fn get_text_color(background: Color) -> Color {
        // Calculate relative luminance
        let luminance = 0.299 * background.r as f64
            + 0.587 * background.g as f64
            + 0.114 * background.b as f64;

        if luminance > 186.0 {
            Color::rgb(33, 33, 33)  // Dark text for light backgrounds
        } else {
            Color::rgb(245, 245, 245)  // Light text for dark backgrounds
        }
    }

    /// Lighten a color by a percentage (0.0 to 1.0)
    pub fn lighten(color: Color, amount: f64) -> Color {
        let amount = amount.clamp(0.0, 1.0);
        Color::rgba(
            (color.r as f64 + (255.0 - color.r as f64) * amount) as u8,
            (color.g as f64 + (255.0 - color.g as f64) * amount) as u8,
            (color.b as f64 + (255.0 - color.b as f64) * amount) as u8,
            color.a,
        )
    }

    /// Darken a color by a percentage (0.0 to 1.0)
    pub fn darken(color: Color, amount: f64) -> Color {
        let amount = amount.clamp(0.0, 1.0);
        Color::rgba(
            (color.r as f64 * (1.0 - amount)) as u8,
            (color.g as f64 * (1.0 - amount)) as u8,
            (color.b as f64 * (1.0 - amount)) as u8,
            color.a,
        )
    }

    /// Create a gradient of colors between two colors
    pub fn gradient(start: Color, end: Color, steps: usize) -> Vec<Color> {
        if steps <= 1 {
            return vec![start];
        }

        (0..steps)
            .map(|i| {
                let t = i as f64 / (steps - 1) as f64;
                Color::rgba(
                    (start.r as f64 + (end.r as f64 - start.r as f64) * t) as u8,
                    (start.g as f64 + (end.g as f64 - start.g as f64) * t) as u8,
                    (start.b as f64 + (end.b as f64 - start.b as f64) * t) as u8,
                    (start.a as f64 + (end.a as f64 - start.a as f64) * t) as u8,
                )
            })
            .collect()
    }

    /// Generate a monochrome palette from a base color
    pub fn monochrome_palette(base: Color, count: usize) -> Vec<Color> {
        let darkest = Self::darken(base, 0.6);
        let lightest = Self::lighten(base, 0.6);
        Self::gradient(darkest, lightest, count)
    }

    /// Adjust the saturation of a color (0.0 = grayscale, 1.0 = original, >1.0 = more saturated)
    pub fn adjust_saturation(color: Color, factor: f64) -> Color {
        let gray = 0.299 * color.r as f64 + 0.587 * color.g as f64 + 0.114 * color.b as f64;
        Color::rgba(
            (gray + (color.r as f64 - gray) * factor).clamp(0.0, 255.0) as u8,
            (gray + (color.g as f64 - gray) * factor).clamp(0.0, 255.0) as u8,
            (gray + (color.b as f64 - gray) * factor).clamp(0.0, 255.0) as u8,
            color.a,
        )
    }

    /// Create complementary colors
    pub fn complementary(color: Color) -> Color {
        Color::rgba(
            255 - color.r,
            255 - color.g,
            255 - color.b,
            color.a,
        )
    }

    /// Blend two colors together
    pub fn blend(color1: Color, color2: Color, ratio: f64) -> Color {
        let ratio = ratio.clamp(0.0, 1.0);
        Color::rgba(
            (color1.r as f64 * (1.0 - ratio) + color2.r as f64 * ratio) as u8,
            (color1.g as f64 * (1.0 - ratio) + color2.g as f64 * ratio) as u8,
            (color1.b as f64 * (1.0 - ratio) + color2.b as f64 * ratio) as u8,
            (color1.a as f64 * (1.0 - ratio) + color2.a as f64 * ratio) as u8,
        )
    }
}

/// Builder for creating custom chart styles
#[derive(Debug, Clone)]
pub struct ChartStyleBuilder {
    style: ChartStyle,
}

impl Default for ChartStyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartStyleBuilder {
    /// Create a new style builder with default values
    pub fn new() -> Self {
        Self {
            style: ChartStyle::default(),
        }
    }

    /// Start from a preset
    pub fn from_preset(preset: ChartStylePreset) -> Self {
        Self {
            style: preset.to_style(),
        }
    }

    /// Set the color scheme
    pub fn color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.style.colors = scheme.colors();
        self
    }

    /// Set specific colors
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.style.colors = colors;
        self
    }

    /// Set the background color
    pub fn background(mut self, color: Option<Color>) -> Self {
        self.style.background = color;
        self
    }

    /// Set the plot area background
    pub fn plot_area_background(mut self, color: Option<Color>) -> Self {
        self.style.plot_area_background = color;
        self
    }

    /// Set the border style
    pub fn border(mut self, border: Option<BorderStyle>) -> Self {
        self.style.border = border;
        self
    }

    /// Add a solid border
    pub fn solid_border(mut self, color: Color, width: f32) -> Self {
        self.style.border = Some(BorderStyle {
            color,
            width,
            style: LineStyle::Solid,
        });
        self
    }

    /// Add a dashed border
    pub fn dashed_border(mut self, color: Color, width: f32) -> Self {
        self.style.border = Some(BorderStyle {
            color,
            width,
            style: LineStyle::Dash,
        });
        self
    }

    /// Build the final style
    pub fn build(self) -> ChartStyle {
        self.style
    }

    /// Apply this style to a chart
    pub fn apply_to(self, chart: &mut Chart) {
        chart.style = self.build();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_colors() {
        let scheme = ColorScheme::Office;
        let colors = scheme.colors();

        assert_eq!(colors.len(), 8);
        assert_eq!(colors[0], Color::rgb(79, 129, 189));
    }

    #[test]
    fn test_color_scheme_color_at() {
        let scheme = ColorScheme::Colorful;
        let colors = scheme.colors();

        // Normal index
        assert_eq!(scheme.color_at(0), colors[0]);
        assert_eq!(scheme.color_at(3), colors[3]);

        // Wrapping
        assert_eq!(scheme.color_at(8), colors[0]);
        assert_eq!(scheme.color_at(10), colors[2]);
    }

    #[test]
    fn test_color_scheme_all() {
        let all = ColorScheme::all();
        assert!(all.len() >= 10);
        assert!(all.contains(&ColorScheme::Office));
    }

    #[test]
    fn test_style_preset_to_style() {
        let preset = ChartStylePreset::Modern;
        let style = preset.to_style();

        assert!(style.background.is_some());
        assert!(!style.colors.is_empty());
    }

    #[test]
    fn test_dark_preset() {
        let style = ChartStylePreset::Dark.to_style();

        assert!(style.background.is_some());
        let bg = style.background.unwrap();
        // Dark preset should have a dark background
        assert!(bg.r < 100 && bg.g < 100 && bg.b < 100);
    }

    #[test]
    fn test_style_utils_lighten() {
        let color = Color::rgb(100, 100, 100);
        let lighter = StyleUtils::lighten(color, 0.5);

        assert!(lighter.r > color.r);
        assert!(lighter.g > color.g);
        assert!(lighter.b > color.b);
    }

    #[test]
    fn test_style_utils_darken() {
        let color = Color::rgb(200, 200, 200);
        let darker = StyleUtils::darken(color, 0.5);

        assert!(darker.r < color.r);
        assert!(darker.g < color.g);
        assert!(darker.b < color.b);
    }

    #[test]
    fn test_style_utils_gradient() {
        let start = Color::rgb(0, 0, 0);
        let end = Color::rgb(255, 255, 255);
        let gradient = StyleUtils::gradient(start, end, 5);

        assert_eq!(gradient.len(), 5);
        assert_eq!(gradient[0], start);
        assert_eq!(gradient[4], end);
        // Middle should be gray-ish
        assert!(gradient[2].r > 100 && gradient[2].r < 155);
    }

    #[test]
    fn test_style_utils_get_text_color() {
        // Dark background should get light text
        let light_text = StyleUtils::get_text_color(Color::rgb(0, 0, 0));
        assert!(light_text.r > 200);

        // Light background should get dark text
        let dark_text = StyleUtils::get_text_color(Color::rgb(255, 255, 255));
        assert!(dark_text.r < 100);
    }

    #[test]
    fn test_style_utils_complementary() {
        let color = Color::rgb(255, 0, 0);  // Red
        let comp = StyleUtils::complementary(color);

        assert_eq!(comp, Color::rgb(0, 255, 255));  // Cyan
    }

    #[test]
    fn test_style_utils_blend() {
        let red = Color::rgb(255, 0, 0);
        let blue = Color::rgb(0, 0, 255);
        let blend = StyleUtils::blend(red, blue, 0.5);

        // Should be purple-ish
        assert!(blend.r > 100 && blend.r < 150);
        assert!(blend.b > 100 && blend.b < 150);
    }

    #[test]
    fn test_style_utils_monochrome_palette() {
        let base = Color::rgb(0, 100, 200);
        let palette = StyleUtils::monochrome_palette(base, 5);

        assert_eq!(palette.len(), 5);
        // First should be darker, last should be lighter
        assert!(palette[0].r < palette[4].r || palette[0].g < palette[4].g || palette[0].b < palette[4].b);
    }

    #[test]
    fn test_chart_style_builder() {
        let style = ChartStyleBuilder::new()
            .color_scheme(ColorScheme::Ocean)
            .background(Some(Color::WHITE))
            .solid_border(Color::rgb(0, 100, 200), 2.0)
            .build();

        assert_eq!(style.colors, ColorScheme::Ocean.colors());
        assert_eq!(style.background, Some(Color::WHITE));
        assert!(style.border.is_some());
        assert_eq!(style.border.as_ref().unwrap().width, 2.0);
    }

    #[test]
    fn test_chart_style_builder_from_preset() {
        let style = ChartStyleBuilder::from_preset(ChartStylePreset::Dark)
            .color_scheme(ColorScheme::Sunset)
            .build();

        // Should have Sunset colors but Dark background
        assert_eq!(style.colors, ColorScheme::Sunset.colors());
        assert!(style.background.is_some());
        let bg = style.background.unwrap();
        assert!(bg.r < 100);  // Still dark background
    }

    #[test]
    fn test_apply_color_scheme_to_chart() {
        let mut chart = Chart::new("test", ChartType::default());
        StyleUtils::apply_color_scheme(&mut chart, ColorScheme::Forest);

        assert_eq!(chart.style.colors, ColorScheme::Forest.colors());
    }

    #[test]
    fn test_apply_preset_to_chart() {
        let mut chart = Chart::new("test", ChartType::default());
        StyleUtils::apply_preset(&mut chart, ChartStylePreset::Vintage);

        let expected_style = ChartStylePreset::Vintage.to_style();
        assert_eq!(chart.style.colors, expected_style.colors);
        assert_eq!(chart.style.background, expected_style.background);
    }

    #[test]
    fn test_style_preset_names_and_descriptions() {
        for preset in ChartStylePreset::all() {
            assert!(!preset.name().is_empty());
            assert!(!preset.description().is_empty());
        }
    }

    #[test]
    fn test_color_scheme_names() {
        for scheme in ColorScheme::all() {
            assert!(!scheme.name().is_empty());
        }
    }

    #[test]
    fn test_adjust_saturation() {
        let color = Color::rgb(255, 100, 100);

        // Desaturate
        let desaturated = StyleUtils::adjust_saturation(color, 0.0);
        assert_eq!(desaturated.r, desaturated.g);
        assert_eq!(desaturated.g, desaturated.b);

        // Keep original
        let original = StyleUtils::adjust_saturation(color, 1.0);
        assert_eq!(original, color);
    }
}
