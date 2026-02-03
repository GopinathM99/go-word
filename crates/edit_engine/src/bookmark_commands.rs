//! Bookmark commands for creating, editing, and navigating bookmarks
//!
//! Bookmarks provide named anchors in the document that can be:
//! - Navigated to via Go To dialog
//! - Referenced by internal hyperlinks
//! - Used for cross-references

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    Bookmark, BookmarkRange, DocumentTree, Node, NodeId, Selection,
};
use serde::{Deserialize, Serialize};

/// Insert a bookmark at the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertBookmark {
    /// The name for the bookmark
    pub name: String,
}

impl InsertBookmark {
    /// Create a new insert bookmark command
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Command for InsertBookmark {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Create the bookmark from the current selection
        let bookmark = Bookmark::from_selection(&self.name, selection.anchor, selection.focus);

        // Try to insert the bookmark
        let bookmark_id = new_tree
            .bookmark_registry_mut()
            .insert(bookmark)
            .map_err(|e| EditError::InvalidCommand(format!("Invalid bookmark: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(DeleteBookmark {
            bookmark_id: Some(bookmark_id),
            name: None,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection, // Selection stays the same
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteBookmark {
            bookmark_id: None,
            name: Some(self.name.clone()),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Bookmark"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Delete a bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBookmark {
    /// Delete by bookmark ID
    pub bookmark_id: Option<NodeId>,
    /// Delete by bookmark name
    pub name: Option<String>,
}

impl DeleteBookmark {
    /// Create a delete bookmark command by ID
    pub fn by_id(bookmark_id: NodeId) -> Self {
        Self {
            bookmark_id: Some(bookmark_id),
            name: None,
        }
    }

    /// Create a delete bookmark command by name
    pub fn by_name(name: impl Into<String>) -> Self {
        Self {
            bookmark_id: None,
            name: Some(name.into()),
        }
    }
}

impl Command for DeleteBookmark {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Find and remove the bookmark
        let removed_bookmark = if let Some(id) = self.bookmark_id {
            new_tree.remove_bookmark(id)
        } else if let Some(ref name) = self.name {
            new_tree.remove_bookmark_by_name(name)
        } else {
            return Err(EditError::InvalidCommand(
                "DeleteBookmark requires either bookmark_id or name".to_string(),
            ));
        };

        let removed_bookmark = removed_bookmark.ok_or_else(|| {
            EditError::InvalidCommand("Bookmark not found".to_string())
        })?;

        // Create the inverse command (re-insert the bookmark)
        let inverse = Box::new(InsertBookmarkWithData {
            name: removed_bookmark.name().to_string(),
            range: removed_bookmark.range().clone(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Get the bookmark info for the inverse
        let bookmark = if let Some(id) = self.bookmark_id {
            tree.get_bookmark(id)
        } else if let Some(ref name) = self.name {
            tree.get_bookmark_by_name(name)
        } else {
            None
        };

        if let Some(b) = bookmark {
            Box::new(InsertBookmarkWithData {
                name: b.name().to_string(),
                range: b.range().clone(),
            })
        } else {
            // Fallback - this shouldn't happen in normal operation
            Box::new(InsertBookmark::new(""))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Bookmark"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Internal command for re-inserting a bookmark with specific data (used for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InsertBookmarkWithData {
    name: String,
    range: BookmarkRange,
}

impl Command for InsertBookmarkWithData {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Create the bookmark with the stored data
        let bookmark = match &self.range {
            BookmarkRange::Point(pos) => Bookmark::new_point(&self.name, *pos),
            BookmarkRange::Range { start, end } => Bookmark::new_range(&self.name, *start, *end),
        };

        let bookmark_id = new_tree
            .bookmark_registry_mut()
            .insert(bookmark)
            .map_err(|e| EditError::InvalidCommand(format!("Invalid bookmark: {}", e)))?;

        let inverse = Box::new(DeleteBookmark::by_id(bookmark_id));

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteBookmark::by_name(&self.name))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Insert Bookmark"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Rename a bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameBookmark {
    /// The current name of the bookmark
    pub old_name: String,
    /// The new name for the bookmark
    pub new_name: String,
}

impl RenameBookmark {
    /// Create a new rename bookmark command
    pub fn new(old_name: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self {
            old_name: old_name.into(),
            new_name: new_name.into(),
        }
    }
}

impl Command for RenameBookmark {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Find the bookmark by old name
        let bookmark = new_tree
            .get_bookmark_by_name(&self.old_name)
            .ok_or_else(|| {
                EditError::InvalidCommand(format!("Bookmark '{}' not found", self.old_name))
            })?;

        let bookmark_id = bookmark.id();

        // Rename the bookmark
        new_tree
            .rename_bookmark(bookmark_id, &self.new_name)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot rename bookmark: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(RenameBookmark {
            old_name: self.new_name.clone(),
            new_name: self.old_name.clone(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RenameBookmark {
            old_name: self.new_name.clone(),
            new_name: self.old_name.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Rename Bookmark"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Navigate to a bookmark (updates selection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoToBookmark {
    /// The name of the bookmark to navigate to
    pub name: String,
}

impl GoToBookmark {
    /// Create a new go to bookmark command
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Command for GoToBookmark {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Find the bookmark
        let bookmark = tree.get_bookmark_by_name(&self.name).ok_or_else(|| {
            EditError::InvalidCommand(format!("Bookmark '{}' not found", self.name))
        })?;

        // Create the new selection based on bookmark range
        let new_selection = match bookmark.range() {
            BookmarkRange::Point(pos) => Selection::collapsed(*pos),
            BookmarkRange::Range { start, end } => Selection::new(*start, *end),
        };

        // Create the inverse command (go back to original selection)
        let inverse = Box::new(SetSelection {
            selection: *selection,
        });

        Ok(CommandResult {
            tree: tree.clone(),
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // Can't really invert this without knowing the previous selection
        Box::new(GoToBookmark::new(&self.name))
    }

    fn transform_selection(&self, _selection: &Selection) -> Selection {
        // This command always replaces the selection
        Selection::default()
    }

    fn display_name(&self) -> &str {
        "Go To Bookmark"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Internal command to restore a selection (used for undo of GoToBookmark)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetSelection {
    selection: Selection,
}

impl Command for SetSelection {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: self.selection,
            inverse: Box::new(SetSelection {
                selection: self.selection,
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetSelection {
            selection: self.selection,
        })
    }

    fn transform_selection(&self, _selection: &Selection) -> Selection {
        self.selection
    }

    fn display_name(&self) -> &str {
        "Set Selection"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Information about a bookmark for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkInfo {
    /// The bookmark ID (as string for JSON serialization)
    pub id: String,
    /// The bookmark name
    pub name: String,
    /// Whether this is a point bookmark (vs range)
    pub is_point: bool,
    /// Preview text near the bookmark location (if available)
    pub preview: Option<String>,
    /// The paragraph containing the bookmark start
    pub paragraph_id: String,
    /// Character offset in the paragraph
    pub offset: usize,
}

impl BookmarkInfo {
    /// Create bookmark info from a bookmark
    pub fn from_bookmark(bookmark: &Bookmark, preview: Option<String>) -> Self {
        let start_pos = bookmark.start_position();
        Self {
            id: bookmark.id().to_string(),
            name: bookmark.name().to_string(),
            is_point: bookmark.is_point(),
            preview,
            paragraph_id: start_pos.node_id.to_string(),
            offset: start_pos.offset,
        }
    }
}

/// Get a list of all bookmarks in the document (utility function)
pub fn list_bookmarks(tree: &DocumentTree) -> Vec<BookmarkInfo> {
    tree.all_bookmarks()
        .map(|bookmark| {
            // Try to get preview text from the paragraph
            let preview = get_bookmark_preview(tree, bookmark);
            BookmarkInfo::from_bookmark(bookmark, preview)
        })
        .collect()
}

/// Get preview text for a bookmark (first ~30 characters at the bookmark location)
fn get_bookmark_preview(tree: &DocumentTree, bookmark: &Bookmark) -> Option<String> {
    let start_pos = bookmark.start_position();

    // Get the paragraph
    let para = tree.get_paragraph(start_pos.node_id)?;

    // Collect text from runs
    let mut text = String::new();
    let mut current_offset = 0;

    for &run_id in para.children() {
        if let Some(run) = tree.get_run(run_id) {
            let run_len = run.text.chars().count();
            let run_end = current_offset + run_len;

            // Check if this run contains or comes after the bookmark start
            if run_end > start_pos.offset {
                let start_in_run = if start_pos.offset > current_offset {
                    start_pos.offset - current_offset
                } else {
                    0
                };

                // Get text from this run
                let chars: Vec<char> = run.text.chars().collect();
                for &c in &chars[start_in_run..] {
                    text.push(c);
                    if text.len() >= 30 {
                        break;
                    }
                }

                if text.len() >= 30 {
                    break;
                }
            }

            current_offset = run_end;
        }
    }

    if text.is_empty() {
        None
    } else {
        if text.len() >= 30 {
            text.truncate(27);
            text.push_str("...");
        }
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Position, Run};

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
    fn test_insert_bookmark() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        let cmd = InsertBookmark::new("test_bookmark");
        let result = cmd.apply(&tree, &selection).unwrap();

        // Bookmark should exist
        assert!(result.tree.has_bookmark("test_bookmark"));
        let bookmark = result.tree.get_bookmark_by_name("test_bookmark").unwrap();
        assert!(bookmark.is_point());
        assert_eq!(bookmark.start_position().offset, 5);
    }

    #[test]
    fn test_insert_range_bookmark() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        let cmd = InsertBookmark::new("range_bookmark");
        let result = cmd.apply(&tree, &selection).unwrap();

        let bookmark = result.tree.get_bookmark_by_name("range_bookmark").unwrap();
        assert!(bookmark.is_range());
    }

    #[test]
    fn test_delete_bookmark() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        // First insert a bookmark
        tree.insert_bookmark("to_delete", &selection).unwrap();
        assert!(tree.has_bookmark("to_delete"));

        // Now delete it
        let cmd = DeleteBookmark::by_name("to_delete");
        let result = cmd.apply(&tree, &selection).unwrap();

        assert!(!result.tree.has_bookmark("to_delete"));
    }

    #[test]
    fn test_rename_bookmark() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        // Insert a bookmark
        tree.insert_bookmark("old_name", &selection).unwrap();

        // Rename it
        let cmd = RenameBookmark::new("old_name", "new_name");
        let result = cmd.apply(&tree, &selection).unwrap();

        assert!(!result.tree.has_bookmark("old_name"));
        assert!(result.tree.has_bookmark("new_name"));
    }

    #[test]
    fn test_go_to_bookmark() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a bookmark at position 7
        let bookmark_pos = Position::new(para_id, 7);
        tree.insert_point_bookmark("target", bookmark_pos).unwrap();

        // Current selection at position 0
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Go to the bookmark
        let cmd = GoToBookmark::new("target");
        let result = cmd.apply(&tree, &selection).unwrap();

        // Selection should now be at the bookmark position
        assert_eq!(result.selection.focus.offset, 7);
        assert!(result.selection.is_collapsed());
    }

    #[test]
    fn test_duplicate_bookmark_name_fails() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        // Insert first bookmark
        let cmd1 = InsertBookmark::new("duplicate");
        let result = cmd1.apply(&tree, &selection).unwrap();

        // Try to insert another with same name
        let cmd2 = InsertBookmark::new("duplicate");
        let result2 = cmd2.apply(&result.tree, &selection);

        assert!(result2.is_err());
    }

    #[test]
    fn test_invalid_bookmark_name() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 5));

        // Name starts with number
        let cmd = InsertBookmark::new("123invalid");
        let result = cmd.apply(&tree, &selection);
        assert!(result.is_err());

        // Empty name
        let cmd = InsertBookmark::new("");
        let result = cmd.apply(&tree, &selection);
        assert!(result.is_err());

        // Name with spaces
        let cmd = InsertBookmark::new("has spaces");
        let result = cmd.apply(&tree, &selection);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_bookmarks() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        tree.insert_bookmark("alpha", &selection).unwrap();
        tree.insert_bookmark("beta", &Selection::collapsed(Position::new(para_id, 5))).unwrap();
        tree.insert_bookmark("gamma", &Selection::collapsed(Position::new(para_id, 10))).unwrap();

        let bookmarks = list_bookmarks(&tree);
        assert_eq!(bookmarks.len(), 3);

        // Check that all bookmarks are present
        let names: Vec<&str> = bookmarks.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert!(names.contains(&"gamma"));
    }
}
