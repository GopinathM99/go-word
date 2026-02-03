//! Plugin Installation Manager
//!
//! Handles downloading, verifying, installing, updating, and uninstalling plugins.
//! Manages the plugin directory structure and configuration persistence.

use crate::error::{PluginError, Result};
use crate::manifest::PluginManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// State of a plugin installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationState {
    Downloading, Verifying, Installing, Installed,
    Updating, Uninstalling, Failed, Disabled,
}

/// Record of an installed plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub state: InstallationState,
    pub install_path: PathBuf,
    pub installed_at: u64,
    pub updated_at: Option<u64>,
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub auto_update: bool,
}

impl InstalledPlugin {
    pub fn new(manifest: PluginManifest, install_path: PathBuf) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        Self { manifest, state: InstallationState::Installed, install_path, installed_at: now, updated_at: None, enabled: true, settings: HashMap::new(), auto_update: true }
    }
    pub fn is_active(&self) -> bool { self.state == InstallationState::Installed && self.enabled }
    pub fn id(&self) -> &str { &self.manifest.id }
    pub fn version(&self) -> &str { &self.manifest.version }
    pub fn disable(&mut self) { self.enabled = false; }
    pub fn enable(&mut self) { self.enabled = true; }
    pub fn set_setting(&mut self, key: impl Into<String>, value: serde_json::Value) { self.settings.insert(key.into(), value); }
    pub fn get_setting(&self, key: &str) -> Option<&serde_json::Value> { self.settings.get(key) }
    pub fn remove_setting(&mut self, key: &str) -> Option<serde_json::Value> { self.settings.remove(key) }
}

pub struct InstallationManager { plugins_dir: PathBuf, index_path: PathBuf, plugins: HashMap<String, InstalledPlugin> }
impl InstallationManager {
    pub fn new(d: impl Into<PathBuf>) -> Self {
        let dir: PathBuf = d.into();
        Self { index_path: dir.join("plugins-index.json"), plugins_dir: dir, plugins: HashMap::new() }
    }

    /// Get the plugins directory
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Get all installed plugins
    pub fn get_installed_plugins(&self) -> Vec<&InstalledPlugin> {
        self.plugins.values().collect()
    }

    /// Get an installed plugin by ID
    pub fn get_plugin(&self, id: &str) -> Option<&InstalledPlugin> {
        self.plugins.get(id)
    }

    /// Get a mutable reference to an installed plugin
    pub fn get_plugin_mut(&mut self, id: &str) -> Option<&mut InstalledPlugin> {
        self.plugins.get_mut(id)
    }

    /// Check if a plugin is installed
    pub fn is_installed(&self, id: &str) -> bool {
        self.plugins.contains_key(id)
    }

    /// Install a plugin from a manifest
    pub fn install(&mut self, manifest: PluginManifest, path: PathBuf) -> Result<()> {
        if self.plugins.contains_key(&manifest.id) {
            return Err(PluginError::AlreadyInstalled(manifest.id.clone()));
        }
        let plugin = InstalledPlugin::new(manifest.clone(), path);
        self.plugins.insert(manifest.id, plugin);
        Ok(())
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, id: &str) -> Result<InstalledPlugin> {
        self.plugins.remove(id).ok_or_else(|| PluginError::not_found(id))
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, id: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(id).ok_or_else(|| PluginError::not_found(id))?;
        plugin.enable();
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, id: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(id).ok_or_else(|| PluginError::not_found(id))?;
        plugin.disable();
        Ok(())
    }

    /// Get the count of installed plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest(id: &str) -> PluginManifest {
        PluginManifest::new(id, "Test", "1.0.0", "Author")
    }

    #[test]
    fn test_installation_manager_new() {
        let manager = InstallationManager::new("/tmp/plugins");
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_install_plugin() {
        let mut manager = InstallationManager::new("/tmp/plugins");
        let manifest = create_test_manifest("com.test.plugin");

        manager.install(manifest, PathBuf::from("/tmp/plugins/com.test.plugin")).unwrap();
        assert!(manager.is_installed("com.test.plugin"));
        assert_eq!(manager.plugin_count(), 1);
    }

    #[test]
    fn test_uninstall_plugin() {
        let mut manager = InstallationManager::new("/tmp/plugins");
        let manifest = create_test_manifest("com.test.plugin");

        manager.install(manifest, PathBuf::from("/tmp/plugins/com.test.plugin")).unwrap();
        manager.uninstall("com.test.plugin").unwrap();
        assert!(!manager.is_installed("com.test.plugin"));
    }

    #[test]
    fn test_enable_disable_plugin() {
        let mut manager = InstallationManager::new("/tmp/plugins");
        let manifest = create_test_manifest("com.test.plugin");

        manager.install(manifest, PathBuf::from("/tmp/plugins/com.test.plugin")).unwrap();

        manager.disable_plugin("com.test.plugin").unwrap();
        assert!(!manager.get_plugin("com.test.plugin").unwrap().enabled);

        manager.enable_plugin("com.test.plugin").unwrap();
        assert!(manager.get_plugin("com.test.plugin").unwrap().enabled);
    }

    #[test]
    fn test_installed_plugin() {
        let manifest = create_test_manifest("com.test.plugin");
        let mut plugin = InstalledPlugin::new(manifest, PathBuf::from("/tmp/test"));

        assert!(plugin.is_active());
        assert_eq!(plugin.id(), "com.test.plugin");
        assert_eq!(plugin.version(), "1.0.0");

        plugin.set_setting("key", serde_json::json!("value"));
        assert_eq!(plugin.get_setting("key"), Some(&serde_json::json!("value")));

        plugin.remove_setting("key");
        assert!(plugin.get_setting("key").is_none());
    }
}
