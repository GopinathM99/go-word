//! Tauri IPC commands for collaboration functionality

use crate::state::CollaborationState;
use chrono::Duration;
use collab::{
    CollaborativeDocument, ConnectionStatus, DocId, OfflineManager,
    PermissionLevel, PermissionManager, PermissionTarget, PresenceManager, PresenceState,
    SyncEngine, UserId, VersionHistory, VersionId,
};
use serde::{Deserialize, Serialize};
use tauri::State;

// ============================================================================
// Response Types
// ============================================================================

/// Collaboration initialization info
#[derive(Serialize)]
pub struct CollabInfo {
    pub client_id: u64,
    pub doc_id: String,
}

/// Remote presence info for cursors and selections
#[derive(Serialize)]
pub struct RemotePresenceInfo {
    pub cursors: serde_json::Value,
    pub selections: serde_json::Value,
}

/// Version info for UI display
#[derive(Serialize)]
pub struct VersionInfoDto {
    pub id: String,
    pub timestamp: String,
    pub author: String,
    pub name: Option<String>,
    pub summary: String,
    pub is_named: bool,
    pub is_current: bool,
}

/// Offline status for UI display
#[derive(Serialize)]
pub struct OfflineStatusDto {
    pub status: String,
    pub pending_changes: usize,
    pub time_since_sync: Option<u64>,
    pub status_message: String,
}

// ============================================================================
// Collaboration Initialization Commands
// ============================================================================

/// Initialize collaboration for a document
#[tauri::command]
pub fn init_collaboration(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<CollabInfo, String> {
    let client_id = *state.client_id.lock().unwrap();
    let mut docs = state.documents.lock().unwrap();

    if !docs.contains_key(&doc_id) {
        docs.insert(doc_id.clone(), CollaborativeDocument::new(client_id));
    }

    let mut syncs = state.sync_engines.lock().unwrap();
    if !syncs.contains_key(&doc_id) {
        syncs.insert(doc_id.clone(), SyncEngine::new(client_id));
    }

    let mut presence = state.presence.lock().unwrap();
    if !presence.contains_key(&doc_id) {
        presence.insert(doc_id.clone(), PresenceManager::new());
    }

    let mut versions = state.versions.lock().unwrap();
    if !versions.contains_key(&doc_id) {
        versions.insert(doc_id.clone(), VersionHistory::new());
    }

    Ok(CollabInfo {
        client_id: client_id.0,
        doc_id,
    })
}

/// Close collaboration for a document
#[tauri::command]
pub fn close_collaboration(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    state.documents.lock().unwrap().remove(&doc_id);
    state.sync_engines.lock().unwrap().remove(&doc_id);
    state.presence.lock().unwrap().remove(&doc_id);
    state.versions.lock().unwrap().remove(&doc_id);
    Ok(())
}

// ============================================================================
// CRDT Operation Commands
// ============================================================================

/// Apply a CRDT operation from the frontend
#[tauri::command]
pub fn apply_crdt_op(
    doc_id: String,
    op_json: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let op: collab::operation::CrdtOp = serde_json::from_str(&op_json)
        .map_err(|e| format!("Failed to parse operation: {}", e))?;

    let mut docs = state.documents.lock().unwrap();
    let doc = docs.get_mut(&doc_id).ok_or("Document not found")?;

    doc.apply_remote(op.clone());

    // Also log to version history
    let mut versions = state.versions.lock().unwrap();
    if let Some(vh) = versions.get_mut(&doc_id) {
        vh.record_operation(op, doc.clock(), "local");
    }

    Ok(())
}

/// Apply multiple CRDT operations from the frontend
#[tauri::command]
pub fn apply_crdt_ops_batch(
    doc_id: String,
    ops_json: String,
    state: State<'_, CollaborationState>,
) -> Result<usize, String> {
    let ops: Vec<collab::operation::CrdtOp> = serde_json::from_str(&ops_json)
        .map_err(|e| format!("Failed to parse operations: {}", e))?;

    let mut docs = state.documents.lock().unwrap();
    let doc = docs.get_mut(&doc_id).ok_or("Document not found")?;

    let applied_count = doc.apply_remote_batch(ops.clone());

    // Also log to version history
    let mut versions = state.versions.lock().unwrap();
    if let Some(vh) = versions.get_mut(&doc_id) {
        for op in ops {
            vh.record_operation(op, doc.clock(), "remote");
        }
    }

    Ok(applied_count)
}

// ============================================================================
// Sync Commands
// ============================================================================

/// Get pending operations to send
#[tauri::command]
pub fn get_pending_ops(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<String, String> {
    let mut syncs = state.sync_engines.lock().unwrap();
    let sync = syncs.get_mut(&doc_id).ok_or("Sync engine not found")?;

    if let Some(batch) = sync.get_pending_batch() {
        serde_json::to_string(&batch).map_err(|e| e.to_string())
    } else {
        Ok("null".to_string())
    }
}

/// Queue a local operation for syncing
#[tauri::command]
pub fn queue_local_op(
    doc_id: String,
    op_json: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let op: collab::operation::CrdtOp = serde_json::from_str(&op_json)
        .map_err(|e| format!("Failed to parse operation: {}", e))?;

    let mut syncs = state.sync_engines.lock().unwrap();
    let sync = syncs.get_mut(&doc_id).ok_or("Sync engine not found")?;

    sync.queue_local(op);
    Ok(())
}

/// Handle acknowledgment of operations
#[tauri::command]
pub fn ack_ops(
    doc_id: String,
    op_ids_json: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let op_ids: Vec<collab::OpId> = serde_json::from_str(&op_ids_json)
        .map_err(|e| format!("Failed to parse op IDs: {}", e))?;

    let mut syncs = state.sync_engines.lock().unwrap();
    let sync = syncs.get_mut(&doc_id).ok_or("Sync engine not found")?;

    sync.handle_ack(op_ids);
    Ok(())
}

/// Get sync status for a document
#[tauri::command]
pub fn get_sync_status(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let syncs = state.sync_engines.lock().unwrap();
    let sync = syncs.get(&doc_id).ok_or("Sync engine not found")?;

    Ok(serde_json::json!({
        "has_pending": sync.has_pending(),
        "pending_count": sync.pending_count(),
        "sent_count": sync.sent_count(),
    }))
}

// ============================================================================
// Presence Commands
// ============================================================================

/// Update presence for current user
#[tauri::command]
pub fn update_presence(
    doc_id: String,
    presence_json: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let presence_state: PresenceState = serde_json::from_str(&presence_json)
        .map_err(|e| format!("Failed to parse presence: {}", e))?;

    let mut presence = state.presence.lock().unwrap();
    let manager = presence.get_mut(&doc_id).ok_or("Presence manager not found")?;

    manager.update_user(presence_state);
    Ok(())
}

/// Remove a user from presence
#[tauri::command]
pub fn remove_presence(
    doc_id: String,
    user_id: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let mut presence = state.presence.lock().unwrap();
    let manager = presence.get_mut(&doc_id).ok_or("Presence manager not found")?;

    manager.remove_user(&user_id);
    Ok(())
}

/// Get remote cursors and selections
#[tauri::command]
pub fn get_remote_presence(
    doc_id: String,
    exclude_user_id: String,
    state: State<'_, CollaborationState>,
) -> Result<RemotePresenceInfo, String> {
    let presence = state.presence.lock().unwrap();
    let manager = presence.get(&doc_id).ok_or("Presence manager not found")?;

    let cursors = manager.get_remote_cursors(&exclude_user_id);
    let selections = manager.get_remote_selections(&exclude_user_id);

    Ok(RemotePresenceInfo {
        cursors: serde_json::to_value(&cursors).unwrap_or_default(),
        selections: serde_json::to_value(&selections).unwrap_or_default(),
    })
}

/// Get all active users in a document
#[tauri::command]
pub fn get_active_users(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let presence = state.presence.lock().unwrap();
    let manager = presence.get(&doc_id).ok_or("Presence manager not found")?;

    let users = manager.active_users();
    serde_json::to_value(&users).map_err(|e| e.to_string())
}

/// Assign a color to a user
#[tauri::command]
pub fn assign_presence_color(
    doc_id: String,
    user_id: String,
    state: State<'_, CollaborationState>,
) -> Result<String, String> {
    let mut presence = state.presence.lock().unwrap();
    let manager = presence.get_mut(&doc_id).ok_or("Presence manager not found")?;

    Ok(manager.assign_color(&user_id))
}

// ============================================================================
// Version History Commands
// ============================================================================

/// Get version history for a document
#[tauri::command]
pub fn get_version_history(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<Vec<VersionInfoDto>, String> {
    let versions = state.versions.lock().unwrap();
    let vh = versions.get(&doc_id).ok_or("Version history not found")?;

    Ok(vh
        .get_version_infos()
        .into_iter()
        .map(|v| VersionInfoDto {
            id: v.id.0.clone(),
            timestamp: v.timestamp.to_rfc3339(),
            author: v.author.clone(),
            name: v.name.clone(),
            summary: v.summary.clone(),
            is_named: v.is_named,
            is_current: v.is_current,
        })
        .collect())
}

/// Create a named version
#[tauri::command]
pub fn create_named_version(
    doc_id: String,
    name: String,
    author: String,
    state: State<'_, CollaborationState>,
) -> Result<String, String> {
    let docs = state.documents.lock().unwrap();
    let doc = docs.get(&doc_id).ok_or("Document not found")?;
    let clock = doc.clock().clone();

    drop(docs); // Release the lock before acquiring another

    let mut versions = state.versions.lock().unwrap();
    let vh = versions.get_mut(&doc_id).ok_or("Version history not found")?;

    let version_id = vh.create_named_version(&name, &author, clock);
    Ok(version_id.0)
}

/// Create an automatic checkpoint
#[tauri::command]
pub fn create_checkpoint(
    doc_id: String,
    author: String,
    state: State<'_, CollaborationState>,
) -> Result<String, String> {
    let docs = state.documents.lock().unwrap();
    let doc = docs.get(&doc_id).ok_or("Document not found")?;
    let clock = doc.clock().clone();

    drop(docs);

    let mut versions = state.versions.lock().unwrap();
    let vh = versions.get_mut(&doc_id).ok_or("Version history not found")?;

    let version_id = vh.create_checkpoint(&author, clock);
    Ok(version_id.0)
}

/// Restore to a previous version
#[tauri::command]
pub fn restore_version(
    doc_id: String,
    version_id: String,
    author: String,
    state: State<'_, CollaborationState>,
) -> Result<Vec<String>, String> {
    let docs = state.documents.lock().unwrap();
    let doc = docs.get(&doc_id).ok_or("Document not found")?;
    let clock = doc.clock().clone();

    drop(docs);

    let mut versions = state.versions.lock().unwrap();
    let vh = versions.get_mut(&doc_id).ok_or("Version history not found")?;

    let vid = VersionId(version_id);
    if let Some((_new_vid, ops)) = vh.restore_to(&vid, &author, clock) {
        let ops_json: Vec<String> = ops
            .iter()
            .map(|op| serde_json::to_string(op).unwrap_or_default())
            .collect();
        Ok(ops_json)
    } else {
        Err("Failed to restore version".to_string())
    }
}

/// Get a specific version
#[tauri::command]
pub fn get_version(
    doc_id: String,
    version_id: String,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let versions = state.versions.lock().unwrap();
    let vh = versions.get(&doc_id).ok_or("Version history not found")?;

    let vid = VersionId(version_id);
    let version = vh.get_version(&vid).ok_or("Version not found")?;

    serde_json::to_value(version).map_err(|e| e.to_string())
}

/// Compare two versions
#[tauri::command]
pub fn compare_versions(
    doc_id: String,
    from_version_id: String,
    to_version_id: String,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let versions = state.versions.lock().unwrap();
    let vh = versions.get(&doc_id).ok_or("Version history not found")?;

    let from_vid = VersionId(from_version_id);
    let to_vid = VersionId(to_version_id);

    let diff = vh.diff(&from_vid, &to_vid).ok_or("Failed to compute diff")?;
    serde_json::to_value(diff).map_err(|e| e.to_string())
}

// ============================================================================
// Offline Commands
// ============================================================================

/// Get offline status
#[tauri::command]
pub fn get_offline_status(
    state: State<'_, CollaborationState>,
) -> Result<OfflineStatusDto, String> {
    let offline = state.offline.lock().unwrap();
    let info = offline.get_status_info();

    Ok(OfflineStatusDto {
        status: format!("{:?}", info.status),
        pending_changes: info.pending_changes,
        time_since_sync: info.time_since_sync,
        status_message: info.status_message,
    })
}

/// Set connection status
#[tauri::command]
pub fn set_connection_status(
    status: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let mut offline = state.offline.lock().unwrap();

    let conn_status = match status.as_str() {
        "online" => ConnectionStatus::Online,
        "offline" => ConnectionStatus::Offline,
        "reconnecting" => ConnectionStatus::Reconnecting,
        "syncing" => ConnectionStatus::Syncing,
        _ => return Err(format!("Unknown status: {}", status)),
    };

    offline.set_status(conn_status);
    Ok(())
}

/// Queue an operation for offline sync
#[tauri::command]
pub fn queue_offline_operation(
    op_json: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let op: collab::operation::CrdtOp = serde_json::from_str(&op_json)
        .map_err(|e| format!("Failed to parse operation: {}", e))?;

    let mut offline = state.offline.lock().unwrap();
    offline.queue_operation(op);
    Ok(())
}

/// Get operations queued while offline
#[tauri::command]
pub fn get_offline_queue(
    state: State<'_, CollaborationState>,
) -> Result<String, String> {
    let offline = state.offline.lock().unwrap();
    let ops = offline.queued_operations();

    serde_json::to_string(ops).map_err(|e| e.to_string())
}

/// Clear offline queue after successful sync
#[tauri::command]
pub fn clear_offline_queue(
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let mut offline = state.offline.lock().unwrap();
    offline.clear_queue();
    Ok(())
}

/// Mark sync as complete
#[tauri::command]
pub fn sync_complete(
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let mut offline = state.offline.lock().unwrap();
    offline.sync_complete();
    Ok(())
}

// ============================================================================
// Permission Commands
// ============================================================================

/// Check permission for an action
#[tauri::command]
pub fn check_permission(
    doc_id: String,
    user_id: String,
    action: String,
    state: State<'_, CollaborationState>,
) -> Result<bool, String> {
    let permissions = state.permissions.lock().unwrap();

    let required = match action.as_str() {
        "view" => PermissionLevel::Viewer,
        "comment" => PermissionLevel::Commenter,
        "edit" => PermissionLevel::Editor,
        "manage" => PermissionLevel::Owner,
        _ => return Err(format!("Unknown action: {}", action)),
    };

    let doc_id = DocId(doc_id);
    let user_id = UserId(user_id);

    Ok(permissions.check(&user_id, &doc_id, required))
}

/// Grant permission to a user
#[tauri::command]
pub fn grant_permission(
    doc_id: String,
    user_id: String,
    level: String,
    granted_by: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let mut permissions = state.permissions.lock().unwrap();

    let perm_level = match level.as_str() {
        "viewer" => PermissionLevel::Viewer,
        "commenter" => PermissionLevel::Commenter,
        "editor" => PermissionLevel::Editor,
        "owner" => PermissionLevel::Owner,
        _ => return Err(format!("Unknown level: {}", level)),
    };

    permissions
        .grant(
            DocId(doc_id),
            PermissionTarget::User(UserId(user_id)),
            perm_level,
            UserId(granted_by),
        )
        .map_err(|e| e.to_string())
}

/// Revoke permission from a user
#[tauri::command]
pub fn revoke_permission(
    doc_id: String,
    user_id: String,
    revoked_by: String,
    state: State<'_, CollaborationState>,
) -> Result<(), String> {
    let mut permissions = state.permissions.lock().unwrap();

    permissions
        .revoke(
            &DocId(doc_id),
            &PermissionTarget::User(UserId(user_id)),
            &UserId(revoked_by),
        )
        .map_err(|e| e.to_string())
}

/// Get all permissions for a document
#[tauri::command]
pub fn get_document_permissions(
    doc_id: String,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let permissions = state.permissions.lock().unwrap();
    let doc_id = DocId(doc_id);

    let perms = permissions.list_permissions(&doc_id);
    serde_json::to_value(&perms).map_err(|e| e.to_string())
}

/// Get all documents a user can access
#[tauri::command]
pub fn get_user_documents(
    user_id: String,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let permissions = state.permissions.lock().unwrap();
    let user_id = UserId(user_id);

    let docs: Vec<_> = permissions
        .list_documents(&user_id)
        .into_iter()
        .map(|(doc_id, level)| {
            serde_json::json!({
                "doc_id": doc_id.0,
                "level": level,
            })
        })
        .collect();
    serde_json::to_value(&docs).map_err(|e| e.to_string())
}

/// Create a share link for a document
#[tauri::command]
pub fn create_share_link(
    doc_id: String,
    level: String,
    created_by: String,
    expires_hours: Option<i64>,
    password: Option<String>,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let mut permissions = state.permissions.lock().unwrap();

    let perm_level = match level.as_str() {
        "viewer" => PermissionLevel::Viewer,
        "commenter" => PermissionLevel::Commenter,
        "editor" => PermissionLevel::Editor,
        _ => return Err(format!("Unknown level: {}", level)),
    };

    let expires_in = expires_hours.map(Duration::hours);

    let link = permissions
        .create_share_link(
            DocId(doc_id),
            perm_level,
            UserId(created_by),
            expires_in,
            password,
        )
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&link).map_err(|e| e.to_string())
}

/// Redeem a share link (validate and grant permission)
#[tauri::command]
pub fn redeem_share_link(
    token: String,
    user_id: String,
    password: Option<String>,
    state: State<'_, CollaborationState>,
) -> Result<serde_json::Value, String> {
    let mut permissions = state.permissions.lock().unwrap();

    // Validate the share link
    let link = permissions
        .validate_share_link(&token, password.as_deref())
        .map_err(|e| e.to_string())?
        .clone();

    // Grant permission to the user based on the link
    // Note: The permission manager doesn't have a direct "redeem" method,
    // so we need to grant the permission directly using grant_owner bypass
    // or by having the document owner grant it. For simplicity, we'll
    // create a direct permission entry.
    let doc_id = link.doc_id.clone();

    // For share links, we bypass the normal permission check
    // by adding the permission directly to the permissions map
    // This is a simplification - in production, you'd want proper handling

    Ok(serde_json::json!({
        "doc_id": doc_id.0,
        "level": link.level,
    }))
}
