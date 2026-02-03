//! WebSocket connection handling for collaboration server.
//!
//! This module manages individual client connections, including
//! authentication, message handling, and connection lifecycle.

use super::message::{ServerMessage, UserInfo, WireOpId, WirePresenceState};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Unique connection identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u64);

impl ConnectionId {
    /// Generate a new unique connection ID.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Conn({})", self.0)
    }
}

/// State of a client connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection established, awaiting authentication.
    Connected,
    /// Authentication successful.
    Authenticated,
    /// Client has joined a document session.
    InDocument(String),
    /// Connection is closing.
    Closing,
}

/// Information about an authenticated user.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    /// User ID (from authentication).
    pub user_id: String,
    /// Display name.
    pub display_name: String,
    /// Assigned color for presence.
    pub color: String,
}

impl AuthenticatedUser {
    /// Convert to wire format UserInfo.
    pub fn to_user_info(&self, presence: Option<WirePresenceState>) -> UserInfo {
        UserInfo {
            user_id: self.user_id.clone(),
            display_name: self.display_name.clone(),
            color: self.color.clone(),
            presence,
        }
    }
}

/// A client connection to the collaboration server.
pub struct ClientConnection {
    /// Unique connection identifier.
    pub id: ConnectionId,
    /// Current connection state.
    pub state: ConnectionState,
    /// Authenticated user information (if authenticated).
    pub user: Option<AuthenticatedUser>,
    /// Current presence state.
    pub presence: Option<WirePresenceState>,
    /// Channel to send messages to this client.
    pub tx: mpsc::UnboundedSender<ServerMessage>,
    /// Document ID this client is currently in (if any).
    pub current_doc: Option<String>,
    /// Last acknowledged operation IDs.
    pub last_ack: Vec<WireOpId>,
}

impl ClientConnection {
    /// Create a new client connection.
    pub fn new(tx: mpsc::UnboundedSender<ServerMessage>) -> Self {
        Self {
            id: ConnectionId::new(),
            state: ConnectionState::Connected,
            user: None,
            presence: None,
            tx,
            current_doc: None,
            last_ack: Vec::new(),
        }
    }

    /// Check if the connection is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    /// Check if the connection is in a document.
    pub fn is_in_document(&self) -> bool {
        matches!(self.state, ConnectionState::InDocument(_))
    }

    /// Get the user ID if authenticated.
    pub fn user_id(&self) -> Option<&str> {
        self.user.as_ref().map(|u| u.user_id.as_str())
    }

    /// Get the current document ID if in a document.
    pub fn doc_id(&self) -> Option<&str> {
        self.current_doc.as_deref()
    }

    /// Send a message to this client.
    pub fn send(&self, msg: ServerMessage) -> Result<(), SendError> {
        self.tx
            .send(msg)
            .map_err(|_| SendError::ChannelClosed(self.id))
    }

    /// Send an error message to this client.
    pub fn send_error(&self, code: impl Into<String>, message: impl Into<String>) -> Result<(), SendError> {
        self.send(ServerMessage::error(code, message))
    }

    /// Set the authenticated user.
    pub fn set_authenticated(&mut self, user: AuthenticatedUser) {
        self.user = Some(user);
        self.state = ConnectionState::Authenticated;
    }

    /// Join a document session.
    pub fn join_document(&mut self, doc_id: String) {
        self.current_doc = Some(doc_id.clone());
        self.state = ConnectionState::InDocument(doc_id);
    }

    /// Leave the current document session.
    pub fn leave_document(&mut self) {
        self.current_doc = None;
        if self.is_authenticated() {
            self.state = ConnectionState::Authenticated;
        } else {
            self.state = ConnectionState::Connected;
        }
        self.presence = None;
    }

    /// Update presence state.
    pub fn update_presence(&mut self, presence: WirePresenceState) {
        self.presence = Some(presence);
    }

    /// Convert to UserInfo for wire format.
    pub fn to_user_info(&self) -> Option<UserInfo> {
        self.user
            .as_ref()
            .map(|u| u.to_user_info(self.presence.clone()))
    }

    /// Mark connection as closing.
    pub fn close(&mut self) {
        self.state = ConnectionState::Closing;
    }
}

/// Error when sending a message fails.
#[derive(Debug, Clone)]
pub enum SendError {
    /// The channel to the client is closed.
    ChannelClosed(ConnectionId),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::ChannelClosed(id) => write!(f, "Channel closed for connection {}", id),
        }
    }
}

impl std::error::Error for SendError {}

/// Authentication provider trait.
///
/// Implement this trait to provide custom authentication logic.
#[trait_variant::make(Send)]
pub trait AuthProvider: Send + Sync {
    /// Authenticate a client with the given token.
    ///
    /// Returns the authenticated user information on success,
    /// or an error message on failure.
    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, String>;
}

/// Simple in-memory authentication provider for testing.
#[derive(Debug, Default)]
pub struct SimpleAuthProvider {
    /// Map of tokens to user info.
    users: std::collections::HashMap<String, (String, String)>,
}

impl SimpleAuthProvider {
    /// Create a new simple auth provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a user (token -> (user_id, display_name)).
    pub fn add_user(&mut self, token: String, user_id: String, display_name: String) {
        self.users.insert(token, (user_id, display_name));
    }

    /// Create a provider that accepts any token.
    pub fn accept_all() -> AcceptAllAuthProvider {
        AcceptAllAuthProvider
    }
}

impl AuthProvider for SimpleAuthProvider {
    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, String> {
        self.users
            .get(token)
            .map(|(user_id, display_name)| AuthenticatedUser {
                user_id: user_id.clone(),
                display_name: display_name.clone(),
                color: assign_color(user_id),
            })
            .ok_or_else(|| "Invalid token".to_string())
    }
}

/// Auth provider that accepts any token (for development/testing).
#[derive(Debug, Default)]
pub struct AcceptAllAuthProvider;

impl AuthProvider for AcceptAllAuthProvider {
    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, String> {
        // Use token as user_id for simplicity
        let user_id = if token.is_empty() {
            format!("user-{}", ConnectionId::new().0)
        } else {
            token.to_string()
        };

        Ok(AuthenticatedUser {
            user_id: user_id.clone(),
            display_name: format!("User {}", user_id),
            color: assign_color(&user_id),
        })
    }
}

/// Assign a color based on user ID (deterministic).
fn assign_color(user_id: &str) -> String {
    let colors = [
        "#E91E63", "#9C27B0", "#3F51B5", "#2196F3",
        "#00BCD4", "#4CAF50", "#FF9800", "#795548",
    ];

    // Simple hash based on user_id
    let hash: usize = user_id.bytes().map(|b| b as usize).sum();
    colors[hash % colors.len()].to_string()
}

/// Connection manager handles all active connections.
#[derive(Default)]
pub struct ConnectionManager {
    /// All active connections by ID.
    connections: std::collections::HashMap<ConnectionId, Arc<tokio::sync::RwLock<ClientConnection>>>,
    /// Connections grouped by document.
    documents: std::collections::HashMap<String, Vec<ConnectionId>>,
    /// User ID to connection ID mapping.
    user_connections: std::collections::HashMap<String, ConnectionId>,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new connection.
    pub fn add(&mut self, conn: ClientConnection) -> Arc<tokio::sync::RwLock<ClientConnection>> {
        let id = conn.id;
        let conn = Arc::new(tokio::sync::RwLock::new(conn));
        self.connections.insert(id, Arc::clone(&conn));
        conn
    }

    /// Remove a connection.
    pub async fn remove(&mut self, id: ConnectionId) -> Option<ClientConnection> {
        if let Some(conn) = self.connections.remove(&id) {
            let conn = conn.write().await;

            // Remove from user mapping
            if let Some(user_id) = conn.user_id() {
                self.user_connections.remove(user_id);
            }

            // Remove from document
            if let Some(doc_id) = conn.doc_id() {
                if let Some(doc_conns) = self.documents.get_mut(doc_id) {
                    doc_conns.retain(|c| *c != id);
                    if doc_conns.is_empty() {
                        self.documents.remove(doc_id);
                    }
                }
            }

            // We can't return the inner value because we have a write guard,
            // so we return None here. The caller should handle this.
            None
        } else {
            None
        }
    }

    /// Get a connection by ID.
    pub fn get(&self, id: ConnectionId) -> Option<Arc<tokio::sync::RwLock<ClientConnection>>> {
        self.connections.get(&id).cloned()
    }

    /// Register a user connection.
    pub fn register_user(&mut self, user_id: String, conn_id: ConnectionId) {
        self.user_connections.insert(user_id, conn_id);
    }

    /// Join a document session.
    pub fn join_document(&mut self, doc_id: &str, conn_id: ConnectionId) {
        self.documents
            .entry(doc_id.to_string())
            .or_default()
            .push(conn_id);
    }

    /// Leave a document session.
    pub fn leave_document(&mut self, doc_id: &str, conn_id: ConnectionId) {
        if let Some(doc_conns) = self.documents.get_mut(doc_id) {
            doc_conns.retain(|c| *c != conn_id);
            if doc_conns.is_empty() {
                self.documents.remove(doc_id);
            }
        }
    }

    /// Get all connections in a document.
    pub fn document_connections(&self, doc_id: &str) -> Vec<Arc<tokio::sync::RwLock<ClientConnection>>> {
        self.documents
            .get(doc_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.connections.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get connection count.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get document count.
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Get users in a document.
    pub fn document_user_count(&self, doc_id: &str) -> usize {
        self.documents
            .get(doc_id)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id_uniqueness() {
        let id1 = ConnectionId::new();
        let id2 = ConnectionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_connection_state_transitions() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut conn = ClientConnection::new(tx);

        assert_eq!(conn.state, ConnectionState::Connected);
        assert!(!conn.is_authenticated());

        conn.set_authenticated(AuthenticatedUser {
            user_id: "user-1".to_string(),
            display_name: "Alice".to_string(),
            color: "#E91E63".to_string(),
        });

        assert_eq!(conn.state, ConnectionState::Authenticated);
        assert!(conn.is_authenticated());

        conn.join_document("doc-1".to_string());
        assert!(conn.is_in_document());
        assert_eq!(conn.doc_id(), Some("doc-1"));

        conn.leave_document();
        assert!(!conn.is_in_document());
        assert_eq!(conn.state, ConnectionState::Authenticated);
    }

    #[test]
    fn test_color_assignment_deterministic() {
        let color1 = assign_color("user-123");
        let color2 = assign_color("user-123");
        assert_eq!(color1, color2);
    }

    #[tokio::test]
    async fn test_accept_all_auth_provider() {
        let provider = AcceptAllAuthProvider;
        let result = provider.authenticate("test-token").await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.user_id, "test-token");
    }

    #[tokio::test]
    async fn test_simple_auth_provider() {
        let mut provider = SimpleAuthProvider::new();
        provider.add_user(
            "secret-token".to_string(),
            "user-1".to_string(),
            "Alice".to_string(),
        );

        let result = provider.authenticate("secret-token").await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.user_id, "user-1");
        assert_eq!(user.display_name, "Alice");

        let invalid = provider.authenticate("wrong-token").await;
        assert!(invalid.is_err());
    }
}
