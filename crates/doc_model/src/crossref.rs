//! Cross-Reference System
//!
//! This module implements a comprehensive cross-reference system for the word processor.
//! Cross-references allow users to reference other parts of the document such as headings,
//! bookmarks, footnotes, captions (figures, tables, equations), and navigate to them.
//!
//! ## Features
//!
//! - Reference multiple target types: headings, bookmarks, footnotes, endnotes, captions
//! - Multiple display formats: text, number, page number, relative position, full caption
//! - Hyperlink support for navigation
//! - Automatic tracking and updating of references
//! - Detection of broken references when targets are deleted or moved
//!
//! ## Integration
//!
//! Cross-references build on existing systems:
//! - Uses REF field from the field system (C1) for evaluation
//! - References captions from the caption system (C3) by bookmark name
//! - References footnotes/endnotes from the footnote system (A3)

use crate::{
    field::{Field, FieldInstruction, RefDisplayType, RefOptions},
    BookmarkRegistry, CaptionLabel, CaptionRegistry, Node, NodeId, NodeType, NoteId, NoteStore,
    Position,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Cross-Reference Types
// =============================================================================

/// The type of element being referenced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossRefType {
    /// Reference to a heading paragraph
    Heading,
    /// Reference to a bookmark
    Bookmark,
    /// Reference to a footnote
    Footnote,
    /// Reference to an endnote
    Endnote,
    /// Reference to an equation caption
    Equation,
    /// Reference to a figure caption
    Figure,
    /// Reference to a table caption
    Table,
    /// Reference to a custom caption label
    CustomCaption,
}

impl CrossRefType {
    /// Get a display name for the reference type
    pub fn display_name(&self) -> &str {
        match self {
            CrossRefType::Heading => "Heading",
            CrossRefType::Bookmark => "Bookmark",
            CrossRefType::Footnote => "Footnote",
            CrossRefType::Endnote => "Endnote",
            CrossRefType::Equation => "Equation",
            CrossRefType::Figure => "Figure",
            CrossRefType::Table => "Table",
            CrossRefType::CustomCaption => "Custom Caption",
        }
    }

    /// Check if this is a caption type
    pub fn is_caption(&self) -> bool {
        matches!(
            self,
            CrossRefType::Equation
                | CrossRefType::Figure
                | CrossRefType::Table
                | CrossRefType::CustomCaption
        )
    }

    /// Check if this is a note type (footnote/endnote)
    pub fn is_note(&self) -> bool {
        matches!(self, CrossRefType::Footnote | CrossRefType::Endnote)
    }

    /// Convert to CaptionLabel if this is a caption type
    pub fn to_caption_label(&self) -> Option<CaptionLabel> {
        match self {
            CrossRefType::Figure => Some(CaptionLabel::Figure),
            CrossRefType::Table => Some(CaptionLabel::Table),
            CrossRefType::Equation => Some(CaptionLabel::Equation),
            _ => None,
        }
    }
}

// =============================================================================
// Cross-Reference Display Options
// =============================================================================

/// How the cross-reference should be displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CrossRefDisplay {
    /// Display the full text content (e.g., "Chapter 1" or "Figure 1: Title")
    #[default]
    Text,
    /// Display only the number (e.g., "1" or "1-1")
    Number,
    /// Display the page number where the target is located (e.g., "page 5")
    PageNumber,
    /// Display relative position (e.g., "above" or "below")
    AboveBelow,
    /// Display full caption text (e.g., "Figure 1: The complete title")
    FullCaption,
    /// Display label and number only (e.g., "Figure 1")
    LabelAndNumber,
    /// Display the paragraph number in context (e.g., "1.2.3")
    ParagraphNumber,
    /// Display paragraph number without context (e.g., "3")
    ParagraphNumberNoContext,
}

impl CrossRefDisplay {
    /// Convert to RefDisplayType for field evaluation
    pub fn to_ref_display_type(&self) -> RefDisplayType {
        match self {
            CrossRefDisplay::Text => RefDisplayType::Content,
            CrossRefDisplay::Number => RefDisplayType::Content, // Will need custom handling
            CrossRefDisplay::PageNumber => RefDisplayType::PageNumber,
            CrossRefDisplay::AboveBelow => RefDisplayType::RelativePosition,
            CrossRefDisplay::FullCaption => RefDisplayType::Content,
            CrossRefDisplay::LabelAndNumber => RefDisplayType::Content,
            CrossRefDisplay::ParagraphNumber => RefDisplayType::ParagraphNumberFullContext,
            CrossRefDisplay::ParagraphNumberNoContext => RefDisplayType::ParagraphNumber,
        }
    }

    /// Get a display name for UI
    pub fn display_name(&self) -> &str {
        match self {
            CrossRefDisplay::Text => "Heading/Bookmark Text",
            CrossRefDisplay::Number => "Number Only",
            CrossRefDisplay::PageNumber => "Page Number",
            CrossRefDisplay::AboveBelow => "Above/Below",
            CrossRefDisplay::FullCaption => "Entire Caption",
            CrossRefDisplay::LabelAndNumber => "Label and Number",
            CrossRefDisplay::ParagraphNumber => "Paragraph Number (Full Context)",
            CrossRefDisplay::ParagraphNumberNoContext => "Paragraph Number (No Context)",
        }
    }

    /// Get available display options for a given reference type
    pub fn available_for_type(ref_type: CrossRefType) -> Vec<CrossRefDisplay> {
        match ref_type {
            CrossRefType::Heading => vec![
                CrossRefDisplay::Text,
                CrossRefDisplay::PageNumber,
                CrossRefDisplay::AboveBelow,
                CrossRefDisplay::ParagraphNumber,
                CrossRefDisplay::ParagraphNumberNoContext,
            ],
            CrossRefType::Bookmark => vec![
                CrossRefDisplay::Text,
                CrossRefDisplay::PageNumber,
                CrossRefDisplay::AboveBelow,
                CrossRefDisplay::ParagraphNumber,
            ],
            CrossRefType::Footnote | CrossRefType::Endnote => vec![
                CrossRefDisplay::Number,
                CrossRefDisplay::PageNumber,
                CrossRefDisplay::AboveBelow,
            ],
            CrossRefType::Figure | CrossRefType::Table | CrossRefType::Equation => vec![
                CrossRefDisplay::FullCaption,
                CrossRefDisplay::LabelAndNumber,
                CrossRefDisplay::Number,
                CrossRefDisplay::PageNumber,
                CrossRefDisplay::AboveBelow,
            ],
            CrossRefType::CustomCaption => vec![
                CrossRefDisplay::FullCaption,
                CrossRefDisplay::LabelAndNumber,
                CrossRefDisplay::Number,
                CrossRefDisplay::PageNumber,
                CrossRefDisplay::AboveBelow,
            ],
        }
    }
}

// =============================================================================
// Cross-Reference
// =============================================================================

/// A cross-reference to another part of the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Unique identifier for this cross-reference node
    id: NodeId,
    /// Parent node (typically a run or paragraph)
    parent: Option<NodeId>,
    /// Type of element being referenced
    pub ref_type: CrossRefType,
    /// Target identifier (bookmark name, heading ID, or caption bookmark)
    pub target_id: String,
    /// How to display the reference
    pub display: CrossRefDisplay,
    /// Whether to include a hyperlink to the target
    pub include_hyperlink: bool,
    /// The underlying REF field ID (for field evaluation)
    pub field_id: Option<NodeId>,
    /// Cached display text (updated during field evaluation)
    pub cached_text: Option<String>,
    /// Whether the reference is broken (target not found)
    pub is_broken: bool,
    /// Error message if broken
    pub error_message: Option<String>,
    /// Custom caption label name (for CustomCaption type)
    pub custom_label: Option<String>,
}

impl CrossReference {
    /// Create a new cross-reference
    pub fn new(ref_type: CrossRefType, target_id: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            ref_type,
            target_id: target_id.into(),
            display: CrossRefDisplay::default(),
            include_hyperlink: true,
            field_id: None,
            cached_text: None,
            is_broken: false,
            error_message: None,
            custom_label: None,
        }
    }

    /// Create a heading reference
    pub fn heading(heading_id: impl Into<String>) -> Self {
        Self::new(CrossRefType::Heading, heading_id)
    }

    /// Create a bookmark reference
    pub fn bookmark(bookmark_name: impl Into<String>) -> Self {
        Self::new(CrossRefType::Bookmark, bookmark_name)
    }

    /// Create a footnote reference
    pub fn footnote(note_id: NoteId) -> Self {
        Self::new(CrossRefType::Footnote, note_id.to_string())
    }

    /// Create an endnote reference
    pub fn endnote(note_id: NoteId) -> Self {
        Self::new(CrossRefType::Endnote, note_id.to_string())
    }

    /// Create a figure caption reference
    pub fn figure(caption_bookmark: impl Into<String>) -> Self {
        Self::new(CrossRefType::Figure, caption_bookmark)
    }

    /// Create a table caption reference
    pub fn table(caption_bookmark: impl Into<String>) -> Self {
        Self::new(CrossRefType::Table, caption_bookmark)
    }

    /// Create an equation caption reference
    pub fn equation(caption_bookmark: impl Into<String>) -> Self {
        Self::new(CrossRefType::Equation, caption_bookmark)
    }

    /// Create a custom caption reference
    pub fn custom_caption(label: impl Into<String>, caption_bookmark: impl Into<String>) -> Self {
        let mut crossref = Self::new(CrossRefType::CustomCaption, caption_bookmark);
        crossref.custom_label = Some(label.into());
        crossref
    }

    /// Set the display format
    pub fn with_display(mut self, display: CrossRefDisplay) -> Self {
        self.display = display;
        self
    }

    /// Set whether to include a hyperlink
    pub fn with_hyperlink(mut self, include: bool) -> Self {
        self.include_hyperlink = include;
        self
    }

    /// Get the cross-reference ID
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the target ID
    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    /// Set the target ID
    pub fn set_target_id(&mut self, target_id: impl Into<String>) {
        self.target_id = target_id.into();
        self.is_broken = false;
        self.error_message = None;
        self.cached_text = None;
    }

    /// Get the display text (cached or placeholder)
    pub fn display_text(&self) -> &str {
        if let Some(ref text) = self.cached_text {
            text.as_str()
        } else if self.is_broken {
            "Error! Reference source not found."
        } else {
            "[...]"
        }
    }

    /// Set the cached display text
    pub fn set_cached_text(&mut self, text: impl Into<String>) {
        self.cached_text = Some(text.into());
    }

    /// Mark this reference as broken
    pub fn mark_broken(&mut self, message: impl Into<String>) {
        self.is_broken = true;
        self.error_message = Some(message.into());
        self.cached_text = Some("Error! Reference source not found.".to_string());
    }

    /// Mark this reference as valid
    pub fn mark_valid(&mut self) {
        self.is_broken = false;
        self.error_message = None;
    }

    /// Check if this reference needs updating
    pub fn needs_update(&self) -> bool {
        self.cached_text.is_none() || self.is_broken
    }

    /// Create a REF field for this cross-reference
    pub fn create_ref_field(&self) -> Field {
        let options = RefOptions {
            bookmark: self.target_id.clone(),
            display: self.display.to_ref_display_type(),
            hyperlink: self.include_hyperlink,
            include_position: self.display == CrossRefDisplay::AboveBelow,
        };

        let mut field = Field::new(FieldInstruction::Ref { options });

        // Copy cached text to field if available
        if let Some(ref text) = self.cached_text {
            field.set_result(text.clone());
        }

        field
    }

    /// Get the bookmark name for this reference (for linking)
    pub fn get_bookmark_name(&self) -> &str {
        &self.target_id
    }
}

impl Node for CrossReference {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Field // Cross-references are rendered as fields
    }

    fn children(&self) -> &[NodeId] {
        &[] // Cross-references have no children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        false
    }

    fn text_content(&self) -> Option<&str> {
        self.cached_text.as_deref()
    }
}

// =============================================================================
// Available Target
// =============================================================================

/// A potential cross-reference target available in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableTarget {
    /// The target identifier (bookmark name, heading ID, etc.)
    pub id: String,
    /// Display text for the target
    pub display_text: String,
    /// The type of target
    pub target_type: CrossRefType,
    /// Page number where the target is located (if known)
    pub page_number: Option<u32>,
    /// Preview text for the reference (what it would display)
    pub preview: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AvailableTarget {
    /// Create a new available target
    pub fn new(
        id: impl Into<String>,
        display_text: impl Into<String>,
        target_type: CrossRefType,
    ) -> Self {
        let display = display_text.into();
        Self {
            id: id.into(),
            preview: display.clone(),
            display_text: display,
            target_type,
            page_number: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the page number
    pub fn with_page(mut self, page: u32) -> Self {
        self.page_number = Some(page);
        self
    }

    /// Set the preview text
    pub fn with_preview(mut self, preview: impl Into<String>) -> Self {
        self.preview = preview.into();
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// =============================================================================
// Broken Reference
// =============================================================================

/// Information about a broken cross-reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenReference {
    /// The cross-reference ID
    pub ref_id: NodeId,
    /// The type of reference
    pub ref_type: CrossRefType,
    /// The target ID that was not found
    pub target_id: String,
    /// Error message
    pub error_message: String,
    /// Position in the document (if known)
    pub position: Option<Position>,
    /// Suggested fixes (alternative targets)
    pub suggested_targets: Vec<String>,
}

// =============================================================================
// Cross-Reference Registry
// =============================================================================

/// Registry for managing cross-references in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrossRefRegistry {
    /// All cross-references indexed by ID
    references: HashMap<NodeId, CrossReference>,
    /// Index by target ID for tracking updates
    target_index: HashMap<String, Vec<NodeId>>,
    /// Cross-references marked as needing update
    dirty_refs: Vec<NodeId>,
    /// Index of broken references
    broken_refs: Vec<NodeId>,
}

impl CrossRefRegistry {
    /// Create a new cross-reference registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a cross-reference
    pub fn insert(&mut self, crossref: CrossReference) -> NodeId {
        let id = crossref.id;
        let target_id = crossref.target_id.clone();

        // Add to target index
        self.target_index
            .entry(target_id)
            .or_default()
            .push(id);

        // Mark as dirty initially
        self.dirty_refs.push(id);

        // Check if broken
        if crossref.is_broken {
            self.broken_refs.push(id);
        }

        self.references.insert(id, crossref);
        id
    }

    /// Remove a cross-reference
    pub fn remove(&mut self, id: NodeId) -> Option<CrossReference> {
        if let Some(crossref) = self.references.remove(&id) {
            // Remove from target index
            if let Some(refs) = self.target_index.get_mut(&crossref.target_id) {
                refs.retain(|&r| r != id);
                if refs.is_empty() {
                    self.target_index.remove(&crossref.target_id);
                }
            }

            // Remove from dirty list
            self.dirty_refs.retain(|&r| r != id);

            // Remove from broken list
            self.broken_refs.retain(|&r| r != id);

            Some(crossref)
        } else {
            None
        }
    }

    /// Get a cross-reference by ID
    pub fn get(&self, id: NodeId) -> Option<&CrossReference> {
        self.references.get(&id)
    }

    /// Get a mutable cross-reference by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut CrossReference> {
        self.references.get_mut(&id)
    }

    /// Get all cross-references
    pub fn all(&self) -> impl Iterator<Item = &CrossReference> {
        self.references.values()
    }

    /// Get all cross-reference IDs
    pub fn all_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.references.keys().copied()
    }

    /// Get cross-references by type
    pub fn by_type(&self, ref_type: CrossRefType) -> Vec<&CrossReference> {
        self.references
            .values()
            .filter(|r| r.ref_type == ref_type)
            .collect()
    }

    /// Get cross-references that reference a specific target
    pub fn by_target(&self, target_id: &str) -> Vec<&CrossReference> {
        self.target_index
            .get(target_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.references.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get IDs of cross-references that reference a specific target
    pub fn ids_by_target(&self, target_id: &str) -> Vec<NodeId> {
        self.target_index
            .get(target_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Mark a cross-reference as dirty (needs update)
    pub fn mark_dirty(&mut self, id: NodeId) {
        if !self.dirty_refs.contains(&id) {
            self.dirty_refs.push(id);
        }
    }

    /// Mark all cross-references as dirty
    pub fn mark_all_dirty(&mut self) {
        self.dirty_refs.clear();
        for id in self.references.keys() {
            self.dirty_refs.push(*id);
        }
    }

    /// Get dirty cross-reference IDs
    pub fn dirty_refs(&self) -> &[NodeId] {
        &self.dirty_refs
    }

    /// Clear dirty list
    pub fn clear_dirty(&mut self) {
        self.dirty_refs.clear();
    }

    /// Get broken cross-reference IDs
    pub fn broken_refs(&self) -> &[NodeId] {
        &self.broken_refs
    }

    /// Get broken references with details
    pub fn get_broken_references(&self) -> Vec<BrokenReference> {
        self.broken_refs
            .iter()
            .filter_map(|&id| {
                self.references.get(&id).map(|r| BrokenReference {
                    ref_id: id,
                    ref_type: r.ref_type,
                    target_id: r.target_id.clone(),
                    error_message: r
                        .error_message
                        .clone()
                        .unwrap_or_else(|| "Target not found".to_string()),
                    position: None, // Would need document context
                    suggested_targets: Vec::new(), // Would need to compute
                })
            })
            .collect()
    }

    /// Update broken status for a reference
    pub fn update_broken_status(&mut self, id: NodeId, is_broken: bool, message: Option<String>) {
        if let Some(crossref) = self.references.get_mut(&id) {
            if is_broken {
                crossref.mark_broken(message.unwrap_or_else(|| "Target not found".to_string()));
                if !self.broken_refs.contains(&id) {
                    self.broken_refs.push(id);
                }
            } else {
                crossref.mark_valid();
                self.broken_refs.retain(|&r| r != id);
            }
        }
    }

    /// Handle a target being renamed (update all references to it)
    pub fn handle_target_renamed(&mut self, old_target_id: &str, new_target_id: &str) {
        if let Some(ref_ids) = self.target_index.remove(old_target_id) {
            for ref_id in &ref_ids {
                if let Some(crossref) = self.references.get_mut(ref_id) {
                    crossref.target_id = new_target_id.to_string();
                    // Mark as dirty to update display text
                    if !self.dirty_refs.contains(ref_id) {
                        self.dirty_refs.push(*ref_id);
                    }
                }
            }
            self.target_index
                .insert(new_target_id.to_string(), ref_ids);
        }
    }

    /// Handle a target being deleted (mark all references as broken)
    pub fn handle_target_deleted(&mut self, target_id: &str) {
        if let Some(ref_ids) = self.target_index.get(target_id) {
            for ref_id in ref_ids {
                if let Some(crossref) = self.references.get_mut(ref_id) {
                    crossref.mark_broken("Target has been deleted");
                    if !self.broken_refs.contains(ref_id) {
                        self.broken_refs.push(*ref_id);
                    }
                }
            }
        }
    }

    /// Number of cross-references
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    /// Check if there are any broken references
    pub fn has_broken_refs(&self) -> bool {
        !self.broken_refs.is_empty()
    }

    /// Number of broken references
    pub fn broken_count(&self) -> usize {
        self.broken_refs.len()
    }
}

// =============================================================================
// Cross-Reference Validator
// =============================================================================

/// Validates cross-references against available targets
pub struct CrossRefValidator;

impl CrossRefValidator {
    /// Validate a cross-reference against available targets
    pub fn validate(
        crossref: &CrossReference,
        bookmarks: &BookmarkRegistry,
        captions: &CaptionRegistry,
        notes: &NoteStore,
        headings: &[AvailableTarget],
    ) -> Result<(), String> {
        match crossref.ref_type {
            CrossRefType::Bookmark => {
                if bookmarks.contains_name(&crossref.target_id) {
                    Ok(())
                } else {
                    Err(format!("Bookmark '{}' not found", crossref.target_id))
                }
            }
            CrossRefType::Heading => {
                if headings.iter().any(|h| h.id == crossref.target_id) {
                    Ok(())
                } else {
                    Err(format!("Heading '{}' not found", crossref.target_id))
                }
            }
            CrossRefType::Footnote => {
                if let Some(note_id) = NoteId::from_string(&crossref.target_id) {
                    if notes.get_footnote(note_id).is_some() {
                        Ok(())
                    } else {
                        Err(format!("Footnote '{}' not found", crossref.target_id))
                    }
                } else {
                    Err("Invalid footnote ID".to_string())
                }
            }
            CrossRefType::Endnote => {
                if let Some(note_id) = NoteId::from_string(&crossref.target_id) {
                    if notes.get_endnote(note_id).is_some() {
                        Ok(())
                    } else {
                        Err(format!("Endnote '{}' not found", crossref.target_id))
                    }
                } else {
                    Err("Invalid endnote ID".to_string())
                }
            }
            CrossRefType::Figure | CrossRefType::Table | CrossRefType::Equation => {
                // Captions use bookmark names
                if bookmarks.contains_name(&crossref.target_id) {
                    Ok(())
                } else {
                    // Also check if any caption has this bookmark name
                    let found = captions
                        .all()
                        .any(|c| c.bookmark_name() == crossref.target_id);
                    if found {
                        Ok(())
                    } else {
                        Err(format!("Caption '{}' not found", crossref.target_id))
                    }
                }
            }
            CrossRefType::CustomCaption => {
                if bookmarks.contains_name(&crossref.target_id) {
                    Ok(())
                } else {
                    let found = captions
                        .all()
                        .any(|c| c.bookmark_name() == crossref.target_id);
                    if found {
                        Ok(())
                    } else {
                        Err(format!(
                            "Custom caption '{}' not found",
                            crossref.target_id
                        ))
                    }
                }
            }
        }
    }

    /// Validate all cross-references in a registry
    pub fn validate_all(
        registry: &mut CrossRefRegistry,
        bookmarks: &BookmarkRegistry,
        captions: &CaptionRegistry,
        notes: &NoteStore,
        headings: &[AvailableTarget],
    ) -> Vec<BrokenReference> {
        let mut broken = Vec::new();

        let ids: Vec<NodeId> = registry.all_ids().collect();
        for id in ids {
            if let Some(crossref) = registry.get(id) {
                let crossref_clone = crossref.clone();
                let result =
                    Self::validate(&crossref_clone, bookmarks, captions, notes, headings);

                match result {
                    Ok(()) => {
                        registry.update_broken_status(id, false, None);
                    }
                    Err(message) => {
                        registry.update_broken_status(id, true, Some(message.clone()));
                        broken.push(BrokenReference {
                            ref_id: id,
                            ref_type: crossref_clone.ref_type,
                            target_id: crossref_clone.target_id.clone(),
                            error_message: message,
                            position: None,
                            suggested_targets: Vec::new(),
                        });
                    }
                }
            }
        }

        broken
    }
}

// =============================================================================
// Target Discovery
// =============================================================================

/// Discovers available cross-reference targets from document state
pub struct TargetDiscovery;

impl TargetDiscovery {
    /// Get all available heading targets
    pub fn get_headings(
        _headings: &[(NodeId, String, u8)], // (paragraph_id, text, level)
    ) -> Vec<AvailableTarget> {
        _headings
            .iter()
            .map(|(id, text, level)| {
                let bookmark_name = format!("_Heading_{}", id);
                AvailableTarget::new(&bookmark_name, text.clone(), CrossRefType::Heading)
                    .with_metadata("level", level.to_string())
            })
            .collect()
    }

    /// Get all available bookmark targets
    pub fn get_bookmarks(bookmarks: &BookmarkRegistry) -> Vec<AvailableTarget> {
        bookmarks
            .all()
            .map(|b| {
                AvailableTarget::new(b.name(), b.name(), CrossRefType::Bookmark)
            })
            .collect()
    }

    /// Get all available caption targets for a specific type
    pub fn get_captions(
        captions: &CaptionRegistry,
        ref_type: CrossRefType,
    ) -> Vec<AvailableTarget> {
        let label = match ref_type {
            CrossRefType::Figure => CaptionLabel::Figure,
            CrossRefType::Table => CaptionLabel::Table,
            CrossRefType::Equation => CaptionLabel::Equation,
            _ => return Vec::new(),
        };

        captions
            .by_label(&label)
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let number = i + 1;
                let display = format!("{} {}: {}", label.display_text(), number, c.text());
                AvailableTarget::new(c.bookmark_name(), display, ref_type)
                    .with_metadata("number", number.to_string())
                    .with_preview(format!("{} {}", label.display_text(), number))
            })
            .collect()
    }

    /// Get all available footnote targets
    pub fn get_footnotes(notes: &NoteStore) -> Vec<AvailableTarget> {
        notes
            .footnotes()
            .map(|n| {
                AvailableTarget::new(
                    n.id().to_string(),
                    format!("Footnote {}", n.mark()),
                    CrossRefType::Footnote,
                )
                .with_metadata("mark", n.mark().to_string())
            })
            .collect()
    }

    /// Get all available endnote targets
    pub fn get_endnotes(notes: &NoteStore) -> Vec<AvailableTarget> {
        notes
            .endnotes()
            .map(|n| {
                AvailableTarget::new(
                    n.id().to_string(),
                    format!("Endnote {}", n.mark()),
                    CrossRefType::Endnote,
                )
                .with_metadata("mark", n.mark().to_string())
            })
            .collect()
    }

    /// Get all available targets for a given type
    pub fn get_targets_for_type(
        ref_type: CrossRefType,
        bookmarks: &BookmarkRegistry,
        captions: &CaptionRegistry,
        notes: &NoteStore,
        headings: &[(NodeId, String, u8)],
    ) -> Vec<AvailableTarget> {
        match ref_type {
            CrossRefType::Heading => Self::get_headings(headings),
            CrossRefType::Bookmark => Self::get_bookmarks(bookmarks),
            CrossRefType::Footnote => Self::get_footnotes(notes),
            CrossRefType::Endnote => Self::get_endnotes(notes),
            CrossRefType::Figure | CrossRefType::Table | CrossRefType::Equation => {
                Self::get_captions(captions, ref_type)
            }
            CrossRefType::CustomCaption => {
                // Would need custom label registry
                Vec::new()
            }
        }
    }
}

// =============================================================================
// Cross-Reference Updater
// =============================================================================

/// Updates cross-reference display text based on current document state
pub struct CrossRefUpdater;

impl CrossRefUpdater {
    /// Generate display text for a cross-reference
    pub fn generate_display_text(
        crossref: &CrossReference,
        bookmarks: &BookmarkRegistry,
        captions: &CaptionRegistry,
        notes: &NoteStore,
        page_numbers: &HashMap<String, u32>,
        current_position: Option<Position>,
    ) -> String {
        match crossref.ref_type {
            CrossRefType::Bookmark => {
                Self::generate_bookmark_text(crossref, bookmarks, page_numbers, current_position)
            }
            CrossRefType::Heading => {
                Self::generate_heading_text(crossref, bookmarks, page_numbers, current_position)
            }
            CrossRefType::Footnote | CrossRefType::Endnote => {
                Self::generate_note_text(crossref, notes, page_numbers, current_position)
            }
            CrossRefType::Figure | CrossRefType::Table | CrossRefType::Equation => {
                Self::generate_caption_text(crossref, captions, page_numbers, current_position)
            }
            CrossRefType::CustomCaption => {
                Self::generate_caption_text(crossref, captions, page_numbers, current_position)
            }
        }
    }

    fn generate_bookmark_text(
        crossref: &CrossReference,
        bookmarks: &BookmarkRegistry,
        page_numbers: &HashMap<String, u32>,
        _current_position: Option<Position>,
    ) -> String {
        match crossref.display {
            CrossRefDisplay::PageNumber => {
                if let Some(&page) = page_numbers.get(&crossref.target_id) {
                    page.to_string()
                } else {
                    "?".to_string()
                }
            }
            CrossRefDisplay::AboveBelow => {
                // Would need position comparison
                "above".to_string()
            }
            _ => {
                // Text content
                if let Some(bookmark) = bookmarks.get_by_name(&crossref.target_id) {
                    // Would need document access to get the text at bookmark position
                    format!("[{}]", bookmark.name())
                } else {
                    "Error! Bookmark not found.".to_string()
                }
            }
        }
    }

    fn generate_heading_text(
        crossref: &CrossReference,
        _bookmarks: &BookmarkRegistry,
        page_numbers: &HashMap<String, u32>,
        _current_position: Option<Position>,
    ) -> String {
        match crossref.display {
            CrossRefDisplay::PageNumber => {
                if let Some(&page) = page_numbers.get(&crossref.target_id) {
                    page.to_string()
                } else {
                    "?".to_string()
                }
            }
            CrossRefDisplay::AboveBelow => "above".to_string(),
            CrossRefDisplay::ParagraphNumber => "[#.#.#]".to_string(),
            CrossRefDisplay::ParagraphNumberNoContext => "[#]".to_string(),
            _ => {
                // Would need document access to get heading text
                "[Heading]".to_string()
            }
        }
    }

    fn generate_note_text(
        crossref: &CrossReference,
        notes: &NoteStore,
        page_numbers: &HashMap<String, u32>,
        _current_position: Option<Position>,
    ) -> String {
        let note_id = NoteId::from_string(&crossref.target_id);

        let note = note_id.and_then(|id| match crossref.ref_type {
            CrossRefType::Footnote => notes.get_footnote(id),
            CrossRefType::Endnote => notes.get_endnote(id),
            _ => None,
        });

        match crossref.display {
            CrossRefDisplay::Number => {
                if let Some(n) = note {
                    n.mark().to_string()
                } else {
                    "?".to_string()
                }
            }
            CrossRefDisplay::PageNumber => {
                if let Some(&page) = page_numbers.get(&crossref.target_id) {
                    page.to_string()
                } else {
                    "?".to_string()
                }
            }
            CrossRefDisplay::AboveBelow => "above".to_string(),
            _ => {
                if let Some(n) = note {
                    n.mark().to_string()
                } else {
                    "?".to_string()
                }
            }
        }
    }

    fn generate_caption_text(
        crossref: &CrossReference,
        captions: &CaptionRegistry,
        page_numbers: &HashMap<String, u32>,
        _current_position: Option<Position>,
    ) -> String {
        // Find the caption by bookmark name
        let caption = captions.all().find(|c| c.bookmark_name() == crossref.target_id);

        let label = match crossref.ref_type {
            CrossRefType::Figure => CaptionLabel::Figure,
            CrossRefType::Table => CaptionLabel::Table,
            CrossRefType::Equation => CaptionLabel::Equation,
            CrossRefType::CustomCaption => {
                if let Some(label_name) = &crossref.custom_label {
                    CaptionLabel::Custom(label_name.clone())
                } else {
                    CaptionLabel::Figure // Fallback
                }
            }
            _ => CaptionLabel::Figure,
        };

        match crossref.display {
            CrossRefDisplay::FullCaption => {
                if let Some(c) = caption {
                    let number = captions.get_caption_number(c.id()).unwrap_or(0);
                    let format = captions
                        .get_format(&label)
                        .cloned()
                        .unwrap_or_else(|| crate::caption::CaptionFormat::new(label.clone()));
                    format!(
                        "{} {}{}{}",
                        label.display_text(),
                        format.format_number(number),
                        format.separator,
                        c.text()
                    )
                } else {
                    "Error! Caption not found.".to_string()
                }
            }
            CrossRefDisplay::LabelAndNumber => {
                if let Some(c) = caption {
                    let number = captions.get_caption_number(c.id()).unwrap_or(0);
                    let format = captions
                        .get_format(&label)
                        .cloned()
                        .unwrap_or_else(|| crate::caption::CaptionFormat::new(label.clone()));
                    format!("{} {}", label.display_text(), format.format_number(number))
                } else {
                    "Error! Caption not found.".to_string()
                }
            }
            CrossRefDisplay::Number => {
                if let Some(c) = caption {
                    let number = captions.get_caption_number(c.id()).unwrap_or(0);
                    let format = captions
                        .get_format(&label)
                        .cloned()
                        .unwrap_or_else(|| crate::caption::CaptionFormat::new(label.clone()));
                    format.format_number(number)
                } else {
                    "?".to_string()
                }
            }
            CrossRefDisplay::PageNumber => {
                if let Some(&page) = page_numbers.get(&crossref.target_id) {
                    page.to_string()
                } else {
                    "?".to_string()
                }
            }
            CrossRefDisplay::AboveBelow => "above".to_string(),
            _ => {
                // Default to label and number
                if let Some(c) = caption {
                    let number = captions.get_caption_number(c.id()).unwrap_or(0);
                    let format = captions
                        .get_format(&label)
                        .cloned()
                        .unwrap_or_else(|| crate::caption::CaptionFormat::new(label.clone()));
                    format!("{} {}", label.display_text(), format.format_number(number))
                } else {
                    "Error! Caption not found.".to_string()
                }
            }
        }
    }

    /// Update all cross-references in a registry
    pub fn update_all(
        registry: &mut CrossRefRegistry,
        bookmarks: &BookmarkRegistry,
        captions: &CaptionRegistry,
        notes: &NoteStore,
        page_numbers: &HashMap<String, u32>,
    ) {
        let ids: Vec<NodeId> = registry.dirty_refs().to_vec();

        for id in ids {
            if let Some(crossref) = registry.get(id) {
                let crossref_clone = crossref.clone();
                let text = Self::generate_display_text(
                    &crossref_clone,
                    bookmarks,
                    captions,
                    notes,
                    page_numbers,
                    None,
                );

                if let Some(crossref) = registry.get_mut(id) {
                    crossref.set_cached_text(text);
                }
            }
        }

        registry.clear_dirty();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_ref_type_display_name() {
        assert_eq!(CrossRefType::Heading.display_name(), "Heading");
        assert_eq!(CrossRefType::Figure.display_name(), "Figure");
        assert_eq!(CrossRefType::Footnote.display_name(), "Footnote");
    }

    #[test]
    fn test_cross_ref_type_is_caption() {
        assert!(CrossRefType::Figure.is_caption());
        assert!(CrossRefType::Table.is_caption());
        assert!(CrossRefType::Equation.is_caption());
        assert!(!CrossRefType::Heading.is_caption());
        assert!(!CrossRefType::Bookmark.is_caption());
    }

    #[test]
    fn test_cross_ref_type_is_note() {
        assert!(CrossRefType::Footnote.is_note());
        assert!(CrossRefType::Endnote.is_note());
        assert!(!CrossRefType::Heading.is_note());
        assert!(!CrossRefType::Figure.is_note());
    }

    #[test]
    fn test_cross_ref_display_available_for_type() {
        let heading_displays = CrossRefDisplay::available_for_type(CrossRefType::Heading);
        assert!(heading_displays.contains(&CrossRefDisplay::Text));
        assert!(heading_displays.contains(&CrossRefDisplay::PageNumber));

        let figure_displays = CrossRefDisplay::available_for_type(CrossRefType::Figure);
        assert!(figure_displays.contains(&CrossRefDisplay::FullCaption));
        assert!(figure_displays.contains(&CrossRefDisplay::LabelAndNumber));
    }

    #[test]
    fn test_cross_reference_creation() {
        let crossref = CrossReference::heading("heading_1");
        assert_eq!(crossref.ref_type, CrossRefType::Heading);
        assert_eq!(crossref.target_id, "heading_1");
        assert!(crossref.include_hyperlink);

        let crossref = CrossReference::figure("_RefFigure_123")
            .with_display(CrossRefDisplay::LabelAndNumber)
            .with_hyperlink(false);
        assert_eq!(crossref.ref_type, CrossRefType::Figure);
        assert_eq!(crossref.display, CrossRefDisplay::LabelAndNumber);
        assert!(!crossref.include_hyperlink);
    }

    #[test]
    fn test_cross_reference_bookmark() {
        let crossref = CrossReference::bookmark("my_bookmark");
        assert_eq!(crossref.ref_type, CrossRefType::Bookmark);
        assert_eq!(crossref.target_id, "my_bookmark");
    }

    #[test]
    fn test_cross_reference_footnote() {
        let note_id = NoteId::new();
        let crossref = CrossReference::footnote(note_id);
        assert_eq!(crossref.ref_type, CrossRefType::Footnote);
        assert_eq!(crossref.target_id, note_id.to_string());
    }

    #[test]
    fn test_cross_reference_broken_status() {
        let mut crossref = CrossReference::heading("heading_1");
        assert!(!crossref.is_broken);

        crossref.mark_broken("Target not found");
        assert!(crossref.is_broken);
        assert_eq!(crossref.error_message, Some("Target not found".to_string()));
        assert!(crossref.display_text().contains("Error"));

        crossref.mark_valid();
        assert!(!crossref.is_broken);
        assert!(crossref.error_message.is_none());
    }

    #[test]
    fn test_cross_reference_cached_text() {
        let mut crossref = CrossReference::heading("heading_1");
        assert!(crossref.cached_text.is_none());

        crossref.set_cached_text("Chapter 1");
        assert_eq!(crossref.display_text(), "Chapter 1");
    }

    #[test]
    fn test_cross_ref_registry_insert_remove() {
        let mut registry = CrossRefRegistry::new();

        let crossref = CrossReference::heading("heading_1");
        let id = crossref.id();
        registry.insert(crossref);

        assert_eq!(registry.len(), 1);
        assert!(registry.get(id).is_some());

        let removed = registry.remove(id);
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_cross_ref_registry_by_target() {
        let mut registry = CrossRefRegistry::new();

        let crossref1 = CrossReference::heading("heading_1");
        let crossref2 = CrossReference::heading("heading_1");
        let crossref3 = CrossReference::heading("heading_2");

        registry.insert(crossref1);
        registry.insert(crossref2);
        registry.insert(crossref3);

        let refs_to_h1 = registry.by_target("heading_1");
        assert_eq!(refs_to_h1.len(), 2);

        let refs_to_h2 = registry.by_target("heading_2");
        assert_eq!(refs_to_h2.len(), 1);
    }

    #[test]
    fn test_cross_ref_registry_by_type() {
        let mut registry = CrossRefRegistry::new();

        registry.insert(CrossReference::heading("h1"));
        registry.insert(CrossReference::heading("h2"));
        registry.insert(CrossReference::figure("fig1"));
        registry.insert(CrossReference::bookmark("bm1"));

        let headings = registry.by_type(CrossRefType::Heading);
        assert_eq!(headings.len(), 2);

        let figures = registry.by_type(CrossRefType::Figure);
        assert_eq!(figures.len(), 1);
    }

    #[test]
    fn test_cross_ref_registry_handle_target_renamed() {
        let mut registry = CrossRefRegistry::new();

        let crossref = CrossReference::heading("old_name");
        let id = crossref.id();
        registry.insert(crossref);

        registry.handle_target_renamed("old_name", "new_name");

        let crossref = registry.get(id).unwrap();
        assert_eq!(crossref.target_id, "new_name");
        assert!(registry.dirty_refs().contains(&id));
    }

    #[test]
    fn test_cross_ref_registry_handle_target_deleted() {
        let mut registry = CrossRefRegistry::new();

        let crossref = CrossReference::heading("to_delete");
        let id = crossref.id();
        registry.insert(crossref);

        registry.handle_target_deleted("to_delete");

        let crossref = registry.get(id).unwrap();
        assert!(crossref.is_broken);
        assert!(registry.has_broken_refs());
    }

    #[test]
    fn test_cross_ref_registry_dirty_refs() {
        let mut registry = CrossRefRegistry::new();

        let crossref = CrossReference::heading("h1");
        let id = crossref.id();
        registry.insert(crossref);

        // Should be dirty after insert
        assert!(registry.dirty_refs().contains(&id));

        registry.clear_dirty();
        assert!(registry.dirty_refs().is_empty());

        registry.mark_dirty(id);
        assert!(registry.dirty_refs().contains(&id));

        registry.mark_all_dirty();
        assert!(!registry.dirty_refs().is_empty());
    }

    #[test]
    fn test_available_target_creation() {
        let target = AvailableTarget::new("target_1", "Chapter 1", CrossRefType::Heading)
            .with_page(5)
            .with_preview("Ch. 1")
            .with_metadata("level", "1");

        assert_eq!(target.id, "target_1");
        assert_eq!(target.display_text, "Chapter 1");
        assert_eq!(target.page_number, Some(5));
        assert_eq!(target.preview, "Ch. 1");
        assert_eq!(target.metadata.get("level"), Some(&"1".to_string()));
    }

    #[test]
    fn test_cross_reference_create_ref_field() {
        let crossref = CrossReference::bookmark("my_bookmark")
            .with_display(CrossRefDisplay::PageNumber)
            .with_hyperlink(true);

        let field = crossref.create_ref_field();

        if let FieldInstruction::Ref { options } = &field.instruction {
            assert_eq!(options.bookmark, "my_bookmark");
            assert!(options.hyperlink);
            assert_eq!(options.display, RefDisplayType::PageNumber);
        } else {
            panic!("Expected REF field instruction");
        }
    }

    #[test]
    fn test_target_discovery_bookmarks() {
        let mut bookmarks = BookmarkRegistry::new();
        bookmarks
            .insert(crate::Bookmark::new_point(
                "bookmark1",
                Position::new(NodeId::new(), 0),
            ))
            .unwrap();
        bookmarks
            .insert(crate::Bookmark::new_point(
                "bookmark2",
                Position::new(NodeId::new(), 0),
            ))
            .unwrap();

        let targets = TargetDiscovery::get_bookmarks(&bookmarks);
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().any(|t| t.id == "bookmark1"));
        assert!(targets.iter().any(|t| t.id == "bookmark2"));
    }

    #[test]
    fn test_custom_caption_reference() {
        let crossref = CrossReference::custom_caption("Listing", "_RefListing_123");
        assert_eq!(crossref.ref_type, CrossRefType::CustomCaption);
        assert_eq!(crossref.custom_label, Some("Listing".to_string()));
        assert_eq!(crossref.target_id, "_RefListing_123");
    }

    #[test]
    fn test_cross_ref_display_to_ref_display_type() {
        assert_eq!(
            CrossRefDisplay::Text.to_ref_display_type(),
            RefDisplayType::Content
        );
        assert_eq!(
            CrossRefDisplay::PageNumber.to_ref_display_type(),
            RefDisplayType::PageNumber
        );
        assert_eq!(
            CrossRefDisplay::AboveBelow.to_ref_display_type(),
            RefDisplayType::RelativePosition
        );
        assert_eq!(
            CrossRefDisplay::ParagraphNumber.to_ref_display_type(),
            RefDisplayType::ParagraphNumberFullContext
        );
    }

    #[test]
    fn test_broken_reference_info() {
        let mut registry = CrossRefRegistry::new();

        let crossref = CrossReference::heading("missing_heading");
        let id = crossref.id();
        registry.insert(crossref);

        registry.update_broken_status(id, true, Some("Heading not found".to_string()));

        let broken = registry.get_broken_references();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].ref_id, id);
        assert_eq!(broken[0].error_message, "Heading not found");
    }
}
