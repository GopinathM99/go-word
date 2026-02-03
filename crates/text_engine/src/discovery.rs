//! Font Discovery Module
//!
//! This module handles system font discovery using font-kit.
//! It enumerates available fonts on the system and builds an index
//! for fast font lookup by family name.

use crate::{FontStyle, FontWeight, Result, TextError};
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Information about a discovered font
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Font family name
    pub family: String,
    /// PostScript name (unique identifier)
    pub postscript_name: Option<String>,
    /// Font file path
    pub path: Option<PathBuf>,
    /// Font index within the file (for TTC files)
    pub font_index: u32,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Whether this font supports the basic Latin character set
    pub supports_latin: bool,
    /// Whether this font supports CJK characters
    pub supports_cjk: bool,
}

/// Font variant key for indexing
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontVariantKey {
    pub family: String,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl FontVariantKey {
    pub fn new(family: impl Into<String>, weight: FontWeight, style: FontStyle) -> Self {
        Self {
            family: family.into(),
            weight,
            style,
        }
    }
}

/// Index of discovered system fonts
#[derive(Debug, Default)]
pub struct FontIndex {
    /// Map from lowercase family name to list of available variants
    families: HashMap<String, Vec<FontInfo>>,
    /// Map from variant key to font info
    variants: HashMap<FontVariantKey, FontInfo>,
    /// List of all discovered font families
    family_names: Vec<String>,
}

impl FontIndex {
    /// Create a new empty font index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a font to the index
    pub fn add_font(&mut self, info: FontInfo) {
        let family_lower = info.family.to_lowercase();

        // Add to family list if new
        if !self.families.contains_key(&family_lower) {
            self.family_names.push(info.family.clone());
        }

        // Add to families map
        self.families
            .entry(family_lower)
            .or_default()
            .push(info.clone());

        // Add to variants map
        let key = FontVariantKey::new(&info.family, info.weight, info.style);
        self.variants.insert(key, info);
    }

    /// Check if a font family is available
    pub fn has_family(&self, family: &str) -> bool {
        self.families.contains_key(&family.to_lowercase())
    }

    /// Get all variants of a font family
    pub fn get_family(&self, family: &str) -> Option<&Vec<FontInfo>> {
        self.families.get(&family.to_lowercase())
    }

    /// Get a specific font variant
    pub fn get_variant(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&FontInfo> {
        let key = FontVariantKey::new(family, weight, style);
        self.variants.get(&key)
    }

    /// Find the best matching variant for a font request
    pub fn find_best_match(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&FontInfo> {
        // First try exact match
        if let Some(info) = self.get_variant(family, weight, style) {
            return Some(info);
        }

        // Get family variants
        let variants = self.get_family(family)?;

        // Try to find closest match
        // Priority: exact style > any weight with same style > any variant
        let mut best_match: Option<&FontInfo> = None;
        let mut best_score = 0;

        for variant in variants {
            let mut score = 0;

            // Style match is more important
            if variant.style == style {
                score += 10;
            }

            // Weight match
            if variant.weight == weight {
                score += 5;
            } else if variant.weight == FontWeight::Normal && weight == FontWeight::Bold {
                // Prefer normal over nothing when looking for bold
                score += 2;
            } else if variant.weight == FontWeight::Bold && weight == FontWeight::Normal {
                // Bold can work for normal
                score += 1;
            }

            if score > best_score || best_match.is_none() {
                best_score = score;
                best_match = Some(variant);
            }
        }

        best_match
    }

    /// Get list of all available font families
    pub fn list_families(&self) -> &[String] {
        &self.family_names
    }

    /// Get total number of font families
    pub fn family_count(&self) -> usize {
        self.families.len()
    }

    /// Get total number of font variants
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }
}

/// Convert font-kit Weight to our FontWeight
fn convert_weight(weight: Weight) -> FontWeight {
    if weight.0 >= 600.0 {
        FontWeight::Bold
    } else {
        FontWeight::Normal
    }
}

/// Convert font-kit Style to our FontStyle
fn convert_style(style: Style) -> FontStyle {
    match style {
        Style::Italic | Style::Oblique => FontStyle::Italic,
        Style::Normal => FontStyle::Normal,
    }
}

/// Font discovery service for enumerating system fonts
pub struct FontDiscovery {
    /// System font source
    source: SystemSource,
    /// Cached font index
    index: Arc<RwLock<Option<FontIndex>>>,
}

impl FontDiscovery {
    /// Create a new font discovery service
    pub fn new() -> Self {
        Self {
            source: SystemSource::new(),
            index: Arc::new(RwLock::new(None)),
        }
    }

    /// Discover all system fonts and build an index
    /// This can be slow on first call; results are cached.
    pub fn discover_fonts(&self) -> Result<FontIndex> {
        // Check cache first
        {
            let cache = self.index.read().unwrap();
            if let Some(ref index) = *cache {
                return Ok(index.clone());
            }
        }

        // Build new index
        let mut index = FontIndex::new();

        // Get all font families from system
        let families = self
            .source
            .all_families()
            .map_err(|e| TextError::FontNotFound(format!("Failed to enumerate fonts: {}", e)))?;

        for family_name in families {
            // Try to get handles for this family
            if let Ok(family) = self.source.select_family_by_name(&family_name) {
                for handle in family.fonts() {
                    if let Ok(font) = handle.load() {
                        let properties = font.properties();

                        let (path, font_index) = match handle {
                            Handle::Path { path, font_index } => {
                                (Some(path.clone()), *font_index)
                            }
                            Handle::Memory { .. } => (None, 0),
                        };

                        let info = FontInfo {
                            family: family_name.clone(),
                            postscript_name: font.postscript_name(),
                            path,
                            font_index,
                            weight: convert_weight(properties.weight),
                            style: convert_style(properties.style),
                            supports_latin: true, // Assume most fonts support Latin
                            supports_cjk: family_name.contains("CJK")
                                || family_name.contains("Gothic")
                                || family_name.contains("Mincho")
                                || family_name.contains("Ming")
                                || family_name.contains("Song")
                                || family_name.contains("Hei"),
                        };

                        index.add_font(info);
                    }
                }
            }
        }

        // Cache the result
        {
            let mut cache = self.index.write().unwrap();
            *cache = Some(index.clone());
        }

        Ok(index)
    }

    /// Clear the font cache (useful when system fonts change)
    pub fn clear_cache(&self) {
        let mut cache = self.index.write().unwrap();
        *cache = None;
    }

    /// Load font data from a font info
    pub fn load_font_data(&self, info: &FontInfo) -> Result<Vec<u8>> {
        if let Some(ref path) = info.path {
            std::fs::read(path)
                .map_err(|e| TextError::FontNotFound(format!("Failed to read font file: {}", e)))
        } else {
            Err(TextError::FontNotFound(
                "Font has no file path".to_string(),
            ))
        }
    }

    /// Select a font by family name and properties using font-kit
    pub fn select_font(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Result<FontInfo> {
        let properties = Properties {
            weight: match weight {
                FontWeight::Bold => Weight::BOLD,
                FontWeight::Normal => Weight::NORMAL,
            },
            style: match style {
                FontStyle::Italic => Style::Italic,
                FontStyle::Normal => Style::Normal,
            },
            ..Default::default()
        };

        let family_name = match family.to_lowercase().as_str() {
            "sans-serif" => FamilyName::SansSerif,
            "serif" => FamilyName::Serif,
            "monospace" => FamilyName::Monospace,
            "cursive" => FamilyName::Cursive,
            "fantasy" => FamilyName::Fantasy,
            _ => FamilyName::Title(family.to_string()),
        };

        let handle = self
            .source
            .select_best_match(&[family_name], &properties)
            .map_err(|_| TextError::FontNotFound(format!("Font not found: {}", family)))?;

        let font = handle
            .load()
            .map_err(|e| TextError::InvalidFontData(format!("Failed to load font: {}", e)))?;

        let (path, font_index) = match &handle {
            Handle::Path { path, font_index } => (Some(path.clone()), *font_index),
            Handle::Memory { .. } => (None, 0),
        };

        Ok(FontInfo {
            family: font.family_name(),
            postscript_name: font.postscript_name(),
            path,
            font_index,
            weight,
            style,
            supports_latin: true,
            supports_cjk: false,
        })
    }
}

impl Default for FontDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FontIndex {
    fn clone(&self) -> Self {
        Self {
            families: self.families.clone(),
            variants: self.variants.clone(),
            family_names: self.family_names.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_index_operations() {
        let mut index = FontIndex::new();

        let info = FontInfo {
            family: "Test Font".to_string(),
            postscript_name: Some("TestFont-Regular".to_string()),
            path: None,
            font_index: 0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            supports_latin: true,
            supports_cjk: false,
        };

        index.add_font(info.clone());

        assert!(index.has_family("Test Font"));
        assert!(index.has_family("test font")); // Case insensitive
        assert!(!index.has_family("Unknown Font"));

        let found = index.get_variant("Test Font", FontWeight::Normal, FontStyle::Normal);
        assert!(found.is_some());
        assert_eq!(found.unwrap().family, "Test Font");
    }

    #[test]
    fn test_font_discovery_finds_fonts() {
        let discovery = FontDiscovery::new();
        let index = discovery.discover_fonts();

        // Should find at least some fonts on any system
        assert!(index.is_ok());
        let index = index.unwrap();
        assert!(index.family_count() > 0);
    }

    #[test]
    fn test_select_generic_font() {
        let discovery = FontDiscovery::new();

        // Should be able to select a generic sans-serif font
        let result = discovery.select_font("sans-serif", FontWeight::Normal, FontStyle::Normal);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.family.is_empty());
    }
}
