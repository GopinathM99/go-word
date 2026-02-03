//! Document session management for the collaboration server.
//!
//! This module provides the `DocumentSession` struct that manages the state
//! of a collaborative editing session for a single document, including connected
//! clients, document state, and operation broadcasting.

use crate::bridge::CollaborativeDocument;
use crate::clock::VectorClock;
use crate::operation::CrdtOp;
use crate::permissions::{DocId, PermissionLevel, PermissionManager, UserId};
use crate::presence::PresenceManager;
use crate::server::client::{ClientConnection, ClientError, ClientId, OutgoingMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Configuration for a document session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum number of clients allowed in the session.
    pub max_clients: usize,
    /// Idle timeout in milliseconds before a client is considered inactive.
    pub idle_timeout_ms: u64,
    /// Interval for presence updates in milliseconds.
    pub presence_update_interval_ms: u64,
    /// Whether to allow anonymous clients (viewers without authentication).
    pub allow_anonymous: bool,
    /// Maximum operations per batch for broadcasting.
    pub max_ops_per_batch: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_clients: 100,
            idle_timeout_ms: 300_000, // 5 minutes
            presence_update_interval_ms: 1000,
            allow_anonymous: false,
            max_ops_per_batch: 100,
        }
    }
}

/// Status of a document session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is initializing.
    Initializing,
    /// Session is active and accepting clients.
    Active,
    /// Session is paused (e.g., for maintenance).
    Paused,
    /// Session is closing.
    Closing,
    /// Session is closed.
    Closed,
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Initializing
    }
}

/// Errors that can occur with document sessions.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SessionError {
    /// Client not found in the session.
    #[error("Client not found: {0}")]
    ClientNotFound(ClientId),

    /// Session is full.
    #[error("Session is full (max: {0})")]
    SessionFull(usize),

    /// Session is not active.
    #[error("Session is not active: {0:?}")]
    SessionNotActive(SessionStatus),

    /// Permission denied for the operation.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Client error.
    #[error("Client error: {0}")]
    ClientError(#[from] ClientError),

    /// Document error.
    #[error("Document error: {0}")]
    DocumentError(String),

    /// Duplicate client ID.
    #[error("Duplicate client ID: {0}")]
    DuplicateClientId(ClientId),
}

/// A document session managing collaborative editing for a single document.
///
/// Each `DocumentSession` tracks connected clients, maintains the collaborative
/// document state (CRDT), and handles operation broadcasting.
pub struct DocumentSession {
    /// The document ID for this session.
    document_id: DocId,
    /// Connected clients mapped by their client ID.
    clients: HashMap<ClientId, ClientConnection>,
    /// The collaborative document state.
    document: CollaborativeDocument,
    /// Current vector clock for the document.
    version: VectorClock,
    /// Presence manager for tracking user cursors and selections.
    presence: PresenceManager,
    /// Session configuration.
    config: SessionConfig,
    /// Session status.
    status: SessionStatus,
    /// Session creation timestamp.
    created_at: u64,
    /// Last activity timestamp.
    last_activity: u64,
    /// Total operations processed.
    total_operations: u64,
    /// Session metadata.
    metadata: Option<serde_json::Value>,
}

impl DocumentSession {
    /// Create a new document session.
    ///
    /// # Arguments
    ///
    /// * `document_id` - The document ID for this session.
    /// * `document` - The collaborative document state.
    /// * `config` - Session configuration.
    ///
    /// # Returns
    ///
    /// A new `DocumentSession` instance.
    pub fn new(
        document_id: DocId,
        document: CollaborativeDocument,
        config: SessionConfig,
    ) -> Self {
        let now = current_timestamp_ms();
        Self {
            document_id,
            clients: HashMap::new(),
            document,
            version: VectorClock::new(),
            presence: PresenceManager::with_idle_threshold(config.idle_timeout_ms),
            config,
            status: SessionStatus::Active,
            created_at: now,
            last_activity: now,
            total_operations: 0,
            metadata: None,
        }
    }

    /// Create a new document session with an empty document.
    ///
    /// # Arguments
    ///
    /// * `document_id` - The document ID for this session.
    /// * `client_id` - The CRDT client ID for the server.
    /// * `config` - Session configuration.
    ///
    /// # Returns
    ///
    /// A new `DocumentSession` instance with an empty document.
    pub fn new_empty(
        document_id: DocId,
        client_id: crate::op_id::ClientId,
        config: SessionConfig,
    ) -> Self {
        let document = CollaborativeDocument::new(client_id);
        Self::new(document_id, document, config)
    }

    /// Get the document ID.
    pub fn document_id(&self) -> &DocId {
        &self.document_id
    }

    /// Get the current version (vector clock).
    pub fn version(&self) -> &VectorClock {
        &self.version
    }

    /// Get the session status.
    pub fn status(&self) -> SessionStatus {
        self.status
    }

    /// Set the session status.
    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status;
    }

    /// Get the session configuration.
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Get the creation timestamp.
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Get the last activity timestamp.
    pub fn last_activity(&self) -> u64 {
        self.last_activity
    }

    /// Get the total operations processed.
    pub fn total_operations(&self) -> u64 {
        self.total_operations
    }

    /// Get the session metadata.
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    /// Set the session metadata.
    pub fn set_metadata(&mut self, metadata: Option<serde_json::Value>) {
        self.metadata = metadata;
    }

    /// Get a reference to the collaborative document.
    pub fn document(&self) -> &CollaborativeDocument {
        &self.document
    }

    /// Get a mutable reference to the collaborative document.
    pub fn document_mut(&mut self) -> &mut CollaborativeDocument {
        &mut self.document
    }

    /// Get a reference to the presence manager.
    pub fn presence(&self) -> &PresenceManager {
        &self.presence
    }

    /// Get a mutable reference to the presence manager.
    pub fn presence_mut(&mut self) -> &mut PresenceManager {
        &mut self.presence
    }

    /// Update the last activity timestamp.
    fn touch(&mut self) {
        self.last_activity = current_timestamp_ms();
    }

    // ========== Client Management ==========

    /// Join a client to this session.
    ///
    /// # Arguments
    ///
    /// * `client` - The client connection to add.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the client was added successfully, or an error.
    pub fn join_session(&mut self, client: ClientConnection) -> Result<(), SessionError> {
        // Check session status
        if self.status != SessionStatus::Active {
            return Err(SessionError::SessionNotActive(self.status));
        }

        // Check capacity
        if self.clients.len() >= self.config.max_clients {
            return Err(SessionError::SessionFull(self.config.max_clients));
        }

        // Check for duplicate client ID
        let client_id = client.client_id().clone();
        if self.clients.contains_key(&client_id) {
            return Err(SessionError::DuplicateClientId(client_id));
        }

        // Add to presence manager
        let presence_state = client.to_presence_state();
        self.presence.update_user(presence_state);

        // Add client
        self.clients.insert(client_id.clone(), client);
        self.touch();

        // Broadcast user joined event to other clients
        let _ = self.broadcast_to_others(
            &client_id,
            OutgoingMessage::UserJoined {
                user_id: self.clients.get(&client_id).map(|c| c.user_id().0.clone()).unwrap_or_default(),
                display_name: self.clients.get(&client_id).map(|c| c.display_name().to_string()).unwrap_or_default(),
                color: self.clients.get(&client_id).map(|c| c.color().to_string()).unwrap_or_default(),
            },
        );

        Ok(())
    }

    /// Remove a client from this session.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID to remove.
    ///
    /// # Returns
    ///
    /// `Ok(ClientConnection)` if the client was removed, or an error.
    pub fn leave_session(&mut self, client_id: &ClientId) -> Result<ClientConnection, SessionError> {
        let client = self.clients.remove(client_id)
            .ok_or_else(|| SessionError::ClientNotFound(client_id.clone()))?;

        // Remove from presence manager
        self.presence.remove_user(&client.user_id().0);
        self.touch();

        // Broadcast user left event
        let _ = self.broadcast_to_all(OutgoingMessage::UserLeft {
            user_id: client.user_id().0.clone(),
        });

        Ok(client)
    }

    /// Get a reference to a connected client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID.
    ///
    /// # Returns
    ///
    /// `Some(&ClientConnection)` if found, `None` otherwise.
    pub fn get_client(&self, client_id: &ClientId) -> Option<&ClientConnection> {
        self.clients.get(client_id)
    }

    /// Get a mutable reference to a connected client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID.
    ///
    /// # Returns
    ///
    /// `Some(&mut ClientConnection)` if found, `None` otherwise.
    pub fn get_client_mut(&mut self, client_id: &ClientId) -> Option<&mut ClientConnection> {
        self.clients.get_mut(client_id)
    }

    /// Get all connected clients.
    ///
    /// # Returns
    ///
    /// A HashMap of client IDs to client connections.
    pub fn get_clients(&self) -> &HashMap<ClientId, ClientConnection> {
        &self.clients
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Check if a client is connected.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID.
    ///
    /// # Returns
    ///
    /// `true` if the client is connected, `false` otherwise.
    pub fn has_client(&self, client_id: &ClientId) -> bool {
        self.clients.contains_key(client_id)
    }

    /// Check if the session is empty (no connected clients).
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Get all client IDs.
    pub fn client_ids(&self) -> Vec<ClientId> {
        self.clients.keys().cloned().collect()
    }

    /// Get all clients with a specific permission level or higher.
    pub fn clients_with_permission(&self, min_level: PermissionLevel) -> Vec<&ClientConnection> {
        self.clients
            .values()
            .filter(|c| c.permission_level() >= min_level)
            .collect()
    }

    // ========== Broadcasting ==========

    /// Broadcast a message to all connected clients.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast.
    ///
    /// # Returns
    ///
    /// A vector of client IDs that failed to receive the message.
    pub fn broadcast_to_all(&mut self, message: OutgoingMessage) -> Vec<ClientId> {
        let mut failed = Vec::new();
        let client_ids: Vec<ClientId> = self.clients.keys().cloned().collect();

        for client_id in client_ids {
            if let Some(client) = self.clients.get_mut(&client_id) {
                if let Err(_) = client.send(message.clone()) {
                    failed.push(client_id);
                }
            }
        }

        self.touch();
        failed
    }

    /// Broadcast a message to all clients except the specified one.
    ///
    /// # Arguments
    ///
    /// * `exclude_client_id` - The client ID to exclude.
    /// * `message` - The message to broadcast.
    ///
    /// # Returns
    ///
    /// A vector of client IDs that failed to receive the message.
    pub fn broadcast_to_others(
        &mut self,
        exclude_client_id: &ClientId,
        message: OutgoingMessage,
    ) -> Vec<ClientId> {
        let mut failed = Vec::new();
        let client_ids: Vec<ClientId> = self.clients.keys()
            .filter(|id| *id != exclude_client_id)
            .cloned()
            .collect();

        for client_id in client_ids {
            if let Some(client) = self.clients.get_mut(&client_id) {
                if let Err(_) = client.send(message.clone()) {
                    failed.push(client_id);
                }
            }
        }

        self.touch();
        failed
    }

    /// Broadcast operations to all clients except the sender.
    ///
    /// # Arguments
    ///
    /// * `sender_client_id` - The client ID that originated the operations.
    /// * `ops` - The operations to broadcast.
    ///
    /// # Returns
    ///
    /// A vector of client IDs that failed to receive the operations.
    pub fn broadcast_operations(
        &mut self,
        sender_client_id: &ClientId,
        ops: Vec<CrdtOp>,
    ) -> Vec<ClientId> {
        if ops.is_empty() {
            return Vec::new();
        }

        self.broadcast_to_others(sender_client_id, OutgoingMessage::Operations(ops))
    }

    /// Send a message to a specific client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID.
    /// * `message` - The message to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn send_to_client(
        &mut self,
        client_id: &ClientId,
        message: OutgoingMessage,
    ) -> Result<(), SessionError> {
        let client = self.clients.get_mut(client_id)
            .ok_or_else(|| SessionError::ClientNotFound(client_id.clone()))?;

        client.send(message)?;
        self.touch();
        Ok(())
    }

    // ========== Operation Handling ==========

    /// Apply operations from a client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID that sent the operations.
    /// * `ops` - The operations to apply.
    ///
    /// # Returns
    ///
    /// `Ok(applied_count)` with the number of operations applied, or an error.
    pub fn apply_operations(
        &mut self,
        client_id: &ClientId,
        ops: Vec<CrdtOp>,
    ) -> Result<usize, SessionError> {
        // Check session status
        if self.status != SessionStatus::Active {
            return Err(SessionError::SessionNotActive(self.status));
        }

        // Check client exists and has edit permission
        let client = self.clients.get(client_id)
            .ok_or_else(|| SessionError::ClientNotFound(client_id.clone()))?;

        if !client.can_edit() {
            return Err(SessionError::PermissionDenied(
                "Client does not have edit permission".to_string(),
            ));
        }

        // Apply operations to the document
        let applied = self.document.apply_remote_batch(ops.clone());
        self.total_operations += applied as u64;

        // Update version clock
        for op in &ops {
            let op_id = op.id();
            let current = self.version.get(op_id.client_id);
            if op_id.seq > current {
                self.version.set(op_id.client_id, op_id.seq);
            }
        }

        // Record message received
        if let Some(client) = self.clients.get_mut(client_id) {
            client.record_message_received();
        }

        // Broadcast to other clients
        if applied > 0 {
            let _ = self.broadcast_operations(client_id, ops);
        }

        self.touch();
        Ok(applied)
    }

    /// Get operations since a given vector clock.
    ///
    /// # Arguments
    ///
    /// * `since` - The vector clock to compare against.
    ///
    /// # Returns
    ///
    /// A vector of operations that occurred after the given clock.
    pub fn ops_since(&self, since: &VectorClock) -> Vec<&CrdtOp> {
        self.document.ops_since(since)
    }

    // ========== Presence ==========

    /// Update presence for a client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID.
    /// * `presence` - The presence state update.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn update_presence(
        &mut self,
        client_id: &ClientId,
        presence: crate::presence::PresenceState,
    ) -> Result<(), SessionError> {
        // Check client exists
        if !self.clients.contains_key(client_id) {
            return Err(SessionError::ClientNotFound(client_id.clone()));
        }

        // Update presence manager
        self.presence.update_user(presence.clone());

        // Broadcast to other clients
        let _ = self.broadcast_to_others(client_id, OutgoingMessage::PresenceUpdate(presence));

        self.touch();
        Ok(())
    }

    // ========== Session Management ==========

    /// Clean up idle clients.
    ///
    /// # Returns
    ///
    /// A vector of client IDs that were removed due to inactivity.
    pub fn cleanup_idle_clients(&mut self) -> Vec<ClientId> {
        let idle_client_ids: Vec<ClientId> = self.clients
            .iter()
            .filter(|(_, client)| client.is_idle(self.config.idle_timeout_ms))
            .map(|(id, _)| id.clone())
            .collect();

        for client_id in &idle_client_ids {
            let _ = self.leave_session(client_id);
        }

        // Also clean up presence manager
        self.presence.cleanup_idle();

        idle_client_ids
    }

    /// Get session statistics.
    pub fn stats(&self) -> SessionStats {
        SessionStats {
            document_id: self.document_id.clone(),
            status: self.status,
            client_count: self.clients.len(),
            total_operations: self.total_operations,
            created_at: self.created_at,
            last_activity: self.last_activity,
            version: self.version.clone(),
        }
    }

    /// Check if the session should be closed (empty and idle for too long).
    pub fn should_close(&self, empty_timeout_ms: u64) -> bool {
        if !self.is_empty() {
            return false;
        }

        let now = current_timestamp_ms();
        now.saturating_sub(self.last_activity) > empty_timeout_ms
    }

    /// Close the session.
    ///
    /// Notifies all connected clients and sets status to Closed.
    pub fn close(&mut self) {
        self.status = SessionStatus::Closing;

        // Notify all clients
        let _ = self.broadcast_to_all(OutgoingMessage::Error {
            code: "SESSION_CLOSED".to_string(),
            message: "The document session has been closed".to_string(),
        });

        // Disconnect all clients
        let client_ids: Vec<ClientId> = self.clients.keys().cloned().collect();
        for client_id in client_ids {
            if let Some(client) = self.clients.get_mut(&client_id) {
                client.disconnect();
            }
        }

        self.clients.clear();
        self.status = SessionStatus::Closed;
    }

    /// Create a snapshot of the session for initial sync.
    ///
    /// # Arguments
    ///
    /// * `for_client_id` - The client requesting the snapshot.
    ///
    /// # Returns
    ///
    /// An `OutgoingMessage::SessionSnapshot` with the current state.
    pub fn create_snapshot(&self, _for_client_id: &ClientId) -> Result<OutgoingMessage, SessionError> {
        // Serialize document state (simplified - in production would use proper serialization)
        let document_state = serde_json::to_vec(&self.document.pending_ops())
            .map_err(|e| SessionError::DocumentError(e.to_string()))?;

        let active_users: Vec<crate::presence::PresenceState> = self.presence
            .all_users()
            .into_iter()
            .cloned()
            .collect();

        Ok(OutgoingMessage::SessionSnapshot {
            document_state,
            version: self.version.clone(),
            active_users,
        })
    }
}

/// Statistics about a document session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionStats {
    /// Document ID.
    pub document_id: DocId,
    /// Session status.
    pub status: SessionStatus,
    /// Number of connected clients.
    pub client_count: usize,
    /// Total operations processed.
    pub total_operations: u64,
    /// Session creation timestamp.
    pub created_at: u64,
    /// Last activity timestamp.
    pub last_activity: u64,
    /// Current version (vector clock).
    pub version: VectorClock,
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
    use crate::permissions::UserId;
    use tokio::sync::mpsc;

    fn create_test_sender() -> (mpsc::UnboundedSender<OutgoingMessage>, mpsc::UnboundedReceiver<OutgoingMessage>) {
        mpsc::unbounded_channel()
    }

    fn create_test_client(client_id: &str, user_id: &str) -> (ClientConnection, mpsc::UnboundedReceiver<OutgoingMessage>) {
        let (sender, receiver) = create_test_sender();
        let client = ClientConnection::new(
            ClientId::new(client_id),
            UserId::from(user_id),
            format!("User {}", user_id),
            sender,
            PermissionLevel::Editor,
            "#E91E63".to_string(),
        );
        (client, receiver)
    }

    fn create_test_session() -> DocumentSession {
        let doc_id = DocId::from("test-doc");
        let crdt_client_id = crate::op_id::ClientId::new(0);
        DocumentSession::new_empty(doc_id, crdt_client_id, SessionConfig::default())
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.max_clients, 100);
        assert_eq!(config.idle_timeout_ms, 300_000);
        assert!(!config.allow_anonymous);
    }

    #[test]
    fn test_document_session_new() {
        let session = create_test_session();
        assert_eq!(session.document_id().0, "test-doc");
        assert_eq!(session.status(), SessionStatus::Active);
        assert_eq!(session.client_count(), 0);
        assert!(session.is_empty());
    }

    #[test]
    fn test_join_session() {
        let mut session = create_test_session();
        let (client, _receiver) = create_test_client("client-1", "user-1");

        let result = session.join_session(client);
        assert!(result.is_ok());
        assert_eq!(session.client_count(), 1);
        assert!(!session.is_empty());
        assert!(session.has_client(&ClientId::new("client-1")));
    }

    #[test]
    fn test_join_session_duplicate_client() {
        let mut session = create_test_session();
        let (client1, _) = create_test_client("client-1", "user-1");
        let (client2, _) = create_test_client("client-1", "user-2"); // Same client ID

        session.join_session(client1).unwrap();
        let result = session.join_session(client2);

        assert!(matches!(result, Err(SessionError::DuplicateClientId(_))));
    }

    #[test]
    fn test_join_session_full() {
        let config = SessionConfig {
            max_clients: 2,
            ..Default::default()
        };
        let mut session = DocumentSession::new_empty(
            DocId::from("test-doc"),
            crate::op_id::ClientId::new(0),
            config,
        );

        let (client1, _) = create_test_client("client-1", "user-1");
        let (client2, _) = create_test_client("client-2", "user-2");
        let (client3, _) = create_test_client("client-3", "user-3");

        session.join_session(client1).unwrap();
        session.join_session(client2).unwrap();
        let result = session.join_session(client3);

        assert!(matches!(result, Err(SessionError::SessionFull(2))));
    }

    #[test]
    fn test_leave_session() {
        let mut session = create_test_session();
        let (client, _) = create_test_client("client-1", "user-1");

        session.join_session(client).unwrap();
        assert_eq!(session.client_count(), 1);

        let result = session.leave_session(&ClientId::new("client-1"));
        assert!(result.is_ok());
        assert_eq!(session.client_count(), 0);
        assert!(session.is_empty());
    }

    #[test]
    fn test_leave_session_not_found() {
        let mut session = create_test_session();

        let result = session.leave_session(&ClientId::new("nonexistent"));
        assert!(matches!(result, Err(SessionError::ClientNotFound(_))));
    }

    #[test]
    fn test_get_client() {
        let mut session = create_test_session();
        let (client, _) = create_test_client("client-1", "user-1");

        session.join_session(client).unwrap();

        let client_ref = session.get_client(&ClientId::new("client-1"));
        assert!(client_ref.is_some());
        assert_eq!(client_ref.unwrap().user_id().0, "user-1");

        let nonexistent = session.get_client(&ClientId::new("nonexistent"));
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_client_ids() {
        let mut session = create_test_session();
        let (client1, _) = create_test_client("client-1", "user-1");
        let (client2, _) = create_test_client("client-2", "user-2");

        session.join_session(client1).unwrap();
        session.join_session(client2).unwrap();

        let ids = session.client_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&ClientId::new("client-1")));
        assert!(ids.contains(&ClientId::new("client-2")));
    }

    #[test]
    fn test_broadcast_to_all() {
        let mut session = create_test_session();
        let (client1, mut receiver1) = create_test_client("client-1", "user-1");
        let (client2, mut receiver2) = create_test_client("client-2", "user-2");

        session.join_session(client1).unwrap();
        session.join_session(client2).unwrap();

        // Clear any join notifications first
        let _ = receiver1.try_recv();
        let _ = receiver2.try_recv();

        let failed = session.broadcast_to_all(OutgoingMessage::Ping { timestamp: 12345 });
        assert!(failed.is_empty());

        // Both clients should receive the message
        let msg1 = receiver1.try_recv().unwrap();
        let msg2 = receiver2.try_recv().unwrap();

        assert!(matches!(msg1, OutgoingMessage::Ping { timestamp: 12345 }));
        assert!(matches!(msg2, OutgoingMessage::Ping { timestamp: 12345 }));
    }

    #[test]
    fn test_broadcast_to_others() {
        let mut session = create_test_session();
        let (client1, mut receiver1) = create_test_client("client-1", "user-1");
        let (client2, mut receiver2) = create_test_client("client-2", "user-2");

        session.join_session(client1).unwrap();
        session.join_session(client2).unwrap();

        // Clear any join notifications
        while receiver1.try_recv().is_ok() {}
        while receiver2.try_recv().is_ok() {}

        let failed = session.broadcast_to_others(
            &ClientId::new("client-1"),
            OutgoingMessage::Ping { timestamp: 12345 },
        );
        assert!(failed.is_empty());

        // Only client-2 should receive the message
        let msg2 = receiver2.try_recv().unwrap();
        assert!(matches!(msg2, OutgoingMessage::Ping { timestamp: 12345 }));

        // client-1 should not receive the message
        assert!(receiver1.try_recv().is_err());
    }

    #[test]
    fn test_send_to_client() {
        let mut session = create_test_session();
        let (client, mut receiver) = create_test_client("client-1", "user-1");

        session.join_session(client).unwrap();

        // Clear any join notifications
        while receiver.try_recv().is_ok() {}

        let result = session.send_to_client(
            &ClientId::new("client-1"),
            OutgoingMessage::Ping { timestamp: 12345 },
        );
        assert!(result.is_ok());

        let msg = receiver.try_recv().unwrap();
        assert!(matches!(msg, OutgoingMessage::Ping { timestamp: 12345 }));
    }

    #[test]
    fn test_send_to_client_not_found() {
        let mut session = create_test_session();

        let result = session.send_to_client(
            &ClientId::new("nonexistent"),
            OutgoingMessage::Ping { timestamp: 12345 },
        );

        assert!(matches!(result, Err(SessionError::ClientNotFound(_))));
    }

    #[test]
    fn test_session_status_changes() {
        let mut session = create_test_session();
        assert_eq!(session.status(), SessionStatus::Active);

        session.set_status(SessionStatus::Paused);
        assert_eq!(session.status(), SessionStatus::Paused);

        // Cannot join paused session
        let (client, _) = create_test_client("client-1", "user-1");
        let result = session.join_session(client);
        assert!(matches!(result, Err(SessionError::SessionNotActive(_))));
    }

    #[test]
    fn test_session_stats() {
        let mut session = create_test_session();
        let (client, _) = create_test_client("client-1", "user-1");

        session.join_session(client).unwrap();

        let stats = session.stats();
        assert_eq!(stats.document_id.0, "test-doc");
        assert_eq!(stats.status, SessionStatus::Active);
        assert_eq!(stats.client_count, 1);
        assert_eq!(stats.total_operations, 0);
    }

    #[test]
    fn test_session_close() {
        let mut session = create_test_session();
        let (client, mut receiver) = create_test_client("client-1", "user-1");

        session.join_session(client).unwrap();

        // Clear any join notifications
        while receiver.try_recv().is_ok() {}

        session.close();

        assert_eq!(session.status(), SessionStatus::Closed);
        assert!(session.is_empty());

        // Client should have received close notification
        let msg = receiver.try_recv().unwrap();
        assert!(matches!(msg, OutgoingMessage::Error { code, .. } if code == "SESSION_CLOSED"));
    }

    #[test]
    fn test_clients_with_permission() {
        let mut session = create_test_session();

        let (sender1, _) = create_test_sender();
        let client1 = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "User 1".to_string(),
            sender1,
            PermissionLevel::Viewer,
            "#E91E63".to_string(),
        );

        let (sender2, _) = create_test_sender();
        let client2 = ClientConnection::new(
            ClientId::new("client-2"),
            UserId::from("user-2"),
            "User 2".to_string(),
            sender2,
            PermissionLevel::Editor,
            "#9C27B0".to_string(),
        );

        let (sender3, _) = create_test_sender();
        let client3 = ClientConnection::new(
            ClientId::new("client-3"),
            UserId::from("user-3"),
            "User 3".to_string(),
            sender3,
            PermissionLevel::Owner,
            "#3F51B5".to_string(),
        );

        session.join_session(client1).unwrap();
        session.join_session(client2).unwrap();
        session.join_session(client3).unwrap();

        // All clients (Viewer+)
        let viewers = session.clients_with_permission(PermissionLevel::Viewer);
        assert_eq!(viewers.len(), 3);

        // Editors and above
        let editors = session.clients_with_permission(PermissionLevel::Editor);
        assert_eq!(editors.len(), 2);

        // Owners only
        let owners = session.clients_with_permission(PermissionLevel::Owner);
        assert_eq!(owners.len(), 1);
    }

    #[test]
    fn test_session_should_close() {
        let mut session = create_test_session();

        // Not empty, should not close
        let (client, _) = create_test_client("client-1", "user-1");
        session.join_session(client).unwrap();
        assert!(!session.should_close(1000));

        // Remove client
        session.leave_session(&ClientId::new("client-1")).unwrap();

        // Empty but recent activity, should not close
        assert!(!session.should_close(1000));

        // Simulate old last_activity
        session.last_activity = current_timestamp_ms().saturating_sub(5000);

        // Empty and idle, should close
        assert!(session.should_close(1000));
    }

    #[test]
    fn test_session_metadata() {
        let mut session = create_test_session();

        assert!(session.metadata().is_none());

        session.set_metadata(Some(serde_json::json!({"title": "Test Document"})));

        let metadata = session.metadata().unwrap();
        assert_eq!(metadata["title"], "Test Document");
    }

    #[test]
    fn test_session_status_default() {
        let status = SessionStatus::default();
        assert_eq!(status, SessionStatus::Initializing);
    }
}
