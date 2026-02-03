//! Plugin manifest definitions
//!
//! This module defines the structure of plugin manifests, which describe
//! plugin metadata, permissions, activation events, and contributions.

use serde::{Deserialize, Serialize};

/// Plugin manifest containing all metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifest {
    /// Unique identifier (e.g., "com.example.myplugin")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Semantic version string
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Author name or organization
    pub author: String,
    /// Entry point file (e.g., "main.js")
    pub entry: String,
    /// Required permissions
    pub permissions: Vec<Permission>,
    /// Events that trigger plugin activation
    pub activation_events: Vec<ActivationEvent>,
    /// UI and functionality contributions
    pub contributes: Contributions,
}

impl PluginManifest {
    /// Create a new plugin manifest with required fields
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            author: author.into(),
            entry: "main.js".to_string(),
            permissions: Vec::new(),
            activation_events: Vec::new(),
            contributes: Contributions::default(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the entry point
    pub fn with_entry(mut self, entry: impl Into<String>) -> Self {
        self.entry = entry.into();
        self
    }

    /// Add a permission
    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Add an activation event
    pub fn with_activation_event(mut self, event: ActivationEvent) -> Self {
        self.activation_events.push(event);
        self
    }

    /// Set contributions
    pub fn with_contributions(mut self, contributions: Contributions) -> Self {
        self.contributes = contributions;
        self
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.id.is_empty() {
            return Err(ManifestValidationError::EmptyId);
        }
        if !self.id.contains('.') {
            return Err(ManifestValidationError::InvalidIdFormat(self.id.clone()));
        }
        if self.name.is_empty() {
            return Err(ManifestValidationError::EmptyName);
        }
        if self.version.is_empty() {
            return Err(ManifestValidationError::EmptyVersion);
        }
        if self.entry.is_empty() {
            return Err(ManifestValidationError::EmptyEntry);
        }
        Ok(())
    }
}

/// Errors that can occur during manifest validation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ManifestValidationError {
    #[error("Plugin ID cannot be empty")]
    EmptyId,
    #[error("Plugin ID must be in reverse domain format: {0}")]
    InvalidIdFormat(String),
    #[error("Plugin name cannot be empty")]
    EmptyName,
    #[error("Plugin version cannot be empty")]
    EmptyVersion,
    #[error("Plugin entry point cannot be empty")]
    EmptyEntry,
}

/// Permissions that plugins can request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read document content
    DocumentRead,
    /// Modify document content
    DocumentWrite,
    /// Add items to the toolbar
    UiToolbar,
    /// Create side panels
    UiPanel,
    /// Show dialogs
    UiDialog,
    /// Make network requests
    Network,
    /// Store persistent data
    Storage,
    /// Access clipboard
    Clipboard,
}

impl Permission {
    /// Get a human-readable description of the permission
    pub fn description(&self) -> &'static str {
        match self {
            Permission::DocumentRead => "Read document content",
            Permission::DocumentWrite => "Modify document content",
            Permission::UiToolbar => "Add toolbar items",
            Permission::UiPanel => "Create side panels",
            Permission::UiDialog => "Show dialogs",
            Permission::Network => "Make network requests",
            Permission::Storage => "Store persistent data",
            Permission::Clipboard => "Access clipboard",
        }
    }

    /// Check if this is a potentially dangerous permission
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Permission::DocumentWrite | Permission::Network | Permission::Clipboard
        )
    }
}

/// Events that can trigger plugin activation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ActivationEvent {
    /// Activate when a specific command is invoked
    OnCommand(String),
    /// Activate when a document matching the glob pattern is opened
    OnDocumentOpen(String),
    /// Activate on application startup
    OnStartup,
    /// Activate for documents with a specific language
    OnLanguage(String),
}

impl ActivationEvent {
    /// Check if this event matches a command
    pub fn matches_command(&self, command: &str) -> bool {
        matches!(self, ActivationEvent::OnCommand(cmd) if cmd == command)
    }

    /// Check if this event is a startup event
    pub fn is_startup(&self) -> bool {
        matches!(self, ActivationEvent::OnStartup)
    }

    /// Check if this event matches a document path
    pub fn matches_document(&self, path: &str) -> bool {
        match self {
            ActivationEvent::OnDocumentOpen(pattern) => {
                // Simple glob matching (supports * wildcard)
                glob_match(pattern, path)
            }
            _ => false,
        }
    }

    /// Check if this event matches a language
    pub fn matches_language(&self, language: &str) -> bool {
        matches!(self, ActivationEvent::OnLanguage(lang) if lang == language)
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with("*.") {
        let ext = &pattern[1..]; // includes the dot
        return path.ends_with(ext);
    }
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 1];
        return path.starts_with(prefix);
    }
    pattern == path
}

/// Plugin contributions to the UI and functionality
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Contributions {
    /// Commands the plugin provides
    pub commands: Vec<CommandContribution>,
    /// Toolbar items
    pub toolbar_items: Vec<ToolbarContribution>,
    /// Side panels
    pub panels: Vec<PanelContribution>,
    /// Menu entries
    pub menus: Vec<MenuContribution>,
}

impl Contributions {
    /// Create empty contributions
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a command contribution
    pub fn with_command(mut self, command: CommandContribution) -> Self {
        self.commands.push(command);
        self
    }

    /// Add a toolbar item
    pub fn with_toolbar_item(mut self, item: ToolbarContribution) -> Self {
        self.toolbar_items.push(item);
        self
    }

    /// Add a panel
    pub fn with_panel(mut self, panel: PanelContribution) -> Self {
        self.panels.push(panel);
        self
    }

    /// Add a menu item
    pub fn with_menu(mut self, menu: MenuContribution) -> Self {
        self.menus.push(menu);
        self
    }
}

/// A command contribution from a plugin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandContribution {
    /// Command identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Optional keyboard shortcut
    pub keybinding: Option<String>,
    /// Optional icon identifier
    pub icon: Option<String>,
}

impl CommandContribution {
    /// Create a new command contribution
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            keybinding: None,
            icon: None,
        }
    }

    /// Set the keybinding
    pub fn with_keybinding(mut self, keybinding: impl Into<String>) -> Self {
        self.keybinding = Some(keybinding.into());
        self
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A toolbar contribution from a plugin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolbarContribution {
    /// Associated command ID
    pub command: String,
    /// Toolbar group to place in
    pub group: String,
    /// Position within the group
    pub priority: i32,
}

impl ToolbarContribution {
    /// Create a new toolbar contribution
    pub fn new(command: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            group: group.into(),
            priority: 0,
        }
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// A panel contribution from a plugin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelContribution {
    /// Panel identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Icon identifier
    pub icon: Option<String>,
    /// Default location (left, right, bottom)
    pub location: PanelLocation,
}

impl PanelContribution {
    /// Create a new panel contribution
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon: None,
            location: PanelLocation::Right,
        }
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the location
    pub fn with_location(mut self, location: PanelLocation) -> Self {
        self.location = location;
        self
    }
}

/// Panel location options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelLocation {
    Left,
    #[default]
    Right,
    Bottom,
}

/// A menu contribution from a plugin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuContribution {
    /// Associated command ID
    pub command: String,
    /// Parent menu path (e.g., "file", "edit/advanced")
    pub menu: String,
    /// Position within the menu group
    pub group: Option<String>,
}

impl MenuContribution {
    /// Create a new menu contribution
    pub fn new(command: impl Into<String>, menu: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            menu: menu.into(),
            group: None,
        }
    }

    /// Set the group
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );

        assert_eq!(manifest.id, "com.example.test");
        assert_eq!(manifest.name, "Test Plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.entry, "main.js");
    }

    #[test]
    fn test_manifest_builder_pattern() {
        let manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        )
        .with_description("A test plugin")
        .with_entry("index.js")
        .with_permission(Permission::DocumentRead)
        .with_activation_event(ActivationEvent::OnStartup);

        assert_eq!(manifest.description, "A test plugin");
        assert_eq!(manifest.entry, "index.js");
        assert_eq!(manifest.permissions, vec![Permission::DocumentRead]);
        assert_eq!(manifest.activation_events, vec![ActivationEvent::OnStartup]);
    }

    #[test]
    fn test_manifest_validation_valid() {
        let manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_validation_empty_id() {
        let mut manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );
        manifest.id = String::new();
        assert_eq!(manifest.validate(), Err(ManifestValidationError::EmptyId));
    }

    #[test]
    fn test_manifest_validation_invalid_id_format() {
        let mut manifest = PluginManifest::new(
            "invalid-id",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );
        assert!(matches!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidIdFormat(_))
        ));
    }

    #[test]
    fn test_manifest_validation_empty_name() {
        let mut manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );
        manifest.name = String::new();
        assert_eq!(manifest.validate(), Err(ManifestValidationError::EmptyName));
    }

    #[test]
    fn test_manifest_validation_empty_version() {
        let mut manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );
        manifest.version = String::new();
        assert_eq!(manifest.validate(), Err(ManifestValidationError::EmptyVersion));
    }

    #[test]
    fn test_manifest_validation_empty_entry() {
        let mut manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        );
        manifest.entry = String::new();
        assert_eq!(manifest.validate(), Err(ManifestValidationError::EmptyEntry));
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = PluginManifest::new(
            "com.example.test",
            "Test Plugin",
            "1.0.0",
            "Test Author",
        )
        .with_permission(Permission::DocumentRead);

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn test_permission_description() {
        assert_eq!(Permission::DocumentRead.description(), "Read document content");
        assert_eq!(Permission::Network.description(), "Make network requests");
    }

    #[test]
    fn test_permission_sensitivity() {
        assert!(!Permission::DocumentRead.is_sensitive());
        assert!(Permission::DocumentWrite.is_sensitive());
        assert!(Permission::Network.is_sensitive());
        assert!(Permission::Clipboard.is_sensitive());
    }

    #[test]
    fn test_activation_event_matches_command() {
        let event = ActivationEvent::OnCommand("test.command".to_string());
        assert!(event.matches_command("test.command"));
        assert!(!event.matches_command("other.command"));

        let startup = ActivationEvent::OnStartup;
        assert!(!startup.matches_command("test.command"));
    }

    #[test]
    fn test_activation_event_is_startup() {
        assert!(ActivationEvent::OnStartup.is_startup());
        assert!(!ActivationEvent::OnCommand("test".to_string()).is_startup());
    }

    #[test]
    fn test_activation_event_matches_document() {
        let event = ActivationEvent::OnDocumentOpen("*.docx".to_string());
        assert!(event.matches_document("test.docx"));
        assert!(!event.matches_document("test.txt"));

        let any = ActivationEvent::OnDocumentOpen("*".to_string());
        assert!(any.matches_document("anything.xyz"));
    }

    #[test]
    fn test_activation_event_matches_language() {
        let event = ActivationEvent::OnLanguage("markdown".to_string());
        assert!(event.matches_language("markdown"));
        assert!(!event.matches_language("html"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything.txt"));
        assert!(glob_match("*.txt", "file.txt"));
        assert!(!glob_match("*.txt", "file.doc"));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("file.txt", "file.txt"));
        assert!(!glob_match("file.txt", "other.txt"));
    }

    #[test]
    fn test_glob_match_directory() {
        assert!(glob_match("docs/*", "docs/file.txt"));
        assert!(!glob_match("docs/*", "other/file.txt"));
    }

    #[test]
    fn test_contributions_builder() {
        let contributions = Contributions::new()
            .with_command(CommandContribution::new("test.cmd", "Test Command"))
            .with_toolbar_item(ToolbarContribution::new("test.cmd", "main"))
            .with_panel(PanelContribution::new("test.panel", "Test Panel"))
            .with_menu(MenuContribution::new("test.cmd", "file"));

        assert_eq!(contributions.commands.len(), 1);
        assert_eq!(contributions.toolbar_items.len(), 1);
        assert_eq!(contributions.panels.len(), 1);
        assert_eq!(contributions.menus.len(), 1);
    }

    #[test]
    fn test_command_contribution() {
        let cmd = CommandContribution::new("test.cmd", "Test Command")
            .with_keybinding("Ctrl+Shift+T")
            .with_icon("test-icon");

        assert_eq!(cmd.id, "test.cmd");
        assert_eq!(cmd.title, "Test Command");
        assert_eq!(cmd.keybinding, Some("Ctrl+Shift+T".to_string()));
        assert_eq!(cmd.icon, Some("test-icon".to_string()));
    }

    #[test]
    fn test_toolbar_contribution() {
        let item = ToolbarContribution::new("test.cmd", "main")
            .with_priority(10);

        assert_eq!(item.command, "test.cmd");
        assert_eq!(item.group, "main");
        assert_eq!(item.priority, 10);
    }

    #[test]
    fn test_panel_contribution() {
        let panel = PanelContribution::new("test.panel", "Test Panel")
            .with_icon("panel-icon")
            .with_location(PanelLocation::Left);

        assert_eq!(panel.id, "test.panel");
        assert_eq!(panel.title, "Test Panel");
        assert_eq!(panel.icon, Some("panel-icon".to_string()));
        assert_eq!(panel.location, PanelLocation::Left);
    }

    #[test]
    fn test_menu_contribution() {
        let menu = MenuContribution::new("test.cmd", "file")
            .with_group("export");

        assert_eq!(menu.command, "test.cmd");
        assert_eq!(menu.menu, "file");
        assert_eq!(menu.group, Some("export".to_string()));
    }

    #[test]
    fn test_panel_location_default() {
        assert_eq!(PanelLocation::default(), PanelLocation::Right);
    }
}
