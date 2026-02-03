//! Permission management for plugins
//!
//! This module handles granting, revoking, and checking permissions for plugins.

use crate::manifest::Permission;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Manages permissions for all plugins
#[derive(Debug, Default)]
pub struct PermissionManager {
    /// Permissions that have been granted to plugins
    granted: HashMap<String, HashSet<Permission>>,
    /// Permissions that are pending approval
    pending: HashMap<String, Vec<Permission>>,
    /// Permissions that have been explicitly denied
    denied: HashMap<String, HashSet<Permission>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a plugin has a specific permission
    pub fn check_permission(&self, plugin_id: &str, permission: Permission) -> bool {
        self.granted
            .get(plugin_id)
            .map(|perms| perms.contains(&permission))
            .unwrap_or(false)
    }

    /// Check if a plugin has all the given permissions
    pub fn check_permissions(&self, plugin_id: &str, permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .all(|p| self.check_permission(plugin_id, *p))
    }

    /// Grant a permission to a plugin
    pub fn grant_permission(&mut self, plugin_id: &str, permission: Permission) {
        self.granted
            .entry(plugin_id.to_string())
            .or_default()
            .insert(permission);

        // Remove from pending if it was there
        if let Some(pending) = self.pending.get_mut(plugin_id) {
            pending.retain(|p| p != &permission);
        }

        // Remove from denied if it was there
        if let Some(denied) = self.denied.get_mut(plugin_id) {
            denied.remove(&permission);
        }
    }

    /// Grant multiple permissions to a plugin
    pub fn grant_permissions(&mut self, plugin_id: &str, permissions: &[Permission]) {
        for permission in permissions {
            self.grant_permission(plugin_id, *permission);
        }
    }

    /// Revoke a permission from a plugin
    pub fn revoke_permission(&mut self, plugin_id: &str, permission: Permission) {
        if let Some(perms) = self.granted.get_mut(plugin_id) {
            perms.remove(&permission);
        }
    }

    /// Revoke all permissions from a plugin
    pub fn revoke_all_permissions(&mut self, plugin_id: &str) {
        self.granted.remove(plugin_id);
        self.pending.remove(plugin_id);
        self.denied.remove(plugin_id);
    }

    /// Deny a permission for a plugin
    pub fn deny_permission(&mut self, plugin_id: &str, permission: Permission) {
        self.denied
            .entry(plugin_id.to_string())
            .or_default()
            .insert(permission);

        // Remove from pending if it was there
        if let Some(pending) = self.pending.get_mut(plugin_id) {
            pending.retain(|p| p != &permission);
        }
    }

    /// Check if a permission has been explicitly denied
    pub fn is_denied(&self, plugin_id: &str, permission: Permission) -> bool {
        self.denied
            .get(plugin_id)
            .map(|perms| perms.contains(&permission))
            .unwrap_or(false)
    }

    /// Add permissions to the pending list for approval
    pub fn request_permissions(&mut self, plugin_id: &str, permissions: Vec<Permission>) {
        // Filter permissions that should be added to pending
        let to_add: Vec<Permission> = permissions
            .into_iter()
            .filter(|permission| {
                !self.check_permission(plugin_id, *permission)
                    && !self.is_denied(plugin_id, *permission)
            })
            .collect();

        let pending = self.pending.entry(plugin_id.to_string()).or_default();
        for permission in to_add {
            if !pending.contains(&permission) {
                pending.push(permission);
            }
        }
    }

    /// Get all pending permission requests for a plugin
    pub fn get_pending_permissions(&self, plugin_id: &str) -> Vec<Permission> {
        self.pending.get(plugin_id).cloned().unwrap_or_default()
    }

    /// Get all granted permissions for a plugin
    pub fn get_granted_permissions(&self, plugin_id: &str) -> Vec<Permission> {
        self.granted
            .get(plugin_id)
            .map(|perms| perms.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get permissions that a plugin requires but doesn't have
    pub fn get_missing_permissions(
        &self,
        plugin_id: &str,
        required: &[Permission],
    ) -> Vec<Permission> {
        required
            .iter()
            .filter(|p| !self.check_permission(plugin_id, **p))
            .copied()
            .collect()
    }

    /// Check if all required permissions are granted
    pub fn has_all_required(&self, plugin_id: &str, required: &[Permission]) -> bool {
        self.get_missing_permissions(plugin_id, required).is_empty()
    }

    /// Get a summary of all plugins and their permissions
    pub fn get_all_grants(&self) -> HashMap<String, Vec<Permission>> {
        self.granted
            .iter()
            .map(|(id, perms)| (id.clone(), perms.iter().copied().collect()))
            .collect()
    }

    /// Export permission state for persistence
    pub fn export(&self) -> PermissionState {
        PermissionState {
            granted: self
                .granted
                .iter()
                .map(|(id, perms)| (id.clone(), perms.iter().copied().collect()))
                .collect(),
            denied: self
                .denied
                .iter()
                .map(|(id, perms)| (id.clone(), perms.iter().copied().collect()))
                .collect(),
        }
    }

    /// Import permission state from persistence
    pub fn import(&mut self, state: PermissionState) {
        for (id, perms) in state.granted {
            let entry = self.granted.entry(id).or_default();
            for perm in perms {
                entry.insert(perm);
            }
        }
        for (id, perms) in state.denied {
            let entry = self.denied.entry(id).or_default();
            for perm in perms {
                entry.insert(perm);
            }
        }
    }
}

/// Serializable permission state for persistence
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionState {
    /// Granted permissions by plugin ID
    pub granted: HashMap<String, Vec<Permission>>,
    /// Denied permissions by plugin ID
    pub denied: HashMap<String, Vec<Permission>>,
}

/// Information about a permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Plugin requesting the permission
    pub plugin_id: String,
    /// Plugin display name
    pub plugin_name: String,
    /// Requested permissions
    pub permissions: Vec<Permission>,
    /// Reason for the request
    pub reason: Option<String>,
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(
        plugin_id: impl Into<String>,
        plugin_name: impl Into<String>,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            plugin_name: plugin_name.into(),
            permissions,
            reason: None,
        }
    }

    /// Add a reason for the request
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Get sensitive permissions from the request
    pub fn get_sensitive_permissions(&self) -> Vec<Permission> {
        self.permissions
            .iter()
            .filter(|p| p.is_sensitive())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_manager_new() {
        let manager = PermissionManager::new();
        assert!(manager.granted.is_empty());
        assert!(manager.pending.is_empty());
        assert!(manager.denied.is_empty());
    }

    #[test]
    fn test_grant_permission() {
        let mut manager = PermissionManager::new();
        manager.grant_permission("test.plugin", Permission::DocumentRead);

        assert!(manager.check_permission("test.plugin", Permission::DocumentRead));
        assert!(!manager.check_permission("test.plugin", Permission::DocumentWrite));
    }

    #[test]
    fn test_grant_multiple_permissions() {
        let mut manager = PermissionManager::new();
        manager.grant_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::DocumentWrite],
        );

        assert!(manager.check_permission("test.plugin", Permission::DocumentRead));
        assert!(manager.check_permission("test.plugin", Permission::DocumentWrite));
    }

    #[test]
    fn test_check_permissions() {
        let mut manager = PermissionManager::new();
        manager.grant_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::UiToolbar],
        );

        assert!(manager.check_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::UiToolbar]
        ));
        assert!(!manager.check_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::Network]
        ));
    }

    #[test]
    fn test_revoke_permission() {
        let mut manager = PermissionManager::new();
        manager.grant_permission("test.plugin", Permission::DocumentRead);
        manager.revoke_permission("test.plugin", Permission::DocumentRead);

        assert!(!manager.check_permission("test.plugin", Permission::DocumentRead));
    }

    #[test]
    fn test_revoke_all_permissions() {
        let mut manager = PermissionManager::new();
        manager.grant_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::DocumentWrite],
        );
        manager.revoke_all_permissions("test.plugin");

        assert!(!manager.check_permission("test.plugin", Permission::DocumentRead));
        assert!(!manager.check_permission("test.plugin", Permission::DocumentWrite));
    }

    #[test]
    fn test_deny_permission() {
        let mut manager = PermissionManager::new();
        manager.deny_permission("test.plugin", Permission::Network);

        assert!(manager.is_denied("test.plugin", Permission::Network));
        assert!(!manager.check_permission("test.plugin", Permission::Network));
    }

    #[test]
    fn test_request_permissions() {
        let mut manager = PermissionManager::new();
        manager.request_permissions(
            "test.plugin",
            vec![Permission::DocumentRead, Permission::Network],
        );

        let pending = manager.get_pending_permissions("test.plugin");
        assert_eq!(pending.len(), 2);
        assert!(pending.contains(&Permission::DocumentRead));
        assert!(pending.contains(&Permission::Network));
    }

    #[test]
    fn test_request_permissions_excludes_granted() {
        let mut manager = PermissionManager::new();
        manager.grant_permission("test.plugin", Permission::DocumentRead);
        manager.request_permissions(
            "test.plugin",
            vec![Permission::DocumentRead, Permission::Network],
        );

        let pending = manager.get_pending_permissions("test.plugin");
        assert_eq!(pending.len(), 1);
        assert!(pending.contains(&Permission::Network));
    }

    #[test]
    fn test_request_permissions_excludes_denied() {
        let mut manager = PermissionManager::new();
        manager.deny_permission("test.plugin", Permission::Network);
        manager.request_permissions(
            "test.plugin",
            vec![Permission::DocumentRead, Permission::Network],
        );

        let pending = manager.get_pending_permissions("test.plugin");
        assert_eq!(pending.len(), 1);
        assert!(pending.contains(&Permission::DocumentRead));
    }

    #[test]
    fn test_grant_removes_from_pending() {
        let mut manager = PermissionManager::new();
        manager.request_permissions("test.plugin", vec![Permission::DocumentRead]);
        manager.grant_permission("test.plugin", Permission::DocumentRead);

        let pending = manager.get_pending_permissions("test.plugin");
        assert!(pending.is_empty());
    }

    #[test]
    fn test_get_granted_permissions() {
        let mut manager = PermissionManager::new();
        manager.grant_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::UiToolbar],
        );

        let granted = manager.get_granted_permissions("test.plugin");
        assert_eq!(granted.len(), 2);
    }

    #[test]
    fn test_get_missing_permissions() {
        let mut manager = PermissionManager::new();
        manager.grant_permission("test.plugin", Permission::DocumentRead);

        let required = vec![
            Permission::DocumentRead,
            Permission::DocumentWrite,
            Permission::Network,
        ];
        let missing = manager.get_missing_permissions("test.plugin", &required);

        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&Permission::DocumentWrite));
        assert!(missing.contains(&Permission::Network));
    }

    #[test]
    fn test_has_all_required() {
        let mut manager = PermissionManager::new();
        manager.grant_permissions(
            "test.plugin",
            &[Permission::DocumentRead, Permission::DocumentWrite],
        );

        assert!(manager.has_all_required(
            "test.plugin",
            &[Permission::DocumentRead, Permission::DocumentWrite]
        ));
        assert!(!manager.has_all_required(
            "test.plugin",
            &[Permission::DocumentRead, Permission::Network]
        ));
    }

    #[test]
    fn test_export_import() {
        let mut manager = PermissionManager::new();
        manager.grant_permission("test.plugin", Permission::DocumentRead);
        manager.deny_permission("test.plugin", Permission::Network);

        let state = manager.export();

        let mut new_manager = PermissionManager::new();
        new_manager.import(state);

        assert!(new_manager.check_permission("test.plugin", Permission::DocumentRead));
        assert!(new_manager.is_denied("test.plugin", Permission::Network));
    }

    #[test]
    fn test_permission_state_serialization() {
        let mut state = PermissionState::default();
        state
            .granted
            .insert("test.plugin".to_string(), vec![Permission::DocumentRead]);

        let json = serde_json::to_string(&state).unwrap();
        let parsed: PermissionState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.granted.get("test.plugin").unwrap().len(), 1);
    }

    #[test]
    fn test_permission_request() {
        let request = PermissionRequest::new(
            "com.example.plugin",
            "Example Plugin",
            vec![Permission::Network, Permission::Storage],
        )
        .with_reason("Needed to sync data");

        assert_eq!(request.plugin_id, "com.example.plugin");
        assert_eq!(request.plugin_name, "Example Plugin");
        assert_eq!(request.permissions.len(), 2);
        assert_eq!(request.reason, Some("Needed to sync data".to_string()));
    }

    #[test]
    fn test_permission_request_sensitive() {
        let request = PermissionRequest::new(
            "test",
            "Test",
            vec![
                Permission::DocumentRead,
                Permission::Network,
                Permission::Clipboard,
            ],
        );

        let sensitive = request.get_sensitive_permissions();
        assert_eq!(sensitive.len(), 2);
        assert!(sensitive.contains(&Permission::Network));
        assert!(sensitive.contains(&Permission::Clipboard));
    }

    #[test]
    fn test_get_all_grants() {
        let mut manager = PermissionManager::new();
        manager.grant_permission("plugin.a", Permission::DocumentRead);
        manager.grant_permission("plugin.b", Permission::Network);

        let grants = manager.get_all_grants();
        assert_eq!(grants.len(), 2);
        assert!(grants.contains_key("plugin.a"));
        assert!(grants.contains_key("plugin.b"));
    }
}
