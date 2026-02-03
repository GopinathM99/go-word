//! Operation routing for the collaboration server.
//!
//! This module provides the `OperationRouter` struct that manages multiple
//! document sessions, routes incoming operations to the correct session,
//! and handles document lifecycle (creation, loading, cleanup).

use crate::bridge::CollaborativeDocument;
use crate::clock::VectorClock;
use crate::op_id::ClientId as CrdtClientId;
use crate::operation::CrdtOp;
use crate::permissions::{DocId, PermissionLevel, PermissionManager, UserId};
use crate::server::client::{ClientConnection, ClientError, ClientId, OutgoingMessage};
use crate::server::session::{DocumentSession, SessionConfig, SessionError, SessionStats, SessionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Configuration for the operation router.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Default session configuration for new sessions.
    pub default_session_config: SessionConfig,
    /// Maximum number of concurrent sessions.
    pub max_sessions: usize,
    /// Empty session timeout in milliseconds (before cleanup).
    pub empty_session_timeout_ms: u64,
    /// Cleanup interval in milliseconds.
    pub cleanup_interval_ms: u64,
    /// Whether to auto-create sessions for new documents.
    pub auto_create_sessions: bool,
    /// Starting CRDT client ID for server-side operations.
    pub server_client_id_start: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_session_config: SessionConfig::default(),
            max_sessions: 1000,
            empty_session_timeout_ms: 300_000, // 5 minutes
            cleanup_interval_ms: 60_000,       // 1 minute
            auto_create_sessions: true,
            server_client_id_start: 1_000_000,
        }
    }
}

/// Errors that can occur with the operation router.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RouterError {
    /// Session not found.
    #[error("Session not found for document: {0}")]
    SessionNotFound(DocId),

    /// Maximum sessions reached.
    #[error("Maximum sessions reached: {0}")]
    MaxSessionsReached(usize),

    /// Session already exists.
    #[error("Session already exists for document: {0}")]
    SessionAlreadyExists(DocId),

    /// Session error.
    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Document loading error.
    #[error("Failed to load document: {0}")]
    LoadError(String),

    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Client error.
    #[error("Client error: {0}")]
    ClientError(#[from] ClientError),
}

/// Incoming message from a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IncomingMessage {
    /// Operations to apply to the document.
    Operations {
        document_id: String,
        ops: Vec<CrdtOp>,
    },
    /// Presence update.
    PresenceUpdate {
        document_id: String,
        presence: crate::presence::PresenceState,
    },
    /// Join a document session.
    Join { document_id: String },
    /// Leave a document session.
    Leave { document_id: String },
    /// Request document snapshot.
    RequestSnapshot { document_id: String },
    /// Ping for keepalive.
    Ping { timestamp: u64 },
    /// Pong response.
    Pong { timestamp: u64 },
    /// Request sync from a vector clock.
    SyncRequest {
        document_id: String,
        since_clock: VectorClock,
    },
}

/// Result of routing an incoming message.
#[derive(Clone, Debug)]
pub enum RouteResult {
    /// Message was handled successfully.
    Success,
    /// Operations were applied, returns count.
    OperationsApplied(usize),
    /// Snapshot was sent.
    SnapshotSent,
    /// Sync operations were sent, returns count.
    SyncSent(usize),
    /// Client joined session.
    Joined,
    /// Client left session.
    Left,
    /// Pong response sent.
    PongSent,
    /// Error occurred.
    Error(String),
}

/// Document loader trait for loading documents from storage.
pub trait DocumentLoader: Send + Sync {
    /// Load a document by ID.
    fn load(&self, doc_id: &DocId) -> Result<Option<CollaborativeDocument>, RouterError>;

    /// Save a document.
    fn save(&self, doc_id: &DocId, document: &CollaborativeDocument) -> Result<(), RouterError>;

    /// Check if a document exists.
    fn exists(&self, doc_id: &DocId) -> bool;
}

/// In-memory document loader (for testing).
#[derive(Default)]
pub struct InMemoryDocumentLoader {
    documents: RwLock<HashMap<DocId, CollaborativeDocument>>,
}

impl InMemoryDocumentLoader {
    /// Create a new in-memory document loader.
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Store a document.
    pub async fn store(&self, doc_id: DocId, document: CollaborativeDocument) {
        let mut docs = self.documents.write().await;
        docs.insert(doc_id, document);
    }
}

impl DocumentLoader for InMemoryDocumentLoader {
    fn load(&self, doc_id: &DocId) -> Result<Option<CollaborativeDocument>, RouterError> {
        // Note: This is a sync implementation for simplicity
        // In production, you'd want an async version
        Ok(None) // For now, always return None (create new)
    }

    fn save(&self, _doc_id: &DocId, _document: &CollaborativeDocument) -> Result<(), RouterError> {
        // In production, implement actual persistence
        Ok(())
    }

    fn exists(&self, _doc_id: &DocId) -> bool {
        false
    }
}

/// The operation router manages multiple document sessions.
///
/// It is responsible for:
/// - Creating and managing document sessions
/// - Routing incoming operations to the correct session
/// - Handling document loading and saving
/// - Cleaning up empty sessions
pub struct OperationRouter {
    /// Active document sessions.
    sessions: HashMap<DocId, DocumentSession>,
    /// Router configuration.
    config: RouterConfig,
    /// Permission manager for access control.
    permission_manager: PermissionManager,
    /// Document loader for persistence.
    document_loader: Arc<dyn DocumentLoader>,
    /// Counter for generating CRDT client IDs.
    next_crdt_client_id: AtomicU64,
    /// Router creation timestamp.
    created_at: u64,
    /// Last cleanup timestamp.
    last_cleanup: u64,
    /// Total operations routed.
    total_operations: AtomicU64,
    /// Total sessions created.
    total_sessions_created: AtomicU64,
}

impl OperationRouter {
    /// Create a new operation router.
    ///
    /// # Arguments
    ///
    /// * `config` - Router configuration.
    /// * `permission_manager` - Permission manager for access control.
    /// * `document_loader` - Document loader for persistence.
    ///
    /// # Returns
    ///
    /// A new `OperationRouter` instance.
    pub fn new(
        config: RouterConfig,
        permission_manager: PermissionManager,
        document_loader: Arc<dyn DocumentLoader>,
    ) -> Self {
        let now = current_timestamp_ms();
        Self {
            sessions: HashMap::new(),
            config,
            permission_manager,
            document_loader,
            next_crdt_client_id: AtomicU64::new(1_000_000),
            created_at: now,
            last_cleanup: now,
            total_operations: AtomicU64::new(0),
            total_sessions_created: AtomicU64::new(0),
        }
    }

    /// Create a new operation router with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(
            RouterConfig::default(),
            PermissionManager::new(),
            Arc::new(InMemoryDocumentLoader::new()),
        )
    }

    /// Get the router configuration.
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// Get a reference to the permission manager.
    pub fn permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }

    /// Get a mutable reference to the permission manager.
    pub fn permission_manager_mut(&mut self) -> &mut PermissionManager {
        &mut self.permission_manager
    }

    /// Generate a new CRDT client ID.
    fn generate_crdt_client_id(&self) -> CrdtClientId {
        let id = self.next_crdt_client_id.fetch_add(1, Ordering::SeqCst);
        CrdtClientId::new(id)
    }

    // ========== Session Management ==========

    /// Get or create a session for a document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    ///
    /// # Returns
    ///
    /// `Ok(&mut DocumentSession)` if successful, or an error.
    pub fn get_or_create_session(&mut self, doc_id: &DocId) -> Result<&mut DocumentSession, RouterError> {
        if !self.sessions.contains_key(doc_id) {
            if !self.config.auto_create_sessions {
                return Err(RouterError::SessionNotFound(doc_id.clone()));
            }
            self.create_session(doc_id.clone())?;
        }
        Ok(self.sessions.get_mut(doc_id).unwrap())
    }

    /// Create a new session for a document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn create_session(&mut self, doc_id: DocId) -> Result<(), RouterError> {
        // Check if session already exists
        if self.sessions.contains_key(&doc_id) {
            return Err(RouterError::SessionAlreadyExists(doc_id));
        }

        // Check max sessions
        if self.sessions.len() >= self.config.max_sessions {
            return Err(RouterError::MaxSessionsReached(self.config.max_sessions));
        }

        // Try to load existing document
        let document = match self.document_loader.load(&doc_id)? {
            Some(doc) => doc,
            None => {
                // Create new empty document
                let crdt_client_id = self.generate_crdt_client_id();
                CollaborativeDocument::new(crdt_client_id)
            }
        };

        let session = DocumentSession::new(
            doc_id.clone(),
            document,
            self.config.default_session_config.clone(),
        );

        self.sessions.insert(doc_id, session);
        self.total_sessions_created.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    /// Create a session with an existing document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    /// * `document` - The collaborative document.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn create_session_with_document(
        &mut self,
        doc_id: DocId,
        document: CollaborativeDocument,
    ) -> Result<(), RouterError> {
        if self.sessions.contains_key(&doc_id) {
            return Err(RouterError::SessionAlreadyExists(doc_id));
        }

        if self.sessions.len() >= self.config.max_sessions {
            return Err(RouterError::MaxSessionsReached(self.config.max_sessions));
        }

        let session = DocumentSession::new(
            doc_id.clone(),
            document,
            self.config.default_session_config.clone(),
        );

        self.sessions.insert(doc_id, session);
        self.total_sessions_created.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    /// Get a reference to a session.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    ///
    /// # Returns
    ///
    /// `Some(&DocumentSession)` if found, `None` otherwise.
    pub fn get_session(&self, doc_id: &DocId) -> Option<&DocumentSession> {
        self.sessions.get(doc_id)
    }

    /// Get a mutable reference to a session.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    ///
    /// # Returns
    ///
    /// `Some(&mut DocumentSession)` if found, `None` otherwise.
    pub fn get_session_mut(&mut self, doc_id: &DocId) -> Option<&mut DocumentSession> {
        self.sessions.get_mut(doc_id)
    }

    /// Check if a session exists.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    ///
    /// # Returns
    ///
    /// `true` if the session exists, `false` otherwise.
    pub fn has_session(&self, doc_id: &DocId) -> bool {
        self.sessions.contains_key(doc_id)
    }

    /// Close and remove a session.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    ///
    /// # Returns
    ///
    /// `Ok(DocumentSession)` if the session was removed, or an error.
    pub fn close_session(&mut self, doc_id: &DocId) -> Result<DocumentSession, RouterError> {
        let mut session = self.sessions.remove(doc_id)
            .ok_or_else(|| RouterError::SessionNotFound(doc_id.clone()))?;

        // Save document before closing
        let _ = self.document_loader.save(doc_id, session.document());

        session.close();
        Ok(session)
    }

    /// Get all session IDs.
    pub fn session_ids(&self) -> Vec<DocId> {
        self.sessions.keys().cloned().collect()
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    // ========== Client Management ==========

    /// Join a client to a document session.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    /// * `client` - The client connection.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error.
    pub fn join_client(
        &mut self,
        doc_id: &DocId,
        client: ClientConnection,
    ) -> Result<(), RouterError> {
        // Check permission
        let user_id = client.user_id();
        let permission = self.permission_manager.get_level(user_id, doc_id);
        if !permission.can_view() {
            return Err(RouterError::PermissionDenied(
                format!("User {} does not have view permission for document {}", user_id, doc_id)
            ));
        }

        let session = self.get_or_create_session(doc_id)?;
        session.join_session(client)?;

        Ok(())
    }

    /// Remove a client from a document session.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    /// * `client_id` - The client ID.
    ///
    /// # Returns
    ///
    /// `Ok(ClientConnection)` if successful, or an error.
    pub fn leave_client(
        &mut self,
        doc_id: &DocId,
        client_id: &ClientId,
    ) -> Result<ClientConnection, RouterError> {
        let session = self.sessions.get_mut(doc_id)
            .ok_or_else(|| RouterError::SessionNotFound(doc_id.clone()))?;

        Ok(session.leave_session(client_id)?)
    }

    // ========== Operation Routing ==========

    /// Route operations to the correct session.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID.
    /// * `client_id` - The client ID that sent the operations.
    /// * `ops` - The operations to route.
    ///
    /// # Returns
    ///
    /// `Ok(applied_count)` with the number of operations applied, or an error.
    pub fn route_operations(
        &mut self,
        doc_id: &DocId,
        client_id: &ClientId,
        ops: Vec<CrdtOp>,
    ) -> Result<usize, RouterError> {
        let session = self.sessions.get_mut(doc_id)
            .ok_or_else(|| RouterError::SessionNotFound(doc_id.clone()))?;

        // Check that client has edit permission
        let client = session.get_client(client_id)
            .ok_or_else(|| RouterError::SessionError(SessionError::ClientNotFound(client_id.clone())))?;

        if !client.can_edit() {
            return Err(RouterError::PermissionDenied(
                "Client does not have edit permission".to_string()
            ));
        }

        let applied = session.apply_operations(client_id, ops)?;
        self.total_operations.fetch_add(applied as u64, Ordering::SeqCst);

        Ok(applied)
    }

    /// Route an incoming message.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID.
    /// * `message` - The incoming message.
    ///
    /// # Returns
    ///
    /// A `RouteResult` indicating the outcome.
    pub fn route_message(
        &mut self,
        client_id: &ClientId,
        message: IncomingMessage,
    ) -> RouteResult {
        match message {
            IncomingMessage::Operations { document_id, ops } => {
                let doc_id = DocId::from(document_id);
                match self.route_operations(&doc_id, client_id, ops) {
                    Ok(count) => RouteResult::OperationsApplied(count),
                    Err(e) => RouteResult::Error(e.to_string()),
                }
            }

            IncomingMessage::PresenceUpdate { document_id, presence } => {
                let doc_id = DocId::from(document_id);
                match self.sessions.get_mut(&doc_id) {
                    Some(session) => {
                        match session.update_presence(client_id, presence) {
                            Ok(()) => RouteResult::Success,
                            Err(e) => RouteResult::Error(e.to_string()),
                        }
                    }
                    None => RouteResult::Error(format!("Session not found: {}", doc_id)),
                }
            }

            IncomingMessage::Join { document_id } => {
                // Note: This assumes the client is already created elsewhere
                // In a real implementation, you'd create the client here
                RouteResult::Joined
            }

            IncomingMessage::Leave { document_id } => {
                let doc_id = DocId::from(document_id);
                match self.leave_client(&doc_id, client_id) {
                    Ok(_) => RouteResult::Left,
                    Err(e) => RouteResult::Error(e.to_string()),
                }
            }

            IncomingMessage::RequestSnapshot { document_id } => {
                let doc_id = DocId::from(document_id);
                match self.sessions.get_mut(&doc_id) {
                    Some(session) => {
                        match session.create_snapshot(client_id) {
                            Ok(snapshot) => {
                                match session.send_to_client(client_id, snapshot) {
                                    Ok(()) => RouteResult::SnapshotSent,
                                    Err(e) => RouteResult::Error(e.to_string()),
                                }
                            }
                            Err(e) => RouteResult::Error(e.to_string()),
                        }
                    }
                    None => RouteResult::Error(format!("Session not found: {}", doc_id)),
                }
            }

            IncomingMessage::Ping { timestamp } => {
                // Find which session this client is in and respond
                for session in self.sessions.values_mut() {
                    if let Some(client) = session.get_client_mut(client_id) {
                        let _ = client.send(OutgoingMessage::Pong { timestamp });
                        return RouteResult::PongSent;
                    }
                }
                RouteResult::Error("Client not found in any session".to_string())
            }

            IncomingMessage::Pong { .. } => {
                // Just acknowledge receipt
                RouteResult::Success
            }

            IncomingMessage::SyncRequest { document_id, since_clock } => {
                let doc_id = DocId::from(document_id);
                match self.sessions.get_mut(&doc_id) {
                    Some(session) => {
                        let ops: Vec<CrdtOp> = session.ops_since(&since_clock)
                            .into_iter()
                            .cloned()
                            .collect();
                        let count = ops.len();

                        if let Err(e) = session.send_to_client(
                            client_id,
                            OutgoingMessage::Operations(ops),
                        ) {
                            return RouteResult::Error(e.to_string());
                        }

                        RouteResult::SyncSent(count)
                    }
                    None => RouteResult::Error(format!("Session not found: {}", doc_id)),
                }
            }
        }
    }

    // ========== Cleanup ==========

    /// Clean up empty and stale sessions.
    ///
    /// # Returns
    ///
    /// A vector of document IDs that were cleaned up.
    pub fn cleanup_sessions(&mut self) -> Vec<DocId> {
        let now = current_timestamp_ms();
        self.last_cleanup = now;

        let empty_timeout = self.config.empty_session_timeout_ms;

        // Find sessions to clean up
        let to_cleanup: Vec<DocId> = self.sessions
            .iter()
            .filter(|(_, session)| {
                session.should_close(empty_timeout) || session.status() == SessionStatus::Closed
            })
            .map(|(id, _)| id.clone())
            .collect();

        // Close and remove sessions
        for doc_id in &to_cleanup {
            let _ = self.close_session(doc_id);
        }

        to_cleanup
    }

    /// Clean up idle clients across all sessions.
    ///
    /// # Returns
    ///
    /// A vector of (doc_id, client_id) pairs that were cleaned up.
    pub fn cleanup_idle_clients(&mut self) -> Vec<(DocId, ClientId)> {
        let mut cleaned = Vec::new();

        for (doc_id, session) in &mut self.sessions {
            let idle_clients = session.cleanup_idle_clients();
            for client_id in idle_clients {
                cleaned.push((doc_id.clone(), client_id));
            }
        }

        cleaned
    }

    // ========== Statistics ==========

    /// Get statistics for all sessions.
    pub fn session_stats(&self) -> Vec<SessionStats> {
        self.sessions.values().map(|s| s.stats()).collect()
    }

    /// Get router statistics.
    pub fn stats(&self) -> RouterStats {
        RouterStats {
            session_count: self.sessions.len(),
            total_clients: self.sessions.values().map(|s| s.client_count()).sum(),
            total_operations: self.total_operations.load(Ordering::SeqCst),
            total_sessions_created: self.total_sessions_created.load(Ordering::SeqCst),
            created_at: self.created_at,
            last_cleanup: self.last_cleanup,
        }
    }
}

/// Statistics about the operation router.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouterStats {
    /// Number of active sessions.
    pub session_count: usize,
    /// Total clients across all sessions.
    pub total_clients: usize,
    /// Total operations routed.
    pub total_operations: u64,
    /// Total sessions created.
    pub total_sessions_created: u64,
    /// Router creation timestamp.
    pub created_at: u64,
    /// Last cleanup timestamp.
    pub last_cleanup: u64,
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

    fn create_test_router() -> OperationRouter {
        let mut router = OperationRouter::with_defaults();
        // Grant owner permission to test users
        router.permission_manager.grant_owner(DocId::from("test-doc"), UserId::from("user-1"));
        router
    }

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert_eq!(config.max_sessions, 1000);
        assert!(config.auto_create_sessions);
    }

    #[test]
    fn test_router_new() {
        let router = OperationRouter::with_defaults();
        assert_eq!(router.session_count(), 0);
        assert!(router.session_ids().is_empty());
    }

    #[test]
    fn test_create_session() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("test-doc");

        let result = router.create_session(doc_id.clone());
        assert!(result.is_ok());
        assert!(router.has_session(&doc_id));
        assert_eq!(router.session_count(), 1);
    }

    #[test]
    fn test_create_session_duplicate() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("test-doc");

        router.create_session(doc_id.clone()).unwrap();
        let result = router.create_session(doc_id.clone());

        assert!(matches!(result, Err(RouterError::SessionAlreadyExists(_))));
    }

    #[test]
    fn test_create_session_max_reached() {
        let config = RouterConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let mut router = OperationRouter::new(
            config,
            PermissionManager::new(),
            Arc::new(InMemoryDocumentLoader::new()),
        );

        router.create_session(DocId::from("doc-1")).unwrap();
        router.create_session(DocId::from("doc-2")).unwrap();
        let result = router.create_session(DocId::from("doc-3"));

        assert!(matches!(result, Err(RouterError::MaxSessionsReached(2))));
    }

    #[test]
    fn test_get_or_create_session() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("test-doc");

        // First call creates
        let result = router.get_or_create_session(&doc_id);
        assert!(result.is_ok());
        assert_eq!(router.session_count(), 1);

        // Second call gets existing
        let result = router.get_or_create_session(&doc_id);
        assert!(result.is_ok());
        assert_eq!(router.session_count(), 1); // Still 1
    }

    #[test]
    fn test_get_or_create_session_disabled() {
        let config = RouterConfig {
            auto_create_sessions: false,
            ..Default::default()
        };
        let mut router = OperationRouter::new(
            config,
            PermissionManager::new(),
            Arc::new(InMemoryDocumentLoader::new()),
        );

        let doc_id = DocId::from("test-doc");
        let result = router.get_or_create_session(&doc_id);

        assert!(matches!(result, Err(RouterError::SessionNotFound(_))));
    }

    #[test]
    fn test_close_session() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("test-doc");

        router.create_session(doc_id.clone()).unwrap();
        assert!(router.has_session(&doc_id));

        let result = router.close_session(&doc_id);
        assert!(result.is_ok());
        assert!(!router.has_session(&doc_id));
    }

    #[test]
    fn test_close_session_not_found() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("nonexistent");

        let result = router.close_session(&doc_id);
        assert!(matches!(result, Err(RouterError::SessionNotFound(_))));
    }

    #[test]
    fn test_join_client() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, _) = create_test_client("client-1", "user-1");

        let result = router.join_client(&doc_id, client);
        assert!(result.is_ok());

        let session = router.get_session(&doc_id).unwrap();
        assert_eq!(session.client_count(), 1);
    }

    #[test]
    fn test_join_client_no_permission() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("test-doc");

        // Don't grant any permissions
        let (client, _) = create_test_client("client-1", "user-1");

        let result = router.join_client(&doc_id, client);
        assert!(matches!(result, Err(RouterError::PermissionDenied(_))));
    }

    #[test]
    fn test_leave_client() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, _) = create_test_client("client-1", "user-1");

        router.join_client(&doc_id, client).unwrap();
        assert_eq!(router.get_session(&doc_id).unwrap().client_count(), 1);

        let result = router.leave_client(&doc_id, &ClientId::new("client-1"));
        assert!(result.is_ok());
        assert_eq!(router.get_session(&doc_id).unwrap().client_count(), 0);
    }

    #[test]
    fn test_route_operations() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, _) = create_test_client("client-1", "user-1");

        router.join_client(&doc_id, client).unwrap();

        // Route empty operations (should work)
        let result = router.route_operations(
            &doc_id,
            &ClientId::new("client-1"),
            vec![],
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_route_operations_no_permission() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");

        // Create a viewer client
        let (sender, _) = create_test_sender();
        let client = ClientConnection::new(
            ClientId::new("client-1"),
            UserId::from("user-1"),
            "User 1".to_string(),
            sender,
            PermissionLevel::Viewer, // Viewer, not Editor
            "#E91E63".to_string(),
        );

        router.join_client(&doc_id, client).unwrap();

        let result = router.route_operations(
            &doc_id,
            &ClientId::new("client-1"),
            vec![],
        );

        assert!(matches!(result, Err(RouterError::PermissionDenied(_))));
    }

    #[test]
    fn test_route_message_ping() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, mut receiver) = create_test_client("client-1", "user-1");

        router.join_client(&doc_id, client).unwrap();

        // Clear any join notifications
        while receiver.try_recv().is_ok() {}

        let result = router.route_message(
            &ClientId::new("client-1"),
            IncomingMessage::Ping { timestamp: 12345 },
        );

        assert!(matches!(result, RouteResult::PongSent));

        // Check pong was sent
        let msg = receiver.try_recv().unwrap();
        assert!(matches!(msg, OutgoingMessage::Pong { timestamp: 12345 }));
    }

    #[test]
    fn test_route_message_leave() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, _) = create_test_client("client-1", "user-1");

        router.join_client(&doc_id, client).unwrap();
        assert_eq!(router.get_session(&doc_id).unwrap().client_count(), 1);

        let result = router.route_message(
            &ClientId::new("client-1"),
            IncomingMessage::Leave { document_id: "test-doc".to_string() },
        );

        assert!(matches!(result, RouteResult::Left));
        assert_eq!(router.get_session(&doc_id).unwrap().client_count(), 0);
    }

    #[test]
    fn test_cleanup_sessions() {
        let config = RouterConfig {
            empty_session_timeout_ms: 1,
            ..Default::default()
        };
        let mut router = OperationRouter::new(
            config,
            PermissionManager::new(),
            Arc::new(InMemoryDocumentLoader::new()),
        );

        // Create an empty session
        router.create_session(DocId::from("test-doc")).unwrap();

        // Force the session to be old
        if let Some(session) = router.get_session_mut(&DocId::from("test-doc")) {
            session.set_metadata(Some(serde_json::json!({"test": true})));
        }

        // Wait a bit to ensure timeout
        std::thread::sleep(std::time::Duration::from_millis(10));

        let cleaned = router.cleanup_sessions();
        assert_eq!(cleaned.len(), 1);
        assert!(!router.has_session(&DocId::from("test-doc")));
    }

    #[test]
    fn test_session_stats() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, _) = create_test_client("client-1", "user-1");

        router.join_client(&doc_id, client).unwrap();

        let stats = router.session_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].document_id.0, "test-doc");
        assert_eq!(stats[0].client_count, 1);
    }

    #[test]
    fn test_router_stats() {
        let mut router = create_test_router();
        let doc_id = DocId::from("test-doc");
        let (client, _) = create_test_client("client-1", "user-1");

        router.join_client(&doc_id, client).unwrap();

        let stats = router.stats();
        assert_eq!(stats.session_count, 1);
        assert_eq!(stats.total_clients, 1);
        assert_eq!(stats.total_sessions_created, 1);
    }

    #[test]
    fn test_create_session_with_document() {
        let mut router = OperationRouter::with_defaults();
        let doc_id = DocId::from("test-doc");
        let document = CollaborativeDocument::new(CrdtClientId::new(1));

        let result = router.create_session_with_document(doc_id.clone(), document);
        assert!(result.is_ok());
        assert!(router.has_session(&doc_id));
    }

    #[test]
    fn test_route_message_session_not_found() {
        let mut router = OperationRouter::with_defaults();

        let result = router.route_message(
            &ClientId::new("client-1"),
            IncomingMessage::Leave { document_id: "nonexistent".to_string() },
        );

        assert!(matches!(result, RouteResult::Error(_)));
    }

    #[test]
    fn test_multiple_sessions() {
        let mut router = OperationRouter::with_defaults();
        router.permission_manager.grant_owner(DocId::from("doc-1"), UserId::from("user-1"));
        router.permission_manager.grant_owner(DocId::from("doc-2"), UserId::from("user-2"));

        let (client1, _) = create_test_client("client-1", "user-1");
        let (client2, _) = create_test_client("client-2", "user-2");

        router.join_client(&DocId::from("doc-1"), client1).unwrap();
        router.join_client(&DocId::from("doc-2"), client2).unwrap();

        assert_eq!(router.session_count(), 2);

        let session_ids = router.session_ids();
        assert_eq!(session_ids.len(), 2);
    }
}
