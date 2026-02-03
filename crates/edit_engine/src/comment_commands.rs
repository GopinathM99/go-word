//! Comment commands for creating, editing, and managing comments
//!
//! Comments provide a way to annotate document text for review and discussion.
//! These commands support:
//! - Adding comments to text selections
//! - Editing comment content
//! - Replying to comments (threaded discussions)
//! - Resolving and reopening comments
//! - Deleting comments and replies
//! - Navigating to comments

use crate::{Command, CommandResult, EditError, Result};
use chrono::{DateTime, Utc};
use doc_model::{
    Comment, CommentAnchor, CommentId, CommentReply, DocumentTree, Node, NodeId, Position, ReplyId,
    Selection,
};
use serde::{Deserialize, Serialize};

// =============================================================================
// Add Comment Command
// =============================================================================

/// Add a comment to the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddComment {
    /// Author of the comment
    pub author: String,
    /// Content of the comment
    pub content: String,
}

impl AddComment {
    /// Create a new add comment command
    pub fn new(author: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            author: author.into(),
            content: content.into(),
        }
    }
}

impl Command for AddComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Add the comment at the selection
        let comment_id = new_tree
            .add_comment_at_selection(selection, &self.author, &self.content)
            .map_err(|e| EditError::InvalidCommand(format!("Invalid comment: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(DeleteComment { comment_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection, // Selection stays the same
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // We don't know the comment ID yet, so return a placeholder
        // The actual inverse is set in apply()
        Box::new(DeleteComment {
            comment_id: CommentId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Add Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Delete Comment Command
// =============================================================================

/// Delete a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteComment {
    /// ID of the comment to delete
    pub comment_id: CommentId,
}

impl DeleteComment {
    /// Create a new delete comment command
    pub fn new(comment_id: CommentId) -> Self {
        Self { comment_id }
    }
}

impl Command for DeleteComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the comment data before removing (for undo)
        let comment = new_tree
            .get_comment(self.comment_id)
            .ok_or_else(|| EditError::InvalidCommand("Comment not found".to_string()))?
            .clone();

        // Remove the comment
        new_tree.remove_comment(self.comment_id);

        // Create the inverse command (restore the comment)
        let inverse = Box::new(RestoreComment {
            comment_data: CommentData::from_comment(&comment),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(comment) = tree.get_comment(self.comment_id) {
            Box::new(RestoreComment {
                comment_data: CommentData::from_comment(comment),
            })
        } else {
            // Fallback - shouldn't happen in normal operation
            Box::new(AddComment::new("", ""))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Edit Comment Command
// =============================================================================

/// Edit a comment's content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditComment {
    /// ID of the comment to edit
    pub comment_id: CommentId,
    /// New content for the comment
    pub new_content: String,
}

impl EditComment {
    /// Create a new edit comment command
    pub fn new(comment_id: CommentId, new_content: impl Into<String>) -> Self {
        Self {
            comment_id,
            new_content: new_content.into(),
        }
    }
}

impl Command for EditComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the old content for undo
        let old_content = new_tree
            .get_comment(self.comment_id)
            .ok_or_else(|| EditError::InvalidCommand("Comment not found".to_string()))?
            .content()
            .to_string();

        // Update the comment content
        new_tree
            .edit_comment(self.comment_id, &self.new_content)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot edit comment: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(EditComment {
            comment_id: self.comment_id,
            new_content: old_content,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let old_content = tree
            .get_comment(self.comment_id)
            .map(|c| c.content().to_string())
            .unwrap_or_default();

        Box::new(EditComment {
            comment_id: self.comment_id,
            new_content: old_content,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Edit Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Reply to Comment Command
// =============================================================================

/// Add a reply to a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyToComment {
    /// ID of the comment to reply to
    pub comment_id: CommentId,
    /// Author of the reply
    pub author: String,
    /// Content of the reply
    pub content: String,
}

impl ReplyToComment {
    /// Create a new reply to comment command
    pub fn new(
        comment_id: CommentId,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            comment_id,
            author: author.into(),
            content: content.into(),
        }
    }
}

impl Command for ReplyToComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Add the reply
        let reply_id = new_tree
            .add_reply(self.comment_id, &self.author, &self.content)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot add reply: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(DeleteReply {
            comment_id: self.comment_id,
            reply_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // We don't know the reply ID yet
        Box::new(DeleteReply {
            comment_id: self.comment_id,
            reply_id: ReplyId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Reply to Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Delete Reply Command
// =============================================================================

/// Delete a reply from a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteReply {
    /// ID of the comment containing the reply
    pub comment_id: CommentId,
    /// ID of the reply to delete
    pub reply_id: ReplyId,
}

impl DeleteReply {
    /// Create a new delete reply command
    pub fn new(comment_id: CommentId, reply_id: ReplyId) -> Self {
        Self {
            comment_id,
            reply_id,
        }
    }
}

impl Command for DeleteReply {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the reply data before removing (for undo)
        let reply = new_tree
            .get_comment(self.comment_id)
            .and_then(|c| c.get_reply(self.reply_id))
            .ok_or_else(|| EditError::InvalidCommand("Reply not found".to_string()))?
            .clone();

        // Remove the reply
        new_tree
            .delete_reply(self.comment_id, self.reply_id)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot delete reply: {}", e)))?;

        // Create the inverse command (restore the reply)
        let inverse = Box::new(RestoreReply {
            comment_id: self.comment_id,
            reply_data: ReplyData::from_reply(&reply),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(comment) = tree.get_comment(self.comment_id) {
            if let Some(reply) = comment.get_reply(self.reply_id) {
                return Box::new(RestoreReply {
                    comment_id: self.comment_id,
                    reply_data: ReplyData::from_reply(reply),
                });
            }
        }
        // Fallback
        Box::new(ReplyToComment::new(self.comment_id, "", ""))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Reply"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Edit Reply Command
// =============================================================================

/// Edit a reply's content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditReply {
    /// ID of the comment containing the reply
    pub comment_id: CommentId,
    /// ID of the reply to edit
    pub reply_id: ReplyId,
    /// New content for the reply
    pub new_content: String,
}

impl EditReply {
    /// Create a new edit reply command
    pub fn new(
        comment_id: CommentId,
        reply_id: ReplyId,
        new_content: impl Into<String>,
    ) -> Self {
        Self {
            comment_id,
            reply_id,
            new_content: new_content.into(),
        }
    }
}

impl Command for EditReply {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the old content for undo
        let old_content = new_tree
            .get_comment(self.comment_id)
            .and_then(|c| c.get_reply(self.reply_id))
            .ok_or_else(|| EditError::InvalidCommand("Reply not found".to_string()))?
            .content()
            .to_string();

        // Update the reply content
        new_tree
            .edit_reply(self.comment_id, self.reply_id, &self.new_content)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot edit reply: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(EditReply {
            comment_id: self.comment_id,
            reply_id: self.reply_id,
            new_content: old_content,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let old_content = tree
            .get_comment(self.comment_id)
            .and_then(|c| c.get_reply(self.reply_id))
            .map(|r| r.content().to_string())
            .unwrap_or_default();

        Box::new(EditReply {
            comment_id: self.comment_id,
            reply_id: self.reply_id,
            new_content: old_content,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Edit Reply"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Resolve Comment Command
// =============================================================================

/// Resolve a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveComment {
    /// ID of the comment to resolve
    pub comment_id: CommentId,
    /// Who is resolving the comment
    pub resolved_by: String,
}

impl ResolveComment {
    /// Create a new resolve comment command
    pub fn new(comment_id: CommentId, resolved_by: impl Into<String>) -> Self {
        Self {
            comment_id,
            resolved_by: resolved_by.into(),
        }
    }
}

impl Command for ResolveComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Check if already resolved (for proper undo state)
        let was_resolved = new_tree
            .get_comment(self.comment_id)
            .ok_or_else(|| EditError::InvalidCommand("Comment not found".to_string()))?
            .is_resolved();

        if was_resolved {
            return Err(EditError::InvalidCommand(
                "Comment is already resolved".to_string(),
            ));
        }

        // Resolve the comment
        new_tree
            .resolve_comment(self.comment_id, &self.resolved_by)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot resolve comment: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(ReopenComment {
            comment_id: self.comment_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(ReopenComment {
            comment_id: self.comment_id,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Resolve Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Reopen Comment Command
// =============================================================================

/// Reopen a resolved comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReopenComment {
    /// ID of the comment to reopen
    pub comment_id: CommentId,
}

impl ReopenComment {
    /// Create a new reopen comment command
    pub fn new(comment_id: CommentId) -> Self {
        Self { comment_id }
    }
}

impl Command for ReopenComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get who resolved it (for undo)
        let comment = new_tree
            .get_comment(self.comment_id)
            .ok_or_else(|| EditError::InvalidCommand("Comment not found".to_string()))?;

        if !comment.is_resolved() {
            return Err(EditError::InvalidCommand(
                "Comment is not resolved".to_string(),
            ));
        }

        let resolved_by = comment.resolved_by().map(|s| s.to_string()).unwrap_or_default();

        // Reopen the comment
        new_tree
            .reopen_comment(self.comment_id)
            .map_err(|e| EditError::InvalidCommand(format!("Cannot reopen comment: {}", e)))?;

        // Create the inverse command
        let inverse = Box::new(ResolveComment {
            comment_id: self.comment_id,
            resolved_by,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        let resolved_by = tree
            .get_comment(self.comment_id)
            .and_then(|c| c.resolved_by())
            .map(|s| s.to_string())
            .unwrap_or_default();

        Box::new(ResolveComment {
            comment_id: self.comment_id,
            resolved_by,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Reopen Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Navigate to Comment Command
// =============================================================================

/// Navigate to a comment (updates selection to comment's anchor)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateToComment {
    /// ID of the comment to navigate to
    pub comment_id: CommentId,
}

impl NavigateToComment {
    /// Create a new navigate to comment command
    pub fn new(comment_id: CommentId) -> Self {
        Self { comment_id }
    }
}

impl Command for NavigateToComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Find the comment
        let comment = tree
            .get_comment(self.comment_id)
            .ok_or_else(|| EditError::InvalidCommand("Comment not found".to_string()))?;

        // Create selection at the comment's anchor
        let anchor = comment.anchor();
        let new_selection = Selection::new(anchor.start, anchor.end);

        // Create the inverse command (go back to original selection)
        let inverse = Box::new(SetSelectionCommand {
            selection: *selection,
        });

        Ok(CommandResult {
            tree: tree.clone(),
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NavigateToComment {
            comment_id: self.comment_id,
        })
    }

    fn transform_selection(&self, _selection: &Selection) -> Selection {
        Selection::default()
    }

    fn display_name(&self) -> &str {
        "Navigate to Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Internal Helper Commands
// =============================================================================

/// Internal command to restore a deleted comment (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreComment {
    comment_data: CommentData,
}

impl Command for RestoreComment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Recreate the comment
        let comment = self.comment_data.to_comment();
        let comment_id = comment.id();
        new_tree.comment_store_mut().insert(comment);

        // Create the inverse command
        let inverse = Box::new(DeleteComment { comment_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteComment {
            comment_id: self.comment_data.id,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Comment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Internal command to restore a deleted reply (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreReply {
    comment_id: CommentId,
    reply_data: ReplyData,
}

impl Command for RestoreReply {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the comment
        let comment = new_tree
            .get_comment_mut(self.comment_id)
            .ok_or_else(|| EditError::InvalidCommand("Comment not found".to_string()))?;

        // Recreate the reply
        let reply = self.reply_data.to_reply();
        let reply_id = reply.id();
        comment.add_reply(reply);

        // Create the inverse command
        let inverse = Box::new(DeleteReply {
            comment_id: self.comment_id,
            reply_id,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(DeleteReply {
            comment_id: self.comment_id,
            reply_id: self.reply_data.id,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Reply"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Internal command to set selection (for undo of navigation)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetSelectionCommand {
    selection: Selection,
}

impl Command for SetSelectionCommand {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: self.selection,
            inverse: Box::new(SetSelectionCommand {
                selection: self.selection,
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetSelectionCommand {
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

// =============================================================================
// Data Structures for Serialization
// =============================================================================

/// Serializable comment data for undo/redo
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommentData {
    id: CommentId,
    anchor_start: Position,
    anchor_end: Position,
    author: String,
    date: DateTime<Utc>,
    content: String,
    replies: Vec<ReplyData>,
    resolved: bool,
    resolved_by: Option<String>,
    resolved_date: Option<DateTime<Utc>>,
}

impl CommentData {
    fn from_comment(comment: &Comment) -> Self {
        Self {
            id: comment.id(),
            anchor_start: comment.anchor().start,
            anchor_end: comment.anchor().end,
            author: comment.author().to_string(),
            date: comment.date(),
            content: comment.content().to_string(),
            replies: comment.replies().iter().map(ReplyData::from_reply).collect(),
            resolved: comment.is_resolved(),
            resolved_by: comment.resolved_by().map(|s| s.to_string()),
            resolved_date: comment.resolved_date(),
        }
    }

    fn to_comment(&self) -> Comment {
        let anchor = CommentAnchor::new(self.anchor_start, self.anchor_end);
        let mut comment = Comment::with_id(
            self.id,
            anchor,
            &self.author,
            self.date,
            &self.content,
        );

        // Add replies
        for reply_data in &self.replies {
            comment.add_reply(reply_data.to_reply());
        }

        // Set resolved state
        if self.resolved {
            if let Some(ref resolved_by) = self.resolved_by {
                comment.resolve(resolved_by);
            }
        }

        comment
    }
}

/// Serializable reply data for undo/redo
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplyData {
    id: ReplyId,
    author: String,
    date: DateTime<Utc>,
    content: String,
}

impl ReplyData {
    fn from_reply(reply: &CommentReply) -> Self {
        Self {
            id: reply.id(),
            author: reply.author().to_string(),
            date: reply.date(),
            content: reply.content().to_string(),
        }
    }

    fn to_reply(&self) -> CommentReply {
        CommentReply::with_id_and_date(self.id, &self.author, self.date, &self.content)
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Information about a comment for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentInfo {
    /// The comment ID
    pub id: String,
    /// Author of the comment
    pub author: String,
    /// Date created (ISO 8601 format)
    pub date: String,
    /// Content of the comment
    pub content: String,
    /// Number of replies
    pub reply_count: usize,
    /// Whether the comment is resolved
    pub resolved: bool,
    /// Who resolved the comment (if resolved)
    pub resolved_by: Option<String>,
    /// When the comment was resolved (if resolved)
    pub resolved_date: Option<String>,
    /// Preview of the commented text (if available)
    pub text_preview: Option<String>,
    /// Start position (node ID as string)
    pub anchor_start_node: String,
    /// Start offset
    pub anchor_start_offset: usize,
    /// End position (node ID as string)
    pub anchor_end_node: String,
    /// End offset
    pub anchor_end_offset: usize,
}

impl CommentInfo {
    /// Create comment info from a comment
    pub fn from_comment(comment: &Comment, text_preview: Option<String>) -> Self {
        Self {
            id: comment.id().to_string(),
            author: comment.author().to_string(),
            date: comment.date().to_rfc3339(),
            content: comment.content().to_string(),
            reply_count: comment.replies().len(),
            resolved: comment.is_resolved(),
            resolved_by: comment.resolved_by().map(|s| s.to_string()),
            resolved_date: comment.resolved_date().map(|d| d.to_rfc3339()),
            text_preview,
            anchor_start_node: comment.anchor().start.node_id.to_string(),
            anchor_start_offset: comment.anchor().start.offset,
            anchor_end_node: comment.anchor().end.node_id.to_string(),
            anchor_end_offset: comment.anchor().end.offset,
        }
    }
}

/// Information about a reply for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyInfo {
    /// The reply ID
    pub id: String,
    /// Author of the reply
    pub author: String,
    /// Date created (ISO 8601 format)
    pub date: String,
    /// Content of the reply
    pub content: String,
}

impl ReplyInfo {
    /// Create reply info from a reply
    pub fn from_reply(reply: &CommentReply) -> Self {
        Self {
            id: reply.id().to_string(),
            author: reply.author().to_string(),
            date: reply.date().to_rfc3339(),
            content: reply.content().to_string(),
        }
    }
}

/// Get a list of all comments in the document
pub fn list_comments(tree: &DocumentTree) -> Vec<CommentInfo> {
    tree.all_comments()
        .map(|comment| {
            let preview = get_comment_text_preview(tree, comment);
            CommentInfo::from_comment(comment, preview)
        })
        .collect()
}

/// Get a list of comments sorted by position
pub fn list_comments_by_position(tree: &DocumentTree) -> Vec<CommentInfo> {
    tree.comments_sorted_by_position()
        .into_iter()
        .map(|comment| {
            let preview = get_comment_text_preview(tree, comment);
            CommentInfo::from_comment(comment, preview)
        })
        .collect()
}

/// Get a list of comments sorted by date
pub fn list_comments_by_date(tree: &DocumentTree) -> Vec<CommentInfo> {
    tree.comments_sorted_by_date()
        .into_iter()
        .map(|comment| {
            let preview = get_comment_text_preview(tree, comment);
            CommentInfo::from_comment(comment, preview)
        })
        .collect()
}

/// Get comments filtered by author
pub fn list_comments_by_author(tree: &DocumentTree, author: &str) -> Vec<CommentInfo> {
    tree.comments_by_author(author)
        .into_iter()
        .map(|comment| {
            let preview = get_comment_text_preview(tree, comment);
            CommentInfo::from_comment(comment, preview)
        })
        .collect()
}

/// Get unresolved comments
pub fn list_unresolved_comments(tree: &DocumentTree) -> Vec<CommentInfo> {
    tree.unresolved_comments()
        .into_iter()
        .map(|comment| {
            let preview = get_comment_text_preview(tree, comment);
            CommentInfo::from_comment(comment, preview)
        })
        .collect()
}

/// Get resolved comments
pub fn list_resolved_comments(tree: &DocumentTree) -> Vec<CommentInfo> {
    tree.resolved_comments()
        .into_iter()
        .map(|comment| {
            let preview = get_comment_text_preview(tree, comment);
            CommentInfo::from_comment(comment, preview)
        })
        .collect()
}

/// Get replies for a comment
pub fn get_comment_replies(tree: &DocumentTree, comment_id: CommentId) -> Vec<ReplyInfo> {
    tree.get_comment(comment_id)
        .map(|comment| {
            comment
                .replies()
                .iter()
                .map(ReplyInfo::from_reply)
                .collect()
        })
        .unwrap_or_default()
}

/// Get preview text for a comment (the text that the comment is attached to)
fn get_comment_text_preview(tree: &DocumentTree, comment: &Comment) -> Option<String> {
    let anchor = comment.anchor();
    let start = anchor.start;

    // Try to get text from the paragraph
    let para = tree.get_paragraph(start.node_id)?;

    let mut text = String::new();
    let mut current_offset = 0;

    for &run_id in para.children() {
        if let Some(run) = tree.get_run(run_id) {
            let run_len = run.text.chars().count();
            let run_end = current_offset + run_len;

            // Check if this run contains or comes after the comment start
            if run_end > start.offset {
                let start_in_run = if start.offset > current_offset {
                    start.offset - current_offset
                } else {
                    0
                };

                // Determine end offset
                let end_offset = if anchor.end.node_id == start.node_id {
                    anchor.end.offset
                } else {
                    // Comment spans multiple nodes, just get first ~30 chars
                    start.offset + 30
                };

                let end_in_run = if end_offset < run_end {
                    end_offset - current_offset
                } else {
                    run_len
                };

                // Get text from this run
                let chars: Vec<char> = run.text.chars().collect();
                for &c in &chars[start_in_run..end_in_run.min(chars.len())] {
                    text.push(c);
                    if text.len() >= 50 {
                        break;
                    }
                }

                if text.len() >= 50 {
                    break;
                }
            }

            current_offset = run_end;

            // If we've passed the anchor end, stop
            if anchor.end.node_id == start.node_id && current_offset >= anchor.end.offset {
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

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Run};

    fn create_test_tree() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();

        let run = Run::new("Hello, World! This is some sample text for testing comments.");
        tree.insert_run(run, para_id, None).unwrap();

        (tree, para_id)
    }

    #[test]
    fn test_add_comment() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        let cmd = AddComment::new("Alice", "This is a comment");
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.comment_count(), 1);
        let comments: Vec<_> = result.tree.all_comments().collect();
        assert_eq!(comments[0].author(), "Alice");
        assert_eq!(comments[0].content(), "This is a comment");
    }

    #[test]
    fn test_delete_comment() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        // First add a comment
        let comment_id = tree
            .add_comment_at_selection(&selection, "Alice", "Test comment")
            .unwrap();

        // Now delete it
        let cmd = DeleteComment::new(comment_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.comment_count(), 0);
    }

    #[test]
    fn test_edit_comment() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        // Add a comment
        let comment_id = tree
            .add_comment_at_selection(&selection, "Alice", "Original content")
            .unwrap();

        // Edit it
        let cmd = EditComment::new(comment_id, "Updated content");
        let result = cmd.apply(&tree, &selection).unwrap();

        let comment = result.tree.get_comment(comment_id).unwrap();
        assert_eq!(comment.content(), "Updated content");
    }

    #[test]
    fn test_reply_to_comment() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        // Add a comment
        let comment_id = tree
            .add_comment_at_selection(&selection, "Alice", "Original comment")
            .unwrap();

        // Reply to it
        let cmd = ReplyToComment::new(comment_id, "Bob", "This is a reply");
        let result = cmd.apply(&tree, &selection).unwrap();

        let comment = result.tree.get_comment(comment_id).unwrap();
        assert_eq!(comment.replies().len(), 1);
        assert_eq!(comment.replies()[0].author(), "Bob");
        assert_eq!(comment.replies()[0].content(), "This is a reply");
    }

    #[test]
    fn test_resolve_and_reopen_comment() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        // Add a comment
        let comment_id = tree
            .add_comment_at_selection(&selection, "Alice", "Comment")
            .unwrap();

        // Resolve it
        let cmd = ResolveComment::new(comment_id, "Bob");
        let result = cmd.apply(&tree, &selection).unwrap();

        let comment = result.tree.get_comment(comment_id).unwrap();
        assert!(comment.is_resolved());
        assert_eq!(comment.resolved_by(), Some("Bob"));

        // Reopen it
        let cmd = ReopenComment::new(comment_id);
        let result = cmd.apply(&result.tree, &selection).unwrap();

        let comment = result.tree.get_comment(comment_id).unwrap();
        assert!(!comment.is_resolved());
    }

    #[test]
    fn test_navigate_to_comment() {
        let (mut tree, para_id) = create_test_tree();

        // Add a comment at offset 10-20
        let comment_id = tree
            .add_comment(
                Position::new(para_id, 10),
                Position::new(para_id, 20),
                "Alice",
                "Comment",
            )
            .unwrap();

        // Current selection at start
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Navigate to comment
        let cmd = NavigateToComment::new(comment_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Selection should now be at comment's anchor
        assert_eq!(result.selection.anchor.offset, 10);
        assert_eq!(result.selection.focus.offset, 20);
    }

    #[test]
    fn test_list_comments() {
        let (mut tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        // Add multiple comments
        tree.add_comment(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
            "Alice",
            "First comment",
        )
        .unwrap();
        tree.add_comment(
            Position::new(para_id, 10),
            Position::new(para_id, 15),
            "Bob",
            "Second comment",
        )
        .unwrap();

        let comments = list_comments(&tree);
        assert_eq!(comments.len(), 2);

        // Check filtering
        let alice_comments = list_comments_by_author(&tree, "Alice");
        assert_eq!(alice_comments.len(), 1);
        assert_eq!(alice_comments[0].author, "Alice");
    }

    #[test]
    fn test_comment_undo_redo() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::new(
            Position::new(para_id, 0),
            Position::new(para_id, 5),
        );

        // Add a comment
        let cmd = AddComment::new("Alice", "Test comment");
        let result = cmd.apply(&tree, &selection).unwrap();
        assert_eq!(result.tree.comment_count(), 1);

        // Undo (apply inverse)
        let undo_result = result.inverse.apply(&result.tree, &result.selection).unwrap();
        assert_eq!(undo_result.tree.comment_count(), 0);

        // Redo (apply inverse of inverse)
        let redo_result = undo_result
            .inverse
            .apply(&undo_result.tree, &undo_result.selection)
            .unwrap();
        assert_eq!(redo_result.tree.comment_count(), 1);
    }
}
