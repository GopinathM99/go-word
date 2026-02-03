//! Caption system for images, tables, and equations
//!
//! Captions are auto-numbered paragraphs associated with objects like images,
//! tables, and equations. They use SEQ fields for automatic numbering and
//! can be cross-referenced from other parts of the document.

use crate::{
    field::{Field, FieldInstruction, NumberFormat, SeqOptions},
    Node, NodeId, Paragraph, Run, StyleId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Caption Label Types
// =============================================================================

/// Predefined and custom caption label types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaptionLabel {
    /// Figure captions (for images, diagrams, charts)
    Figure,
    /// Table captions
    Table,
    /// Equation captions
    Equation,
    /// Custom user-defined label
    Custom(String),
}

impl CaptionLabel {
    /// Get the display text for this label
    pub fn display_text(&self) -> &str {
        match self {
            CaptionLabel::Figure => "Figure",
            CaptionLabel::Table => "Table",
            CaptionLabel::Equation => "Equation",
            CaptionLabel::Custom(s) => s.as_str(),
        }
    }

    /// Get the SEQ field identifier for this label
    pub fn seq_identifier(&self) -> String {
        match self {
            CaptionLabel::Figure => "Figure".to_string(),
            CaptionLabel::Table => "Table".to_string(),
            CaptionLabel::Equation => "Equation".to_string(),
            CaptionLabel::Custom(s) => s.clone(),
        }
    }

    /// Get the default style ID for this label type
    pub fn default_style_id(&self) -> StyleId {
        StyleId::new("Caption")
    }
}

impl Default for CaptionLabel {
    fn default() -> Self {
        CaptionLabel::Figure
    }
}

impl std::fmt::Display for CaptionLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_text())
    }
}

// =============================================================================
// Caption Position
// =============================================================================

/// Position of caption relative to the target object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CaptionPosition {
    /// Caption appears above the object
    Above,
    /// Caption appears below the object (default for figures)
    #[default]
    Below,
}

impl CaptionPosition {
    /// Check if the caption should be above the object
    pub fn is_above(&self) -> bool {
        matches!(self, CaptionPosition::Above)
    }

    /// Check if the caption should be below the object
    pub fn is_below(&self) -> bool {
        matches!(self, CaptionPosition::Below)
    }
}

// =============================================================================
// Caption Format
// =============================================================================

/// Format settings for a caption label type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionFormat {
    /// The label type this format applies to
    pub label: CaptionLabel,
    /// Separator between number and text (e.g., ": " or " - ")
    pub separator: String,
    /// Number format for the sequence number
    pub number_format: NumberFormat,
    /// Whether to include chapter numbers (e.g., "Figure 2-1")
    pub include_chapter: bool,
    /// Chapter separator (e.g., "-" for "Figure 2-1" or "." for "Figure 2.1")
    pub chapter_separator: String,
    /// Heading style used to determine chapter numbers
    pub chapter_style: Option<StyleId>,
    /// Default position for this caption type
    pub default_position: CaptionPosition,
    /// Paragraph style to apply to caption paragraphs
    pub paragraph_style: StyleId,
}

impl CaptionFormat {
    /// Create a new caption format with defaults
    pub fn new(label: CaptionLabel) -> Self {
        let default_position = match &label {
            CaptionLabel::Table => CaptionPosition::Above,
            _ => CaptionPosition::Below,
        };

        Self {
            label,
            separator: ": ".to_string(),
            number_format: NumberFormat::Arabic,
            include_chapter: false,
            chapter_separator: "-".to_string(),
            chapter_style: None,
            default_position,
            paragraph_style: StyleId::new("Caption"),
        }
    }

    /// Set the separator
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Set the number format
    pub fn with_number_format(mut self, format: NumberFormat) -> Self {
        self.number_format = format;
        self
    }

    /// Enable chapter numbering
    pub fn with_chapter_numbering(mut self, chapter_style: StyleId, separator: &str) -> Self {
        self.include_chapter = true;
        self.chapter_style = Some(chapter_style);
        self.chapter_separator = separator.to_string();
        self
    }

    /// Set the default position
    pub fn with_default_position(mut self, position: CaptionPosition) -> Self {
        self.default_position = position;
        self
    }

    /// Set the paragraph style
    pub fn with_paragraph_style(mut self, style: StyleId) -> Self {
        self.paragraph_style = style;
        self
    }

    /// Format the caption number (without chapter)
    pub fn format_number(&self, number: u32) -> String {
        self.number_format.format(number)
    }

    /// Format the caption number with optional chapter prefix
    pub fn format_full_number(&self, chapter: Option<u32>, number: u32) -> String {
        if self.include_chapter {
            if let Some(ch) = chapter {
                return format!(
                    "{}{}{}",
                    self.number_format.format(ch),
                    self.chapter_separator,
                    self.number_format.format(number)
                );
            }
        }
        self.format_number(number)
    }
}

impl Default for CaptionFormat {
    fn default() -> Self {
        Self::new(CaptionLabel::Figure)
    }
}

// =============================================================================
// Caption
// =============================================================================

/// A caption attached to an object (image, table, equation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caption {
    /// Unique identifier for this caption
    id: NodeId,
    /// The label type (Figure, Table, etc.)
    pub label: CaptionLabel,
    /// Reference to the SEQ field node that provides numbering
    pub number_field_id: NodeId,
    /// User's caption text (after the number)
    pub text: String,
    /// Position relative to the target object
    pub position: CaptionPosition,
    /// Optional reference to the target object (image, table) being captioned
    pub target_id: Option<NodeId>,
    /// The paragraph node that contains this caption
    pub paragraph_id: NodeId,
    /// Bookmark name for cross-referencing (auto-generated)
    pub bookmark_name: String,
    /// Whether this caption should be included in a list of figures/tables
    pub include_in_list: bool,
}

impl Caption {
    /// Create a new caption
    pub fn new(
        label: CaptionLabel,
        text: impl Into<String>,
        position: CaptionPosition,
        target_id: Option<NodeId>,
        paragraph_id: NodeId,
        number_field_id: NodeId,
    ) -> Self {
        let id = NodeId::new();
        let bookmark_name = format!("_Ref{}_{}", label.seq_identifier(), id);

        Self {
            id,
            label,
            number_field_id,
            text: text.into(),
            position,
            target_id,
            paragraph_id,
            bookmark_name,
            include_in_list: true,
        }
    }

    /// Get the caption ID
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the caption text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the caption text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Get the bookmark name for cross-referencing
    pub fn bookmark_name(&self) -> &str {
        &self.bookmark_name
    }

    /// Check if this caption has a target object
    pub fn has_target(&self) -> bool {
        self.target_id.is_some()
    }

    /// Set whether this caption should be included in lists (TOF, TOT, etc.)
    pub fn set_include_in_list(&mut self, include: bool) {
        self.include_in_list = include;
    }
}

// =============================================================================
// Caption Registry
// =============================================================================

/// Registry for managing captions in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptionRegistry {
    /// All captions indexed by ID
    captions: HashMap<NodeId, Caption>,
    /// Captions indexed by target ID (for finding caption of an image/table)
    target_index: HashMap<NodeId, NodeId>,
    /// Captions indexed by paragraph ID
    paragraph_index: HashMap<NodeId, NodeId>,
    /// Caption formats by label type
    formats: HashMap<String, CaptionFormat>,
    /// Caption ordering for each label type (for numbering)
    ordering: HashMap<String, Vec<NodeId>>,
}

impl CaptionRegistry {
    /// Create a new caption registry
    pub fn new() -> Self {
        let mut registry = Self {
            captions: HashMap::new(),
            target_index: HashMap::new(),
            paragraph_index: HashMap::new(),
            formats: HashMap::new(),
            ordering: HashMap::new(),
        };

        // Initialize default formats
        registry.set_format(CaptionFormat::new(CaptionLabel::Figure));
        registry.set_format(CaptionFormat::new(CaptionLabel::Table));
        registry.set_format(CaptionFormat::new(CaptionLabel::Equation));

        registry
    }

    /// Insert a caption into the registry
    pub fn insert(&mut self, caption: Caption) -> NodeId {
        let id = caption.id;
        let label_key = caption.label.seq_identifier();

        // Update indices
        if let Some(target_id) = caption.target_id {
            self.target_index.insert(target_id, id);
        }
        self.paragraph_index.insert(caption.paragraph_id, id);

        // Add to ordering
        self.ordering
            .entry(label_key)
            .or_default()
            .push(id);

        // Insert the caption
        self.captions.insert(id, caption);

        id
    }

    /// Remove a caption from the registry
    pub fn remove(&mut self, id: NodeId) -> Option<Caption> {
        if let Some(caption) = self.captions.remove(&id) {
            // Remove from indices
            if let Some(target_id) = caption.target_id {
                self.target_index.remove(&target_id);
            }
            self.paragraph_index.remove(&caption.paragraph_id);

            // Remove from ordering
            let label_key = caption.label.seq_identifier();
            if let Some(order) = self.ordering.get_mut(&label_key) {
                order.retain(|&cid| cid != id);
            }

            Some(caption)
        } else {
            None
        }
    }

    /// Get a caption by ID
    pub fn get(&self, id: NodeId) -> Option<&Caption> {
        self.captions.get(&id)
    }

    /// Get a mutable caption by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Caption> {
        self.captions.get_mut(&id)
    }

    /// Get the caption for a target object (image, table)
    pub fn get_by_target(&self, target_id: NodeId) -> Option<&Caption> {
        self.target_index
            .get(&target_id)
            .and_then(|id| self.captions.get(id))
    }

    /// Get the caption for a paragraph
    pub fn get_by_paragraph(&self, paragraph_id: NodeId) -> Option<&Caption> {
        self.paragraph_index
            .get(&paragraph_id)
            .and_then(|id| self.captions.get(id))
    }

    /// Get all captions
    pub fn all(&self) -> impl Iterator<Item = &Caption> {
        self.captions.values()
    }

    /// Get all caption IDs
    pub fn all_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.captions.keys().copied()
    }

    /// Get captions by label type
    pub fn by_label(&self, label: &CaptionLabel) -> Vec<&Caption> {
        let label_key = label.seq_identifier();
        self.ordering
            .get(&label_key)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.captions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the ordered caption IDs for a label type
    pub fn ordering_for_label(&self, label: &CaptionLabel) -> Vec<NodeId> {
        let label_key = label.seq_identifier();
        self.ordering.get(&label_key).cloned().unwrap_or_default()
    }

    /// Update the ordering of captions for a label type
    /// This should be called after document structure changes
    pub fn update_ordering(&mut self, label: &CaptionLabel, order: Vec<NodeId>) {
        let label_key = label.seq_identifier();
        self.ordering.insert(label_key, order);
    }

    /// Get the number (position) of a caption in its sequence
    pub fn get_caption_number(&self, id: NodeId) -> Option<u32> {
        let caption = self.captions.get(&id)?;
        let label_key = caption.label.seq_identifier();
        let ordering = self.ordering.get(&label_key)?;

        ordering
            .iter()
            .position(|&cid| cid == id)
            .map(|pos| (pos + 1) as u32)
    }

    /// Get the format for a label type
    pub fn get_format(&self, label: &CaptionLabel) -> Option<&CaptionFormat> {
        let label_key = label.seq_identifier();
        self.formats.get(&label_key)
    }

    /// Get a mutable format for a label type
    pub fn get_format_mut(&mut self, label: &CaptionLabel) -> Option<&mut CaptionFormat> {
        let label_key = label.seq_identifier();
        self.formats.get_mut(&label_key)
    }

    /// Set the format for a label type
    pub fn set_format(&mut self, format: CaptionFormat) {
        let label_key = format.label.seq_identifier();
        self.formats.insert(label_key, format);
    }

    /// Get the number of captions
    pub fn len(&self) -> usize {
        self.captions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.captions.is_empty()
    }

    /// Get the count of captions for a label type
    pub fn count_by_label(&self, label: &CaptionLabel) -> usize {
        self.by_label(label).len()
    }

    /// Check if a target already has a caption
    pub fn target_has_caption(&self, target_id: NodeId) -> bool {
        self.target_index.contains_key(&target_id)
    }
}

// =============================================================================
// Caption Builder
// =============================================================================

/// Builder for creating captions with associated nodes
#[derive(Debug)]
pub struct CaptionBuilder {
    label: CaptionLabel,
    text: String,
    position: CaptionPosition,
    target_id: Option<NodeId>,
    format: Option<CaptionFormat>,
}

impl CaptionBuilder {
    /// Create a new caption builder
    pub fn new(label: CaptionLabel) -> Self {
        Self {
            label,
            text: String::new(),
            position: CaptionPosition::Below,
            target_id: None,
            format: None,
        }
    }

    /// Set the caption text
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the position
    pub fn with_position(mut self, position: CaptionPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the target object
    pub fn with_target(mut self, target_id: NodeId) -> Self {
        self.target_id = Some(target_id);
        self
    }

    /// Set a custom format
    pub fn with_format(mut self, format: CaptionFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Build the caption along with its SEQ field and paragraph
    pub fn build(self) -> CaptionComponents {
        let format = self.format.unwrap_or_else(|| CaptionFormat::new(self.label.clone()));

        // Create the SEQ field for auto-numbering
        let seq_field = Field::new(FieldInstruction::Seq {
            options: SeqOptions {
                identifier: self.label.seq_identifier(),
                format: format.number_format,
                ..Default::default()
            },
        });
        let seq_field_id = seq_field.id();

        // Create the caption paragraph
        let mut paragraph = Paragraph::with_paragraph_style(format.paragraph_style.clone());
        let paragraph_id = paragraph.id();

        // Create runs for the caption content
        // Label text run
        let label_run = Run::new(format!("{} ", self.label.display_text()));
        let label_run_id = label_run.id();
        paragraph.add_child(label_run_id);

        // The SEQ field will be added as a child
        paragraph.add_child(seq_field_id);

        // Separator and text run (if there's text)
        let text_run = if !self.text.is_empty() {
            let run = Run::new(format!("{}{}", format.separator, self.text));
            let run_id = run.id();
            paragraph.add_child(run_id);
            Some(run)
        } else {
            None
        };

        // Create the caption
        let caption = Caption::new(
            self.label,
            self.text,
            self.position,
            self.target_id,
            paragraph_id,
            seq_field_id,
        );

        CaptionComponents {
            caption,
            paragraph,
            seq_field,
            label_run,
            text_run,
        }
    }
}

/// Components created when building a caption
#[derive(Debug)]
pub struct CaptionComponents {
    /// The caption metadata
    pub caption: Caption,
    /// The paragraph node containing the caption
    pub paragraph: Paragraph,
    /// The SEQ field for auto-numbering
    pub seq_field: Field,
    /// The run containing the label text
    pub label_run: Run,
    /// Optional run containing separator and caption text
    pub text_run: Option<Run>,
}

// =============================================================================
// Cross-Reference Display Types for Captions
// =============================================================================

/// What to display when cross-referencing a caption
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CaptionRefDisplayType {
    /// Display full caption (e.g., "Figure 1: Description")
    FullCaption,
    /// Display label and number only (e.g., "Figure 1")
    #[default]
    LabelAndNumber,
    /// Display label only (e.g., "Figure")
    LabelOnly,
    /// Display number only (e.g., "1")
    NumberOnly,
    /// Display page number where caption appears
    PageNumber,
    /// Display "above" or "below" relative to reference position
    RelativePosition,
    /// Display the caption text only (without label and number)
    CaptionTextOnly,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caption_label_display() {
        assert_eq!(CaptionLabel::Figure.display_text(), "Figure");
        assert_eq!(CaptionLabel::Table.display_text(), "Table");
        assert_eq!(CaptionLabel::Equation.display_text(), "Equation");
        assert_eq!(CaptionLabel::Custom("Chart".to_string()).display_text(), "Chart");
    }

    #[test]
    fn test_caption_label_seq_identifier() {
        assert_eq!(CaptionLabel::Figure.seq_identifier(), "Figure");
        assert_eq!(CaptionLabel::Table.seq_identifier(), "Table");
        assert_eq!(CaptionLabel::Custom("Listing".to_string()).seq_identifier(), "Listing");
    }

    #[test]
    fn test_caption_format_defaults() {
        let format = CaptionFormat::new(CaptionLabel::Figure);
        assert_eq!(format.separator, ": ");
        assert_eq!(format.number_format, NumberFormat::Arabic);
        assert!(!format.include_chapter);
        assert_eq!(format.default_position, CaptionPosition::Below);

        let table_format = CaptionFormat::new(CaptionLabel::Table);
        assert_eq!(table_format.default_position, CaptionPosition::Above);
    }

    #[test]
    fn test_caption_format_number() {
        let format = CaptionFormat::new(CaptionLabel::Figure);
        assert_eq!(format.format_number(1), "1");
        assert_eq!(format.format_number(10), "10");

        let format_roman = CaptionFormat::new(CaptionLabel::Figure)
            .with_number_format(NumberFormat::UppercaseRoman);
        assert_eq!(format_roman.format_number(1), "I");
        assert_eq!(format_roman.format_number(4), "IV");
    }

    #[test]
    fn test_caption_format_with_chapter() {
        let format = CaptionFormat::new(CaptionLabel::Figure)
            .with_chapter_numbering(StyleId::new("Heading1"), "-");

        assert!(format.include_chapter);
        assert_eq!(format.chapter_separator, "-");
        assert_eq!(format.format_full_number(Some(2), 3), "2-3");
        assert_eq!(format.format_full_number(None, 3), "3");
    }

    #[test]
    fn test_caption_creation() {
        let paragraph_id = NodeId::new();
        let field_id = NodeId::new();
        let target_id = NodeId::new();

        let caption = Caption::new(
            CaptionLabel::Figure,
            "Test caption",
            CaptionPosition::Below,
            Some(target_id),
            paragraph_id,
            field_id,
        );

        assert_eq!(caption.label, CaptionLabel::Figure);
        assert_eq!(caption.text, "Test caption");
        assert_eq!(caption.position, CaptionPosition::Below);
        assert_eq!(caption.target_id, Some(target_id));
        assert!(caption.bookmark_name.starts_with("_RefFigure_"));
    }

    #[test]
    fn test_caption_registry() {
        let mut registry = CaptionRegistry::new();

        let para_id = NodeId::new();
        let field_id = NodeId::new();
        let target_id = NodeId::new();

        let caption = Caption::new(
            CaptionLabel::Figure,
            "First figure",
            CaptionPosition::Below,
            Some(target_id),
            para_id,
            field_id,
        );
        let caption_id = caption.id();

        registry.insert(caption);

        // Test retrieval
        assert!(registry.get(caption_id).is_some());
        assert_eq!(registry.get(caption_id).unwrap().text(), "First figure");

        // Test by target
        assert!(registry.get_by_target(target_id).is_some());

        // Test by paragraph
        assert!(registry.get_by_paragraph(para_id).is_some());

        // Test count
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.count_by_label(&CaptionLabel::Figure), 1);
        assert_eq!(registry.count_by_label(&CaptionLabel::Table), 0);
    }

    #[test]
    fn test_caption_numbering() {
        let mut registry = CaptionRegistry::new();

        // Add three figure captions
        let captions: Vec<_> = (1..=3)
            .map(|i| {
                let caption = Caption::new(
                    CaptionLabel::Figure,
                    format!("Figure caption {}", i),
                    CaptionPosition::Below,
                    None,
                    NodeId::new(),
                    NodeId::new(),
                );
                caption
            })
            .collect();

        let ids: Vec<_> = captions.iter().map(|c| c.id()).collect();
        for caption in captions {
            registry.insert(caption);
        }

        // Check numbering
        assert_eq!(registry.get_caption_number(ids[0]), Some(1));
        assert_eq!(registry.get_caption_number(ids[1]), Some(2));
        assert_eq!(registry.get_caption_number(ids[2]), Some(3));
    }

    #[test]
    fn test_caption_removal() {
        let mut registry = CaptionRegistry::new();

        let target_id = NodeId::new();
        let caption = Caption::new(
            CaptionLabel::Figure,
            "To be removed",
            CaptionPosition::Below,
            Some(target_id),
            NodeId::new(),
            NodeId::new(),
        );
        let caption_id = caption.id();

        registry.insert(caption);
        assert_eq!(registry.len(), 1);

        let removed = registry.remove(caption_id);
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
        assert!(!registry.target_has_caption(target_id));
    }

    #[test]
    fn test_caption_builder() {
        let target_id = NodeId::new();

        let components = CaptionBuilder::new(CaptionLabel::Figure)
            .with_text("A sample image")
            .with_position(CaptionPosition::Below)
            .with_target(target_id)
            .build();

        assert_eq!(components.caption.label, CaptionLabel::Figure);
        assert_eq!(components.caption.text, "A sample image");
        assert_eq!(components.caption.position, CaptionPosition::Below);
        assert_eq!(components.caption.target_id, Some(target_id));
        assert!(components.text_run.is_some());

        // Verify the SEQ field
        if let FieldInstruction::Seq { options } = &components.seq_field.instruction {
            assert_eq!(options.identifier, "Figure");
        } else {
            panic!("Expected SEQ field instruction");
        }
    }

    #[test]
    fn test_caption_builder_no_text() {
        let components = CaptionBuilder::new(CaptionLabel::Table).build();

        assert_eq!(components.caption.text, "");
        assert!(components.text_run.is_none());
    }

    #[test]
    fn test_caption_format_custom_label() {
        let format = CaptionFormat::new(CaptionLabel::Custom("Listing".to_string()));
        assert_eq!(format.label.display_text(), "Listing");
        assert_eq!(format.label.seq_identifier(), "Listing");
    }

    #[test]
    fn test_caption_registry_by_label() {
        let mut registry = CaptionRegistry::new();

        // Add figures
        for i in 1..=3 {
            let caption = Caption::new(
                CaptionLabel::Figure,
                format!("Figure {}", i),
                CaptionPosition::Below,
                None,
                NodeId::new(),
                NodeId::new(),
            );
            registry.insert(caption);
        }

        // Add tables
        for i in 1..=2 {
            let caption = Caption::new(
                CaptionLabel::Table,
                format!("Table {}", i),
                CaptionPosition::Above,
                None,
                NodeId::new(),
                NodeId::new(),
            );
            registry.insert(caption);
        }

        let figures = registry.by_label(&CaptionLabel::Figure);
        assert_eq!(figures.len(), 3);

        let tables = registry.by_label(&CaptionLabel::Table);
        assert_eq!(tables.len(), 2);

        let equations = registry.by_label(&CaptionLabel::Equation);
        assert_eq!(equations.len(), 0);
    }

    #[test]
    fn test_caption_text_modification() {
        let mut registry = CaptionRegistry::new();

        let caption = Caption::new(
            CaptionLabel::Figure,
            "Original text",
            CaptionPosition::Below,
            None,
            NodeId::new(),
            NodeId::new(),
        );
        let caption_id = caption.id();
        registry.insert(caption);

        // Modify the text
        if let Some(caption) = registry.get_mut(caption_id) {
            caption.set_text("Updated text");
        }

        assert_eq!(registry.get(caption_id).unwrap().text(), "Updated text");
    }

    #[test]
    fn test_default_formats_initialized() {
        let registry = CaptionRegistry::new();

        assert!(registry.get_format(&CaptionLabel::Figure).is_some());
        assert!(registry.get_format(&CaptionLabel::Table).is_some());
        assert!(registry.get_format(&CaptionLabel::Equation).is_some());
    }

    #[test]
    fn test_update_ordering() {
        let mut registry = CaptionRegistry::new();

        let captions: Vec<_> = (1..=3)
            .map(|i| {
                Caption::new(
                    CaptionLabel::Figure,
                    format!("Figure {}", i),
                    CaptionPosition::Below,
                    None,
                    NodeId::new(),
                    NodeId::new(),
                )
            })
            .collect();

        let ids: Vec<_> = captions.iter().map(|c| c.id()).collect();
        for caption in captions {
            registry.insert(caption);
        }

        // Initial order: 1, 2, 3
        assert_eq!(registry.get_caption_number(ids[0]), Some(1));
        assert_eq!(registry.get_caption_number(ids[2]), Some(3));

        // Reorder to: 3, 1, 2
        let new_order = vec![ids[2], ids[0], ids[1]];
        registry.update_ordering(&CaptionLabel::Figure, new_order);

        // Check new numbering
        assert_eq!(registry.get_caption_number(ids[0]), Some(2));
        assert_eq!(registry.get_caption_number(ids[2]), Some(1));
    }
}
