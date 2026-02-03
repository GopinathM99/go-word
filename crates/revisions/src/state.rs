//! Revision state management - tracks all revisions and display settings

use crate::{
    DeletedContent, FormatChangeInfo, MoveInfo, Result, Revision, RevisionError, RevisionId,
    RevisionRange, RevisionStatus, RevisionType,
};
use chrono::{DateTime, Utc};
use doc_model::{NodeId, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Display mode for tracked changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MarkupMode {
    /// Show document as it was before all changes (original state)
    Original,
    /// Show final result with all changes applied, no highlighting
    NoMarkup,
    /// Show all changes with colors and formatting indicators
    #[default]
    AllMarkup,
    /// Show minimal change indicators (underline for insertions, strikethrough for deletions)
    SimpleMarkup,
}

impl MarkupMode {
    /// Get a display name for the markup mode
    pub fn display_name(&self) -> &'static str {
        match self {
            MarkupMode::Original => "Original",
            MarkupMode::NoMarkup => "No Markup",
            MarkupMode::AllMarkup => "All Markup",
            MarkupMode::SimpleMarkup => "Simple Markup",
        }
    }
}

/// Color configuration for revision display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionColors {
    /// Colors for different authors (author name -> color)
    pub author_colors: HashMap<String, String>,
    /// Default colors to cycle through for new authors
    pub default_colors: Vec<String>,
    /// Insertion highlight color (fallback when author color not set)
    pub insertion_color: String,
    /// Deletion highlight color (fallback when author color not set)
    pub deletion_color: String,
    /// Move source highlight color
    pub move_from_color: String,
    /// Move destination highlight color
    pub move_to_color: String,
    /// Format change highlight color
    pub format_change_color: String,
}

impl Default for RevisionColors {
    fn default() -> Self {
        Self {
            author_colors: HashMap::new(),
            default_colors: vec![
                "#C00000".to_string(), // Red
                "#0070C0".to_string(), // Blue
                "#00B050".to_string(), // Green
                "#7030A0".to_string(), // Purple
                "#FFC000".to_string(), // Orange
                "#002060".to_string(), // Dark blue
                "#C65911".to_string(), // Brown
                "#00B0F0".to_string(), // Light blue
            ],
            insertion_color: "#00B050".to_string(),     // Green
            deletion_color: "#C00000".to_string(),      // Red
            move_from_color: "#7030A0".to_string(),     // Purple
            move_to_color: "#0070C0".to_string(),       // Blue
            format_change_color: "#FFC000".to_string(), // Orange
        }
    }
}

impl RevisionColors {
    /// Get or assign a color for an author
    pub fn get_author_color(&mut self, author: &str) -> String {
        if let Some(color) = self.author_colors.get(author) {
            return color.clone();
        }

        // Assign a new color
        let color_index = self.author_colors.len() % self.default_colors.len();
        let color = self.default_colors[color_index].clone();
        self.author_colors.insert(author.to_string(), color.clone());
        color
    }

    /// Set a specific color for an author
    pub fn set_author_color(&mut self, author: impl Into<String>, color: impl Into<String>) {
        self.author_colors.insert(author.into(), color.into());
    }
}

/// Filter options for viewing revisions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevisionFilter {
    /// Filter by specific authors (empty = all authors)
    pub authors: Vec<String>,
    /// Filter by revision types (empty = all types)
    pub types: Vec<RevisionTypeFilter>,
    /// Filter by date range (None = no date filter)
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    /// Filter by status
    pub status: Option<RevisionStatus>,
}

/// Simplified revision type for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionTypeFilter {
    Insert,
    Delete,
    FormatChange,
    Move,
}

impl RevisionFilter {
    /// Check if a revision matches this filter
    pub fn matches(&self, revision: &Revision) -> bool {
        // Check author filter
        if !self.authors.is_empty() && !self.authors.contains(&revision.author) {
            return false;
        }

        // Check type filter
        if !self.types.is_empty() {
            let rev_type = match &revision.revision_type {
                RevisionType::Insert { .. } => RevisionTypeFilter::Insert,
                RevisionType::Delete { .. } => RevisionTypeFilter::Delete,
                RevisionType::FormatChange { .. } => RevisionTypeFilter::FormatChange,
                RevisionType::Move { .. } => RevisionTypeFilter::Move,
            };
            if !self.types.contains(&rev_type) {
                return false;
            }
        }

        // Check date range
        if let Some(from) = self.date_from {
            if revision.timestamp < from {
                return false;
            }
        }
        if let Some(to) = self.date_to {
            if revision.timestamp > to {
                return false;
            }
        }

        // Check status
        if let Some(status) = self.status {
            if revision.status != status {
                return false;
            }
        }

        true
    }
}

/// Main revision tracking state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionState {
    /// Whether tracking is currently enabled
    pub tracking_enabled: bool,
    /// Current display mode
    pub show_markup: MarkupMode,
    /// All revisions indexed by ID
    revisions: HashMap<RevisionId, Revision>,
    /// Revisions ordered by timestamp for navigation
    revision_order: Vec<RevisionId>,
    /// Color configuration
    pub colors: RevisionColors,
    /// Current filter (for display)
    pub filter: RevisionFilter,
    /// Current author name (used when creating new revisions)
    pub current_author: String,
    /// Whether to lock tracking (prevent changes to tracking state)
    pub tracking_locked: bool,
    /// Counter for generating unique revision IDs
    next_revision_index: u64,
}

impl Default for RevisionState {
    fn default() -> Self {
        Self::new()
    }
}

impl RevisionState {
    /// Create a new revision state
    pub fn new() -> Self {
        Self {
            tracking_enabled: false,
            show_markup: MarkupMode::AllMarkup,
            revisions: HashMap::new(),
            revision_order: Vec::new(),
            colors: RevisionColors::default(),
            filter: RevisionFilter::default(),
            current_author: "Unknown".to_string(),
            tracking_locked: false,
            next_revision_index: 0,
        }
    }

    /// Create a new revision state with a default author
    pub fn with_author(author: impl Into<String>) -> Self {
        let mut state = Self::new();
        state.current_author = author.into();
        state
    }

    // =========================================================================
    // Tracking Control
    // =========================================================================

    /// Enable track changes
    pub fn enable_tracking(&mut self) -> Result<()> {
        if self.tracking_locked {
            return Err(RevisionError::InvalidOperation(
                "Tracking is locked".to_string(),
            ));
        }
        self.tracking_enabled = true;
        Ok(())
    }

    /// Disable track changes
    pub fn disable_tracking(&mut self) -> Result<()> {
        if self.tracking_locked {
            return Err(RevisionError::InvalidOperation(
                "Tracking is locked".to_string(),
            ));
        }
        self.tracking_enabled = false;
        Ok(())
    }

    /// Toggle track changes
    pub fn toggle_tracking(&mut self) -> Result<bool> {
        if self.tracking_locked {
            return Err(RevisionError::InvalidOperation(
                "Tracking is locked".to_string(),
            ));
        }
        self.tracking_enabled = !self.tracking_enabled;
        Ok(self.tracking_enabled)
    }

    /// Check if tracking is enabled
    pub fn is_tracking(&self) -> bool {
        self.tracking_enabled
    }

    /// Lock tracking (prevent enable/disable changes)
    pub fn lock_tracking(&mut self) {
        self.tracking_locked = true;
    }

    /// Unlock tracking
    pub fn unlock_tracking(&mut self) {
        self.tracking_locked = false;
    }

    // =========================================================================
    // Display Mode
    // =========================================================================

    /// Set the markup display mode
    pub fn set_markup_mode(&mut self, mode: MarkupMode) {
        self.show_markup = mode;
    }

    /// Get the current markup mode
    pub fn markup_mode(&self) -> MarkupMode {
        self.show_markup
    }

    // =========================================================================
    // Author Management
    // =========================================================================

    /// Set the current author for new revisions
    pub fn set_current_author(&mut self, author: impl Into<String>) -> Result<()> {
        let author = author.into();
        if author.trim().is_empty() {
            return Err(RevisionError::InvalidAuthor(
                "Author name cannot be empty".to_string(),
            ));
        }
        self.current_author = author;
        Ok(())
    }

    /// Get the current author
    pub fn current_author(&self) -> &str {
        &self.current_author
    }

    /// Get all unique authors who have made revisions
    pub fn all_authors(&self) -> Vec<&str> {
        let mut authors: Vec<&str> = self.revisions.values().map(|r| r.author.as_str()).collect();
        authors.sort();
        authors.dedup();
        authors
    }

    // =========================================================================
    // Revision Creation
    // =========================================================================

    /// Record an insertion revision
    pub fn record_insert(&mut self, range: RevisionRange) -> Result<RevisionId> {
        if !self.tracking_enabled {
            return Err(RevisionError::TrackingDisabled);
        }

        let revision = Revision::insert(&self.current_author, range);
        let id = revision.id;
        self.add_revision(revision);
        Ok(id)
    }

    /// Record a deletion revision
    pub fn record_delete(
        &mut self,
        range: RevisionRange,
        deleted_content: DeletedContent,
    ) -> Result<RevisionId> {
        if !self.tracking_enabled {
            return Err(RevisionError::TrackingDisabled);
        }

        let revision = Revision::delete(&self.current_author, range, deleted_content);
        let id = revision.id;
        self.add_revision(revision);
        Ok(id)
    }

    /// Record a format change revision
    pub fn record_format_change(
        &mut self,
        range: RevisionRange,
        format_info: FormatChangeInfo,
    ) -> Result<RevisionId> {
        if !self.tracking_enabled {
            return Err(RevisionError::TrackingDisabled);
        }

        let revision = Revision::format_change(&self.current_author, range, format_info);
        let id = revision.id;
        self.add_revision(revision);
        Ok(id)
    }

    /// Record a move revision (returns two IDs: source and destination)
    pub fn record_move(&mut self, move_info: MoveInfo) -> Result<(RevisionId, RevisionId)> {
        if !self.tracking_enabled {
            return Err(RevisionError::TrackingDisabled);
        }

        // Create two linked revisions for move operations
        let mut source_revision = Revision::move_content(&self.current_author, move_info.clone());
        let source_id = source_revision.id;

        // Create the destination revision and link them
        let mut dest_revision = Revision::move_content(&self.current_author, move_info);
        let dest_id = dest_revision.id;

        source_revision.linked_revision = Some(dest_id);
        dest_revision.linked_revision = Some(source_id);

        self.add_revision(source_revision);
        self.add_revision(dest_revision);

        Ok((source_id, dest_id))
    }

    /// Add a revision (internal)
    fn add_revision(&mut self, revision: Revision) {
        let id = revision.id;
        self.revisions.insert(id, revision);
        self.revision_order.push(id);
        self.next_revision_index += 1;
    }

    /// Add a pre-built revision (for deserialization/testing)
    pub fn add_existing_revision(&mut self, revision: Revision) {
        let id = revision.id;
        if !self.revisions.contains_key(&id) {
            self.revisions.insert(id, revision);
            self.revision_order.push(id);
        }
    }

    // =========================================================================
    // Revision Retrieval
    // =========================================================================

    /// Get a revision by ID
    pub fn get(&self, id: RevisionId) -> Option<&Revision> {
        self.revisions.get(&id)
    }

    /// Get a mutable revision by ID
    pub fn get_mut(&mut self, id: RevisionId) -> Option<&mut Revision> {
        self.revisions.get_mut(&id)
    }

    /// Get all revisions
    pub fn all_revisions(&self) -> impl Iterator<Item = &Revision> {
        self.revisions.values()
    }

    /// Get all pending revisions
    pub fn pending_revisions(&self) -> impl Iterator<Item = &Revision> {
        self.revisions.values().filter(|r| r.is_pending())
    }

    /// Get revisions in chronological order
    pub fn revisions_in_order(&self) -> Vec<&Revision> {
        self.revision_order
            .iter()
            .filter_map(|id| self.revisions.get(id))
            .collect()
    }

    /// Get filtered revisions
    pub fn filtered_revisions(&self) -> Vec<&Revision> {
        self.revisions
            .values()
            .filter(|r| self.filter.matches(r))
            .collect()
    }

    /// Get revisions for a specific node
    pub fn revisions_for_node(&self, node_id: NodeId) -> Vec<&Revision> {
        self.revisions
            .values()
            .filter(|r| r.range().node_id == node_id)
            .collect()
    }

    /// Get revisions at a specific position
    pub fn revisions_at_position(&self, position: &Position) -> Vec<&Revision> {
        self.revisions
            .values()
            .filter(|r| r.range().contains_position(position))
            .collect()
    }

    /// Get revisions by author
    pub fn revisions_by_author(&self, author: &str) -> Vec<&Revision> {
        self.revisions
            .values()
            .filter(|r| r.author == author)
            .collect()
    }

    /// Count total revisions
    pub fn revision_count(&self) -> usize {
        self.revisions.len()
    }

    /// Count pending revisions
    pub fn pending_count(&self) -> usize {
        self.revisions.values().filter(|r| r.is_pending()).count()
    }

    /// Check if there are any pending revisions
    pub fn has_pending_revisions(&self) -> bool {
        self.revisions.values().any(|r| r.is_pending())
    }

    // =========================================================================
    // Accept/Reject Operations
    // =========================================================================

    /// Accept a single revision
    pub fn accept_revision(&mut self, id: RevisionId) -> Result<()> {
        let revision = self
            .revisions
            .get_mut(&id)
            .ok_or(RevisionError::RevisionNotFound(id.as_uuid()))?;

        if !revision.is_pending() {
            return Err(RevisionError::RevisionAlreadyProcessed(id.as_uuid()));
        }

        revision.status = RevisionStatus::Accepted;

        // Handle linked revisions (for moves)
        if let Some(linked_id) = revision.linked_revision {
            if let Some(linked) = self.revisions.get_mut(&linked_id) {
                if linked.is_pending() {
                    linked.status = RevisionStatus::Accepted;
                }
            }
        }

        Ok(())
    }

    /// Reject a single revision
    pub fn reject_revision(&mut self, id: RevisionId) -> Result<()> {
        let revision = self
            .revisions
            .get_mut(&id)
            .ok_or(RevisionError::RevisionNotFound(id.as_uuid()))?;

        if !revision.is_pending() {
            return Err(RevisionError::RevisionAlreadyProcessed(id.as_uuid()));
        }

        revision.status = RevisionStatus::Rejected;

        // Handle linked revisions (for moves)
        if let Some(linked_id) = revision.linked_revision {
            if let Some(linked) = self.revisions.get_mut(&linked_id) {
                if linked.is_pending() {
                    linked.status = RevisionStatus::Rejected;
                }
            }
        }

        Ok(())
    }

    /// Accept all pending revisions
    pub fn accept_all(&mut self) -> Vec<RevisionId> {
        let pending_ids: Vec<RevisionId> = self
            .revisions
            .values()
            .filter(|r| r.is_pending())
            .map(|r| r.id)
            .collect();

        for id in &pending_ids {
            if let Some(revision) = self.revisions.get_mut(id) {
                revision.status = RevisionStatus::Accepted;
            }
        }

        pending_ids
    }

    /// Reject all pending revisions
    pub fn reject_all(&mut self) -> Vec<RevisionId> {
        let pending_ids: Vec<RevisionId> = self
            .revisions
            .values()
            .filter(|r| r.is_pending())
            .map(|r| r.id)
            .collect();

        for id in &pending_ids {
            if let Some(revision) = self.revisions.get_mut(id) {
                revision.status = RevisionStatus::Rejected;
            }
        }

        pending_ids
    }

    /// Accept all revisions by a specific author
    pub fn accept_by_author(&mut self, author: &str) -> Vec<RevisionId> {
        let matching_ids: Vec<RevisionId> = self
            .revisions
            .values()
            .filter(|r| r.is_pending() && r.author == author)
            .map(|r| r.id)
            .collect();

        for id in &matching_ids {
            if let Some(revision) = self.revisions.get_mut(id) {
                revision.status = RevisionStatus::Accepted;
            }
        }

        matching_ids
    }

    /// Reject all revisions by a specific author
    pub fn reject_by_author(&mut self, author: &str) -> Vec<RevisionId> {
        let matching_ids: Vec<RevisionId> = self
            .revisions
            .values()
            .filter(|r| r.is_pending() && r.author == author)
            .map(|r| r.id)
            .collect();

        for id in &matching_ids {
            if let Some(revision) = self.revisions.get_mut(id) {
                revision.status = RevisionStatus::Rejected;
            }
        }

        matching_ids
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Get the next revision after the given position
    pub fn next_revision(&self, current_position: &Position) -> Option<&Revision> {
        // Find revisions that start after the current position
        // We compare by offset within the same node, otherwise by timestamp
        let mut candidates: Vec<&Revision> = self
            .revisions
            .values()
            .filter(|r| {
                r.is_pending() && self.filter.matches(r) && {
                    let range = r.range();
                    // If same node, check if after current offset
                    if range.node_id == current_position.node_id {
                        range.start_offset > current_position.offset
                    } else {
                        // For different nodes, compare by UUID for consistent ordering
                        range.node_id.as_uuid() > current_position.node_id.as_uuid()
                    }
                }
            })
            .collect();

        // Sort by position (using UUID for cross-node comparison)
        candidates.sort_by(|a, b| {
            let ra = a.range();
            let rb = b.range();
            ra.node_id
                .as_uuid()
                .cmp(&rb.node_id.as_uuid())
                .then(ra.start_offset.cmp(&rb.start_offset))
        });

        candidates.first().copied()
    }

    /// Get the previous revision before the given position
    pub fn previous_revision(&self, current_position: &Position) -> Option<&Revision> {
        // Find revisions that end before the current position
        let mut candidates: Vec<&Revision> = self
            .revisions
            .values()
            .filter(|r| {
                r.is_pending() && self.filter.matches(r) && {
                    let range = r.range();
                    // If same node, check if before current offset
                    if range.node_id == current_position.node_id {
                        range.end_offset < current_position.offset
                    } else {
                        // For different nodes, compare by UUID for consistent ordering
                        range.node_id.as_uuid() < current_position.node_id.as_uuid()
                    }
                }
            })
            .collect();

        // Sort by position (descending, using UUID for cross-node comparison)
        candidates.sort_by(|a, b| {
            let ra = a.range();
            let rb = b.range();
            rb.node_id
                .as_uuid()
                .cmp(&ra.node_id.as_uuid())
                .then(rb.end_offset.cmp(&ra.end_offset))
        });

        candidates.first().copied()
    }

    /// Get the position to navigate to for a revision
    pub fn position_for_revision(&self, id: RevisionId) -> Option<Position> {
        self.revisions.get(&id).map(|r| {
            let range = r.range();
            Position::new(range.node_id, range.start_offset)
        })
    }

    // =========================================================================
    // Range Adjustment
    // =========================================================================

    /// Adjust all revision ranges for an insertion
    pub fn adjust_for_insertion(&mut self, at: &Position, length: usize) {
        for revision in self.revisions.values_mut() {
            match &mut revision.revision_type {
                RevisionType::Insert { range }
                | RevisionType::Delete { range, .. }
                | RevisionType::FormatChange { range, .. } => {
                    range.adjust_for_insertion(at, length);
                }
                RevisionType::Move { move_info } => {
                    move_info.from_range.adjust_for_insertion(at, length);
                    move_info.to_range.adjust_for_insertion(at, length);
                }
            }
        }
    }

    /// Adjust all revision ranges for a deletion
    pub fn adjust_for_deletion(&mut self, range: &RevisionRange) {
        for revision in self.revisions.values_mut() {
            match &mut revision.revision_type {
                RevisionType::Insert { range: rev_range }
                | RevisionType::Delete { range: rev_range, .. }
                | RevisionType::FormatChange { range: rev_range, .. } => {
                    rev_range.adjust_for_deletion(range);
                }
                RevisionType::Move { move_info } => {
                    move_info.from_range.adjust_for_deletion(range);
                    move_info.to_range.adjust_for_deletion(range);
                }
            }
        }
    }

    // =========================================================================
    // Cleanup
    // =========================================================================

    /// Remove all accepted revisions (cleanup)
    pub fn clear_accepted(&mut self) {
        let accepted_ids: Vec<RevisionId> = self
            .revisions
            .values()
            .filter(|r| r.is_accepted())
            .map(|r| r.id)
            .collect();

        for id in accepted_ids {
            self.revisions.remove(&id);
            self.revision_order.retain(|&i| i != id);
        }
    }

    /// Remove all rejected revisions (cleanup)
    pub fn clear_rejected(&mut self) {
        let rejected_ids: Vec<RevisionId> = self
            .revisions
            .values()
            .filter(|r| r.is_rejected())
            .map(|r| r.id)
            .collect();

        for id in rejected_ids {
            self.revisions.remove(&id);
            self.revision_order.retain(|&i| i != id);
        }
    }

    /// Clear all revisions
    pub fn clear_all(&mut self) {
        self.revisions.clear();
        self.revision_order.clear();
        self.next_revision_index = 0;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> RevisionState {
        let mut state = RevisionState::with_author("TestUser");
        state.enable_tracking().unwrap();
        state
    }

    #[test]
    fn test_enable_disable_tracking() {
        let mut state = RevisionState::new();
        assert!(!state.is_tracking());

        state.enable_tracking().unwrap();
        assert!(state.is_tracking());

        state.disable_tracking().unwrap();
        assert!(!state.is_tracking());

        state.toggle_tracking().unwrap();
        assert!(state.is_tracking());
    }

    #[test]
    fn test_tracking_lock() {
        let mut state = RevisionState::new();
        state.enable_tracking().unwrap();
        state.lock_tracking();

        // Should fail when locked
        assert!(state.disable_tracking().is_err());
        assert!(state.toggle_tracking().is_err());

        state.unlock_tracking();
        state.disable_tracking().unwrap();
        assert!(!state.is_tracking());
    }

    #[test]
    fn test_record_insert() {
        let mut state = create_test_state();
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let id = state.record_insert(range).unwrap();

        let revision = state.get(id).unwrap();
        assert_eq!(revision.author, "TestUser");
        assert!(revision.is_pending());
        matches!(revision.revision_type, RevisionType::Insert { .. });
    }

    #[test]
    fn test_record_delete() {
        let mut state = create_test_state();
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);
        let content = DeletedContent::new("deleted text");

        let id = state.record_delete(range, content).unwrap();

        let revision = state.get(id).unwrap();
        matches!(revision.revision_type, RevisionType::Delete { .. });
    }

    #[test]
    fn test_record_without_tracking() {
        let mut state = RevisionState::new();
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let result = state.record_insert(range);
        assert!(matches!(result, Err(RevisionError::TrackingDisabled)));
    }

    #[test]
    fn test_accept_revision() {
        let mut state = create_test_state();
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let id = state.record_insert(range).unwrap();
        state.accept_revision(id).unwrap();

        let revision = state.get(id).unwrap();
        assert!(revision.is_accepted());
    }

    #[test]
    fn test_reject_revision() {
        let mut state = create_test_state();
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let id = state.record_insert(range).unwrap();
        state.reject_revision(id).unwrap();

        let revision = state.get(id).unwrap();
        assert!(revision.is_rejected());
    }

    #[test]
    fn test_accept_already_processed() {
        let mut state = create_test_state();
        let node_id = NodeId::new();
        let range = RevisionRange::new(node_id, 0, 10);

        let id = state.record_insert(range).unwrap();
        state.accept_revision(id).unwrap();

        let result = state.accept_revision(id);
        assert!(matches!(result, Err(RevisionError::RevisionAlreadyProcessed(_))));
    }

    #[test]
    fn test_accept_all() {
        let mut state = create_test_state();
        let node_id = NodeId::new();

        state.record_insert(RevisionRange::new(node_id, 0, 5)).unwrap();
        state.record_insert(RevisionRange::new(node_id, 5, 10)).unwrap();
        state.record_insert(RevisionRange::new(node_id, 10, 15)).unwrap();

        let accepted = state.accept_all();
        assert_eq!(accepted.len(), 3);
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn test_accept_by_author() {
        let mut state = RevisionState::new();
        state.enable_tracking().unwrap();
        let node_id = NodeId::new();

        state.set_current_author("Alice").unwrap();
        state.record_insert(RevisionRange::new(node_id, 0, 5)).unwrap();

        state.set_current_author("Bob").unwrap();
        state.record_insert(RevisionRange::new(node_id, 5, 10)).unwrap();

        state.set_current_author("Alice").unwrap();
        state.record_insert(RevisionRange::new(node_id, 10, 15)).unwrap();

        let accepted = state.accept_by_author("Alice");
        assert_eq!(accepted.len(), 2);
        assert_eq!(state.pending_count(), 1);
    }

    #[test]
    fn test_markup_mode() {
        let mut state = RevisionState::new();
        assert_eq!(state.markup_mode(), MarkupMode::AllMarkup);

        state.set_markup_mode(MarkupMode::NoMarkup);
        assert_eq!(state.markup_mode(), MarkupMode::NoMarkup);
    }

    #[test]
    fn test_author_colors() {
        let mut colors = RevisionColors::default();

        let color1 = colors.get_author_color("Alice");
        let color2 = colors.get_author_color("Bob");
        let color3 = colors.get_author_color("Alice");

        // Same author should get same color
        assert_eq!(color1, color3);
        // Different authors should get different colors (with 8 default colors)
        assert_ne!(color1, color2);
    }

    #[test]
    fn test_revision_filter() {
        let node_id = NodeId::new();
        let revision = Revision::insert("Alice", RevisionRange::new(node_id, 0, 10));

        let filter = RevisionFilter {
            authors: vec!["Alice".to_string()],
            ..Default::default()
        };
        assert!(filter.matches(&revision));

        let filter2 = RevisionFilter {
            authors: vec!["Bob".to_string()],
            ..Default::default()
        };
        assert!(!filter2.matches(&revision));
    }

    #[test]
    fn test_revisions_for_node() {
        let mut state = create_test_state();
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        state.record_insert(RevisionRange::new(node1, 0, 5)).unwrap();
        state.record_insert(RevisionRange::new(node1, 5, 10)).unwrap();
        state.record_insert(RevisionRange::new(node2, 0, 5)).unwrap();

        let revisions = state.revisions_for_node(node1);
        assert_eq!(revisions.len(), 2);
    }

    #[test]
    fn test_clear_accepted() {
        let mut state = create_test_state();
        let node_id = NodeId::new();

        let id1 = state.record_insert(RevisionRange::new(node_id, 0, 5)).unwrap();
        let _id2 = state.record_insert(RevisionRange::new(node_id, 5, 10)).unwrap();

        state.accept_revision(id1).unwrap();
        assert_eq!(state.revision_count(), 2);

        state.clear_accepted();
        assert_eq!(state.revision_count(), 1);
    }

    #[test]
    fn test_linked_revisions() {
        let mut state = create_test_state();
        let node_id = NodeId::new();

        let move_info = MoveInfo {
            from_range: RevisionRange::new(node_id, 0, 10),
            to_range: RevisionRange::new(node_id, 20, 30),
        };

        let (source_id, dest_id) = state.record_move(move_info).unwrap();

        // Accepting source should also accept destination
        state.accept_revision(source_id).unwrap();

        let source = state.get(source_id).unwrap();
        let dest = state.get(dest_id).unwrap();

        assert!(source.is_accepted());
        assert!(dest.is_accepted());
    }
}
