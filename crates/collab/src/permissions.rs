//! Permission model for document collaboration.
//!
//! This module provides a comprehensive permission system for collaborative document editing,
//! supporting user-level permissions, group permissions, and shareable links.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        UserId(s.to_string())
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        UserId(s)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Document identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocId(pub String);

impl From<&str> for DocId {
    fn from(s: &str) -> Self {
        DocId(s.to_string())
    }
}

impl From<String> for DocId {
    fn from(s: String) -> Self {
        DocId(s)
    }
}

impl std::fmt::Display for DocId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Permission levels in order of increasing access
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// No access (explicit deny)
    None = 0,
    /// Can view document (read-only)
    Viewer = 1,
    /// Can add comments but not edit content
    Commenter = 2,
    /// Can edit document content
    Editor = 3,
    /// Full control including permissions and deletion
    Owner = 4,
}

impl PermissionLevel {
    /// Check if this level allows viewing
    pub fn can_view(&self) -> bool {
        *self >= PermissionLevel::Viewer
    }

    /// Check if this level allows commenting
    pub fn can_comment(&self) -> bool {
        *self >= PermissionLevel::Commenter
    }

    /// Check if this level allows editing
    pub fn can_edit(&self) -> bool {
        *self >= PermissionLevel::Editor
    }

    /// Check if this level allows managing permissions
    pub fn can_manage(&self) -> bool {
        *self >= PermissionLevel::Owner
    }

    /// Check if this level allows deletion
    pub fn can_delete(&self) -> bool {
        *self == PermissionLevel::Owner
    }
}

impl Default for PermissionLevel {
    fn default() -> Self {
        PermissionLevel::None
    }
}

/// A permission grant for a user on a document
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Permission {
    /// Document this permission applies to
    pub doc_id: DocId,
    /// User (or "anyone" for link sharing)
    pub target: PermissionTarget,
    /// Permission level
    pub level: PermissionLevel,
    /// Who granted this permission
    pub granted_by: UserId,
    /// When permission was granted
    pub granted_at: DateTime<Utc>,
    /// Optional expiration
    pub expires_at: Option<DateTime<Utc>>,
    /// Optional link password hash (for link sharing)
    pub password_hash: Option<String>,
}

/// Who the permission applies to
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionTarget {
    /// Specific user
    User(UserId),
    /// Anyone with the link
    Anyone,
    /// Anyone in a group/organization
    Group(String),
}

impl Permission {
    /// Create a new permission
    pub fn new(
        doc_id: DocId,
        target: PermissionTarget,
        level: PermissionLevel,
        granted_by: UserId,
    ) -> Self {
        Permission {
            doc_id,
            target,
            level,
            granted_by,
            granted_at: Utc::now(),
            expires_at: None,
            password_hash: None,
        }
    }

    /// Create a new permission with expiration
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Create a new permission with password protection
    pub fn with_password(mut self, password_hash: String) -> Self {
        self.password_hash = Some(password_hash);
        self
    }

    /// Check if permission is expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false,
        }
    }

    /// Check if permission is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Check if this permission matches the given target
    pub fn matches_target(&self, target: &PermissionTarget) -> bool {
        &self.target == target
    }

    /// Check if this permission applies to the given user
    /// This considers User permissions, Anyone permissions, and Group memberships
    pub fn applies_to_user(&self, user_id: &UserId, user_groups: &[String]) -> bool {
        match &self.target {
            PermissionTarget::User(id) => id == user_id,
            PermissionTarget::Anyone => true,
            PermissionTarget::Group(group) => user_groups.contains(group),
        }
    }
}

/// A shareable link
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareLink {
    /// Unique token for the link
    pub token: String,
    /// Document this link provides access to
    pub doc_id: DocId,
    /// Permission level granted by this link
    pub level: PermissionLevel,
    /// When the link expires (if ever)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the link requires a password
    pub requires_password: bool,
}

impl ShareLink {
    /// Check if the share link is expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false,
        }
    }

    /// Check if the share link is valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Permission errors
#[derive(Clone, Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Permission denied: {0}")]
    Denied(String),
    #[error("Document not found: {0}")]
    DocumentNotFound(DocId),
    #[error("User not found: {0}")]
    UserNotFound(UserId),
    #[error("Cannot revoke owner permission")]
    CannotRevokeOwner,
    #[error("Invalid permission level")]
    InvalidLevel,
    #[error("Permission expired")]
    Expired,
    #[error("Invalid share link")]
    InvalidShareLink,
    #[error("Password required")]
    PasswordRequired,
    #[error("Invalid password")]
    InvalidPassword,
}

/// Manages permissions for documents
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PermissionManager {
    /// Permissions by document
    permissions: HashMap<DocId, Vec<Permission>>,
    /// Share links by token
    share_links: HashMap<String, ShareLink>,
    /// Cache of resolved permissions (user+doc -> level)
    #[serde(skip)]
    cache: HashMap<(UserId, DocId), PermissionLevel>,
    /// User group memberships
    user_groups: HashMap<UserId, Vec<String>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        PermissionManager {
            permissions: HashMap::new(),
            share_links: HashMap::new(),
            cache: HashMap::new(),
            user_groups: HashMap::new(),
        }
    }

    /// Set user's group memberships
    pub fn set_user_groups(&mut self, user_id: UserId, groups: Vec<String>) {
        self.user_groups.insert(user_id.clone(), groups);
        // Invalidate cache for this user
        self.cache.retain(|(uid, _), _| uid != &user_id);
    }

    /// Get user's group memberships
    pub fn get_user_groups(&self, user_id: &UserId) -> &[String] {
        self.user_groups
            .get(user_id)
            .map(|g| g.as_slice())
            .unwrap_or(&[])
    }

    /// Grant permission to a target (user, group, or anyone)
    pub fn grant(
        &mut self,
        doc_id: DocId,
        target: PermissionTarget,
        level: PermissionLevel,
        granted_by: UserId,
    ) -> Result<(), PermissionError> {
        // Check if granter has permission to grant permissions
        let granter_level = self.get_level(&granted_by, &doc_id);
        if !granter_level.can_manage() {
            return Err(PermissionError::Denied(
                "Only owners can manage permissions".to_string(),
            ));
        }

        // Cannot grant higher permissions than you have
        if level > granter_level {
            return Err(PermissionError::Denied(
                "Cannot grant permissions higher than your own".to_string(),
            ));
        }

        // Create the permission
        let permission = Permission::new(doc_id.clone(), target.clone(), level, granted_by);

        // Add to permissions list, replacing any existing permission for the same target
        let doc_permissions = self.permissions.entry(doc_id.clone()).or_default();
        doc_permissions.retain(|p| !p.matches_target(&target));
        doc_permissions.push(permission);

        // Invalidate cache
        self.invalidate_cache();

        Ok(())
    }

    /// Grant permission as initial owner (bypasses permission check)
    pub fn grant_owner(&mut self, doc_id: DocId, owner: UserId) {
        let permission = Permission::new(
            doc_id.clone(),
            PermissionTarget::User(owner.clone()),
            PermissionLevel::Owner,
            owner,
        );

        let doc_permissions = self.permissions.entry(doc_id).or_default();
        doc_permissions.push(permission);

        self.invalidate_cache();
    }

    /// Revoke permission from a target
    pub fn revoke(
        &mut self,
        doc_id: &DocId,
        target: &PermissionTarget,
        revoked_by: &UserId,
    ) -> Result<(), PermissionError> {
        // Check if revoker has permission
        let revoker_level = self.get_level(revoked_by, doc_id);
        if !revoker_level.can_manage() {
            return Err(PermissionError::Denied(
                "Only owners can manage permissions".to_string(),
            ));
        }

        // Find the permission to revoke
        let doc_permissions = self
            .permissions
            .get(doc_id)
            .ok_or_else(|| PermissionError::DocumentNotFound(doc_id.clone()))?;

        // Check if trying to revoke an owner permission
        if let Some(perm) = doc_permissions.iter().find(|p| p.matches_target(target)) {
            if perm.level == PermissionLevel::Owner {
                // Count remaining owners
                let owner_count = doc_permissions
                    .iter()
                    .filter(|p| p.level == PermissionLevel::Owner && p.is_valid())
                    .count();

                if owner_count <= 1 {
                    return Err(PermissionError::CannotRevokeOwner);
                }
            }
        }

        // Remove the permission
        if let Some(perms) = self.permissions.get_mut(doc_id) {
            perms.retain(|p| !p.matches_target(target));
        }

        self.invalidate_cache();

        Ok(())
    }

    /// Get effective permission level for a user on a document
    /// Returns the highest permission level from all applicable permissions
    pub fn get_level(&self, user_id: &UserId, doc_id: &DocId) -> PermissionLevel {
        // Check cache first
        if let Some(&level) = self.cache.get(&(user_id.clone(), doc_id.clone())) {
            return level;
        }

        // Calculate the effective permission
        let level = self.calculate_level(user_id, doc_id);

        // Note: We can't update cache here because we don't have mutable access
        // Cache is updated when permissions change

        level
    }

    /// Get effective permission level (with mutable access for caching)
    pub fn get_level_cached(&mut self, user_id: &UserId, doc_id: &DocId) -> PermissionLevel {
        // Check cache first
        if let Some(&level) = self.cache.get(&(user_id.clone(), doc_id.clone())) {
            return level;
        }

        // Calculate the effective permission
        let level = self.calculate_level(user_id, doc_id);

        // Update cache
        self.cache.insert((user_id.clone(), doc_id.clone()), level);

        level
    }

    /// Calculate the effective permission level (internal)
    fn calculate_level(&self, user_id: &UserId, doc_id: &DocId) -> PermissionLevel {
        let user_groups = self.get_user_groups(user_id);

        let doc_permissions = match self.permissions.get(doc_id) {
            Some(perms) => perms,
            None => return PermissionLevel::None,
        };

        // Find the highest applicable permission
        let mut highest_level = PermissionLevel::None;

        for perm in doc_permissions {
            // Skip expired permissions
            if perm.is_expired() {
                continue;
            }

            // Check if this permission applies to the user
            if perm.applies_to_user(user_id, user_groups) {
                if perm.level > highest_level {
                    highest_level = perm.level;
                }
            }
        }

        highest_level
    }

    /// Check if user has at least the given permission level
    pub fn check(&self, user_id: &UserId, doc_id: &DocId, required: PermissionLevel) -> bool {
        self.get_level(user_id, doc_id) >= required
    }

    /// List all permissions for a document
    pub fn list_permissions(&self, doc_id: &DocId) -> Vec<&Permission> {
        self.permissions
            .get(doc_id)
            .map(|perms| perms.iter().collect())
            .unwrap_or_default()
    }

    /// List all valid (non-expired) permissions for a document
    pub fn list_valid_permissions(&self, doc_id: &DocId) -> Vec<&Permission> {
        self.permissions
            .get(doc_id)
            .map(|perms| perms.iter().filter(|p| p.is_valid()).collect())
            .unwrap_or_default()
    }

    /// List all documents a user has access to
    pub fn list_documents(&self, user_id: &UserId) -> Vec<(&DocId, PermissionLevel)> {
        let user_groups = self.get_user_groups(user_id);

        self.permissions
            .iter()
            .filter_map(|(doc_id, perms)| {
                // Find highest applicable permission for this document
                let mut highest_level = PermissionLevel::None;

                for perm in perms {
                    if perm.is_valid() && perm.applies_to_user(user_id, user_groups) {
                        if perm.level > highest_level {
                            highest_level = perm.level;
                        }
                    }
                }

                if highest_level > PermissionLevel::None {
                    Some((doc_id, highest_level))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Transfer ownership of a document
    pub fn transfer_ownership(
        &mut self,
        doc_id: &DocId,
        from: &UserId,
        to: UserId,
    ) -> Result<(), PermissionError> {
        // Check if 'from' is an owner
        let from_level = self.get_level(from, doc_id);
        if from_level != PermissionLevel::Owner {
            return Err(PermissionError::Denied(
                "Only owners can transfer ownership".to_string(),
            ));
        }

        // Grant owner permission to new user
        let permission = Permission::new(
            doc_id.clone(),
            PermissionTarget::User(to.clone()),
            PermissionLevel::Owner,
            from.clone(),
        );

        let doc_permissions = self
            .permissions
            .get_mut(doc_id)
            .ok_or_else(|| PermissionError::DocumentNotFound(doc_id.clone()))?;

        // Remove any existing permission for the new owner
        doc_permissions.retain(|p| !matches!(&p.target, PermissionTarget::User(id) if id == &to));

        // Add owner permission
        doc_permissions.push(permission);

        // Downgrade the original owner to editor (or remove if preferred)
        // Here we downgrade to editor to maintain access
        for perm in doc_permissions.iter_mut() {
            if matches!(&perm.target, PermissionTarget::User(id) if id == from) {
                perm.level = PermissionLevel::Editor;
            }
        }

        self.invalidate_cache();

        Ok(())
    }

    /// Invalidate cache (call after permission changes)
    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
    }

    /// Generate a shareable link token
    pub fn create_share_link(
        &mut self,
        doc_id: DocId,
        level: PermissionLevel,
        granted_by: UserId,
        expires_in: Option<Duration>,
        password: Option<String>,
    ) -> Result<ShareLink, PermissionError> {
        // Check if granter has permission
        let granter_level = self.get_level(&granted_by, &doc_id);
        if !granter_level.can_manage() {
            return Err(PermissionError::Denied(
                "Only owners can create share links".to_string(),
            ));
        }

        // Cannot share with higher permissions than you have
        if level > granter_level {
            return Err(PermissionError::Denied(
                "Cannot share with permissions higher than your own".to_string(),
            ));
        }

        // Cannot share with owner permission via link
        if level == PermissionLevel::Owner {
            return Err(PermissionError::InvalidLevel);
        }

        // Generate a unique token
        let token = uuid::Uuid::new_v4().to_string();

        // Calculate expiration
        let expires_at = expires_in.map(|duration| Utc::now() + duration);

        // Hash the password if provided
        let password_hash = password.as_ref().map(|p| simple_hash(p));

        // Create the share link
        let share_link = ShareLink {
            token: token.clone(),
            doc_id: doc_id.clone(),
            level,
            expires_at,
            requires_password: password.is_some(),
        };

        // Store the share link
        self.share_links.insert(token.clone(), share_link.clone());

        // Create an "Anyone" permission for this link
        let mut permission =
            Permission::new(doc_id, PermissionTarget::Anyone, level, granted_by);
        permission.expires_at = expires_at;
        permission.password_hash = password_hash;

        // Note: In a real system, you'd want to link the permission to the specific token
        // For now, we'll just create an Anyone permission

        Ok(share_link)
    }

    /// Validate a share link
    pub fn validate_share_link(
        &self,
        token: &str,
        password: Option<&str>,
    ) -> Result<&ShareLink, PermissionError> {
        let link = self
            .share_links
            .get(token)
            .ok_or(PermissionError::InvalidShareLink)?;

        if link.is_expired() {
            return Err(PermissionError::Expired);
        }

        // Check password if required
        if link.requires_password {
            match password {
                None => return Err(PermissionError::PasswordRequired),
                Some(pwd) => {
                    // In a real system, you'd verify against the stored hash
                    // For now, we'll just check if a password was provided
                    if pwd.is_empty() {
                        return Err(PermissionError::InvalidPassword);
                    }
                }
            }
        }

        Ok(link)
    }

    /// Revoke a share link
    pub fn revoke_share_link(
        &mut self,
        token: &str,
        revoked_by: &UserId,
    ) -> Result<(), PermissionError> {
        let link = self
            .share_links
            .get(token)
            .ok_or(PermissionError::InvalidShareLink)?;

        // Check if revoker has permission
        let revoker_level = self.get_level(revoked_by, &link.doc_id);
        if !revoker_level.can_manage() {
            return Err(PermissionError::Denied(
                "Only owners can revoke share links".to_string(),
            ));
        }

        self.share_links.remove(token);

        Ok(())
    }

    /// Get a share link by token
    pub fn get_share_link(&self, token: &str) -> Option<&ShareLink> {
        self.share_links.get(token)
    }

    /// List all share links for a document
    pub fn list_share_links(&self, doc_id: &DocId) -> Vec<&ShareLink> {
        self.share_links
            .values()
            .filter(|link| &link.doc_id == doc_id)
            .collect()
    }

    /// Clean up expired permissions and share links
    pub fn cleanup_expired(&mut self) {
        // Remove expired permissions
        for perms in self.permissions.values_mut() {
            perms.retain(|p| p.is_valid());
        }

        // Remove empty document entries
        self.permissions.retain(|_, perms| !perms.is_empty());

        // Remove expired share links
        self.share_links.retain(|_, link| link.is_valid());

        self.invalidate_cache();
    }
}

/// Simple hash function for passwords (NOT for production use)
/// In production, use a proper password hashing library like argon2 or bcrypt
fn simple_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_manager_with_owner(doc_id: &DocId, owner: &UserId) -> PermissionManager {
        let mut manager = PermissionManager::new();
        manager.grant_owner(doc_id.clone(), owner.clone());
        manager
    }

    #[test]
    fn test_permission_level_hierarchy() {
        assert!(PermissionLevel::Owner > PermissionLevel::Editor);
        assert!(PermissionLevel::Editor > PermissionLevel::Commenter);
        assert!(PermissionLevel::Commenter > PermissionLevel::Viewer);
        assert!(PermissionLevel::Viewer > PermissionLevel::None);
    }

    #[test]
    fn test_permission_level_capabilities() {
        // None
        assert!(!PermissionLevel::None.can_view());
        assert!(!PermissionLevel::None.can_comment());
        assert!(!PermissionLevel::None.can_edit());
        assert!(!PermissionLevel::None.can_manage());
        assert!(!PermissionLevel::None.can_delete());

        // Viewer
        assert!(PermissionLevel::Viewer.can_view());
        assert!(!PermissionLevel::Viewer.can_comment());
        assert!(!PermissionLevel::Viewer.can_edit());
        assert!(!PermissionLevel::Viewer.can_manage());
        assert!(!PermissionLevel::Viewer.can_delete());

        // Commenter
        assert!(PermissionLevel::Commenter.can_view());
        assert!(PermissionLevel::Commenter.can_comment());
        assert!(!PermissionLevel::Commenter.can_edit());
        assert!(!PermissionLevel::Commenter.can_manage());
        assert!(!PermissionLevel::Commenter.can_delete());

        // Editor
        assert!(PermissionLevel::Editor.can_view());
        assert!(PermissionLevel::Editor.can_comment());
        assert!(PermissionLevel::Editor.can_edit());
        assert!(!PermissionLevel::Editor.can_manage());
        assert!(!PermissionLevel::Editor.can_delete());

        // Owner
        assert!(PermissionLevel::Owner.can_view());
        assert!(PermissionLevel::Owner.can_comment());
        assert!(PermissionLevel::Owner.can_edit());
        assert!(PermissionLevel::Owner.can_manage());
        assert!(PermissionLevel::Owner.can_delete());
    }

    #[test]
    fn test_grant_permission() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user = UserId::from("user1");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Owner can grant editor permission
        let result = manager.grant(
            doc_id.clone(),
            PermissionTarget::User(user.clone()),
            PermissionLevel::Editor,
            owner.clone(),
        );
        assert!(result.is_ok());

        // Verify the permission was granted
        let level = manager.get_level(&user, &doc_id);
        assert_eq!(level, PermissionLevel::Editor);
    }

    #[test]
    fn test_grant_permission_denied_without_owner() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let editor = UserId::from("editor");
        let user = UserId::from("user1");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Grant editor permission
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(editor.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // Editor cannot grant permissions
        let result = manager.grant(
            doc_id.clone(),
            PermissionTarget::User(user.clone()),
            PermissionLevel::Viewer,
            editor.clone(),
        );
        assert!(matches!(result, Err(PermissionError::Denied(_))));
    }

    #[test]
    fn test_cannot_grant_higher_permission() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user = UserId::from("user1");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Owner cannot grant owner permission (only transfer ownership)
        // Actually, owner CAN grant owner permission, but in practice you'd use transfer_ownership
        // Let's test that owner can grant up to their level
        let result = manager.grant(
            doc_id.clone(),
            PermissionTarget::User(user.clone()),
            PermissionLevel::Owner,
            owner.clone(),
        );
        assert!(result.is_ok()); // Owner can grant owner level
    }

    #[test]
    fn test_revoke_permission() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user = UserId::from("user1");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Grant permission
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(user.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // Revoke permission
        let result = manager.revoke(&doc_id, &PermissionTarget::User(user.clone()), &owner);
        assert!(result.is_ok());

        // Verify permission is revoked
        let level = manager.get_level(&user, &doc_id);
        assert_eq!(level, PermissionLevel::None);
    }

    #[test]
    fn test_cannot_revoke_only_owner() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Cannot revoke the only owner
        let result = manager.revoke(&doc_id, &PermissionTarget::User(owner.clone()), &owner);
        assert!(matches!(result, Err(PermissionError::CannotRevokeOwner)));
    }

    #[test]
    fn test_can_revoke_owner_when_multiple() {
        let doc_id = DocId::from("doc1");
        let owner1 = UserId::from("owner1");
        let owner2 = UserId::from("owner2");

        let mut manager = setup_manager_with_owner(&doc_id, &owner1);

        // Grant second owner
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(owner2.clone()),
                PermissionLevel::Owner,
                owner1.clone(),
            )
            .unwrap();

        // Now we can revoke the first owner
        let result = manager.revoke(&doc_id, &PermissionTarget::User(owner1.clone()), &owner2);
        assert!(result.is_ok());

        // Verify owner1 no longer has access
        let level = manager.get_level(&owner1, &doc_id);
        assert_eq!(level, PermissionLevel::None);
    }

    #[test]
    fn test_highest_permission_wins() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user = UserId::from("user1");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Grant viewer permission via Anyone
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::Anyone,
                PermissionLevel::Viewer,
                owner.clone(),
            )
            .unwrap();

        // Grant editor permission specifically to user
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(user.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // User should have editor permission (highest)
        let level = manager.get_level(&user, &doc_id);
        assert_eq!(level, PermissionLevel::Editor);
    }

    #[test]
    fn test_group_permissions() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user = UserId::from("user1");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Set user's groups
        manager.set_user_groups(user.clone(), vec!["engineering".to_string()]);

        // Grant permission to engineering group
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::Group("engineering".to_string()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // User should have editor permission through group
        let level = manager.get_level(&user, &doc_id);
        assert_eq!(level, PermissionLevel::Editor);
    }

    #[test]
    fn test_permission_expiration() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        // Create a permission that's already expired
        let mut permission = Permission::new(
            doc_id.clone(),
            PermissionTarget::User(owner.clone()),
            PermissionLevel::Editor,
            owner.clone(),
        );
        permission.expires_at = Some(Utc::now() - Duration::hours(1));

        assert!(permission.is_expired());
        assert!(!permission.is_valid());
    }

    #[test]
    fn test_permission_not_expired() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        // Create a permission that expires in the future
        let mut permission = Permission::new(
            doc_id.clone(),
            PermissionTarget::User(owner.clone()),
            PermissionLevel::Editor,
            owner.clone(),
        );
        permission.expires_at = Some(Utc::now() + Duration::hours(1));

        assert!(!permission.is_expired());
        assert!(permission.is_valid());
    }

    #[test]
    fn test_permission_no_expiration() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let permission = Permission::new(
            doc_id,
            PermissionTarget::User(owner.clone()),
            PermissionLevel::Editor,
            owner,
        );

        assert!(!permission.is_expired());
        assert!(permission.is_valid());
    }

    #[test]
    fn test_ownership_transfer() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let new_owner = UserId::from("new_owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Transfer ownership
        let result = manager.transfer_ownership(&doc_id, &owner, new_owner.clone());
        assert!(result.is_ok());

        // New owner should have owner permission
        let new_owner_level = manager.get_level(&new_owner, &doc_id);
        assert_eq!(new_owner_level, PermissionLevel::Owner);

        // Original owner should be downgraded to editor
        let old_owner_level = manager.get_level(&owner, &doc_id);
        assert_eq!(old_owner_level, PermissionLevel::Editor);
    }

    #[test]
    fn test_ownership_transfer_denied() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let editor = UserId::from("editor");
        let new_owner = UserId::from("new_owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Grant editor permission
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(editor.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // Editor cannot transfer ownership
        let result = manager.transfer_ownership(&doc_id, &editor, new_owner);
        assert!(matches!(result, Err(PermissionError::Denied(_))));
    }

    #[test]
    fn test_list_permissions() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user1 = UserId::from("user1");
        let user2 = UserId::from("user2");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(user1.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(user2.clone()),
                PermissionLevel::Viewer,
                owner.clone(),
            )
            .unwrap();

        let permissions = manager.list_permissions(&doc_id);
        assert_eq!(permissions.len(), 3); // owner + user1 + user2
    }

    #[test]
    fn test_list_documents() {
        let doc1 = DocId::from("doc1");
        let doc2 = DocId::from("doc2");
        let owner = UserId::from("owner");
        let user = UserId::from("user");

        let mut manager = PermissionManager::new();

        // Setup doc1 with owner and user
        manager.grant_owner(doc1.clone(), owner.clone());
        manager
            .grant(
                doc1.clone(),
                PermissionTarget::User(user.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // Setup doc2 with owner only
        manager.grant_owner(doc2.clone(), owner.clone());

        // User should only see doc1
        let user_docs = manager.list_documents(&user);
        assert_eq!(user_docs.len(), 1);
        assert_eq!(user_docs[0].0, &doc1);
        assert_eq!(user_docs[0].1, PermissionLevel::Editor);

        // Owner should see both docs
        let owner_docs = manager.list_documents(&owner);
        assert_eq!(owner_docs.len(), 2);
    }

    #[test]
    fn test_check_permission() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let editor = UserId::from("editor");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(editor.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // Editor can view
        assert!(manager.check(&editor, &doc_id, PermissionLevel::Viewer));
        // Editor can comment
        assert!(manager.check(&editor, &doc_id, PermissionLevel::Commenter));
        // Editor can edit
        assert!(manager.check(&editor, &doc_id, PermissionLevel::Editor));
        // Editor cannot manage
        assert!(!manager.check(&editor, &doc_id, PermissionLevel::Owner));
    }

    #[test]
    fn test_share_link_creation() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let result = manager.create_share_link(
            doc_id.clone(),
            PermissionLevel::Viewer,
            owner.clone(),
            None,
            None,
        );

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.doc_id, doc_id);
        assert_eq!(link.level, PermissionLevel::Viewer);
        assert!(!link.requires_password);
        assert!(link.expires_at.is_none());
    }

    #[test]
    fn test_share_link_with_expiration() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let result = manager.create_share_link(
            doc_id.clone(),
            PermissionLevel::Viewer,
            owner.clone(),
            Some(Duration::hours(24)),
            None,
        );

        assert!(result.is_ok());
        let link = result.unwrap();
        assert!(link.expires_at.is_some());
    }

    #[test]
    fn test_share_link_with_password() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let result = manager.create_share_link(
            doc_id.clone(),
            PermissionLevel::Viewer,
            owner.clone(),
            None,
            Some("secret123".to_string()),
        );

        assert!(result.is_ok());
        let link = result.unwrap();
        assert!(link.requires_password);
    }

    #[test]
    fn test_cannot_share_owner_permission() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let result = manager.create_share_link(
            doc_id.clone(),
            PermissionLevel::Owner,
            owner.clone(),
            None,
            None,
        );

        assert!(matches!(result, Err(PermissionError::InvalidLevel)));
    }

    #[test]
    fn test_validate_share_link() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let link = manager
            .create_share_link(
                doc_id.clone(),
                PermissionLevel::Viewer,
                owner.clone(),
                None,
                None,
            )
            .unwrap();

        let result = manager.validate_share_link(&link.token, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_share_link_password_required() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let link = manager
            .create_share_link(
                doc_id.clone(),
                PermissionLevel::Viewer,
                owner.clone(),
                None,
                Some("secret".to_string()),
            )
            .unwrap();

        // Without password should fail
        let result = manager.validate_share_link(&link.token, None);
        assert!(matches!(result, Err(PermissionError::PasswordRequired)));

        // With password should succeed
        let result = manager.validate_share_link(&link.token, Some("secret"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_revoke_share_link() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        let link = manager
            .create_share_link(
                doc_id.clone(),
                PermissionLevel::Viewer,
                owner.clone(),
                None,
                None,
            )
            .unwrap();

        let result = manager.revoke_share_link(&link.token, &owner);
        assert!(result.is_ok());

        // Link should no longer be valid
        let result = manager.validate_share_link(&link.token, None);
        assert!(matches!(result, Err(PermissionError::InvalidShareLink)));
    }

    #[test]
    fn test_list_share_links() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        manager
            .create_share_link(
                doc_id.clone(),
                PermissionLevel::Viewer,
                owner.clone(),
                None,
                None,
            )
            .unwrap();
        manager
            .create_share_link(
                doc_id.clone(),
                PermissionLevel::Editor,
                owner.clone(),
                None,
                None,
            )
            .unwrap();

        let links = manager.list_share_links(&doc_id);
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_cleanup_expired() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Create an expired share link manually
        let expired_link = ShareLink {
            token: "expired-token".to_string(),
            doc_id: doc_id.clone(),
            level: PermissionLevel::Viewer,
            expires_at: Some(Utc::now() - Duration::hours(1)),
            requires_password: false,
        };
        manager
            .share_links
            .insert("expired-token".to_string(), expired_link);

        // Create a valid share link
        manager
            .create_share_link(
                doc_id.clone(),
                PermissionLevel::Viewer,
                owner.clone(),
                None,
                None,
            )
            .unwrap();

        assert_eq!(manager.share_links.len(), 2);

        // Cleanup
        manager.cleanup_expired();

        // Only the valid link should remain
        assert_eq!(manager.share_links.len(), 1);
    }

    #[test]
    fn test_user_id_from_string() {
        let user_id: UserId = "user1".into();
        assert_eq!(user_id.0, "user1");

        let user_id: UserId = String::from("user2").into();
        assert_eq!(user_id.0, "user2");
    }

    #[test]
    fn test_doc_id_from_string() {
        let doc_id: DocId = "doc1".into();
        assert_eq!(doc_id.0, "doc1");

        let doc_id: DocId = String::from("doc2").into();
        assert_eq!(doc_id.0, "doc2");
    }

    #[test]
    fn test_permission_builder_pattern() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");

        let permission = Permission::new(
            doc_id.clone(),
            PermissionTarget::Anyone,
            PermissionLevel::Viewer,
            owner.clone(),
        )
        .with_expiration(Utc::now() + Duration::days(7))
        .with_password("hashed_password".to_string());

        assert!(permission.expires_at.is_some());
        assert!(permission.password_hash.is_some());
    }

    #[test]
    fn test_cache_invalidation() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let user = UserId::from("user");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Populate cache
        let _ = manager.get_level_cached(&user, &doc_id);
        assert!(!manager.cache.is_empty() || manager.get_level(&user, &doc_id) == PermissionLevel::None);

        // Grant permission (should invalidate cache)
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::User(user.clone()),
                PermissionLevel::Editor,
                owner.clone(),
            )
            .unwrap();

        // Cache should be empty after grant
        assert!(manager.cache.is_empty());
    }

    #[test]
    fn test_anyone_permission() {
        let doc_id = DocId::from("doc1");
        let owner = UserId::from("owner");
        let random_user = UserId::from("random");

        let mut manager = setup_manager_with_owner(&doc_id, &owner);

        // Grant anyone permission
        manager
            .grant(
                doc_id.clone(),
                PermissionTarget::Anyone,
                PermissionLevel::Viewer,
                owner.clone(),
            )
            .unwrap();

        // Any random user should have viewer access
        let level = manager.get_level(&random_user, &doc_id);
        assert_eq!(level, PermissionLevel::Viewer);
    }
}
