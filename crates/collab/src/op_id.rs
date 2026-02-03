//! Operation identifiers for collaborative editing.
//!
//! This module provides types for uniquely identifying clients and operations
//! in a distributed collaborative editing system.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a client/peer in the collaborative system.
///
/// Client IDs are used for:
/// - Identifying the source of operations
/// - Breaking ties in concurrent operations (LWW registers)
/// - Tracking which client has made specific changes
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

impl ClientId {
    /// Create a new ClientId with the given value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client({})", self.0)
    }
}

impl From<u64> for ClientId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<ClientId> for u64 {
    fn from(id: ClientId) -> Self {
        id.0
    }
}

/// Unique identifier for an operation.
///
/// Combines a client ID with a local sequence number to create
/// a globally unique operation identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpId {
    /// The client that created this operation
    pub client_id: ClientId,
    /// Local sequence number (monotonically increasing per client)
    pub seq: u64,
}

impl OpId {
    /// Create a new OpId with a ClientId and sequence number.
    pub fn new(client_id: impl Into<ClientId>, seq: u64) -> Self {
        Self {
            client_id: client_id.into(),
            seq,
        }
    }

    /// Create the root OpId (represents the beginning of a sequence).
    pub fn root() -> Self {
        Self {
            client_id: ClientId(0),
            seq: 0,
        }
    }

    /// Check if this is the root OpId.
    pub fn is_root(&self) -> bool {
        self.client_id.0 == 0 && self.seq == 0
    }
}

impl fmt::Display for OpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Op({}, {})", self.client_id.0, self.seq)
    }
}

impl PartialOrd for OpId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OpId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by sequence, then by client_id for deterministic ordering
        match self.seq.cmp(&other.seq) {
            std::cmp::Ordering::Equal => self.client_id.cmp(&other.client_id),
            ord => ord,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_creation() {
        let id = ClientId::new(42);
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn test_client_id_ordering() {
        let id1 = ClientId::new(1);
        let id2 = ClientId::new(2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_op_id_ordering() {
        let op1 = OpId::new(ClientId::new(1), 1);
        let op2 = OpId::new(ClientId::new(2), 1);
        let op3 = OpId::new(ClientId::new(1), 2);

        // Same seq, different client - client_id breaks tie
        assert!(op1 < op2);
        // Different seq - seq takes priority
        assert!(op1 < op3);
        assert!(op2 < op3);
    }

    #[test]
    fn test_client_id_from_u64() {
        let id: ClientId = 42u64.into();
        assert_eq!(id.value(), 42);

        let val: u64 = id.into();
        assert_eq!(val, 42);
    }
}
