//! Command integration for revision tracking
//!
//! This module provides helpers for integrating revision tracking with
//! the edit_engine command system.

use crate::{
    DeletedContent, FormatChangeInfo, MoveInfo, Result, RevisionError, RevisionId,
    RevisionRange, RevisionState,
};
use doc_model::{CharacterProperties, DocumentTree, Node, NodeId, ParagraphProperties, Position};
use serde::{Deserialize, Serialize};

/// A tracked text change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedInsert {
    /// Position where text was inserted
    pub position: Position,
    /// The inserted text
    pub text: String,
    /// Revision ID (if tracking is enabled)
    pub revision_id: Option<RevisionId>,
}

impl TrackedInsert {
    /// Create a new tracked insert
    pub fn new(position: Position, text: impl Into<String>) -> Self {
        Self {
            position,
            text: text.into(),
            revision_id: None,
        }
    }

    /// Record this insert in the revision state
    pub fn record(&mut self, state: &mut RevisionState) -> Result<()> {
        if !state.is_tracking() {
            return Ok(());
        }

        let char_count = self.text.chars().count();
        let range = RevisionRange::from_position(self.position, char_count);
        let id = state.record_insert(range)?;
        self.revision_id = Some(id);
        Ok(())
    }
}

/// A tracked deletion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedDelete {
    /// Start position of deletion
    pub start: Position,
    /// End position of deletion
    pub end: Position,
    /// The deleted content
    pub deleted_content: DeletedContent,
    /// Revision ID (if tracking is enabled)
    pub revision_id: Option<RevisionId>,
}

impl TrackedDelete {
    /// Create a new tracked delete
    pub fn new(start: Position, end: Position, deleted_text: impl Into<String>) -> Self {
        Self {
            start,
            end,
            deleted_content: DeletedContent::new(deleted_text),
            revision_id: None,
        }
    }

    /// Create with formatting information
    pub fn with_formatting(
        start: Position,
        end: Position,
        deleted_text: impl Into<String>,
        props: CharacterProperties,
    ) -> Self {
        Self {
            start,
            end,
            deleted_content: DeletedContent::with_formatting(deleted_text, props),
            revision_id: None,
        }
    }

    /// Record this delete in the revision state
    pub fn record(&mut self, state: &mut RevisionState) -> Result<()> {
        if !state.is_tracking() {
            return Ok(());
        }

        let range = RevisionRange::new(
            self.start.node_id,
            self.start.offset,
            self.end.offset,
        );
        let id = state.record_delete(range, self.deleted_content.clone())?;
        self.revision_id = Some(id);
        Ok(())
    }
}

/// A tracked format change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFormatChange {
    /// Start position of format change
    pub start: Position,
    /// End position of format change
    pub end: Position,
    /// Format change details
    pub format_info: FormatChangeInfo,
    /// Revision ID (if tracking is enabled)
    pub revision_id: Option<RevisionId>,
}

impl TrackedFormatChange {
    /// Create a new tracked character format change
    pub fn character_format(
        start: Position,
        end: Position,
        old_props: Option<CharacterProperties>,
        new_props: Option<CharacterProperties>,
    ) -> Self {
        Self {
            start,
            end,
            format_info: FormatChangeInfo::character_change(old_props, new_props),
            revision_id: None,
        }
    }

    /// Create a new tracked paragraph format change
    pub fn paragraph_format(
        start: Position,
        end: Position,
        old_props: Option<ParagraphProperties>,
        new_props: Option<ParagraphProperties>,
    ) -> Self {
        Self {
            start,
            end,
            format_info: FormatChangeInfo::paragraph_change(old_props, new_props),
            revision_id: None,
        }
    }

    /// Record this format change in the revision state
    pub fn record(&mut self, state: &mut RevisionState) -> Result<()> {
        if !state.is_tracking() {
            return Ok(());
        }

        let range = RevisionRange::new(
            self.start.node_id,
            self.start.offset,
            self.end.offset,
        );
        let id = state.record_format_change(range, self.format_info.clone())?;
        self.revision_id = Some(id);
        Ok(())
    }
}

/// A tracked move operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedMove {
    /// Source range (where content was moved from)
    pub from_start: Position,
    pub from_end: Position,
    /// Destination position (where content was moved to)
    pub to_start: Position,
    pub to_end: Position,
    /// Revision IDs (source and destination)
    pub revision_ids: Option<(RevisionId, RevisionId)>,
}

impl TrackedMove {
    /// Create a new tracked move
    pub fn new(
        from_start: Position,
        from_end: Position,
        to_start: Position,
        to_end: Position,
    ) -> Self {
        Self {
            from_start,
            from_end,
            to_start,
            to_end,
            revision_ids: None,
        }
    }

    /// Record this move in the revision state
    pub fn record(&mut self, state: &mut RevisionState) -> Result<()> {
        if !state.is_tracking() {
            return Ok(());
        }

        let move_info = MoveInfo {
            from_range: RevisionRange::new(
                self.from_start.node_id,
                self.from_start.offset,
                self.from_end.offset,
            ),
            to_range: RevisionRange::new(
                self.to_start.node_id,
                self.to_start.offset,
                self.to_end.offset,
            ),
        };
        let ids = state.record_move(move_info)?;
        self.revision_ids = Some(ids);
        Ok(())
    }
}

// =============================================================================
// Revision Application Helpers
// =============================================================================

/// Result of accepting a revision
#[derive(Debug, Clone)]
pub struct AcceptResult {
    /// The revision that was accepted
    pub revision_id: RevisionId,
    /// Whether any document modifications are needed
    pub needs_modification: bool,
    /// Detailed information about what changed
    pub details: AcceptDetails,
}

/// Details of what happens when accepting a revision
#[derive(Debug, Clone)]
pub enum AcceptDetails {
    /// Insert accepted - content is now permanent, no changes needed
    InsertAccepted,
    /// Delete accepted - hidden content should be removed from document
    DeleteAccepted { range: RevisionRange },
    /// Format change accepted - formatting is now permanent
    FormatChangeAccepted,
    /// Move accepted - both source removal and destination are permanent
    MoveAccepted,
}

/// Result of rejecting a revision
#[derive(Debug, Clone)]
pub struct RejectResult {
    /// The revision that was rejected
    pub revision_id: RevisionId,
    /// Whether any document modifications are needed
    pub needs_modification: bool,
    /// Detailed information about what changed
    pub details: RejectDetails,
}

/// Details of what happens when rejecting a revision
#[derive(Debug, Clone)]
pub enum RejectDetails {
    /// Insert rejected - inserted content should be removed
    InsertRejected { range: RevisionRange },
    /// Delete rejected - hidden content should be restored
    DeleteRejected {
        range: RevisionRange,
        content: DeletedContent,
    },
    /// Format change rejected - old formatting should be restored
    FormatChangeRejected {
        range: RevisionRange,
        old_character_props: Option<CharacterProperties>,
        old_paragraph_props: Option<ParagraphProperties>,
    },
    /// Move rejected - content should be moved back
    MoveRejected { from_range: RevisionRange, to_range: RevisionRange },
}

/// Process accepting a revision and determine document changes needed
pub fn process_accept(state: &mut RevisionState, id: RevisionId) -> Result<AcceptResult> {
    let revision = state
        .get(id)
        .ok_or(RevisionError::RevisionNotFound(id.as_uuid()))?
        .clone();

    if !revision.is_pending() {
        return Err(RevisionError::RevisionAlreadyProcessed(id.as_uuid()));
    }

    // Determine what document changes are needed
    let details = match &revision.revision_type {
        crate::RevisionType::Insert { .. } => {
            // Accepting insert: content stays, just remove tracking
            AcceptDetails::InsertAccepted
        }
        crate::RevisionType::Delete { range, .. } => {
            // Accepting delete: actually remove the hidden content
            AcceptDetails::DeleteAccepted { range: range.clone() }
        }
        crate::RevisionType::FormatChange { .. } => {
            // Accepting format change: formatting stays
            AcceptDetails::FormatChangeAccepted
        }
        crate::RevisionType::Move { .. } => {
            // Accepting move: content stays at destination
            AcceptDetails::MoveAccepted
        }
    };

    let needs_modification = matches!(details, AcceptDetails::DeleteAccepted { .. });

    // Mark as accepted in state
    state.accept_revision(id)?;

    Ok(AcceptResult {
        revision_id: id,
        needs_modification,
        details,
    })
}

/// Process rejecting a revision and determine document changes needed
pub fn process_reject(state: &mut RevisionState, id: RevisionId) -> Result<RejectResult> {
    let revision = state
        .get(id)
        .ok_or(RevisionError::RevisionNotFound(id.as_uuid()))?
        .clone();

    if !revision.is_pending() {
        return Err(RevisionError::RevisionAlreadyProcessed(id.as_uuid()));
    }

    // Determine what document changes are needed
    let (needs_modification, details) = match &revision.revision_type {
        crate::RevisionType::Insert { range } => {
            // Rejecting insert: remove the inserted content
            (
                true,
                RejectDetails::InsertRejected { range: range.clone() },
            )
        }
        crate::RevisionType::Delete { range, deleted_content } => {
            // Rejecting delete: restore the hidden content
            (
                true,
                RejectDetails::DeleteRejected {
                    range: range.clone(),
                    content: deleted_content.clone(),
                },
            )
        }
        crate::RevisionType::FormatChange { range, format_info } => {
            // Rejecting format change: restore old formatting
            (
                true,
                RejectDetails::FormatChangeRejected {
                    range: range.clone(),
                    old_character_props: format_info.old_character_props.clone(),
                    old_paragraph_props: format_info.old_paragraph_props.clone(),
                },
            )
        }
        crate::RevisionType::Move { move_info } => {
            // Rejecting move: move content back
            (
                true,
                RejectDetails::MoveRejected {
                    from_range: move_info.from_range.clone(),
                    to_range: move_info.to_range.clone(),
                },
            )
        }
    };

    // Mark as rejected in state
    state.reject_revision(id)?;

    Ok(RejectResult {
        revision_id: id,
        needs_modification,
        details,
    })
}

// =============================================================================
// Document Content Extraction
// =============================================================================

/// Extract text content from a document range (for creating deletion records)
pub fn extract_text_from_range(
    tree: &DocumentTree,
    node_id: NodeId,
    start_offset: usize,
    end_offset: usize,
) -> Option<String> {
    // Check if it's a paragraph
    if let Some(para) = tree.get_paragraph(node_id) {
        let mut result = String::new();
        let mut current_offset = 0;

        for &run_id in para.children() {
            if let Some(run) = tree.get_run(run_id) {
                let run_len = run.text.chars().count();
                let run_start = current_offset;
                let run_end = current_offset + run_len;

                // Check if this run overlaps with our range
                if run_end > start_offset && run_start < end_offset {
                    let extract_start = start_offset.saturating_sub(run_start);
                    let extract_end = (end_offset - run_start).min(run_len);

                    // Extract the relevant portion
                    let chars: Vec<char> = run.text.chars().collect();
                    if extract_start < chars.len() {
                        let end = extract_end.min(chars.len());
                        result.extend(&chars[extract_start..end]);
                    }
                }

                current_offset = run_end;
                if current_offset >= end_offset {
                    break;
                }
            }
        }

        return Some(result);
    }

    // Check if it's a run directly
    if let Some(run) = tree.get_run(node_id) {
        let chars: Vec<char> = run.text.chars().collect();
        let start = start_offset.min(chars.len());
        let end = end_offset.min(chars.len());
        return Some(chars[start..end].iter().collect());
    }

    None
}

/// Get character properties for a range (for creating format change records)
pub fn get_character_properties_for_range(
    tree: &DocumentTree,
    node_id: NodeId,
    offset: usize,
) -> Option<CharacterProperties> {
    // If node is a paragraph, find the run at the offset
    if let Some(para) = tree.get_paragraph(node_id) {
        let mut current_offset = 0;

        for &run_id in para.children() {
            if let Some(run) = tree.get_run(run_id) {
                let run_len = run.text.chars().count();
                if current_offset + run_len > offset {
                    // Found the run at this offset
                    return Some(run.direct_formatting.clone());
                }
                current_offset += run_len;
            }
        }
    }

    // If node is a run
    if let Some(run) = tree.get_run(node_id) {
        return Some(run.direct_formatting.clone());
    }

    None
}

/// Get paragraph properties for a node
pub fn get_paragraph_properties_for_node(
    tree: &DocumentTree,
    node_id: NodeId,
) -> Option<ParagraphProperties> {
    if let Some(para) = tree.get_paragraph(node_id) {
        return Some(para.direct_formatting.clone());
    }

    // If it's a run, get the parent paragraph
    if let Some(run) = tree.get_run(node_id) {
        if let Some(para_id) = run.parent() {
            if let Some(para) = tree.get_paragraph(para_id) {
                return Some(para.direct_formatting.clone());
            }
        }
    }

    None
}

// =============================================================================
// Revision Summary
// =============================================================================

/// Summary of revisions in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevisionSummary {
    /// Total number of revisions
    pub total: usize,
    /// Number of pending revisions
    pub pending: usize,
    /// Number of accepted revisions
    pub accepted: usize,
    /// Number of rejected revisions
    pub rejected: usize,
    /// Number of insertions
    pub insertions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Number of format changes
    pub format_changes: usize,
    /// Number of moves
    pub moves: usize,
    /// Unique authors
    pub authors: Vec<String>,
}

impl RevisionSummary {
    /// Create a summary from a revision state
    pub fn from_state(state: &RevisionState) -> Self {
        let mut summary = Self::default();
        let mut authors = std::collections::HashSet::new();

        for revision in state.all_revisions() {
            summary.total += 1;

            match revision.status {
                crate::RevisionStatus::Pending => summary.pending += 1,
                crate::RevisionStatus::Accepted => summary.accepted += 1,
                crate::RevisionStatus::Rejected => summary.rejected += 1,
            }

            match &revision.revision_type {
                crate::RevisionType::Insert { .. } => summary.insertions += 1,
                crate::RevisionType::Delete { .. } => summary.deletions += 1,
                crate::RevisionType::FormatChange { .. } => summary.format_changes += 1,
                crate::RevisionType::Move { .. } => summary.moves += 1,
            }

            authors.insert(revision.author.clone());
        }

        summary.authors = authors.into_iter().collect();
        summary.authors.sort();
        summary
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracked_insert() {
        let mut state = RevisionState::with_author("TestUser");
        state.enable_tracking().unwrap();

        let position = Position::new(NodeId::new(), 0);
        let mut insert = TrackedInsert::new(position, "Hello");

        insert.record(&mut state).unwrap();

        assert!(insert.revision_id.is_some());
        assert_eq!(state.revision_count(), 1);
    }

    #[test]
    fn test_tracked_insert_no_tracking() {
        let mut state = RevisionState::new();
        // Tracking not enabled

        let position = Position::new(NodeId::new(), 0);
        let mut insert = TrackedInsert::new(position, "Hello");

        insert.record(&mut state).unwrap();

        assert!(insert.revision_id.is_none());
        assert_eq!(state.revision_count(), 0);
    }

    #[test]
    fn test_tracked_delete() {
        let mut state = RevisionState::with_author("TestUser");
        state.enable_tracking().unwrap();

        let node_id = NodeId::new();
        let mut delete = TrackedDelete::new(
            Position::new(node_id, 0),
            Position::new(node_id, 5),
            "Hello",
        );

        delete.record(&mut state).unwrap();

        assert!(delete.revision_id.is_some());
        let revision = state.get(delete.revision_id.unwrap()).unwrap();
        matches!(revision.revision_type, crate::RevisionType::Delete { .. });
    }

    #[test]
    fn test_process_accept_insert() {
        let mut state = RevisionState::with_author("TestUser");
        state.enable_tracking().unwrap();

        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 5);
        let id = state.record_insert(range).unwrap();

        let result = process_accept(&mut state, id).unwrap();

        assert!(!result.needs_modification);
        matches!(result.details, AcceptDetails::InsertAccepted);
    }

    #[test]
    fn test_process_accept_delete() {
        let mut state = RevisionState::with_author("TestUser");
        state.enable_tracking().unwrap();

        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 5);
        let content = DeletedContent::new("Hello");
        let id = state.record_delete(range, content).unwrap();

        let result = process_accept(&mut state, id).unwrap();

        assert!(result.needs_modification);
        matches!(result.details, AcceptDetails::DeleteAccepted { .. });
    }

    #[test]
    fn test_process_reject_insert() {
        let mut state = RevisionState::with_author("TestUser");
        state.enable_tracking().unwrap();

        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 5);
        let id = state.record_insert(range).unwrap();

        let result = process_reject(&mut state, id).unwrap();

        assert!(result.needs_modification);
        matches!(result.details, RejectDetails::InsertRejected { .. });
    }

    #[test]
    fn test_revision_summary() {
        let mut state = RevisionState::with_author("Alice");
        state.enable_tracking().unwrap();

        let node_id = NodeId::new();
        state.record_insert(RevisionRange::new(node_id, 0, 5)).unwrap();
        state.record_delete(
            RevisionRange::new(node_id, 5, 10),
            DeletedContent::new("text"),
        ).unwrap();

        state.set_current_author("Bob").unwrap();
        state.record_insert(RevisionRange::new(node_id, 10, 15)).unwrap();

        let summary = RevisionSummary::from_state(&state);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.pending, 3);
        assert_eq!(summary.insertions, 2);
        assert_eq!(summary.deletions, 1);
        assert_eq!(summary.authors.len(), 2);
    }
}
