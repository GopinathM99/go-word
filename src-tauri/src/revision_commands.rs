//! Track Changes / Revision Tracking Tauri Commands
//!
//! This module provides Tauri commands for the revision tracking system.

use crate::state::RevisionStateWrapper;
use revisions::{
    MarkupMode, RevisionId, RevisionStatus, RevisionSummary, RevisionTypeFilter,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Position DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub node_id: String,
    pub offset: usize,
}

/// Revision DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionDto {
    /// Unique revision ID
    pub id: String,
    /// Type of revision
    pub revision_type: String,
    /// Author who made the change
    pub author: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Current status
    pub status: String,
    /// Optional comment
    pub comment: Option<String>,
    /// Node ID where revision applies
    pub node_id: String,
    /// Start offset
    pub start_offset: usize,
    /// End offset
    pub end_offset: usize,
    /// Deleted text (for deletion revisions)
    pub deleted_text: Option<String>,
}

impl From<&revisions::Revision> for RevisionDto {
    fn from(r: &revisions::Revision) -> Self {
        let range = r.range();
        let (rev_type, deleted_text) = match &r.revision_type {
            revisions::RevisionType::Insert { .. } => ("insert".to_string(), None),
            revisions::RevisionType::Delete { deleted_content, .. } => {
                ("delete".to_string(), Some(deleted_content.text.clone()))
            }
            revisions::RevisionType::FormatChange { .. } => ("formatChange".to_string(), None),
            revisions::RevisionType::Move { .. } => ("move".to_string(), None),
        };

        Self {
            id: r.id.to_string(),
            revision_type: rev_type,
            author: r.author.clone(),
            timestamp: r.timestamp.to_rfc3339(),
            status: match r.status {
                RevisionStatus::Pending => "pending".to_string(),
                RevisionStatus::Accepted => "accepted".to_string(),
                RevisionStatus::Rejected => "rejected".to_string(),
            },
            comment: r.comment.clone(),
            node_id: range.node_id.to_string(),
            start_offset: range.start_offset,
            end_offset: range.end_offset,
            deleted_text,
        }
    }
}

/// Revision summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionSummaryDto {
    pub total: usize,
    pub pending: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub format_changes: usize,
    pub moves: usize,
    pub authors: Vec<String>,
}

impl From<RevisionSummary> for RevisionSummaryDto {
    fn from(s: RevisionSummary) -> Self {
        Self {
            total: s.total,
            pending: s.pending,
            accepted: s.accepted,
            rejected: s.rejected,
            insertions: s.insertions,
            deletions: s.deletions,
            format_changes: s.format_changes,
            moves: s.moves,
            authors: s.authors,
        }
    }
}

/// Revision state DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionStateDto {
    /// Whether tracking is enabled
    pub tracking_enabled: bool,
    /// Current markup mode
    pub markup_mode: String,
    /// Current author
    pub current_author: String,
    /// Whether tracking is locked
    pub tracking_locked: bool,
    /// Summary of revisions
    pub summary: RevisionSummaryDto,
}

/// Toggle track changes on/off
#[tauri::command]
pub fn toggle_track_changes(
    state: State<'_, RevisionStateWrapper>,
) -> Result<bool, String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.toggle_tracking().map_err(|e| e.to_string())
}

/// Enable track changes
#[tauri::command]
pub fn enable_track_changes(
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.enable_tracking().map_err(|e| e.to_string())
}

/// Disable track changes
#[tauri::command]
pub fn disable_track_changes(
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.disable_tracking().map_err(|e| e.to_string())
}

/// Check if track changes is enabled
#[tauri::command]
pub fn is_tracking_changes(
    state: State<'_, RevisionStateWrapper>,
) -> Result<bool, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(revision_state.is_tracking())
}

/// Set the markup display mode
#[tauri::command]
pub fn set_markup_mode(
    mode: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let markup_mode = match mode.as_str() {
        "original" => MarkupMode::Original,
        "noMarkup" | "no_markup" => MarkupMode::NoMarkup,
        "allMarkup" | "all_markup" => MarkupMode::AllMarkup,
        "simpleMarkup" | "simple_markup" => MarkupMode::SimpleMarkup,
        _ => return Err(format!("Invalid markup mode: {}", mode)),
    };
    revision_state.set_markup_mode(markup_mode);
    Ok(())
}

/// Get the current markup mode
#[tauri::command]
pub fn get_markup_mode(
    state: State<'_, RevisionStateWrapper>,
) -> Result<String, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let mode = match revision_state.markup_mode() {
        MarkupMode::Original => "original",
        MarkupMode::NoMarkup => "noMarkup",
        MarkupMode::AllMarkup => "allMarkup",
        MarkupMode::SimpleMarkup => "simpleMarkup",
    };
    Ok(mode.to_string())
}

/// Set the current author for new revisions
#[tauri::command]
pub fn set_revision_author(
    author: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.set_current_author(author).map_err(|e| e.to_string())
}

/// Get the current revision author
#[tauri::command]
pub fn get_revision_author(
    state: State<'_, RevisionStateWrapper>,
) -> Result<String, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(revision_state.current_author().to_string())
}

/// Accept a single revision
#[tauri::command]
pub fn accept_revision(
    revision_id: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&revision_id)
        .map_err(|e| format!("Invalid revision ID: {}", e))?;
    let id = RevisionId::from_uuid(uuid);
    revision_state.accept_revision(id).map_err(|e| e.to_string())
}

/// Reject a single revision
#[tauri::command]
pub fn reject_revision(
    revision_id: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&revision_id)
        .map_err(|e| format!("Invalid revision ID: {}", e))?;
    let id = RevisionId::from_uuid(uuid);
    revision_state.reject_revision(id).map_err(|e| e.to_string())
}

/// Accept all pending revisions
#[tauri::command]
pub fn accept_all_revisions(
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<String>, String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let ids = revision_state.accept_all();
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

/// Reject all pending revisions
#[tauri::command]
pub fn reject_all_revisions(
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<String>, String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let ids = revision_state.reject_all();
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

/// Accept all revisions by a specific author
#[tauri::command]
pub fn accept_revisions_by_author(
    author: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<String>, String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let ids = revision_state.accept_by_author(&author);
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

/// Reject all revisions by a specific author
#[tauri::command]
pub fn reject_revisions_by_author(
    author: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<String>, String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let ids = revision_state.reject_by_author(&author);
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

/// Get all revisions
#[tauri::command]
pub fn get_revisions(
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<RevisionDto>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let revisions: Vec<RevisionDto> = revision_state
        .revisions_in_order()
        .iter()
        .map(|r| RevisionDto::from(*r))
        .collect();
    Ok(revisions)
}

/// Get pending revisions only
#[tauri::command]
pub fn get_pending_revisions(
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<RevisionDto>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let revisions: Vec<RevisionDto> = revision_state
        .pending_revisions()
        .map(RevisionDto::from)
        .collect();
    Ok(revisions)
}

/// Get a single revision by ID
#[tauri::command]
pub fn get_revision(
    revision_id: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Option<RevisionDto>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&revision_id)
        .map_err(|e| format!("Invalid revision ID: {}", e))?;
    let id = RevisionId::from_uuid(uuid);
    Ok(revision_state.get(id).map(RevisionDto::from))
}

/// Get revisions by a specific author
#[tauri::command]
pub fn get_revisions_by_author(
    author: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<RevisionDto>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let revisions: Vec<RevisionDto> = revision_state
        .revisions_by_author(&author)
        .iter()
        .map(|r| RevisionDto::from(*r))
        .collect();
    Ok(revisions)
}

/// Get all unique authors who have made revisions
#[tauri::command]
pub fn get_revision_authors(
    state: State<'_, RevisionStateWrapper>,
) -> Result<Vec<String>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(revision_state.all_authors().iter().map(|s| s.to_string()).collect())
}

/// Navigate to a specific revision (returns the position)
#[tauri::command]
pub fn navigate_to_revision(
    revision_id: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Option<Position>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&revision_id)
        .map_err(|e| format!("Invalid revision ID: {}", e))?;
    let id = RevisionId::from_uuid(uuid);

    Ok(revision_state.position_for_revision(id).map(|p| Position {
        node_id: p.node_id.to_string(),
        offset: p.offset,
    }))
}

/// Get next revision from current position
#[tauri::command]
pub fn get_next_revision(
    node_id: String,
    offset: usize,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Option<RevisionDto>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&node_id)
        .map_err(|e| format!("Invalid node ID: {}", e))?;
    let position = doc_model::Position::new(doc_model::NodeId::from_uuid(uuid), offset);

    Ok(revision_state.next_revision(&position).map(RevisionDto::from))
}

/// Get previous revision from current position
#[tauri::command]
pub fn get_previous_revision(
    node_id: String,
    offset: usize,
    state: State<'_, RevisionStateWrapper>,
) -> Result<Option<RevisionDto>, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&node_id)
        .map_err(|e| format!("Invalid node ID: {}", e))?;
    let position = doc_model::Position::new(doc_model::NodeId::from_uuid(uuid), offset);

    Ok(revision_state.previous_revision(&position).map(RevisionDto::from))
}

/// Get revision summary/statistics
#[tauri::command]
pub fn get_revision_summary(
    state: State<'_, RevisionStateWrapper>,
) -> Result<RevisionSummaryDto, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let summary = RevisionSummary::from_state(&revision_state);
    Ok(RevisionSummaryDto::from(summary))
}

/// Get the full revision tracking state
#[tauri::command]
pub fn get_revision_state(
    state: State<'_, RevisionStateWrapper>,
) -> Result<RevisionStateDto, String> {
    let revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let summary = RevisionSummary::from_state(&revision_state);

    Ok(RevisionStateDto {
        tracking_enabled: revision_state.is_tracking(),
        markup_mode: match revision_state.markup_mode() {
            MarkupMode::Original => "original".to_string(),
            MarkupMode::NoMarkup => "noMarkup".to_string(),
            MarkupMode::AllMarkup => "allMarkup".to_string(),
            MarkupMode::SimpleMarkup => "simpleMarkup".to_string(),
        },
        current_author: revision_state.current_author().to_string(),
        tracking_locked: revision_state.tracking_locked,
        summary: RevisionSummaryDto::from(summary),
    })
}

/// Set revision filter by authors
#[tauri::command]
pub fn set_revision_filter_authors(
    authors: Vec<String>,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.filter.authors = authors;
    Ok(())
}

/// Set revision filter by types
#[tauri::command]
pub fn set_revision_filter_types(
    types: Vec<String>,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    let type_filters: Result<Vec<RevisionTypeFilter>, String> = types
        .iter()
        .map(|t| match t.as_str() {
            "insert" => Ok(RevisionTypeFilter::Insert),
            "delete" => Ok(RevisionTypeFilter::Delete),
            "formatChange" | "format_change" => Ok(RevisionTypeFilter::FormatChange),
            "move" => Ok(RevisionTypeFilter::Move),
            _ => Err(format!("Invalid revision type: {}", t)),
        })
        .collect();
    revision_state.filter.types = type_filters?;
    Ok(())
}

/// Clear all revision filters
#[tauri::command]
pub fn clear_revision_filters(
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.filter = revisions::RevisionFilter::default();
    Ok(())
}

/// Clear all accepted revisions (cleanup)
#[tauri::command]
pub fn clear_accepted_revisions(
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.clear_accepted();
    Ok(())
}

/// Clear all rejected revisions (cleanup)
#[tauri::command]
pub fn clear_rejected_revisions(
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.clear_rejected();
    Ok(())
}

/// Set author color for revision highlighting
#[tauri::command]
pub fn set_author_color(
    author: String,
    color: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<(), String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    revision_state.colors.set_author_color(author, color);
    Ok(())
}

/// Get author color for revision highlighting
#[tauri::command]
pub fn get_author_color(
    author: String,
    state: State<'_, RevisionStateWrapper>,
) -> Result<String, String> {
    let mut revision_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(revision_state.colors.get_author_color(&author))
}
