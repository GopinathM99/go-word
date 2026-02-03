//! Recovery system for crash recovery
//!
//! This module provides functionality to detect and recover from application crashes
//! by managing recovery files and detecting orphaned autosave data.

use crate::{AutosaveMetadata, Result, StoreError};
use doc_model::DocumentTree;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for the recovery system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Directory where recovery files are stored
    pub recovery_dir: PathBuf,
    /// How long to keep recovery files (in seconds)
    pub retention_secs: u64,
    /// Whether to automatically clean up old recovery files
    pub auto_cleanup: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            recovery_dir: PathBuf::from(".autosave"),
            retention_secs: 7 * 24 * 60 * 60, // 7 days
            auto_cleanup: true,
        }
    }
}

impl RecoveryConfig {
    /// Create a config with a custom recovery directory
    pub fn with_recovery_dir(mut self, dir: PathBuf) -> Self {
        self.recovery_dir = dir;
        self
    }

    /// Create a config with a custom retention period
    pub fn with_retention(mut self, secs: u64) -> Self {
        self.retention_secs = secs;
        self
    }
}

/// Information about a recoverable file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryFile {
    /// Unique identifier for this recovery file
    pub id: String,
    /// Document ID from the original document
    pub document_id: String,
    /// Timestamp when the recovery file was created (Unix timestamp in ms)
    pub timestamp: u64,
    /// Path to the recovery file
    pub path: PathBuf,
    /// Original file path (if known)
    pub original_path: Option<PathBuf>,
    /// Human-readable description of when the file was created
    pub time_description: String,
    /// Size of the recovery file in bytes
    pub file_size: u64,
}

impl RecoveryFile {
    /// Create a new RecoveryFile from metadata and paths
    fn from_metadata(
        metadata: &AutosaveMetadata,
        recovery_path: PathBuf,
        file_size: u64,
    ) -> Self {
        let time_desc = Self::format_time_description(metadata.timestamp);

        Self {
            id: format!("{}_{}", metadata.document_id, metadata.timestamp),
            document_id: metadata.document_id.clone(),
            timestamp: metadata.timestamp,
            path: recovery_path,
            original_path: metadata.original_path.clone(),
            time_description: time_desc,
            file_size,
        }
    }

    /// Format a timestamp into a human-readable description
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

/// Recovery manager for crash recovery
pub struct RecoveryManager {
    /// Configuration
    config: RecoveryConfig,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new(config: RecoveryConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &RecoveryConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: RecoveryConfig) {
        self.config = config;
    }

    /// Check if there are any recovery files available (crash detection)
    pub async fn has_recovery_files(&self) -> bool {
        if !self.config.recovery_dir.exists() {
            return false;
        }

        match self.list_recovery_files().await {
            Ok(files) => !files.is_empty(),
            Err(_) => false,
        }
    }

    /// List all available recovery files
    pub async fn list_recovery_files(&self) -> Result<Vec<RecoveryFile>> {
        let mut recovery_files = Vec::new();

        if !self.config.recovery_dir.exists() {
            return Ok(recovery_files);
        }

        let mut entries = tokio::fs::read_dir(&self.config.recovery_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Look for metadata files
            if path.extension().map_or(false, |ext| ext == "meta") {
                if let Ok(recovery_file) = self.load_recovery_info(&path).await {
                    recovery_files.push(recovery_file);
                }
            }
        }

        // Sort by timestamp, most recent first
        recovery_files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(recovery_files)
    }

    /// Load recovery info from a metadata file
    async fn load_recovery_info(&self, meta_path: &PathBuf) -> Result<RecoveryFile> {
        // Read metadata
        let content = tokio::fs::read_to_string(meta_path).await?;
        let metadata: AutosaveMetadata = serde_json::from_str(&content)?;

        // Find the corresponding document file
        let doc_path = meta_path.with_extension("wdj");
        if !doc_path.exists() {
            return Err(StoreError::FileNotFound(doc_path.display().to_string()));
        }

        // Get file size
        let file_meta = tokio::fs::metadata(&doc_path).await?;
        let file_size = file_meta.len();

        Ok(RecoveryFile::from_metadata(&metadata, doc_path, file_size))
    }

    /// Get a specific recovery file by ID
    pub async fn get_recovery_file(&self, recovery_id: &str) -> Result<Option<RecoveryFile>> {
        let files = self.list_recovery_files().await?;
        Ok(files.into_iter().find(|f| f.id == recovery_id))
    }

    /// Recover a document from a recovery file
    pub async fn recover_document(&self, recovery_id: &str) -> Result<DocumentTree> {
        let file = self
            .get_recovery_file(recovery_id)
            .await?
            .ok_or_else(|| StoreError::FileNotFound(recovery_id.to_string()))?;

        crate::load_document(&file.path).await
    }

    /// Discard a recovery file (delete it)
    pub async fn discard_recovery(&self, recovery_id: &str) -> Result<()> {
        let file = self
            .get_recovery_file(recovery_id)
            .await?
            .ok_or_else(|| StoreError::FileNotFound(recovery_id.to_string()))?;

        // Delete the document file
        if file.path.exists() {
            tokio::fs::remove_file(&file.path).await?;
        }

        // Delete the metadata file
        let meta_path = file.path.with_extension("meta");
        if meta_path.exists() {
            tokio::fs::remove_file(&meta_path).await?;
        }

        Ok(())
    }

    /// Discard all recovery files
    pub async fn discard_all_recovery(&self) -> Result<()> {
        let files = self.list_recovery_files().await?;

        for file in files {
            self.discard_recovery(&file.id).await?;
        }

        Ok(())
    }

    /// Clean up old recovery files based on retention policy
    pub async fn cleanup_old_files(&self) -> Result<usize> {
        if !self.config.auto_cleanup {
            return Ok(0);
        }

        let files = self.list_recovery_files().await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let retention_ms = self.config.retention_secs * 1000;
        let cutoff = now.saturating_sub(retention_ms);

        let mut cleaned = 0;

        for file in files {
            if file.timestamp < cutoff {
                if let Ok(()) = self.discard_recovery(&file.id).await {
                    cleaned += 1;
                }
            }
        }

        Ok(cleaned)
    }

    /// Check for orphaned recovery files (crash detection on startup)
    /// Returns true if orphaned files were found, indicating a crash
    pub async fn detect_crash(&self) -> bool {
        // If there are any recovery files, it might indicate a crash
        // (normal shutdown should clean up recovery files)
        self.has_recovery_files().await
    }

    /// Get recovery files for a specific document
    pub async fn get_recovery_files_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<RecoveryFile>> {
        let files = self.list_recovery_files().await?;
        Ok(files
            .into_iter()
            .filter(|f| f.document_id == document_id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutosaveConfig;
    use tempfile::TempDir;

    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert_eq!(config.retention_secs, 7 * 24 * 60 * 60);
        assert!(config.auto_cleanup);
    }

    #[test]
    fn test_recovery_config_builders() {
        let config = RecoveryConfig::default()
            .with_recovery_dir(PathBuf::from("/custom/path"))
            .with_retention(3600);

        assert_eq!(config.recovery_dir, PathBuf::from("/custom/path"));
        assert_eq!(config.retention_secs, 3600);
    }

    #[test]
    fn test_recovery_file_time_description() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Just now
        assert_eq!(RecoveryFile::format_time_description(now), "Just now");

        // 5 minutes ago
        let five_mins_ago = now - 5 * 60 * 1000;
        assert_eq!(
            RecoveryFile::format_time_description(five_mins_ago),
            "5 minutes ago"
        );

        // 1 minute ago
        let one_min_ago = now - 60 * 1000;
        assert_eq!(
            RecoveryFile::format_time_description(one_min_ago),
            "1 minute ago"
        );

        // 2 hours ago
        let two_hours_ago = now - 2 * 60 * 60 * 1000;
        assert_eq!(
            RecoveryFile::format_time_description(two_hours_ago),
            "2 hours ago"
        );

        // 3 days ago
        let three_days_ago = now - 3 * 24 * 60 * 60 * 1000;
        assert_eq!(
            RecoveryFile::format_time_description(three_days_ago),
            "3 days ago"
        );
    }

    #[tokio::test]
    async fn test_recovery_manager_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf());

        let manager = RecoveryManager::new(config);

        assert!(!manager.has_recovery_files().await);
        assert!(manager.list_recovery_files().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_recovery_manager_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf());

        // Create a recovery file using autosave
        let autosave_config = AutosaveConfig::default()
            .with_location(temp_dir.path().to_path_buf());
        let autosave = crate::AutosaveManager::new("test-doc", autosave_config);

        let tree = doc_model::DocumentTree::with_empty_paragraph();
        autosave.mark_dirty();
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        autosave.autosave(&tree).await.unwrap();

        // Now check recovery
        let manager = RecoveryManager::new(config);
        assert!(manager.has_recovery_files().await);

        let files = manager.list_recovery_files().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].document_id, "test-doc");
    }

    #[tokio::test]
    async fn test_recovery_and_discard() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf());

        // Create a recovery file
        let autosave_config = AutosaveConfig::default()
            .with_location(temp_dir.path().to_path_buf());
        let autosave = crate::AutosaveManager::new("test-doc", autosave_config);

        let tree = doc_model::DocumentTree::with_empty_paragraph();
        autosave.mark_dirty();
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        autosave.autosave(&tree).await.unwrap();

        let manager = RecoveryManager::new(config);

        // Get the recovery file
        let files = manager.list_recovery_files().await.unwrap();
        let recovery_id = &files[0].id;

        // Recover the document
        let recovered = manager.recover_document(recovery_id).await.unwrap();
        assert_eq!(recovered.root_id(), tree.root_id());

        // Discard the recovery
        manager.discard_recovery(recovery_id).await.unwrap();

        // Should be gone now
        assert!(!manager.has_recovery_files().await);
    }

    #[tokio::test]
    async fn test_recovery_cleanup_old_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf())
            .with_retention(0); // Immediate cleanup

        // Create a recovery file
        let autosave_config = AutosaveConfig::default()
            .with_location(temp_dir.path().to_path_buf());
        let autosave = crate::AutosaveManager::new("test-doc", autosave_config);

        let tree = doc_model::DocumentTree::with_empty_paragraph();
        autosave.mark_dirty();
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        autosave.autosave(&tree).await.unwrap();

        let manager = RecoveryManager::new(config);

        // Should have 1 file
        assert!(manager.has_recovery_files().await);

        // Wait a bit and cleanup
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let cleaned = manager.cleanup_old_files().await.unwrap();
        assert_eq!(cleaned, 1);

        // Should be gone now
        assert!(!manager.has_recovery_files().await);
    }

    #[tokio::test]
    async fn test_detect_crash() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf());

        let manager = RecoveryManager::new(config.clone());

        // No crash detected initially
        assert!(!manager.detect_crash().await);

        // Create a recovery file (simulating crash)
        let autosave_config = AutosaveConfig::default()
            .with_location(temp_dir.path().to_path_buf());
        let autosave = crate::AutosaveManager::new("test-doc", autosave_config);

        let tree = doc_model::DocumentTree::with_empty_paragraph();
        autosave.mark_dirty();
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        autosave.autosave(&tree).await.unwrap();

        // Now crash should be detected
        let manager = RecoveryManager::new(config);
        assert!(manager.detect_crash().await);
    }

    #[tokio::test]
    async fn test_get_recovery_files_for_document() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf());

        // Create recovery files for two documents
        for doc_id in &["doc-1", "doc-2"] {
            let autosave_config = AutosaveConfig::default()
                .with_location(temp_dir.path().to_path_buf());
            let autosave = crate::AutosaveManager::new(*doc_id, autosave_config);

            let tree = doc_model::DocumentTree::with_empty_paragraph();
            autosave.mark_dirty();
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
            autosave.autosave(&tree).await.unwrap();
        }

        let manager = RecoveryManager::new(config);

        let doc1_files = manager
            .get_recovery_files_for_document("doc-1")
            .await
            .unwrap();
        assert_eq!(doc1_files.len(), 1);
        assert_eq!(doc1_files[0].document_id, "doc-1");

        let doc2_files = manager
            .get_recovery_files_for_document("doc-2")
            .await
            .unwrap();
        assert_eq!(doc2_files.len(), 1);
        assert_eq!(doc2_files[0].document_id, "doc-2");

        let nonexistent_files = manager
            .get_recovery_files_for_document("doc-3")
            .await
            .unwrap();
        assert!(nonexistent_files.is_empty());
    }

    #[tokio::test]
    async fn test_discard_all_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecoveryConfig::default()
            .with_recovery_dir(temp_dir.path().to_path_buf());

        // Create multiple recovery files
        for doc_id in &["doc-1", "doc-2", "doc-3"] {
            let autosave_config = AutosaveConfig::default()
                .with_location(temp_dir.path().to_path_buf());
            let autosave = crate::AutosaveManager::new(*doc_id, autosave_config);

            let tree = doc_model::DocumentTree::with_empty_paragraph();
            autosave.mark_dirty();
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
            autosave.autosave(&tree).await.unwrap();
        }

        let manager = RecoveryManager::new(config);

        // Should have 3 files
        let files = manager.list_recovery_files().await.unwrap();
        assert_eq!(files.len(), 3);

        // Discard all
        manager.discard_all_recovery().await.unwrap();

        // Should be empty
        assert!(!manager.has_recovery_files().await);
    }
}
