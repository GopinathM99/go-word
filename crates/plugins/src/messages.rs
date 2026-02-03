//! Message types for plugin communication
//!
//! This module defines the message protocol used for communication
//! between the host application and plugins.

use crate::error::SerializablePluginError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global message ID counter
static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique message ID
pub fn next_message_id() -> u64 {
    MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Reset message ID counter (for testing)
#[cfg(test)]
pub fn reset_message_id_counter() {
    MESSAGE_ID_COUNTER.store(1, Ordering::SeqCst);
}

/// Message sent from the host to a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostMessage {
    /// Unique message identifier
    pub id: u64,
    /// Type of message
    pub message_type: HostMessageType,
    /// Method to call (for requests) or event name
    pub method: String,
    /// Parameters for the method or event
    pub params: Option<Value>,
}

impl HostMessage {
    /// Create a new request message
    pub fn request(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            id: next_message_id(),
            message_type: HostMessageType::Request,
            method: method.into(),
            params,
        }
    }

    /// Create a new event message
    pub fn event(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            id: next_message_id(),
            message_type: HostMessageType::Event,
            method: method.into(),
            params,
        }
    }

    /// Create a request with specific ID (for testing or correlation)
    pub fn request_with_id(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            id,
            message_type: HostMessageType::Request,
            method: method.into(),
            params,
        }
    }

    /// Check if this is a request
    pub fn is_request(&self) -> bool {
        matches!(self.message_type, HostMessageType::Request)
    }

    /// Check if this is an event
    pub fn is_event(&self) -> bool {
        matches!(self.message_type, HostMessageType::Event)
    }
}

/// Type of message from host to plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostMessageType {
    /// Request expecting a response
    Request,
    /// One-way event notification
    Event,
}

/// Message sent from a plugin to the host
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginMessage {
    /// Message identifier (correlates with request ID for responses)
    pub id: u64,
    /// Type of message
    pub message_type: PluginMessageType,
    /// Result value (for successful responses)
    pub result: Option<Value>,
    /// Error (for failed responses)
    pub error: Option<SerializablePluginError>,
}

impl PluginMessage {
    /// Create a successful response
    pub fn response(id: u64, result: Value) -> Self {
        Self {
            id,
            message_type: PluginMessageType::Response,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error_response(id: u64, error: SerializablePluginError) -> Self {
        Self {
            id,
            message_type: PluginMessageType::Response,
            result: None,
            error: Some(error),
        }
    }

    /// Create a request from plugin to host
    pub fn request(method: impl Into<String>, params: Option<Value>) -> PluginRequest {
        PluginRequest {
            id: next_message_id(),
            method: method.into(),
            params,
        }
    }

    /// Check if this is a successful response
    pub fn is_success(&self) -> bool {
        self.result.is_some() && self.error.is_none()
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the result if successful
    pub fn get_result(&self) -> Option<&Value> {
        if self.error.is_none() {
            self.result.as_ref()
        } else {
            None
        }
    }

    /// Get the error if failed
    pub fn get_error(&self) -> Option<&SerializablePluginError> {
        self.error.as_ref()
    }
}

/// Type of message from plugin to host
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginMessageType {
    /// Response to a host request
    Response,
    /// Request from plugin to host
    Request,
}

/// A request from a plugin to the host
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginRequest {
    /// Unique request identifier
    pub id: u64,
    /// Method to call on the host
    pub method: String,
    /// Parameters for the method
    pub params: Option<Value>,
}

impl PluginRequest {
    /// Create a new plugin request
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            id: next_message_id(),
            method: method.into(),
            params,
        }
    }

    /// Convert to a PluginMessage for sending
    pub fn into_message(self) -> PluginMessage {
        PluginMessage {
            id: self.id,
            message_type: PluginMessageType::Request,
            result: self.params,
            error: None,
        }
    }
}

/// Standard host methods that plugins can call
pub mod host_methods {
    /// Get document content
    pub const GET_DOCUMENT: &str = "document.get";
    /// Set document content
    pub const SET_DOCUMENT: &str = "document.set";
    /// Get selection
    pub const GET_SELECTION: &str = "selection.get";
    /// Set selection
    pub const SET_SELECTION: &str = "selection.set";
    /// Show notification
    pub const SHOW_NOTIFICATION: &str = "ui.showNotification";
    /// Show dialog
    pub const SHOW_DIALOG: &str = "ui.showDialog";
    /// Update toolbar
    pub const UPDATE_TOOLBAR: &str = "ui.updateToolbar";
    /// Read from storage
    pub const STORAGE_GET: &str = "storage.get";
    /// Write to storage
    pub const STORAGE_SET: &str = "storage.set";
    /// Delete from storage
    pub const STORAGE_DELETE: &str = "storage.delete";
    /// Make HTTP request
    pub const HTTP_REQUEST: &str = "network.httpRequest";
    /// Read clipboard
    pub const CLIPBOARD_READ: &str = "clipboard.read";
    /// Write clipboard
    pub const CLIPBOARD_WRITE: &str = "clipboard.write";
    /// Log message
    pub const LOG: &str = "log";
}

/// Standard events sent to plugins
pub mod plugin_events {
    /// Document opened
    pub const DOCUMENT_OPENED: &str = "document.opened";
    /// Document closed
    pub const DOCUMENT_CLOSED: &str = "document.closed";
    /// Document changed
    pub const DOCUMENT_CHANGED: &str = "document.changed";
    /// Selection changed
    pub const SELECTION_CHANGED: &str = "selection.changed";
    /// Plugin activated
    pub const ACTIVATED: &str = "plugin.activated";
    /// Plugin deactivated
    pub const DEACTIVATED: &str = "plugin.deactivated";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PluginErrorCode;

    #[test]
    fn test_host_message_request() {
        reset_message_id_counter();
        let msg = HostMessage::request("test.method", Some(serde_json::json!({"key": "value"})));

        assert_eq!(msg.id, 1);
        assert!(msg.is_request());
        assert!(!msg.is_event());
        assert_eq!(msg.method, "test.method");
        assert!(msg.params.is_some());
    }

    #[test]
    fn test_host_message_event() {
        reset_message_id_counter();
        let msg = HostMessage::event("document.changed", None);

        assert_eq!(msg.id, 1);
        assert!(msg.is_event());
        assert!(!msg.is_request());
        assert!(msg.params.is_none());
    }

    #[test]
    fn test_host_message_with_id() {
        let msg = HostMessage::request_with_id(42, "test", None);
        assert_eq!(msg.id, 42);
    }

    #[test]
    fn test_plugin_message_response() {
        let msg = PluginMessage::response(1, serde_json::json!("result"));

        assert_eq!(msg.id, 1);
        assert!(msg.is_success());
        assert!(!msg.is_error());
        assert_eq!(msg.get_result(), Some(&serde_json::json!("result")));
        assert!(msg.get_error().is_none());
    }

    #[test]
    fn test_plugin_message_error_response() {
        let err = SerializablePluginError::new(PluginErrorCode::ExecutionError, "Failed");
        let msg = PluginMessage::error_response(1, err.clone());

        assert_eq!(msg.id, 1);
        assert!(!msg.is_success());
        assert!(msg.is_error());
        assert!(msg.get_result().is_none());
        assert_eq!(msg.get_error(), Some(&err));
    }

    #[test]
    fn test_plugin_request() {
        reset_message_id_counter();
        let req = PluginRequest::new("host.method", Some(serde_json::json!({"param": 1})));

        assert_eq!(req.id, 1);
        assert_eq!(req.method, "host.method");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_plugin_request_into_message() {
        let req = PluginRequest::new("test", None);
        let msg = req.clone().into_message();

        assert_eq!(msg.id, req.id);
        assert_eq!(msg.message_type, PluginMessageType::Request);
    }

    #[test]
    fn test_message_serialization() {
        let msg = HostMessage::request("test", Some(serde_json::json!({"x": 1})));
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: HostMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_plugin_message_serialization() {
        let msg = PluginMessage::response(1, serde_json::json!({"result": true}));
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: PluginMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_message_id_increments() {
        reset_message_id_counter();
        let id1 = next_message_id();
        let id2 = next_message_id();
        let id3 = next_message_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_host_methods_constants() {
        assert_eq!(host_methods::GET_DOCUMENT, "document.get");
        assert_eq!(host_methods::SHOW_NOTIFICATION, "ui.showNotification");
    }

    #[test]
    fn test_plugin_events_constants() {
        assert_eq!(plugin_events::DOCUMENT_OPENED, "document.opened");
        assert_eq!(plugin_events::ACTIVATED, "plugin.activated");
    }
}
