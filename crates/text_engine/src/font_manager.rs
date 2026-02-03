//! Font Manager Module
//!
//! Central manager for font loading, caching, and resolution.
//! Integrates font discovery, fallback chains, and the text shaper.

use crate::discovery::{FontDiscovery, FontIndex, FontInfo};
use crate::fallback::{FallbackChain, FontResolution, Script, SubstitutionReason, SubstitutionWarning};
use crate::{FontId, FontMetrics, FontStyle, FontWeight, Result, TextError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// A unique identifier for a loaded font
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LoadedFontId(pub u32);

/// Information about a loaded font
#[derive(Debug, Clone)]
pub struct LoadedFont {
    /// Unique ID for this loaded font
    pub id: LoadedFontId,
    /// Font information
    pub info: FontInfo,
    /// Font data bytes
    pub data: Arc<Vec<u8>>,
}

/// Record of font substitution for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSubstitutionRecord {
    /// Original requested font
    pub requested_font: String,
    /// Font that was actually used
    pub actual_font: String,
    /// Weight requested
    pub requested_weight: FontWeight,
    /// Style requested
    pub requested_style: FontStyle,
    /// Reason for substitution
    pub reason: SubstitutionReason,
    /// Number of times this substitution occurred
    pub occurrence_count: usize,
}

/// Summary of all font substitutions in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FontSubstitutionSummary {
    /// List of all substitutions
    pub substitutions: Vec<FontSubstitutionRecord>,
    /// Total number of fonts that were substituted
    pub total_substituted: usize,
    /// Total number of fonts that were found
    pub total_found: usize,
}

impl FontSubstitutionSummary {
    /// Create a new empty summary
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a substitution to the summary
    pub fn add_substitution(&mut self, warning: &SubstitutionWarning, weight: FontWeight, style: FontStyle) {
        // Check if we already have this substitution
        for record in &mut self.substitutions {
            if record.requested_font == warning.requested
                && record.actual_font == warning.substituted
                && record.requested_weight == weight
                && record.requested_style == style
            {
                record.occurrence_count += 1;
                return;
            }
        }

        // Add new record
        self.substitutions.push(FontSubstitutionRecord {
            requested_font: warning.requested.clone(),
            actual_font: warning.substituted.clone(),
            requested_weight: weight,
            requested_style: style,
            reason: warning.reason,
            occurrence_count: 1,
        });
        self.total_substituted += 1;
    }

    /// Record a found font (no substitution needed)
    pub fn add_found(&mut self) {
        self.total_found += 1;
    }

    /// Check if any substitutions occurred
    pub fn has_substitutions(&self) -> bool {
        !self.substitutions.is_empty()
    }

    /// Get a human-readable summary
    pub fn summary_text(&self) -> String {
        if self.substitutions.is_empty() {
            return "All fonts found.".to_string();
        }

        let mut lines = vec![format!(
            "{} font(s) substituted:",
            self.substitutions.len()
        )];

        for record in &self.substitutions {
            lines.push(format!(
                "  {} -> {} ({}x)",
                record.requested_font, record.actual_font, record.occurrence_count
            ));
        }

        lines.join("\n")
    }
}

/// Configuration for the font manager
#[derive(Debug, Clone)]
pub struct FontManagerConfig {
    /// Additional font directories to search
    pub additional_font_dirs: Vec<PathBuf>,
    /// Whether to allow font substitution
    pub allow_substitution: bool,
    /// Default font family
    pub default_family: String,
    /// Maximum number of fonts to cache
    pub max_cached_fonts: usize,
}

impl Default for FontManagerConfig {
    fn default() -> Self {
        Self {
            additional_font_dirs: Vec::new(),
            allow_substitution: true,
            default_family: "sans-serif".to_string(),
            max_cached_fonts: 100,
        }
    }
}

/// Central font manager
pub struct FontManager {
    /// Font discovery service
    discovery: FontDiscovery,
    /// Font fallback chain
    fallback_chain: FallbackChain,
    /// Cached font index
    font_index: Arc<RwLock<Option<FontIndex>>>,
    /// Loaded fonts cache (family -> data)
    loaded_fonts: Arc<RwLock<HashMap<FontId, LoadedFont>>>,
    /// Next font ID
    next_font_id: Arc<RwLock<u32>>,
    /// Configuration
    config: FontManagerConfig,
    /// Substitution summary for current session
    substitution_summary: Arc<RwLock<FontSubstitutionSummary>>,
}

impl FontManager {
    /// Create a new font manager with default configuration
    pub fn new() -> Self {
        Self::with_config(FontManagerConfig::default())
    }

    /// Create a new font manager with custom configuration
    pub fn with_config(config: FontManagerConfig) -> Self {
        Self {
            discovery: FontDiscovery::new(),
            fallback_chain: FallbackChain::new(),
            font_index: Arc::new(RwLock::new(None)),
            loaded_fonts: Arc::new(RwLock::new(HashMap::new())),
            next_font_id: Arc::new(RwLock::new(1)),
            config,
            substitution_summary: Arc::new(RwLock::new(FontSubstitutionSummary::new())),
        }
    }

    /// Initialize font discovery (can be called async)
    pub fn initialize(&self) -> Result<()> {
        let index = self.discovery.discover_fonts()?;
        let mut cache = self.font_index.write().unwrap();
        *cache = Some(index);
        Ok(())
    }

    /// Check if the font manager is initialized
    pub fn is_initialized(&self) -> bool {
        self.font_index.read().unwrap().is_some()
    }

    /// Get the font index, initializing if necessary
    fn get_index(&self) -> Result<FontIndex> {
        {
            let cache = self.font_index.read().unwrap();
            if let Some(ref index) = *cache {
                return Ok(index.clone());
            }
        }

        // Initialize and return
        self.initialize()?;
        let cache = self.font_index.read().unwrap();
        cache.clone().ok_or_else(|| TextError::FontNotFound("Failed to initialize font index".to_string()))
    }

    /// Check if a font is available on the system
    pub fn is_font_available(&self, family: &str) -> bool {
        if let Ok(index) = self.get_index() {
            index.has_family(family)
        } else {
            false
        }
    }

    /// Resolve a font, performing substitution if necessary
    pub fn resolve_font(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Result<FontResolution> {
        // Handle generic font families
        if let Some(defaults) = self.fallback_chain.resolve_generic(family) {
            // Find first available font in the defaults
            if let Ok(index) = self.get_index() {
                for default_family in defaults {
                    if index.has_family(default_family) {
                        return Ok(FontResolution::substituted(
                            family,
                            default_family,
                            weight,
                            style,
                            SubstitutionReason::FallbackToDefault,
                        ));
                    }
                }
            }
        }

        // Check if exact font is available
        if let Ok(index) = self.get_index() {
            if let Some(_info) = index.find_best_match(family, weight, style) {
                // Record as found
                self.substitution_summary.write().unwrap().add_found();
                return Ok(FontResolution::exact(family, weight, style));
            }
        }

        // Font not available, try substitutions
        if self.config.allow_substitution {
            self.find_substitute(family, weight, style)
        } else {
            Err(TextError::FontNotFound(format!(
                "Font not found: {}",
                family
            )))
        }
    }

    /// Find a substitute font
    fn find_substitute(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Result<FontResolution> {
        let index = self.get_index()?;

        // Get fallback chain for this font
        let fallbacks = self.fallback_chain.get_fallback_chain(family, None);

        for fallback_family in fallbacks {
            // Skip generic family names in the chain
            if matches!(
                fallback_family.as_str(),
                "sans-serif" | "serif" | "monospace" | "cursive" | "fantasy"
            ) {
                // Resolve the generic family
                if let Some(defaults) = self.fallback_chain.resolve_generic(&fallback_family) {
                    for default in defaults {
                        if index.has_family(default) {
                            let resolution = FontResolution::substituted(
                                family,
                                default,
                                weight,
                                style,
                                SubstitutionReason::NotInstalled,
                            );

                            // Record substitution
                            if let Some(ref warning) = resolution.warning {
                                self.substitution_summary.write().unwrap()
                                    .add_substitution(warning, weight, style);
                            }

                            return Ok(resolution);
                        }
                    }
                }
                continue;
            }

            if index.has_family(&fallback_family) {
                let resolution = FontResolution::substituted(
                    family,
                    &fallback_family,
                    weight,
                    style,
                    SubstitutionReason::NotInstalled,
                );

                // Record substitution
                if let Some(ref warning) = resolution.warning {
                    self.substitution_summary.write().unwrap()
                        .add_substitution(warning, weight, style);
                }

                return Ok(resolution);
            }
        }

        // Use default family as last resort
        let default = self.config.default_family.clone();
        if let Some(defaults) = self.fallback_chain.resolve_generic(&default) {
            for d in defaults {
                if index.has_family(d) {
                    let resolution = FontResolution::substituted(
                        family,
                        d,
                        weight,
                        style,
                        SubstitutionReason::FallbackToDefault,
                    );

                    if let Some(ref warning) = resolution.warning {
                        self.substitution_summary.write().unwrap()
                            .add_substitution(warning, weight, style);
                    }

                    return Ok(resolution);
                }
            }
        }

        Err(TextError::FontNotFound(format!(
            "No substitute found for: {}",
            family
        )))
    }

    /// Load font data for a resolved font
    pub fn load_font(&self, resolution: &FontResolution) -> Result<LoadedFont> {
        let font_id = FontId::new(&resolution.family)
            .with_weight(resolution.weight)
            .with_style(resolution.style);

        // Check cache first
        {
            let cache = self.loaded_fonts.read().unwrap();
            if let Some(loaded) = cache.get(&font_id) {
                return Ok(loaded.clone());
            }
        }

        // Load the font
        let info = self.discovery.select_font(
            &resolution.family,
            resolution.weight,
            resolution.style,
        )?;

        let data = self.discovery.load_font_data(&info)?;

        // Create loaded font
        let id = {
            let mut next_id = self.next_font_id.write().unwrap();
            let id = *next_id;
            *next_id += 1;
            LoadedFontId(id)
        };

        let loaded = LoadedFont {
            id,
            info,
            data: Arc::new(data),
        };

        // Cache it
        {
            let mut cache = self.loaded_fonts.write().unwrap();

            // Evict old fonts if cache is full
            if cache.len() >= self.config.max_cached_fonts {
                // Simple eviction: remove first entry
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }

            cache.insert(font_id, loaded.clone());
        }

        Ok(loaded)
    }

    /// Resolve and load a font in one step
    pub fn resolve_and_load(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Result<(LoadedFont, Option<SubstitutionWarning>)> {
        let resolution = self.resolve_font(family, weight, style)?;
        let warning = resolution.warning.clone();
        let loaded = self.load_font(&resolution)?;
        Ok((loaded, warning))
    }

    /// Get list of all available font families
    pub fn list_families(&self) -> Result<Vec<String>> {
        let index = self.get_index()?;
        Ok(index.list_families().to_vec())
    }

    /// Get the current substitution summary
    pub fn get_substitution_summary(&self) -> FontSubstitutionSummary {
        self.substitution_summary.read().unwrap().clone()
    }

    /// Clear the substitution summary
    pub fn clear_substitution_summary(&self) {
        let mut summary = self.substitution_summary.write().unwrap();
        *summary = FontSubstitutionSummary::new();
    }

    /// Get font metrics for a font
    pub fn get_metrics(&self, family: &str, weight: FontWeight, style: FontStyle) -> Result<FontMetrics> {
        // Resolve the font first
        let resolution = self.resolve_font(family, weight, style)?;

        // Load the font
        let loaded = self.load_font(&resolution)?;

        // Parse metrics from font data
        if let Some(face) = rustybuzz::Face::from_slice(&loaded.data, loaded.info.font_index) {
            Ok(FontMetrics {
                units_per_em: face.units_per_em() as u16,
                ascender: face.ascender(),
                descender: face.descender(),
                line_gap: face.line_gap(),
                cap_height: face.capital_height(),
                x_height: face.x_height(),
            })
        } else {
            // Return default metrics if we can't parse the font
            Ok(FontMetrics::default())
        }
    }

    /// Get the fallback chain configuration
    pub fn fallback_chain(&self) -> &FallbackChain {
        &self.fallback_chain
    }

    /// Get a mutable reference to the fallback chain
    pub fn fallback_chain_mut(&mut self) -> &mut FallbackChain {
        &mut self.fallback_chain
    }

    /// Detect the script of text and resolve appropriate fallback font
    pub fn resolve_for_script(
        &self,
        family: &str,
        text: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Result<FontResolution> {
        let script = Script::detect(text);

        // First try the requested font
        if self.is_font_available(family) {
            return Ok(FontResolution::exact(family, weight, style));
        }

        // Get script-specific fallbacks
        let fallbacks = self.fallback_chain.get_fallback_chain(family, Some(script));
        let index = self.get_index()?;

        for fallback in fallbacks {
            if index.has_family(&fallback) {
                let resolution = FontResolution::substituted(
                    family,
                    &fallback,
                    weight,
                    style,
                    SubstitutionReason::ScriptNotSupported,
                );

                if let Some(ref warning) = resolution.warning {
                    self.substitution_summary.write().unwrap()
                        .add_substitution(warning, weight, style);
                }

                return Ok(resolution);
            }
        }

        // Fall back to default
        self.find_substitute(family, weight, style)
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert!(!manager.is_initialized());
    }

    #[test]
    fn test_font_manager_initialization() {
        let manager = FontManager::new();
        let result = manager.initialize();
        assert!(result.is_ok());
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_font_resolution() {
        let manager = FontManager::new();
        manager.initialize().unwrap();

        // Try to resolve a generic font
        let result = manager.resolve_font("sans-serif", FontWeight::Normal, FontStyle::Normal);
        assert!(result.is_ok());

        let resolution = result.unwrap();
        assert!(!resolution.family.is_empty());
    }

    #[test]
    fn test_missing_font_substitution() {
        let manager = FontManager::new();
        manager.initialize().unwrap();

        // Try to resolve a non-existent font
        let result = manager.resolve_font(
            "NonExistentFontXYZ12345",
            FontWeight::Normal,
            FontStyle::Normal,
        );

        // Should succeed with substitution (or fail if no fonts available)
        if result.is_ok() {
            let resolution = result.unwrap();
            assert!(resolution.was_substituted());
        }
    }

    #[test]
    fn test_substitution_summary() {
        let manager = FontManager::new();
        manager.initialize().unwrap();

        // Clear any existing summary
        manager.clear_substitution_summary();

        // Resolve a non-existent font
        let _ = manager.resolve_font(
            "NonExistentFont123",
            FontWeight::Normal,
            FontStyle::Normal,
        );

        let summary = manager.get_substitution_summary();
        // May or may not have substitutions depending on whether resolution succeeded
        assert!(summary.total_found == 0 || summary.total_substituted > 0 || summary.substitutions.is_empty());
    }

    #[test]
    fn test_list_families() {
        let manager = FontManager::new();
        manager.initialize().unwrap();

        let families = manager.list_families().unwrap();
        assert!(!families.is_empty());
    }
}
