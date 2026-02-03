//! Client connection state management for the collaboration server.
//!
//! This module provides the `ClientConnection` struct that tracks the state
//! of connected clients including their identity, permissions, and communication channel.

use crate::permissions::{PermissionLevel, UserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

/// Unique identifier for a client connection.
///
/// This is separate from `UserId` because a single user may have multiple
/// concurrent connections (e.g., multiple browser tabs).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub String);

impl ClientId {
    /// Create a new client ID.
    pub fn new(id: impl Into<String>) -> Self {
        ClientId(id.into())
    }

    /// Generate a new unique client ID.
    pub fn generate() -> Self {
        ClientId(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        ClientId(s.to_string())
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        ClientId(s)
    }
}

/// Messages that can be sent to a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OutgoingMessage {
    /// CRDT operations to apply to the document.
    Operations(Vec<crate::operation::CrdtOp>),
    /// Presence update from another user.
    PresenceUpdate(crate::presence::PresenceState),
    /// User joined the session.
    UserJoined {
        user_id: String,
        display_name: String,
        color: String,
    },
    /// User left the session.
    UserLeft { user_id: String },
    /// Session state snapshot (for initial sync).
    SessionSnapshot {
        document_state: Vec<u8>,
        version: crate::clock::VectorClock,
        active_users: Vec<crate::presence::PresenceState>,
    },
    /// Acknowledgment of received operations.
    Acknowledgment { op_ids: Vec<crate::op_id::OpId> },
    /// Error message.
    Error { code: String, message: String },
    /// Ping for keepalive.
    Ping { timestamp: u64 },
    /// Pong response.
    Pong { timestamp: u64 },
    /// Permission change notification.
    PermissionChanged { new_level: PermissionLevel },
    /// Document metadata update.
    MetadataUpdate { metadata: serde_json::Value },
}

/// Connection status of a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Client is connecting but not yet fully authenticated.
    Connecting,
    /// Client is connected and authenticated.
    Connected,
    /// Client is disconnected but may reconnect.
    Disconnected,
    /// Client connection has been terminated.
    Terminated,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Connecting
    }
}

/// Represents a connected client's state.
///
/// Each `ClientConnection` tracks a single client's connection to a document session,
/// including their identity, permissions, and the channel for sending messages.
#[derive(Debug)]
pub struct ClientConnection {
    /// Unique identifier for this connection.
    client_id: ClientId,
    /// The user ID associated with this connection.
    user_id: UserId,
    /// Display name for presence features.
    display_name: String,
    /// Channel for sending messages to this client.
    sender: mpsc::UnboundedSender<OutgoingMessage>,
    /// The client's permission level for the document.
    permission_level: PermissionLevel,
    /// Timestamp of last activity (ms since epoch).
    last_activity: u64,
    /// Current connection status.
    status: ConnectionStatus,
    /// Assigned color for presence display.
    color: String,
    /// Client's vector clock for sync tracking.
    vector_clock: crate::clock::VectorClock,
    /// Number of messages sent to this client.
    messages_sent: u64,
    /// Number of messages received from this client.
    messages_received: u64,
    /// Optional metadata associated with this connection.
    metadata: Option<serde_json::Value>,
}

impl ClientConnection {
    /// Create a new client connection.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Unique identifier for this connection.
    /// * `user_id` - The user ID associated with this connection.
    /// * `display_name` - Display name for presence features.
    /// * `sender` - Channel for sending messages to this client.
    /// * `permission_level` - The client's permission level.
    /// * `color` - Assigned color for presence display.
    ///
    /// # Returns
    ///
    /// A new `ClientConnection` instance.
    pub fn new(
        client_id: ClientId,
        user_id: UserId,
        display_name: String,
        sender: mpsc::UnboundedSender<OutgoingMessage>,
        permission_level: PermissionLevel,
        color: String,
    ) -> Self {
        Self {
            client_id,
            user_id,
            display_name,
            sender,
            permission_level,
            last_activity: current_timestamp_ms(),
            status: ConnectionStatus::Connected,
            color,
            vector_clock: crate::clock::VectorClock::new(),
            messages_sent: 0,
            messages_received: 0,
            metadata: None,
        }
    }

    /// Get the client ID.
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Get the user ID.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Get the display name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Set the display name.
    pub fn set_display_name(&mut self, name: String) {
        self.display_name = name;
        self.touch();
    }

    /// Get the permission level.
    pub fn permission_level(&self) -> PermissionLevel {
        self.permission_level
    }

    /// Set the permission level.
    pub fn set_permission_level(&mut self, level: PermissionLevel) {
        self.permission_level = level;
        self.touch();
    }

    /// Get the last activity timestamp.
    pub fn last_activity(&self) -> u64 {
        self.last_activity
    }

    /// Get the connection status.
    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    /// Set the connection status.
    pub fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;
        self.touch();
    }

    /// Get the assigned color.
    pub fn color(&self) -> &str {
        &self.color
    }

    /// Set the assigned color.
    pub fn set_color(&mut self, color: String) {
        self.color = color;
    }

    /// Get the client's vector clock.
    pub fn vector_clock(&self) -> &crate::clock::VectorClock {
        &self.vector_clock
    }

    /// Get a mutable reference to the client's vector clock.
    pub fn vector_clock_mut(&mut self) -> &mut crate::clock::VectorClock {
        &mut self.vector_clock
    }

    /// Update the client's vector clock.
    pub fn set_vector_clock(&mut self, clock: crate::clock::VectorClock) {
        self.vector_clock = clock;
        self.touch();
    }

    /// Get the number of messages sent.
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent
    }

    /// Get the number of messages received.
    pub fn messages_received(&self) -> u64 {
        self.messages_received
    }

    /// Record a received message.
    pub fn record_message_received(&mut self) {
        self.messages_received += 1;
        self.touch();
    }

    /// Get the optional metadata.
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    /// Set the metadata.
    pub fn set_metadata(&mut self, metadata: Option<serde_json::Value>) {
        self.metadata = metadata;
    }

    /// Update the last activity timestamp.
    pub fn touch(&mut self) {
        self.last_activity = current_timestamp_ms();
    }

    /// Check if the client is idle (no activity for the given threshold).
    pub fn is_idle(&self, threshold_ms: u64) -> bool {
        let now = current_timestamp_ms();
        now.saturating_sub(self.last_activity) > threshold_ms
    }

    /// Check if the client is connected.
    pub fn is_connected(&self) -> bool {
        self.status == ConnectionStatus::Connected
    }

    /// Check if the client can view the document.
    pub fn can_view(&self) -> bool {
        self.permission_level.can_view()
    }

    /// Check if the client can edit the document.
    pub fn can_edit(&self) -> bool {
        self.permission_level.can_edit()
    }

    /// Check if the client can comment on the document.
    pub fn can_comment(&self) -> bool {
        self.permission_level.can_comment()
    }

    /// Check if the client can manage the document.
    pub fn can_manage(&self) -> bool {
        self.permission_level.can_manage()
    }

    /// Send a message to this client.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message was sent successfully, or an error if the channel is closed.
    pub fn send(&mut self, message: OutgoingMessage) -> Result<(), ClientError> {
        if self.status != ConnectionStatus::Connected {
            return Err(ClientError::NotConnected);
        }

        self.sender
            .send(message)
            .map_err(|_| ClientError::ChannelClosed)?;

        self.messages_sent += 1;
        Ok(())
    }

    /// Send operations to this client.
    ///
    /// # Arguments
    ///
    /// * `ops` - The CRDT operations to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn send_operations(&mut self, ops: Vec<crate::operation::CrdtOp>) -> Result<(), ClientError> {
        self.send(OutgoingMessage::Operations(ops))
    }

    /// Send a presence update to this client.
    ///
    /// # Arguments
    ///
    /// * `presence` - The presence state to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn send_presence_update(
        &mut self,
        presence: crate::presence::PresenceState,
    ) -> Result<(), ClientError> {
        self.send(OutgoingMessage::PresenceUpdate(presence))
    }

    /// Send an error message to this client.
    ///
    /// # Arguments
    ///
    /// * `code` - Error code.
    /// * `message` - Error message.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn send_error(&mut self, code: impl Into<String>, message: impl Into<String>) -> Result<(), ClientError> {
        self.send(OutgoingMessage::Error {
            code: code.into(),
            message: message.into(),
        })
    }

    /// Send a ping to this client.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn send_ping(&mut self) -> Result<(), ClientError> {
        self.send(OutgoingMessage::Ping {
            timestamp: current_timestamp_ms(),
        })
    }

    /// Disconnect this client gracefully.
    pub fn disconnect(&mut self) {
        self.status = ConnectionStatus::Disconnected;
    }

    /// Terminate this client connection.
    pub fn terminate(&mut self) {
        self.status = ConnectionStatus::Terminated;
    }

    /// Create a `PresenceState` from this client's information.
    pub fn to_presence_state(&self) -> crate::presence::PresenceState {
        crate::presence::PresenceState::new(
            self.user_id.0.clone(),
            self.display_name.clone(),
            self.color.clone(),
        )
    }

    /// Get connection statistics.
    pub fn stats(&self) -> ClientStats {
        ClientStats {
            client_id: self.client_id.clone(),
            user_id: self.user_id.clone(),
            status: self.status,
            messages_sent: self.messages_sent,
            messages_received: self.messages_received,
            last_activity: self.last_activity,
            permission_level: self.permission_level,
        }
    }
}

/// Statistics about a client connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientStats {
    /// Client ID.
    pub client_id: ClientId,
    /// User ID.
    pub user_id: UserId,
    /// Connection status.
    pub status: ConnectionStatus,
    /// Number of messages sent.
    pub messages_sent: u64,
    /// Number of messages received.
    pub messages_received: u64,
    /// Last activity timestamp.
    pub last_activity: u64,
    /// Permission level.
    pub permission_level: PermissionLevel,
}

/// Errors that can occur with client connections.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ClientError {
    /// The client is not connected.
    #[error("Client is not connected")]
    NotConnected,

    /// The message channel is closed.
    #[error("Message channel is closed")]
    ChannelClosed,

    /// Permission denied for the operation.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// The client connection timed out.
    #[error("Connection timed out")]
    Timeout,

    /// Invalid message format.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

/// Builder for creating `ClientConnection` instances.
pub struct ClientConnectionBuilder {
    client_id: Option<ClientId>,
    user_id: Option<UserId>,
    display_name: Option<String>,
    sender: Option<mpsc::UnboundedSender<OutgoingMessage>>,
    permission_level: PermissionLevel,
    color: Option<String>,
    metadata: Option<serde_json::Value>,
}

impl ClientConnectionBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            client_id: None,
            user_id: None,
            display_name: None,
            sender: None,
            permission_level: PermissionLevel::Viewer,
            color: None,
            metadata: None,
        }
    }

    /// Set the client ID.
    pub fn client_id(mut self, id: ClientId) -> Self {
        self.client_id = Some(id);
        self
    }

    /// Set the user ID.
    pub fn user_id(mut self, id: UserId) -> Self {
        self.user_id = Some(id);
        self
    }

    /// Set the display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set the message sender channel.
    pub fn sender(mut self, sender: mpsc::UnboundedSender<OutgoingMessage>) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Set the permission level.
    pub fn permission_level(mut self, level: PermissionLevel) -> Self {
        self.permission_level = level;
        self
    }

    /// Set the color.
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the metadata.
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build the `ClientConnection`.
    ///
    /// # Returns
    ///
    /// `Ok(ClientConnection)` if all required fields are set, or an error.
    pub fn build(self) -> Result<ClientConnection, ClientError> {
        let client_id = self.client_id.unwrap_or_else(ClientId::generate);
        let user_id = self.user_id.ok_or_else(|| {
            ClientError::InvalidMessage("user_id is required".to_string())
        })?;
        let display_name = self.display_name.unwrap_or_else(|| user_id.0.clone());
        let sender = self.sender.ok_or_else(|| {
            ClientError::InvalidMessage("sender is required".to_string())
        })?;
        let color = self.color.unwrap_or_else(|| "#2196F3".to_string());

        let mut conn = ClientConnection::new(
            client_id,
            user_id,
            display_name,
            sender,
            self.permission_level,
            color,
        );
        conn.set_metadata(self.metadata);

        Ok(conn)
    }
}

impl Default for ClientConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the current timestamp in milliseconds since epoch.
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sender() -> (mpsc::UnboundedSender<OutgoingMessage>, mpsc::UnboundedReceiver<OutgoingMessage>) {
        mpsc::unbounded_channel()
    }

    #[test]
    fn test_client_id_generation() {
        let id1 = ClientId::generate();
        let id2 = ClientId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_client_id_from_string() {
        let id: ClientId = "test-client".into();
        assert_eq!(id.0, "test-client");
    }

    #[test]
    fn test_client_connection_new() {
        let (sender, _receiver) = create_test_sender();
        let conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        assert_eq!(conn.client_id().0, "client-1");
        assert_eq!(conn.user_id().0, "user-1");
        assert_eq!(conn.display_name(), "Alice");
        assert_eq!(conn.permission_level(), PermissionLevel::Editor);
        assert!(conn.is_connected());
        assert!(conn.can_edit());
        assert!(conn.can_view());
        assert!(!conn.can_manage());
    }

    #[test]
    fn test_client_connection_send() {
        let (sender, mut receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        // Send a message
        let result = conn.send(OutgoingMessage::Ping { timestamp: 12345 });
        assert!(result.is_ok());
        assert_eq!(conn.messages_sent(), 1);

        // Verify message was received
        let msg = receiver.try_recv().unwrap();
        if let OutgoingMessage::Ping { timestamp } = msg {
            assert_eq!(timestamp, 12345);
        } else {
            panic!("Expected Ping message");
        }
    }

    #[test]
    fn test_client_connection_send_operations() {
        let (sender, mut receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        let result = conn.send_operations(vec![]);
        assert!(result.is_ok());

        let msg = receiver.try_recv().unwrap();
        assert!(matches!(msg, OutgoingMessage::Operations(_)));
    }

    #[test]
    fn test_client_connection_send_error() {
        let (sender, mut receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        let result = conn.send_error("ERR001", "Test error");
        assert!(result.is_ok());

        let msg = receiver.try_recv().unwrap();
        if let OutgoingMessage::Error { code, message } = msg {
            assert_eq!(code, "ERR001");
            assert_eq!(message, "Test error");
        } else {
            panic!("Expected Error message");
        }
    }

    #[test]
    fn test_client_connection_disconnect() {
        let (sender, _receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        assert!(conn.is_connected());
        conn.disconnect();
        assert!(!conn.is_connected());
        assert_eq!(conn.status(), ConnectionStatus::Disconnected);

        // Should not be able to send after disconnect
        let result = conn.send(OutgoingMessage::Ping { timestamp: 12345 });
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[test]
    fn test_client_connection_terminate() {
        let (sender, _receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        conn.terminate();
        assert_eq!(conn.status(), ConnectionStatus::Terminated);
        assert!(!conn.is_connected());
    }

    #[test]
    fn test_client_connection_permission_checks() {
        let (sender, _receiver) = create_test_sender();

        // Viewer
        let conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender.clone(),
            PermissionLevel::Viewer,
            "#E91E63".to_string(),
        );
        assert!(conn.can_view());
        assert!(!conn.can_comment());
        assert!(!conn.can_edit());
        assert!(!conn.can_manage());

        // Commenter
        let conn = ClientConnection::new(
            ClientId::new("client-2"),
            UserId::from("user-2"),
            "Bob".to_string(),
            sender.clone(),
            PermissionLevel::Commenter,
            "#9C27B0".to_string(),
        );
        assert!(conn.can_view());
        assert!(conn.can_comment());
        assert!(!conn.can_edit());
        assert!(!conn.can_manage());

        // Editor
        let conn = ClientConnection::new(
            ClientId::new("client-3"),
            UserId::from("user-3"),
            "Charlie".to_string(),
            sender.clone(),
            PermissionLevel::Editor,
            "#3F51B5".to_string(),
        );
        assert!(conn.can_view());
        assert!(conn.can_comment());
        assert!(conn.can_edit());
        assert!(!conn.can_manage());

        // Owner
        let conn = ClientConnection::new(
            ClientId::new("client-4"),
            UserId::from("user-4"),
            "Diana".to_string(),
            sender,
            PermissionLevel::Owner,
            "#2196F3".to_string(),
        );
        assert!(conn.can_view());
        assert!(conn.can_comment());
        assert!(conn.can_edit());
        assert!(conn.can_manage());
    }

    #[test]
    fn test_client_connection_is_idle() {
        let (sender, _receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        // Fresh connection should not be idle
        assert!(!conn.is_idle(60_000));

        // Manually set old timestamp to simulate idle
        conn.last_activity = current_timestamp_ms().saturating_sub(120_000);
        assert!(conn.is_idle(60_000));

        // Touch should reset
        conn.touch();
        assert!(!conn.is_idle(60_000));
    }

    #[test]
    fn test_client_connection_stats() {
        let (sender, _receiver) = create_test_sender();
        let mut conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        conn.record_message_received();
        conn.record_message_received();
        let _ = conn.send_ping();

        let stats = conn.stats();
        assert_eq!(stats.client_id.0, "client-1");
        assert_eq!(stats.user_id.0, "user-1");
        assert_eq!(stats.messages_received, 2);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.permission_level, PermissionLevel::Editor);
    }

    #[test]
    fn test_client_connection_to_presence_state() {
        let (sender, _receiver) = create_test_sender();
        let conn = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "Alice".to_string(),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );

        let presence = conn.to_presence_state();
        assert_eq!(presence.user_id, "user-1");
        assert_eq!(presence.display_name, "Alice");
        assert_eq!(presence.color, "#E91E63");
    }

    #[test]
    fn test_client_connection_builder() {
        let (sender, _receiver) = create_test_sender();

        let conn = ClientConnectionBuilder::new()
            .client_id(ClientId::new("client-1"))
            .user_id(UserId::from("user-1"))
            .display_name("Alice")
            .sender(sender)
            .permission_level(PermissionLevel::Editor)
            .color("#E91E63")
            .build()
            .unwrap();

        assert_eq!(conn.client_id().0, "client-1");
        assert_eq!(conn.user_id().0, "user-1");
        assert_eq!(conn.display_name(), "Alice");
        assert_eq!(conn.permission_level(), PermissionLevel::Editor);
    }

    #[test]
    fn test_client_connection_builder_missing_user_id() {
        let (sender, _receiver) = create_test_sender();

        let result = ClientConnectionBuilder::new()
            .sender(sender)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_client_connection_builder_missing_sender() {
        let result = ClientConnectionBuilder::new()
            .user_id(UserId::from("user-1"))
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_client_connection_builder_defaults() {
        let (sender, _receiver) = create_test_sender();

        let conn = ClientConnectionBuilder::new()
            .user_id(UserId::from("user-1"))
            .sender(sender)
            .build()
            .unwrap();

        // Should have auto-generated client_id
        assert!(!conn.client_id().0.is_empty());
        // Display name defaults to user_id
        assert_eq!(conn.display_name(), "user-1");
        // Default permission level is Viewer
        assert_eq!(conn.permission_level(), PermissionLevel::Viewer);
        // Default color
        assert_eq!(conn.color(), "#2196F3");
    }

    #[test]
    fn test_connection_status_default() {
        let status = ConnectionStatus::default();
        assert_eq!(status, ConnectionStatus::Connecting);
    }

    #[test]
    fn test_outgoing_message_serialization() {
        let msg = OutgoingMessage::Error {
            code: "ERR001".to_string(),
            message: "Test error".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: OutgoingMessage = serde_json::from_str(&json).unwrap();

        if let OutgoingMessage::Error { code, message } = deserialized {
            assert_eq!(code, "ERR001");
            assert_eq!(message, "Test error");
        } else {
            panic!("Expected Error message");
        }
    }
}
