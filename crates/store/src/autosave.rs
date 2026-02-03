//! Autosave functionality with configurable intervals, debouncing, and background saving
//!
//! This module provides automatic document saving to a recovery location,
//! separate from the original file, to prevent data loss.

use crate::Result;
use doc_model::DocumentTree;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Autosave configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveConfig {
    /// Whether autosave is enabled
    pub enabled: bool,
    /// Interval between autosaves in seconds (default: 300 = 5 minutes)
    pub interval_secs: u64,
    /// Maximum number of autosave versions to keep per document
    pub max_versions: usize,
    /// Directory for autosave files
    pub location: PathBuf,
    /// Minimum time between saves to debounce rapid changes (in milliseconds)
    pub debounce_ms: u64,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300, // 5 minutes
            max_versions: 5,
            location: PathBuf::from(".autosave"),
            debounce_ms: 1000, // 1 second debounce
        }
    }
}

impl AutosaveConfig {
    /// Create a new config with custom interval
    pub fn with_interval(mut self, secs: u64) -> Self {
        self.interval_secs = secs;
        self
    }

    /// Create a new config with custom location
    pub fn with_location(mut self, location: PathBuf) -> Self {
        self.location = location;
        self
    }

    /// Create a new config with autosave disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

/// Current autosave status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveStatus {
    /// Whether autosave is enabled
    pub enabled: bool,
    /// Whether there are unsaved changes
    pub has_unsaved_changes: bool,
    /// Whether a save is currently in progress
    pub is_saving: bool,
    /// Timestamp of last successful save (Unix timestamp in ms)
    pub last_save_time: Option<u64>,
    /// Error message from last save attempt (if any)
    pub last_error: Option<String>,
    /// Time until next scheduled autosave (in seconds)
    pub next_save_in_secs: Option<u64>,
}

/// Autosave manager with debouncing and background saving
pub struct AutosaveManager {
    /// Configuration
    config: AutosaveConfig,
    /// Document ID being autosaved
    document_id: String,
    /// Original file path (if known)
    original_path: Option<PathBuf>,
    /// Whether there are unsaved changes
    dirty: Arc<AtomicBool>,
    /// Last time the document was marked dirty
    last_dirty_time: Arc<RwLock<Option<Instant>>>,
    /// Last successful save time (Unix timestamp in ms)
    last_save_time: Arc<AtomicU64>,
    /// Whether a save is in progress
    is_saving: Arc<AtomicBool>,
    /// Last error message
    last_error: Arc<RwLock<Option<String>>>,
    /// Change counter for debouncing
    change_counter: Arc<AtomicU64>,
}

impl AutosaveManager {
    /// Create a new autosave manager
    pub fn new(document_id: impl Into<String>, config: AutosaveConfig) -> Self {
        Self {
            config,
            document_id: document_id.into(),
            original_path: None,
            dirty: Arc::new(AtomicBool::new(false)),
            last_dirty_time: Arc::new(RwLock::new(None)),
            last_save_time: Arc::new(AtomicU64::new(0)),
            is_saving: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(RwLock::new(None)),
            change_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Set the original file path
    pub fn set_original_path(&mut self, path: Option<PathBuf>) {
        self.original_path = path;
    }

    /// Get the original file path
    pub fn original_path(&self) -> Option<&PathBuf> {
        self.original_path.as_ref()
    }

    /// Get the document ID
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Get the configuration
    pub fn config(&self) -> &AutosaveConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: AutosaveConfig) {
        self.config = config;
    }

    /// Mark the document as dirty (has unsaved changes)
    /// This is debounced to avoid excessive saves on rapid changes
    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::SeqCst);
        self.change_counter.fetch_add(1, Ordering::SeqCst);

        // Update last dirty time in background
        let last_dirty = self.last_dirty_time.clone();
        tokio::spawn(async move {
            let mut guard = last_dirty.write().await;
            *guard = Some(Instant::now());
        });
    }

    /// Mark the document as clean (saved)
    pub fn mark_clean(&self) {
        self.dirty.store(false, Ordering::SeqCst);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_save_time.store(now, Ordering::SeqCst);
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    /// Check if a save is in progress
    pub fn is_saving(&self) -> bool {
        self.is_saving.load(Ordering::SeqCst)
    }

    /// Get the current autosave status
    pub async fn status(&self) -> AutosaveStatus {
        let last_save = self.last_save_time.load(Ordering::SeqCst);
        let last_dirty = self.last_dirty_time.read().await;

        let next_save_in_secs = if self.config.enabled && self.dirty.load(Ordering::SeqCst) {
            if let Some(dirty_time) = *last_dirty {
                let elapsed = dirty_time.elapsed().as_secs();
                if elapsed < self.config.interval_secs {
                    Some(self.config.interval_secs - elapsed)
                } else {
                    Some(0)
                }
            } else {
                None
            }
        } else {
            None
        };

        AutosaveStatus {
            enabled: self.config.enabled,
            has_unsaved_changes: self.dirty.load(Ordering::SeqCst),
            is_saving: self.is_saving.load(Ordering::SeqCst),
            last_save_time: if last_save > 0 { Some(last_save) } else { None },
            last_error: self.last_error.read().await.clone(),
            next_save_in_secs,
        }
    }

    /// Get the autosave file path for this document
    pub fn autosave_path(&self) -> PathBuf {
        self.config
            .location
            .join(format!("{}.autosave.wdj", self.document_id))
    }

    /// Get the autosave metadata file path
    pub fn metadata_path(&self) -> PathBuf {
        self.config
            .location
            .join(format!("{}.autosave.meta", self.document_id))
    }

    /// Check if debounce period has passed since last change
    async fn should_save_now(&self) -> bool {
        if !self.config.enabled || !self.dirty.load(Ordering::SeqCst) {
            return false;
        }

        let last_dirty = self.last_dirty_time.read().await;
        if let Some(dirty_time) = *last_dirty {
            let elapsed_ms = dirty_time.elapsed().as_millis() as u64;
            elapsed_ms >= self.config.debounce_ms
        } else {
            false
        }
    }

    /// Perform an autosave if dirty and debounce period has passed
    /// Returns true if a save was performed, false otherwise
    pub async fn autosave(&self, tree: &DocumentTree) -> Result<bool> {
        // Check if we should save
        if !self.should_save_now().await {
            return Ok(false);
        }

        // Check if already saving
        if self.is_saving.swap(true, Ordering::SeqCst) {
            return Ok(false);
        }

        // Ensure autosave directory exists
        tokio::fs::create_dir_all(&self.config.location).await?;

        // Save to autosave file
        let result = self.save_to_recovery(tree).await;

        // Update state
        self.is_saving.store(false, Ordering::SeqCst);

        match result {
            Ok(()) => {
                self.mark_clean();
                let mut error = self.last_error.write().await;
                *error = None;
                Ok(true)
            }
            Err(e) => {
                let mut error = self.last_error.write().await;
                *error = Some(e.to_string());
                Err(e)
            }
        }
    }

    /// Save to recovery location with metadata
    async fn save_to_recovery(&self, tree: &DocumentTree) -> Result<()> {
        // Create metadata
        let metadata = AutosaveMetadata {
            document_id: self.document_id.clone(),
            original_path: self.original_path.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            version: tree.document.version(),
        };

        // Save metadata
        let meta_json = serde_json::to_string_pretty(&metadata)?;
        tokio::fs::write(self.metadata_path(), meta_json).await?;

        // Save document
        crate::save_document(tree, self.autosave_path()).await?;

        // Clean up old versions if needed
        self.cleanup_old_versions().await?;

        Ok(())
    }

    /// Clean up old autosave versions beyond max_versions
    async fn cleanup_old_versions(&self) -> Result<()> {
        // For now, we only keep one version per document
        // Future: implement version rotation
        Ok(())
    }

    /// Clean up autosave file after successful save to original location
    pub async fn cleanup(&self) -> Result<()> {
        let path = self.autosave_path();
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }

        let meta_path = self.metadata_path();
        if meta_path.exists() {
            tokio::fs::remove_file(&meta_path).await?;
        }

        Ok(())
    }

    /// Check if an autosave file exists for recovery
    pub fn has_recovery(&self) -> bool {
        self.autosave_path().exists()
    }

    /// Load the autosave file for recovery
    pub async fn recover(&self) -> Result<DocumentTree> {
        crate::load_document(self.autosave_path()).await
    }

    /// Start the autosave background task
    /// Returns a handle that can be used to stop the task
    pub fn start_background_task(
        self: Arc<Self>,
        tree: Arc<RwLock<DocumentTree>>,
    ) -> tokio::task::JoinHandle<()> {
        let manager = self;

        tokio::spawn(async move {
            let interval = Duration::from_secs(manager.config.interval_secs);
            let mut last_change_count = manager.change_counter.load(Ordering::SeqCst);

            loop {
                tokio::time::sleep(interval).await;

                if !manager.config.enabled {
                    continue;
                }

                // Check if there were any changes
                let current_count = manager.change_counter.load(Ordering::SeqCst);
                if current_count == last_change_count {
                    continue;
                }
                last_change_count = current_count;

                // Perform autosave
                let tree_guard = tree.read().await;
                if let Err(e) = manager.autosave(&tree_guard).await {
                    tracing::warn!("Autosave failed: {}", e);
                }
            }
        })
    }
}

/// Metadata stored alongside autosave files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveMetadata {
    /// Document ID
    pub document_id: String,
    /// Original file path (if known)
    pub original_path: Option<PathBuf>,
    /// Timestamp when autosave was created (Unix timestamp in ms)
    pub timestamp: u64,
    /// Document version at time of save
    pub version: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_autosave_config_default() {
        let config = AutosaveConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 300);
        assert_eq!(config.max_versions, 5);
        assert_eq!(config.debounce_ms, 1000);
    }

    #[test]
    fn test_autosave_config_disabled() {
        let config = AutosaveConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_autosave_config_builders() {
        let config = AutosaveConfig::default()
            .with_interval(60)
            .with_location(PathBuf::from("/custom/path"));

        assert_eq!(config.interval_secs, 60);
        assert_eq!(config.location, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_autosave_manager_new() {
        let config = AutosaveConfig::default();
        let manager = AutosaveManager::new("test-doc", config);

        assert_eq!(manager.document_id(), "test-doc");
        assert!(!manager.is_dirty());
        assert!(!manager.is_saving());
    }

    #[tokio::test]
    async fn test_autosave_manager_dirty_state() {
        let config = AutosaveConfig::default();
        let manager = AutosaveManager::new("test-doc", config);

        assert!(!manager.is_dirty());
        manager.mark_dirty();
        // Give tokio a chance to update the state
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(manager.is_dirty());
        manager.mark_clean();
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_autosave_path() {
        let config = AutosaveConfig::default().with_location(PathBuf::from("/tmp/autosave"));
        let manager = AutosaveManager::new("doc-123", config);

        assert_eq!(
            manager.autosave_path(),
            PathBuf::from("/tmp/autosave/doc-123.autosave.wdj")
        );
    }

    #[tokio::test]
    async fn test_autosave_full_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutosaveConfig::default()
            .with_location(temp_dir.path().to_path_buf())
            .with_interval(1);

        let manager = AutosaveManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        // Initially not dirty - should not save
        let saved = manager.autosave(&tree).await.unwrap();
        assert!(!saved);

        // Mark dirty
        manager.mark_dirty();

        // Wait for debounce
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should save now
        let saved = manager.autosave(&tree).await.unwrap();
        assert!(saved);
        assert!(!manager.is_dirty());
        assert!(manager.autosave_path().exists());
        assert!(manager.metadata_path().exists());

        // Cleanup
        manager.cleanup().await.unwrap();
        assert!(!manager.autosave_path().exists());
        assert!(!manager.metadata_path().exists());
    }

    #[tokio::test]
    async fn test_autosave_status() {
        let config = AutosaveConfig::default();
        let manager = AutosaveManager::new("test-doc", config);

        let status = manager.status().await;
        assert!(status.enabled);
        assert!(!status.has_unsaved_changes);
        assert!(!status.is_saving);
        assert!(status.last_save_time.is_none());
        assert!(status.last_error.is_none());

        manager.mark_dirty();
        let status = manager.status().await;
        assert!(status.has_unsaved_changes);
    }

    #[tokio::test]
    async fn test_autosave_recover() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutosaveConfig::default()
            .with_location(temp_dir.path().to_path_buf());

        let manager = AutosaveManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        // Save
        manager.mark_dirty();
        tokio::time::sleep(Duration::from_millis(1100)).await;
        manager.autosave(&tree).await.unwrap();

        // Verify recovery is available
        assert!(manager.has_recovery());

        // Recover
        let recovered = manager.recover().await.unwrap();
        assert_eq!(recovered.root_id(), tree.root_id());
    }

    #[test]
    fn test_autosave_metadata_serialization() {
        let metadata = AutosaveMetadata {
            document_id: "test-doc".to_string(),
            original_path: Some(PathBuf::from("/path/to/doc.wdj")),
            timestamp: 1234567890000,
            version: 42,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: AutosaveMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.document_id, metadata.document_id);
        assert_eq!(parsed.original_path, metadata.original_path);
        assert_eq!(parsed.timestamp, metadata.timestamp);
        assert_eq!(parsed.version, metadata.version);
    }

    #[tokio::test]
    async fn test_autosave_debounce() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutosaveConfig {
            enabled: true,
            interval_secs: 1,
            max_versions: 5,
            location: temp_dir.path().to_path_buf(),
            debounce_ms: 500,
        };

        let manager = AutosaveManager::new("test-doc", config);
        let tree = DocumentTree::with_empty_paragraph();

        // Mark dirty
        manager.mark_dirty();

        // Try to save immediately - should be debounced
        let saved = manager.autosave(&tree).await.unwrap();
        assert!(!saved);

        // Wait for debounce period
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Now it should save
        let saved = manager.autosave(&tree).await.unwrap();
        assert!(saved);
    }

    #[test]
    fn test_original_path() {
        let config = AutosaveConfig::default();
        let mut manager = AutosaveManager::new("test-doc", config);

        assert!(manager.original_path().is_none());

        manager.set_original_path(Some(PathBuf::from("/path/to/doc.wdj")));
        assert_eq!(
            manager.original_path(),
            Some(&PathBuf::from("/path/to/doc.wdj"))
        );
    }
}
