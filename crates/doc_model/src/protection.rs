//! Document Protection Module
//!
//! Provides document-level protection settings including form mode,
//! read-only protection, and editing restrictions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of document protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtectionType {
    /// No protection applied
    None,
    /// Document is read-only (no edits allowed)
    ReadOnly,
    /// Only form fields / content controls can be edited
    FormFieldsOnly,
    /// Only comments can be added
    CommentsOnly,
    /// All changes are tracked (cannot turn off track changes)
    TrackedChangesOnly,
}

impl Default for ProtectionType {
    fn default() -> Self {
        ProtectionType::None
    }
}

impl ProtectionType {
    /// Get the OOXML attribute value for this protection type
    pub fn ooxml_value(&self) -> &'static str {
        match self {
            ProtectionType::None => "none",
            ProtectionType::ReadOnly => "readOnly",
            ProtectionType::FormFieldsOnly => "forms",
            ProtectionType::CommentsOnly => "comments",
            ProtectionType::TrackedChangesOnly => "trackedChanges",
        }
    }

    /// Parse from OOXML attribute value
    pub fn from_ooxml(value: &str) -> Self {
        match value {
            "readOnly" => ProtectionType::ReadOnly,
            "forms" => ProtectionType::FormFieldsOnly,
            "comments" => ProtectionType::CommentsOnly,
            "trackedChanges" => ProtectionType::TrackedChangesOnly,
            _ => ProtectionType::None,
        }
    }

    /// Check if editing of body text is allowed
    pub fn allows_body_editing(&self) -> bool {
        matches!(self, ProtectionType::None | ProtectionType::TrackedChangesOnly)
    }

    /// Check if comments are allowed
    pub fn allows_comments(&self) -> bool {
        matches!(
            self,
            ProtectionType::None
                | ProtectionType::CommentsOnly
                | ProtectionType::TrackedChangesOnly
        )
    }

    /// Check if form fields can be edited
    pub fn allows_form_fields(&self) -> bool {
        matches!(
            self,
            ProtectionType::None | ProtectionType::FormFieldsOnly
        )
    }
}

/// Hash algorithm used for password protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// Legacy XOR-based hash (weak, for compatibility)
    LegacyXor,
    /// SHA-1
    Sha1,
    /// SHA-256
    Sha256,
    /// SHA-384
    Sha384,
    /// SHA-512
    Sha512,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Sha256
    }
}

impl HashAlgorithm {
    /// Get the OOXML attribute value
    pub fn ooxml_value(&self) -> &'static str {
        match self {
            HashAlgorithm::LegacyXor => "xor",
            HashAlgorithm::Sha1 => "SHA-1",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha384 => "SHA-384",
            HashAlgorithm::Sha512 => "SHA-512",
        }
    }

    /// Parse from OOXML attribute value
    pub fn from_ooxml(value: &str) -> Self {
        match value {
            "xor" => HashAlgorithm::LegacyXor,
            "SHA-1" => HashAlgorithm::Sha1,
            "SHA-256" => HashAlgorithm::Sha256,
            "SHA-384" => HashAlgorithm::Sha384,
            "SHA-512" => HashAlgorithm::Sha512,
            _ => HashAlgorithm::Sha256,
        }
    }
}

/// Password protection settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordProtection {
    /// Hash of the password
    pub hash_value: String,
    /// Salt used for hashing
    pub salt_value: String,
    /// Number of hash iterations (spin count)
    pub spin_count: u32,
    /// Hash algorithm used
    pub algorithm: HashAlgorithm,
}

impl PasswordProtection {
    /// Create new password protection settings
    pub fn new(hash: impl Into<String>, salt: impl Into<String>, spin_count: u32) -> Self {
        Self {
            hash_value: hash.into(),
            salt_value: salt.into(),
            spin_count,
            algorithm: HashAlgorithm::default(),
        }
    }

    /// Set the hash algorithm
    pub fn with_algorithm(mut self, algorithm: HashAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }
}

/// An exception to document protection for a specific editor/user
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditException {
    /// Identifier for the editor (email, username, etc.)
    pub editor: String,
    /// Type of exception: "group" or "individual"
    pub editor_type: EditorType,
    /// IDs of content regions this editor can modify
    pub editable_regions: Vec<String>,
}

/// Type of editor for protection exceptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorType {
    /// Individual user
    Individual,
    /// Group of users
    Group,
    /// Everyone (all editors)
    Everyone,
}

impl EditException {
    /// Create an exception for an individual editor
    pub fn individual(editor: impl Into<String>) -> Self {
        Self {
            editor: editor.into(),
            editor_type: EditorType::Individual,
            editable_regions: Vec::new(),
        }
    }

    /// Create an exception for a group
    pub fn group(group_name: impl Into<String>) -> Self {
        Self {
            editor: group_name.into(),
            editor_type: EditorType::Group,
            editable_regions: Vec::new(),
        }
    }

    /// Create an exception for everyone
    pub fn everyone() -> Self {
        Self {
            editor: "everyone".to_string(),
            editor_type: EditorType::Everyone,
            editable_regions: Vec::new(),
        }
    }

    /// Add an editable region
    pub fn with_region(mut self, region_id: impl Into<String>) -> Self {
        self.editable_regions.push(region_id.into());
        self
    }
}

/// Document protection configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentProtection {
    /// Type of protection applied
    pub protection_type: ProtectionType,
    /// Whether protection is currently enforced
    pub enforced: bool,
    /// Password protection (optional)
    pub password: Option<PasswordProtection>,
    /// Editing exceptions for specific users/groups
    pub exceptions: Vec<EditException>,
    /// Whether formatting is restricted
    pub formatting_restricted: bool,
    /// Allowed styles when formatting is restricted (style IDs)
    pub allowed_styles: Vec<String>,
}

impl Default for DocumentProtection {
    fn default() -> Self {
        Self {
            protection_type: ProtectionType::None,
            enforced: false,
            password: None,
            exceptions: Vec::new(),
            formatting_restricted: false,
            allowed_styles: Vec::new(),
        }
    }
}

impl DocumentProtection {
    /// Create unprotected document settings
    pub fn none() -> Self {
        Self::default()
    }

    /// Create read-only protection
    pub fn read_only() -> Self {
        Self {
            protection_type: ProtectionType::ReadOnly,
            enforced: true,
            ..Default::default()
        }
    }

    /// Create form-fields-only protection
    pub fn forms_only() -> Self {
        Self {
            protection_type: ProtectionType::FormFieldsOnly,
            enforced: true,
            ..Default::default()
        }
    }

    /// Create comments-only protection
    pub fn comments_only() -> Self {
        Self {
            protection_type: ProtectionType::CommentsOnly,
            enforced: true,
            ..Default::default()
        }
    }

    /// Create tracked-changes-only protection
    pub fn tracked_changes_only() -> Self {
        Self {
            protection_type: ProtectionType::TrackedChangesOnly,
            enforced: true,
            ..Default::default()
        }
    }

    /// Set password protection
    pub fn with_password(mut self, password: PasswordProtection) -> Self {
        self.password = Some(password);
        self
    }

    /// Add an editing exception
    pub fn with_exception(mut self, exception: EditException) -> Self {
        self.exceptions.push(exception);
        self
    }

    /// Restrict formatting to allowed styles
    pub fn with_formatting_restriction(mut self, allowed_styles: Vec<String>) -> Self {
        self.formatting_restricted = true;
        self.allowed_styles = allowed_styles;
        self
    }

    /// Check if protection is active
    pub fn is_protected(&self) -> bool {
        self.enforced && self.protection_type != ProtectionType::None
    }

    /// Check if a password is required to unprotect
    pub fn requires_password(&self) -> bool {
        self.is_protected() && self.password.is_some()
    }

    /// Check if a user can edit the document body
    pub fn can_edit_body(&self, editor: Option<&str>) -> bool {
        if !self.is_protected() {
            return true;
        }
        if self.protection_type.allows_body_editing() {
            return true;
        }
        // Check exceptions
        if let Some(editor_id) = editor {
            self.exceptions.iter().any(|e| {
                e.editor_type == EditorType::Everyone || e.editor == editor_id
            })
        } else {
            false
        }
    }

    /// Check if a user can edit form fields
    pub fn can_edit_forms(&self) -> bool {
        if !self.is_protected() {
            return true;
        }
        self.protection_type.allows_form_fields()
    }

    /// Check if a user can add comments
    pub fn can_add_comments(&self) -> bool {
        if !self.is_protected() {
            return true;
        }
        self.protection_type.allows_comments()
    }

    /// Check if a style is allowed under formatting restrictions
    pub fn is_style_allowed(&self, style_id: &str) -> bool {
        if !self.formatting_restricted {
            return true;
        }
        self.allowed_styles.iter().any(|s| s == style_id)
    }

    /// Enforce protection
    pub fn enforce(&mut self) {
        self.enforced = true;
    }

    /// Remove enforcement (unprotect)
    pub fn unenforce(&mut self) {
        self.enforced = false;
    }

    /// Get the OOXML representation info
    pub fn ooxml_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("w:edit".to_string(), self.protection_type.ooxml_value().to_string());
        attrs.insert("w:enforcement".to_string(), if self.enforced { "1" } else { "0" }.to_string());
        if let Some(ref pwd) = self.password {
            attrs.insert("w:algorithmName".to_string(), pwd.algorithm.ooxml_value().to_string());
            attrs.insert("w:hashValue".to_string(), pwd.hash_value.clone());
            attrs.insert("w:saltValue".to_string(), pwd.salt_value.clone());
            attrs.insert("w:spinCount".to_string(), pwd.spin_count.to_string());
        }
        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_type_default() {
        assert_eq!(ProtectionType::default(), ProtectionType::None);
    }

    #[test]
    fn test_protection_type_ooxml_roundtrip() {
        let types = [
            ProtectionType::None,
            ProtectionType::ReadOnly,
            ProtectionType::FormFieldsOnly,
            ProtectionType::CommentsOnly,
            ProtectionType::TrackedChangesOnly,
        ];
        for pt in &types {
            let ooxml = pt.ooxml_value();
            let parsed = ProtectionType::from_ooxml(ooxml);
            assert_eq!(*pt, parsed);
        }
    }

    #[test]
    fn test_protection_type_permissions() {
        assert!(ProtectionType::None.allows_body_editing());
        assert!(!ProtectionType::ReadOnly.allows_body_editing());
        assert!(!ProtectionType::FormFieldsOnly.allows_body_editing());
        assert!(!ProtectionType::CommentsOnly.allows_body_editing());
        assert!(ProtectionType::TrackedChangesOnly.allows_body_editing());

        assert!(ProtectionType::None.allows_comments());
        assert!(!ProtectionType::ReadOnly.allows_comments());
        assert!(!ProtectionType::FormFieldsOnly.allows_comments());
        assert!(ProtectionType::CommentsOnly.allows_comments());
        assert!(ProtectionType::TrackedChangesOnly.allows_comments());

        assert!(ProtectionType::None.allows_form_fields());
        assert!(!ProtectionType::ReadOnly.allows_form_fields());
        assert!(ProtectionType::FormFieldsOnly.allows_form_fields());
        assert!(!ProtectionType::CommentsOnly.allows_form_fields());
        assert!(!ProtectionType::TrackedChangesOnly.allows_form_fields());
    }

    #[test]
    fn test_document_protection_default() {
        let prot = DocumentProtection::default();
        assert_eq!(prot.protection_type, ProtectionType::None);
        assert!(!prot.enforced);
        assert!(!prot.is_protected());
        assert!(prot.can_edit_body(None));
        assert!(prot.can_edit_forms());
        assert!(prot.can_add_comments());
    }

    #[test]
    fn test_read_only_protection() {
        let prot = DocumentProtection::read_only();
        assert!(prot.is_protected());
        assert!(!prot.can_edit_body(None));
        assert!(!prot.can_edit_forms());
        assert!(!prot.can_add_comments());
    }

    #[test]
    fn test_forms_only_protection() {
        let prot = DocumentProtection::forms_only();
        assert!(prot.is_protected());
        assert!(!prot.can_edit_body(None));
        assert!(prot.can_edit_forms());
        assert!(!prot.can_add_comments());
    }

    #[test]
    fn test_comments_only_protection() {
        let prot = DocumentProtection::comments_only();
        assert!(prot.is_protected());
        assert!(!prot.can_edit_body(None));
        assert!(!prot.can_edit_forms());
        assert!(prot.can_add_comments());
    }

    #[test]
    fn test_tracked_changes_protection() {
        let prot = DocumentProtection::tracked_changes_only();
        assert!(prot.is_protected());
        assert!(prot.can_edit_body(None));
        assert!(!prot.can_edit_forms());
        assert!(prot.can_add_comments());
    }

    #[test]
    fn test_password_protection() {
        let prot = DocumentProtection::read_only()
            .with_password(PasswordProtection::new("abc123hash", "salt456", 100000));
        assert!(prot.requires_password());
    }

    #[test]
    fn test_edit_exceptions() {
        let prot = DocumentProtection::read_only()
            .with_exception(EditException::individual("user@example.com"));
        assert!(prot.can_edit_body(Some("user@example.com")));
        assert!(!prot.can_edit_body(Some("other@example.com")));
        assert!(!prot.can_edit_body(None));
    }

    #[test]
    fn test_everyone_exception() {
        let prot = DocumentProtection::read_only()
            .with_exception(EditException::everyone());
        assert!(prot.can_edit_body(Some("anyone@example.com")));
    }

    #[test]
    fn test_formatting_restriction() {
        let prot = DocumentProtection::default()
            .with_formatting_restriction(vec!["Heading1".to_string(), "Normal".to_string()]);
        assert!(prot.is_style_allowed("Heading1"));
        assert!(prot.is_style_allowed("Normal"));
        assert!(!prot.is_style_allowed("Title"));
    }

    #[test]
    fn test_enforce_unenforce() {
        let mut prot = DocumentProtection::read_only();
        assert!(prot.is_protected());

        prot.unenforce();
        assert!(!prot.is_protected());
        assert!(prot.can_edit_body(None));

        prot.enforce();
        assert!(prot.is_protected());
    }

    #[test]
    fn test_ooxml_attributes() {
        let prot = DocumentProtection::read_only()
            .with_password(
                PasswordProtection::new("hash", "salt", 100000)
                    .with_algorithm(HashAlgorithm::Sha512)
            );
        let attrs = prot.ooxml_attributes();
        assert_eq!(attrs.get("w:edit").unwrap(), "readOnly");
        assert_eq!(attrs.get("w:enforcement").unwrap(), "1");
        assert_eq!(attrs.get("w:algorithmName").unwrap(), "SHA-512");
        assert_eq!(attrs.get("w:hashValue").unwrap(), "hash");
        assert_eq!(attrs.get("w:saltValue").unwrap(), "salt");
        assert_eq!(attrs.get("w:spinCount").unwrap(), "100000");
    }

    #[test]
    fn test_hash_algorithm_roundtrip() {
        let algorithms = [
            HashAlgorithm::LegacyXor,
            HashAlgorithm::Sha1,
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha384,
            HashAlgorithm::Sha512,
        ];
        for algo in &algorithms {
            let ooxml = algo.ooxml_value();
            let parsed = HashAlgorithm::from_ooxml(ooxml);
            assert_eq!(*algo, parsed);
        }
    }

    #[test]
    fn test_protection_serialization() {
        let prot = DocumentProtection::forms_only()
            .with_password(PasswordProtection::new("hash", "salt", 50000))
            .with_exception(EditException::individual("admin@example.com").with_region("section1"));

        let json = serde_json::to_string(&prot).unwrap();
        let parsed: DocumentProtection = serde_json::from_str(&json).unwrap();

        assert_eq!(prot.protection_type, parsed.protection_type);
        assert_eq!(prot.enforced, parsed.enforced);
        assert_eq!(prot.exceptions.len(), parsed.exceptions.len());
    }
}
