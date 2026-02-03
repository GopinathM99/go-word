//! Command system for document editing

use doc_model::{DocumentTree, Node, NodeId, NodeType, Paragraph, Position, Run, RunStyle, Selection};
use serde::{Deserialize, Serialize};

/// Result of applying a command
#[derive(Debug)]
pub struct CommandResult {
    /// The new document tree after the command
    pub tree: DocumentTree,
    /// The new selection after the command
    pub selection: Selection,
    /// The inverse command (for undo)
    pub inverse: Box<dyn Command>,
}

/// Trait for all editing commands
pub trait Command: std::fmt::Debug + Send + Sync {
    /// Apply this command to a document
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> crate::Result<CommandResult>;

    /// Get the inverse of this command
    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command>;

    /// Transform a selection through this command
    fn transform_selection(&self, selection: &Selection) -> Selection;

    /// Try to merge this command with another (for batching)
    fn merge_with(&self, _other: &dyn Command) -> Option<Box<dyn Command>> {
        None
    }

    /// Get a display name for this command
    fn display_name(&self) -> &str;

    /// Clone this command into a box
    fn clone_box(&self) -> Box<dyn Command>;
}

// ============================================================================
// Helper functions for working with document positions
// ============================================================================

/// Represents a resolved position within the document tree
#[derive(Debug, Clone)]
pub struct ResolvedPosition {
    /// The paragraph containing this position
    pub paragraph_id: NodeId,
    /// The run containing this position (if any)
    pub run_id: Option<NodeId>,
    /// The index of the run within the paragraph
    pub run_index: Option<usize>,
    /// The offset within the run's text
    pub offset_in_run: usize,
}

/// Resolve a position to find the containing paragraph and run
fn resolve_position(tree: &DocumentTree, position: &Position) -> Option<ResolvedPosition> {
    let node_type = tree.node_type(position.node_id)?;

    match node_type {
        NodeType::Paragraph => {
            // Position is relative to the paragraph - need to find which run
            let para = tree.get_paragraph(position.node_id)?;
            let mut accumulated_offset = 0;

            for (index, &run_id) in para.children().iter().enumerate() {
                if let Some(run) = tree.get_run(run_id) {
                    let run_len = run.text.chars().count();
                    if accumulated_offset + run_len >= position.offset {
                        return Some(ResolvedPosition {
                            paragraph_id: position.node_id,
                            run_id: Some(run_id),
                            run_index: Some(index),
                            offset_in_run: position.offset - accumulated_offset,
                        });
                    }
                    accumulated_offset += run_len;
                }
            }

            // Position is at the end of the paragraph (or paragraph is empty)
            let last_run = para.children().last().copied();
            let last_run_index = if para.children().is_empty() { None } else { Some(para.children().len() - 1) };
            Some(ResolvedPosition {
                paragraph_id: position.node_id,
                run_id: last_run,
                run_index: last_run_index,
                offset_in_run: if let Some(run_id) = last_run {
                    tree.get_run(run_id).map(|r| r.text.chars().count()).unwrap_or(0)
                } else {
                    0
                },
            })
        }
        NodeType::Run => {
            // Position is directly in a run
            let run = tree.get_run(position.node_id)?;
            let para_id = run.parent()?;
            let para = tree.get_paragraph(para_id)?;
            let run_index = para.children().iter().position(|&id| id == position.node_id)?;

            Some(ResolvedPosition {
                paragraph_id: para_id,
                run_id: Some(position.node_id),
                run_index: Some(run_index),
                offset_in_run: position.offset,
            })
        }
        _ => None,
    }
}

/// Get the total character length of a paragraph
fn paragraph_char_length(tree: &DocumentTree, para_id: NodeId) -> usize {
    let para = match tree.get_paragraph(para_id) {
        Some(p) => p,
        None => return 0,
    };

    para.children()
        .iter()
        .filter_map(|&run_id| tree.get_run(run_id))
        .map(|run| run.text.chars().count())
        .sum()
}

/// Extract text from a paragraph within a range
fn extract_paragraph_text(tree: &DocumentTree, para_id: NodeId, start_offset: usize, end_offset: usize) -> String {
    let para = match tree.get_paragraph(para_id) {
        Some(p) => p,
        None => return String::new(),
    };

    let mut result = String::new();
    let mut current_offset = 0;

    for &run_id in para.children() {
        if let Some(run) = tree.get_run(run_id) {
            let run_len = run.text.chars().count();
            let run_start = current_offset;
            let run_end = current_offset + run_len;

            // Check if this run overlaps with our range
            if run_end > start_offset && run_start < end_offset {
                let extract_start = if start_offset > run_start { start_offset - run_start } else { 0 };
                let extract_end = if end_offset < run_end { end_offset - run_start } else { run_len };

                // Extract the relevant portion of the run's text
                let chars: Vec<char> = run.text.chars().collect();
                result.extend(&chars[extract_start..extract_end]);
            }

            current_offset = run_end;
            if current_offset >= end_offset {
                break;
            }
        }
    }

    result
}

/// Get the paragraph index in the document
fn get_paragraph_index(tree: &DocumentTree, para_id: NodeId) -> Option<usize> {
    tree.document.children().iter().position(|&id| id == para_id)
}

/// Insert text at a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertText {
    pub position: Position,
    pub text: String,
}

impl InsertText {
    pub fn new(position: Position, text: impl Into<String>) -> Self {
        Self {
            position,
            text: text.into(),
        }
    }
}

impl Command for InsertText {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> crate::Result<CommandResult> {
        let mut new_tree = tree.clone();
        let inserted_char_count = self.text.chars().count();

        // Resolve the position to find the target paragraph and run
        let resolved = resolve_position(&new_tree, &self.position)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Cannot resolve position: {:?}", self.position)
            ))?;

        // Handle insertion based on whether there's an existing run
        match resolved.run_id {
            Some(run_id) => {
                // Insert into existing run
                let run = new_tree.get_run_mut(run_id)
                    .ok_or_else(|| crate::EditError::InvalidCommand(
                        format!("Run not found: {:?}", run_id)
                    ))?;

                // Convert char offset to byte offset for string manipulation
                let byte_offset: usize = run.text.chars()
                    .take(resolved.offset_in_run)
                    .map(|c| c.len_utf8())
                    .sum();

                run.text.insert_str(byte_offset, &self.text);
            }
            None => {
                // No runs exist, create a new run in the paragraph
                let run = Run::new(&self.text);
                new_tree.insert_run(run, resolved.paragraph_id, None)
                    .map_err(|e| crate::EditError::DocModel(e))?;
            }
        }

        // Calculate new cursor position
        let new_selection = Selection::collapsed(Position::new(
            self.position.node_id,
            self.position.offset + inserted_char_count,
        ));

        // Create the inverse command (delete the inserted text)
        let inverse = Box::new(DeleteRange {
            start: self.position,
            end: Position::new(self.position.node_id, self.position.offset + inserted_char_count),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        let char_count = self.text.chars().count();
        Box::new(DeleteRange {
            start: self.position,
            end: Position::new(self.position.node_id, self.position.offset + char_count),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        let char_count = self.text.chars().count();

        // Transform anchor
        let new_anchor = if selection.anchor.node_id == self.position.node_id
            && selection.anchor.offset >= self.position.offset
        {
            Position::new(selection.anchor.node_id, selection.anchor.offset + char_count)
        } else {
            selection.anchor
        };

        // Transform focus
        let new_focus = if selection.focus.node_id == self.position.node_id
            && selection.focus.offset >= self.position.offset
        {
            Position::new(selection.focus.node_id, selection.focus.offset + char_count)
        } else {
            selection.focus
        };

        Selection::new(new_anchor, new_focus)
    }

    fn display_name(&self) -> &str {
        "Insert Text"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }

    fn merge_with(&self, _other: &dyn Command) -> Option<Box<dyn Command>> {
        // Try to downcast to InsertText
        // For now, we check if it's sequential insertions at the same position
        // This is a simplified merge - real implementation would need more checks
        None
    }
}

/// Delete a range of text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRange {
    pub start: Position,
    pub end: Position,
}

impl DeleteRange {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Extract the text that will be deleted (for undo)
    fn extract_deleted_text(&self, tree: &DocumentTree) -> String {
        // Handle same-node deletion (most common case)
        if self.start.node_id == self.end.node_id {
            let node_type = tree.node_type(self.start.node_id);

            match node_type {
                Some(NodeType::Paragraph) => {
                    return extract_paragraph_text(
                        tree,
                        self.start.node_id,
                        self.start.offset,
                        self.end.offset,
                    );
                }
                Some(NodeType::Run) => {
                    if let Some(run) = tree.get_run(self.start.node_id) {
                        let chars: Vec<char> = run.text.chars().collect();
                        let start = self.start.offset.min(chars.len());
                        let end = self.end.offset.min(chars.len());
                        return chars[start..end].iter().collect();
                    }
                }
                _ => {}
            }
        }

        // Cross-paragraph deletion (more complex)
        // For now, return empty - full implementation below handles the actual deletion
        String::new()
    }
}

impl Command for DeleteRange {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> crate::Result<CommandResult> {
        let mut new_tree = tree.clone();

        // First, extract the text that will be deleted (for undo)
        let deleted_text = self.extract_deleted_text(tree);

        // Handle same-paragraph deletion
        if self.start.node_id == self.end.node_id {
            let node_type = new_tree.node_type(self.start.node_id);

            match node_type {
                Some(NodeType::Paragraph) => {
                    // Delete within a paragraph - need to modify runs
                    let start_offset = self.start.offset;
                    let end_offset = self.end.offset;

                    delete_range_in_paragraph(&mut new_tree, self.start.node_id, start_offset, end_offset)?;
                }
                Some(NodeType::Run) => {
                    // Delete directly within a run
                    let run = new_tree.get_run_mut(self.start.node_id)
                        .ok_or_else(|| crate::EditError::InvalidCommand(
                            format!("Run not found: {:?}", self.start.node_id)
                        ))?;

                    let chars: Vec<char> = run.text.chars().collect();
                    let start = self.start.offset.min(chars.len());
                    let end = self.end.offset.min(chars.len());

                    // Reconstruct the text without the deleted portion
                    run.text = chars[..start].iter().chain(chars[end..].iter()).collect();
                }
                _ => {
                    return Err(crate::EditError::InvalidCommand(
                        format!("Cannot delete in node type: {:?}", node_type)
                    ));
                }
            }
        } else {
            // Cross-paragraph deletion
            return Err(crate::EditError::InvalidCommand(
                "Cross-paragraph deletion not yet implemented".to_string()
            ));
        }

        let new_selection = Selection::collapsed(self.start);

        let inverse = Box::new(InsertText {
            position: self.start,
            text: deleted_text,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let deleted_text = self.extract_deleted_text(tree);
        Box::new(InsertText {
            position: self.start,
            text: deleted_text,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        let delete_length = if self.start.node_id == self.end.node_id {
            self.end.offset.saturating_sub(self.start.offset)
        } else {
            0 // Cross-node deletion - can't easily compute
        };

        // Transform anchor
        let new_anchor = if selection.anchor.node_id == self.start.node_id {
            if selection.anchor.offset >= self.end.offset {
                Position::new(selection.anchor.node_id, selection.anchor.offset - delete_length)
            } else if selection.anchor.offset > self.start.offset {
                Position::new(selection.anchor.node_id, self.start.offset)
            } else {
                selection.anchor
            }
        } else {
            selection.anchor
        };

        // Transform focus
        let new_focus = if selection.focus.node_id == self.start.node_id {
            if selection.focus.offset >= self.end.offset {
                Position::new(selection.focus.node_id, selection.focus.offset - delete_length)
            } else if selection.focus.offset > self.start.offset {
                Position::new(selection.focus.node_id, self.start.offset)
            } else {
                selection.focus
            }
        } else {
            selection.focus
        };

        Selection::new(new_anchor, new_focus)
    }

    fn display_name(&self) -> &str {
        "Delete"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Delete a range of characters within a single paragraph
fn delete_range_in_paragraph(
    tree: &mut DocumentTree,
    para_id: NodeId,
    start_offset: usize,
    end_offset: usize,
) -> crate::Result<()> {
    // Collect run information first to avoid borrow issues
    let run_info: Vec<(NodeId, usize, usize)> = {
        let para = tree.get_paragraph(para_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Paragraph not found: {:?}", para_id)
            ))?;

        let mut info = Vec::new();
        let mut offset = 0;

        for &run_id in para.children() {
            if let Some(run) = tree.get_run(run_id) {
                let len = run.text.chars().count();
                info.push((run_id, offset, len));
                offset += len;
            }
        }

        info
    };

    // Track runs to remove (those that become empty)
    let mut runs_to_remove = Vec::new();

    // Process each run that overlaps with the deletion range
    for (run_id, run_start, run_len) in run_info {
        let run_end = run_start + run_len;

        // Check if this run overlaps with the deletion range
        if run_end > start_offset && run_start < end_offset {
            let delete_start_in_run = if start_offset > run_start { start_offset - run_start } else { 0 };
            let delete_end_in_run = if end_offset < run_end { end_offset - run_start } else { run_len };

            // Get the run and modify its text
            if let Some(run) = tree.get_run_mut(run_id) {
                let chars: Vec<char> = run.text.chars().collect();
                let new_text: String = chars[..delete_start_in_run]
                    .iter()
                    .chain(chars[delete_end_in_run..].iter())
                    .collect();

                if new_text.is_empty() {
                    runs_to_remove.push(run_id);
                } else {
                    run.text = new_text;
                }
            }
        }
    }

    // Remove empty runs
    for run_id in runs_to_remove {
        let _ = tree.remove_run(run_id);
    }

    Ok(())
}

/// Split paragraph at position (Enter key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitParagraph {
    pub position: Position,
    /// The ID of the new paragraph that was created (set after apply, used for undo)
    #[serde(skip)]
    new_paragraph_id: Option<NodeId>,
}

impl SplitParagraph {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            new_paragraph_id: None,
        }
    }
}

impl Command for SplitParagraph {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> crate::Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Resolve the position to find the paragraph
        let resolved = resolve_position(&new_tree, &self.position)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Cannot resolve position: {:?}", self.position)
            ))?;

        let para_id = resolved.paragraph_id;

        // Get paragraph index for inserting new paragraph
        let para_index = get_paragraph_index(&new_tree, para_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Paragraph not in document: {:?}", para_id)
            ))?;

        // Collect information about runs to move to the new paragraph
        let runs_to_move: Vec<(NodeId, String, RunStyle)> = {
            let para = new_tree.get_paragraph(para_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Paragraph not found: {:?}", para_id)
                ))?;

            let mut runs = Vec::new();
            let mut current_offset = 0;
            let split_at = self.position.offset;

            for &run_id in para.children() {
                if let Some(run) = new_tree.get_run(run_id) {
                    let run_len = run.text.chars().count();
                    let run_end = current_offset + run_len;

                    if run_end > split_at {
                        // This run crosses or is after the split point
                        let chars: Vec<char> = run.text.chars().collect();
                        let split_in_run = if split_at > current_offset { split_at - current_offset } else { 0 };

                        if split_in_run < run_len {
                            // Part of this run goes to the new paragraph
                            let text_for_new: String = chars[split_in_run..].iter().collect();
                            if !text_for_new.is_empty() {
                                runs.push((run_id, text_for_new, run.style.clone()));
                            }
                        }
                    }
                    current_offset = run_end;
                }
            }

            runs
        };

        // Truncate runs in the original paragraph at the split point
        {
            let para = new_tree.get_paragraph(para_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Paragraph not found: {:?}", para_id)
                ))?;

            let mut current_offset = 0;
            let split_at = self.position.offset;
            let run_ids: Vec<NodeId> = para.children().to_vec();

            let mut runs_to_remove = Vec::new();

            for run_id in run_ids {
                if let Some(run) = new_tree.get_run_mut(run_id) {
                    let run_len = run.text.chars().count();
                    let run_end = current_offset + run_len;

                    if current_offset >= split_at {
                        // Entire run is after split point - mark for removal
                        runs_to_remove.push(run_id);
                    } else if run_end > split_at {
                        // Run crosses the split point - truncate it
                        let chars: Vec<char> = run.text.chars().collect();
                        let keep_count = split_at - current_offset;
                        run.text = chars[..keep_count].iter().collect();
                    }

                    current_offset = run_end;
                }
            }

            // Remove runs that are entirely after the split point
            for run_id in runs_to_remove {
                let _ = new_tree.remove_run(run_id);
            }
        }

        // Create the new paragraph
        let new_para = Paragraph::new();
        let new_para_id = new_para.id();

        // Insert the new paragraph after the current one
        new_tree.insert_paragraph(new_para, new_tree.root_id(), Some(para_index + 1))
            .map_err(|e| crate::EditError::DocModel(e))?;

        // Add runs to the new paragraph
        for (_old_run_id, text, style) in runs_to_move {
            let new_run = Run::with_style(text, style);
            new_tree.insert_run(new_run, new_para_id, None)
                .map_err(|e| crate::EditError::DocModel(e))?;
        }

        // New selection is at the start of the new paragraph
        let new_selection = Selection::collapsed(Position::new(new_para_id, 0));

        // Create inverse command that will merge the paragraphs back
        let inverse = Box::new(MergeParagraph {
            paragraph_id: new_para_id,
            merge_position: self.position.offset,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(MergeParagraph {
            paragraph_id: self.new_paragraph_id.unwrap_or_else(NodeId::new),
            merge_position: self.position.offset,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        // If the selection is after the split point in the same paragraph,
        // it needs to be moved to the new paragraph
        *selection
    }

    fn display_name(&self) -> &str {
        "Split Paragraph"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            position: self.position,
            new_paragraph_id: self.new_paragraph_id,
        })
    }
}

/// Merge paragraph with previous (Backspace at start)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeParagraph {
    /// The paragraph to merge (will be removed)
    pub paragraph_id: NodeId,
    /// The offset in the previous paragraph where content will be appended
    /// (used for creating proper undo command)
    pub merge_position: usize,
}

impl MergeParagraph {
    pub fn new(paragraph_id: NodeId) -> Self {
        Self {
            paragraph_id,
            merge_position: 0,
        }
    }

    /// Create a merge command with knowledge of the merge position
    pub fn with_merge_position(paragraph_id: NodeId, merge_position: usize) -> Self {
        Self {
            paragraph_id,
            merge_position,
        }
    }
}

impl Command for MergeParagraph {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> crate::Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Find the paragraph index
        let para_index = get_paragraph_index(&new_tree, self.paragraph_id)
            .ok_or_else(|| crate::EditError::InvalidCommand(
                format!("Paragraph not in document: {:?}", self.paragraph_id)
            ))?;

        // Must have a previous paragraph to merge with
        if para_index == 0 {
            return Err(crate::EditError::InvalidCommand(
                "Cannot merge first paragraph".to_string()
            ));
        }

        // Get the previous paragraph ID
        let prev_para_id = new_tree.document.children()[para_index - 1];

        // Calculate the position where the merge happens (end of previous paragraph)
        let merge_offset = paragraph_char_length(&new_tree, prev_para_id);

        // Collect runs from the paragraph being merged
        let runs_to_move: Vec<(String, RunStyle)> = {
            let para = new_tree.get_paragraph(self.paragraph_id)
                .ok_or_else(|| crate::EditError::InvalidCommand(
                    format!("Paragraph not found: {:?}", self.paragraph_id)
                ))?;

            para.children()
                .iter()
                .filter_map(|&run_id| {
                    new_tree.get_run(run_id).map(|run| (run.text.clone(), run.style.clone()))
                })
                .collect()
        };

        // Add runs to the previous paragraph
        for (text, style) in runs_to_move {
            let new_run = Run::with_style(text, style);
            new_tree.insert_run(new_run, prev_para_id, None)
                .map_err(|e| crate::EditError::DocModel(e))?;
        }

        // Remove the merged paragraph (this also removes its runs)
        new_tree.remove_paragraph(self.paragraph_id)
            .map_err(|e| crate::EditError::DocModel(e))?;

        // New selection is at the merge point in the previous paragraph
        let new_selection = Selection::collapsed(Position::new(prev_para_id, merge_offset));

        // Create inverse command that will split the paragraph at the merge point
        let inverse = Box::new(SplitParagraph {
            position: Position::new(prev_para_id, merge_offset),
            new_paragraph_id: None,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Find the previous paragraph to create the split command
        let para_index = get_paragraph_index(tree, self.paragraph_id);

        if let Some(idx) = para_index {
            if idx > 0 {
                let prev_para_id = tree.document.children()[idx - 1];
                let merge_offset = paragraph_char_length(tree, prev_para_id);

                return Box::new(SplitParagraph {
                    position: Position::new(prev_para_id, merge_offset),
                    new_paragraph_id: Some(self.paragraph_id),
                });
            }
        }

        // Fallback - shouldn't happen in normal operation
        Box::new(SplitParagraph {
            position: Position::new(self.paragraph_id, 0),
            new_paragraph_id: None,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        // If the selection is in the merged paragraph, move it to the previous paragraph
        *selection
    }

    fn display_name(&self) -> &str {
        "Merge Paragraph"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}
