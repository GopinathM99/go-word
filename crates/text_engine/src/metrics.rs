//! Text metrics calculations

use crate::FontMetrics;

/// Calculate line height from font metrics and font size
pub fn calculate_line_height(metrics: &FontMetrics, font_size: f32, line_spacing: f32) -> f32 {
    let em = font_size;
    let ascender = metrics.ascender as f32 / metrics.units_per_em as f32 * em;
    let descender = metrics.descender.abs() as f32 / metrics.units_per_em as f32 * em;
    let line_gap = metrics.line_gap as f32 / metrics.units_per_em as f32 * em;

    (ascender + descender + line_gap) * line_spacing
}

/// Calculate baseline offset from top of line
pub fn calculate_baseline_offset(metrics: &FontMetrics, font_size: f32) -> f32 {
    let em = font_size;
    metrics.ascender as f32 / metrics.units_per_em as f32 * em
}

/// Calculate text width (simple approximation)
pub fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    // Simple approximation: average character width is ~60% of em
    text.chars().count() as f32 * font_size * 0.6
}
