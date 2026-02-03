//! Revision model - Core types for tracking document changes

use chrono::{DateTime, Utc};
use doc_model::{CharacterProperties, NodeId, ParagraphProperties, Position};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a revision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RevisionId(pub Uuid);

impl RevisionId {
    /// Create a new unique revision ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RevisionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RevisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A range within the document for revision tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionRange {
    /// The node containing this range
    pub node_id: NodeId,
    /// Start offset within the node (in characters)
    pub start_offset: usize,
    /// End offset within the node (in characters)
    pub end_offset: usize,
}

impl RevisionRange {
    /// Create a new revision range
    pub fn new(node_id: NodeId, start_offset: usize, end_offset: usize) -> Self {
        Self {
            node_id,
            start_offset,
            end_offset,
        }
    }

    /// Create a range from a position with length
    pub fn from_position(position: Position, length: usize) -> Self {
        Self {
            node_id: position.node_id,
            start_offset: position.offset,
            end_offset: position.offset + length,
        }
    }

    /// Create a collapsed range (insertion point)
    pub fn collapsed(node_id: NodeId, offset: usize) -> Self {
        Self {
            node_id,
            start_offset: offset,
            end_offset: offset,
        }
    }

    /// Check if this range is collapsed (zero-length)
    pub fn is_collapsed(&self) -> bool {
        self.start_offset == self.end_offset
    }

    /// Get the length of this range
    pub fn length(&self) -> usize {
        self.end_offset.saturating_sub(self.start_offset)
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &RevisionRange) -> bool {
        if self.node_id != other.node_id {
            return false;
        }
        self.start_offset < other.end_offset && other.start_offset < self.end_offset
    }

    /// Check if this range contains a position
    pub fn contains_position(&self, position: &Position) -> bool {
        self.node_id == position.node_id
            && position.offset >= self.start_offset
            && position.offset <= self.end_offset
    }

    /// Adjust offsets after an insertion
    pub fn adjust_for_insertion(&mut self, at: &Position, length: usize) {
        if self.node_id != at.node_id {
            return;
        }

        // If insertion is before or at start, shift both offsets
        if at.offset <= self.start_offset {
            self.start_offset += length;
            self.end_offset += length;
        }
        // If insertion is within range, expand end
        else if at.offset < self.end_offset {
            self.end_offset += length;
        }
        // If insertion is after range, no change needed
    }

    /// Adjust offsets after a deletion
    pub fn adjust_for_deletion(&mut self, range: &RevisionRange) {
        if self.node_id != range.node_id {
            return;
        }

        let delete_len = range.length();

        // If deletion is entirely before this range
        if range.end_offset <= self.start_offset {
            self.start_offset -= delete_len;
            self.end_offset -= delete_len;
        }
        // If deletion entirely contains this range, collapse it
        else if range.start_offset <= self.start_offset && range.end_offset >= self.end_offset {
            self.start_offset = range.start_offset;
            self.end_offset = range.start_offset;
        }
        // If deletion overlaps with start (but not end)
        else if range.start_offset <= self.start_offset && range.end_offset < self.end_offset {
            let overlap = range.end_offset - self.start_offset;
            let remaining_len = self.length() - overlap;
            self.start_offset = range.start_offset;
            self.end_offset = self.start_offset + remaining_len;
        }
        // If deletion is entirely within this range
        else if range.start_offset > self.start_offset && range.end_offset < self.end_offset {
            self.end_offset -= delete_len;
        }
        // If deletion overlaps with end (but not start)
        else if range.start_offset > self.start_offset && range.start_offset < self.end_offset {
            self.end_offset = range.start_offset;
        }
    }
}

/// Stored content for deletion revisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedContent {
    /// The text that was deleted
    pub text: String,
    /// Character formatting of the deleted content (if any)
    pub character_props: Option<CharacterProperties>,
    /// The run IDs that were affected
    pub affected_runs: Vec<NodeId>,
}

impl DeletedContent {
    /// Create new deleted content
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            character_props: None,
            affected_runs: Vec::new(),
        }
    }

    /// Create with formatting
    pub fn with_formatting(text: impl Into<String>, props: CharacterProperties) -> Self {
        Self {
            text: text.into(),
            character_props: Some(props),
            affected_runs: Vec::new(),
        }
    }
}

/// Format change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatChangeInfo {
    /// Old character properties
    pub old_character_props: Option<CharacterProperties>,
    /// New character properties
    pub new_character_props: Option<CharacterProperties>,
    /// Old paragraph properties
    pub old_paragraph_props: Option<ParagraphProperties>,
    /// New paragraph properties
    pub new_paragraph_props: Option<ParagraphProperties>,
}

impl FormatChangeInfo {
    /// Create a character formatting change
    pub fn character_change(
        old: Option<CharacterProperties>,
        new: Option<CharacterProperties>,
    ) -> Self {
        Self {
            old_character_props: old,
            new_character_props: new,
            old_paragraph_props: None,
            new_paragraph_props: None,
        }
    }

    /// Create a paragraph formatting change
    pub fn paragraph_change(
        old: Option<ParagraphProperties>,
        new: Option<ParagraphProperties>,
    ) -> Self {
        Self {
            old_character_props: None,
            new_character_props: None,
            old_paragraph_props: old,
            new_paragraph_props: new,
        }
    }
}

/// Move operation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveInfo {
    /// Original range (where content was moved from)
    pub from_range: RevisionRange,
    /// Destination range (where content was moved to)
    pub to_range: RevisionRange,
}

/// Type of revision (change)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevisionType {
    /// Text was inserted
    Insert {
        /// Range of inserted content
        range: RevisionRange,
    },
    /// Text was deleted (content is hidden, not removed)
    Delete {
        /// Range where deletion occurred
        range: RevisionRange,
        /// The deleted content (for potential restoration)
        deleted_content: DeletedContent,
    },
    /// Formatting was changed
    FormatChange {
        /// Range of formatted content
        range: RevisionRange,
        /// Format change details
        format_info: FormatChangeInfo,
    },
    /// Content was moved
    Move {
        /// Move operation details
        move_info: MoveInfo,
    },
}

impl RevisionType {
    /// Get a display name for the revision type
    pub fn display_name(&self) -> &'static str {
        match self {
            RevisionType::Insert { .. } => "Inserted",
            RevisionType::Delete { .. } => "Deleted",
            RevisionType::FormatChange { .. } => "Formatted",
            RevisionType::Move { .. } => "Moved",
        }
    }

    /// Get the primary range affected by this revision
    pub fn primary_range(&self) -> &RevisionRange {
        match self {
            RevisionType::Insert { range } => range,
            RevisionType::Delete { range, .. } => range,
            RevisionType::FormatChange { range, .. } => range,
            RevisionType::Move { move_info } => &move_info.to_range,
        }
    }
}

/// Current state of a revision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RevisionStatus {
    /// Revision is pending (neither accepted nor rejected)
    #[default]
    Pending,
    /// Revision has been accepted (change is permanent)
    Accepted,
    /// Revision has been rejected (change is reverted)
    Rejected,
}

/// A single revision (tracked change)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    /// Unique identifier
    pub id: RevisionId,
    /// Type of revision
    pub revision_type: RevisionType,
    /// Author who made this change
    pub author: String,
    /// When the change was made
    pub timestamp: DateTime<Utc>,
    /// Current status
    pub status: RevisionStatus,
    /// Optional comment/note about this revision
    pub comment: Option<String>,
    /// Linked revision ID (for move operations, links source and destination)
    pub linked_revision: Option<RevisionId>,
}

impl Revision {
    /// Create a new insertion revision
    pub fn insert(author: impl Into<String>, range: RevisionRange) -> Self {
        Self {
            id: RevisionId::new(),
            revision_type: RevisionType::Insert { range },
            author: author.into(),
            timestamp: Utc::now(),
            status: RevisionStatus::Pending,
            comment: None,
            linked_revision: None,
        }
    }

    /// Create a new deletion revision
    pub fn delete(
        author: impl Into<String>,
        range: RevisionRange,
        deleted_content: DeletedContent,
    ) -> Self {
        Self {
            id: RevisionId::new(),
            revision_type: RevisionType::Delete {
                range,
                deleted_content,
            },
            author: author.into(),
            timestamp: Utc::now(),
            status: RevisionStatus::Pending,
            comment: None,
            linked_revision: None,
        }
    }

    /// Create a new format change revision
    pub fn format_change(
        author: impl Into<String>,
        range: RevisionRange,
        format_info: FormatChangeInfo,
    ) -> Self {
        Self {
            id: RevisionId::new(),
            revision_type: RevisionType::FormatChange { range, format_info },
            author: author.into(),
            timestamp: Utc::now(),
            status: RevisionStatus::Pending,
            comment: None,
            linked_revision: None,
        }
    }

    /// Create a new move revision
    pub fn move_content(author: impl Into<String>, move_info: MoveInfo) -> Self {
        Self {
            id: RevisionId::new(),
            revision_type: RevisionType::Move { move_info },
            author: author.into(),
            timestamp: Utc::now(),
            status: RevisionStatus::Pending,
            comment: None,
            linked_revision: None,
        }
    }

    /// Check if this revision is pending
    pub fn is_pending(&self) -> bool {
        self.status == RevisionStatus::Pending
    }

    /// Check if this revision is accepted
    pub fn is_accepted(&self) -> bool {
        self.status == RevisionStatus::Accepted
    }

    /// Check if this revision is rejected
    pub fn is_rejected(&self) -> bool {
        self.status == RevisionStatus::Rejected
    }

    /// Get the primary range of this revision
    pub fn range(&self) -> &RevisionRange {
        self.revision_type.primary_range()
    }

    /// Add a comment to this revision
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Link this revision to another (for move operations)
    pub fn with_linked_revision(mut self, linked: RevisionId) -> Self {
        self.linked_revision = Some(linked);
        self
    }

    /// Set a specific timestamp (useful for testing)
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set a specific ID (useful for testing)
    pub fn with_id(mut self, id: RevisionId) -> Self {
        self.id = id;
        self
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revision_range_overlap() {
        let node_id = NodeId::new();
        let range1 = RevisionRange::new(node_id, 0, 10);
        let range2 = RevisionRange::new(node_id, 5, 15);
        let range3 = RevisionRange::new(node_id, 10, 20);
        let range4 = RevisionRange::new(NodeId::new(), 0, 10);

        assert!(range1.overlaps(&range2));
        assert!(!range1.overlaps(&range3)); // Adjacent, not overlapping
        assert!(!range1.overlaps(&range4)); // Different nodes
    }

    #[test]
    fn test_revision_range_adjust_for_insertion() {
        let node_id = NodeId::new();
        let mut range = RevisionRange::new(node_id, 10, 20);

        // Insert before range - shifts both offsets
        range.adjust_for_insertion(&Position::new(node_id, 5), 5);
        assert_eq!(range.start_offset, 15);
        assert_eq!(range.end_offset, 25);

        // Insert within range - expands end
        let mut range2 = RevisionRange::new(node_id, 10, 20);
        range2.adjust_for_insertion(&Position::new(node_id, 15), 5);
        assert_eq!(range2.start_offset, 10);
        assert_eq!(range2.end_offset, 25);

        // Insert after range - no change
        let mut range3 = RevisionRange::new(node_id, 10, 20);
        range3.adjust_for_insertion(&Position::new(node_id, 25), 5);
        assert_eq!(range3.start_offset, 10);
        assert_eq!(range3.end_offset, 20);
    }

    #[test]
    fn test_revision_range_adjust_for_deletion() {
        let node_id = NodeId::new();

        // Delete before range - shifts both offsets
        let mut range = RevisionRange::new(node_id, 10, 20);
        range.adjust_for_deletion(&RevisionRange::new(node_id, 0, 5));
        assert_eq!(range.start_offset, 5);
        assert_eq!(range.end_offset, 15);

        // Delete overlapping start
        let mut range2 = RevisionRange::new(node_id, 10, 20);
        range2.adjust_for_deletion(&RevisionRange::new(node_id, 5, 12));
        assert_eq!(range2.start_offset, 5);
        assert_eq!(range2.end_offset, 13);
    }

    #[test]
    fn test_revision_creation() {
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let revision = Revision::insert("TestAuthor", range.clone());

        assert_eq!(revision.author, "TestAuthor");
        assert!(revision.is_pending());
        assert!(!revision.is_accepted());
        assert!(!revision.is_rejected());
        matches!(revision.revision_type, RevisionType::Insert { .. });
    }

    #[test]
    fn test_revision_with_comment() {
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let revision = Revision::insert("Author", range).with_comment("Test comment");

        assert_eq!(revision.comment, Some("Test comment".to_string()));
    }

    #[test]
    fn test_deleted_content() {
        let content = DeletedContent::new("Hello, World!");
        assert_eq!(content.text, "Hello, World!");
        assert!(content.character_props.is_none());

        let props = CharacterProperties {
            bold: Some(true),
            ..Default::default()
        };
        let content2 = DeletedContent::with_formatting("Formatted text", props.clone());
        assert_eq!(content2.text, "Formatted text");
        assert_eq!(content2.character_props, Some(props));
    }
}
