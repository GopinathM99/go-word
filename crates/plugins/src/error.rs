//! Error types for the plugin system
//!
//! This module defines all error types that can occur during plugin operations.

use crate::manifest::ManifestValidationError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for plugin operations
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin already loaded
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    /// Plugin not loaded
    #[error("Plugin not loaded: {0}")]
    NotLoaded(String),

    /// Manifest parsing error
    #[error("Failed to parse manifest: {0}")]
    ManifestParse(String),

    /// Manifest validation error
    #[error("Invalid manifest: {0}")]
    ManifestValidation(#[from] ManifestValidationError),

    /// Permission denied
    #[error("Permission denied: {plugin_id} requires {permission}")]
    PermissionDenied {
        plugin_id: String,
        permission: String,
    },

    /// Method not found
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Plugin execution error
    #[error("Plugin execution error: {0}")]
    Execution(String),

    /// Plugin communication error
    #[error("Plugin communication error: {0}")]
    Communication(String),

    /// Plugin timeout
    #[error("Plugin operation timed out: {0}")]
    Timeout(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Sandbox violation
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Invalid state
    #[error("Invalid plugin state: {0}")]
    InvalidState(String),

    /// Registry error
    #[error("Registry error: {0}")]
    Registry(String),

    /// Plugin already installed
    #[error("Plugin already installed: {0}")]
    AlreadyInstalled(String),

    /// Installation failed
    #[error("Installation failed: {0}")]
    InstallationFailed(String),
}

impl PluginError {
    /// Create a new NotFound error
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound(id.into())
    }

    /// Create a new AlreadyLoaded error
    pub fn already_loaded(id: impl Into<String>) -> Self {
        Self::AlreadyLoaded(id.into())
    }

    /// Create a new NotLoaded error
    pub fn not_loaded(id: impl Into<String>) -> Self {
        Self::NotLoaded(id.into())
    }

    /// Create a new PermissionDenied error
    pub fn permission_denied(plugin_id: impl Into<String>, permission: impl Into<String>) -> Self {
        Self::PermissionDenied {
            plugin_id: plugin_id.into(),
            permission: permission.into(),
        }
    }

    /// Create a new Execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::Execution(msg.into())
    }

    /// Create a new Communication error
    pub fn communication(msg: impl Into<String>) -> Self {
        Self::Communication(msg.into())
    }

    /// Create a new Timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a new SandboxViolation error
    pub fn sandbox_violation(msg: impl Into<String>) -> Self {
        Self::SandboxViolation(msg.into())
    }

    /// Create a new ResourceLimitExceeded error
    pub fn resource_limit_exceeded(msg: impl Into<String>) -> Self {
        Self::ResourceLimitExceeded(msg.into())
    }
}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        Self::ManifestParse(err.to_string())
    }
}

/// Serializable plugin error for message passing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializablePluginError {
    /// Error code
    pub code: PluginErrorCode,
    /// Error message
    pub message: String,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

impl SerializablePluginError {
    /// Create a new serializable error
    pub fn new(code: PluginErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Add additional data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl From<&PluginError> for SerializablePluginError {
    fn from(err: &PluginError) -> Self {
        let code = match err {
            PluginError::NotFound(_) => PluginErrorCode::NotFound,
            PluginError::AlreadyLoaded(_) => PluginErrorCode::AlreadyLoaded,
            PluginError::NotLoaded(_) => PluginErrorCode::NotLoaded,
            PluginError::ManifestParse(_) => PluginErrorCode::ManifestError,
            PluginError::ManifestValidation(_) => PluginErrorCode::ManifestError,
            PluginError::PermissionDenied { .. } => PluginErrorCode::PermissionDenied,
            PluginError::MethodNotFound(_) => PluginErrorCode::MethodNotFound,
            PluginError::Execution(_) => PluginErrorCode::ExecutionError,
            PluginError::Communication(_) => PluginErrorCode::CommunicationError,
            PluginError::Timeout(_) => PluginErrorCode::Timeout,
            PluginError::Io(_) => PluginErrorCode::IoError,
            PluginError::SandboxViolation(_) => PluginErrorCode::SandboxViolation,
            PluginError::ResourceLimitExceeded(_) => PluginErrorCode::ResourceLimitExceeded,
            PluginError::InvalidState(_) => PluginErrorCode::InvalidState,
            PluginError::Registry(_) => PluginErrorCode::RegistryError,
            PluginError::AlreadyInstalled(_) => PluginErrorCode::AlreadyLoaded,
            PluginError::InstallationFailed(_) => PluginErrorCode::IoError,
        };
        Self::new(code, err.to_string())
    }
}

/// Error codes for serializable errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginErrorCode {
    NotFound,
    AlreadyLoaded,
    NotLoaded,
    ManifestError,
    PermissionDenied,
    MethodNotFound,
    ExecutionError,
    CommunicationError,
    Timeout,
    IoError,
    SandboxViolation,
    ResourceLimitExceeded,
    InvalidState,
    RegistryError,
    Unknown,
}

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_not_found() {
        let err = PluginError::not_found("com.example.test");
        assert!(matches!(err, PluginError::NotFound(ref id) if id == "com.example.test"));
        assert_eq!(err.to_string(), "Plugin not found: com.example.test");
    }

    #[test]
    fn test_error_already_loaded() {
        let err = PluginError::already_loaded("com.example.test");
        assert!(matches!(err, PluginError::AlreadyLoaded(ref id) if id == "com.example.test"));
    }

    #[test]
    fn test_error_not_loaded() {
        let err = PluginError::not_loaded("com.example.test");
        assert!(matches!(err, PluginError::NotLoaded(ref id) if id == "com.example.test"));
    }

    #[test]
    fn test_error_permission_denied() {
        let err = PluginError::permission_denied("com.example.test", "network");
        match err {
            PluginError::PermissionDenied { plugin_id, permission } => {
                assert_eq!(plugin_id, "com.example.test");
                assert_eq!(permission, "network");
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn test_error_execution() {
        let err = PluginError::execution("Script error");
        assert!(matches!(err, PluginError::Execution(ref msg) if msg == "Script error"));
    }

    #[test]
    fn test_error_sandbox_violation() {
        let err = PluginError::sandbox_violation("Attempted file access");
        assert!(matches!(err, PluginError::SandboxViolation(ref msg) if msg == "Attempted file access"));
    }

    #[test]
    fn test_serializable_error() {
        let err = SerializablePluginError::new(PluginErrorCode::NotFound, "Plugin not found");
        assert_eq!(err.code, PluginErrorCode::NotFound);
        assert_eq!(err.message, "Plugin not found");
        assert!(err.data.is_none());
    }

    #[test]
    fn test_serializable_error_with_data() {
        let err = SerializablePluginError::new(PluginErrorCode::ExecutionError, "Error")
            .with_data(serde_json::json!({"line": 42}));
        assert!(err.data.is_some());
    }

    #[test]
    fn test_error_to_serializable() {
        let err = PluginError::not_found("test");
        let serializable: SerializablePluginError = (&err).into();
        assert_eq!(serializable.code, PluginErrorCode::NotFound);
    }

    #[test]
    fn test_serializable_error_serialization() {
        let err = SerializablePluginError::new(PluginErrorCode::Timeout, "Timed out");
        let json = serde_json::to_string(&err).unwrap();
        let parsed: SerializablePluginError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, parsed);
    }
}
