//! Hyperlink commands for creating, editing, and removing hyperlinks

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    DocumentTree, Hyperlink, HyperlinkTarget, Node, NodeId, NodeType,
    Paragraph, Position, Run, RunStyle, Selection,
};
use serde::{Deserialize, Serialize};

/// Insert a hyperlink wrapping the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertHyperlink {
    /// The link target
    pub target: HyperlinkTarget,
    /// Optional tooltip
    pub tooltip: Option<String>,
    /// Display text (if different from selection or if selection is collapsed)
    pub display_text: Option<String>,
}

impl InsertHyperlink {
    /// Create a new insert hyperlink command
    pub fn new(target: HyperlinkTarget) -> Self {
        Self {
            target,
            tooltip: None,
            display_text: None,
        }
    }

    /// Create with tooltip
    pub fn with_tooltip(target: HyperlinkTarget, tooltip: impl Into<String>) -> Self {
        Self {
            target,
            tooltip: Some(tooltip.into()),
            display_text: None,
        }
    }

    /// Create with display text
    pub fn with_display_text(target: HyperlinkTarget, display_text: impl Into<String>) -> Self {
        Self {
            target,
            tooltip: None,
            display_text: Some(display_text.into()),
        }
    }

    /// Create with both tooltip and display text
    pub fn with_all(
        target: HyperlinkTarget,
        tooltip: Option<String>,
        display_text: Option<String>,
    ) -> Self {
        Self {
            target,
            tooltip,
            display_text,
        }
    }
}

impl Command for InsertHyperlink {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Validate the target first
        self.target.validate().map_err(|e| {
            EditError::InvalidCommand(format!("Invalid hyperlink target: {}", e))
        })?;

        let mut new_tree = tree.clone();

        // Get the paragraph containing the selection
        let para_id = get_paragraph_for_position(&new_tree, &selection.anchor)?;

        // Determine the text to use for the hyperlink
        let link_text = if let Some(ref text) = self.display_text {
            text.clone()
        } else if selection.is_collapsed() {
            // If selection is collapsed and no display text, use the URL
            self.target.to_url()
        } else {
            // Extract the selected text
            extract_selected_text(&new_tree, selection)?
        };

        // If selection is not collapsed, we need to replace the selected text
        if !selection.is_collapsed() {
            // Delete the selected content first
            delete_selection_content(&mut new_tree, selection)?;
        }

        // Create the hyperlink with a run containing the link text
        let mut hyperlink = match &self.tooltip {
            Some(tip) => Hyperlink::with_tooltip(self.target.clone(), tip.clone()),
            None => Hyperlink::new(self.target.clone()),
        };

        // Create a run with hyperlink styling (blue, underlined)
        let mut link_style = RunStyle::default();
        link_style.color = Some("#0000FF".to_string());
        link_style.underline = Some(true);

        let run = Run::with_style(&link_text, link_style);
        let run_id = run.id();

        // Find the insertion point in the paragraph
        let insert_index = find_insertion_index(&new_tree, para_id, &selection.start())?;

        // Insert the hyperlink
        let hyperlink_id = new_tree.insert_hyperlink(hyperlink, para_id, Some(insert_index))
            .map_err(|e| EditError::DocModel(e))?;

        // Insert the run into the hyperlink
        new_tree.insert_run_into_hyperlink(run, hyperlink_id, None)
            .map_err(|e| EditError::DocModel(e))?;

        // Calculate new selection position (at the end of the inserted hyperlink)
        let new_offset = selection.start().offset + link_text.chars().count();
        let new_selection = Selection::collapsed(Position::new(para_id, new_offset));

        // Create the inverse command
        let inverse = Box::new(RemoveHyperlinkById {
            hyperlink_id,
            original_selection: *selection,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // This will be replaced by the proper inverse in apply()
        Box::new(RemoveHyperlink::new())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Hyperlink"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Remove a hyperlink but keep the text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveHyperlink {
    /// Optional specific hyperlink ID to remove (if None, removes hyperlink at selection)
    pub hyperlink_id: Option<NodeId>,
}

impl RemoveHyperlink {
    pub fn new() -> Self {
        Self { hyperlink_id: None }
    }

    pub fn with_id(hyperlink_id: NodeId) -> Self {
        Self {
            hyperlink_id: Some(hyperlink_id),
        }
    }
}

impl Default for RemoveHyperlink {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for RemoveHyperlink {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Find the hyperlink to remove
        let hyperlink_id = match self.hyperlink_id {
            Some(id) => id,
            None => {
                // Find hyperlink at selection
                find_hyperlink_at_selection(&new_tree, selection)?
            }
        };

        // Get the hyperlink info before removing
        let hyperlink = new_tree.get_hyperlink(hyperlink_id)
            .ok_or_else(|| EditError::InvalidCommand(
                format!("Hyperlink not found: {:?}", hyperlink_id)
            ))?;

        let para_id = hyperlink.parent()
            .ok_or_else(|| EditError::InvalidCommand(
                "Hyperlink has no parent".to_string()
            ))?;

        let target = hyperlink.target.clone();
        let tooltip = hyperlink.tooltip.clone();

        // Get the child run IDs and their text
        let child_ids: Vec<NodeId> = hyperlink.children().to_vec();
        let mut runs_data: Vec<(String, RunStyle)> = Vec::new();

        for &run_id in &child_ids {
            if let Some(run) = new_tree.get_run(run_id) {
                // Remove hyperlink styling
                let mut new_style = run.style.clone();
                new_style.color = None;
                new_style.underline = None;
                runs_data.push((run.text.clone(), new_style));
            }
        }

        // Find the hyperlink's position in the paragraph
        let para = new_tree.get_paragraph(para_id)
            .ok_or_else(|| EditError::InvalidCommand(
                format!("Paragraph not found: {:?}", para_id)
            ))?;

        let hyperlink_index = para.children()
            .iter()
            .position(|&id| id == hyperlink_id)
            .ok_or_else(|| EditError::InvalidCommand(
                "Hyperlink not found in paragraph".to_string()
            ))?;

        // Remove the hyperlink (this also removes child runs)
        new_tree.remove_hyperlink(hyperlink_id)
            .map_err(|e| EditError::DocModel(e))?;

        // Insert the runs directly into the paragraph
        for (i, (text, style)) in runs_data.into_iter().enumerate() {
            let run = Run::with_style(text, style);
            new_tree.insert_run(run, para_id, Some(hyperlink_index + i))
                .map_err(|e| EditError::DocModel(e))?;
        }

        // Create the inverse command
        let inverse = Box::new(InsertHyperlink::with_all(
            target,
            tooltip,
            None, // The text is already in place
        ));

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Get hyperlink info if available
        if let Some(hyperlink_id) = self.hyperlink_id {
            if let Some(hyperlink) = tree.get_hyperlink(hyperlink_id) {
                return Box::new(InsertHyperlink::with_all(
                    hyperlink.target.clone(),
                    hyperlink.tooltip.clone(),
                    None,
                ));
            }
        }
        Box::new(InsertHyperlink::new(HyperlinkTarget::external("")))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Hyperlink"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Remove a specific hyperlink by ID (used for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoveHyperlinkById {
    hyperlink_id: NodeId,
    original_selection: Selection,
}

impl Command for RemoveHyperlinkById {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        let cmd = RemoveHyperlink::with_id(self.hyperlink_id);
        cmd.apply(tree, &self.original_selection)
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(hyperlink) = tree.get_hyperlink(self.hyperlink_id) {
            Box::new(InsertHyperlink::with_all(
                hyperlink.target.clone(),
                hyperlink.tooltip.clone(),
                None,
            ))
        } else {
            Box::new(InsertHyperlink::new(HyperlinkTarget::external("")))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove Hyperlink"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Edit an existing hyperlink's target and/or tooltip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditHyperlink {
    /// The hyperlink to edit (if None, edits hyperlink at selection)
    pub hyperlink_id: Option<NodeId>,
    /// New target (if Some, updates the target)
    pub new_target: Option<HyperlinkTarget>,
    /// New tooltip (if Some, updates the tooltip; use Some(None) to remove tooltip)
    pub new_tooltip: Option<Option<String>>,
}

impl EditHyperlink {
    pub fn new() -> Self {
        Self {
            hyperlink_id: None,
            new_target: None,
            new_tooltip: None,
        }
    }

    pub fn with_target(target: HyperlinkTarget) -> Self {
        Self {
            hyperlink_id: None,
            new_target: Some(target),
            new_tooltip: None,
        }
    }

    pub fn with_tooltip(tooltip: Option<String>) -> Self {
        Self {
            hyperlink_id: None,
            new_target: None,
            new_tooltip: Some(tooltip),
        }
    }

    pub fn with_id(hyperlink_id: NodeId) -> Self {
        Self {
            hyperlink_id: Some(hyperlink_id),
            new_target: None,
            new_tooltip: None,
        }
    }

    pub fn set_target(mut self, target: HyperlinkTarget) -> Self {
        self.new_target = Some(target);
        self
    }

    pub fn set_tooltip(mut self, tooltip: Option<String>) -> Self {
        self.new_tooltip = Some(tooltip);
        self
    }
}

impl Default for EditHyperlink {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EditHyperlink {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Validate new target if provided
        if let Some(ref target) = self.new_target {
            target.validate().map_err(|e| {
                EditError::InvalidCommand(format!("Invalid hyperlink target: {}", e))
            })?;
        }

        let mut new_tree = tree.clone();

        // Find the hyperlink to edit
        let hyperlink_id = match self.hyperlink_id {
            Some(id) => id,
            None => find_hyperlink_at_selection(&new_tree, selection)?,
        };

        // Store old values for undo
        let hyperlink = new_tree.get_hyperlink(hyperlink_id)
            .ok_or_else(|| EditError::InvalidCommand(
                format!("Hyperlink not found: {:?}", hyperlink_id)
            ))?;

        let old_target = hyperlink.target.clone();
        let old_tooltip = hyperlink.tooltip.clone();

        // Apply changes
        let hyperlink = new_tree.get_hyperlink_mut(hyperlink_id)
            .ok_or_else(|| EditError::InvalidCommand(
                format!("Hyperlink not found: {:?}", hyperlink_id)
            ))?;

        if let Some(ref target) = self.new_target {
            hyperlink.set_target(target.clone());
        }

        if let Some(ref tooltip) = self.new_tooltip {
            hyperlink.set_tooltip(tooltip.clone());
        }

        // Create the inverse command
        let mut inverse = EditHyperlink::with_id(hyperlink_id);
        if self.new_target.is_some() {
            inverse = inverse.set_target(old_target);
        }
        if self.new_tooltip.is_some() {
            inverse = inverse.set_tooltip(old_tooltip);
        }

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let hyperlink_id = self.hyperlink_id.unwrap_or_else(NodeId::new);

        if let Some(hyperlink) = tree.get_hyperlink(hyperlink_id) {
            let mut cmd = EditHyperlink::with_id(hyperlink_id);
            if self.new_target.is_some() {
                cmd = cmd.set_target(hyperlink.target.clone());
            }
            if self.new_tooltip.is_some() {
                cmd = cmd.set_tooltip(hyperlink.tooltip.clone());
            }
            Box::new(cmd)
        } else {
            Box::new(EditHyperlink::new())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Edit Hyperlink"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Get the paragraph containing a position
fn get_paragraph_for_position(tree: &DocumentTree, position: &Position) -> Result<NodeId> {
    let node_type = tree.node_type(position.node_id)
        .ok_or_else(|| EditError::InvalidCommand(
            format!("Node not found: {:?}", position.node_id)
        ))?;

    match node_type {
        NodeType::Paragraph => Ok(position.node_id),
        NodeType::Run => {
            let run = tree.get_run(position.node_id)
                .ok_or_else(|| EditError::InvalidCommand(
                    format!("Run not found: {:?}", position.node_id)
                ))?;

            // Parent could be a paragraph or hyperlink
            let parent_id = run.parent()
                .ok_or_else(|| EditError::InvalidCommand(
                    "Run has no parent".to_string()
                ))?;

            if tree.get_paragraph(parent_id).is_some() {
                return Ok(parent_id);
            }

            // Parent is a hyperlink
            if let Some(hyperlink) = tree.get_hyperlink(parent_id) {
                return hyperlink.parent()
                    .ok_or_else(|| EditError::InvalidCommand(
                        "Hyperlink has no parent".to_string()
                    ));
            }

            Err(EditError::InvalidCommand("Cannot determine paragraph".to_string()))
        }
        NodeType::Hyperlink => {
            let hyperlink = tree.get_hyperlink(position.node_id)
                .ok_or_else(|| EditError::InvalidCommand(
                    format!("Hyperlink not found: {:?}", position.node_id)
                ))?;

            hyperlink.parent()
                .ok_or_else(|| EditError::InvalidCommand(
                    "Hyperlink has no parent".to_string()
                ))
        }
        _ => Err(EditError::InvalidCommand(
            format!("Invalid node type for position: {:?}", node_type)
        )),
    }
}

/// Extract text from selection
fn extract_selected_text(tree: &DocumentTree, selection: &Selection) -> Result<String> {
    if selection.is_collapsed() {
        return Ok(String::new());
    }

    let start = selection.start();
    let end = selection.end();

    // Simple case: selection within same node
    if start.node_id == end.node_id {
        let node_type = tree.node_type(start.node_id);

        match node_type {
            Some(NodeType::Paragraph) => {
                let para = tree.get_paragraph(start.node_id)
                    .ok_or_else(|| EditError::InvalidCommand("Paragraph not found".to_string()))?;

                let mut text = String::new();
                let mut offset = 0;

                for &child_id in para.children() {
                    if let Some(run) = tree.get_run(child_id) {
                        let run_len = run.text.chars().count();
                        let run_start = offset;
                        let run_end = offset + run_len;

                        if run_end > start.offset && run_start < end.offset {
                            let extract_start = start.offset.saturating_sub(run_start);
                            let extract_end = (end.offset - run_start).min(run_len);

                            let chars: Vec<char> = run.text.chars().collect();
                            text.extend(&chars[extract_start..extract_end]);
                        }

                        offset = run_end;
                    }
                }

                Ok(text)
            }
            Some(NodeType::Run) => {
                let run = tree.get_run(start.node_id)
                    .ok_or_else(|| EditError::InvalidCommand("Run not found".to_string()))?;

                let chars: Vec<char> = run.text.chars().collect();
                let text: String = chars[start.offset..end.offset].iter().collect();
                Ok(text)
            }
            _ => Err(EditError::InvalidCommand("Invalid selection".to_string())),
        }
    } else {
        // Cross-node selection - simplified for now
        Err(EditError::InvalidCommand(
            "Cross-node selection not yet supported for hyperlinks".to_string()
        ))
    }
}

/// Delete the selected content
fn delete_selection_content(tree: &mut DocumentTree, selection: &Selection) -> Result<()> {
    if selection.is_collapsed() {
        return Ok(());
    }

    let start = selection.start();
    let end = selection.end();

    // Simple case: selection within same paragraph
    if start.node_id == end.node_id {
        let node_type = tree.node_type(start.node_id);

        if let Some(NodeType::Paragraph) = node_type {
            let para = tree.get_paragraph(start.node_id)
                .ok_or_else(|| EditError::InvalidCommand("Paragraph not found".to_string()))?;

            let children: Vec<NodeId> = para.children().to_vec();
            let mut offset = 0;
            let mut runs_to_modify: Vec<(NodeId, usize, usize)> = Vec::new();
            let mut runs_to_remove: Vec<NodeId> = Vec::new();

            for &child_id in &children {
                if let Some(run) = tree.get_run(child_id) {
                    let run_len = run.text.chars().count();
                    let run_start = offset;
                    let run_end = offset + run_len;

                    if run_end > start.offset && run_start < end.offset {
                        let delete_start = start.offset.saturating_sub(run_start);
                        let delete_end = (end.offset - run_start).min(run_len);

                        if delete_start == 0 && delete_end == run_len {
                            // Remove entire run
                            runs_to_remove.push(child_id);
                        } else {
                            // Modify run
                            runs_to_modify.push((child_id, delete_start, delete_end));
                        }
                    }

                    offset = run_end;
                }
            }

            // Remove runs
            for run_id in runs_to_remove {
                let _ = tree.remove_run(run_id);
            }

            // Modify runs
            for (run_id, delete_start, delete_end) in runs_to_modify {
                if let Some(run) = tree.get_run_mut(run_id) {
                    let chars: Vec<char> = run.text.chars().collect();
                    run.text = chars[..delete_start]
                        .iter()
                        .chain(chars[delete_end..].iter())
                        .collect();
                }
            }
        }
    }

    Ok(())
}

/// Find the insertion index in a paragraph for a given position
fn find_insertion_index(tree: &DocumentTree, para_id: NodeId, position: &Position) -> Result<usize> {
    let para = tree.get_paragraph(para_id)
        .ok_or_else(|| EditError::InvalidCommand(
            format!("Paragraph not found: {:?}", para_id)
        ))?;

    let mut offset = 0;

    for (index, &child_id) in para.children().iter().enumerate() {
        if let Some(run) = tree.get_run(child_id) {
            let run_len = run.text.chars().count();
            if offset + run_len >= position.offset {
                return Ok(index);
            }
            offset += run_len;
        } else if tree.get_hyperlink(child_id).is_some() {
            // Handle hyperlinks
            if let Some(hyperlink) = tree.get_hyperlink(child_id) {
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
        }
    }

    Ok(para.children().len())
}

/// Find hyperlink at the current selection
fn find_hyperlink_at_selection(tree: &DocumentTree, selection: &Selection) -> Result<NodeId> {
    let position = &selection.anchor;
    let para_id = get_paragraph_for_position(tree, position)?;

    let para = tree.get_paragraph(para_id)
        .ok_or_else(|| EditError::InvalidCommand(
            format!("Paragraph not found: {:?}", para_id)
        ))?;

    let mut offset = 0;

    for &child_id in para.children() {
        if let Some(hyperlink) = tree.get_hyperlink(child_id) {
            let mut hyperlink_len = 0;
            for &run_id in hyperlink.children() {
                if let Some(run) = tree.get_run(run_id) {
                    hyperlink_len += run.text.chars().count();
                }
            }

            if position.offset >= offset && position.offset < offset + hyperlink_len {
                return Ok(child_id);
            }
            offset += hyperlink_len;
        } else if let Some(run) = tree.get_run(child_id) {
            offset += run.text.chars().count();
        }
    }

    Err(EditError::InvalidCommand(
        "No hyperlink found at selection".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree_with_text(text: &str) -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();

        if !text.is_empty() {
            let run = Run::new(text);
            tree.insert_run(run, para_id, None).unwrap();
        }

        (tree, para_id)
    }

    #[test]
    fn test_insert_hyperlink_collapsed_selection() {
        let (tree, para_id) = create_test_tree_with_text("");
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertHyperlink::new(HyperlinkTarget::external("https://example.com"));
        let result = cmd.apply(&tree, &selection).unwrap();

        // Should have inserted the URL as text
        assert!(result.tree.hyperlinks().count() > 0);
    }

    #[test]
    fn test_insert_hyperlink_with_display_text() {
        let (tree, para_id) = create_test_tree_with_text("");
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertHyperlink::with_all(
            HyperlinkTarget::external("https://example.com"),
            Some("Click here".to_string()),
            Some("Example".to_string()),
        );
        let result = cmd.apply(&tree, &selection).unwrap();

        let hyperlink = result.tree.hyperlinks().next().unwrap();
        assert!(hyperlink.tooltip.is_some());
    }

    #[test]
    fn test_edit_hyperlink() {
        let (mut tree, para_id) = create_test_tree_with_text("");

        // First insert a hyperlink
        let hyperlink = Hyperlink::new(HyperlinkTarget::external("https://old.com"));
        let hyperlink_id = tree.insert_hyperlink(hyperlink, para_id, None).unwrap();

        let run = Run::new("Old Link");
        tree.insert_run_into_hyperlink(run, hyperlink_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Edit the hyperlink
        let cmd = EditHyperlink::with_id(hyperlink_id)
            .set_target(HyperlinkTarget::external("https://new.com"))
            .set_tooltip(Some("New tooltip".to_string()));

        let result = cmd.apply(&tree, &selection).unwrap();

        let edited_hyperlink = result.tree.get_hyperlink(hyperlink_id).unwrap();
        assert!(matches!(&edited_hyperlink.target, HyperlinkTarget::External(url) if url == "https://new.com"));
        assert_eq!(edited_hyperlink.tooltip, Some("New tooltip".to_string()));
    }

    #[test]
    fn test_remove_hyperlink() {
        let (mut tree, para_id) = create_test_tree_with_text("");

        // First insert a hyperlink
        let hyperlink = Hyperlink::new(HyperlinkTarget::external("https://example.com"));
        let hyperlink_id = tree.insert_hyperlink(hyperlink, para_id, None).unwrap();

        let run = Run::new("Link Text");
        tree.insert_run_into_hyperlink(run, hyperlink_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Remove the hyperlink
        let cmd = RemoveHyperlink::with_id(hyperlink_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Hyperlink should be gone
        assert!(result.tree.get_hyperlink(hyperlink_id).is_none());

        // But the text should still be there as a regular run
        let para = result.tree.get_paragraph(para_id).unwrap();
        assert!(!para.children().is_empty());
    }

    #[test]
    fn test_hyperlink_validation() {
        let (tree, para_id) = create_test_tree_with_text("");
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Empty URL should fail
        let cmd = InsertHyperlink::new(HyperlinkTarget::external(""));
        assert!(cmd.apply(&tree, &selection).is_err());

        // JavaScript URL should fail
        let cmd = InsertHyperlink::new(HyperlinkTarget::external("javascript:alert('xss')"));
        assert!(cmd.apply(&tree, &selection).is_err());
    }
}
