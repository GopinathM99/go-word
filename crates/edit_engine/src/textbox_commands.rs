//! Text box commands for inserting, editing, and modifying text boxes
//!
//! This module provides commands for working with text boxes in the document,
//! including creation, deletion, content editing, styling, and positioning.

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    Anchor, DocumentTree, Node, NodeId, NodeType, Paragraph, Position, Run, Selection, Size,
    TextBox, TextBoxStyle,
};
use serde::{Deserialize, Serialize};

/// Insert a text box at the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTextBox {
    /// Anchor configuration
    pub anchor: Anchor,
    /// Size specification
    pub size: Size,
    /// Style properties
    pub style: TextBoxStyle,
    /// Optional initial text content
    pub initial_text: Option<String>,
    /// Optional name for the text box
    pub name: Option<String>,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
}

impl InsertTextBox {
    /// Create a new insert text box command with default settings
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            anchor: Anchor::default(),
            size: Size::points(width, height),
            style: TextBoxStyle::default(),
            initial_text: None,
            name: None,
            alt_text: None,
        }
    }

    /// Create an inline text box
    pub fn inline(width: f32, height: f32) -> Self {
        Self {
            anchor: Anchor::inline(),
            size: Size::points(width, height),
            style: TextBoxStyle::default(),
            initial_text: None,
            name: None,
            alt_text: None,
        }
    }

    /// Create a page-anchored text box
    pub fn at_page_position(width: f32, height: f32, x: f32, y: f32) -> Self {
        Self {
            anchor: Anchor::page(x, y),
            size: Size::points(width, height),
            style: TextBoxStyle::default(),
            initial_text: None,
            name: None,
            alt_text: None,
        }
    }

    /// Set initial text content
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.initial_text = Some(text.into());
        self
    }

    /// Set the name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the alt text
    pub fn with_alt_text(mut self, alt_text: impl Into<String>) -> Self {
        self.alt_text = Some(alt_text.into());
        self
    }

    /// Set the style
    pub fn with_style(mut self, style: TextBoxStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the anchor
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
}

impl Command for InsertTextBox {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the paragraph containing the selection
        let para_id = get_paragraph_for_position(&new_tree, &selection.anchor)?;

        // Create the text box
        let mut textbox = TextBox::new();
        textbox.set_anchor(self.anchor.clone());
        textbox.set_size(self.size.clone());
        textbox.set_style(self.style.clone());

        if let Some(ref name) = self.name {
            textbox.set_name(name.clone());
        }
        if let Some(ref alt) = self.alt_text {
            textbox.set_alt_text(alt.clone());
        }

        let textbox_id = textbox.id();

        // Find the insertion index in the paragraph
        let insert_index = find_insertion_index(&new_tree, para_id, &selection.start())?;

        // Insert the text box
        new_tree
            .insert_textbox(textbox, para_id, Some(insert_index))
            .map_err(|e| EditError::DocModel(e))?;

        // If there's initial text, create a paragraph with the text inside the text box
        if let Some(ref text) = self.initial_text {
            let mut para = Paragraph::new();
            let para_content_id = para.id();

            new_tree
                .insert_paragraph_into_textbox(para, textbox_id, None)
                .map_err(|e| EditError::DocModel(e))?;

            if !text.is_empty() {
                let run = Run::new(text);
                new_tree
                    .insert_run(run, para_content_id, None)
                    .map_err(|e| EditError::DocModel(e))?;
            }
        } else {
            // Create an empty paragraph in the text box
            let para = Paragraph::new();
            new_tree
                .insert_paragraph_into_textbox(para, textbox_id, None)
                .map_err(|e| EditError::DocModel(e))?;
        }

        // New selection is after the text box (treat as single character)
        let new_selection =
            Selection::collapsed(Position::new(para_id, selection.start().offset + 1));

        // Create the inverse command
        let inverse = Box::new(DeleteTextBox { textbox_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // This will be replaced by the proper inverse in apply()
        Box::new(DeleteTextBox {
            textbox_id: NodeId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        // Text boxes take up one character position
        if selection.anchor.offset >= selection.focus.offset {
            Selection::new(
                Position::new(selection.anchor.node_id, selection.anchor.offset + 1),
                selection.focus,
            )
        } else {
            Selection::new(
                selection.anchor,
                Position::new(selection.focus.node_id, selection.focus.offset + 1),
            )
        }
    }

    fn display_name(&self) -> &str {
        "Insert Text Box"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Delete a text box by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTextBox {
    /// The text box node ID to delete
    pub textbox_id: NodeId,
}

impl DeleteTextBox {
    pub fn new(textbox_id: NodeId) -> Self {
        Self { textbox_id }
    }
}

impl Command for DeleteTextBox {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the text box before removing
        let textbox = new_tree
            .get_textbox(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?
            .clone();

        // Store text box data for undo
        let anchor = textbox.anchor.clone();
        let size = textbox.size.clone();
        let style = textbox.style.clone();
        let name = textbox.name.clone();
        let alt_text = textbox.alt_text.clone();

        // Extract content text for undo
        let initial_text = extract_textbox_content(&new_tree, self.textbox_id);

        // Remove the text box
        new_tree
            .remove_textbox(self.textbox_id)
            .map_err(|e| EditError::DocModel(e))?;

        // Create the inverse command
        let inverse = InsertTextBox {
            anchor,
            size,
            style,
            initial_text: if initial_text.is_empty() {
                None
            } else {
                Some(initial_text)
            },
            name,
            alt_text,
        };

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(textbox) = tree.get_textbox(self.textbox_id) {
            let initial_text = extract_textbox_content(tree, self.textbox_id);
            Box::new(InsertTextBox {
                anchor: textbox.anchor.clone(),
                size: textbox.size.clone(),
                style: textbox.style.clone(),
                initial_text: if initial_text.is_empty() {
                    None
                } else {
                    Some(initial_text)
                },
                name: textbox.name.clone(),
                alt_text: textbox.alt_text.clone(),
            })
        } else {
            Box::new(InsertTextBox::new(200.0, 100.0))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Text Box"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set text box content (replaces all content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTextBoxContent {
    /// The text box node ID
    pub textbox_id: NodeId,
    /// New content text
    pub content: String,
}

impl SetTextBoxContent {
    pub fn new(textbox_id: NodeId, content: impl Into<String>) -> Self {
        Self {
            textbox_id,
            content: content.into(),
        }
    }
}

impl Command for SetTextBoxContent {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get old content for undo
        let old_content = extract_textbox_content(&new_tree, self.textbox_id);

        // Get the text box
        let textbox = new_tree
            .get_textbox(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;

        // Remove old content paragraphs
        let old_para_ids: Vec<NodeId> = textbox.content.clone();
        for para_id in old_para_ids {
            let _ = new_tree.remove_paragraph_from_textbox(para_id, self.textbox_id);
        }

        // Create new paragraph with content
        let mut para = Paragraph::new();
        let para_id = para.id();

        new_tree
            .insert_paragraph_into_textbox(para, self.textbox_id, None)
            .map_err(|e| EditError::DocModel(e))?;

        if !self.content.is_empty() {
            let run = Run::new(&self.content);
            new_tree
                .insert_run(run, para_id, None)
                .map_err(|e| EditError::DocModel(e))?;
        }

        // Create inverse command
        let inverse = Box::new(SetTextBoxContent {
            textbox_id: self.textbox_id,
            content: old_content,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let old_content = extract_textbox_content(tree, self.textbox_id);
        Box::new(SetTextBoxContent {
            textbox_id: self.textbox_id,
            content: old_content,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Text Box Content"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set text box style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTextBoxStyle {
    /// The text box node ID
    pub textbox_id: NodeId,
    /// New style
    pub style: TextBoxStyle,
}

impl SetTextBoxStyle {
    pub fn new(textbox_id: NodeId, style: TextBoxStyle) -> Self {
        Self { textbox_id, style }
    }
}

impl Command for SetTextBoxStyle {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get current style for undo
        let textbox = new_tree
            .get_textbox(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        let old_style = textbox.style.clone();

        // Apply new style
        let textbox = new_tree
            .get_textbox_mut(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        textbox.set_style(self.style.clone());

        // Create inverse command
        let inverse = Box::new(SetTextBoxStyle {
            textbox_id: self.textbox_id,
            style: old_style,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(textbox) = tree.get_textbox(self.textbox_id) {
            Box::new(SetTextBoxStyle {
                textbox_id: self.textbox_id,
                style: textbox.style.clone(),
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Text Box Style"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set text box anchor (position and wrap settings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTextBoxAnchor {
    /// The text box node ID
    pub textbox_id: NodeId,
    /// New anchor configuration
    pub anchor: Anchor,
}

impl SetTextBoxAnchor {
    pub fn new(textbox_id: NodeId, anchor: Anchor) -> Self {
        Self { textbox_id, anchor }
    }
}

impl Command for SetTextBoxAnchor {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get current anchor for undo
        let textbox = new_tree
            .get_textbox(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        let old_anchor = textbox.anchor.clone();

        // Apply new anchor
        let textbox = new_tree
            .get_textbox_mut(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        textbox.set_anchor(self.anchor.clone());

        // Create inverse command
        let inverse = Box::new(SetTextBoxAnchor {
            textbox_id: self.textbox_id,
            anchor: old_anchor,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(textbox) = tree.get_textbox(self.textbox_id) {
            Box::new(SetTextBoxAnchor {
                textbox_id: self.textbox_id,
                anchor: textbox.anchor.clone(),
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Text Box Anchor"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Resize a text box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeTextBox {
    /// The text box node ID
    pub textbox_id: NodeId,
    /// New size
    pub size: Size,
}

impl ResizeTextBox {
    pub fn new(textbox_id: NodeId, size: Size) -> Self {
        Self { textbox_id, size }
    }

    /// Create a resize command with point dimensions
    pub fn points(textbox_id: NodeId, width: f32, height: f32) -> Self {
        Self {
            textbox_id,
            size: Size::points(width, height),
        }
    }
}

impl Command for ResizeTextBox {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get current size for undo
        let textbox = new_tree
            .get_textbox(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        let old_size = textbox.size.clone();

        // Apply new size
        let textbox = new_tree
            .get_textbox_mut(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        textbox.set_size(self.size.clone());

        // Create inverse command
        let inverse = Box::new(ResizeTextBox {
            textbox_id: self.textbox_id,
            size: old_size,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(textbox) = tree.get_textbox(self.textbox_id) {
            Box::new(ResizeTextBox {
                textbox_id: self.textbox_id,
                size: textbox.size.clone(),
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Resize Text Box"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Update text box properties (name, alt text, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTextBoxProperties {
    /// The text box node ID
    pub textbox_id: NodeId,
    /// New name (if Some, updates)
    pub name: Option<Option<String>>,
    /// New alt text (if Some, updates)
    pub alt_text: Option<Option<String>>,
}

impl UpdateTextBoxProperties {
    pub fn new(textbox_id: NodeId) -> Self {
        Self {
            textbox_id,
            name: None,
            alt_text: None,
        }
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_alt_text(mut self, alt_text: Option<String>) -> Self {
        self.alt_text = Some(alt_text);
        self
    }
}

impl Command for UpdateTextBoxProperties {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get current values for undo
        let textbox = new_tree
            .get_textbox(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;
        let old_name = textbox.name.clone();
        let old_alt_text = textbox.alt_text.clone();

        // Apply updates
        let textbox = new_tree
            .get_textbox_mut(self.textbox_id)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", self.textbox_id))
            })?;

        if let Some(ref name) = self.name {
            textbox.name = name.clone();
        }
        if let Some(ref alt) = self.alt_text {
            textbox.alt_text = alt.clone();
        }

        // Create inverse command
        let mut inverse = UpdateTextBoxProperties::new(self.textbox_id);
        if self.name.is_some() {
            inverse.name = Some(old_name);
        }
        if self.alt_text.is_some() {
            inverse.alt_text = Some(old_alt_text);
        }

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(textbox) = tree.get_textbox(self.textbox_id) {
            let mut inverse = UpdateTextBoxProperties::new(self.textbox_id);
            if self.name.is_some() {
                inverse.name = Some(textbox.name.clone());
            }
            if self.alt_text.is_some() {
                inverse.alt_text = Some(textbox.alt_text.clone());
            }
            Box::new(inverse)
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Update Text Box Properties"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Extract text content from a text box
fn extract_textbox_content(tree: &DocumentTree, textbox_id: NodeId) -> String {
    let Some(textbox) = tree.get_textbox(textbox_id) else {
        return String::new();
    };

    let mut content = String::new();
    for (i, &para_id) in textbox.content.iter().enumerate() {
        if let Some(para) = tree.get_paragraph(para_id) {
            for &child_id in para.children() {
                if let Some(run) = tree.get_run(child_id) {
                    content.push_str(&run.text);
                }
            }
        }
        if i < textbox.content.len() - 1 {
            content.push('\n');
        }
    }
    content
}

/// Get the paragraph containing a position
fn get_paragraph_for_position(tree: &DocumentTree, position: &Position) -> Result<NodeId> {
    let node_type = tree.node_type(position.node_id).ok_or_else(|| {
        EditError::InvalidCommand(format!("Node not found: {:?}", position.node_id))
    })?;

    match node_type {
        NodeType::Paragraph => Ok(position.node_id),
        NodeType::Run => {
            let run = tree.get_run(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Run not found: {:?}", position.node_id))
            })?;

            let parent_id = run
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Run has no parent".to_string()))?;

            if tree.get_paragraph(parent_id).is_some() {
                return Ok(parent_id);
            }

            // Parent might be a hyperlink
            if let Some(hyperlink) = tree.get_hyperlink(parent_id) {
                return hyperlink.parent().ok_or_else(|| {
                    EditError::InvalidCommand("Hyperlink has no parent".to_string())
                });
            }

            Err(EditError::InvalidCommand(
                "Cannot determine paragraph".to_string(),
            ))
        }
        NodeType::Hyperlink => {
            let hyperlink = tree.get_hyperlink(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Hyperlink not found: {:?}", position.node_id))
            })?;

            hyperlink
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Hyperlink has no parent".to_string()))
        }
        NodeType::Image => {
            let image = tree.get_image(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Image not found: {:?}", position.node_id))
            })?;

            image
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Image has no parent".to_string()))
        }
        NodeType::Shape => {
            let shape = tree.get_shape(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Shape not found: {:?}", position.node_id))
            })?;

            shape
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Shape has no parent".to_string()))
        }
        NodeType::TextBox => {
            let textbox = tree.get_textbox(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Text box not found: {:?}", position.node_id))
            })?;

            textbox
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Text box has no parent".to_string()))
        }
        _ => Err(EditError::InvalidCommand(format!(
            "Invalid node type for position: {:?}",
            node_type
        ))),
    }
}

/// Find the insertion index in a paragraph for a given position
fn find_insertion_index(tree: &DocumentTree, para_id: NodeId, position: &Position) -> Result<usize> {
    let para = tree.get_paragraph(para_id).ok_or_else(|| {
        EditError::InvalidCommand(format!("Paragraph not found: {:?}", para_id))
    })?;

    let mut offset = 0;

    for (index, &child_id) in para.children().iter().enumerate() {
        // Check for run
        if let Some(run) = tree.get_run(child_id) {
            let run_len = run.text.chars().count();
            if offset + run_len >= position.offset {
                return Ok(index);
            }
            offset += run_len;
        }
        // Check for hyperlink
        else if let Some(hyperlink) = tree.get_hyperlink(child_id) {
            let mut hyperlink_len = 0;
            for &run_id in hyperlink.children() {
                if let Some(run) = tree.get_run(run_id) {
                    hyperlink_len += run.text.chars().count();
                }
            }
            if offset + hyperlink_len >= position.offset {
                return Ok(index);
            }
            offset += hyperlink_len;
        }
        // Check for image (counts as 1 character)
        else if tree.get_image(child_id).is_some() {
            if offset + 1 >= position.offset {
                return Ok(index);
            }
            offset += 1;
        }
        // Check for shape (counts as 1 character)
        else if tree.get_shape(child_id).is_some() {
            if offset + 1 >= position.offset {
                return Ok(index);
            }
            offset += 1;
        }
        // Check for text box (counts as 1 character)
        else if tree.get_textbox(child_id).is_some() {
            if offset + 1 >= position.offset {
                return Ok(index);
            }
            offset += 1;
        }
    }

    Ok(para.children().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();
        (tree, para_id)
    }

    #[test]
    fn test_insert_textbox() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertTextBox::new(200.0, 100.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Should have one text box
        assert_eq!(result.tree.textboxes().count(), 1);

        let textbox = result.tree.textboxes().next().unwrap();
        assert!(textbox.is_floating());
    }

    #[test]
    fn test_insert_inline_textbox() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertTextBox::inline(100.0, 50.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let textbox = result.tree.textboxes().next().unwrap();
        assert!(textbox.is_inline());
    }

    #[test]
    fn test_delete_textbox() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a text box first
        let textbox = TextBox::with_size(200.0, 100.0);
        let textbox_id = tree.insert_textbox(textbox, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Delete the text box
        let cmd = DeleteTextBox::new(textbox_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.textboxes().count(), 0);
    }

    #[test]
    fn test_set_textbox_style() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a text box first
        let textbox = TextBox::with_size(200.0, 100.0);
        let textbox_id = tree.insert_textbox(textbox, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Set new style
        let new_style = TextBoxStyle::transparent();
        let cmd = SetTextBoxStyle::new(textbox_id, new_style);
        let result = cmd.apply(&tree, &selection).unwrap();

        let updated = result.tree.get_textbox(textbox_id).unwrap();
        assert!(updated.style.fill.is_none());
    }

    #[test]
    fn test_resize_textbox() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a text box first
        let textbox = TextBox::with_size(200.0, 100.0);
        let textbox_id = tree.insert_textbox(textbox, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Resize the text box
        let cmd = ResizeTextBox::points(textbox_id, 300.0, 150.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let resized = result.tree.get_textbox(textbox_id).unwrap();
        assert_eq!(resized.effective_width(500.0), 300.0);
        assert_eq!(resized.effective_height(500.0), 150.0);
    }

    #[test]
    fn test_textbox_with_initial_text() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertTextBox::new(200.0, 100.0).with_text("Hello, World!");
        let result = cmd.apply(&tree, &selection).unwrap();

        let textbox = result.tree.textboxes().next().unwrap();
        let content = extract_textbox_content(&result.tree, textbox.id());
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_set_textbox_content() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a text box with initial content
        let textbox = TextBox::with_size(200.0, 100.0);
        let textbox_id = tree.insert_textbox(textbox, para_id, None).unwrap();

        // Add initial paragraph
        let para = Paragraph::new();
        let content_para_id = para.id();
        tree.insert_paragraph_into_textbox(para, textbox_id, None).unwrap();

        let run = Run::new("Initial");
        tree.insert_run(run, content_para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Set new content
        let cmd = SetTextBoxContent::new(textbox_id, "New Content");
        let result = cmd.apply(&tree, &selection).unwrap();

        let content = extract_textbox_content(&result.tree, textbox_id);
        assert_eq!(content, "New Content");
    }
}
