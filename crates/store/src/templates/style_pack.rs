//! Style pack for exporting and importing document styles

use doc_model::{
    CharacterProperties, ParagraphProperties, Style, StyleId, StyleRegistry, StyleType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{TemplateError, TemplateResult};

/// A style pack containing exportable styles from a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePack {
    /// Style pack name
    pub name: String,
    /// Description of the style pack
    pub description: String,
    /// Author of the style pack
    pub author: String,
    /// Version of the style pack
    pub version: String,
    /// Creation timestamp (ISO 8601)
    pub created: String,
    /// The styles in this pack
    pub styles: Vec<ExportedStyle>,
}

/// An exported style with all its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedStyle {
    /// Style identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Style type
    pub style_type: String,
    /// Base style ID (if any)
    pub based_on: Option<String>,
    /// Next style ID (for paragraph styles)
    pub next_style: Option<String>,
    /// Whether this is a built-in style
    pub built_in: bool,
    /// Priority for sorting
    pub priority: u32,
    /// Paragraph properties
    pub paragraph_props: ParagraphProperties,
    /// Character properties
    pub character_props: CharacterProperties,
}

impl From<&Style> for ExportedStyle {
    fn from(style: &Style) -> Self {
        Self {
            id: style.id.to_string(),
            name: style.name.clone(),
            style_type: match style.style_type {
                StyleType::Paragraph => "paragraph".to_string(),
                StyleType::Character => "character".to_string(),
                StyleType::Table => "table".to_string(),
                StyleType::Numbering => "numbering".to_string(),
            },
            based_on: style.based_on.as_ref().map(|s| s.to_string()),
            next_style: style.next_style.as_ref().map(|s| s.to_string()),
            built_in: style.built_in,
            priority: style.priority,
            paragraph_props: style.paragraph_props.clone(),
            character_props: style.character_props.clone(),
        }
    }
}

impl StylePack {
    /// Create a new empty style pack
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            author: String::new(),
            version: "1.0".to_string(),
            created: Self::now_iso8601(),
            styles: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Add a style to the pack
    pub fn add_style(&mut self, style: &Style) {
        self.styles.push(ExportedStyle::from(style));
    }

    /// Export all non-built-in styles from a registry
    pub fn from_registry(registry: &StyleRegistry, name: impl Into<String>) -> Self {
        let mut pack = Self::new(name);

        for style in registry.all_styles() {
            if !style.built_in {
                pack.add_style(style);
            }
        }

        pack
    }

    /// Export all styles (including built-in) from a registry
    pub fn from_registry_all(registry: &StyleRegistry, name: impl Into<String>) -> Self {
        let mut pack = Self::new(name);

        for style in registry.all_styles() {
            pack.add_style(style);
        }

        pack
    }

    /// Export only specified styles from a registry
    pub fn from_registry_filtered(
        registry: &StyleRegistry,
        name: impl Into<String>,
        style_ids: &[&str],
    ) -> Self {
        let mut pack = Self::new(name);

        for id in style_ids {
            if let Some(style) = registry.get(&StyleId::new(*id)) {
                pack.add_style(style);
            }
        }

        pack
    }

    /// Import styles into a registry
    ///
    /// Returns the number of styles imported
    pub fn import_into(&self, registry: &mut StyleRegistry) -> TemplateResult<usize> {
        let mut count = 0;

        for exported in &self.styles {
            // Skip built-in styles (they should already exist)
            if exported.built_in {
                continue;
            }

            let style_type = match exported.style_type.as_str() {
                "paragraph" => StyleType::Paragraph,
                "character" => StyleType::Character,
                "table" => StyleType::Table,
                "numbering" => StyleType::Numbering,
                other => {
                    return Err(TemplateError::InvalidFormat(format!(
                        "Unknown style type: {}",
                        other
                    )))
                }
            };

            let mut style = match style_type {
                StyleType::Paragraph => {
                    Style::paragraph(exported.id.as_str(), exported.name.as_str())
                }
                StyleType::Character => {
                    Style::character(exported.id.as_str(), exported.name.as_str())
                }
                _ => continue, // Skip table and numbering for now
            };

            style.priority = exported.priority;
            style.paragraph_props = exported.paragraph_props.clone();
            style.character_props = exported.character_props.clone();

            if let Some(ref base) = exported.based_on {
                style.based_on = Some(StyleId::new(base));
            }

            if let Some(ref next) = exported.next_style {
                style.next_style = Some(StyleId::new(next));
            }

            registry.register(style);
            count += 1;
        }

        Ok(count)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> TemplateResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> TemplateResult<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Get current timestamp in ISO 8601 format
    fn now_iso8601() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        format!("{}Z", duration.as_secs())
    }
}

/// Options for applying a style pack to a document
#[derive(Debug, Clone, Default)]
pub struct StylePackApplyOptions {
    /// Whether to overwrite existing styles with the same ID
    pub overwrite_existing: bool,
    /// Whether to include built-in style overrides
    pub include_builtin_overrides: bool,
    /// Only import styles matching these IDs (empty = all)
    pub filter_style_ids: Vec<String>,
}

impl StylePackApplyOptions {
    /// Create new options
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow overwriting existing styles
    pub fn with_overwrite(mut self) -> Self {
        self.overwrite_existing = true;
        self
    }

    /// Include built-in style overrides
    pub fn with_builtin_overrides(mut self) -> Self {
        self.include_builtin_overrides = true;
        self
    }

    /// Filter to specific style IDs
    pub fn with_filter(mut self, style_ids: Vec<String>) -> Self {
        self.filter_style_ids = style_ids;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_pack_creation() {
        let pack = StylePack::new("Test Pack")
            .with_description("A test style pack")
            .with_author("Test Author");

        assert_eq!(pack.name, "Test Pack");
        assert_eq!(pack.description, "A test style pack");
        assert_eq!(pack.author, "Test Author");
        assert!(pack.styles.is_empty());
    }

    #[test]
    fn test_style_pack_export() {
        let mut registry = StyleRegistry::new();

        // Add a custom style
        let custom_style = Style::paragraph("CustomStyle", "My Custom Style")
            .with_based_on("Normal")
            .with_character_props(CharacterProperties {
                bold: Some(true),
                font_size: Some(14.0),
                ..Default::default()
            });
        registry.register(custom_style);

        // Export non-built-in styles
        let pack = StylePack::from_registry(&registry, "Test Export");

        assert_eq!(pack.styles.len(), 1);
        assert_eq!(pack.styles[0].id, "CustomStyle");
        assert_eq!(pack.styles[0].name, "My Custom Style");
    }

    #[test]
    fn test_style_pack_import() {
        let mut source_registry = StyleRegistry::new();

        // Add a custom style
        let custom_style = Style::paragraph("ImportTest", "Import Test Style")
            .with_character_props(CharacterProperties {
                italic: Some(true),
                ..Default::default()
            });
        source_registry.register(custom_style);

        // Export
        let pack = StylePack::from_registry(&source_registry, "Test");

        // Import into a new registry
        let mut target_registry = StyleRegistry::new();
        let count = pack.import_into(&mut target_registry).unwrap();

        assert_eq!(count, 1);
        assert!(target_registry.contains(&StyleId::new("ImportTest")));

        let imported = target_registry.get(&StyleId::new("ImportTest")).unwrap();
        assert_eq!(imported.character_props.italic, Some(true));
    }

    #[test]
    fn test_style_pack_json_round_trip() {
        let pack = StylePack::new("JSON Test")
            .with_description("Testing JSON serialization");

        let json = pack.to_json().unwrap();
        let loaded = StylePack::from_json(&json).unwrap();

        assert_eq!(loaded.name, "JSON Test");
        assert_eq!(loaded.description, "Testing JSON serialization");
    }

    #[test]
    fn test_style_pack_filtered_export() {
        let mut registry = StyleRegistry::new();

        let style1 = Style::paragraph("Style1", "Style 1");
        let style2 = Style::paragraph("Style2", "Style 2");
        let style3 = Style::paragraph("Style3", "Style 3");

        registry.register(style1);
        registry.register(style2);
        registry.register(style3);

        // Export only Style1 and Style3
        let pack = StylePack::from_registry_filtered(&registry, "Filtered", &["Style1", "Style3"]);

        assert_eq!(pack.styles.len(), 2);
        let ids: Vec<_> = pack.styles.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"Style1"));
        assert!(ids.contains(&"Style3"));
        assert!(!ids.contains(&"Style2"));
    }
}
