//! Plugin host for managing loaded plugins
//!
//! This module provides the main PluginHost struct that manages
//! loading, unloading, and communicating with plugins.

use crate::error::{PluginError, Result};
use crate::manifest::PluginManifest;
use crate::messages::{HostMessage, PluginMessage};
use crate::permissions::PermissionManager;
use crate::sandbox::{ResourceUsage, SandboxConfig};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::mpsc;

/// Unique identifier for a plugin instance
pub type PluginId = String;

/// State of a loaded plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is loading
    Loading,
    /// Plugin is loaded and ready
    Ready,
    /// Plugin is currently executing
    Running,
    /// Plugin is suspended
    Suspended,
    /// Plugin encountered an error
    Error,
    /// Plugin is being unloaded
    Unloading,
}

/// A loaded plugin instance
#[derive(Debug)]
pub struct LoadedPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Current state
    pub state: PluginState,
    /// Path to the plugin directory
    pub path: String,
    /// Resource usage tracking
    pub resource_usage: ResourceUsage,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
    /// Whether the plugin is enabled
    pub enabled: bool,
}

impl LoadedPlugin {
    /// Create a new loaded plugin
    pub fn new(manifest: PluginManifest, path: impl Into<String>) -> Self {
        Self {
            manifest,
            state: PluginState::Loading,
            path: path.into(),
            resource_usage: ResourceUsage::new(),
            sandbox_config: SandboxConfig::default(),
            enabled: true,
        }
    }

    /// Get the plugin ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Check if the plugin is ready
    pub fn is_ready(&self) -> bool {
        self.state == PluginState::Ready
    }

    /// Check if the plugin can be executed
    pub fn can_execute(&self) -> bool {
        self.enabled && matches!(self.state, PluginState::Ready | PluginState::Running)
    }

    /// Set the plugin state
    pub fn set_state(&mut self, state: PluginState) {
        self.state = state;
    }

    /// Enable the plugin
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the plugin
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// The plugin host manages all loaded plugins
pub struct PluginHost {
    /// Loaded plugins by ID
    plugins: HashMap<String, LoadedPlugin>,
    /// Permission manager
    permissions: PermissionManager,
    /// Channel to send messages to plugins
    message_tx: mpsc::Sender<HostMessage>,
    /// Channel to receive messages from plugins
    message_rx: mpsc::Receiver<PluginMessage>,
    /// Pending requests awaiting responses
    pending_requests: HashMap<u64, PendingRequest>,
    /// Plugin load order for deterministic iteration
    load_order: Vec<String>,
}

/// A pending request waiting for a response
struct PendingRequest {
    /// Plugin ID the request was sent to
    plugin_id: String,
    /// Method that was called
    method: String,
    /// Response sender
    response_tx: tokio::sync::oneshot::Sender<Result<Value>>,
}

impl PluginHost {
    /// Create a new plugin host
    pub fn new() -> Self {
        let (message_tx, _) = mpsc::channel(100);
        let (_, message_rx) = mpsc::channel(100);

        Self {
            plugins: HashMap::new(),
            permissions: PermissionManager::new(),
            message_tx,
            message_rx,
            pending_requests: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Create a plugin host with custom channels
    pub fn with_channels(
        message_tx: mpsc::Sender<HostMessage>,
        message_rx: mpsc::Receiver<PluginMessage>,
    ) -> Self {
        Self {
            plugins: HashMap::new(),
            permissions: PermissionManager::new(),
            message_tx,
            message_rx,
            pending_requests: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Load a plugin from a directory path
    pub async fn load_plugin(&mut self, path: &Path) -> Result<PluginId> {
        // Read and parse the manifest
        let manifest_path = path.join("manifest.json");
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::Io(format!("Failed to read manifest: {}", e)))?;

        let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;

        // Validate the manifest
        manifest.validate()?;

        let plugin_id = manifest.id.clone();

        // Check if already loaded
        if self.plugins.contains_key(&plugin_id) {
            return Err(PluginError::already_loaded(&plugin_id));
        }

        // Create loaded plugin
        let mut plugin = LoadedPlugin::new(manifest, path.to_string_lossy().to_string());
        plugin.set_state(PluginState::Ready);

        // Request permissions for the plugin
        self.permissions.request_permissions(
            &plugin_id,
            plugin.manifest.permissions.clone(),
        );

        // Store the plugin
        self.plugins.insert(plugin_id.clone(), plugin);
        self.load_order.push(plugin_id.clone());

        Ok(plugin_id)
    }

    /// Load a plugin from a manifest directly (for testing)
    pub fn load_plugin_from_manifest(
        &mut self,
        manifest: PluginManifest,
        path: impl Into<String>,
    ) -> Result<PluginId> {
        manifest.validate()?;

        let plugin_id = manifest.id.clone();

        if self.plugins.contains_key(&plugin_id) {
            return Err(PluginError::already_loaded(&plugin_id));
        }

        let mut plugin = LoadedPlugin::new(manifest, path);
        plugin.set_state(PluginState::Ready);

        self.permissions.request_permissions(
            &plugin_id,
            plugin.manifest.permissions.clone(),
        );

        self.plugins.insert(plugin_id.clone(), plugin);
        self.load_order.push(plugin_id.clone());

        Ok(plugin_id)
    }

    /// Unload a plugin
    pub async fn unload_plugin(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        plugin.set_state(PluginState::Unloading);

        // Revoke all permissions
        self.permissions.revoke_all_permissions(id);

        // Remove from plugins
        self.plugins.remove(id);
        self.load_order.retain(|pid| pid != id);

        Ok(())
    }

    /// Call a method on a plugin
    pub async fn call_plugin(
        &self,
        id: &str,
        method: &str,
        args: Value,
    ) -> Result<Value> {
        let plugin = self
            .plugins
            .get(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        if !plugin.can_execute() {
            return Err(PluginError::InvalidState(format!(
                "Plugin {} is not ready for execution",
                id
            )));
        }

        // Create and send the message
        let message = HostMessage::request(method, Some(args));

        self.message_tx
            .send(message)
            .await
            .map_err(|e| PluginError::communication(format!("Failed to send message: {}", e)))?;

        // In a real implementation, we would wait for the response
        // For now, return a placeholder
        Ok(Value::Null)
    }

    /// Send an event to all plugins
    pub async fn broadcast_event(&self, event: &str, data: Option<Value>) -> Result<()> {
        let message = HostMessage::event(event, data);

        for plugin_id in &self.load_order {
            if let Some(plugin) = self.plugins.get(plugin_id) {
                if plugin.can_execute() {
                    let _ = self.message_tx.send(message.clone()).await;
                }
            }
        }

        Ok(())
    }

    /// Send an event to plugins that registered for it
    pub async fn send_event_to_interested(
        &self,
        event: &str,
        data: Option<Value>,
    ) -> Result<()> {
        let message = HostMessage::event(event, data);

        for plugin_id in &self.load_order {
            if let Some(plugin) = self.plugins.get(plugin_id) {
                if plugin.can_execute() {
                    // Check if plugin registered for this event
                    let interested = plugin.manifest.activation_events.iter().any(|ae| {
                        ae.matches_command(event)
                    });

                    if interested {
                        let _ = self.message_tx.send(message.clone()).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<&PluginManifest> {
        self.load_order
            .iter()
            .filter_map(|id| self.plugins.get(id).map(|p| &p.manifest))
            .collect()
    }

    /// Get a loaded plugin by ID
    pub fn get_plugin(&self, id: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(id)
    }

    /// Get a mutable reference to a loaded plugin
    pub fn get_plugin_mut(&mut self, id: &str) -> Option<&mut LoadedPlugin> {
        self.plugins.get_mut(id)
    }

    /// Check if a plugin is loaded
    pub fn is_plugin_loaded(&self, id: &str) -> bool {
        self.plugins.contains_key(id)
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        plugin.enable();
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        plugin.disable();
        Ok(())
    }

    /// Get the permission manager
    pub fn permissions(&self) -> &PermissionManager {
        &self.permissions
    }

    /// Get a mutable reference to the permission manager
    pub fn permissions_mut(&mut self) -> &mut PermissionManager {
        &mut self.permissions
    }

    /// Grant all requested permissions for a plugin
    pub fn grant_all_permissions(&mut self, id: &str) -> Result<()> {
        let plugin = self
            .plugins
            .get(id)
            .ok_or_else(|| PluginError::not_found(id))?;

        let permissions = plugin.manifest.permissions.clone();
        self.permissions.grant_permissions(id, &permissions);
        Ok(())
    }

    /// Get plugins that should activate on startup
    pub fn get_startup_plugins(&self) -> Vec<&str> {
        self.load_order
            .iter()
            .filter_map(|id| {
                self.plugins.get(id).and_then(|p| {
                    if p.manifest.activation_events.iter().any(|e| e.is_startup()) {
                        Some(id.as_str())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Get plugins that should activate for a command
    pub fn get_plugins_for_command(&self, command: &str) -> Vec<&str> {
        self.load_order
            .iter()
            .filter_map(|id| {
                self.plugins.get(id).and_then(|p| {
                    if p.manifest.activation_events.iter().any(|e| e.matches_command(command)) {
                        Some(id.as_str())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Get plugins that should activate for a document
    pub fn get_plugins_for_document(&self, path: &str) -> Vec<&str> {
        self.load_order
            .iter()
            .filter_map(|id| {
                self.plugins.get(id).and_then(|p| {
                    if p.manifest.activation_events.iter().any(|e| e.matches_document(path)) {
                        Some(id.as_str())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ActivationEvent, Permission};

    fn create_test_manifest(id: &str) -> PluginManifest {
        PluginManifest::new(id, "Test Plugin", "1.0.0", "Test Author")
    }

    #[test]
    fn test_plugin_host_new() {
        let host = PluginHost::new();
        assert_eq!(host.plugin_count(), 0);
    }

    #[test]
    fn test_load_plugin_from_manifest() {
        let mut host = PluginHost::new();
        let manifest = create_test_manifest("com.test.plugin");

        let result = host.load_plugin_from_manifest(manifest, "/path/to/plugin");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "com.test.plugin");
        assert_eq!(host.plugin_count(), 1);
    }

    #[test]
    fn test_load_duplicate_plugin() {
        let mut host = PluginHost::new();
        let manifest1 = create_test_manifest("com.test.plugin");
        let manifest2 = create_test_manifest("com.test.plugin");

        host.load_plugin_from_manifest(manifest1, "/path/1").unwrap();
        let result = host.load_plugin_from_manifest(manifest2, "/path/2");

        assert!(matches!(result, Err(PluginError::AlreadyLoaded(_))));
    }

    #[tokio::test]
    async fn test_unload_plugin() {
        let mut host = PluginHost::new();
        let manifest = create_test_manifest("com.test.plugin");

        host.load_plugin_from_manifest(manifest, "/path").unwrap();
        assert!(host.is_plugin_loaded("com.test.plugin"));

        host.unload_plugin("com.test.plugin").await.unwrap();
        assert!(!host.is_plugin_loaded("com.test.plugin"));
    }

    #[tokio::test]
    async fn test_unload_nonexistent_plugin() {
        let mut host = PluginHost::new();
        let result = host.unload_plugin("nonexistent").await;
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_get_loaded_plugins() {
        let mut host = PluginHost::new();

        host.load_plugin_from_manifest(
            create_test_manifest("com.test.plugin1"),
            "/path/1",
        ).unwrap();
        host.load_plugin_from_manifest(
            create_test_manifest("com.test.plugin2"),
            "/path/2",
        ).unwrap();

        let plugins = host.get_loaded_plugins();
        assert_eq!(plugins.len(), 2);
    }

    #[test]
    fn test_is_plugin_loaded() {
        let mut host = PluginHost::new();
        host.load_plugin_from_manifest(
            create_test_manifest("com.test.plugin"),
            "/path",
        ).unwrap();

        assert!(host.is_plugin_loaded("com.test.plugin"));
        assert!(!host.is_plugin_loaded("com.other.plugin"));
    }

    #[test]
    fn test_enable_disable_plugin() {
        let mut host = PluginHost::new();
        host.load_plugin_from_manifest(
            create_test_manifest("com.test.plugin"),
            "/path",
        ).unwrap();

        host.disable_plugin("com.test.plugin").unwrap();
        assert!(!host.get_plugin("com.test.plugin").unwrap().enabled);

        host.enable_plugin("com.test.plugin").unwrap();
        assert!(host.get_plugin("com.test.plugin").unwrap().enabled);
    }

    #[test]
    fn test_get_plugin() {
        let mut host = PluginHost::new();
        host.load_plugin_from_manifest(
            create_test_manifest("com.test.plugin"),
            "/path",
        ).unwrap();

        let plugin = host.get_plugin("com.test.plugin");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().manifest.id, "com.test.plugin");

        assert!(host.get_plugin("nonexistent").is_none());
    }

    #[test]
    fn test_grant_all_permissions() {
        let mut host = PluginHost::new();
        let manifest = create_test_manifest("com.test.plugin")
            .with_permission(Permission::DocumentRead)
            .with_permission(Permission::Network);

        host.load_plugin_from_manifest(manifest, "/path").unwrap();
        host.grant_all_permissions("com.test.plugin").unwrap();

        assert!(host.permissions().check_permission("com.test.plugin", Permission::DocumentRead));
        assert!(host.permissions().check_permission("com.test.plugin", Permission::Network));
    }

    #[test]
    fn test_get_startup_plugins() {
        let mut host = PluginHost::new();

        let manifest1 = create_test_manifest("com.test.startup")
            .with_activation_event(ActivationEvent::OnStartup);
        let manifest2 = create_test_manifest("com.test.ondemand")
            .with_activation_event(ActivationEvent::OnCommand("test".to_string()));

        host.load_plugin_from_manifest(manifest1, "/path/1").unwrap();
        host.load_plugin_from_manifest(manifest2, "/path/2").unwrap();

        let startup = host.get_startup_plugins();
        assert_eq!(startup.len(), 1);
        assert_eq!(startup[0], "com.test.startup");
    }

    #[test]
    fn test_get_plugins_for_command() {
        let mut host = PluginHost::new();

        let manifest = create_test_manifest("com.test.plugin")
            .with_activation_event(ActivationEvent::OnCommand("myCommand".to_string()));

        host.load_plugin_from_manifest(manifest, "/path").unwrap();

        let plugins = host.get_plugins_for_command("myCommand");
        assert_eq!(plugins.len(), 1);

        let plugins = host.get_plugins_for_command("otherCommand");
        assert_eq!(plugins.len(), 0);
    }

    #[test]
    fn test_get_plugins_for_document() {
        let mut host = PluginHost::new();

        let manifest = create_test_manifest("com.test.plugin")
            .with_activation_event(ActivationEvent::OnDocumentOpen("*.docx".to_string()));

        host.load_plugin_from_manifest(manifest, "/path").unwrap();

        let plugins = host.get_plugins_for_document("test.docx");
        assert_eq!(plugins.len(), 1);

        let plugins = host.get_plugins_for_document("test.txt");
        assert_eq!(plugins.len(), 0);
    }

    #[test]
    fn test_loaded_plugin_state() {
        let manifest = create_test_manifest("com.test.plugin");
        let mut plugin = LoadedPlugin::new(manifest, "/path");

        assert_eq!(plugin.state, PluginState::Loading);
        assert!(!plugin.is_ready());

        plugin.set_state(PluginState::Ready);
        assert!(plugin.is_ready());
        assert!(plugin.can_execute());

        plugin.disable();
        assert!(!plugin.can_execute());

        plugin.enable();
        assert!(plugin.can_execute());
    }

    #[test]
    fn test_plugin_load_order_preserved() {
        let mut host = PluginHost::new();

        host.load_plugin_from_manifest(
            create_test_manifest("com.test.first"),
            "/path/1",
        ).unwrap();
        host.load_plugin_from_manifest(
            create_test_manifest("com.test.second"),
            "/path/2",
        ).unwrap();
        host.load_plugin_from_manifest(
            create_test_manifest("com.test.third"),
            "/path/3",
        ).unwrap();

        let plugins = host.get_loaded_plugins();
        assert_eq!(plugins[0].id, "com.test.first");
        assert_eq!(plugins[1].id, "com.test.second");
        assert_eq!(plugins[2].id, "com.test.third");
    }
}
