//! Footnote and Endnote Commands
//!
//! Commands for creating, editing, navigating, and managing footnotes and endnotes.
//!
//! ## Features
//!
//! - Insert footnotes and endnotes at the current position
//! - Delete notes
//! - Edit note content
//! - Convert between footnote and endnote
//! - Navigate to note or back to reference
//! - Configure footnote/endnote properties

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    DocumentTree, EndnoteProperties, FootnoteProperties, Node, NodeId, Note, NoteId, NoteType,
    Paragraph, Position, Run, Selection,
};
use serde::{Deserialize, Serialize};

// =============================================================================
// Insert Footnote Command
// =============================================================================

/// Insert a footnote at the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertFootnote {
    /// Initial content text for the footnote (optional)
    pub initial_content: Option<String>,
    /// Section ID for the footnote (optional)
    pub section_id: Option<String>,
}

impl InsertFootnote {
    /// Create a new insert footnote command
    pub fn new() -> Self {
        Self {
            initial_content: None,
            section_id: None,
        }
    }

    /// Create with initial content
    pub fn with_content(content: impl Into<String>) -> Self {
        Self {
            initial_content: Some(content.into()),
            section_id: None,
        }
    }

    /// Set the section ID
    pub fn in_section(mut self, section_id: impl Into<String>) -> Self {
        self.section_id = Some(section_id.into());
        self
    }
}

impl Default for InsertFootnote {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for InsertFootnote {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Parse section ID if provided
        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        // Insert the footnote at the focus position
        let position = selection.focus;
        let (note_id, ref_id) = new_tree.insert_footnote(position, section_id);

        // If there's initial content, create a paragraph with it
        if let Some(content) = &self.initial_content {
            if !content.is_empty() {
                let para = Paragraph::new();
                let para_id = para.id();
                new_tree.nodes.paragraphs.insert(para_id, para);

                let run = Run::new(content);
                let run_id = run.id();
                new_tree.nodes.runs.insert(run_id, run);

                if let Some(para) = new_tree.nodes.paragraphs.get_mut(&para_id) {
                    para.add_child(run_id);
                }

                new_tree.add_footnote_content(note_id, para_id);
            }
        }

        // Create the inverse command
        let inverse = Box::new(DeleteNote {
            note_id: note_id.to_string(),
            note_type: NoteType::Footnote,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection, // Selection stays at the reference
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // Cannot invert without knowing the note ID
        Box::new(InsertFootnote::new())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Footnote"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Insert Endnote Command
// =============================================================================

/// Insert an endnote at the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertEndnote {
    /// Initial content text for the endnote (optional)
    pub initial_content: Option<String>,
    /// Section ID for the endnote (optional)
    pub section_id: Option<String>,
}

impl InsertEndnote {
    /// Create a new insert endnote command
    pub fn new() -> Self {
        Self {
            initial_content: None,
            section_id: None,
        }
    }

    /// Create with initial content
    pub fn with_content(content: impl Into<String>) -> Self {
        Self {
            initial_content: Some(content.into()),
            section_id: None,
        }
    }

    /// Set the section ID
    pub fn in_section(mut self, section_id: impl Into<String>) -> Self {
        self.section_id = Some(section_id.into());
        self
    }
}

impl Default for InsertEndnote {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for InsertEndnote {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Parse section ID if provided
        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        // Insert the endnote at the focus position
        let position = selection.focus;
        let (note_id, ref_id) = new_tree.insert_endnote(position, section_id);

        // If there's initial content, create a paragraph with it
        if let Some(content) = &self.initial_content {
            if !content.is_empty() {
                let para = Paragraph::new();
                let para_id = para.id();
                new_tree.nodes.paragraphs.insert(para_id, para);

                let run = Run::new(content);
                let run_id = run.id();
                new_tree.nodes.runs.insert(run_id, run);

                if let Some(para) = new_tree.nodes.paragraphs.get_mut(&para_id) {
                    para.add_child(run_id);
                }

                new_tree.add_endnote_content(note_id, para_id);
            }
        }

        // Create the inverse command
        let inverse = Box::new(DeleteNote {
            note_id: note_id.to_string(),
            note_type: NoteType::Endnote,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(InsertEndnote::new())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Endnote"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Delete Note Command
// =============================================================================

/// Delete a footnote or endnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNote {
    /// ID of the note to delete
    pub note_id: String,
    /// Type of note
    pub note_type: NoteType,
}

impl DeleteNote {
    /// Create a delete footnote command
    pub fn footnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Footnote,
        }
    }

    /// Create a delete endnote command
    pub fn endnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Endnote,
        }
    }
}

impl Command for DeleteNote {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let note_id = NoteId::from_string(&self.note_id).ok_or_else(|| {
            EditError::InvalidCommand(format!("Invalid note ID: {}", self.note_id))
        })?;

        // Get the note before deleting (for undo)
        let note = new_tree
            .notes
            .get_note(note_id, self.note_type)
            .ok_or_else(|| EditError::InvalidCommand("Note not found".to_string()))?
            .clone();

        // Delete the note
        new_tree
            .delete_note(note_id, self.note_type)
            .ok_or_else(|| EditError::InvalidCommand("Failed to delete note".to_string()))?;

        // Create the inverse command (restore note)
        let inverse = Box::new(RestoreNote {
            note: note.clone(),
            ref_position: note.reference_position,
            section_id: note.section_id.map(|id| id.to_string()),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Would need the original note data
        Box::new(DeleteNote {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.note_type {
            NoteType::Footnote => "Delete Footnote",
            NoteType::Endnote => "Delete Endnote",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Restore Note Command (Internal, for undo)
// =============================================================================

/// Restore a deleted note (used for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreNote {
    /// The note to restore
    note: Note,
    /// Original reference position
    ref_position: Option<Position>,
    /// Original section ID
    section_id: Option<String>,
}

impl Command for RestoreNote {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        // Determine the position
        let position = self.ref_position.ok_or_else(|| {
            EditError::InvalidCommand("No reference position for note".to_string())
        })?;

        // Insert the appropriate note type
        let note_id = if self.note.is_footnote() {
            let (id, _) = new_tree.insert_footnote(position, section_id);
            id
        } else {
            let (id, _) = new_tree.insert_endnote(position, section_id);
            id
        };

        // Restore the content
        let note_type = self.note.note_type;
        if let Some(note) = new_tree.notes.get_note_mut(note_id, note_type) {
            for &para_id in self.note.content() {
                note.add_content(para_id);
            }
        }

        let inverse = Box::new(DeleteNote {
            note_id: note_id.to_string(),
            note_type: self.note.note_type,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteNote {
            note_id: self.note.id().to_string(),
            note_type: self.note.note_type,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.note.note_type {
            NoteType::Footnote => "Restore Footnote",
            NoteType::Endnote => "Restore Endnote",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Edit Note Content Command
// =============================================================================

/// Edit the content of a footnote or endnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditNoteContent {
    /// ID of the note to edit
    pub note_id: String,
    /// Type of note
    pub note_type: NoteType,
    /// New content text
    pub content: String,
}

impl EditNoteContent {
    /// Create an edit footnote content command
    pub fn footnote(note_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Footnote,
            content: content.into(),
        }
    }

    /// Create an edit endnote content command
    pub fn endnote(note_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Endnote,
            content: content.into(),
        }
    }
}

impl Command for EditNoteContent {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let note_id = NoteId::from_string(&self.note_id).ok_or_else(|| {
            EditError::InvalidCommand(format!("Invalid note ID: {}", self.note_id))
        })?;

        // Get the note
        let note = new_tree
            .notes
            .get_note_mut(note_id, self.note_type)
            .ok_or_else(|| EditError::InvalidCommand("Note not found".to_string()))?;

        // Store old content IDs for undo
        let old_content: Vec<NodeId> = note.content().to_vec();

        // Clear existing content
        note.clear_content();

        // Create new content
        let para = Paragraph::new();
        let para_id = para.id();
        new_tree.nodes.paragraphs.insert(para_id, para);

        let run = Run::new(&self.content);
        let run_id = run.id();
        new_tree.nodes.runs.insert(run_id, run);

        if let Some(para) = new_tree.nodes.paragraphs.get_mut(&para_id) {
            para.add_child(run_id);
        }

        // Get the note again and add content
        if let Some(note) = new_tree.notes.get_note_mut(note_id, self.note_type) {
            note.add_content(para_id);
        }

        // Create inverse (would need to store old content properly)
        let inverse = Box::new(EditNoteContent {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
            content: String::new(), // Simplified - full implementation would restore old content
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(EditNoteContent {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
            content: String::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.note_type {
            NoteType::Footnote => "Edit Footnote",
            NoteType::Endnote => "Edit Endnote",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Convert Note Command
// =============================================================================

/// Convert a footnote to endnote or vice versa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertNote {
    /// ID of the note to convert
    pub note_id: String,
    /// Current type of note (will be converted to the other type)
    pub current_type: NoteType,
}

impl ConvertNote {
    /// Create a command to convert a footnote to endnote
    pub fn footnote_to_endnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            current_type: NoteType::Footnote,
        }
    }

    /// Create a command to convert an endnote to footnote
    pub fn endnote_to_footnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            current_type: NoteType::Endnote,
        }
    }
}

impl Command for ConvertNote {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let note_id = NoteId::from_string(&self.note_id).ok_or_else(|| {
            EditError::InvalidCommand(format!("Invalid note ID: {}", self.note_id))
        })?;

        // Convert the note
        let new_note_id = new_tree.notes.convert_note(note_id).ok_or_else(|| {
            EditError::InvalidCommand("Failed to convert note".to_string())
        })?;

        // Renumber both types
        new_tree.notes.renumber_footnotes();
        new_tree.notes.renumber_endnotes();

        // Create inverse command
        let new_type = match self.current_type {
            NoteType::Footnote => NoteType::Endnote,
            NoteType::Endnote => NoteType::Footnote,
        };

        let inverse = Box::new(ConvertNote {
            note_id: new_note_id.to_string(),
            current_type: new_type,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        let new_type = match self.current_type {
            NoteType::Footnote => NoteType::Endnote,
            NoteType::Endnote => NoteType::Footnote,
        };

        Box::new(ConvertNote {
            note_id: self.note_id.clone(),
            current_type: new_type,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.current_type {
            NoteType::Footnote => "Convert Footnote to Endnote",
            NoteType::Endnote => "Convert Endnote to Footnote",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Navigate to Note Command
// =============================================================================

/// Navigate from reference to note (or vice versa)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateToNote {
    /// ID of the note to navigate to
    pub note_id: String,
    /// Type of note
    pub note_type: NoteType,
}

impl NavigateToNote {
    /// Create a navigate to footnote command
    pub fn footnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Footnote,
        }
    }

    /// Create a navigate to endnote command
    pub fn endnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Endnote,
        }
    }
}

impl Command for NavigateToNote {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let note_id = NoteId::from_string(&self.note_id).ok_or_else(|| {
            EditError::InvalidCommand(format!("Invalid note ID: {}", self.note_id))
        })?;

        // Get the note to find its first content paragraph
        let note = tree
            .notes
            .get_note(note_id, self.note_type)
            .ok_or_else(|| EditError::InvalidCommand("Note not found".to_string()))?;

        // Navigate to the first content paragraph
        let new_selection = if let Some(&first_para_id) = note.content().first() {
            // Position at the start of the first paragraph
            Selection::collapsed(Position::new(first_para_id, 0))
        } else {
            // No content - stay at current selection
            *selection
        };

        // Create inverse (go back to reference)
        let inverse = Box::new(NavigateToNoteRef {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
            previous_selection: Some(*selection),
        });

        Ok(CommandResult {
            tree: tree.clone(),
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NavigateToNoteRef {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
            previous_selection: None,
        })
    }

    fn transform_selection(&self, _selection: &Selection) -> Selection {
        Selection::default()
    }

    fn display_name(&self) -> &str {
        match self.note_type {
            NoteType::Footnote => "Go to Footnote",
            NoteType::Endnote => "Go to Endnote",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Navigate to Note Reference Command
// =============================================================================

/// Navigate from note back to its reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateToNoteRef {
    /// ID of the note
    pub note_id: String,
    /// Type of note
    pub note_type: NoteType,
    /// Previous selection to restore (optional)
    pub previous_selection: Option<Selection>,
}

impl NavigateToNoteRef {
    /// Create a navigate to footnote reference command
    pub fn footnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Footnote,
            previous_selection: None,
        }
    }

    /// Create a navigate to endnote reference command
    pub fn endnote(note_id: impl Into<String>) -> Self {
        Self {
            note_id: note_id.into(),
            note_type: NoteType::Endnote,
            previous_selection: None,
        }
    }
}

impl Command for NavigateToNoteRef {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let note_id = NoteId::from_string(&self.note_id).ok_or_else(|| {
            EditError::InvalidCommand(format!("Invalid note ID: {}", self.note_id))
        })?;

        // Get the reference position
        let reference_position = tree
            .get_note_reference_position(note_id, self.note_type)
            .ok_or_else(|| EditError::InvalidCommand("Note reference not found".to_string()))?;

        let new_selection = Selection::collapsed(reference_position);

        // Create inverse (go back to note)
        let inverse = Box::new(NavigateToNote {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
        });

        Ok(CommandResult {
            tree: tree.clone(),
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NavigateToNote {
            note_id: self.note_id.clone(),
            note_type: self.note_type,
        })
    }

    fn transform_selection(&self, _selection: &Selection) -> Selection {
        Selection::default()
    }

    fn display_name(&self) -> &str {
        match self.note_type {
            NoteType::Footnote => "Go to Footnote Reference",
            NoteType::Endnote => "Go to Endnote Reference",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set Footnote Properties Command
// =============================================================================

/// Set footnote properties for a section or the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFootnoteProperties {
    /// Section ID (None for document-level)
    pub section_id: Option<String>,
    /// New properties
    pub properties: FootnoteProperties,
}

impl SetFootnoteProperties {
    /// Create a command to set document-level footnote properties
    pub fn document(properties: FootnoteProperties) -> Self {
        Self {
            section_id: None,
            properties,
        }
    }

    /// Create a command to set section-level footnote properties
    pub fn section(section_id: impl Into<String>, properties: FootnoteProperties) -> Self {
        Self {
            section_id: Some(section_id.into()),
            properties,
        }
    }
}

impl Command for SetFootnoteProperties {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        // Store old properties for undo
        let old_props = new_tree.get_footnote_properties(section_id).clone();

        // Set new properties
        new_tree.set_footnote_properties(section_id, self.properties.clone());

        // Create inverse command
        let inverse = Box::new(SetFootnoteProperties {
            section_id: self.section_id.clone(),
            properties: old_props,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        let old_props = tree.get_footnote_properties(section_id).clone();

        Box::new(SetFootnoteProperties {
            section_id: self.section_id.clone(),
            properties: old_props,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Footnote Properties"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set Endnote Properties Command
// =============================================================================

/// Set endnote properties for a section or the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEndnoteProperties {
    /// Section ID (None for document-level)
    pub section_id: Option<String>,
    /// New properties
    pub properties: EndnoteProperties,
}

impl SetEndnoteProperties {
    /// Create a command to set document-level endnote properties
    pub fn document(properties: EndnoteProperties) -> Self {
        Self {
            section_id: None,
            properties,
        }
    }

    /// Create a command to set section-level endnote properties
    pub fn section(section_id: impl Into<String>, properties: EndnoteProperties) -> Self {
        Self {
            section_id: Some(section_id.into()),
            properties,
        }
    }
}

impl Command for SetEndnoteProperties {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        // Store old properties for undo
        let old_props = new_tree.get_endnote_properties(section_id).clone();

        // Set new properties
        new_tree.set_endnote_properties(section_id, self.properties.clone());

        // Create inverse command
        let inverse = Box::new(SetEndnoteProperties {
            section_id: self.section_id.clone(),
            properties: old_props,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let section_id = self
            .section_id
            .as_ref()
            .and_then(|s| NodeId::from_string(s));

        let old_props = tree.get_endnote_properties(section_id).clone();

        Box::new(SetEndnoteProperties {
            section_id: self.section_id.clone(),
            properties: old_props,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Endnote Properties"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Note Info (for UI)
// =============================================================================

/// Information about a note for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInfo {
    /// Note ID as string
    pub id: String,
    /// Note type
    pub note_type: NoteType,
    /// Formatted mark (e.g., "1", "i", "*")
    pub mark: String,
    /// Preview of note content
    pub preview: Option<String>,
    /// Page number where reference appears
    pub reference_page: Option<usize>,
    /// Section ID if applicable
    pub section_id: Option<String>,
}

impl NoteInfo {
    /// Create note info from a note
    pub fn from_note(note: &Note, preview: Option<String>) -> Self {
        Self {
            id: note.id().to_string(),
            note_type: note.note_type,
            mark: note.mark().to_string(),
            preview,
            reference_page: note.reference_page,
            section_id: note.section_id.map(|id| id.to_string()),
        }
    }
}

/// Get a list of all footnotes in the document
pub fn list_footnotes(tree: &DocumentTree) -> Vec<NoteInfo> {
    tree.notes
        .footnotes()
        .map(|note| {
            let preview = get_note_preview(tree, note);
            NoteInfo::from_note(note, preview)
        })
        .collect()
}

/// Get a list of all endnotes in the document
pub fn list_endnotes(tree: &DocumentTree) -> Vec<NoteInfo> {
    tree.notes
        .endnotes()
        .map(|note| {
            let preview = get_note_preview(tree, note);
            NoteInfo::from_note(note, preview)
        })
        .collect()
}

/// Get preview text for a note (first ~50 characters)
fn get_note_preview(tree: &DocumentTree, note: &Note) -> Option<String> {
    let first_para_id = note.content().first()?;
    let para = tree.get_paragraph(*first_para_id)?;

    let mut text = String::new();
    for &run_id in para.children() {
        if let Some(run) = tree.get_run(run_id) {
            text.push_str(&run.text);
            if text.len() >= 50 {
                break;
            }
        }
    }

    if text.is_empty() {
        None
    } else {
        if text.len() >= 50 {
            text.truncate(47);
            text.push_str("...");
        }
        Some(text)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{NumberingScheme, RestartNumbering};

    fn create_test_tree() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();

        let run = Run::new("Hello, World!");
        tree.insert_run(run, para_id, None).unwrap();

        (tree, para_id)
    }

    #[test]
    fn test_insert_footnote() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        let cmd = InsertFootnote::with_content("This is a footnote.");
        let result = cmd.apply(&tree, &selection).unwrap();

        assert!(result.tree.has_footnotes());
        assert_eq!(result.tree.footnote_count(), 1);
    }

    #[test]
    fn test_insert_endnote() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        let cmd = InsertEndnote::with_content("This is an endnote.");
        let result = cmd.apply(&tree, &selection).unwrap();

        assert!(result.tree.has_endnotes());
        assert_eq!(result.tree.endnote_count(), 1);
    }

    #[test]
    fn test_delete_footnote() {
        let (mut tree, para_id) = create_test_tree();
        let position = Position::new(para_id, 5);

        // Insert a footnote
        let (note_id, _) = tree.insert_footnote(position, None);
        assert_eq!(tree.footnote_count(), 1);

        // Delete the footnote
        let selection = Selection::collapsed(position);
        let cmd = DeleteNote::footnote(note_id.to_string());
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.footnote_count(), 0);
    }

    #[test]
    fn test_convert_note() {
        let (mut tree, para_id) = create_test_tree();
        let position = Position::new(para_id, 5);

        // Insert a footnote
        let (note_id, _) = tree.insert_footnote(position, None);
        assert_eq!(tree.footnote_count(), 1);
        assert_eq!(tree.endnote_count(), 0);

        // Convert to endnote
        let selection = Selection::collapsed(position);
        let cmd = ConvertNote::footnote_to_endnote(note_id.to_string());
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.footnote_count(), 0);
        assert_eq!(result.tree.endnote_count(), 1);
    }

    #[test]
    fn test_set_footnote_properties() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let props = FootnoteProperties {
            numbering: NumberingScheme::LowerRoman,
            restart: RestartNumbering::PerPage,
            start_at: 1,
            ..Default::default()
        };

        let cmd = SetFootnoteProperties::document(props.clone());
        let result = cmd.apply(&tree, &selection).unwrap();

        let stored_props = result.tree.get_footnote_properties(None);
        assert_eq!(stored_props.numbering, NumberingScheme::LowerRoman);
        assert_eq!(stored_props.restart, RestartNumbering::PerPage);
    }

    #[test]
    fn test_list_footnotes() {
        let (mut tree, para_id) = create_test_tree();

        // Insert multiple footnotes
        for i in 0..3 {
            let pos = Position::new(para_id, i * 5);
            tree.insert_footnote(pos, None);
        }

        let footnotes = list_footnotes(&tree);
        assert_eq!(footnotes.len(), 3);
    }

    #[test]
    fn test_navigate_to_note() {
        let (mut tree, para_id) = create_test_tree();
        let position = Position::new(para_id, 5);

        // Insert a footnote with content
        let (note_id, _) = tree.insert_footnote(position, None);

        // Add content to the footnote
        let content_para = Paragraph::new();
        let content_para_id = content_para.id();
        tree.nodes.paragraphs.insert(content_para_id, content_para);
        tree.add_footnote_content(note_id, content_para_id);

        // Navigate to the note
        let selection = Selection::collapsed(position);
        let cmd = NavigateToNote::footnote(note_id.to_string());
        let result = cmd.apply(&tree, &selection).unwrap();

        // Selection should be at the content paragraph
        assert_eq!(result.selection.focus.node_id, content_para_id);
    }
}
