//! Field commands for inserting, updating, and managing document fields
//!
//! Fields are dynamic content placeholders like page numbers, dates, TOC, etc.

use crate::{Command, CommandResult, EditError, Result};
use doc_model::field::{
    Field, FieldContext, FieldEvaluator, FieldInstruction, FieldRegistry, NumberFormat,
    RefDisplayType, RefOptions, SeqOptions, TocEntry, TocSwitches,
};
use doc_model::{DocumentTree, Node, NodeId, Selection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Insert Field Command
// =============================================================================

/// Insert a field at the current position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertField {
    /// The field instruction
    pub instruction: FieldInstruction,
    /// Optional initial result text
    pub initial_text: Option<String>,
}

impl InsertField {
    /// Create a new insert field command
    pub fn new(instruction: FieldInstruction) -> Self {
        Self {
            instruction,
            initial_text: None,
        }
    }

    /// Create a page number field
    pub fn page() -> Self {
        Self::new(FieldInstruction::Page {
            format: NumberFormat::Arabic,
        })
    }

    /// Create a total pages field
    pub fn num_pages() -> Self {
        Self::new(FieldInstruction::NumPages {
            format: NumberFormat::Arabic,
        })
    }

    /// Create a date field
    pub fn date(format: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Date {
            format: format.into(),
        })
    }

    /// Create a time field
    pub fn time(format: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Time {
            format: format.into(),
        })
    }

    /// Create a TOC field
    pub fn toc(switches: TocSwitches) -> Self {
        Self::new(FieldInstruction::Toc { switches })
    }

    /// Create a REF field
    pub fn reference(bookmark: impl Into<String>, display: RefDisplayType) -> Self {
        Self::new(FieldInstruction::Ref {
            options: RefOptions {
                bookmark: bookmark.into(),
                display,
                hyperlink: true,
                include_position: false,
            },
        })
    }

    /// Create a SEQ field
    pub fn seq(identifier: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Seq {
            options: SeqOptions {
                identifier: identifier.into(),
                ..Default::default()
            },
        })
    }

    /// Create an AUTHOR field
    pub fn author() -> Self {
        Self::new(FieldInstruction::Author)
    }

    /// Create a TITLE field
    pub fn title() -> Self {
        Self::new(FieldInstruction::Title)
    }

    /// Set initial text for the field
    pub fn with_initial_text(mut self, text: impl Into<String>) -> Self {
        self.initial_text = Some(text.into());
        self
    }
}

impl Command for InsertField {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Create the field
        let mut field = Field::new(self.instruction.clone());
        if let Some(ref text) = self.initial_text {
            field.set_result(text.clone());
        }

        let field_id = field.id();

        // For now, we'll insert the field into the field registry
        // In a full implementation, fields would be inserted into the document tree
        // as inline elements within paragraphs

        // The inverse command removes the field
        let inverse = Box::new(DeleteField { field_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // Can't properly invert without knowing the field ID
        Box::new(DeleteField {
            field_id: NodeId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Field"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Delete Field Command
// =============================================================================

/// Delete a field from the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteField {
    /// The field ID to delete
    pub field_id: NodeId,
}

impl DeleteField {
    /// Create a new delete field command
    pub fn new(field_id: NodeId) -> Self {
        Self { field_id }
    }
}

impl Command for DeleteField {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let new_tree = tree.clone();

        // In a full implementation, remove the field from the tree
        // For now, this is a placeholder

        let inverse = Box::new(InsertField::new(FieldInstruction::Page {
            format: NumberFormat::Arabic,
        }));

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(InsertField::page())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Field"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Update Field Command
// =============================================================================

/// Update a specific field's result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateField {
    /// The field ID to update
    pub field_id: NodeId,
}

impl UpdateField {
    /// Create a new update field command
    pub fn new(field_id: NodeId) -> Self {
        Self { field_id }
    }
}

impl Command for UpdateField {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let new_tree = tree.clone();

        // In a full implementation:
        // 1. Get the field from the registry
        // 2. Build the field context
        // 3. Evaluate the field
        // 4. Update the cached result

        let inverse = Box::new(UpdateField {
            field_id: self.field_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(UpdateField {
            field_id: self.field_id,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Update Field"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Lock/Unlock Field Command
// =============================================================================

/// Lock or unlock a field for auto-updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFieldLock {
    /// The field ID
    pub field_id: NodeId,
    /// Whether to lock the field
    pub locked: bool,
}

impl SetFieldLock {
    /// Create a lock command
    pub fn lock(field_id: NodeId) -> Self {
        Self {
            field_id,
            locked: true,
        }
    }

    /// Create an unlock command
    pub fn unlock(field_id: NodeId) -> Self {
        Self {
            field_id,
            locked: false,
        }
    }
}

impl Command for SetFieldLock {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let new_tree = tree.clone();

        // In a full implementation, update the field's locked status

        let inverse = Box::new(SetFieldLock {
            field_id: self.field_id,
            locked: !self.locked,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetFieldLock {
            field_id: self.field_id,
            locked: !self.locked,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        if self.locked {
            "Lock Field"
        } else {
            "Unlock Field"
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Toggle Field Codes Command
// =============================================================================

/// Toggle showing field codes vs results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleFieldCodes {
    /// Specific field ID (None = toggle all fields)
    pub field_id: Option<NodeId>,
}

impl ToggleFieldCodes {
    /// Toggle codes for a specific field
    pub fn for_field(field_id: NodeId) -> Self {
        Self {
            field_id: Some(field_id),
        }
    }

    /// Toggle codes for all fields
    pub fn for_all() -> Self {
        Self { field_id: None }
    }
}

impl Command for ToggleFieldCodes {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let new_tree = tree.clone();

        // In a full implementation, toggle the show_code flag on fields

        let inverse = Box::new(ToggleFieldCodes {
            field_id: self.field_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Toggle Field Codes"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Field Update Engine
// =============================================================================

/// Engine for batch updating fields
pub struct FieldUpdateEngine;

impl FieldUpdateEngine {
    /// Update all fields in a field registry
    pub fn update_all(
        registry: &mut FieldRegistry,
        context: &FieldContext,
    ) {
        // Reset sequence counters for consistent numbering
        registry.reset_all_seq();

        // Get all field IDs first to avoid borrow issues
        let field_ids: Vec<NodeId> = registry.all_ids().collect();

        for field_id in field_ids {
            Self::update_field(registry, field_id, context);
        }

        registry.clear_dirty();
    }

    /// Update dirty fields only
    pub fn update_dirty(
        registry: &mut FieldRegistry,
        context: &FieldContext,
    ) {
        let dirty_ids: Vec<NodeId> = registry.dirty_fields().to_vec();

        for field_id in dirty_ids {
            Self::update_field(registry, field_id, context);
        }

        registry.clear_dirty();
    }

    /// Update a single field
    pub fn update_field(
        registry: &mut FieldRegistry,
        field_id: NodeId,
        context: &FieldContext,
    ) {
        // First, get what we need from the field
        let field_info = {
            if let Some(field) = registry.get(field_id) {
                if field.locked {
                    return;
                }
                Some((field.instruction.clone(), FieldEvaluator::evaluate(field, context)))
            } else {
                None
            }
        };

        // Now process based on the field info
        if let Some((instruction, default_result)) = field_info {
            let result = match &instruction {
                FieldInstruction::Seq { options } => {
                    let value = if options.current_only {
                        registry.current_seq(&options.identifier)
                    } else if options.repeat_previous {
                        registry.current_seq(&options.identifier)
                    } else if let Some(reset_val) = options.reset_to {
                        registry.reset_seq(&options.identifier, reset_val);
                        reset_val
                    } else {
                        registry.next_seq(&options.identifier)
                    };
                    options.format.format(value)
                }
                _ => default_result,
            };

            // Update the field result
            if let Some(field) = registry.get_mut(field_id) {
                field.set_result(result);
            }
        }
    }

    /// Build field context from document and layout
    pub fn build_context(
        tree: &DocumentTree,
        total_pages: u32,
        page_for_field: impl Fn(NodeId) -> u32,
    ) -> FieldContext {
        let mut context = FieldContext::new()
            .with_page_info(1, total_pages)
            .with_now();

        // Add document metadata
        context.title = tree.document.metadata.title.clone();
        context.author = tree.document.metadata.author.clone();

        // Build TOC entries from headings
        context.toc_entries = Self::scan_headings(tree, &page_for_field);

        // Build bookmark page map
        for bookmark in tree.all_bookmarks() {
            let page = page_for_field(bookmark.start_position().node_id);
            context.bookmark_pages.insert(bookmark.name().to_string(), page);

            // Get bookmark content
            if let Some(para) = tree.get_paragraph(bookmark.start_position().node_id) {
                let mut text = String::new();
                for &run_id in para.children() {
                    if let Some(run) = tree.get_run(run_id) {
                        text.push_str(&run.text);
                    }
                }
                context.bookmark_content.insert(bookmark.name().to_string(), text);
            }
        }

        // Calculate word and character counts
        let text_content = tree.text_content();
        context.word_count = text_content.split_whitespace().count() as u32;
        context.char_count = text_content.chars().count() as u32;

        context
    }

    /// Scan document for headings (for TOC generation)
    fn scan_headings(
        tree: &DocumentTree,
        page_for_field: &impl Fn(NodeId) -> u32,
    ) -> Vec<TocEntry> {
        let mut entries = Vec::new();

        for para in tree.paragraphs() {
            // Check if paragraph has an outline level (heading style)
            let outline_level = para.direct_formatting.outline_level
                .or_else(|| {
                    // Check paragraph style for outline level
                    para.paragraph_style_id.as_ref()
                        .and_then(|style_id| tree.styles.resolve(style_id))
                        .and_then(|resolved| resolved.paragraph_props.outline_level)
                });

            if let Some(level) = outline_level {
                if level > 0 && level <= 9 {
                    // Get paragraph text
                    let mut text = String::new();
                    for &child_id in para.children() {
                        if let Some(run) = tree.get_run(child_id) {
                            text.push_str(&run.text);
                        }
                    }

                    let page_number = page_for_field(para.id());

                    entries.push(TocEntry {
                        text: text.trim().to_string(),
                        level,
                        page_number,
                        bookmark: None, // Could generate bookmarks for TOC links
                        paragraph_id: para.id(),
                    });
                }
            }
        }

        entries
    }

    /// Update fields that need layout info (PAGE, NUMPAGES)
    /// Called during/after layout
    pub fn update_layout_fields(
        registry: &mut FieldRegistry,
        field_to_page: &HashMap<NodeId, u32>,
        total_pages: u32,
    ) {
        let field_ids: Vec<NodeId> = registry.all_ids().collect();

        for field_id in field_ids {
            if let Some(field) = registry.get(field_id) {
                if !field.auto_updates_on_layout() {
                    continue;
                }

                let page = field_to_page.get(&field_id).copied().unwrap_or(1);

                let result = match &field.instruction {
                    FieldInstruction::Page { format } => {
                        format.format(page)
                    }
                    FieldInstruction::NumPages { format } => {
                        format.format(total_pages)
                    }
                    FieldInstruction::Section => {
                        // Would need section info
                        "1".to_string()
                    }
                    FieldInstruction::SectionPages => {
                        total_pages.to_string()
                    }
                    _ => continue,
                };

                if let Some(field) = registry.get_mut(field_id) {
                    field.set_result(result);
                }
            }
        }
    }
}

// =============================================================================
// Field Information DTOs
// =============================================================================

/// Information about a field for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    /// Field ID
    pub id: String,
    /// Field code name (e.g., "PAGE", "TOC")
    pub code_name: String,
    /// Full field code string
    pub code_string: String,
    /// Current result text
    pub result: String,
    /// Whether the field is locked
    pub locked: bool,
    /// Whether showing field code
    pub show_code: bool,
    /// Whether the field is dirty
    pub dirty: bool,
}

impl FieldInfo {
    /// Create from a field
    pub fn from_field(field: &Field) -> Self {
        Self {
            id: field.id().to_string(),
            code_name: field.instruction.code_name().to_string(),
            code_string: field.instruction.display_string(),
            result: field.display_text(),
            locked: field.locked,
            show_code: field.show_code,
            dirty: field.dirty,
        }
    }
}

/// Get information about all fields in a registry
pub fn list_fields(registry: &FieldRegistry) -> Vec<FieldInfo> {
    registry.all().map(FieldInfo::from_field).collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_field_command_creation() {
        let cmd = InsertField::page();
        assert!(matches!(cmd.instruction, FieldInstruction::Page { .. }));

        let cmd = InsertField::num_pages();
        assert!(matches!(cmd.instruction, FieldInstruction::NumPages { .. }));

        let cmd = InsertField::date("yyyy-MM-dd");
        if let FieldInstruction::Date { format } = &cmd.instruction {
            assert_eq!(format, "yyyy-MM-dd");
        } else {
            panic!("Expected Date instruction");
        }
    }

    #[test]
    fn test_insert_field_with_initial_text() {
        let cmd = InsertField::page().with_initial_text("1");
        assert_eq!(cmd.initial_text, Some("1".to_string()));
    }

    #[test]
    fn test_field_update_engine_seq() {
        let mut registry = FieldRegistry::new();

        // Insert some SEQ fields
        let field1 = Field::seq("Figure");
        let id1 = registry.insert(field1);

        let field2 = Field::seq("Figure");
        let id2 = registry.insert(field2);

        let field3 = Field::seq("Table");
        let id3 = registry.insert(field3);

        let context = FieldContext::new();
        FieldUpdateEngine::update_all(&mut registry, &context);

        assert_eq!(registry.get(id1).unwrap().cached_text.as_deref(), Some("1"));
        assert_eq!(registry.get(id2).unwrap().cached_text.as_deref(), Some("2"));
        assert_eq!(registry.get(id3).unwrap().cached_text.as_deref(), Some("1"));
    }

    #[test]
    fn test_field_update_engine_page() {
        let mut registry = FieldRegistry::new();

        let field = Field::page();
        let id = registry.insert(field);

        let mut field_to_page = HashMap::new();
        field_to_page.insert(id, 5);

        FieldUpdateEngine::update_layout_fields(&mut registry, &field_to_page, 10);

        assert_eq!(registry.get(id).unwrap().cached_text.as_deref(), Some("5"));
    }

    #[test]
    fn test_field_update_engine_numpages() {
        let mut registry = FieldRegistry::new();

        let field = Field::num_pages();
        let id = registry.insert(field);

        let field_to_page = HashMap::new();
        FieldUpdateEngine::update_layout_fields(&mut registry, &field_to_page, 25);

        assert_eq!(registry.get(id).unwrap().cached_text.as_deref(), Some("25"));
    }

    #[test]
    fn test_locked_field_not_updated() {
        let mut registry = FieldRegistry::new();

        let mut field = Field::page();
        field.lock();
        field.set_result("LOCKED".to_string());
        let id = registry.insert(field);

        let mut field_to_page = HashMap::new();
        field_to_page.insert(id, 5);

        FieldUpdateEngine::update_layout_fields(&mut registry, &field_to_page, 10);

        // Should still be LOCKED, not updated
        assert_eq!(registry.get(id).unwrap().cached_text.as_deref(), Some("LOCKED"));
    }

    #[test]
    fn test_field_info() {
        let mut field = Field::page();
        field.set_result("5".to_string());

        let info = FieldInfo::from_field(&field);

        assert_eq!(info.code_name, "PAGE");
        assert_eq!(info.result, "5");
        assert!(!info.locked);
        assert!(!info.show_code);
    }

    #[test]
    fn test_list_fields() {
        let mut registry = FieldRegistry::new();
        registry.insert(Field::page());
        registry.insert(Field::num_pages());
        registry.insert(Field::author());

        let fields = list_fields(&registry);
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn test_set_field_lock() {
        let cmd = SetFieldLock::lock(NodeId::new());
        assert!(cmd.locked);

        let cmd = SetFieldLock::unlock(NodeId::new());
        assert!(!cmd.locked);
    }

    #[test]
    fn test_toggle_field_codes() {
        let field_id = NodeId::new();

        let cmd = ToggleFieldCodes::for_field(field_id);
        assert_eq!(cmd.field_id, Some(field_id));

        let cmd = ToggleFieldCodes::for_all();
        assert_eq!(cmd.field_id, None);
    }
}
