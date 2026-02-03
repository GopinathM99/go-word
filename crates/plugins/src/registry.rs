//! Plugin registry for discovery and management
//!
//! This module handles plugin discovery from the filesystem,
//! installation, uninstallation, and updates.

use crate::error::{PluginError, Result};
use crate::manifest::PluginManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin registry for managing installed plugins
#[derive(Debug)]
pub struct PluginRegistry {
    /// Base directory for plugins
    plugins_dir: PathBuf,
    /// Discovered plugins by ID
    discovered: HashMap<String, DiscoveredPlugin>,
    /// Plugin metadata cache
    metadata_cache: HashMap<String, PluginMetadata>,
}

/// A discovered plugin on the filesystem
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Path to the plugin directory
    pub path: PathBuf,
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Installation timestamp
    pub installed_at: Option<u64>,
}

impl DiscoveredPlugin {
    /// Create a new discovered plugin
    pub fn new(manifest: PluginManifest, path: PathBuf) -> Self {
        Self {
            manifest,
            path,
            enabled: true,
            installed_at: None,
        }
    }

    /// Get the plugin ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }
}

/// Metadata about an installed plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: String,
    /// Installed version
    pub version: String,
    /// Installation timestamp (Unix epoch)
    pub installed_at: u64,
    /// Last update timestamp
    pub updated_at: Option<u64>,
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// User-defined settings
    pub settings: HashMap<String, serde_json::Value>,
}

impl PluginMetadata {
    /// Create new metadata for a plugin
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            installed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: None,
            enabled: true,
            settings: HashMap::new(),
        }
    }
}

/// Information about an available plugin update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdate {
    /// Plugin ID
    pub plugin_id: String,
    /// Current version
    pub current_version: String,
    /// Available version
    pub available_version: String,
    /// Changelog or release notes
    pub changelog: Option<String>,
    /// Download URL
    pub download_url: Option<String>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            discovered: HashMap::new(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Get the plugins directory
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Discover all plugins in the plugins directory
    pub fn discover(&mut self) -> Result<Vec<String>> {
        self.discovered.clear();
        let mut discovered_ids = Vec::new();

        if !self.plugins_dir.exists() {
            return Ok(discovered_ids);
        }

        let entries = std::fs::read_dir(&self.plugins_dir)
            .map_err(|e| PluginError::Io(format!("Failed to read plugins directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                match self.discover_plugin(&path) {
                    Ok(plugin) => {
                        let id = plugin.manifest.id.clone();
                        self.discovered.insert(id.clone(), plugin);
                        discovered_ids.push(id);
                    }
                    Err(e) => {
                        // Log the error but continue discovering other plugins
                        eprintln!("Failed to discover plugin at {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(discovered_ids)
    }

    /// Discover a single plugin from a directory
    fn discover_plugin(&self, path: &Path) -> Result<DiscoveredPlugin> {
        let manifest_path = path.join("manifest.json");

        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::Io(format!("Failed to read manifest: {}", e)))?;

        let manifest: PluginManifest = serde_json::from_str(&content)?;
        manifest.validate()?;

        let mut plugin = DiscoveredPlugin::new(manifest, path.to_path_buf());

        // Check if we have cached metadata
        if let Some(metadata) = self.metadata_cache.get(&plugin.manifest.id) {
            plugin.enabled = metadata.enabled;
            plugin.installed_at = Some(metadata.installed_at);
        }

        Ok(plugin)
    }

    /// Get all discovered plugins
    pub fn get_discovered(&self) -> Vec<&DiscoveredPlugin> {
        self.discovered.values().collect()
    }

    /// Get a discovered plugin by ID
    pub fn get_plugin(&self, id: &str) -> Option<&DiscoveredPlugin> {
        self.discovered.get(id)
    }

    /// Check if a plugin is installed
    pub fn is_installed(&self, id: &str) -> bool {
        self.discovered.contains_key(id)
    }

    /// Install a plugin from a path
    pub fn install(&mut self, source_path: &Path) -> Result<String> {
        // Discover the plugin to validate it
        let plugin = self.discover_plugin(source_path)?;
        let plugin_id = plugin.manifest.id.clone();

        // Check if already installed
        if self.discovered.contains_key(&plugin_id) {
            return Err(PluginError::already_loaded(&plugin_id));
        }

        // Create destination directory
        let dest_dir = self.plugins_dir.join(&plugin_id);
        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir)
                .map_err(|e| PluginError::Io(format!("Failed to remove existing directory: {}", e)))?;
        }

        // Copy plugin files
        copy_dir_recursive(source_path, &dest_dir)?;

        // Create metadata
        let metadata = PluginMetadata::new(&plugin_id, &plugin.manifest.version);
        self.metadata_cache.insert(plugin_id.clone(), metadata);

        // Add to discovered
        let mut installed_plugin = DiscoveredPlugin::new(plugin.manifest, dest_dir);
        installed_plugin.installed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.discovered.insert(plugin_id.clone(), installed_plugin);

        Ok(plugin_id)
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .discovered
            .get(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        // Remove plugin directory
        if plugin.path.exists() {
            std::fs::remove_dir_all(&plugin.path)
                .map_err(|e| PluginError::Io(format!("Failed to remove plugin directory: {}", e)))?;
        }

        // Remove from discovered and metadata
        self.discovered.remove(id);
        self.metadata_cache.remove(id);

        Ok(())
    }

    /// Enable a plugin
    pub fn enable(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .discovered
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        plugin.enabled = true;

        if let Some(metadata) = self.metadata_cache.get_mut(id) {
            metadata.enabled = true;
        }

        Ok(())
    }

    /// Disable a plugin
    pub fn disable(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .discovered
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        plugin.enabled = false;

        if let Some(metadata) = self.metadata_cache.get_mut(id) {
            metadata.enabled = false;
        }

        Ok(())
    }

    /// Get enabled plugins
    pub fn get_enabled(&self) -> Vec<&DiscoveredPlugin> {
        self.discovered.values().filter(|p| p.enabled).collect()
    }

    /// Get disabled plugins
    pub fn get_disabled(&self) -> Vec<&DiscoveredPlugin> {
        self.discovered.values().filter(|p| !p.enabled).collect()
    }

    /// Update plugin metadata
    pub fn update_metadata<F>(&mut self, id: &str, f: F) -> Result<()>
    where
        F: FnOnce(&mut PluginMetadata),
    {
        let metadata = self
            .metadata_cache
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        f(metadata);
        Ok(())
    }

    /// Get plugin metadata
    pub fn get_metadata(&self, id: &str) -> Option<&PluginMetadata> {
        self.metadata_cache.get(id)
    }

    /// Export registry state for persistence
    pub fn export_state(&self) -> RegistryState {
        RegistryState {
            metadata: self.metadata_cache.clone(),
        }
    }

    /// Import registry state from persistence
    pub fn import_state(&mut self, state: RegistryState) {
        self.metadata_cache = state.metadata;
    }

    /// Get the count of installed plugins
    pub fn plugin_count(&self) -> usize {
        self.discovered.len()
    }

    /// Search plugins by name or description
    pub fn search(&self, query: &str) -> Vec<&DiscoveredPlugin> {
        let query_lower = query.to_lowercase();
        self.discovered
            .values()
            .filter(|p| {
                p.manifest.name.to_lowercase().contains(&query_lower)
                    || p.manifest.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

/// Serializable registry state for persistence
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryState {
    /// Plugin metadata by ID
    pub metadata: HashMap<String, PluginMetadata>,
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)
        .map_err(|e| PluginError::Io(format!("Failed to create directory: {}", e)))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| PluginError::Io(format!("Failed to read directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| PluginError::Io(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)
                .map_err(|e| PluginError::Io(format!("Failed to copy file: {}", e)))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_plugin_dir(dir: &Path, id: &str) -> PathBuf {
        let plugin_dir = dir.join(id);
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest = PluginManifest::new(id, "Test Plugin", "1.0.0", "Test Author");
        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("manifest.json"), manifest_json).unwrap();

        plugin_dir
    }

    #[test]
    fn test_registry_new() {
        let registry = PluginRegistry::new("/path/to/plugins");
        assert_eq!(registry.plugins_dir(), Path::new("/path/to/plugins"));
        assert_eq!(registry.plugin_count(), 0);
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path());

        let discovered = registry.discover().unwrap();
        assert!(discovered.is_empty());
    }

    #[test]
    fn test_discover_plugins() {
        let temp_dir = tempdir().unwrap();
        create_test_plugin_dir(temp_dir.path(), "com.test.plugin1");
        create_test_plugin_dir(temp_dir.path(), "com.test.plugin2");

        let mut registry = PluginRegistry::new(temp_dir.path());
        let discovered = registry.discover().unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(registry.is_installed("com.test.plugin1"));
        assert!(registry.is_installed("com.test.plugin2"));
    }

    #[test]
    fn test_get_discovered() {
        let temp_dir = tempdir().unwrap();
        create_test_plugin_dir(temp_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.discover().unwrap();

        let plugins = registry.get_discovered();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.id, "com.test.plugin");
    }

    #[test]
    fn test_get_plugin() {
        let temp_dir = tempdir().unwrap();
        create_test_plugin_dir(temp_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.discover().unwrap();

        let plugin = registry.get_plugin("com.test.plugin");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().manifest.id, "com.test.plugin");

        assert!(registry.get_plugin("nonexistent").is_none());
    }

    #[test]
    fn test_enable_disable() {
        let temp_dir = tempdir().unwrap();
        create_test_plugin_dir(temp_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.discover().unwrap();

        // Initially enabled
        assert!(registry.get_plugin("com.test.plugin").unwrap().enabled);
        assert_eq!(registry.get_enabled().len(), 1);
        assert_eq!(registry.get_disabled().len(), 0);

        // Disable
        registry.disable("com.test.plugin").unwrap();
        assert!(!registry.get_plugin("com.test.plugin").unwrap().enabled);
        assert_eq!(registry.get_enabled().len(), 0);
        assert_eq!(registry.get_disabled().len(), 1);

        // Enable
        registry.enable("com.test.plugin").unwrap();
        assert!(registry.get_plugin("com.test.plugin").unwrap().enabled);
    }

    #[test]
    fn test_install_plugin() {
        let temp_dir = tempdir().unwrap();
        let source_dir = tempdir().unwrap();

        // Create a plugin in the source directory
        let plugin_dir = create_test_plugin_dir(source_dir.path(), "com.test.new");

        let mut registry = PluginRegistry::new(temp_dir.path());

        // Install
        let id = registry.install(&plugin_dir).unwrap();
        assert_eq!(id, "com.test.new");
        assert!(registry.is_installed("com.test.new"));

        // Verify files were copied
        let installed_path = temp_dir.path().join("com.test.new");
        assert!(installed_path.exists());
        assert!(installed_path.join("manifest.json").exists());
    }

    #[test]
    fn test_install_duplicate() {
        let temp_dir = tempdir().unwrap();
        let source_dir = tempdir().unwrap();

        let plugin_dir = create_test_plugin_dir(source_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.install(&plugin_dir).unwrap();

        // Try to install again
        let result = registry.install(&plugin_dir);
        assert!(matches!(result, Err(PluginError::AlreadyLoaded(_))));
    }

    #[test]
    fn test_uninstall_plugin() {
        let temp_dir = tempdir().unwrap();
        create_test_plugin_dir(temp_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.discover().unwrap();

        registry.uninstall("com.test.plugin").unwrap();
        assert!(!registry.is_installed("com.test.plugin"));
        assert!(!temp_dir.path().join("com.test.plugin").exists());
    }

    #[test]
    fn test_uninstall_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path());

        let result = registry.uninstall("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata::new("com.test.plugin", "1.0.0");

        assert_eq!(metadata.id, "com.test.plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.enabled);
        assert!(metadata.installed_at > 0);
    }

    #[test]
    fn test_update_metadata() {
        let temp_dir = tempdir().unwrap();
        let source_dir = tempdir().unwrap();

        let plugin_dir = create_test_plugin_dir(source_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.install(&plugin_dir).unwrap();

        registry.update_metadata("com.test.plugin", |meta| {
            meta.settings.insert("key".to_string(), serde_json::json!("value"));
        }).unwrap();

        let meta = registry.get_metadata("com.test.plugin").unwrap();
        assert!(meta.settings.contains_key("key"));
    }

    #[test]
    fn test_export_import_state() {
        let temp_dir = tempdir().unwrap();
        let source_dir = tempdir().unwrap();

        let plugin_dir = create_test_plugin_dir(source_dir.path(), "com.test.plugin");

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.install(&plugin_dir).unwrap();
        registry.disable("com.test.plugin").unwrap();

        let state = registry.export_state();

        let mut new_registry = PluginRegistry::new(temp_dir.path());
        new_registry.import_state(state);

        let meta = new_registry.get_metadata("com.test.plugin").unwrap();
        assert!(!meta.enabled);
    }

    #[test]
    fn test_search_plugins() {
        let temp_dir = tempdir().unwrap();

        // Create plugins with different names
        let plugin1_dir = temp_dir.path().join("com.test.spellcheck");
        fs::create_dir_all(&plugin1_dir).unwrap();
        let manifest1 = PluginManifest::new(
            "com.test.spellcheck",
            "Spell Checker",
            "1.0.0",
            "Test",
        ).with_description("Check spelling in documents");
        fs::write(
            plugin1_dir.join("manifest.json"),
            serde_json::to_string(&manifest1).unwrap(),
        ).unwrap();

        let plugin2_dir = temp_dir.path().join("com.test.grammar");
        fs::create_dir_all(&plugin2_dir).unwrap();
        let manifest2 = PluginManifest::new(
            "com.test.grammar",
            "Grammar Helper",
            "1.0.0",
            "Test",
        ).with_description("Fix grammar issues");
        fs::write(
            plugin2_dir.join("manifest.json"),
            serde_json::to_string(&manifest2).unwrap(),
        ).unwrap();

        let mut registry = PluginRegistry::new(temp_dir.path());
        registry.discover().unwrap();

        // Search by name
        let results = registry.search("spell");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manifest.id, "com.test.spellcheck");

        // Search by description
        let results = registry.search("grammar");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manifest.id, "com.test.grammar");

        // Search with no results
        let results = registry.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_discovered_plugin() {
        let manifest = PluginManifest::new(
            "com.test.plugin",
            "Test",
            "1.0.0",
            "Author",
        );
        let plugin = DiscoveredPlugin::new(manifest, PathBuf::from("/path"));

        assert_eq!(plugin.id(), "com.test.plugin");
        assert!(plugin.enabled);
    }

    #[test]
    fn test_registry_state_serialization() {
        let mut state = RegistryState::default();
        state.metadata.insert(
            "test".to_string(),
            PluginMetadata::new("test", "1.0.0"),
        );

        let json = serde_json::to_string(&state).unwrap();
        let parsed: RegistryState = serde_json::from_str(&json).unwrap();

        assert!(parsed.metadata.contains_key("test"));
    }
}
