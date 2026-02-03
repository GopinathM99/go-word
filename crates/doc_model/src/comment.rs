//! Comment model - annotations and discussions on document text
//!
//! Comments provide a way to annotate text ranges in the document for
//! review, discussion, and collaboration. Comments support:
//! - Anchoring to text ranges (start and end positions)
//! - Threaded replies
//! - Resolution status tracking
//! - Author and timestamp metadata

use crate::{NodeId, Position};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a comment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentId(Uuid);

impl CommentId {
    /// Create a new random CommentId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a CommentId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CommentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CommentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for CommentId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<CommentId> for Uuid {
    fn from(id: CommentId) -> Self {
        id.0
    }
}

/// Unique identifier for a comment reply
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReplyId(Uuid);

impl ReplyId {
    /// Create a new random ReplyId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a ReplyId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ReplyId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ReplyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ReplyId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ReplyId> for Uuid {
    fn from(id: ReplyId) -> Self {
        id.0
    }
}

/// Anchor point for a comment in the document
///
/// Comments anchor to text ranges. When text is edited, anchors need to
/// be updated to track with the text they're attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentAnchor {
    /// Start position of the commented text
    pub start: Position,
    /// End position of the commented text
    pub end: Position,
}

impl CommentAnchor {
    /// Create a new comment anchor from start and end positions
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create an anchor from a selection (normalizing direction)
    pub fn from_positions(pos1: Position, pos2: Position) -> Self {
        // Normalize so start is before end
        if pos1.node_id == pos2.node_id {
            if pos1.offset <= pos2.offset {
                Self { start: pos1, end: pos2 }
            } else {
                Self { start: pos2, end: pos1 }
            }
        } else {
            // Cross-node: use as-is (proper ordering requires document context)
            Self { start: pos1, end: pos2 }
        }
    }

    /// Check if this anchor is a point (zero-length range)
    pub fn is_point(&self) -> bool {
        self.start == self.end
    }

    /// Check if this anchor contains a position
    pub fn contains(&self, position: &Position) -> bool {
        if self.start.node_id == self.end.node_id && position.node_id == self.start.node_id {
            // Same node - simple offset comparison
            position.offset >= self.start.offset && position.offset <= self.end.offset
        } else if position.node_id == self.start.node_id {
            position.offset >= self.start.offset
        } else if position.node_id == self.end.node_id {
            position.offset <= self.end.offset
        } else {
            // Position in a different node - would need document context
            false
        }
    }

    /// Check if this anchor overlaps with a range
    pub fn overlaps_range(&self, start: &Position, end: &Position) -> bool {
        // Simple case: same node for all positions
        if self.start.node_id == self.end.node_id
            && start.node_id == end.node_id
            && self.start.node_id == start.node_id
        {
            // Check for overlap
            !(self.end.offset < start.offset || self.start.offset > end.offset)
        } else {
            // Cross-node ranges - simplified check
            self.contains(start) || self.contains(end)
        }
    }

    /// Adjust anchor positions after text is inserted
    pub fn adjust_for_insert(&mut self, insert_pos: &Position, insert_len: usize) {
        // Adjust start position
        if self.start.node_id == insert_pos.node_id && self.start.offset >= insert_pos.offset {
            self.start.offset += insert_len;
        }
        // Adjust end position
        if self.end.node_id == insert_pos.node_id && self.end.offset >= insert_pos.offset {
            self.end.offset += insert_len;
        }
    }

    /// Adjust anchor positions after text is deleted
    /// Returns true if the anchor is still valid, false if it should be orphaned
    pub fn adjust_for_delete(&mut self, delete_start: &Position, delete_end: &Position) -> bool {
        // Same node deletion
        if self.start.node_id == delete_start.node_id
            && delete_start.node_id == delete_end.node_id
        {
            let delete_len = delete_end.offset.saturating_sub(delete_start.offset);

            // Adjust start position
            if self.start.offset >= delete_end.offset {
                // Start is after deletion - shift back
                self.start.offset -= delete_len;
            } else if self.start.offset > delete_start.offset {
                // Start is within deletion - move to delete start
                self.start.offset = delete_start.offset;
            }

            // Adjust end position
            if self.end.offset >= delete_end.offset {
                // End is after deletion - shift back
                self.end.offset -= delete_len;
            } else if self.end.offset > delete_start.offset {
                // End is within deletion - move to delete start
                self.end.offset = delete_start.offset;
            }

            // Check if anchor collapsed to a point within the deleted range
            if self.start == self.end && self.start.offset == delete_start.offset {
                // The entire anchored text was deleted
                return false;
            }
        }

        true
    }
}

/// A reply to a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReply {
    /// Unique identifier for this reply
    id: ReplyId,
    /// Author of the reply
    author: String,
    /// When the reply was created
    date: DateTime<Utc>,
    /// Content of the reply
    content: String,
}

impl CommentReply {
    /// Create a new comment reply
    pub fn new(author: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: ReplyId::new(),
            author: author.into(),
            date: Utc::now(),
            content: content.into(),
        }
    }

    /// Create a reply with a specific ID and date (for deserialization/undo)
    pub fn with_id_and_date(
        id: ReplyId,
        author: impl Into<String>,
        date: DateTime<Utc>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id,
            author: author.into(),
            date,
            content: content.into(),
        }
    }

    /// Get the reply ID
    pub fn id(&self) -> ReplyId {
        self.id
    }

    /// Get the author
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Get the date
    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }

    /// Get the content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }
}

/// A comment on document text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique identifier for this comment
    id: CommentId,
    /// The text range this comment is attached to
    anchor: CommentAnchor,
    /// Author of the comment
    author: String,
    /// When the comment was created
    date: DateTime<Utc>,
    /// Content of the comment
    content: String,
    /// Replies to this comment
    replies: Vec<CommentReply>,
    /// Whether the comment has been resolved
    resolved: bool,
    /// Who resolved the comment (if resolved)
    resolved_by: Option<String>,
    /// When the comment was resolved (if resolved)
    resolved_date: Option<DateTime<Utc>>,
}

impl Comment {
    /// Create a new comment
    pub fn new(
        anchor: CommentAnchor,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: CommentId::new(),
            anchor,
            author: author.into(),
            date: Utc::now(),
            content: content.into(),
            replies: Vec::new(),
            resolved: false,
            resolved_by: None,
            resolved_date: None,
        }
    }

    /// Create a comment from selection positions
    pub fn from_selection(
        start: Position,
        end: Position,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let anchor = CommentAnchor::from_positions(start, end);
        Self::new(anchor, author, content)
    }

    /// Create a comment with a specific ID (for deserialization/undo)
    pub fn with_id(
        id: CommentId,
        anchor: CommentAnchor,
        author: impl Into<String>,
        date: DateTime<Utc>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id,
            anchor,
            author: author.into(),
            date,
            content: content.into(),
            replies: Vec::new(),
            resolved: false,
            resolved_by: None,
            resolved_date: None,
        }
    }

    /// Get the comment ID
    pub fn id(&self) -> CommentId {
        self.id
    }

    /// Get the anchor
    pub fn anchor(&self) -> &CommentAnchor {
        &self.anchor
    }

    /// Get a mutable reference to the anchor (for position adjustments)
    pub fn anchor_mut(&mut self) -> &mut CommentAnchor {
        &mut self.anchor
    }

    /// Get the author
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Get the date
    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }

    /// Get the content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Get the replies
    pub fn replies(&self) -> &[CommentReply] {
        &self.replies
    }

    /// Get mutable access to replies
    pub fn replies_mut(&mut self) -> &mut Vec<CommentReply> {
        &mut self.replies
    }

    /// Add a reply to this comment
    pub fn add_reply(&mut self, reply: CommentReply) {
        self.replies.push(reply);
    }

    /// Remove a reply by ID
    pub fn remove_reply(&mut self, reply_id: ReplyId) -> Option<CommentReply> {
        if let Some(pos) = self.replies.iter().position(|r| r.id == reply_id) {
            Some(self.replies.remove(pos))
        } else {
            None
        }
    }

    /// Get a reply by ID
    pub fn get_reply(&self, reply_id: ReplyId) -> Option<&CommentReply> {
        self.replies.iter().find(|r| r.id == reply_id)
    }

    /// Get a mutable reply by ID
    pub fn get_reply_mut(&mut self, reply_id: ReplyId) -> Option<&mut CommentReply> {
        self.replies.iter_mut().find(|r| r.id == reply_id)
    }

    /// Check if the comment is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    /// Get who resolved the comment
    pub fn resolved_by(&self) -> Option<&str> {
        self.resolved_by.as_deref()
    }

    /// Get when the comment was resolved
    pub fn resolved_date(&self) -> Option<DateTime<Utc>> {
        self.resolved_date
    }

    /// Resolve the comment
    pub fn resolve(&mut self, resolved_by: impl Into<String>) {
        self.resolved = true;
        self.resolved_by = Some(resolved_by.into());
        self.resolved_date = Some(Utc::now());
    }

    /// Reopen the comment (unresolve it)
    pub fn reopen(&mut self) {
        self.resolved = false;
        self.resolved_by = None;
        self.resolved_date = None;
    }

    /// Check if this comment's anchor contains a position
    pub fn contains_position(&self, position: &Position) -> bool {
        self.anchor.contains(position)
    }

    /// Check if this comment's anchor overlaps with a range
    pub fn overlaps_range(&self, start: &Position, end: &Position) -> bool {
        self.anchor.overlaps_range(start, end)
    }
}

/// Store for managing comments within a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentStore {
    /// Comments indexed by ID
    comments: HashMap<CommentId, Comment>,
}

impl CommentStore {
    /// Create a new empty comment store
    pub fn new() -> Self {
        Self {
            comments: HashMap::new(),
        }
    }

    /// Insert a comment into the store
    pub fn insert(&mut self, comment: Comment) -> CommentId {
        let id = comment.id();
        self.comments.insert(id, comment);
        id
    }

    /// Remove a comment by ID
    pub fn remove(&mut self, id: CommentId) -> Option<Comment> {
        self.comments.remove(&id)
    }

    /// Get a comment by ID
    pub fn get(&self, id: CommentId) -> Option<&Comment> {
        self.comments.get(&id)
    }

    /// Get a mutable comment by ID
    pub fn get_mut(&mut self, id: CommentId) -> Option<&mut Comment> {
        self.comments.get_mut(&id)
    }

    /// Check if a comment exists
    pub fn contains(&self, id: CommentId) -> bool {
        self.comments.contains_key(&id)
    }

    /// Get all comments
    pub fn all(&self) -> impl Iterator<Item = &Comment> {
        self.comments.values()
    }

    /// Get all comments as mutable
    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut Comment> {
        self.comments.values_mut()
    }

    /// Get the number of comments
    pub fn len(&self) -> usize {
        self.comments.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    /// Clear all comments
    pub fn clear(&mut self) {
        self.comments.clear();
    }

    /// Find comments at or containing a position
    pub fn find_at_position(&self, position: &Position) -> Vec<&Comment> {
        self.comments
            .values()
            .filter(|c| c.contains_position(position))
            .collect()
    }

    /// Find comments overlapping with a range
    pub fn find_in_range(&self, start: &Position, end: &Position) -> Vec<&Comment> {
        self.comments
            .values()
            .filter(|c| c.overlaps_range(start, end))
            .collect()
    }

    /// Find comments by node ID (comments that have anchors in the node)
    pub fn find_in_node(&self, node_id: NodeId) -> Vec<&Comment> {
        self.comments
            .values()
            .filter(|c| {
                c.anchor.start.node_id == node_id || c.anchor.end.node_id == node_id
            })
            .collect()
    }

    /// Filter comments by author
    pub fn filter_by_author(&self, author: &str) -> Vec<&Comment> {
        self.comments
            .values()
            .filter(|c| c.author == author)
            .collect()
    }

    /// Filter comments by resolved status
    pub fn filter_by_resolved(&self, resolved: bool) -> Vec<&Comment> {
        self.comments
            .values()
            .filter(|c| c.resolved == resolved)
            .collect()
    }

    /// Get unresolved comments
    pub fn unresolved(&self) -> Vec<&Comment> {
        self.filter_by_resolved(false)
    }

    /// Get resolved comments
    pub fn resolved(&self) -> Vec<&Comment> {
        self.filter_by_resolved(true)
    }

    /// Get comments sorted by date (oldest first)
    pub fn sorted_by_date(&self) -> Vec<&Comment> {
        let mut comments: Vec<&Comment> = self.comments.values().collect();
        comments.sort_by_key(|c| c.date);
        comments
    }

    /// Get comments sorted by position in document
    /// (Comments in earlier nodes/offsets come first)
    pub fn sorted_by_position(&self) -> Vec<&Comment> {
        let mut comments: Vec<&Comment> = self.comments.values().collect();
        comments.sort_by(|a, b| {
            // First compare by start node (as string for consistent ordering)
            let node_cmp = a.anchor.start.node_id.to_string()
                .cmp(&b.anchor.start.node_id.to_string());
            if node_cmp != std::cmp::Ordering::Equal {
                return node_cmp;
            }
            // Then by start offset
            a.anchor.start.offset.cmp(&b.anchor.start.offset)
        });
        comments
    }

    /// Adjust all comment anchors after text is inserted
    pub fn adjust_for_insert(&mut self, insert_pos: &Position, insert_len: usize) {
        for comment in self.comments.values_mut() {
            comment.anchor_mut().adjust_for_insert(insert_pos, insert_len);
        }
    }

    /// Adjust all comment anchors after text is deleted
    /// Returns IDs of comments that became orphaned (should be handled by caller)
    pub fn adjust_for_delete(&mut self, delete_start: &Position, delete_end: &Position) -> Vec<CommentId> {
        let mut orphaned = Vec::new();

        for comment in self.comments.values_mut() {
            if !comment.anchor_mut().adjust_for_delete(delete_start, delete_end) {
                orphaned.push(comment.id());
            }
        }

        orphaned
    }

    /// Mark comments as orphaned by moving them to a special state
    /// (This preserves the comments but marks them as detached from content)
    pub fn mark_orphaned(&mut self, comment_ids: &[CommentId]) {
        for id in comment_ids {
            if let Some(comment) = self.comments.get_mut(id) {
                // Mark as orphaned by setting anchor to an invalid state
                // The anchor positions become meaningless but the comment is preserved
                // In practice, you might want to handle this in the UI
                let _ = comment; // Placeholder - could add an `orphaned: bool` field
            }
        }
    }
}

/// Validation error for comments
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentValidationError {
    /// Content is empty
    EmptyContent,
    /// Author is empty
    EmptyAuthor,
    /// Comment not found
    NotFound,
    /// Reply not found
    ReplyNotFound,
}

impl std::fmt::Display for CommentValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentValidationError::EmptyContent => write!(f, "Comment content cannot be empty"),
            CommentValidationError::EmptyAuthor => write!(f, "Comment author cannot be empty"),
            CommentValidationError::NotFound => write!(f, "Comment not found"),
            CommentValidationError::ReplyNotFound => write!(f, "Reply not found"),
        }
    }
}

impl std::error::Error for CommentValidationError {}

/// Validate comment content
pub fn validate_comment_content(content: &str) -> Result<(), CommentValidationError> {
    if content.trim().is_empty() {
        return Err(CommentValidationError::EmptyContent);
    }
    Ok(())
}

/// Validate comment author
pub fn validate_comment_author(author: &str) -> Result<(), CommentValidationError> {
    if author.trim().is_empty() {
        return Err(CommentValidationError::EmptyAuthor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_position(node_id: NodeId, offset: usize) -> Position {
        Position::new(node_id, offset)
    }

    #[test]
    fn test_comment_creation() {
        let node_id = NodeId::new();
        let start = make_position(node_id, 5);
        let end = make_position(node_id, 15);

        let comment = Comment::from_selection(start, end, "Alice", "This is a comment");

        assert_eq!(comment.author(), "Alice");
        assert_eq!(comment.content(), "This is a comment");
        assert!(!comment.is_resolved());
        assert!(comment.replies().is_empty());
    }

    #[test]
    fn test_comment_anchor_normalization() {
        let node_id = NodeId::new();
        let pos1 = make_position(node_id, 15);
        let pos2 = make_position(node_id, 5);

        let anchor = CommentAnchor::from_positions(pos1, pos2);

        // Should be normalized so start < end
        assert_eq!(anchor.start.offset, 5);
        assert_eq!(anchor.end.offset, 15);
    }

    #[test]
    fn test_comment_anchor_contains() {
        let node_id = NodeId::new();
        let anchor = CommentAnchor::new(
            make_position(node_id, 5),
            make_position(node_id, 15),
        );

        assert!(anchor.contains(&make_position(node_id, 5)));
        assert!(anchor.contains(&make_position(node_id, 10)));
        assert!(anchor.contains(&make_position(node_id, 15)));
        assert!(!anchor.contains(&make_position(node_id, 4)));
        assert!(!anchor.contains(&make_position(node_id, 16)));
    }

    #[test]
    fn test_comment_reply() {
        let node_id = NodeId::new();
        let start = make_position(node_id, 5);
        let end = make_position(node_id, 15);

        let mut comment = Comment::from_selection(start, end, "Alice", "Original comment");

        let reply = CommentReply::new("Bob", "This is a reply");
        let reply_id = reply.id();
        comment.add_reply(reply);

        assert_eq!(comment.replies().len(), 1);
        assert_eq!(comment.replies()[0].author(), "Bob");
        assert_eq!(comment.replies()[0].content(), "This is a reply");

        // Get reply
        let reply = comment.get_reply(reply_id);
        assert!(reply.is_some());

        // Remove reply
        let removed = comment.remove_reply(reply_id);
        assert!(removed.is_some());
        assert!(comment.replies().is_empty());
    }

    #[test]
    fn test_comment_resolve() {
        let node_id = NodeId::new();
        let start = make_position(node_id, 5);
        let end = make_position(node_id, 15);

        let mut comment = Comment::from_selection(start, end, "Alice", "Comment");

        assert!(!comment.is_resolved());

        comment.resolve("Bob");

        assert!(comment.is_resolved());
        assert_eq!(comment.resolved_by(), Some("Bob"));
        assert!(comment.resolved_date().is_some());

        comment.reopen();

        assert!(!comment.is_resolved());
        assert!(comment.resolved_by().is_none());
        assert!(comment.resolved_date().is_none());
    }

    #[test]
    fn test_comment_store() {
        let mut store = CommentStore::new();
        let node_id = NodeId::new();

        let comment1 = Comment::from_selection(
            make_position(node_id, 0),
            make_position(node_id, 10),
            "Alice",
            "First comment",
        );
        let id1 = store.insert(comment1);

        let comment2 = Comment::from_selection(
            make_position(node_id, 20),
            make_position(node_id, 30),
            "Bob",
            "Second comment",
        );
        let id2 = store.insert(comment2);

        assert_eq!(store.len(), 2);
        assert!(store.get(id1).is_some());
        assert!(store.get(id2).is_some());

        // Find at position
        let at_pos = store.find_at_position(&make_position(node_id, 5));
        assert_eq!(at_pos.len(), 1);
        assert_eq!(at_pos[0].content(), "First comment");

        // Filter by author
        let by_alice = store.filter_by_author("Alice");
        assert_eq!(by_alice.len(), 1);

        // Remove
        let removed = store.remove(id1);
        assert!(removed.is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_anchor_adjust_for_insert() {
        let node_id = NodeId::new();
        let mut anchor = CommentAnchor::new(
            make_position(node_id, 10),
            make_position(node_id, 20),
        );

        // Insert before anchor
        anchor.adjust_for_insert(&make_position(node_id, 5), 3);
        assert_eq!(anchor.start.offset, 13);
        assert_eq!(anchor.end.offset, 23);

        // Insert within anchor
        anchor.adjust_for_insert(&make_position(node_id, 15), 2);
        assert_eq!(anchor.start.offset, 13);
        assert_eq!(anchor.end.offset, 25);
    }

    #[test]
    fn test_anchor_adjust_for_delete() {
        let node_id = NodeId::new();
        let mut anchor = CommentAnchor::new(
            make_position(node_id, 10),
            make_position(node_id, 20),
        );

        // Delete before anchor
        let valid = anchor.adjust_for_delete(
            &make_position(node_id, 0),
            &make_position(node_id, 5),
        );
        assert!(valid);
        assert_eq!(anchor.start.offset, 5);
        assert_eq!(anchor.end.offset, 15);

        // Delete entire anchor range
        let mut anchor2 = CommentAnchor::new(
            make_position(node_id, 10),
            make_position(node_id, 20),
        );
        let valid = anchor2.adjust_for_delete(
            &make_position(node_id, 10),
            &make_position(node_id, 20),
        );
        assert!(!valid); // Should become orphaned
    }

    #[test]
    fn test_comment_store_adjust_for_edits() {
        let mut store = CommentStore::new();
        let node_id = NodeId::new();

        let comment = Comment::from_selection(
            make_position(node_id, 10),
            make_position(node_id, 20),
            "Alice",
            "Comment",
        );
        let id = store.insert(comment);

        // Insert text before comment
        store.adjust_for_insert(&make_position(node_id, 5), 5);

        let comment = store.get(id).unwrap();
        assert_eq!(comment.anchor().start.offset, 15);
        assert_eq!(comment.anchor().end.offset, 25);
    }

    #[test]
    fn test_filter_by_resolved() {
        let mut store = CommentStore::new();
        let node_id = NodeId::new();

        let comment1 = Comment::from_selection(
            make_position(node_id, 0),
            make_position(node_id, 10),
            "Alice",
            "Unresolved",
        );

        let mut comment2 = Comment::from_selection(
            make_position(node_id, 20),
            make_position(node_id, 30),
            "Bob",
            "Resolved",
        );
        comment2.resolve("Charlie");

        store.insert(comment1);
        store.insert(comment2);

        assert_eq!(store.unresolved().len(), 1);
        assert_eq!(store.resolved().len(), 1);
    }
}
