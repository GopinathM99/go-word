//! Font discovery and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
}

/// Font identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontId {
    pub family: String,
    pub style: FontStyle,
    pub weight: FontWeight,
}

impl FontId {
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            style: FontStyle::Normal,
            weight: FontWeight::Normal,
        }
    }

    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }
}

/// Font metrics
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Units per em
    pub units_per_em: u16,
    /// Ascender (positive)
    pub ascender: i16,
    /// Descender (negative)
    pub descender: i16,
    /// Line gap
    pub line_gap: i16,
    /// Cap height
    pub cap_height: Option<i16>,
    /// x-height
    pub x_height: Option<i16>,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            units_per_em: 1000,
            ascender: 800,
            descender: -200,
            line_gap: 0,
            cap_height: Some(700),
            x_height: Some(500),
        }
    }
}

/// Simple font metrics cache (legacy, prefer font_manager::FontManager)
#[deprecated(note = "Use font_manager::FontManager instead for full font management")]
pub struct SimpleFontCache {
    /// Default font family
    default_family: String,
    /// Font substitution map
    substitutions: HashMap<String, String>,
    /// Cached font metrics
    metrics_cache: HashMap<FontId, FontMetrics>,
}

#[allow(deprecated)]
impl SimpleFontCache {
    /// Create a new font cache
    pub fn new() -> Self {
        let mut substitutions = HashMap::new();
        // Add some common substitutions
        substitutions.insert("Times New Roman".into(), "Times".into());
        substitutions.insert("Arial".into(), "Helvetica".into());

        Self {
            default_family: "sans-serif".into(),
            substitutions,
            metrics_cache: HashMap::new(),
        }
    }

    /// Get the default font family
    pub fn default_family(&self) -> &str {
        &self.default_family
    }

    /// Set the default font family
    pub fn set_default_family(&mut self, family: impl Into<String>) {
        self.default_family = family.into();
    }

    /// Get font metrics for a font ID
    pub fn get_metrics(&mut self, font_id: &FontId) -> FontMetrics {
        if let Some(metrics) = self.metrics_cache.get(font_id) {
            return metrics.clone();
        }

        // Return default metrics
        let metrics = FontMetrics::default();
        self.metrics_cache.insert(font_id.clone(), metrics.clone());
        metrics
    }

    /// Resolve a font family to an available font
    pub fn resolve_family(&self, family: &str) -> String {
        if let Some(substitute) = self.substitutions.get(family) {
            substitute.clone()
        } else {
            family.to_string()
        }
    }
}

#[allow(deprecated)]
impl Default for SimpleFontCache {
    fn default() -> Self {
        Self::new()
    }
}
