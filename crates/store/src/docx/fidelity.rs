//! DOCX Import/Export Fidelity Tracking
//!
//! This module provides:
//! - Import warnings for unsupported features
//! - Export validation for potential data loss
//! - Fidelity scoring for documents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Fidelity Warning Types
// =============================================================================

/// Severity of a fidelity warning
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WarningSeverity {
    /// Informational - no data loss, but behavior may differ
    Info,
    /// Minor - small formatting differences possible
    Minor,
    /// Moderate - some content may look different
    Moderate,
    /// Major - significant content or formatting changes
    Major,
    /// Critical - content may be lost or corrupted
    Critical,
}

/// Category of feature that caused the warning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureCategory {
    /// Text formatting (fonts, styles, etc.)
    TextFormatting,
    /// Paragraph formatting
    ParagraphFormatting,
    /// Table features
    Tables,
    /// Images and pictures
    Images,
    /// Shapes and drawings
    Shapes,
    /// Text boxes
    TextBoxes,
    /// Track changes/revisions
    TrackChanges,
    /// Comments
    Comments,
    /// Footnotes and endnotes
    Notes,
    /// Fields (PAGE, TOC, etc.)
    Fields,
    /// Headers and footers
    HeadersFooters,
    /// Sections
    Sections,
    /// Columns
    Columns,
    /// Lists and numbering
    Lists,
    /// Styles
    Styles,
    /// Document properties
    DocumentProperties,
    /// Macros and scripts
    Macros,
    /// Embedded objects
    EmbeddedObjects,
    /// Equations/math
    Equations,
    /// Charts
    Charts,
    /// SmartArt
    SmartArt,
    /// Other/unknown
    Other,
}

/// A single fidelity warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityWarning {
    /// Unique warning code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: WarningSeverity,
    /// Feature category
    pub category: FeatureCategory,
    /// Location in document (if applicable)
    pub location: Option<WarningLocation>,
    /// Suggested workaround or action
    pub suggestion: Option<String>,
    /// Count of occurrences
    pub count: usize,
}

/// Location of a warning in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningLocation {
    /// Page number (if known)
    pub page: Option<u32>,
    /// Paragraph index
    pub paragraph: Option<usize>,
    /// Element name or path
    pub element: Option<String>,
}

impl FidelityWarning {
    /// Create a new warning
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: WarningSeverity,
        category: FeatureCategory,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity,
            category,
            location: None,
            suggestion: None,
            count: 1,
        }
    }

    /// Add a location to the warning
    pub fn with_location(mut self, location: WarningLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add a suggestion to the warning
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

// =============================================================================
// Fidelity Tracker
// =============================================================================

/// Tracks fidelity warnings during import/export
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FidelityTracker {
    /// All warnings, keyed by code
    warnings: HashMap<String, FidelityWarning>,
    /// Unsupported elements encountered
    unsupported_elements: HashMap<String, usize>,
    /// Unsupported attributes encountered
    unsupported_attributes: HashMap<String, usize>,
    /// Feature support status
    feature_status: HashMap<FeatureCategory, FeatureStatus>,
}

/// Status of a feature's support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureStatus {
    /// Fully supported
    Supported,
    /// Partially supported (some aspects may not round-trip)
    Partial,
    /// Not supported (will be lost or converted)
    Unsupported,
}

impl FidelityTracker {
    /// Create a new fidelity tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: FidelityWarning) {
        let code = warning.code.clone();
        self.warnings
            .entry(code)
            .and_modify(|w| w.count += 1)
            .or_insert(warning);
    }

    /// Record an unsupported element
    pub fn record_unsupported_element(&mut self, element: impl Into<String>) {
        let element = element.into();
        *self.unsupported_elements.entry(element).or_insert(0) += 1;
    }

    /// Record an unsupported attribute
    pub fn record_unsupported_attribute(&mut self, attribute: impl Into<String>) {
        let attribute = attribute.into();
        *self.unsupported_attributes.entry(attribute).or_insert(0) += 1;
    }

    /// Set feature status
    pub fn set_feature_status(&mut self, category: FeatureCategory, status: FeatureStatus) {
        self.feature_status.insert(category, status);
    }

    /// Get all warnings
    pub fn warnings(&self) -> impl Iterator<Item = &FidelityWarning> {
        self.warnings.values()
    }

    /// Get warnings by severity
    pub fn warnings_by_severity(&self, severity: WarningSeverity) -> Vec<&FidelityWarning> {
        self.warnings.values()
            .filter(|w| w.severity == severity)
            .collect()
    }

    /// Get warnings by category
    pub fn warnings_by_category(&self, category: FeatureCategory) -> Vec<&FidelityWarning> {
        self.warnings.values()
            .filter(|w| w.category == category)
            .collect()
    }

    /// Get the worst severity level encountered
    pub fn worst_severity(&self) -> Option<WarningSeverity> {
        self.warnings.values()
            .map(|w| w.severity)
            .max()
    }

    /// Check if there are any critical warnings
    pub fn has_critical_warnings(&self) -> bool {
        self.warnings.values()
            .any(|w| w.severity == WarningSeverity::Critical)
    }

    /// Get unsupported elements count
    pub fn unsupported_elements(&self) -> &HashMap<String, usize> {
        &self.unsupported_elements
    }

    /// Get unsupported attributes count
    pub fn unsupported_attributes(&self) -> &HashMap<String, usize> {
        &self.unsupported_attributes
    }

    /// Get feature status
    pub fn get_feature_status(&self, category: FeatureCategory) -> FeatureStatus {
        self.feature_status.get(&category).copied().unwrap_or(FeatureStatus::Supported)
    }

    /// Calculate overall fidelity score (0-100)
    pub fn fidelity_score(&self) -> f32 {
        let mut score = 100.0;

        // Deduct points based on warnings
        for warning in self.warnings.values() {
            let deduction = match warning.severity {
                WarningSeverity::Info => 0.1,
                WarningSeverity::Minor => 0.5,
                WarningSeverity::Moderate => 2.0,
                WarningSeverity::Major => 5.0,
                WarningSeverity::Critical => 15.0,
            };
            score -= deduction * (warning.count.min(10) as f32);
        }

        // Deduct for unsupported elements
        let unsupported_count: usize = self.unsupported_elements.values().sum();
        score -= (unsupported_count.min(50) as f32) * 0.2;

        score.max(0.0).min(100.0)
    }

    /// Clear all tracked data
    pub fn clear(&mut self) {
        self.warnings.clear();
        self.unsupported_elements.clear();
        self.unsupported_attributes.clear();
        self.feature_status.clear();
    }

    /// Merge another tracker into this one
    pub fn merge(&mut self, other: &FidelityTracker) {
        for (code, warning) in &other.warnings {
            self.warnings
                .entry(code.clone())
                .and_modify(|w| w.count += warning.count)
                .or_insert(warning.clone());
        }

        for (element, count) in &other.unsupported_elements {
            *self.unsupported_elements.entry(element.clone()).or_insert(0) += count;
        }

        for (attr, count) in &other.unsupported_attributes {
            *self.unsupported_attributes.entry(attr.clone()).or_insert(0) += count;
        }

        for (category, status) in &other.feature_status {
            // Keep the worse status
            let current = self.feature_status.get(category).copied().unwrap_or(FeatureStatus::Supported);
            let new_status = match (current, *status) {
                (FeatureStatus::Unsupported, _) | (_, FeatureStatus::Unsupported) => FeatureStatus::Unsupported,
                (FeatureStatus::Partial, _) | (_, FeatureStatus::Partial) => FeatureStatus::Partial,
                _ => FeatureStatus::Supported,
            };
            self.feature_status.insert(*category, new_status);
        }
    }
}

// =============================================================================
// Fidelity Report
// =============================================================================

/// Complete fidelity report for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityReport {
    /// Overall fidelity score (0-100)
    pub score: f32,
    /// Whether the document meets the 95% fidelity target
    pub meets_target: bool,
    /// Summary of issues by category
    pub category_summary: HashMap<FeatureCategory, CategorySummary>,
    /// All warnings
    pub warnings: Vec<FidelityWarning>,
    /// Recommendations for improving fidelity
    pub recommendations: Vec<String>,
}

/// Summary for a single category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Feature status
    pub status: FeatureStatus,
    /// Number of warnings
    pub warning_count: usize,
    /// Worst severity in this category
    pub worst_severity: Option<WarningSeverity>,
}

impl FidelityReport {
    /// Generate a report from a tracker
    pub fn from_tracker(tracker: &FidelityTracker) -> Self {
        let score = tracker.fidelity_score();
        let meets_target = score >= 95.0;

        // Build category summaries
        let mut category_summary = HashMap::new();
        for (category, status) in &tracker.feature_status {
            let warnings_in_category: Vec<_> = tracker.warnings.values()
                .filter(|w| w.category == *category)
                .collect();

            let worst_severity = warnings_in_category.iter()
                .map(|w| w.severity)
                .max();

            category_summary.insert(*category, CategorySummary {
                status: *status,
                warning_count: warnings_in_category.len(),
                worst_severity,
            });
        }

        // Build recommendations
        let mut recommendations = Vec::new();

        if tracker.has_critical_warnings() {
            recommendations.push(
                "Critical issues detected. Review document carefully before sharing.".to_string()
            );
        }

        if !tracker.unsupported_elements.is_empty() {
            let count = tracker.unsupported_elements.len();
            recommendations.push(format!(
                "{} unsupported DOCX elements encountered. Some content may be simplified.",
                count
            ));
        }

        if tracker.get_feature_status(FeatureCategory::Macros) == FeatureStatus::Unsupported {
            recommendations.push(
                "Macros are not supported and will be removed.".to_string()
            );
        }

        Self {
            score,
            meets_target,
            category_summary,
            warnings: tracker.warnings.values().cloned().collect(),
            recommendations,
        }
    }
}

// =============================================================================
// Import/Export Options
// =============================================================================

/// Options for DOCX import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    /// Whether to import track changes
    pub import_track_changes: bool,
    /// Whether to import comments
    pub import_comments: bool,
    /// Whether to import fields
    pub import_fields: bool,
    /// Whether to import shapes/drawings
    pub import_drawings: bool,
    /// Whether to import embedded objects
    pub import_embedded_objects: bool,
    /// Whether to preserve unknown elements as opaque data
    pub preserve_unknown: bool,
    /// Maximum image resolution (0 = no limit)
    pub max_image_resolution: u32,
    /// Whether to convert unsupported features to closest equivalent
    pub convert_unsupported: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            import_track_changes: true,
            import_comments: true,
            import_fields: true,
            import_drawings: true,
            import_embedded_objects: true,
            preserve_unknown: true,
            max_image_resolution: 0,
            convert_unsupported: true,
        }
    }
}

/// Options for DOCX export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Target Word version compatibility
    pub target_version: WordVersion,
    /// Whether to export track changes
    pub export_track_changes: bool,
    /// Whether to export comments
    pub export_comments: bool,
    /// Whether to export fields (or just their values)
    pub export_fields: bool,
    /// Whether to compress images
    pub compress_images: bool,
    /// Maximum image dimension (0 = no limit)
    pub max_image_dimension: u32,
    /// Whether to embed fonts
    pub embed_fonts: bool,
    /// Whether to include document properties
    pub include_properties: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            target_version: WordVersion::Word2019,
            export_track_changes: true,
            export_comments: true,
            export_fields: true,
            compress_images: true,
            max_image_dimension: 0,
            embed_fonts: false,
            include_properties: true,
        }
    }
}

/// Target Word version for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordVersion {
    /// Word 2007 (OOXML 1.0)
    Word2007,
    /// Word 2010 (OOXML 2.0)
    Word2010,
    /// Word 2013 (OOXML 3.0)
    Word2013,
    /// Word 2016
    Word2016,
    /// Word 2019
    Word2019,
    /// Word 365 (latest features)
    Word365,
}

// =============================================================================
// Predefined Warnings
// =============================================================================

/// Common warning codes
pub mod warning_codes {
    pub const UNSUPPORTED_FONT: &str = "FONT_001";
    pub const FONT_SUBSTITUTED: &str = "FONT_002";
    pub const UNSUPPORTED_TEXT_EFFECT: &str = "TEXT_001";
    pub const COMPLEX_FORMATTING_SIMPLIFIED: &str = "TEXT_002";
    pub const TABLE_MERGE_SIMPLIFIED: &str = "TABLE_001";
    pub const NESTED_TABLE_FLATTENED: &str = "TABLE_002";
    pub const IMAGE_CONVERTED: &str = "IMAGE_001";
    pub const IMAGE_RESOLUTION_REDUCED: &str = "IMAGE_002";
    pub const SHAPE_SIMPLIFIED: &str = "SHAPE_001";
    pub const SHAPE_EFFECT_REMOVED: &str = "SHAPE_002";
    pub const GROUP_UNGROUPED: &str = "SHAPE_003";
    pub const CONNECTOR_REMOVED: &str = "SHAPE_004";
    pub const TRACK_CHANGES_LIMITED: &str = "REVISION_001";
    pub const MOVE_TRACKING_SIMPLIFIED: &str = "REVISION_002";
    pub const COMMENT_THREADING_LOST: &str = "COMMENT_001";
    pub const FIELD_NOT_SUPPORTED: &str = "FIELD_001";
    pub const FIELD_RESULT_USED: &str = "FIELD_002";
    pub const TOC_SIMPLIFIED: &str = "FIELD_003";
    pub const HEADER_FOOTER_SIMPLIFIED: &str = "LAYOUT_001";
    pub const SECTION_BREAK_SIMPLIFIED: &str = "LAYOUT_002";
    pub const COLUMN_LAYOUT_SIMPLIFIED: &str = "LAYOUT_003";
    pub const FOOTNOTE_PLACEMENT_CHANGED: &str = "NOTE_001";
    pub const ENDNOTE_NUMBERING_RESET: &str = "NOTE_002";
    pub const STYLE_NOT_FOUND: &str = "STYLE_001";
    pub const STYLE_SIMPLIFIED: &str = "STYLE_002";
    pub const MACRO_REMOVED: &str = "MACRO_001";
    pub const EMBEDDED_OBJECT_REMOVED: &str = "EMBED_001";
    pub const EQUATION_CONVERTED: &str = "MATH_001";
    pub const CHART_STATIC: &str = "CHART_001";
    pub const SMARTART_CONVERTED: &str = "SMARTART_001";
}

/// Create common warnings
impl FidelityWarning {
    /// Create an unsupported font warning
    pub fn unsupported_font(font_name: &str) -> Self {
        Self::new(
            warning_codes::UNSUPPORTED_FONT,
            format!("Font '{}' is not available and will be substituted", font_name),
            WarningSeverity::Minor,
            FeatureCategory::TextFormatting,
        ).with_suggestion("Install the font or choose a similar alternative")
    }

    /// Create a table merge simplified warning
    pub fn table_merge_simplified() -> Self {
        Self::new(
            warning_codes::TABLE_MERGE_SIMPLIFIED,
            "Complex table cell merging has been simplified",
            WarningSeverity::Moderate,
            FeatureCategory::Tables,
        )
    }

    /// Create a field not supported warning
    pub fn field_not_supported(field_type: &str) -> Self {
        Self::new(
            warning_codes::FIELD_NOT_SUPPORTED,
            format!("Field type '{}' is not fully supported", field_type),
            WarningSeverity::Moderate,
            FeatureCategory::Fields,
        ).with_suggestion("Field will show its last calculated value")
    }

    /// Create a track changes limited warning
    pub fn track_changes_limited() -> Self {
        Self::new(
            warning_codes::TRACK_CHANGES_LIMITED,
            "Some track changes metadata may be simplified",
            WarningSeverity::Minor,
            FeatureCategory::TrackChanges,
        )
    }

    /// Create a comment threading lost warning
    pub fn comment_threading_lost() -> Self {
        Self::new(
            warning_codes::COMMENT_THREADING_LOST,
            "Comment reply threading structure may be simplified",
            WarningSeverity::Minor,
            FeatureCategory::Comments,
        )
    }

    /// Create a shape effect removed warning
    pub fn shape_effect_removed(effect_type: &str) -> Self {
        Self::new(
            warning_codes::SHAPE_EFFECT_REMOVED,
            format!("Shape effect '{}' is not supported", effect_type),
            WarningSeverity::Minor,
            FeatureCategory::Shapes,
        )
    }

    /// Create a macro removed warning
    pub fn macro_removed() -> Self {
        Self::new(
            warning_codes::MACRO_REMOVED,
            "Macros have been removed for security",
            WarningSeverity::Major,
            FeatureCategory::Macros,
        ).with_suggestion("Save as .docm to preserve macros")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fidelity_tracker_new() {
        let tracker = FidelityTracker::new();
        assert!(tracker.warnings.is_empty());
        assert_eq!(tracker.fidelity_score(), 100.0);
    }

    #[test]
    fn test_add_warning() {
        let mut tracker = FidelityTracker::new();

        let warning = FidelityWarning::new(
            "TEST_001",
            "Test warning",
            WarningSeverity::Minor,
            FeatureCategory::Other,
        );

        tracker.add_warning(warning);
        assert_eq!(tracker.warnings.len(), 1);
    }

    #[test]
    fn test_warning_counting() {
        let mut tracker = FidelityTracker::new();

        // Add same warning twice
        tracker.add_warning(FidelityWarning::new(
            "TEST_001",
            "Test",
            WarningSeverity::Minor,
            FeatureCategory::Other,
        ));
        tracker.add_warning(FidelityWarning::new(
            "TEST_001",
            "Test",
            WarningSeverity::Minor,
            FeatureCategory::Other,
        ));

        assert_eq!(tracker.warnings.len(), 1);
        assert_eq!(tracker.warnings.get("TEST_001").unwrap().count, 2);
    }

    #[test]
    fn test_fidelity_score_deductions() {
        let mut tracker = FidelityTracker::new();

        tracker.add_warning(FidelityWarning::new(
            "CRITICAL",
            "Critical issue",
            WarningSeverity::Critical,
            FeatureCategory::Other,
        ));

        assert!(tracker.fidelity_score() < 100.0);
        assert!(tracker.has_critical_warnings());
    }

    #[test]
    fn test_worst_severity() {
        let mut tracker = FidelityTracker::new();

        tracker.add_warning(FidelityWarning::new(
            "MINOR",
            "Minor",
            WarningSeverity::Minor,
            FeatureCategory::Other,
        ));
        tracker.add_warning(FidelityWarning::new(
            "MAJOR",
            "Major",
            WarningSeverity::Major,
            FeatureCategory::Other,
        ));

        assert_eq!(tracker.worst_severity(), Some(WarningSeverity::Major));
    }

    #[test]
    fn test_fidelity_report() {
        let mut tracker = FidelityTracker::new();
        tracker.add_warning(FidelityWarning::new(
            "TEST",
            "Test",
            WarningSeverity::Minor,
            FeatureCategory::Tables,
        ));
        tracker.set_feature_status(FeatureCategory::Tables, FeatureStatus::Partial);

        let report = FidelityReport::from_tracker(&tracker);

        assert!(report.score > 95.0);
        assert!(report.meets_target);
        assert!(report.category_summary.contains_key(&FeatureCategory::Tables));
    }

    #[test]
    fn test_predefined_warnings() {
        let warning = FidelityWarning::unsupported_font("Arial");
        assert_eq!(warning.code, warning_codes::UNSUPPORTED_FONT);
        assert!(warning.message.contains("Arial"));
    }
}
