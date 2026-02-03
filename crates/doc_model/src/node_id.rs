//! Node ID generation and management

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a node in the document tree.
/// Uses UUID v4 for globally unique, stable IDs that survive serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// Create a new random NodeId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a NodeId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Create a NodeId from a string representation
    pub fn from_string(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for NodeId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<NodeId> for Uuid {
    fn from(id: NodeId) -> Self {
        id.0
    }
}
