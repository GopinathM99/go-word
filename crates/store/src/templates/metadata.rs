//! Template metadata structures

use doc_model::Position;
use serde::{Deserialize, Serialize};

/// Template categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Business documents (letterheads, invoices, reports)
    Business,
    /// Academic documents (papers, theses, essays)
    Academic,
    /// Personal documents (letters, resumes)
    Personal,
    /// Creative documents (newsletters, flyers)
    Creative,
    /// Legal documents (contracts, agreements)
    Legal,
    /// Custom category
    Custom(String),
}

impl Default for TemplateCategory {
    fn default() -> Self {
        Self::Personal
    }
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Business => write!(f, "Business"),
            Self::Academic => write!(f, "Academic"),
            Self::Personal => write!(f, "Personal"),
            Self::Creative => write!(f, "Creative"),
            Self::Legal => write!(f, "Legal"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl From<&str> for TemplateCategory {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "business" => Self::Business,
            "academic" => Self::Academic,
            "personal" => Self::Personal,
            "creative" => Self::Creative,
            "legal" => Self::Legal,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// A locked region in a template that cannot be edited
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockedRegion {
    /// Start position of the locked region
    pub start: Position,
    /// End position of the locked region
    pub end: Position,
    /// Reason for locking (e.g., "Company letterhead", "Legal disclaimer")
    pub reason: String,
    /// Optional identifier for the locked region
    pub id: Option<String>,
}

impl LockedRegion {
    /// Create a new locked region
    pub fn new(start: Position, end: Position, reason: impl Into<String>) -> Self {
        Self {
            start,
            end,
            reason: reason.into(),
            id: None,
        }
    }

    /// Create a locked region with an ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Check if a position is within this locked region
    pub fn contains(&self, position: &Position) -> bool {
        // Same node check
        if position.node_id == self.start.node_id && position.node_id == self.end.node_id {
            return position.offset >= self.start.offset && position.offset <= self.end.offset;
        }

        // Cross-node locked regions would need document structure for proper comparison
        // For now, we check if the position is at one of the boundary nodes
        if position.node_id == self.start.node_id {
            return position.offset >= self.start.offset;
        }
        if position.node_id == self.end.node_id {
            return position.offset <= self.end.offset;
        }

        // TODO: Implement full cross-node comparison with document tree traversal
        false
    }

    /// Check if a range overlaps with this locked region
    pub fn overlaps(&self, start: &Position, end: &Position) -> bool {
        // Simplified check - same node
        if self.start.node_id == start.node_id
            && self.end.node_id == end.node_id
            && start.node_id == end.node_id
        {
            // Check for any overlap
            return !(end.offset <= self.start.offset || start.offset >= self.end.offset);
        }

        // Check if either position is contained
        self.contains(start) || self.contains(end)
    }
}

/// Template metadata stored in template.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Unique template identifier
    pub id: String,
    /// Display name of the template
    pub name: String,
    /// Description of what the template is for
    pub description: String,
    /// Template category
    pub category: TemplateCategory,
    /// Author of the template
    pub author: String,
    /// Creation timestamp (ISO 8601)
    pub created: String,
    /// Last modified timestamp (ISO 8601)
    pub modified: Option<String>,
    /// Version of the template
    pub version: String,
    /// Locked regions that cannot be edited
    pub locked_regions: Vec<LockedRegion>,
    /// Tags for searching/filtering
    pub tags: Vec<String>,
    /// Whether a thumbnail is available
    pub has_thumbnail: bool,
    /// Preview text (first few lines of content)
    pub preview_text: Option<String>,
}

impl TemplateMetadata {
    /// Create new template metadata
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category: TemplateCategory::default(),
            author: String::new(),
            created: Self::now_iso8601(),
            modified: None,
            version: "1.0".to_string(),
            locked_regions: Vec::new(),
            tags: Vec::new(),
            has_thumbnail: false,
            preview_text: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: TemplateCategory) -> Self {
        self.category = category;
        self
    }

    /// Set the author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Add a locked region
    pub fn with_locked_region(mut self, region: LockedRegion) -> Self {
        self.locked_regions.push(region);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Mark as having a thumbnail
    pub fn with_thumbnail(mut self) -> Self {
        self.has_thumbnail = true;
        self
    }

    /// Set preview text
    pub fn with_preview(mut self, preview: impl Into<String>) -> Self {
        self.preview_text = Some(preview.into());
        self
    }

    /// Update the modified timestamp
    pub fn touch(&mut self) {
        self.modified = Some(Self::now_iso8601());
    }

    /// Check if a position is in a locked region
    pub fn is_position_locked(&self, position: &Position) -> Option<&LockedRegion> {
        self.locked_regions.iter().find(|r| r.contains(position))
    }

    /// Check if a range overlaps with any locked region
    pub fn overlaps_locked_region(&self, start: &Position, end: &Position) -> Option<&LockedRegion> {
        self.locked_regions.iter().find(|r| r.overlaps(start, end))
    }

    /// Get current timestamp in ISO 8601 format
    fn now_iso8601() -> String {
        // Simple implementation - in production would use proper time library
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        // Format as ISO 8601 (simplified)
        format!("{}Z", secs)
    }
}

impl Default for TemplateMetadata {
    fn default() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string(), "Untitled Template")
    }
}

/// Summary information for template listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSummary {
    /// Template ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Short description
    pub description: String,
    /// Category
    pub category: String,
    /// Author
    pub author: String,
    /// Tags
    pub tags: Vec<String>,
    /// Whether thumbnail is available
    pub has_thumbnail: bool,
    /// Preview text
    pub preview_text: Option<String>,
}

impl From<&TemplateMetadata> for TemplateSummary {
    fn from(meta: &TemplateMetadata) -> Self {
        Self {
            id: meta.id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            category: meta.category.to_string(),
            author: meta.author.clone(),
            tags: meta.tags.clone(),
            has_thumbnail: meta.has_thumbnail,
            preview_text: meta.preview_text.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::NodeId;

    #[test]
    fn test_template_metadata_creation() {
        let meta = TemplateMetadata::new("test-id", "Test Template")
            .with_description("A test template")
            .with_category(TemplateCategory::Business)
            .with_author("Test Author")
            .with_tag("test")
            .with_tag("example");

        assert_eq!(meta.id, "test-id");
        assert_eq!(meta.name, "Test Template");
        assert_eq!(meta.description, "A test template");
        assert_eq!(meta.category, TemplateCategory::Business);
        assert_eq!(meta.author, "Test Author");
        assert_eq!(meta.tags, vec!["test", "example"]);
    }

    #[test]
    fn test_locked_region_contains() {
        let node_id = NodeId::new();
        let region = LockedRegion::new(
            Position::new(node_id, 5),
            Position::new(node_id, 15),
            "Test lock",
        );

        // Position inside
        assert!(region.contains(&Position::new(node_id, 10)));
        // Position at start
        assert!(region.contains(&Position::new(node_id, 5)));
        // Position at end
        assert!(region.contains(&Position::new(node_id, 15)));
        // Position before
        assert!(!region.contains(&Position::new(node_id, 4)));
        // Position after
        assert!(!region.contains(&Position::new(node_id, 16)));
    }

    #[test]
    fn test_locked_region_overlaps() {
        let node_id = NodeId::new();
        let region = LockedRegion::new(
            Position::new(node_id, 10),
            Position::new(node_id, 20),
            "Test lock",
        );

        // Overlapping range
        assert!(region.overlaps(
            &Position::new(node_id, 5),
            &Position::new(node_id, 15)
        ));
        // Range inside
        assert!(region.overlaps(
            &Position::new(node_id, 12),
            &Position::new(node_id, 18)
        ));
        // Range before
        assert!(!region.overlaps(
            &Position::new(node_id, 0),
            &Position::new(node_id, 9)
        ));
        // Range after
        assert!(!region.overlaps(
            &Position::new(node_id, 21),
            &Position::new(node_id, 30)
        ));
    }

    #[test]
    fn test_category_from_string() {
        assert_eq!(TemplateCategory::from("business"), TemplateCategory::Business);
        assert_eq!(TemplateCategory::from("Academic"), TemplateCategory::Academic);
        assert_eq!(
            TemplateCategory::from("custom-type"),
            TemplateCategory::Custom("custom-type".to_string())
        );
    }
}
