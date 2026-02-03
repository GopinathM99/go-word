//! List and numbering commands for toggling, indenting, and managing lists

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    DocumentTree, ListProperties, Node, NodeId, NodeType, NumId, NumberingRegistry,
    Position, Selection,
};
use serde::{Deserialize, Serialize};

// =============================================================================
// Helper Functions
// =============================================================================

/// Get the paragraph ID for a position
fn get_paragraph_for_position(tree: &DocumentTree, position: &Position) -> Result<NodeId> {
    let node_type = tree
        .node_type(position.node_id)
        .ok_or_else(|| EditError::InvalidCommand(format!("Node not found: {:?}", position.node_id)))?;

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

            if let Some(hyperlink) = tree.get_hyperlink(parent_id) {
                return hyperlink
                    .parent()
                    .ok_or_else(|| EditError::InvalidCommand("Hyperlink has no parent".to_string()));
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
        _ => Err(EditError::InvalidCommand(format!(
            "Invalid node type for position: {:?}",
            node_type
        ))),
    }
}

/// Get all paragraph IDs in a selection range
fn get_paragraphs_in_selection(tree: &DocumentTree, selection: &Selection) -> Result<Vec<NodeId>> {
    let start_para = get_paragraph_for_position(tree, &selection.start())?;
    let end_para = get_paragraph_for_position(tree, &selection.end())?;

    if start_para == end_para {
        return Ok(vec![start_para]);
    }

    let mut paragraphs = Vec::new();
    let mut found_start = false;

    for para in tree.paragraphs() {
        let para_id = para.id();
        if para_id == start_para {
            found_start = true;
        }
        if found_start {
            paragraphs.push(para_id);
        }
        if para_id == end_para {
            break;
        }
    }

    Ok(paragraphs)
}

/// Get the current list properties for a paragraph
fn get_list_props(tree: &DocumentTree, para_id: NodeId) -> Option<ListProperties> {
    tree.get_paragraph(para_id)
        .and_then(|p| p.direct_formatting.list_props.clone())
}

// =============================================================================
// Toggle Bullet List Command
// =============================================================================

/// Toggle bullet list on/off for the selected paragraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleBulletList;

impl ToggleBulletList {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToggleBulletList {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ToggleBulletList {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Check if all paragraphs are already in a bullet list
        let all_in_bullet_list = paragraphs.iter().all(|&para_id| {
            if let Some(props) = get_list_props(&new_tree, para_id) {
                if props.is_in_list() {
                    return new_tree
                        .numbering
                        .is_bullet_list(props.num_id.unwrap());
                }
            }
            false
        });

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        // Toggle the list
        let bullet_id = NumberingRegistry::bullet_list_id();
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if all_in_bullet_list {
                    // Remove from list
                    para.direct_formatting.list_props = None;
                } else {
                    // Add to bullet list
                    para.direct_formatting.list_props = Some(ListProperties::new(bullet_id, 0));
                }
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(ToggleBulletList)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Toggle Bullet List"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Toggle Numbered List Command
// =============================================================================

/// Toggle numbered list on/off for the selected paragraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleNumberedList;

impl ToggleNumberedList {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToggleNumberedList {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ToggleNumberedList {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Check if all paragraphs are already in a numbered list
        let numbered_id = NumberingRegistry::numbered_list_id();
        let all_in_numbered_list = paragraphs.iter().all(|&para_id| {
            if let Some(props) = get_list_props(&new_tree, para_id) {
                if let Some(num_id) = props.num_id {
                    return !new_tree.numbering.is_bullet_list(num_id);
                }
            }
            false
        });

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        // Toggle the list
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if all_in_numbered_list {
                    // Remove from list
                    para.direct_formatting.list_props = None;
                } else {
                    // Add to numbered list
                    para.direct_formatting.list_props = Some(ListProperties::new(numbered_id, 0));
                }
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(ToggleNumberedList)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Toggle Numbered List"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Increase List Indent Command
// =============================================================================

/// Increase the indent level of list items (or start a list if not in one)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncreaseListIndent;

impl IncreaseListIndent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IncreaseListIndent {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for IncreaseListIndent {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                match &para.direct_formatting.list_props {
                    Some(props) if props.is_in_list() => {
                        // Increase level (max 8)
                        let new_level = (props.effective_level() + 1).min(8);
                        para.direct_formatting.list_props =
                            Some(ListProperties::new(props.num_id.unwrap(), new_level));
                    }
                    _ => {
                        // Not in a list - start a bullet list at level 0
                        let bullet_id = NumberingRegistry::bullet_list_id();
                        para.direct_formatting.list_props = Some(ListProperties::new(bullet_id, 0));
                    }
                }
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DecreaseListIndent)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Increase Indent"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Decrease List Indent Command
// =============================================================================

/// Decrease the indent level of list items (or remove from list if at level 0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreaseListIndent;

impl DecreaseListIndent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecreaseListIndent {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for DecreaseListIndent {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if let Some(props) = &para.direct_formatting.list_props {
                    if props.is_in_list() {
                        let current_level = props.effective_level();
                        if current_level == 0 {
                            // Remove from list
                            para.direct_formatting.list_props = None;
                        } else {
                            // Decrease level
                            para.direct_formatting.list_props =
                                Some(ListProperties::new(props.num_id.unwrap(), current_level - 1));
                        }
                    }
                }
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(IncreaseListIndent)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Decrease Indent"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Change List Type Command
// =============================================================================

/// Change the list type for selected paragraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeListType {
    /// The new numbering instance ID to use
    pub num_id: NumId,
}

impl ChangeListType {
    pub fn new(num_id: NumId) -> Self {
        Self { num_id }
    }

    pub fn bullet() -> Self {
        Self::new(NumberingRegistry::bullet_list_id())
    }

    pub fn numbered() -> Self {
        Self::new(NumberingRegistry::numbered_list_id())
    }

    pub fn legal() -> Self {
        Self::new(NumberingRegistry::legal_list_id())
    }
}

impl Command for ChangeListType {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                let level = para
                    .direct_formatting
                    .list_props
                    .as_ref()
                    .map(|p| p.effective_level())
                    .unwrap_or(0);
                para.direct_formatting.list_props = Some(ListProperties::new(self.num_id, level));
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(ChangeListType::bullet())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Change List Type"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Remove From List Command
// =============================================================================

/// Remove the selected paragraphs from any list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveFromList;

impl RemoveFromList {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RemoveFromList {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for RemoveFromList {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                para.direct_formatting.list_props = None;
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RemoveFromList)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove from List"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Restart Numbering Command
// =============================================================================

/// Restart numbering for a list at the current paragraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartNumbering {
    /// The paragraph to restart numbering at
    pub para_id: Option<NodeId>,
    /// The value to restart at (defaults to 1)
    pub start_value: u32,
}

impl RestartNumbering {
    pub fn new() -> Self {
        Self {
            para_id: None,
            start_value: 1,
        }
    }

    pub fn at_paragraph(para_id: NodeId) -> Self {
        Self {
            para_id: Some(para_id),
            start_value: 1,
        }
    }

    pub fn with_value(mut self, value: u32) -> Self {
        self.start_value = value;
        self
    }
}

impl Default for RestartNumbering {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for RestartNumbering {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        let para_id = match self.para_id {
            Some(id) => id,
            None => get_paragraph_for_position(&new_tree, &selection.anchor)?,
        };

        // Get the current list properties
        let para = new_tree
            .get_paragraph(para_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Paragraph not found: {:?}", para_id)))?;

        let list_props = para
            .direct_formatting
            .list_props
            .as_ref()
            .ok_or_else(|| EditError::InvalidCommand("Paragraph is not in a list".to_string()))?;

        if let Some(num_id) = list_props.num_id {
            let level = list_props.effective_level();
            // Reset the counter for this level
            new_tree
                .numbering
                .set_counter(num_id, level, self.start_value - 1);
        }

        // No tree modification needed for counter reset, but we return success
        let inverse = Box::new(RestartNumbering::new());

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestartNumbering::new())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restart Numbering"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set List Level Command
// =============================================================================

/// Set the list level for selected paragraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetListLevel {
    /// The target level (0-8)
    pub level: u8,
}

impl SetListLevel {
    pub fn new(level: u8) -> Self {
        Self {
            level: level.min(8),
        }
    }
}

impl Command for SetListLevel {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old list properties for undo
        let old_props: Vec<(NodeId, Option<ListProperties>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree
                    .get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if let Some(props) = &para.direct_formatting.list_props {
                    if let Some(num_id) = props.num_id {
                        para.direct_formatting.list_props =
                            Some(ListProperties::new(num_id, self.level));
                    }
                }
            }
        }

        let inverse = Box::new(RestoreListProperties { props: old_props });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetListLevel::new(0))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set List Level"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Restore List Properties Command (for undo)
// =============================================================================

/// Restore list properties (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreListProperties {
    props: Vec<(NodeId, Option<ListProperties>)>,
}

impl Command for RestoreListProperties {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store current props for redo
        let current_props: Vec<(NodeId, Option<ListProperties>)> = self
            .props
            .iter()
            .filter_map(|(para_id, _)| {
                new_tree
                    .get_paragraph(*para_id)
                    .map(|p| (*para_id, p.direct_formatting.list_props.clone()))
            })
            .collect();

        // Restore old props
        for (para_id, props) in &self.props {
            if let Some(para) = new_tree.get_paragraph_mut(*para_id) {
                para.direct_formatting.list_props = props.clone();
            }
        }

        let inverse = Box::new(RestoreListProperties {
            props: current_props,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestoreListProperties {
            props: self.props.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore List Properties"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Position, Run, Selection};

    fn create_test_tree_with_paragraph() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();

        let run = Run::new("Test paragraph content");
        tree.insert_run(run, para_id, None).unwrap();

        (tree, para_id)
    }

    #[test]
    fn test_toggle_bullet_list() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Toggle on
        let cmd = ToggleBulletList::new();
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert!(para.direct_formatting.list_props.is_some());
        let list_props = para.direct_formatting.list_props.as_ref().unwrap();
        assert!(list_props.is_in_list());

        // Toggle off
        let result2 = cmd.apply(&result.tree, &selection).unwrap();
        let para2 = result2.tree.get_paragraph(para_id).unwrap();
        assert!(para2.direct_formatting.list_props.is_none());
    }

    #[test]
    fn test_toggle_numbered_list() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Toggle on
        let cmd = ToggleNumberedList::new();
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert!(para.direct_formatting.list_props.is_some());
        let list_props = para.direct_formatting.list_props.as_ref().unwrap();
        assert!(list_props.is_in_list());
        assert!(!result
            .tree
            .numbering
            .is_bullet_list(list_props.num_id.unwrap()));
    }

    #[test]
    fn test_increase_decrease_indent() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Start in bullet list
        let cmd_bullet = ToggleBulletList::new();
        let result = cmd_bullet.apply(&tree, &selection).unwrap();

        // Increase indent
        let cmd_inc = IncreaseListIndent::new();
        let result2 = cmd_inc.apply(&result.tree, &selection).unwrap();

        let para = result2.tree.get_paragraph(para_id).unwrap();
        let list_props = para.direct_formatting.list_props.as_ref().unwrap();
        assert_eq!(list_props.effective_level(), 1);

        // Decrease indent
        let cmd_dec = DecreaseListIndent::new();
        let result3 = cmd_dec.apply(&result2.tree, &selection).unwrap();

        let para2 = result3.tree.get_paragraph(para_id).unwrap();
        let list_props2 = para2.direct_formatting.list_props.as_ref().unwrap();
        assert_eq!(list_props2.effective_level(), 0);

        // Decrease again - should remove from list
        let result4 = cmd_dec.apply(&result3.tree, &selection).unwrap();
        let para3 = result4.tree.get_paragraph(para_id).unwrap();
        assert!(para3.direct_formatting.list_props.is_none());
    }

    #[test]
    fn test_change_list_type() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Start with bullet list
        let cmd_bullet = ToggleBulletList::new();
        let result = cmd_bullet.apply(&tree, &selection).unwrap();

        // Change to numbered
        let cmd_change = ChangeListType::numbered();
        let result2 = cmd_change.apply(&result.tree, &selection).unwrap();

        let para = result2.tree.get_paragraph(para_id).unwrap();
        let list_props = para.direct_formatting.list_props.as_ref().unwrap();
        assert!(!result2
            .tree
            .numbering
            .is_bullet_list(list_props.num_id.unwrap()));
    }

    #[test]
    fn test_remove_from_list() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Start in bullet list
        let cmd_bullet = ToggleBulletList::new();
        let result = cmd_bullet.apply(&tree, &selection).unwrap();

        // Remove from list
        let cmd_remove = RemoveFromList::new();
        let result2 = cmd_remove.apply(&result.tree, &selection).unwrap();

        let para = result2.tree.get_paragraph(para_id).unwrap();
        assert!(para.direct_formatting.list_props.is_none());
    }

    #[test]
    fn test_set_list_level() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Start in bullet list
        let cmd_bullet = ToggleBulletList::new();
        let result = cmd_bullet.apply(&tree, &selection).unwrap();

        // Set level to 3
        let cmd_level = SetListLevel::new(3);
        let result2 = cmd_level.apply(&result.tree, &selection).unwrap();

        let para = result2.tree.get_paragraph(para_id).unwrap();
        let list_props = para.direct_formatting.list_props.as_ref().unwrap();
        assert_eq!(list_props.effective_level(), 3);
    }
}
