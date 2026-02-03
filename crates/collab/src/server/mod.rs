//! WebSocket collaboration server.
//!
//! This module provides a WebSocket server for real-time collaborative
//! document editing. It handles client connections, message routing,
//! and operation broadcasting.
//!
//! # Architecture
//!
//! The server uses tokio-tungstenite for WebSocket connections and
//! follows an actor-like pattern where each connection runs in its
//! own task. A central `CollaborationServer` coordinates message
//! routing between clients.
//!
//! # Example
//!
//! ```ignore
//! use collab::server::{CollaborationServer, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ServerConfig::default();
//!     let server = CollaborationServer::new(config);
//!
//!     // Run server on configured port
//!     server.run().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod connection;
pub mod file_store;
pub mod memory_store;
pub mod message;
pub mod router;
pub mod session;
pub mod storage;

use connection::{
    AcceptAllAuthProvider, AuthProvider, ClientConnection, ConnectionId, ConnectionManager,
};
use message::{ClientMessage, ServerMessage, WireCrdtOp, WireOpId, WireVectorClock};

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

// Re-export key types
pub use connection::{AuthenticatedUser, ConnectionState, SendError, SimpleAuthProvider};
pub use file_store::FileOperationStore;
pub use memory_store::MemoryOperationStore;
pub use message::{WirePresenceState, WirePosition, WireRange};
pub use storage::{
    OperationStore, Snapshot, StorageError, StorageResult, StorageStats, StoredOperation, Version,
};

// Re-export new session management types
pub use client::{
    ClientConnection as SessionClientConnection, ClientConnectionBuilder, ClientError as SessionClientError,
    ClientId as SessionClientId, ClientStats, ConnectionStatus, OutgoingMessage,
};
pub use router::{
    DocumentLoader, InMemoryDocumentLoader, IncomingMessage, OperationRouter, RouteResult,
    RouterConfig, RouterError, RouterStats,
};
pub use session::{
    DocumentSession as ManagedDocumentSession, SessionConfig, SessionError, SessionStats, SessionStatus,
};

/// Server configuration.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Address to bind to.
    pub bind_address: String,
    /// Port to listen on.
    pub port: u16,
    /// Maximum connections per document.
    pub max_connections_per_doc: usize,
    /// Maximum total connections.
    pub max_total_connections: usize,
    /// Ping interval in seconds.
    pub ping_interval_secs: u64,
    /// Connection timeout in seconds.
    pub connection_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            max_connections_per_doc: 100,
            max_total_connections: 1000,
            ping_interval_secs: 30,
            connection_timeout_secs: 60,
        }
    }
}

impl ServerConfig {
    /// Create a new configuration with the specified port.
    pub fn with_port(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }

    /// Get the full bind address.
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

/// Commands sent to the server from connection handlers.
#[derive(Debug)]
enum ServerCommand {
    /// Client authenticated.
    Authenticated {
        conn_id: ConnectionId,
        user_id: String,
    },
    /// Client joined a document.
    JoinDocument {
        conn_id: ConnectionId,
        doc_id: String,
    },
    /// Client left a document.
    LeaveDocument {
        conn_id: ConnectionId,
        doc_id: String,
    },
    /// Broadcast operations to document.
    BroadcastOps {
        conn_id: ConnectionId,
        doc_id: String,
        ops: Vec<WireCrdtOp>,
    },
    /// Broadcast presence update.
    BroadcastPresence {
        conn_id: ConnectionId,
        doc_id: String,
        user_id: String,
        presence: WirePresenceState,
    },
    /// Client disconnected.
    Disconnected { conn_id: ConnectionId },
    /// Sync request from client.
    SyncRequest {
        conn_id: ConnectionId,
        doc_id: String,
        since: WireVectorClock,
    },
}

/// Document session state.
#[derive(Default)]
struct DocumentSession {
    /// Operations stored for sync.
    ops: Vec<WireCrdtOp>,
    /// Current vector clock.
    clock: WireVectorClock,
}

impl DocumentSession {
    fn new() -> Self {
        Self::default()
    }

    /// Add operations to the session.
    fn add_ops(&mut self, ops: Vec<WireCrdtOp>) {
        // Update clock from operations
        for op in &ops {
            let current = self.clock.clocks.get(&op.id.client_id).copied().unwrap_or(0);
            if op.id.seq > current {
                self.clock.clocks.insert(op.id.client_id.clone(), op.id.seq);
            }
        }
        self.ops.extend(ops);
    }

    /// Get operations since a vector clock.
    fn ops_since(&self, since: &WireVectorClock) -> Vec<WireCrdtOp> {
        self.ops
            .iter()
            .filter(|op| {
                let since_seq = since.clocks.get(&op.id.client_id).copied().unwrap_or(0);
                op.id.seq > since_seq
            })
            .cloned()
            .collect()
    }
}

/// The main collaboration server.
pub struct CollaborationServer<A: AuthProvider = AcceptAllAuthProvider> {
    /// Server configuration.
    config: ServerConfig,
    /// Authentication provider.
    auth_provider: Arc<A>,
    /// Connection manager.
    connections: Arc<RwLock<ConnectionManager>>,
    /// Document sessions.
    documents: Arc<RwLock<std::collections::HashMap<String, DocumentSession>>>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
}

impl CollaborationServer<AcceptAllAuthProvider> {
    /// Create a new server with default authentication (accepts all).
    pub fn new(config: ServerConfig) -> Self {
        Self::with_auth(config, AcceptAllAuthProvider)
    }
}

impl<A: AuthProvider + 'static> CollaborationServer<A> {
    /// Create a new server with custom authentication.
    pub fn with_auth(config: ServerConfig, auth_provider: A) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            auth_provider: Arc::new(auth_provider),
            connections: Arc::new(RwLock::new(ConnectionManager::new())),
            documents: Arc::new(RwLock::new(std::collections::HashMap::new())),
            shutdown_tx,
        }
    }

    /// Get a shutdown handle.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            tx: self.shutdown_tx.clone(),
        }
    }

    /// Run the server.
    ///
    /// This will bind to the configured address and start accepting
    /// connections. Returns when shutdown is signaled or an error occurs.
    pub async fn run(&self) -> Result<(), ServerError> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::BindFailed(addr.clone(), e))?;

        tracing::info!("Collaboration server listening on {}", addr);

        // Create command channel
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<ServerCommand>();

        // Clone things for the command handler task
        let connections = Arc::clone(&self.connections);
        let documents = Arc::clone(&self.documents);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Spawn command handler task
        let cmd_handler = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                        Self::handle_command(&connections, &documents, cmd).await;
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Command handler received shutdown signal");
                        break;
                    }
                }
            }
        });

        // Accept connections
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            // Check connection limits
                            let conn_count = self.connections.read().await.connection_count();
                            if conn_count >= self.config.max_total_connections {
                                tracing::warn!("Max connections reached, rejecting {}", addr);
                                continue;
                            }

                            self.handle_connection(stream, addr, cmd_tx.clone()).await;
                        }
                        Err(e) => {
                            tracing::error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Server received shutdown signal");
                    break;
                }
            }
        }

        // Wait for command handler to finish
        let _ = cmd_handler.await;

        tracing::info!("Server shutdown complete");
        Ok(())
    }

    /// Handle a new connection.
    async fn handle_connection(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
        cmd_tx: mpsc::UnboundedSender<ServerCommand>,
    ) {
        tracing::debug!("New connection from {}", addr);

        // Upgrade to WebSocket
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                tracing::error!("WebSocket handshake failed for {}: {}", addr, e);
                return;
            }
        };

        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // Create message channel for this connection
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<ServerMessage>();

        // Create connection
        let conn = ClientConnection::new(msg_tx);
        let conn_id = conn.id;

        // Add to manager
        let conn = self.connections.write().await.add(conn);

        let auth_provider = Arc::clone(&self.auth_provider);
        let connections = Arc::clone(&self.connections);
        let _config = self.config.clone(); // Reserved for future use (timeouts, etc.)
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Spawn connection handler task
        tokio::spawn(async move {
            // Outgoing message forwarder
            let outgoing = tokio::spawn(async move {
                while let Some(msg) = msg_rx.recv().await {
                    match msg.to_json() {
                        Ok(json) => {
                            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to serialize message: {}", e);
                        }
                    }
                }
            });

            // Incoming message handler
            loop {
                tokio::select! {
                    msg = ws_rx.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Err(e) = Self::handle_message(
                                    &conn,
                                    &text,
                                    &auth_provider,
                                    &cmd_tx,
                                ).await {
                                    tracing::error!("Message handling error: {}", e);
                                    // Send error to client
                                    let conn_guard = conn.read().await;
                                    let _ = conn_guard.send_error("message_error", e.to_string());
                                }
                            }
                            Some(Ok(Message::Ping(_data))) => {
                                // Respond with pong (handled by tungstenite automatically in most cases)
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                tracing::debug!("Connection {} closed", conn_id);
                                break;
                            }
                            Some(Err(e)) => {
                                tracing::error!("WebSocket error for {}: {}", conn_id, e);
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::debug!("Connection {} received shutdown signal", conn_id);
                        break;
                    }
                }
            }

            // Cleanup
            outgoing.abort();

            // Notify server of disconnect
            let _ = cmd_tx.send(ServerCommand::Disconnected { conn_id });

            // Remove from manager
            connections.write().await.remove(conn_id).await;
        });
    }

    /// Handle an incoming message from a client.
    async fn handle_message(
        conn: &Arc<RwLock<ClientConnection>>,
        text: &str,
        auth_provider: &Arc<A>,
        cmd_tx: &mpsc::UnboundedSender<ServerCommand>,
    ) -> Result<(), MessageError> {
        let msg: ClientMessage =
            serde_json::from_str(text).map_err(|e| MessageError::ParseError(e.to_string()))?;

        let mut conn_guard = conn.write().await;
        let conn_id = conn_guard.id;

        match msg {
            ClientMessage::Auth { token } => {
                if conn_guard.is_authenticated() {
                    conn_guard.send_error("already_authenticated", "Already authenticated")?;
                    return Ok(());
                }

                match auth_provider.authenticate(&token).await {
                    Ok(user) => {
                        let user_id = user.user_id.clone();
                        let display_name = user.display_name.clone();
                        conn_guard.set_authenticated(user);

                        conn_guard.send(ServerMessage::AuthSuccess {
                            user_id: user_id.clone(),
                            display_name,
                        })?;

                        let _ = cmd_tx.send(ServerCommand::Authenticated { conn_id, user_id });
                    }
                    Err(error) => {
                        conn_guard.send(ServerMessage::AuthError { message: error })?;
                    }
                }
            }

            ClientMessage::Join { doc_id } => {
                if !conn_guard.is_authenticated() {
                    conn_guard.send_error("not_authenticated", "Must authenticate first")?;
                    return Ok(());
                }

                // Leave current document if any
                if let Some(old_doc) = conn_guard.doc_id() {
                    let _ = cmd_tx.send(ServerCommand::LeaveDocument {
                        conn_id,
                        doc_id: old_doc.to_string(),
                    });
                }

                conn_guard.join_document(doc_id.clone());

                let _ = cmd_tx.send(ServerCommand::JoinDocument {
                    conn_id,
                    doc_id,
                });
            }

            ClientMessage::Leave { doc_id } => {
                if conn_guard.doc_id() == Some(&doc_id) {
                    conn_guard.leave_document();
                    let _ = cmd_tx.send(ServerCommand::LeaveDocument { conn_id, doc_id });
                }
            }

            ClientMessage::Ops { ops } => {
                if let Some(doc_id) = conn_guard.doc_id() {
                    let doc_id = doc_id.to_string();

                    // Acknowledge operations
                    let op_ids: Vec<WireOpId> = ops.iter().map(|op| op.id.clone()).collect();
                    conn_guard.send(ServerMessage::Ack { op_ids })?;

                    // Broadcast to other clients
                    let _ = cmd_tx.send(ServerCommand::BroadcastOps {
                        conn_id,
                        doc_id,
                        ops,
                    });
                } else {
                    conn_guard.send_error("not_in_document", "Must join a document first")?;
                }
            }

            ClientMessage::Ack { op_ids } => {
                conn_guard.last_ack = op_ids;
            }

            ClientMessage::Presence { state } => {
                if let (Some(doc_id), Some(user_id)) =
                    (conn_guard.doc_id(), conn_guard.user_id())
                {
                    let doc_id = doc_id.to_string();
                    let user_id = user_id.to_string();

                    conn_guard.update_presence(state.clone());

                    let _ = cmd_tx.send(ServerCommand::BroadcastPresence {
                        conn_id,
                        doc_id,
                        user_id,
                        presence: state,
                    });
                }
            }

            ClientMessage::SyncRequest { since } => {
                if let Some(doc_id) = conn_guard.doc_id() {
                    let doc_id = doc_id.to_string();
                    let _ = cmd_tx.send(ServerCommand::SyncRequest {
                        conn_id,
                        doc_id,
                        since,
                    });
                } else {
                    conn_guard.send_error("not_in_document", "Must join a document first")?;
                }
            }

            ClientMessage::Ping => {
                conn_guard.send(ServerMessage::Pong)?;
            }
        }

        Ok(())
    }

    /// Handle a server command.
    async fn handle_command(
        connections: &Arc<RwLock<ConnectionManager>>,
        documents: &Arc<RwLock<std::collections::HashMap<String, DocumentSession>>>,
        cmd: ServerCommand,
    ) {
        match cmd {
            ServerCommand::Authenticated { conn_id, user_id } => {
                connections.write().await.register_user(user_id, conn_id);
            }

            ServerCommand::JoinDocument { conn_id, doc_id } => {
                // Ensure document session exists
                {
                    let mut docs = documents.write().await;
                    docs.entry(doc_id.clone()).or_insert_with(DocumentSession::new);
                }

                // Get current users in document
                let conns = connections.read().await;
                let doc_conns = conns.document_connections(&doc_id);

                let mut users = Vec::new();
                for other_conn in &doc_conns {
                    let other = other_conn.read().await;
                    if other.id != conn_id {
                        if let Some(user_info) = other.to_user_info() {
                            users.push(user_info);
                        }
                    }
                }

                // Get joining user's info
                let joining_user = if let Some(conn) = conns.get(conn_id) {
                    conn.read().await.to_user_info()
                } else {
                    None
                };

                drop(conns);

                // Add to document
                connections.write().await.join_document(&doc_id, conn_id);

                // Send joined message
                let conns = connections.read().await;
                if let Some(conn) = conns.get(conn_id) {
                    let conn_guard = conn.read().await;
                    let _ = conn_guard.send(ServerMessage::Joined {
                        doc_id: doc_id.clone(),
                        users,
                    });
                }

                // Notify other users
                if let Some(user) = joining_user {
                    for other_conn in conns.document_connections(&doc_id) {
                        let other = other_conn.read().await;
                        if other.id != conn_id {
                            let _ = other.send(ServerMessage::UserJoined { user: user.clone() });
                        }
                    }
                }
            }

            ServerCommand::LeaveDocument { conn_id, doc_id } => {
                let conns = connections.read().await;

                // Get leaving user's ID
                let user_id = if let Some(conn) = conns.get(conn_id) {
                    conn.read().await.user_id().map(|s| s.to_string())
                } else {
                    None
                };

                // Notify other users
                if let Some(user_id) = user_id {
                    for other_conn in conns.document_connections(&doc_id) {
                        let other = other_conn.read().await;
                        if other.id != conn_id {
                            let _ = other.send(ServerMessage::UserLeft {
                                user_id: user_id.clone(),
                            });
                        }
                    }
                }

                drop(conns);

                // Remove from document
                connections.write().await.leave_document(&doc_id, conn_id);
            }

            ServerCommand::BroadcastOps { conn_id, doc_id, ops } => {
                // Store operations in document session
                {
                    let mut docs = documents.write().await;
                    if let Some(session) = docs.get_mut(&doc_id) {
                        session.add_ops(ops.clone());
                    }
                }

                // Broadcast to other clients
                let conns = connections.read().await;
                for other_conn in conns.document_connections(&doc_id) {
                    let other = other_conn.read().await;
                    if other.id != conn_id {
                        let _ = other.send(ServerMessage::Ops { ops: ops.clone() });
                    }
                }
            }

            ServerCommand::BroadcastPresence {
                conn_id,
                doc_id,
                user_id,
                presence,
            } => {
                let conns = connections.read().await;
                for other_conn in conns.document_connections(&doc_id) {
                    let other = other_conn.read().await;
                    if other.id != conn_id {
                        let _ = other.send(ServerMessage::Presence {
                            user_id: user_id.clone(),
                            state: presence.clone(),
                        });
                    }
                }
            }

            ServerCommand::SyncRequest {
                conn_id,
                doc_id,
                since,
            } => {
                let docs = documents.read().await;
                let response = if let Some(session) = docs.get(&doc_id) {
                    ServerMessage::SyncResponse {
                        ops: session.ops_since(&since),
                        clock: session.clock.clone(),
                    }
                } else {
                    ServerMessage::SyncResponse {
                        ops: Vec::new(),
                        clock: WireVectorClock::default(),
                    }
                };

                let conns = connections.read().await;
                if let Some(conn) = conns.get(conn_id) {
                    let conn_guard = conn.read().await;
                    let _ = conn_guard.send(response);
                }
            }

            ServerCommand::Disconnected { conn_id } => {
                // This is handled in the connection cleanup
                tracing::debug!("Connection {} disconnected", conn_id);
            }
        }
    }

    /// Get current server statistics.
    pub async fn stats(&self) -> ServerStats {
        let conns = self.connections.read().await;
        let docs = self.documents.read().await;

        ServerStats {
            total_connections: conns.connection_count(),
            total_documents: docs.len(),
        }
    }
}

/// Server statistics.
#[derive(Clone, Debug)]
pub struct ServerStats {
    /// Total active connections.
    pub total_connections: usize,
    /// Total active document sessions.
    pub total_documents: usize,
}

/// Handle for triggering server shutdown.
#[derive(Clone)]
pub struct ShutdownHandle {
    tx: broadcast::Sender<()>,
}

impl ShutdownHandle {
    /// Signal the server to shut down.
    pub fn shutdown(&self) {
        let _ = self.tx.send(());
    }
}

/// Server errors.
#[derive(Debug)]
pub enum ServerError {
    /// Failed to bind to address.
    BindFailed(String, std::io::Error),
    /// WebSocket error.
    WebSocket(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::BindFailed(addr, e) => {
                write!(f, "Failed to bind to {}: {}", addr, e)
            }
            ServerError::WebSocket(e) => write!(f, "WebSocket error: {}", e),
        }
    }
}

impl std::error::Error for ServerError {}

/// Message handling errors.
#[derive(Debug)]
pub enum MessageError {
    /// Failed to parse message.
    ParseError(String),
    /// Failed to send message.
    SendError(SendError),
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::ParseError(e) => write!(f, "Parse error: {}", e),
            MessageError::SendError(e) => write!(f, "Send error: {}", e),
        }
    }
}

impl std::error::Error for MessageError {}

impl From<SendError> for MessageError {
    fn from(e: SendError) -> Self {
        MessageError::SendError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.socket_addr(), "0.0.0.0:8080");
    }

    #[test]
    fn test_server_config_with_port() {
        let config = ServerConfig::with_port(9000);
        assert_eq!(config.port, 9000);
        assert_eq!(config.socket_addr(), "0.0.0.0:9000");
    }

    #[test]
    fn test_document_session() {
        let mut session = DocumentSession::new();

        let ops = vec![
            WireCrdtOp {
                id: WireOpId {
                    client_id: "1".to_string(),
                    seq: 1,
                },
                op_type: "text_insert".to_string(),
                payload: serde_json::json!({}),
            },
            WireCrdtOp {
                id: WireOpId {
                    client_id: "1".to_string(),
                    seq: 2,
                },
                op_type: "text_insert".to_string(),
                payload: serde_json::json!({}),
            },
        ];

        session.add_ops(ops);

        assert_eq!(session.ops.len(), 2);
        assert_eq!(session.clock.clocks.get("1"), Some(&2));

        // Get ops since seq 1
        let since = WireVectorClock {
            clocks: [("1".to_string(), 1)].into_iter().collect(),
        };
        let new_ops = session.ops_since(&since);
        assert_eq!(new_ops.len(), 1);
        assert_eq!(new_ops[0].id.seq, 2);
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::with_port(0);
        let server = CollaborationServer::new(config);

        let stats = server.stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.total_documents, 0);
    }

    #[test]
    fn test_shutdown_handle() {
        let config = ServerConfig::default();
        let server = CollaborationServer::new(config);
        let handle = server.shutdown_handle();

        // Should not panic
        handle.shutdown();
    }
}
