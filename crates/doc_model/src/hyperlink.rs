//! Hyperlink node - wraps text runs with link functionality

use crate::{Node, NodeId, NodeType};
use serde::{Deserialize, Serialize};

/// Target type for a hyperlink
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HyperlinkTarget {
    /// External URL (web links)
    External(String),
    /// Internal document bookmark
    Internal(String),
    /// Email address with optional subject
    Email {
        address: String,
        subject: Option<String>,
    },
}

impl HyperlinkTarget {
    /// Create an external URL target
    pub fn external(url: impl Into<String>) -> Self {
        HyperlinkTarget::External(url.into())
    }

    /// Create an internal bookmark target
    pub fn internal(bookmark: impl Into<String>) -> Self {
        HyperlinkTarget::Internal(bookmark.into())
    }

    /// Create an email target
    pub fn email(address: impl Into<String>, subject: Option<String>) -> Self {
        HyperlinkTarget::Email {
            address: address.into(),
            subject,
        }
    }

    /// Get the URL representation of this target
    pub fn to_url(&self) -> String {
        match self {
            HyperlinkTarget::External(url) => url.clone(),
            HyperlinkTarget::Internal(bookmark) => format!("#{}", bookmark),
            HyperlinkTarget::Email { address, subject } => {
                let mut url = format!("mailto:{}", address);
                if let Some(subj) = subject {
                    url.push_str("?subject=");
                    url.push_str(&urlencoding::encode(subj));
                }
                url
            }
        }
    }

    /// Check if this is an external URL
    pub fn is_external(&self) -> bool {
        matches!(self, HyperlinkTarget::External(_))
    }

    /// Check if this is an internal bookmark
    pub fn is_internal(&self) -> bool {
        matches!(self, HyperlinkTarget::Internal(_))
    }

    /// Check if this is an email link
    pub fn is_email(&self) -> bool {
        matches!(self, HyperlinkTarget::Email { .. })
    }

    /// Validate the target
    pub fn validate(&self) -> Result<(), HyperlinkValidationError> {
        match self {
            HyperlinkTarget::External(url) => {
                // Basic URL validation
                if url.is_empty() {
                    return Err(HyperlinkValidationError::EmptyUrl);
                }
                // Check for dangerous protocols
                let lower_url = url.to_lowercase();
                if lower_url.starts_with("javascript:") ||
                   lower_url.starts_with("data:") ||
                   lower_url.starts_with("vbscript:") {
                    return Err(HyperlinkValidationError::UnsafeProtocol);
                }
                // Ensure it's a valid URL format
                if !url.contains("://") && !url.starts_with("//") && !url.starts_with('/') {
                    // Allow relative URLs or add https:// by default
                    return Ok(());
                }
                Ok(())
            }
            HyperlinkTarget::Internal(bookmark) => {
                if bookmark.is_empty() {
                    return Err(HyperlinkValidationError::EmptyBookmark);
                }
                Ok(())
            }
            HyperlinkTarget::Email { address, .. } => {
                if address.is_empty() {
                    return Err(HyperlinkValidationError::EmptyEmail);
                }
                // Basic email validation (contains @)
                if !address.contains('@') {
                    return Err(HyperlinkValidationError::InvalidEmail);
                }
                Ok(())
            }
        }
    }
}

/// Errors that can occur during hyperlink validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperlinkValidationError {
    EmptyUrl,
    UnsafeProtocol,
    EmptyBookmark,
    EmptyEmail,
    InvalidEmail,
}

impl std::fmt::Display for HyperlinkValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyperlinkValidationError::EmptyUrl => write!(f, "URL cannot be empty"),
            HyperlinkValidationError::UnsafeProtocol => write!(f, "Unsafe protocol detected"),
            HyperlinkValidationError::EmptyBookmark => write!(f, "Bookmark name cannot be empty"),
            HyperlinkValidationError::EmptyEmail => write!(f, "Email address cannot be empty"),
            HyperlinkValidationError::InvalidEmail => write!(f, "Invalid email address format"),
        }
    }
}

impl std::error::Error for HyperlinkValidationError {}

/// A hyperlink that wraps one or more runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperlink {
    id: NodeId,
    parent: Option<NodeId>,
    /// IDs of child runs (the text content of the hyperlink)
    children: Vec<NodeId>,
    /// The link target
    pub target: HyperlinkTarget,
    /// Optional tooltip text shown on hover
    pub tooltip: Option<String>,
}

impl Hyperlink {
    /// Create a new hyperlink with the given target
    pub fn new(target: HyperlinkTarget) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            target,
            tooltip: None,
        }
    }

    /// Create a new hyperlink with a tooltip
    pub fn with_tooltip(target: HyperlinkTarget, tooltip: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            target,
            tooltip: Some(tooltip.into()),
        }
    }

    /// Add a child run ID
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Insert a child at a specific index
    pub fn insert_child(&mut self, index: usize, child_id: NodeId) {
        self.children.insert(index, child_id);
    }

    /// Remove a child by ID
    pub fn remove_child(&mut self, child_id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the target URL as a string
    pub fn target_url(&self) -> String {
        self.target.to_url()
    }

    /// Set the hyperlink target
    pub fn set_target(&mut self, target: HyperlinkTarget) {
        self.target = target;
    }

    /// Set the tooltip
    pub fn set_tooltip(&mut self, tooltip: Option<String>) {
        self.tooltip = tooltip;
    }

    /// Validate this hyperlink
    pub fn validate(&self) -> Result<(), HyperlinkValidationError> {
        self.target.validate()
    }
}

impl Node for Hyperlink {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Hyperlink
    }

    fn children(&self) -> &[NodeId] {
        &self.children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_hyperlink() {
        let target = HyperlinkTarget::external("https://example.com");
        assert!(target.is_external());
        assert!(!target.is_internal());
        assert!(!target.is_email());
        assert_eq!(target.to_url(), "https://example.com");
        assert!(target.validate().is_ok());
    }

    #[test]
    fn test_internal_hyperlink() {
        let target = HyperlinkTarget::internal("chapter-1");
        assert!(!target.is_external());
        assert!(target.is_internal());
        assert!(!target.is_email());
        assert_eq!(target.to_url(), "#chapter-1");
        assert!(target.validate().is_ok());
    }

    #[test]
    fn test_email_hyperlink() {
        let target = HyperlinkTarget::email("test@example.com", Some("Hello".to_string()));
        assert!(!target.is_external());
        assert!(!target.is_internal());
        assert!(target.is_email());
        assert!(target.to_url().starts_with("mailto:test@example.com"));
        assert!(target.validate().is_ok());
    }

    #[test]
    fn test_hyperlink_validation() {
        // Empty URL
        let target = HyperlinkTarget::external("");
        assert!(matches!(target.validate(), Err(HyperlinkValidationError::EmptyUrl)));

        // Unsafe protocol
        let target = HyperlinkTarget::external("javascript:alert('xss')");
        assert!(matches!(target.validate(), Err(HyperlinkValidationError::UnsafeProtocol)));

        // Invalid email
        let target = HyperlinkTarget::email("invalid-email", None);
        assert!(matches!(target.validate(), Err(HyperlinkValidationError::InvalidEmail)));
    }

    #[test]
    fn test_hyperlink_with_tooltip() {
        let hyperlink = Hyperlink::with_tooltip(
            HyperlinkTarget::external("https://example.com"),
            "Click to visit example.com"
        );
        assert_eq!(hyperlink.tooltip, Some("Click to visit example.com".to_string()));
    }
}
