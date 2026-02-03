//! Application state management

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use doc_model::{DocumentPaginationSettings, DocumentTree, NodeId, Section, Selection};
use layout_engine::{ViewMode, ViewModeConfig, DraftViewOptions, OutlineViewOptions};
use perf::PerfMetrics;
use revisions::RevisionState;
use store::{LockedRegionManager, SettingsManager, TemplateManager};
use text_engine::FontManager;

/// Global application state
pub struct AppState {
    /// Map of document IDs to document instances
    pub documents: Mutex<HashMap<String, DocumentState>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }
}

/// State for a single document
pub struct DocumentState {
    /// Document ID
    pub id: String,
    /// File path (if saved)
    pub path: Option<String>,
    /// Whether document has unsaved changes
    pub dirty: bool,
    /// The document tree containing all content
    pub tree: DocumentTree,
    /// Current selection state
    pub selection: Selection,
    /// Document-level pagination settings (widow/orphan control, etc.)
    pub pagination_settings: DocumentPaginationSettings,
    /// Section storage (sections are not part of the main tree yet)
    pub sections: HashMap<NodeId, Section>,
}

impl DocumentState {
    /// Create a new empty document state
    pub fn new(id: String) -> Self {
        Self {
            id,
            path: None,
            dirty: false,
            tree: DocumentTree::with_empty_paragraph(),
            selection: Selection::default(),
            pagination_settings: DocumentPaginationSettings::default(),
            sections: HashMap::new(),
        }
    }

    /// Create a document state with a file path
    pub fn with_path(id: String, path: String) -> Self {
        Self {
            id,
            path: Some(path),
            dirty: false,
            tree: DocumentTree::with_empty_paragraph(),
            selection: Selection::default(),
            pagination_settings: DocumentPaginationSettings::default(),
            sections: HashMap::new(),
        }
    }
}

/// Settings state wrapper for thread-safe access
pub struct SettingsState {
    pub manager: Mutex<SettingsManager>,
}

impl SettingsState {
    /// Create a new settings state with the given app data directory
    pub fn new(app_data_dir: PathBuf) -> Self {
        let mut manager = SettingsManager::new(app_data_dir);
        // Load settings on startup
        if let Err(e) = manager.load_sync() {
            tracing::warn!("Failed to load settings, using defaults: {}", e);
        }
        Self {
            manager: Mutex::new(manager),
        }
    }
}

/// Font manager state wrapper for thread-safe access
pub struct FontManagerState {
    pub manager: Mutex<FontManager>,
}

impl FontManagerState {
    /// Create a new font manager state
    pub fn new() -> Self {
        let manager = FontManager::new();
        // Initialize font discovery in background
        if let Err(e) = manager.initialize() {
            tracing::warn!("Failed to initialize font manager: {:?}", e);
        } else {
            tracing::info!("Font manager initialized successfully");
        }
        Self {
            manager: Mutex::new(manager),
        }
    }
}

impl Default for FontManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics state wrapper for thread-safe access
pub struct PerfMetricsState {
    pub metrics: Mutex<PerfMetrics>,
}

impl PerfMetricsState {
    /// Create a new performance metrics state
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(PerfMetrics::new()),
        }
    }

    /// Create with a custom performance budget
    pub fn with_budget(budget: perf::PerfBudget) -> Self {
        Self {
            metrics: Mutex::new(PerfMetrics::with_budget(budget)),
        }
    }
}

impl Default for PerfMetricsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Revision tracking state wrapper for thread-safe access
pub struct RevisionStateWrapper {
    pub state: Mutex<RevisionState>,
}

impl RevisionStateWrapper {
    /// Create a new revision state wrapper
    pub fn new() -> Self {
        Self {
            state: Mutex::new(RevisionState::new()),
        }
    }

    /// Create with a default author
    pub fn with_author(author: impl Into<String>) -> Self {
        Self {
            state: Mutex::new(RevisionState::with_author(author)),
        }
    }
}

impl Default for RevisionStateWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Template manager state wrapper for thread-safe access
pub struct TemplateState {
    /// Template manager for CRUD operations
    pub manager: Mutex<TemplateManager>,
    /// Locked region manager for the current document
    pub locked_regions: Mutex<LockedRegionManager>,
}

impl TemplateState {
    /// Create a new template state with the given templates directory
    pub fn new(templates_dir: PathBuf) -> Self {
        let manager = TemplateManager::new(templates_dir);
        // Ensure the templates directory exists
        if let Err(e) = manager.ensure_directory() {
            tracing::warn!("Failed to create templates directory: {}", e);
        }
        Self {
            manager: Mutex::new(manager),
            locked_regions: Mutex::new(LockedRegionManager::new()),
        }
    }
}

impl Default for TemplateState {
    fn default() -> Self {
        // Default to current directory - should be overridden in app setup
        Self::new(PathBuf::from("templates"))
    }
}

// =============================================================================
// View Mode State
// =============================================================================

/// View mode state wrapper for thread-safe access per document
pub struct ViewModeState {
    /// Map of document IDs to their view mode configurations
    pub configs: Mutex<HashMap<String, ViewModeConfig>>,
}

impl ViewModeState {
    /// Create a new view mode state
    pub fn new() -> Self {
        Self {
            configs: Mutex::new(HashMap::new()),
        }
    }

    /// Get or create view mode config for a document
    pub fn get_or_create(&self, doc_id: &str) -> ViewModeConfig {
        let mut configs = self.configs.lock().unwrap();
        configs.entry(doc_id.to_string()).or_default().clone()
    }

    /// Set view mode for a document
    pub fn set_mode(&self, doc_id: &str, mode: ViewMode) {
        let mut configs = self.configs.lock().unwrap();
        let config = configs.entry(doc_id.to_string()).or_default();
        config.mode = mode;
    }

    /// Get view mode for a document
    pub fn get_mode(&self, doc_id: &str) -> ViewMode {
        let configs = self.configs.lock().unwrap();
        configs.get(doc_id).map(|c| c.mode).unwrap_or_default()
    }

    /// Set draft view options for a document
    pub fn set_draft_options(&self, doc_id: &str, options: DraftViewOptions) {
        let mut configs = self.configs.lock().unwrap();
        let config = configs.entry(doc_id.to_string()).or_default();
        config.draft_options = options;
    }

    /// Get draft view options for a document
    pub fn get_draft_options(&self, doc_id: &str) -> DraftViewOptions {
        let configs = self.configs.lock().unwrap();
        configs.get(doc_id).map(|c| c.draft_options.clone()).unwrap_or_default()
    }

    /// Set outline view options for a document
    pub fn set_outline_options(&self, doc_id: &str, options: OutlineViewOptions) {
        let mut configs = self.configs.lock().unwrap();
        let config = configs.entry(doc_id.to_string()).or_default();
        config.outline_options = options;
    }

    /// Get outline view options for a document
    pub fn get_outline_options(&self, doc_id: &str) -> OutlineViewOptions {
        let configs = self.configs.lock().unwrap();
        configs.get(doc_id).map(|c| c.outline_options.clone()).unwrap_or_default()
    }

    /// Remove view mode config for a document (when document is closed)
    pub fn remove(&self, doc_id: &str) {
        let mut configs = self.configs.lock().unwrap();
        configs.remove(doc_id);
    }
}

impl Default for ViewModeState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Collaboration State
// =============================================================================

use collab::{
    ClientId, CollaborativeDocument, ConnectionStatus, OfflineManager, PermissionManager,
    PresenceManager, SyncEngine, VersionHistory,
};

/// Collaboration state for real-time editing
pub struct CollaborationState {
    /// Client ID for this instance
    pub client_id: Mutex<ClientId>,
    /// Collaborative documents per document ID
    pub documents: Mutex<HashMap<String, CollaborativeDocument>>,
    /// Sync engines per document
    pub sync_engines: Mutex<HashMap<String, SyncEngine>>,
    /// Presence managers per document
    pub presence: Mutex<HashMap<String, PresenceManager>>,
    /// Version history per document
    pub versions: Mutex<HashMap<String, VersionHistory>>,
    /// Offline manager
    pub offline: Mutex<OfflineManager>,
    /// Permission manager
    pub permissions: Mutex<PermissionManager>,
}

impl CollaborationState {
    pub fn new() -> Self {
        let client_id = ClientId::new(rand::random::<u64>());
        Self {
            client_id: Mutex::new(client_id),
            documents: Mutex::new(HashMap::new()),
            sync_engines: Mutex::new(HashMap::new()),
            presence: Mutex::new(HashMap::new()),
            versions: Mutex::new(HashMap::new()),
            offline: Mutex::new(OfflineManager::new(client_id)),
            permissions: Mutex::new(PermissionManager::new()),
        }
    }
}

impl Default for CollaborationState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Mail Merge State
// =============================================================================

use mail_merge::DataSource;

/// Mail merge state for managing data sources
pub struct MailMergeState {
    /// Loaded data sources by ID
    pub sources: Mutex<HashMap<String, DataSource>>,
}

impl MailMergeState {
    /// Create a new mail merge state
    pub fn new() -> Self {
        Self {
            sources: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MailMergeState {
    fn default() -> Self {
        Self::new()
    }
}
