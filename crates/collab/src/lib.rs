//! Collaboration features for real-time document editing.
//!
//! This crate provides CRDT (Conflict-free Replicated Data Types) implementations
//! for enabling multiple users to edit documents simultaneously without conflicts.
//!
//! # Modules
//!
//! - `clock`: Various clock implementations for ordering and causality tracking
//! - `conflict`: Conflict resolution engine for CRDT operations
//! - `op_id`: Operation identifiers for uniquely identifying operations
//! - `rga`: Replicated Growable Array for text sequences
//! - `crdt_tree`: Tree CRDT for hierarchical document structure
//! - `lww_register`: Last-Writer-Wins register for attribute values
//! - `operation`: Operation types for collaborative editing
//! - `permissions`: Permission and access control for collaboration
//! - `bridge`: Bridge between CRDT operations and the document model
//! - `error`: Error types for the collaboration crate
//!
//! # Example
//!
//! ```
//! use collab::clock::{HybridClock, Timestamp};
//! use collab::op_id::ClientId;
//! use collab::lww_register::{LwwMap, LwwRegister};
//!
//! // Create a clock for this client
//! let clock = HybridClock::new(ClientId::new(1));
//!
//! // Create an LWW map for formatting attributes
//! let mut formatting = LwwMap::<String, bool>::new(ClientId::new(1));
//!
//! // Set a formatting attribute
//! formatting.set("bold".to_string(), true, clock.now());
//!
//! // Get the current value
//! assert_eq!(formatting.get(&"bold".to_string()), Some(&true));
//! ```

pub mod bridge;
pub mod clock;
pub mod conflict;
pub mod crdt_tree;
pub mod error;
pub mod lww_register;
pub mod offline;
pub mod op_id;
pub mod operation;
pub mod permissions;
pub mod presence;
pub mod rga;
pub mod sync;
pub mod version;

/// WebSocket collaboration server module.
///
/// This module is only available when the `server` feature is enabled.
/// It provides a WebSocket server for real-time collaborative editing.
///
/// # Example
///
/// ```ignore
/// use collab::server::{CollaborationServer, ServerConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let config = ServerConfig::with_port(8080);
///     let server = CollaborationServer::new(config);
///     server.run().await.unwrap();
/// }
/// ```
#[cfg(feature = "server")]
pub mod server;

// Re-export commonly used types
pub use clock::{HybridClock, HybridLogicalClock, LamportClock, Timestamp, VectorClock};
pub use crdt_tree::{BlockData, CrdtTree, CrdtTreeNode, HeaderFooterType, TreeOperation};
pub use error::{CollabError, CollabResult};
pub use lww_register::{FormattingAttributes, FormattingMap, LwwMap, LwwOperation, LwwRegister};
pub use op_id::{ClientId, OpId};
pub use permissions::{
    DocId, Permission, PermissionError, PermissionLevel, PermissionManager, PermissionTarget,
    ShareLink, UserId,
};
pub use presence::{
    Position, PresenceManager, PresenceState, RemoteCursor, RemoteSelection, SelectionRange,
};
pub use rga::{Rga, RgaNode, RgaOperation};
pub use sync::{OpState, SyncEngine, SyncManager, SyncState, SyncStatus};
pub use bridge::{CollaborativeDocument, CollaborativeUndoStack, PositionMap};
pub use offline::{ConnectionStatus, MergeResult, OfflineError, OfflineManager, OfflineState, OfflineStatusInfo};
pub use version::{CheckpointConfig, Version, VersionDiff, VersionHistory, VersionId, VersionInfo};
pub use conflict::{are_concurrent, merge_with_resolution, ConflictRecord, ConflictResolver, ConflictResult, ConflictType};
