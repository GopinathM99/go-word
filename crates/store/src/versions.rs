//! Document version tracking and history
//!
//! This module provides functionality to track document versions,
//! compare versions, and restore to previous versions.

use crate::{IntegrityChecker, Result, StoreError};
use doc_model::DocumentTree;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    /// Maximum number of versions to keep
    pub max_versions: usize,
    /// Directory to store version files
    pub versions_dir: PathBuf,
    /// Whether to automatically create versions
    pub auto_version: bool,
    /// Minimum interval between auto-versions (in seconds)
    pub auto_version_interval_secs: u64,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            max_versions: 10,
            versions_dir: PathBuf::from(".versions"),
            auto_version: true,
            auto_version_interval_secs: 3600, // 1 hour
        }
    }
}

impl VersionConfig {
    /// Create a config with custom max versions
    pub fn with_max_versions(mut self, max: usize) -> Self {
        self.max_versions = max;
        self
    }

    /// Create a config with custom versions directory
    pub fn with_versions_dir(mut self, dir: PathBuf) -> Self {
        self.versions_dir = dir;
        self
    }
}

/// A single document version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    /// Version number (1-based, incremental)
    pub version_number: u64,
    /// Timestamp when version was created (Unix timestamp in ms)
    pub timestamp: u64,
    /// Checksum of the document at this version
    pub checksum: String,
    /// Optional summary of changes
    pub changes_summary: Option<String>,
    /// Path to the version file
    pub path: PathBuf,
    /// Size of the version file in bytes
    pub file_size: u64,
    /// Human-readable time description
    pub time_description: String,
}

impl DocumentVersion {
    /// Format timestamp into human-readable description
    fn format_time_description(timestamp_ms: u64) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let diff_secs = (now.saturating_sub(timestamp_ms)) / 1000;

        if diff_secs < 60 {
            "Just now".to_string()
        } else if diff_secs < 3600 {
            let mins = diff_secs / 60;
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if diff_secs < 86400 {
            let hours = diff_secs / 3600;
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else {
            let days = diff_secs / 86400;
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    }
}

/// Version history for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    /// Document ID
    pub document_id: String,
    /// All versions, sorted by version number (newest first)
    pub versions: Vec<DocumentVersion>,
    /// Current version number
    pub current_version: u64,
    /// Next version number to assign
    pub next_version: u64,
}

impl VersionHistory {
    /// Create a new empty version history
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            versions: Vec::new(),
            current_version: 0,
            next_version: 1,
        }
    }

    /// Get the latest version
    pub fn latest(&self) -> Option<&DocumentVersion> {
        self.versions.first()
    }

    /// Get a specific version
    pub fn get(&self, version_number: u64) -> Option<&DocumentVersion> {
        self.versions
            .iter()
            .find(|v| v.version_number == version_number)
    }

    /// Get the number of versions
    pub fn len(&self) -> usize {
        self.versions.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }
}

/// Differences between two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    /// Version number of the older version
    pub old_version: u64,
    /// Version number of the newer version
    pub new_version: u64,
    /// Changes detected
    pub changes: Vec<VersionChange>,
    /// Whether the documents are identical
    pub is_identical: bool,
}

/// Types of changes between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VersionChange {
    /// Paragraph was added
    ParagraphAdded { count: usize },
    /// Paragraph was removed
    ParagraphRemoved { count: usize },
    /// Content was modified
    ContentModified { details: String },
    /// Image was added
    ImageAdded { count: usize },
    /// Image was removed
    ImageRemoved { count: usize },
    /// Table was added
    TableAdded { count: usize },
    /// Table was removed
    TableRemoved { count: usize },
    /// Formatting changed
    FormattingChanged { details: String },
    /// Structure changed
    StructureChanged { details: String },
}

/// Version manager for a document
pub struct VersionManager {
    /// Configuration
    config: VersionConfig,
    /// Document ID
    document_id: String,
    /// Version history
    history: VersionHistory,
    /// Integrity checker for checksums
    checker: IntegrityChecker,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new(document_id: impl Into<String>, config: VersionConfig) -> Self {
        let doc_id = document_id.into();
        Self {
            config,
            document_id: doc_id.clone(),
            history: VersionHistory::new(doc_id),
            checker: IntegrityChecker::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &VersionConfig {
        &self.config
    }

    /// Get the version history
    pub fn history(&self) -> &VersionHistory {
        &self.history
    }

    /// Get the version directory for this document
    fn version_dir(&self) -> PathBuf {
        self.config.versions_dir.join(&self.document_id)
    }

    /// Get the path for a version file
    fn version_path(&self, version_number: u64) -> PathBuf {
        self.version_dir()
            .join(format!("v{}.wdj", version_number))
    }

    /// Get the path for the history metadata file
    fn history_path(&self) -> PathBuf {
        self.version_dir().join("history.json")
    }

    /// Create a new version of the document
    pub async fn create_version(
        &mut self,
        tree: &DocumentTree,
        summary: Option<String>,
    ) -> Result<DocumentVersion> {
        // Ensure version directory exists
        tokio::fs::create_dir_all(self.version_dir()).await?;

        let version_number = self.history.next_version;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let checksum = self.checker.compute_checksum(tree);
        let path = self.version_path(version_number);

        // Save the document
        crate::save_document(tree, &path).await?;

        // Get file size
        let metadata = tokio::fs::metadata(&path).await?;
        let file_size = metadata.len();

        let version = DocumentVersion {
            version_number,
            timestamp,
            checksum,
            changes_summary: summary,
            path,
            file_size,
            time_description: DocumentVersion::format_time_description(timestamp),
        };

        // Add to history
        self.history.versions.insert(0, version.clone());
        self.history.current_version = version_number;
        self.history.next_version = version_number + 1;

        // Cleanup old versions
        self.cleanup_old_versions().await?;

        // Save history
        self.save_history().await?;

        Ok(version)
    }

    /// Load version history from disk
    pub async fn load_history(&mut self) -> Result<()> {
        let history_path = self.history_path();

        if history_path.exists() {
            let content = tokio::fs::read_to_string(&history_path).await?;
            self.history = serde_json::from_str(&content)?;

            // Update time descriptions
            for version in &mut self.history.versions {
                version.time_description =
                    DocumentVersion::format_time_description(version.timestamp);
            }
        }

        Ok(())
    }

    /// Save version history to disk
    async fn save_history(&self) -> Result<()> {
        let history_path = self.history_path();
        let content = serde_json::to_string_pretty(&self.history)?;
        tokio::fs::write(&history_path, content).await?;
        Ok(())
    }

    /// Load a specific version
    pub async fn load_version(&self, version_number: u64) -> Result<DocumentTree> {
        let version = self
            .history
            .get(version_number)
            .ok_or_else(|| StoreError::FileNotFound(format!("Version {}", version_number)))?;

        crate::load_document(&version.path).await
    }

    /// Restore to a previous version
    /// Returns the restored document
    pub async fn restore_version(&mut self, version_number: u64) -> Result<DocumentTree> {
        // Load the version
        let tree = self.load_version(version_number).await?;

        // Create a new version for the restore
        self.create_version(
            &tree,
            Some(format!("Restored from version {}", version_number)),
        )
        .await?;

        Ok(tree)
    }

    /// Compare two versions
    pub async fn compare_versions(
        &self,
        old_version: u64,
        new_version: u64,
    ) -> Result<VersionDiff> {
        let old_tree = self.load_version(old_version).await?;
        let new_tree = self.load_version(new_version).await?;

        let old_checksum = self.checker.compute_checksum(&old_tree);
        let new_checksum = self.checker.compute_checksum(&new_tree);

        if old_checksum == new_checksum {
            return Ok(VersionDiff {
                old_version,
                new_version,
                changes: Vec::new(),
                is_identical: true,
            });
        }

        let mut changes = Vec::new();

        // Compare paragraph counts
        let old_para_count = old_tree.nodes.paragraphs.len();
        let new_para_count = new_tree.nodes.paragraphs.len();

        if new_para_count > old_para_count {
            changes.push(VersionChange::ParagraphAdded {
                count: new_para_count - old_para_count,
            });
        } else if new_para_count < old_para_count {
            changes.push(VersionChange::ParagraphRemoved {
                count: old_para_count - new_para_count,
            });
        }

        // Compare image counts
        let old_image_count = old_tree.nodes.images.len();
        let new_image_count = new_tree.nodes.images.len();

        if new_image_count > old_image_count {
            changes.push(VersionChange::ImageAdded {
                count: new_image_count - old_image_count,
            });
        } else if new_image_count < old_image_count {
            changes.push(VersionChange::ImageRemoved {
                count: old_image_count - new_image_count,
            });
        }

        // Compare table counts
        let old_table_count = old_tree.nodes.tables.len();
        let new_table_count = new_tree.nodes.tables.len();

        if new_table_count > old_table_count {
            changes.push(VersionChange::TableAdded {
                count: new_table_count - old_table_count,
            });
        } else if new_table_count < old_table_count {
            changes.push(VersionChange::TableRemoved {
                count: old_table_count - new_table_count,
            });
        }

        // If no specific changes detected but checksums differ, content was modified
        if changes.is_empty() {
            changes.push(VersionChange::ContentModified {
                details: "Document content was modified".to_string(),
            });
        }

        Ok(VersionDiff {
            old_version,
            new_version,
            changes,
            is_identical: false,
        })
    }

    /// Cleanup old versions beyond max_versions
    async fn cleanup_old_versions(&mut self) -> Result<()> {
        while self.history.versions.len() > self.config.max_versions {
            if let Some(oldest) = self.history.versions.pop() {
                // Delete the version file
                if oldest.path.exists() {
                    tokio::fs::remove_file(&oldest.path).await?;
                }
            }
        }
        Ok(())
    }

    /// Delete all versions
    pub async fn delete_all_versions(&mut self) -> Result<()> {
        let version_dir = self.version_dir();

        if version_dir.exists() {
            tokio::fs::remove_dir_all(&version_dir).await?;
        }

        self.history = VersionHistory::new(&self.document_id);
        Ok(())
    }

    /// Get total size of all versions
    pub fn total_size(&self) -> u64 {
        self.history.versions.iter().map(|v| v.file_size).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_version_config_default() {
        let config = VersionConfig::default();
        assert_eq!(config.max_versions, 10);
        assert!(config.auto_version);
    }

    #[test]
    fn test_version_config_builders() {
        let config = VersionConfig::default()
            .with_max_versions(5)
            .with_versions_dir(PathBuf::from("/custom/path"));

        assert_eq!(config.max_versions, 5);
        assert_eq!(config.versions_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_version_history_new() {
        let history = VersionHistory::new("test-doc");

        assert_eq!(history.document_id, "test-doc");
        assert!(history.is_empty());
        assert_eq!(history.current_version, 0);
        assert_eq!(history.next_version, 1);
    }

    #[test]
    fn test_version_time_description() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Just now
        assert_eq!(
            DocumentVersion::format_time_description(now),
            "Just now"
        );

        // 5 minutes ago
        let five_mins = now - 5 * 60 * 1000;
        assert_eq!(
            DocumentVersion::format_time_description(five_mins),
            "5 minutes ago"
        );
    }

    #[tokio::test]
    async fn test_version_manager_create_version() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        let version = manager
            .create_version(&tree, Some("Initial version".to_string()))
            .await
            .unwrap();

        assert_eq!(version.version_number, 1);
        assert!(version.path.exists());
        assert_eq!(manager.history().len(), 1);
    }

    #[tokio::test]
    async fn test_version_manager_multiple_versions() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        // Create multiple versions
        for i in 1..=3 {
            manager
                .create_version(&tree, Some(format!("Version {}", i)))
                .await
                .unwrap();
        }

        assert_eq!(manager.history().len(), 3);
        assert_eq!(manager.history().current_version, 3);
    }

    #[tokio::test]
    async fn test_version_manager_load_version() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        manager.create_version(&tree, None).await.unwrap();

        let loaded = manager.load_version(1).await.unwrap();
        assert_eq!(loaded.root_id(), tree.root_id());
    }

    #[tokio::test]
    async fn test_version_manager_restore_version() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        manager.create_version(&tree, Some("V1".to_string())).await.unwrap();
        manager.create_version(&tree, Some("V2".to_string())).await.unwrap();

        // Restore to version 1
        let restored = manager.restore_version(1).await.unwrap();
        assert_eq!(restored.root_id(), tree.root_id());

        // Should now have 3 versions (including the restore)
        assert_eq!(manager.history().len(), 3);
    }

    #[tokio::test]
    async fn test_version_manager_compare_same_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        // Create two versions of the same document
        manager.create_version(&tree, None).await.unwrap();
        manager.create_version(&tree, None).await.unwrap();

        // Compare should complete without error
        let diff = manager.compare_versions(1, 2).await.unwrap();

        // The diff should have consistent version numbers
        assert_eq!(diff.old_version, 1);
        assert_eq!(diff.new_version, 2);

        // With identical tree structures saved twice:
        // - Paragraph counts should match (no ParagraphAdded/Removed)
        // - Content may differ slightly due to metadata/timestamps
        let has_structural_changes = diff.changes.iter().any(|c| {
            matches!(
                c,
                VersionChange::ParagraphAdded { .. }
                    | VersionChange::ParagraphRemoved { .. }
                    | VersionChange::ImageAdded { .. }
                    | VersionChange::ImageRemoved { .. }
                    | VersionChange::TableAdded { .. }
                    | VersionChange::TableRemoved { .. }
            )
        });
        assert!(!has_structural_changes);
    }

    #[tokio::test]
    async fn test_version_manager_compare_different() {
        use doc_model::Node;

        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);

        // Create first version
        let tree1 = DocumentTree::with_empty_paragraph();
        manager.create_version(&tree1, None).await.unwrap();

        // Create second version with different content
        let mut tree2 = DocumentTree::new();
        let para = doc_model::Paragraph::new();
        let para_id = para.id();
        tree2.nodes.paragraphs.insert(para_id, para);
        tree2.document.add_body_child(para_id);
        let para2 = doc_model::Paragraph::new();
        let para2_id = para2.id();
        tree2.nodes.paragraphs.insert(para2_id, para2);
        tree2.document.add_body_child(para2_id);

        manager.create_version(&tree2, None).await.unwrap();

        let diff = manager.compare_versions(1, 2).await.unwrap();
        assert!(!diff.is_identical);
        assert!(!diff.changes.is_empty());
    }

    #[tokio::test]
    async fn test_version_manager_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf())
            .with_max_versions(2);

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        // Create 4 versions
        for _ in 0..4 {
            manager.create_version(&tree, None).await.unwrap();
        }

        // Should only keep 2
        assert_eq!(manager.history().len(), 2);
    }

    #[tokio::test]
    async fn test_version_manager_save_load_history() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config.clone());
        let tree = DocumentTree::with_empty_paragraph();

        manager.create_version(&tree, Some("Test".to_string())).await.unwrap();

        // Create new manager and load history
        let mut manager2 = VersionManager::new("test-doc", config);
        manager2.load_history().await.unwrap();

        assert_eq!(manager2.history().len(), 1);
        assert_eq!(
            manager2.history().latest().unwrap().changes_summary,
            Some("Test".to_string())
        );
    }

    #[tokio::test]
    async fn test_version_manager_delete_all() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        manager.create_version(&tree, None).await.unwrap();
        manager.create_version(&tree, None).await.unwrap();

        manager.delete_all_versions().await.unwrap();

        assert!(manager.history().is_empty());
    }

    #[tokio::test]
    async fn test_version_manager_total_size() {
        let temp_dir = TempDir::new().unwrap();
        let config = VersionConfig::default()
            .with_versions_dir(temp_dir.path().to_path_buf());

        let mut manager = VersionManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        manager.create_version(&tree, None).await.unwrap();
        manager.create_version(&tree, None).await.unwrap();

        let total = manager.total_size();
        assert!(total > 0);
    }
}
