//! Font Fallback and Substitution Module
//!
//! This module provides font fallback chains and substitution rules
//! for handling missing fonts. It supports:
//! - Common font substitutions (Arial -> Helvetica, etc.)
//! - Per-script fallback chains (Latin, CJK, Arabic, Hebrew, etc.)
//! - Generic font family resolution (serif, sans-serif, monospace)

use crate::{FontStyle, FontWeight};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Script categories for font fallback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Script {
    /// Basic Latin (ASCII)
    Latin,
    /// Extended Latin (accented characters)
    LatinExtended,
    /// Greek
    Greek,
    /// Cyrillic
    Cyrillic,
    /// Arabic
    Arabic,
    /// Hebrew
    Hebrew,
    /// CJK (Chinese, Japanese, Korean)
    Cjk,
    /// Thai
    Thai,
    /// Devanagari (Hindi, Sanskrit, etc.)
    Devanagari,
    /// Symbol characters
    Symbol,
    /// Emoji
    Emoji,
    /// Unknown or mixed script
    Unknown,
}

impl Script {
    /// Detect the primary script of a text string
    pub fn detect(text: &str) -> Self {
        // Count characters in each script
        let mut latin = 0;
        let mut cjk = 0;
        let mut arabic = 0;
        let mut hebrew = 0;
        let mut cyrillic = 0;
        let mut greek = 0;
        let mut emoji = 0;

        for ch in text.chars() {
            match ch {
                '\u{0000}'..='\u{007F}' => latin += 1,
                '\u{0080}'..='\u{024F}' => latin += 1, // Latin Extended
                '\u{0370}'..='\u{03FF}' => greek += 1,
                '\u{0400}'..='\u{04FF}' => cyrillic += 1,
                '\u{0590}'..='\u{05FF}' => hebrew += 1,
                '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' => arabic += 1,
                '\u{4E00}'..='\u{9FFF}' => cjk += 1, // CJK Unified Ideographs
                '\u{3040}'..='\u{309F}' => cjk += 1, // Hiragana
                '\u{30A0}'..='\u{30FF}' => cjk += 1, // Katakana
                '\u{AC00}'..='\u{D7AF}' => cjk += 1, // Hangul Syllables
                '\u{1F300}'..='\u{1F9FF}' => emoji += 1, // Emoji
                _ => {}
            }
        }

        // Return the dominant script
        let counts = [
            (Script::Latin, latin),
            (Script::Cjk, cjk),
            (Script::Arabic, arabic),
            (Script::Hebrew, hebrew),
            (Script::Cyrillic, cyrillic),
            (Script::Greek, greek),
            (Script::Emoji, emoji),
        ];

        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .filter(|(_, count)| *count > 0)
            .map(|(script, _)| script)
            .unwrap_or(Script::Unknown)
    }
}

/// Font category for generic font families
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontCategory {
    /// Sans-serif fonts
    SansSerif,
    /// Serif fonts
    Serif,
    /// Monospace fonts
    Monospace,
    /// Cursive/handwriting fonts
    Cursive,
    /// Fantasy/decorative fonts
    Fantasy,
    /// System UI font
    SystemUi,
}

/// A font substitution warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionWarning {
    /// The font that was requested
    pub requested: String,
    /// The font that was actually used
    pub substituted: String,
    /// Reason for substitution
    pub reason: SubstitutionReason,
}

/// Reason for font substitution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubstitutionReason {
    /// Font not installed on system
    NotInstalled,
    /// Specific weight/style variant not available
    VariantNotAvailable,
    /// Script not supported by font
    ScriptNotSupported,
    /// Fallback to default font
    FallbackToDefault,
}

/// Result of font resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResolution {
    /// The resolved font family
    pub family: String,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Warning if substitution occurred
    pub warning: Option<SubstitutionWarning>,
}

impl FontResolution {
    /// Create a resolution without substitution
    pub fn exact(family: impl Into<String>, weight: FontWeight, style: FontStyle) -> Self {
        Self {
            family: family.into(),
            weight,
            style,
            warning: None,
        }
    }

    /// Create a resolution with substitution
    pub fn substituted(
        requested: impl Into<String>,
        resolved: impl Into<String>,
        weight: FontWeight,
        style: FontStyle,
        reason: SubstitutionReason,
    ) -> Self {
        let requested = requested.into();
        let resolved = resolved.into();
        Self {
            family: resolved.clone(),
            weight,
            style,
            warning: Some(SubstitutionWarning {
                requested,
                substituted: resolved,
                reason,
            }),
        }
    }

    /// Check if this resolution involved a substitution
    pub fn was_substituted(&self) -> bool {
        self.warning.is_some()
    }
}

/// Font fallback chain configuration
#[derive(Debug, Clone)]
pub struct FallbackChain {
    /// Primary substitution map (font -> fallback list)
    substitutions: HashMap<String, Vec<String>>,
    /// Per-script fallback chains
    script_fallbacks: HashMap<Script, Vec<String>>,
    /// Default fallback fonts by category
    category_defaults: HashMap<FontCategory, Vec<String>>,
}

impl FallbackChain {
    /// Create a new fallback chain with default substitutions
    pub fn new() -> Self {
        let mut chain = Self {
            substitutions: HashMap::new(),
            script_fallbacks: HashMap::new(),
            category_defaults: HashMap::new(),
        };
        chain.initialize_defaults();
        chain
    }

    /// Initialize default substitution rules
    fn initialize_defaults(&mut self) {
        // === Common font substitutions ===

        // Sans-serif family
        self.add_substitution_chain(
            "Arial",
            &["Helvetica", "Helvetica Neue", "Liberation Sans", "sans-serif"],
        );
        self.add_substitution_chain(
            "Helvetica",
            &["Helvetica Neue", "Arial", "Liberation Sans", "sans-serif"],
        );
        self.add_substitution_chain(
            "Verdana",
            &["DejaVu Sans", "Bitstream Vera Sans", "Arial", "sans-serif"],
        );
        self.add_substitution_chain(
            "Tahoma",
            &["Geneva", "Verdana", "Arial", "sans-serif"],
        );
        self.add_substitution_chain(
            "Segoe UI",
            &["San Francisco", "Helvetica Neue", "Arial", "sans-serif"],
        );
        self.add_substitution_chain(
            "Calibri",
            &["Carlito", "Helvetica Neue", "Arial", "sans-serif"],
        );

        // Serif family
        self.add_substitution_chain(
            "Times New Roman",
            &["Times", "Liberation Serif", "DejaVu Serif", "serif"],
        );
        self.add_substitution_chain(
            "Times",
            &["Times New Roman", "Liberation Serif", "serif"],
        );
        self.add_substitution_chain(
            "Georgia",
            &["Times New Roman", "Times", "Liberation Serif", "serif"],
        );
        self.add_substitution_chain(
            "Cambria",
            &["Caladea", "Times New Roman", "Liberation Serif", "serif"],
        );
        self.add_substitution_chain(
            "Palatino",
            &["Palatino Linotype", "Book Antiqua", "Times New Roman", "serif"],
        );

        // Monospace family
        self.add_substitution_chain(
            "Courier New",
            &["Courier", "Liberation Mono", "DejaVu Sans Mono", "monospace"],
        );
        self.add_substitution_chain(
            "Courier",
            &["Courier New", "Liberation Mono", "monospace"],
        );
        self.add_substitution_chain(
            "Consolas",
            &["Menlo", "Monaco", "Liberation Mono", "monospace"],
        );
        self.add_substitution_chain(
            "Monaco",
            &["Menlo", "Consolas", "Liberation Mono", "monospace"],
        );

        // === Per-script fallback chains ===

        // Latin script - platform-aware defaults
        #[cfg(target_os = "macos")]
        self.set_script_fallback(
            Script::Latin,
            vec![
                "San Francisco".to_string(),
                "Helvetica Neue".to_string(),
                "Helvetica".to_string(),
                "Arial".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_script_fallback(
            Script::Latin,
            vec![
                "Segoe UI".to_string(),
                "Arial".to_string(),
                "Tahoma".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_script_fallback(
            Script::Latin,
            vec![
                "DejaVu Sans".to_string(),
                "Liberation Sans".to_string(),
                "FreeSans".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_script_fallback(
            Script::Latin,
            vec![
                "Arial".to_string(),
                "Helvetica".to_string(),
            ],
        );

        // CJK script
        #[cfg(target_os = "macos")]
        self.set_script_fallback(
            Script::Cjk,
            vec![
                "Hiragino Sans".to_string(),
                "Hiragino Kaku Gothic Pro".to_string(),
                "Apple SD Gothic Neo".to_string(),
                "PingFang SC".to_string(),
                "Noto Sans CJK".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_script_fallback(
            Script::Cjk,
            vec![
                "Microsoft YaHei".to_string(),
                "MS Gothic".to_string(),
                "Malgun Gothic".to_string(),
                "SimHei".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_script_fallback(
            Script::Cjk,
            vec![
                "Noto Sans CJK SC".to_string(),
                "Noto Sans CJK JP".to_string(),
                "Noto Sans CJK KR".to_string(),
                "Droid Sans Fallback".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_script_fallback(
            Script::Cjk,
            vec![
                "Noto Sans CJK".to_string(),
            ],
        );

        // Arabic script
        #[cfg(target_os = "macos")]
        self.set_script_fallback(
            Script::Arabic,
            vec![
                "Geeza Pro".to_string(),
                "Baghdad".to_string(),
                "Noto Sans Arabic".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_script_fallback(
            Script::Arabic,
            vec![
                "Arabic Typesetting".to_string(),
                "Segoe UI".to_string(),
                "Tahoma".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_script_fallback(
            Script::Arabic,
            vec![
                "Noto Sans Arabic".to_string(),
                "DejaVu Sans".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_script_fallback(
            Script::Arabic,
            vec![
                "Noto Sans Arabic".to_string(),
            ],
        );

        // Hebrew script
        #[cfg(target_os = "macos")]
        self.set_script_fallback(
            Script::Hebrew,
            vec![
                "Arial Hebrew".to_string(),
                "Lucida Grande".to_string(),
                "Noto Sans Hebrew".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_script_fallback(
            Script::Hebrew,
            vec![
                "David".to_string(),
                "Arial".to_string(),
                "Tahoma".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_script_fallback(
            Script::Hebrew,
            vec![
                "Noto Sans Hebrew".to_string(),
                "DejaVu Sans".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_script_fallback(
            Script::Hebrew,
            vec![
                "Noto Sans Hebrew".to_string(),
            ],
        );

        // Greek script
        self.set_script_fallback(
            Script::Greek,
            vec![
                "Arial".to_string(),
                "Helvetica".to_string(),
                "DejaVu Sans".to_string(),
            ],
        );

        // Cyrillic script
        self.set_script_fallback(
            Script::Cyrillic,
            vec![
                "Arial".to_string(),
                "DejaVu Sans".to_string(),
                "Liberation Sans".to_string(),
            ],
        );

        // Emoji
        #[cfg(target_os = "macos")]
        self.set_script_fallback(
            Script::Emoji,
            vec!["Apple Color Emoji".to_string()],
        );

        #[cfg(target_os = "windows")]
        self.set_script_fallback(
            Script::Emoji,
            vec!["Segoe UI Emoji".to_string()],
        );

        #[cfg(target_os = "linux")]
        self.set_script_fallback(
            Script::Emoji,
            vec!["Noto Color Emoji".to_string()],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_script_fallback(
            Script::Emoji,
            vec!["Noto Color Emoji".to_string()],
        );

        // === Category defaults ===

        // Sans-serif defaults
        #[cfg(target_os = "macos")]
        self.set_category_default(
            FontCategory::SansSerif,
            vec![
                "San Francisco".to_string(),
                "Helvetica Neue".to_string(),
                "Helvetica".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_category_default(
            FontCategory::SansSerif,
            vec![
                "Segoe UI".to_string(),
                "Arial".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_category_default(
            FontCategory::SansSerif,
            vec![
                "DejaVu Sans".to_string(),
                "Liberation Sans".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_category_default(
            FontCategory::SansSerif,
            vec!["Arial".to_string()],
        );

        // Serif defaults
        #[cfg(target_os = "macos")]
        self.set_category_default(
            FontCategory::Serif,
            vec![
                "Times".to_string(),
                "Georgia".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_category_default(
            FontCategory::Serif,
            vec![
                "Times New Roman".to_string(),
                "Georgia".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_category_default(
            FontCategory::Serif,
            vec![
                "DejaVu Serif".to_string(),
                "Liberation Serif".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_category_default(
            FontCategory::Serif,
            vec!["Times New Roman".to_string()],
        );

        // Monospace defaults
        #[cfg(target_os = "macos")]
        self.set_category_default(
            FontCategory::Monospace,
            vec![
                "Menlo".to_string(),
                "Monaco".to_string(),
                "Courier".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_category_default(
            FontCategory::Monospace,
            vec![
                "Consolas".to_string(),
                "Courier New".to_string(),
            ],
        );

        #[cfg(target_os = "linux")]
        self.set_category_default(
            FontCategory::Monospace,
            vec![
                "DejaVu Sans Mono".to_string(),
                "Liberation Mono".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_category_default(
            FontCategory::Monospace,
            vec!["Courier New".to_string()],
        );

        // System UI
        #[cfg(target_os = "macos")]
        self.set_category_default(
            FontCategory::SystemUi,
            vec![
                "-apple-system".to_string(),
                "BlinkMacSystemFont".to_string(),
                "San Francisco".to_string(),
            ],
        );

        #[cfg(target_os = "windows")]
        self.set_category_default(
            FontCategory::SystemUi,
            vec!["Segoe UI".to_string()],
        );

        #[cfg(target_os = "linux")]
        self.set_category_default(
            FontCategory::SystemUi,
            vec![
                "Ubuntu".to_string(),
                "DejaVu Sans".to_string(),
            ],
        );

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        self.set_category_default(
            FontCategory::SystemUi,
            vec!["sans-serif".to_string()],
        );
    }

    /// Add a substitution chain for a font
    pub fn add_substitution_chain(&mut self, font: &str, fallbacks: &[&str]) {
        self.substitutions.insert(
            font.to_lowercase(),
            fallbacks.iter().map(|s| s.to_string()).collect(),
        );
    }

    /// Set script-specific fallback fonts
    pub fn set_script_fallback(&mut self, script: Script, fonts: Vec<String>) {
        self.script_fallbacks.insert(script, fonts);
    }

    /// Set category default fonts
    pub fn set_category_default(&mut self, category: FontCategory, fonts: Vec<String>) {
        self.category_defaults.insert(category, fonts);
    }

    /// Get substitution chain for a font
    pub fn get_substitutions(&self, font: &str) -> Option<&Vec<String>> {
        self.substitutions.get(&font.to_lowercase())
    }

    /// Get fallback fonts for a script
    pub fn get_script_fallbacks(&self, script: Script) -> Option<&Vec<String>> {
        self.script_fallbacks.get(&script)
    }

    /// Get default fonts for a category
    pub fn get_category_defaults(&self, category: FontCategory) -> Option<&Vec<String>> {
        self.category_defaults.get(&category)
    }

    /// Resolve a generic font family to a specific font
    pub fn resolve_generic(&self, generic: &str) -> Option<&Vec<String>> {
        let category = match generic.to_lowercase().as_str() {
            "sans-serif" => FontCategory::SansSerif,
            "serif" => FontCategory::Serif,
            "monospace" => FontCategory::Monospace,
            "cursive" => FontCategory::Cursive,
            "fantasy" => FontCategory::Fantasy,
            "system-ui" => FontCategory::SystemUi,
            _ => return None,
        };
        self.category_defaults.get(&category)
    }

    /// Get complete fallback chain for a font, optionally considering script
    pub fn get_fallback_chain(&self, font: &str, script: Option<Script>) -> Vec<String> {
        let mut chain = Vec::new();

        // First, add direct substitutions
        if let Some(subs) = self.get_substitutions(font) {
            chain.extend(subs.clone());
        }

        // Add script-specific fallbacks
        if let Some(script) = script {
            if let Some(script_fonts) = self.get_script_fallbacks(script) {
                for f in script_fonts {
                    if !chain.contains(f) {
                        chain.push(f.clone());
                    }
                }
            }
        }

        // Finally, add generic sans-serif as last resort
        if let Some(defaults) = self.get_category_defaults(FontCategory::SansSerif) {
            for f in defaults {
                if !chain.contains(f) {
                    chain.push(f.clone());
                }
            }
        }

        chain
    }
}

impl Default for FallbackChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_detection() {
        assert_eq!(Script::detect("Hello World"), Script::Latin);
        assert_eq!(Script::detect(""), Script::Unknown);
        // Note: CJK detection requires actual CJK characters
    }

    #[test]
    fn test_fallback_chain() {
        let chain = FallbackChain::new();

        // Arial should have substitutions
        let arial_subs = chain.get_substitutions("Arial");
        assert!(arial_subs.is_some());

        let subs = arial_subs.unwrap();
        assert!(subs.contains(&"Helvetica".to_string()));
    }

    #[test]
    fn test_generic_font_resolution() {
        let chain = FallbackChain::new();

        let sans_serif = chain.resolve_generic("sans-serif");
        assert!(sans_serif.is_some());
        assert!(!sans_serif.unwrap().is_empty());
    }

    #[test]
    fn test_complete_fallback_chain() {
        let chain = FallbackChain::new();

        let complete = chain.get_fallback_chain("Times New Roman", Some(Script::Latin));
        assert!(!complete.is_empty());
        // Should contain Times as a substitution
        assert!(complete.iter().any(|f| f == "Times" || f == "Liberation Serif"));
    }
}
