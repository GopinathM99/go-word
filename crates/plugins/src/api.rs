//! Plugin API layer for the MS Word clone
//!
//! This module provides the API traits and implementations that plugins use
//! to interact with the host application. All API calls are subject to
//! permission checks via the PermissionManager.
//!
//! # API Categories
//!
//! - **DocumentApi**: Read and modify document content
//! - **CommandApi**: Register and execute commands
//! - **UiApi**: Create UI elements (toolbar items, panels, dialogs)
//! - **StorageApi**: Persist plugin data
//! - **NetworkApi**: Make HTTP requests
//!
//! # Example
//!
//! ```rust,ignore
//! use plugins::api::{PluginApiContext, DocumentApi};
//!
//! let mut ctx = PluginApiContext::new(plugin_id);
//! let snapshot = ctx.get_content()?;
//! ```

use crate::manifest::Permission;
use crate::permissions::PermissionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during API calls
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum ApiError {
    /// Permission denied for the requested operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// The requested resource was not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid argument provided
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// The position is out of bounds
    #[error("Position out of bounds: {0}")]
    OutOfBounds(String),

    /// The range is invalid
    #[error("Invalid range: {0}")]
    InvalidRange(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

// ============================================================================
// Document Types
// ============================================================================

/// A snapshot of the document content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    /// The document's unique identifier
    pub id: String,
    /// The document's title
    pub title: String,
    /// The document's plain text content
    pub text: String,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Paragraph information
    pub paragraphs: Vec<Paragraph>,
}

impl DocumentSnapshot {
    /// Create a new document snapshot
    pub fn new(id: impl Into<String>, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
            metadata: DocumentMetadata::default(),
            paragraphs: Vec::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: DocumentMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add paragraphs
    pub fn with_paragraphs(mut self, paragraphs: Vec<Paragraph>) -> Self {
        self.paragraphs = paragraphs;
        self
    }

    /// Get the document length in characters
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Document metadata
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Author name
    pub author: Option<String>,
    /// Creation timestamp (Unix epoch milliseconds)
    pub created_at: Option<u64>,
    /// Last modified timestamp (Unix epoch milliseconds)
    pub modified_at: Option<u64>,
    /// Word count
    pub word_count: usize,
    /// Character count
    pub char_count: usize,
    /// Custom metadata fields
    pub custom: HashMap<String, Value>,
}

/// A paragraph in the document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    /// Starting position in the document
    pub start: usize,
    /// Ending position in the document
    pub end: usize,
    /// The paragraph's style
    pub style: Option<String>,
}

/// Current selection in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start position of the selection
    pub start: Position,
    /// End position of the selection
    pub end: Position,
    /// Selected text (if any)
    pub text: String,
    /// Whether the selection is collapsed (cursor only)
    pub is_collapsed: bool,
}

impl Selection {
    /// Create a new selection
    pub fn new(start: Position, end: Position, text: impl Into<String>) -> Self {
        let text = text.into();
        let is_collapsed = start == end;
        Self {
            start,
            end,
            text,
            is_collapsed,
        }
    }

    /// Create a collapsed selection (cursor)
    pub fn cursor(position: Position) -> Self {
        Self {
            start: position.clone(),
            end: position,
            text: String::new(),
            is_collapsed: true,
        }
    }

    /// Convert selection to a range
    pub fn to_range(&self) -> Range {
        Range {
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

/// A position in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column/character offset within the line (0-indexed)
    pub column: usize,
    /// Absolute character offset from document start
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create a position from just an offset
    pub fn from_offset(offset: usize) -> Self {
        Self {
            line: 0,
            column: offset,
            offset,
        }
    }

    /// Check if this position is before another
    pub fn is_before(&self, other: &Position) -> bool {
        self.offset < other.offset
    }

    /// Check if this position is after another
    pub fn is_after(&self, other: &Position) -> bool {
        self.offset > other.offset
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// A range in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
}

impl Range {
    /// Create a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a range from offsets
    pub fn from_offsets(start: usize, end: usize) -> Self {
        Self {
            start: Position::from_offset(start),
            end: Position::from_offset(end),
        }
    }

    /// Get the length of the range
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if the range is empty
    pub fn is_empty(&self) -> bool {
        self.start.offset >= self.end.offset
    }

    /// Check if this range contains a position
    pub fn contains(&self, position: &Position) -> bool {
        position.offset >= self.start.offset && position.offset <= self.end.offset
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &Range) -> bool {
        self.start.offset < other.end.offset && other.start.offset < self.end.offset
    }

    /// Validate that the range is valid (start <= end)
    pub fn is_valid(&self) -> bool {
        self.start.offset <= self.end.offset
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}

// ============================================================================
// Command Types
// ============================================================================

/// A command handler function type
pub type CommandHandler = Arc<dyn Fn(&[Value]) -> ApiResult<Value> + Send + Sync>;

/// A handle that can be used to dispose of a registration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Disposable {
    /// Unique identifier for this disposable
    pub id: String,
    /// Type of resource this disposes
    pub resource_type: DisposableType,
}

impl Disposable {
    /// Create a new disposable
    pub fn new(id: impl Into<String>, resource_type: DisposableType) -> Self {
        Self {
            id: id.into(),
            resource_type,
        }
    }

    /// Create a disposable for a command
    pub fn command(id: impl Into<String>) -> Self {
        Self::new(id, DisposableType::Command)
    }

    /// Create a disposable for a toolbar item
    pub fn toolbar_item(id: impl Into<String>) -> Self {
        Self::new(id, DisposableType::ToolbarItem)
    }

    /// Create a disposable for a panel
    pub fn panel(id: impl Into<String>) -> Self {
        Self::new(id, DisposableType::Panel)
    }

    /// Create a disposable for an event listener
    pub fn event_listener(id: impl Into<String>) -> Self {
        Self::new(id, DisposableType::EventListener)
    }
}

/// Types of disposable resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisposableType {
    /// A registered command
    Command,
    /// A toolbar item
    ToolbarItem,
    /// A panel
    Panel,
    /// An event listener
    EventListener,
    /// A menu item
    MenuItem,
}

// ============================================================================
// UI Types
// ============================================================================

/// A toolbar item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolbarItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Tooltip text
    pub tooltip: Option<String>,
    /// Icon identifier or URL
    pub icon: Option<String>,
    /// Command to execute when clicked
    pub command: Option<String>,
    /// Toolbar group to place the item in
    pub group: String,
    /// Priority within the group (higher = more left)
    pub priority: i32,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Whether the item is visible
    pub visible: bool,
}

impl ToolbarItem {
    /// Create a new toolbar item
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tooltip: None,
            icon: None,
            command: None,
            group: "default".to_string(),
            priority: 0,
            enabled: true,
            visible: true,
        }
    }

    /// Set the tooltip
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the command
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the group
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = group.into();
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Message types for dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// Success message
    Success,
    /// Question/confirmation message
    Question,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Info
    }
}

/// Options for an input box dialog
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBoxOptions {
    /// Title for the input box
    pub title: String,
    /// Prompt message
    pub prompt: String,
    /// Default value
    pub default_value: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Whether to mask input (for passwords)
    pub password: bool,
    /// Validation pattern (regex)
    pub validation_pattern: Option<String>,
    /// Validation error message
    pub validation_message: Option<String>,
}

impl InputBoxOptions {
    /// Create new input box options
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            default_value: None,
            placeholder: None,
            password: false,
            validation_pattern: None,
            validation_message: None,
        }
    }

    /// Set the default value
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set the placeholder
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set password mode
    pub fn with_password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Set validation
    pub fn with_validation(
        mut self,
        pattern: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        self.validation_pattern = Some(pattern.into());
        self.validation_message = Some(message.into());
        self
    }
}

/// Options for creating a panel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelOptions {
    /// Panel identifier
    pub id: String,
    /// Panel title
    pub title: String,
    /// Panel location
    pub location: PanelLocation,
    /// Panel icon
    pub icon: Option<String>,
    /// Initial width (for left/right panels)
    pub width: Option<u32>,
    /// Initial height (for bottom panel)
    pub height: Option<u32>,
    /// Whether the panel can be closed by the user
    pub closable: bool,
    /// Whether the panel is initially visible
    pub visible: bool,
}

impl PanelOptions {
    /// Create new panel options
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            location: PanelLocation::Right,
            icon: None,
            width: None,
            height: None,
            closable: true,
            visible: true,
        }
    }

    /// Set the location
    pub fn with_location(mut self, location: PanelLocation) -> Self {
        self.location = location;
        self
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the width
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height
    pub fn with_height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set closable state
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set initial visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Panel location options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelLocation {
    /// Left sidebar
    Left,
    /// Right sidebar
    #[default]
    Right,
    /// Bottom panel
    Bottom,
}

/// A created panel instance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    /// Panel identifier
    pub id: String,
    /// Panel title
    pub title: String,
    /// Current location
    pub location: PanelLocation,
    /// Whether the panel is visible
    pub visible: bool,
    /// Disposable for cleanup
    pub disposable: Disposable,
}

impl Panel {
    /// Create a new panel
    pub fn new(id: impl Into<String>, title: impl Into<String>, location: PanelLocation) -> Self {
        let id = id.into();
        Self {
            disposable: Disposable::panel(&id),
            id,
            title: title.into(),
            location,
            visible: true,
        }
    }
}

// ============================================================================
// Network Types
// ============================================================================

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}

/// Options for a fetch request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchOptions {
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (for POST, PUT, PATCH)
    pub body: Option<String>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Follow redirects
    pub follow_redirects: bool,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(30000), // 30 seconds default
            follow_redirects: true,
        }
    }
}

impl FetchOptions {
    /// Create new fetch options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HTTP method
    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set JSON body
    pub fn with_json_body(mut self, value: &Value) -> Self {
        self.body = Some(serde_json::to_string(value).unwrap_or_default());
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set redirect behavior
    pub fn with_follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }
}

/// HTTP response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// HTTP status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as text
    pub body: String,
    /// Whether the request was successful (2xx status)
    pub ok: bool,
}

impl Response {
    /// Create a new response
    pub fn new(status: u16, status_text: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            ok: (200..300).contains(&status),
            status,
            status_text: status_text.into(),
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Parse the body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.body)
    }

    /// Get a header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

// ============================================================================
// Value Type (JSON-compatible)
// ============================================================================

/// A JSON-compatible value type for API data exchange
pub type Value = serde_json::Value;

// ============================================================================
// API Traits
// ============================================================================

/// Document manipulation API
pub trait DocumentApi {
    /// Get a snapshot of the current document content
    fn get_content(&self) -> ApiResult<DocumentSnapshot>;

    /// Get the current selection
    fn get_selection(&self) -> ApiResult<Selection>;

    /// Insert text at the given position
    fn insert_text(&mut self, position: Position, text: &str) -> ApiResult<()>;

    /// Apply a style to a range of text
    fn apply_style(&mut self, range: Range, style_id: &str) -> ApiResult<()>;

    /// Insert an image at the given position
    fn insert_image(&mut self, position: Position, image_data: &[u8]) -> ApiResult<()>;

    /// Delete text in the given range
    fn delete_text(&mut self, range: Range) -> ApiResult<()>;

    /// Replace text in the given range
    fn replace_text(&mut self, range: Range, text: &str) -> ApiResult<()>;

    /// Get text in the given range
    fn get_text(&self, range: Range) -> ApiResult<String>;

    /// Set the selection
    fn set_selection(&mut self, selection: Selection) -> ApiResult<()>;

    /// Search for text in the document
    fn search(&self, query: &str, options: SearchOptions) -> ApiResult<Vec<Range>>;
}

/// Search options for document search
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Case-sensitive search
    pub case_sensitive: bool,
    /// Whole word matching
    pub whole_word: bool,
    /// Regular expression search
    pub regex: bool,
    /// Maximum number of results
    pub max_results: Option<usize>,
}

impl SearchOptions {
    /// Create new search options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case sensitivity
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Set whole word matching
    pub fn with_whole_word(mut self, whole_word: bool) -> Self {
        self.whole_word = whole_word;
        self
    }

    /// Set regex mode
    pub fn with_regex(mut self, regex: bool) -> Self {
        self.regex = regex;
        self
    }

    /// Set max results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }
}

/// Command registration and execution API
pub trait CommandApi {
    /// Register a command handler
    fn register_command(
        &mut self,
        id: &str,
        handler: CommandHandler,
    ) -> ApiResult<Disposable>;

    /// Execute a registered command
    fn execute_command(&self, id: &str, args: &[Value]) -> ApiResult<Value>;

    /// Check if a command is registered
    fn has_command(&self, id: &str) -> bool;

    /// Get all registered command IDs for this plugin
    fn get_commands(&self) -> Vec<String>;

    /// Unregister a command
    fn unregister_command(&mut self, id: &str) -> ApiResult<()>;
}

/// User interface API
pub trait UiApi {
    /// Add an item to the toolbar
    fn add_toolbar_item(&mut self, item: ToolbarItem) -> ApiResult<Disposable>;

    /// Show a message to the user
    fn show_message(&self, message: &str, msg_type: MessageType) -> ApiResult<()>;

    /// Show an input box and get user input
    fn show_input_box(&self, options: InputBoxOptions) -> ApiResult<Option<String>>;

    /// Create a panel
    fn create_panel(&mut self, options: PanelOptions) -> ApiResult<Panel>;

    /// Update a toolbar item
    fn update_toolbar_item(&mut self, id: &str, updates: ToolbarItemUpdate) -> ApiResult<()>;

    /// Remove a toolbar item
    fn remove_toolbar_item(&mut self, id: &str) -> ApiResult<()>;

    /// Show a confirmation dialog
    fn show_confirm(&self, message: &str, title: &str) -> ApiResult<bool>;

    /// Show a panel
    fn show_panel(&mut self, id: &str) -> ApiResult<()>;

    /// Hide a panel
    fn hide_panel(&mut self, id: &str) -> ApiResult<()>;

    /// Close/destroy a panel
    fn close_panel(&mut self, id: &str) -> ApiResult<()>;
}

/// Updates that can be applied to a toolbar item
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolbarItemUpdate {
    /// New label
    pub label: Option<String>,
    /// New tooltip
    pub tooltip: Option<String>,
    /// New icon
    pub icon: Option<String>,
    /// New enabled state
    pub enabled: Option<bool>,
    /// New visibility
    pub visible: Option<bool>,
}

impl ToolbarItemUpdate {
    /// Create a new update
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the tooltip
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Set the visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }
}

/// Persistent storage API
pub trait StorageApi {
    /// Get a value from storage
    fn get(&self, key: &str) -> ApiResult<Option<Value>>;

    /// Set a value in storage
    fn set(&mut self, key: &str, value: Value) -> ApiResult<()>;

    /// Delete a value from storage
    fn delete(&mut self, key: &str) -> ApiResult<()>;

    /// Check if a key exists
    fn has(&self, key: &str) -> ApiResult<bool>;

    /// Get all keys
    fn keys(&self) -> ApiResult<Vec<String>>;

    /// Clear all storage
    fn clear(&mut self) -> ApiResult<()>;

    /// Get the current storage usage in bytes
    fn usage(&self) -> ApiResult<u64>;
}

/// Network API
pub trait NetworkApi {
    /// Make an HTTP fetch request
    fn fetch(&self, url: &str, options: FetchOptions) -> ApiResult<Response>;
}

// ============================================================================
// Plugin API Context Implementation
// ============================================================================

/// The main API context provided to plugins
///
/// This struct holds references to the various subsystems and enforces
/// permission checks for all API calls.
pub struct PluginApiContext {
    /// The plugin's unique identifier
    plugin_id: String,
    /// Permission manager for checking permissions
    permissions: Arc<RwLock<PermissionManager>>,
    /// Document state (mock for now)
    document: Arc<RwLock<MockDocument>>,
    /// Registered commands
    commands: Arc<RwLock<HashMap<String, CommandHandler>>>,
    /// Toolbar items
    toolbar_items: Arc<RwLock<HashMap<String, ToolbarItem>>>,
    /// Panels
    panels: Arc<RwLock<HashMap<String, Panel>>>,
    /// Storage
    storage: Arc<RwLock<HashMap<String, Value>>>,
    /// Storage usage in bytes
    storage_usage: Arc<RwLock<u64>>,
    /// Maximum storage allowed (bytes)
    max_storage: u64,
}

/// Mock document for testing and development
#[derive(Debug, Clone)]
pub struct MockDocument {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Document content
    pub content: String,
    /// Current selection
    pub selection: Selection,
    /// Applied styles
    pub styles: Vec<AppliedStyle>,
}

/// A style applied to a range
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedStyle {
    /// The range the style applies to
    pub range: Range,
    /// The style ID
    pub style_id: String,
}

impl Default for MockDocument {
    fn default() -> Self {
        Self {
            id: "doc-1".to_string(),
            title: "Untitled Document".to_string(),
            content: String::new(),
            selection: Selection::cursor(Position::default()),
            styles: Vec::new(),
        }
    }
}

impl MockDocument {
    /// Create a new mock document
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            selection: Selection::cursor(Position::default()),
            styles: Vec::new(),
        }
    }
}

impl PluginApiContext {
    /// Create a new plugin API context
    pub fn new(plugin_id: impl Into<String>, permissions: Arc<RwLock<PermissionManager>>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            permissions,
            document: Arc::new(RwLock::new(MockDocument::default())),
            commands: Arc::new(RwLock::new(HashMap::new())),
            toolbar_items: Arc::new(RwLock::new(HashMap::new())),
            panels: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(HashMap::new())),
            storage_usage: Arc::new(RwLock::new(0)),
            max_storage: 10 * 1024 * 1024, // 10 MB default
        }
    }

    /// Create a context for testing with a mock permission manager
    pub fn new_for_testing(plugin_id: impl Into<String>) -> Self {
        let mut permissions = PermissionManager::new();
        let plugin_id = plugin_id.into();

        // Grant all permissions for testing
        permissions.grant_permissions(
            &plugin_id,
            &[
                Permission::DocumentRead,
                Permission::DocumentWrite,
                Permission::UiToolbar,
                Permission::UiPanel,
                Permission::UiDialog,
                Permission::Network,
                Permission::Storage,
                Permission::Clipboard,
            ],
        );

        Self {
            plugin_id,
            permissions: Arc::new(RwLock::new(permissions)),
            document: Arc::new(RwLock::new(MockDocument::default())),
            commands: Arc::new(RwLock::new(HashMap::new())),
            toolbar_items: Arc::new(RwLock::new(HashMap::new())),
            panels: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(HashMap::new())),
            storage_usage: Arc::new(RwLock::new(0)),
            max_storage: 10 * 1024 * 1024,
        }
    }

    /// Set the mock document
    pub fn set_document(&mut self, document: MockDocument) {
        *self.document.write().unwrap() = document;
    }

    /// Get the plugin ID
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Check if the plugin has a permission
    fn check_permission(&self, permission: Permission) -> ApiResult<()> {
        let permissions = self.permissions.read().map_err(|e| {
            ApiError::Internal(format!("Failed to acquire permission lock: {}", e))
        })?;

        if permissions.check_permission(&self.plugin_id, permission) {
            Ok(())
        } else {
            Err(ApiError::PermissionDenied(format!(
                "Plugin '{}' does not have permission: {:?}",
                self.plugin_id, permission
            )))
        }
    }

    /// Validate a position is within document bounds
    fn validate_position(&self, position: &Position) -> ApiResult<()> {
        let doc = self.document.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read document: {}", e))
        })?;

        if position.offset > doc.content.len() {
            return Err(ApiError::OutOfBounds(format!(
                "Position {} is beyond document length {}",
                position.offset,
                doc.content.len()
            )));
        }
        Ok(())
    }

    /// Validate a range is within document bounds
    fn validate_range(&self, range: &Range) -> ApiResult<()> {
        if !range.is_valid() {
            return Err(ApiError::InvalidRange(
                "Range start must be before or equal to end".to_string(),
            ));
        }

        let doc = self.document.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read document: {}", e))
        })?;

        if range.end.offset > doc.content.len() {
            return Err(ApiError::OutOfBounds(format!(
                "Range end {} is beyond document length {}",
                range.end.offset,
                doc.content.len()
            )));
        }
        Ok(())
    }
}

impl DocumentApi for PluginApiContext {
    fn get_content(&self) -> ApiResult<DocumentSnapshot> {
        self.check_permission(Permission::DocumentRead)?;

        let doc = self.document.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read document: {}", e))
        })?;

        Ok(DocumentSnapshot::new(&doc.id, &doc.title, &doc.content))
    }

    fn get_selection(&self) -> ApiResult<Selection> {
        self.check_permission(Permission::DocumentRead)?;

        let doc = self.document.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read document: {}", e))
        })?;

        Ok(doc.selection.clone())
    }

    fn insert_text(&mut self, position: Position, text: &str) -> ApiResult<()> {
        self.check_permission(Permission::DocumentWrite)?;
        self.validate_position(&position)?;

        let mut doc = self.document.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write document: {}", e))
        })?;

        doc.content.insert_str(position.offset, text);
        Ok(())
    }

    fn apply_style(&mut self, range: Range, style_id: &str) -> ApiResult<()> {
        self.check_permission(Permission::DocumentWrite)?;
        self.validate_range(&range)?;

        let mut doc = self.document.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write document: {}", e))
        })?;

        doc.styles.push(AppliedStyle {
            range,
            style_id: style_id.to_string(),
        });
        Ok(())
    }

    fn insert_image(&mut self, position: Position, image_data: &[u8]) -> ApiResult<()> {
        self.check_permission(Permission::DocumentWrite)?;
        self.validate_position(&position)?;

        if image_data.is_empty() {
            return Err(ApiError::InvalidArgument("Image data cannot be empty".to_string()));
        }

        // In a real implementation, this would insert an image placeholder
        // For now, we just acquire the lock to verify we can write
        let _doc = self.document.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write document: {}", e))
        })?;

        // Image insertion would happen here in a real implementation

        Ok(())
    }

    fn delete_text(&mut self, range: Range) -> ApiResult<()> {
        self.check_permission(Permission::DocumentWrite)?;
        self.validate_range(&range)?;

        let mut doc = self.document.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write document: {}", e))
        })?;

        doc.content.replace_range(range.start.offset..range.end.offset, "");
        Ok(())
    }

    fn replace_text(&mut self, range: Range, text: &str) -> ApiResult<()> {
        self.check_permission(Permission::DocumentWrite)?;
        self.validate_range(&range)?;

        let mut doc = self.document.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write document: {}", e))
        })?;

        doc.content.replace_range(range.start.offset..range.end.offset, text);
        Ok(())
    }

    fn get_text(&self, range: Range) -> ApiResult<String> {
        self.check_permission(Permission::DocumentRead)?;
        self.validate_range(&range)?;

        let doc = self.document.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read document: {}", e))
        })?;

        Ok(doc.content[range.start.offset..range.end.offset].to_string())
    }

    fn set_selection(&mut self, selection: Selection) -> ApiResult<()> {
        self.check_permission(Permission::DocumentWrite)?;
        self.validate_position(&selection.start)?;
        self.validate_position(&selection.end)?;

        let mut doc = self.document.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write document: {}", e))
        })?;

        doc.selection = selection;
        Ok(())
    }

    fn search(&self, query: &str, options: SearchOptions) -> ApiResult<Vec<Range>> {
        self.check_permission(Permission::DocumentRead)?;

        if query.is_empty() {
            return Err(ApiError::InvalidArgument("Search query cannot be empty".to_string()));
        }

        let doc = self.document.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read document: {}", e))
        })?;

        let mut results = Vec::new();
        let content = if options.case_sensitive {
            doc.content.clone()
        } else {
            doc.content.to_lowercase()
        };
        let search_query = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut start_pos = 0;
        while let Some(pos) = content[start_pos..].find(&search_query) {
            let absolute_pos = start_pos + pos;
            results.push(Range::from_offsets(absolute_pos, absolute_pos + query.len()));
            start_pos = absolute_pos + 1;

            if let Some(max) = options.max_results {
                if results.len() >= max {
                    break;
                }
            }
        }

        Ok(results)
    }
}

impl CommandApi for PluginApiContext {
    fn register_command(
        &mut self,
        id: &str,
        handler: CommandHandler,
    ) -> ApiResult<Disposable> {
        // Commands don't require special permissions
        let full_id = format!("{}.{}", self.plugin_id, id);

        let mut commands = self.commands.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write commands: {}", e))
        })?;

        if commands.contains_key(&full_id) {
            return Err(ApiError::InvalidArgument(format!(
                "Command '{}' is already registered",
                full_id
            )));
        }

        commands.insert(full_id.clone(), handler);
        Ok(Disposable::command(full_id))
    }

    fn execute_command(&self, id: &str, args: &[Value]) -> ApiResult<Value> {
        let commands = self.commands.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read commands: {}", e))
        })?;

        // Try both the full ID and plugin-prefixed ID
        let handler = commands
            .get(id)
            .or_else(|| commands.get(&format!("{}.{}", self.plugin_id, id)))
            .ok_or_else(|| ApiError::NotFound(format!("Command '{}' not found", id)))?;

        handler(args)
    }

    fn has_command(&self, id: &str) -> bool {
        let commands = match self.commands.read() {
            Ok(c) => c,
            Err(_) => return false,
        };

        commands.contains_key(id)
            || commands.contains_key(&format!("{}.{}", self.plugin_id, id))
    }

    fn get_commands(&self) -> Vec<String> {
        let commands = match self.commands.read() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let prefix = format!("{}.", self.plugin_id);
        commands
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect()
    }

    fn unregister_command(&mut self, id: &str) -> ApiResult<()> {
        let mut commands = self.commands.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write commands: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, id);
        if commands.remove(&full_id).is_none() && commands.remove(id).is_none() {
            return Err(ApiError::NotFound(format!("Command '{}' not found", id)));
        }

        Ok(())
    }
}

impl UiApi for PluginApiContext {
    fn add_toolbar_item(&mut self, item: ToolbarItem) -> ApiResult<Disposable> {
        self.check_permission(Permission::UiToolbar)?;

        let mut items = self.toolbar_items.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write toolbar items: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, item.id);
        if items.contains_key(&full_id) {
            return Err(ApiError::InvalidArgument(format!(
                "Toolbar item '{}' already exists",
                full_id
            )));
        }

        let mut item = item;
        item.id = full_id.clone();
        items.insert(full_id.clone(), item);

        Ok(Disposable::toolbar_item(full_id))
    }

    fn show_message(&self, message: &str, _msg_type: MessageType) -> ApiResult<()> {
        self.check_permission(Permission::UiDialog)?;

        // In a real implementation, this would show a message dialog
        // For now, we just validate the operation
        if message.is_empty() {
            return Err(ApiError::InvalidArgument("Message cannot be empty".to_string()));
        }

        Ok(())
    }

    fn show_input_box(&self, options: InputBoxOptions) -> ApiResult<Option<String>> {
        self.check_permission(Permission::UiDialog)?;

        // In a real implementation, this would show an input dialog
        // For testing, we return the default value if set
        Ok(options.default_value)
    }

    fn create_panel(&mut self, options: PanelOptions) -> ApiResult<Panel> {
        self.check_permission(Permission::UiPanel)?;

        let mut panels = self.panels.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write panels: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, options.id);
        if panels.contains_key(&full_id) {
            return Err(ApiError::InvalidArgument(format!(
                "Panel '{}' already exists",
                full_id
            )));
        }

        let panel = Panel::new(&full_id, &options.title, options.location);
        panels.insert(full_id, panel.clone());

        Ok(panel)
    }

    fn update_toolbar_item(&mut self, id: &str, updates: ToolbarItemUpdate) -> ApiResult<()> {
        self.check_permission(Permission::UiToolbar)?;

        let mut items = self.toolbar_items.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write toolbar items: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, id);
        let key = if items.contains_key(&full_id) {
            full_id
        } else if items.contains_key(id) {
            id.to_string()
        } else {
            return Err(ApiError::NotFound(format!("Toolbar item '{}' not found", id)));
        };

        let item = items.get_mut(&key).unwrap();

        if let Some(label) = updates.label {
            item.label = label;
        }
        if let Some(tooltip) = updates.tooltip {
            item.tooltip = Some(tooltip);
        }
        if let Some(icon) = updates.icon {
            item.icon = Some(icon);
        }
        if let Some(enabled) = updates.enabled {
            item.enabled = enabled;
        }
        if let Some(visible) = updates.visible {
            item.visible = visible;
        }

        Ok(())
    }

    fn remove_toolbar_item(&mut self, id: &str) -> ApiResult<()> {
        self.check_permission(Permission::UiToolbar)?;

        let mut items = self.toolbar_items.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write toolbar items: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, id);
        if items.remove(&full_id).is_none() && items.remove(id).is_none() {
            return Err(ApiError::NotFound(format!("Toolbar item '{}' not found", id)));
        }

        Ok(())
    }

    fn show_confirm(&self, message: &str, _title: &str) -> ApiResult<bool> {
        self.check_permission(Permission::UiDialog)?;

        if message.is_empty() {
            return Err(ApiError::InvalidArgument("Message cannot be empty".to_string()));
        }

        // In a real implementation, this would show a confirmation dialog
        // For testing, we return true
        Ok(true)
    }

    fn show_panel(&mut self, id: &str) -> ApiResult<()> {
        self.check_permission(Permission::UiPanel)?;

        let mut panels = self.panels.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write panels: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, id);
        let key = if panels.contains_key(&full_id) {
            full_id
        } else if panels.contains_key(id) {
            id.to_string()
        } else {
            return Err(ApiError::NotFound(format!("Panel '{}' not found", id)));
        };

        let panel = panels.get_mut(&key).unwrap();
        panel.visible = true;
        Ok(())
    }

    fn hide_panel(&mut self, id: &str) -> ApiResult<()> {
        self.check_permission(Permission::UiPanel)?;

        let mut panels = self.panels.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write panels: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, id);
        let key = if panels.contains_key(&full_id) {
            full_id
        } else if panels.contains_key(id) {
            id.to_string()
        } else {
            return Err(ApiError::NotFound(format!("Panel '{}' not found", id)));
        };

        let panel = panels.get_mut(&key).unwrap();
        panel.visible = false;
        Ok(())
    }

    fn close_panel(&mut self, id: &str) -> ApiResult<()> {
        self.check_permission(Permission::UiPanel)?;

        let mut panels = self.panels.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write panels: {}", e))
        })?;

        let full_id = format!("{}.{}", self.plugin_id, id);
        if panels.remove(&full_id).is_none() && panels.remove(id).is_none() {
            return Err(ApiError::NotFound(format!("Panel '{}' not found", id)));
        }

        Ok(())
    }
}

impl StorageApi for PluginApiContext {
    fn get(&self, key: &str) -> ApiResult<Option<Value>> {
        self.check_permission(Permission::Storage)?;

        let storage = self.storage.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read storage: {}", e))
        })?;

        let full_key = format!("{}.{}", self.plugin_id, key);
        Ok(storage.get(&full_key).cloned())
    }

    fn set(&mut self, key: &str, value: Value) -> ApiResult<()> {
        self.check_permission(Permission::Storage)?;

        let value_size = serde_json::to_string(&value)
            .map_err(|e| ApiError::StorageError(format!("Failed to serialize value: {}", e)))?
            .len() as u64;

        let mut usage = self.storage_usage.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write storage usage: {}", e))
        })?;

        // Check storage limit
        if *usage + value_size > self.max_storage {
            return Err(ApiError::ResourceLimitExceeded(format!(
                "Storage limit exceeded: {} + {} > {} bytes",
                *usage, value_size, self.max_storage
            )));
        }

        let mut storage = self.storage.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write storage: {}", e))
        })?;

        let full_key = format!("{}.{}", self.plugin_id, key);

        // Subtract old value size if replacing
        if let Some(old_value) = storage.get(&full_key) {
            let old_size = serde_json::to_string(old_value)
                .map(|s| s.len() as u64)
                .unwrap_or(0);
            *usage = usage.saturating_sub(old_size);
        }

        storage.insert(full_key, value);
        *usage += value_size;

        Ok(())
    }

    fn delete(&mut self, key: &str) -> ApiResult<()> {
        self.check_permission(Permission::Storage)?;

        let mut storage = self.storage.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write storage: {}", e))
        })?;

        let full_key = format!("{}.{}", self.plugin_id, key);

        if let Some(old_value) = storage.remove(&full_key) {
            let mut usage = self.storage_usage.write().map_err(|e| {
                ApiError::Internal(format!("Failed to write storage usage: {}", e))
            })?;

            let old_size = serde_json::to_string(&old_value)
                .map(|s| s.len() as u64)
                .unwrap_or(0);
            *usage = usage.saturating_sub(old_size);
        }

        Ok(())
    }

    fn has(&self, key: &str) -> ApiResult<bool> {
        self.check_permission(Permission::Storage)?;

        let storage = self.storage.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read storage: {}", e))
        })?;

        let full_key = format!("{}.{}", self.plugin_id, key);
        Ok(storage.contains_key(&full_key))
    }

    fn keys(&self) -> ApiResult<Vec<String>> {
        self.check_permission(Permission::Storage)?;

        let storage = self.storage.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read storage: {}", e))
        })?;

        let prefix = format!("{}.", self.plugin_id);
        let keys: Vec<String> = storage
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .map(|k| k[prefix.len()..].to_string())
            .collect();

        Ok(keys)
    }

    fn clear(&mut self) -> ApiResult<()> {
        self.check_permission(Permission::Storage)?;

        let mut storage = self.storage.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write storage: {}", e))
        })?;

        let prefix = format!("{}.", self.plugin_id);
        storage.retain(|k, _| !k.starts_with(&prefix));

        let mut usage = self.storage_usage.write().map_err(|e| {
            ApiError::Internal(format!("Failed to write storage usage: {}", e))
        })?;
        *usage = 0;

        Ok(())
    }

    fn usage(&self) -> ApiResult<u64> {
        self.check_permission(Permission::Storage)?;

        let usage = self.storage_usage.read().map_err(|e| {
            ApiError::Internal(format!("Failed to read storage usage: {}", e))
        })?;

        Ok(*usage)
    }
}

impl NetworkApi for PluginApiContext {
    fn fetch(&self, url: &str, _options: FetchOptions) -> ApiResult<Response> {
        self.check_permission(Permission::Network)?;

        // Validate URL
        if url.is_empty() {
            return Err(ApiError::InvalidArgument("URL cannot be empty".to_string()));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ApiError::InvalidArgument(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        // In a real implementation, this would make an actual HTTP request
        // For testing, we return a mock response
        Ok(Response::new(200, "OK", "{}"))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> PluginApiContext {
        PluginApiContext::new_for_testing("com.test.plugin")
    }

    fn create_restricted_context() -> PluginApiContext {
        let permissions = PermissionManager::new();
        PluginApiContext::new("com.test.restricted", Arc::new(RwLock::new(permissions)))
    }

    // ========== Document API Tests ==========

    #[test]
    fn test_get_content() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test Doc", "Hello, World!"));

        let snapshot = ctx.get_content().unwrap();
        assert_eq!(snapshot.id, "doc-1");
        assert_eq!(snapshot.title, "Test Doc");
        assert_eq!(snapshot.text, "Hello, World!");
    }

    #[test]
    fn test_get_content_permission_denied() {
        let ctx = create_restricted_context();
        let result = ctx.get_content();
        assert!(matches!(result, Err(ApiError::PermissionDenied(_))));
    }

    #[test]
    fn test_insert_text() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello"));

        ctx.insert_text(Position::from_offset(5), " World!").unwrap();

        let snapshot = ctx.get_content().unwrap();
        assert_eq!(snapshot.text, "Hello World!");
    }

    #[test]
    fn test_insert_text_out_of_bounds() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello"));

        let result = ctx.insert_text(Position::from_offset(100), "text");
        assert!(matches!(result, Err(ApiError::OutOfBounds(_))));
    }

    #[test]
    fn test_delete_text() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello World!"));

        ctx.delete_text(Range::from_offsets(5, 12)).unwrap();

        let snapshot = ctx.get_content().unwrap();
        assert_eq!(snapshot.text, "Hello");
    }

    #[test]
    fn test_replace_text() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello World!"));

        ctx.replace_text(Range::from_offsets(6, 11), "Rust").unwrap();

        let snapshot = ctx.get_content().unwrap();
        assert_eq!(snapshot.text, "Hello Rust!");
    }

    #[test]
    fn test_get_text() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello World!"));

        let text = ctx.get_text(Range::from_offsets(0, 5)).unwrap();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_apply_style() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello World!"));

        ctx.apply_style(Range::from_offsets(0, 5), "bold").unwrap();

        let doc = ctx.document.read().unwrap();
        assert_eq!(doc.styles.len(), 1);
        assert_eq!(doc.styles[0].style_id, "bold");
    }

    #[test]
    fn test_search() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello World Hello"));

        let results = ctx.search("Hello", SearchOptions::new()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].start.offset, 0);
        assert_eq!(results[1].start.offset, 12);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "Hello HELLO hello"));

        let results = ctx.search("hello", SearchOptions::new().with_case_sensitive(false)).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_with_max_results() {
        let mut ctx = create_test_context();
        ctx.set_document(MockDocument::new("doc-1", "Test", "a a a a a"));

        let results = ctx.search("a", SearchOptions::new().with_max_results(2)).unwrap();
        assert_eq!(results.len(), 2);
    }

    // ========== Command API Tests ==========

    #[test]
    fn test_register_command() {
        let mut ctx = create_test_context();

        let handler: CommandHandler = Arc::new(|_| Ok(Value::String("executed".to_string())));
        let disposable = ctx.register_command("myCommand", handler).unwrap();

        assert!(disposable.id.ends_with("myCommand"));
        assert!(ctx.has_command("com.test.plugin.myCommand"));
    }

    #[test]
    fn test_execute_command() {
        let mut ctx = create_test_context();

        let handler: CommandHandler = Arc::new(|args| {
            if args.is_empty() {
                Ok(Value::String("no args".to_string()))
            } else {
                Ok(args[0].clone())
            }
        });
        ctx.register_command("echo", handler).unwrap();

        let result = ctx.execute_command("echo", &[Value::String("hello".to_string())]).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_unregister_command() {
        let mut ctx = create_test_context();

        let handler: CommandHandler = Arc::new(|_| Ok(Value::Null));
        ctx.register_command("temp", handler).unwrap();

        assert!(ctx.has_command("com.test.plugin.temp"));
        ctx.unregister_command("temp").unwrap();
        assert!(!ctx.has_command("com.test.plugin.temp"));
    }

    #[test]
    fn test_get_commands() {
        let mut ctx = create_test_context();

        let handler: CommandHandler = Arc::new(|_| Ok(Value::Null));
        ctx.register_command("cmd1", handler.clone()).unwrap();
        ctx.register_command("cmd2", handler).unwrap();

        let commands = ctx.get_commands();
        assert_eq!(commands.len(), 2);
    }

    // ========== UI API Tests ==========

    #[test]
    fn test_add_toolbar_item() {
        let mut ctx = create_test_context();

        let item = ToolbarItem::new("myButton", "Click Me")
            .with_tooltip("A test button")
            .with_icon("icon.png");

        let disposable = ctx.add_toolbar_item(item).unwrap();
        assert!(disposable.id.contains("myButton"));
    }

    #[test]
    fn test_add_toolbar_item_permission_denied() {
        let mut ctx = create_restricted_context();

        let item = ToolbarItem::new("myButton", "Click Me");
        let result = ctx.add_toolbar_item(item);
        assert!(matches!(result, Err(ApiError::PermissionDenied(_))));
    }

    #[test]
    fn test_update_toolbar_item() {
        let mut ctx = create_test_context();

        let item = ToolbarItem::new("myButton", "Click Me");
        ctx.add_toolbar_item(item).unwrap();

        ctx.update_toolbar_item(
            "myButton",
            ToolbarItemUpdate::new()
                .with_label("Updated Label")
                .with_enabled(false),
        ).unwrap();

        let items = ctx.toolbar_items.read().unwrap();
        let item = items.get("com.test.plugin.myButton").unwrap();
        assert_eq!(item.label, "Updated Label");
        assert!(!item.enabled);
    }

    #[test]
    fn test_create_panel() {
        let mut ctx = create_test_context();

        let options = PanelOptions::new("myPanel", "My Panel")
            .with_location(PanelLocation::Left);

        let panel = ctx.create_panel(options).unwrap();
        assert!(panel.id.contains("myPanel"));
        assert_eq!(panel.location, PanelLocation::Left);
    }

    #[test]
    fn test_show_hide_panel() {
        let mut ctx = create_test_context();

        let options = PanelOptions::new("testPanel", "Test");
        ctx.create_panel(options).unwrap();

        ctx.hide_panel("testPanel").unwrap();
        {
            let panels = ctx.panels.read().unwrap();
            let panel = panels.get("com.test.plugin.testPanel").unwrap();
            assert!(!panel.visible);
        }

        ctx.show_panel("testPanel").unwrap();
        {
            let panels = ctx.panels.read().unwrap();
            let panel = panels.get("com.test.plugin.testPanel").unwrap();
            assert!(panel.visible);
        }
    }

    #[test]
    fn test_show_message() {
        let ctx = create_test_context();
        let result = ctx.show_message("Hello!", MessageType::Info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_input_box() {
        let ctx = create_test_context();
        let options = InputBoxOptions::new("Title", "Enter value:")
            .with_default_value("default");

        let result = ctx.show_input_box(options).unwrap();
        assert_eq!(result, Some("default".to_string()));
    }

    // ========== Storage API Tests ==========

    #[test]
    fn test_storage_set_get() {
        let mut ctx = create_test_context();

        ctx.set("key1", Value::String("value1".to_string())).unwrap();

        let value = ctx.get("key1").unwrap();
        assert_eq!(value, Some(Value::String("value1".to_string())));
    }

    #[test]
    fn test_storage_delete() {
        let mut ctx = create_test_context();

        ctx.set("key1", Value::String("value1".to_string())).unwrap();
        ctx.delete("key1").unwrap();

        let value = ctx.get("key1").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_storage_has() {
        let mut ctx = create_test_context();

        assert!(!ctx.has("key1").unwrap());
        ctx.set("key1", Value::Null).unwrap();
        assert!(ctx.has("key1").unwrap());
    }

    #[test]
    fn test_storage_keys() {
        let mut ctx = create_test_context();

        ctx.set("key1", Value::Null).unwrap();
        ctx.set("key2", Value::Null).unwrap();
        ctx.set("key3", Value::Null).unwrap();

        let keys = ctx.keys().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[test]
    fn test_storage_clear() {
        let mut ctx = create_test_context();

        ctx.set("key1", Value::Null).unwrap();
        ctx.set("key2", Value::Null).unwrap();

        ctx.clear().unwrap();

        let keys = ctx.keys().unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_storage_usage() {
        let mut ctx = create_test_context();

        ctx.set("key1", Value::String("test".to_string())).unwrap();

        let usage = ctx.usage().unwrap();
        assert!(usage > 0);
    }

    #[test]
    fn test_storage_permission_denied() {
        let ctx = create_restricted_context();
        let result = ctx.get("key");
        assert!(matches!(result, Err(ApiError::PermissionDenied(_))));
    }

    // ========== Network API Tests ==========

    #[test]
    fn test_fetch_basic() {
        let ctx = create_test_context();

        let response = ctx.fetch("https://example.com", FetchOptions::new()).unwrap();
        assert!(response.ok);
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_fetch_invalid_url() {
        let ctx = create_test_context();

        let result = ctx.fetch("not-a-url", FetchOptions::new());
        assert!(matches!(result, Err(ApiError::InvalidArgument(_))));
    }

    #[test]
    fn test_fetch_empty_url() {
        let ctx = create_test_context();

        let result = ctx.fetch("", FetchOptions::new());
        assert!(matches!(result, Err(ApiError::InvalidArgument(_))));
    }

    #[test]
    fn test_fetch_permission_denied() {
        let ctx = create_restricted_context();

        let result = ctx.fetch("https://example.com", FetchOptions::new());
        assert!(matches!(result, Err(ApiError::PermissionDenied(_))));
    }

    // ========== Type Tests ==========

    #[test]
    fn test_position() {
        let p1 = Position::new(0, 0, 0);
        let p2 = Position::new(1, 5, 10);

        assert!(p1.is_before(&p2));
        assert!(p2.is_after(&p1));
        assert!(!p1.is_after(&p2));
    }

    #[test]
    fn test_range() {
        let range = Range::from_offsets(5, 10);

        assert_eq!(range.len(), 5);
        assert!(!range.is_empty());
        assert!(range.is_valid());
        assert!(range.contains(&Position::from_offset(7)));
        assert!(!range.contains(&Position::from_offset(15)));
    }

    #[test]
    fn test_range_overlap() {
        let r1 = Range::from_offsets(0, 10);
        let r2 = Range::from_offsets(5, 15);
        let r3 = Range::from_offsets(20, 30);

        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn test_selection() {
        let selection = Selection::new(
            Position::from_offset(5),
            Position::from_offset(10),
            "hello",
        );

        assert!(!selection.is_collapsed);
        assert_eq!(selection.text, "hello");

        let cursor = Selection::cursor(Position::from_offset(5));
        assert!(cursor.is_collapsed);
    }

    #[test]
    fn test_document_snapshot() {
        let snapshot = DocumentSnapshot::new("doc-1", "Test", "Hello World!");

        assert_eq!(snapshot.len(), 12);
        assert!(!snapshot.is_empty());
    }

    #[test]
    fn test_toolbar_item_builder() {
        let item = ToolbarItem::new("btn", "Button")
            .with_tooltip("Click me")
            .with_icon("icon.png")
            .with_command("myCommand")
            .with_group("main")
            .with_priority(10)
            .with_enabled(false)
            .with_visible(true);

        assert_eq!(item.id, "btn");
        assert_eq!(item.label, "Button");
        assert_eq!(item.tooltip, Some("Click me".to_string()));
        assert_eq!(item.icon, Some("icon.png".to_string()));
        assert_eq!(item.command, Some("myCommand".to_string()));
        assert_eq!(item.group, "main");
        assert_eq!(item.priority, 10);
        assert!(!item.enabled);
        assert!(item.visible);
    }

    #[test]
    fn test_fetch_options_builder() {
        let options = FetchOptions::new()
            .with_method(HttpMethod::Post)
            .with_header("Accept", "application/json")
            .with_body("test body")
            .with_timeout(5000)
            .with_follow_redirects(false);

        assert_eq!(options.method, HttpMethod::Post);
        assert_eq!(options.headers.get("Accept"), Some(&"application/json".to_string()));
        assert_eq!(options.body, Some("test body".to_string()));
        assert_eq!(options.timeout_ms, Some(5000));
        assert!(!options.follow_redirects);
    }

    #[test]
    fn test_response_json() {
        let response = Response::new(200, "OK", r#"{"key": "value"}"#);

        let json: serde_json::Value = response.json().unwrap();
        assert_eq!(json["key"], "value");
    }

    #[test]
    fn test_disposable() {
        let d1 = Disposable::command("cmd1");
        assert_eq!(d1.resource_type, DisposableType::Command);

        let d2 = Disposable::toolbar_item("item1");
        assert_eq!(d2.resource_type, DisposableType::ToolbarItem);

        let d3 = Disposable::panel("panel1");
        assert_eq!(d3.resource_type, DisposableType::Panel);
    }

    #[test]
    fn test_input_box_options_builder() {
        let options = InputBoxOptions::new("Title", "Prompt")
            .with_default_value("default")
            .with_placeholder("placeholder")
            .with_password(true)
            .with_validation(r"\d+", "Must be a number");

        assert_eq!(options.title, "Title");
        assert_eq!(options.prompt, "Prompt");
        assert_eq!(options.default_value, Some("default".to_string()));
        assert_eq!(options.placeholder, Some("placeholder".to_string()));
        assert!(options.password);
        assert_eq!(options.validation_pattern, Some(r"\d+".to_string()));
        assert_eq!(options.validation_message, Some("Must be a number".to_string()));
    }

    #[test]
    fn test_panel_options_builder() {
        let options = PanelOptions::new("panel1", "Panel 1")
            .with_location(PanelLocation::Bottom)
            .with_icon("icon.png")
            .with_width(300)
            .with_height(200)
            .with_closable(false)
            .with_visible(false);

        assert_eq!(options.id, "panel1");
        assert_eq!(options.title, "Panel 1");
        assert_eq!(options.location, PanelLocation::Bottom);
        assert_eq!(options.icon, Some("icon.png".to_string()));
        assert_eq!(options.width, Some(300));
        assert_eq!(options.height, Some(200));
        assert!(!options.closable);
        assert!(!options.visible);
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::PermissionDenied("network".to_string());
        assert!(err.to_string().contains("Permission denied"));

        let err = ApiError::NotFound("resource".to_string());
        assert!(err.to_string().contains("Not found"));
    }
}
