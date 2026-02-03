//! Plugin system for MS Word clone
//!
//! This crate provides a complete plugin architecture including:
//!
//! - **Manifest**: Plugin metadata, permissions, activation events, and contributions
//! - **Host**: Plugin lifecycle management (load, unload, enable, disable)
//! - **Permissions**: Fine-grained permission control for plugin capabilities
//! - **Messages**: Communication protocol between host and plugins
//! - **Sandbox**: Resource limits and API restrictions for security
//! - **Registry**: Plugin discovery, installation, and updates
//!
//! # Example
//!
//! ```rust
//! use plugins::{PluginHost, PluginManifest, Permission};
//!
//! // Create a plugin host
//! let mut host = PluginHost::new();
//!
//! // Create a manifest for a plugin
//! let manifest = PluginManifest::new(
//!     "com.example.myplugin",
//!     "My Plugin",
//!     "1.0.0",
//!     "Author Name",
//! )
//! .with_description("A sample plugin")
//! .with_permission(Permission::DocumentRead);
//!
//! // Load the plugin
//! let plugin_id = host.load_plugin_from_manifest(manifest, "/path/to/plugin").unwrap();
//!
//! // Grant permissions
//! host.grant_all_permissions(&plugin_id).unwrap();
//!
//! // Check if plugin is loaded
//! assert!(host.is_plugin_loaded(&plugin_id));
//! ```

pub mod api;
pub mod error;
pub mod host;
pub mod manifest;
pub mod messages;
pub mod permissions;
pub mod registry;
pub mod installation;
pub mod sandbox;

// Re-export main types for convenience
pub use error::{PluginError, PluginErrorCode, Result, SerializablePluginError};
pub use host::{LoadedPlugin, PluginHost, PluginId, PluginState};
pub use manifest::{
    ActivationEvent, CommandContribution, Contributions, MenuContribution, PanelContribution,
    PanelLocation, Permission, PluginManifest, ToolbarContribution,
};
pub use messages::{HostMessage, HostMessageType, PluginMessage, PluginMessageType, PluginRequest};
pub use permissions::{PermissionManager, PermissionRequest, PermissionState};
pub use registry::{DiscoveredPlugin, PluginMetadata, PluginRegistry, PluginUpdate, RegistryState};
pub use installation::{InstallationManager, InstalledPlugin, InstallationState};
pub use sandbox::{ApiRestrictions, ResourceLimitViolation, ResourceUsage, SandboxConfig};
pub use api::{
    // Error types
    ApiError, ApiResult,
    // Document types
    DocumentSnapshot, DocumentMetadata, Paragraph, Selection, Position, Range,
    SearchOptions, AppliedStyle, MockDocument,
    // Command types
    CommandHandler, Disposable, DisposableType,
    // UI types
    ToolbarItem, ToolbarItemUpdate, MessageType, InputBoxOptions, PanelOptions,
    PanelLocation as ApiPanelLocation, Panel,
    // Network types
    HttpMethod, FetchOptions, Response,
    // Value type
    Value,
    // API traits
    DocumentApi, CommandApi, UiApi, StorageApi, NetworkApi,
    // Context
    PluginApiContext,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_plugin_workflow() {
        // Create a host
        let mut host = PluginHost::new();

        // Create a manifest
        let manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        )
        .with_description("A test plugin")
        .with_permission(Permission::DocumentRead)
        .with_permission(Permission::UiToolbar)
        .with_activation_event(ActivationEvent::OnStartup);

        // Load the plugin
        let plugin_id = host
            .load_plugin_from_manifest(manifest, "/test/path")
            .expect("Failed to load plugin");

        // Verify it's loaded
        assert!(host.is_plugin_loaded(&plugin_id));
        assert_eq!(host.plugin_count(), 1);

        // Check startup plugins
        let startup_plugins = host.get_startup_plugins();
        assert_eq!(startup_plugins.len(), 1);
        assert_eq!(startup_plugins[0], plugin_id);

        // Grant permissions
        host.grant_all_permissions(&plugin_id).unwrap();

        // Verify permissions
        assert!(host
            .permissions()
            .check_permission(&plugin_id, Permission::DocumentRead));
        assert!(host
            .permissions()
            .check_permission(&plugin_id, Permission::UiToolbar));
    }

    #[test]
    fn test_permission_flow() {
        let mut manager = PermissionManager::new();
        let plugin_id = "com.example.test";

        // Request permissions
        manager.request_permissions(
            plugin_id,
            vec![
                Permission::DocumentRead,
                Permission::DocumentWrite,
                Permission::Network,
            ],
        );

        // Check pending
        let pending = manager.get_pending_permissions(plugin_id);
        assert_eq!(pending.len(), 3);

        // Grant some permissions
        manager.grant_permission(plugin_id, Permission::DocumentRead);
        manager.deny_permission(plugin_id, Permission::Network);

        // Verify state
        assert!(manager.check_permission(plugin_id, Permission::DocumentRead));
        assert!(!manager.check_permission(plugin_id, Permission::Network));
        assert!(manager.is_denied(plugin_id, Permission::Network));

        // Check remaining pending
        let pending = manager.get_pending_permissions(plugin_id);
        assert_eq!(pending.len(), 1);
        assert!(pending.contains(&Permission::DocumentWrite));
    }

    #[test]
    fn test_sandbox_config() {
        // Default config
        let default = SandboxConfig::default();
        assert!(default.max_memory_bytes > 0);

        // Restrictive config
        let restrictive = SandboxConfig::restrictive();
        assert!(restrictive.max_memory_bytes < default.max_memory_bytes);

        // Permissive config
        let permissive = SandboxConfig::permissive();
        assert!(permissive.max_memory_bytes > default.max_memory_bytes);

        // Host checking
        assert!(!default.is_host_allowed("localhost"));
        assert!(default.is_host_allowed("api.example.com"));
    }

    #[test]
    fn test_api_restrictions() {
        // Create restrictions from permissions
        let restrictions = ApiRestrictions::from_permissions(&[
            Permission::DocumentRead,
            Permission::UiDialog,
        ]);

        // Check allowed methods
        assert!(restrictions.is_method_allowed("document.get"));
        assert!(restrictions.is_method_allowed("ui.showDialog"));
        assert!(restrictions.is_method_allowed("log.info")); // Always allowed

        // Check disallowed methods
        assert!(!restrictions.is_method_allowed("document.set")); // Needs DocumentWrite
        assert!(!restrictions.is_method_allowed("network.fetch")); // Needs Network
    }

    #[test]
    fn test_message_protocol() {
        // Reset counter for predictable IDs
        messages::reset_message_id_counter();

        // Create host messages
        let request = HostMessage::request("test.method", Some(serde_json::json!({"arg": 1})));
        assert!(request.is_request());
        assert_eq!(request.method, "test.method");

        let event = HostMessage::event("document.changed", None);
        assert!(event.is_event());

        // Create plugin messages
        let response = PluginMessage::response(1, serde_json::json!({"result": "ok"}));
        assert!(response.is_success());
        assert!(!response.is_error());

        let error_response = PluginMessage::error_response(
            1,
            SerializablePluginError::new(PluginErrorCode::ExecutionError, "Failed"),
        );
        assert!(error_response.is_error());
        assert!(!error_response.is_success());
    }

    #[test]
    fn test_resource_tracking() {
        let config = SandboxConfig::new().with_max_memory(1000);
        let mut usage = ResourceUsage::new();

        // Track memory
        usage.update_memory(500);
        assert!(!usage.is_memory_exceeded(&config));

        usage.update_memory(1500);
        assert!(usage.is_memory_exceeded(&config));

        // Check violation detection
        let violation = usage.is_any_limit_exceeded(&config);
        assert!(matches!(
            violation,
            Some(ResourceLimitViolation::Memory { .. })
        ));
    }

    #[test]
    fn test_manifest_contributions() {
        let manifest = PluginManifest::new("com.example.test", "Test", "1.0.0", "Author")
            .with_contributions(
                Contributions::new()
                    .with_command(
                        CommandContribution::new("test.cmd", "Test Command")
                            .with_keybinding("Ctrl+T")
                            .with_icon("test-icon"),
                    )
                    .with_toolbar_item(ToolbarContribution::new("test.cmd", "main").with_priority(10))
                    .with_panel(
                        PanelContribution::new("test.panel", "Test Panel")
                            .with_location(PanelLocation::Left),
                    )
                    .with_menu(MenuContribution::new("test.cmd", "file").with_group("tools")),
            );

        assert_eq!(manifest.contributes.commands.len(), 1);
        assert_eq!(manifest.contributes.toolbar_items.len(), 1);
        assert_eq!(manifest.contributes.panels.len(), 1);
        assert_eq!(manifest.contributes.menus.len(), 1);

        // Verify command details
        let cmd = &manifest.contributes.commands[0];
        assert_eq!(cmd.id, "test.cmd");
        assert_eq!(cmd.keybinding, Some("Ctrl+T".to_string()));

        // Verify panel location
        let panel = &manifest.contributes.panels[0];
        assert_eq!(panel.location, PanelLocation::Left);
    }

    #[test]
    fn test_activation_events() {
        let manifest = PluginManifest::new("com.example.test", "Test", "1.0.0", "Author")
            .with_activation_event(ActivationEvent::OnStartup)
            .with_activation_event(ActivationEvent::OnCommand("myCommand".to_string()))
            .with_activation_event(ActivationEvent::OnDocumentOpen("*.docx".to_string()))
            .with_activation_event(ActivationEvent::OnLanguage("markdown".to_string()));

        assert!(manifest.activation_events.iter().any(|e| e.is_startup()));
        assert!(manifest
            .activation_events
            .iter()
            .any(|e| e.matches_command("myCommand")));
        assert!(manifest
            .activation_events
            .iter()
            .any(|e| e.matches_document("test.docx")));
        assert!(manifest
            .activation_events
            .iter()
            .any(|e| e.matches_language("markdown")));
    }

    #[test]
    fn test_error_types() {
        // Test various error creation methods
        let not_found = PluginError::not_found("test");
        assert!(matches!(not_found, PluginError::NotFound(_)));

        let perm_denied = PluginError::permission_denied("plugin", "network");
        assert!(matches!(
            perm_denied,
            PluginError::PermissionDenied { .. }
        ));

        let sandbox = PluginError::sandbox_violation("file access");
        assert!(matches!(sandbox, PluginError::SandboxViolation(_)));

        // Test error conversion to serializable
        let serializable: SerializablePluginError = (&not_found).into();
        assert_eq!(serializable.code, PluginErrorCode::NotFound);
    }
}
