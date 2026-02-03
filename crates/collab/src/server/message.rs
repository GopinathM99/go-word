//! Server-side message types for WebSocket protocol.
//!
//! This module defines the message types used for communication between
//! the collaboration server and clients. Messages are designed to be
//! compatible with the frontend TypeScript client.

use crate::clock::VectorClock;
use crate::op_id::OpId;
use crate::operation::CrdtOp;
use crate::presence::{Position, SelectionRange};
use serde::{Deserialize, Serialize};

/// Operation ID as used in the wire protocol.
/// Matches the frontend OpId type.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct WireOpId {
    pub client_id: String,
    pub seq: u64,
}

impl From<OpId> for WireOpId {
    fn from(op_id: OpId) -> Self {
        Self {
            client_id: op_id.client_id.0.to_string(),
            seq: op_id.seq,
        }
    }
}

impl WireOpId {
    /// Convert to internal OpId representation.
    pub fn to_op_id(&self) -> Option<OpId> {
        self.client_id
            .parse::<u64>()
            .ok()
            .map(|id| OpId::new(id, self.seq))
    }
}

/// Vector clock as used in the wire protocol.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WireVectorClock {
    pub clocks: std::collections::HashMap<String, u64>,
}

impl From<VectorClock> for WireVectorClock {
    fn from(vc: VectorClock) -> Self {
        let clocks = vc
            .iter()
            .map(|(client_id, seq)| (client_id.0.to_string(), seq))
            .collect();
        Self { clocks }
    }
}

impl WireVectorClock {
    /// Convert to internal VectorClock representation.
    pub fn to_vector_clock(&self) -> VectorClock {
        let mut vc = VectorClock::new();
        for (client_id_str, &seq) in &self.clocks {
            if let Ok(id) = client_id_str.parse::<u64>() {
                vc.set(crate::op_id::ClientId::new(id), seq);
            }
        }
        vc
    }
}

/// Wire format for CRDT operations.
/// Uses JSON value for flexibility.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireCrdtOp {
    pub id: WireOpId,
    #[serde(rename = "type")]
    pub op_type: String,
    pub payload: serde_json::Value,
}

impl From<&CrdtOp> for WireCrdtOp {
    fn from(op: &CrdtOp) -> Self {
        let id = WireOpId::from(op.id());
        match op {
            CrdtOp::TextInsert {
                node_id,
                parent_op_id,
                char,
                ..
            } => Self {
                id,
                op_type: "text_insert".to_string(),
                payload: serde_json::json!({
                    "nodeId": node_id.to_string(),
                    "parentOpId": WireOpId::from(*parent_op_id),
                    "char": char.to_string()
                }),
            },
            CrdtOp::TextDelete { target_id, .. } => Self {
                id,
                op_type: "text_delete".to_string(),
                payload: serde_json::json!({
                    "targetId": WireOpId::from(*target_id)
                }),
            },
            CrdtOp::FormatSet {
                node_id,
                start_op_id,
                end_op_id,
                attribute,
                value,
                timestamp,
                ..
            } => Self {
                id,
                op_type: "format_set".to_string(),
                payload: serde_json::json!({
                    "nodeId": node_id.to_string(),
                    "startOpId": WireOpId::from(*start_op_id),
                    "endOpId": WireOpId::from(*end_op_id),
                    "attribute": attribute,
                    "value": value,
                    "timestamp": {
                        "physical": timestamp.physical,
                        "logical": timestamp.logical,
                        "clientId": timestamp.client_id.0.to_string()
                    }
                }),
            },
            CrdtOp::BlockInsert {
                parent_op_id,
                after_sibling,
                node_id,
                data,
                ..
            } => Self {
                id,
                op_type: "block_insert".to_string(),
                payload: serde_json::json!({
                    "parentOpId": WireOpId::from(*parent_op_id),
                    "afterSibling": after_sibling.map(WireOpId::from),
                    "nodeId": node_id.to_string(),
                    "data": serde_json::to_value(data).unwrap_or(serde_json::Value::Null)
                }),
            },
            CrdtOp::BlockDelete { target_id, .. } => Self {
                id,
                op_type: "block_delete".to_string(),
                payload: serde_json::json!({
                    "targetId": WireOpId::from(*target_id)
                }),
            },
            CrdtOp::BlockMove {
                target_id,
                new_parent,
                after_sibling,
                ..
            } => Self {
                id,
                op_type: "block_move".to_string(),
                payload: serde_json::json!({
                    "targetId": WireOpId::from(*target_id),
                    "newParent": WireOpId::from(*new_parent),
                    "afterSibling": after_sibling.map(WireOpId::from)
                }),
            },
            CrdtOp::BlockUpdate {
                target_id,
                data,
                timestamp,
                ..
            } => Self {
                id,
                op_type: "block_update".to_string(),
                payload: serde_json::json!({
                    "targetId": WireOpId::from(*target_id),
                    "data": serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
                    "timestamp": {
                        "physical": timestamp.physical,
                        "logical": timestamp.logical,
                        "clientId": timestamp.client_id.0.to_string()
                    }
                }),
            },
        }
    }
}

/// Presence state as received from clients.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WirePresenceState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<WirePosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<WireRange>,
    pub is_typing: bool,
    pub last_active: u64,
}

/// Position in document.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WirePosition {
    pub node_id: String,
    pub offset: usize,
}

impl From<&Position> for WirePosition {
    fn from(pos: &Position) -> Self {
        Self {
            node_id: pos.node_id.clone(),
            offset: pos.offset,
        }
    }
}

impl From<WirePosition> for Position {
    fn from(wire: WirePosition) -> Self {
        Position::new(wire.node_id, wire.offset)
    }
}

/// Selection range.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireRange {
    pub start: WirePosition,
    pub end: WirePosition,
}

impl From<&SelectionRange> for WireRange {
    fn from(range: &SelectionRange) -> Self {
        Self {
            start: WirePosition::from(&range.start),
            end: WirePosition::from(&range.end),
        }
    }
}

impl From<WireRange> for SelectionRange {
    fn from(wire: WireRange) -> Self {
        SelectionRange::new(wire.start.into(), wire.end.into())
    }
}

/// User information for collaboration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub user_id: String,
    pub display_name: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<WirePresenceState>,
}

/// Messages sent from client to server.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Authentication request.
    Auth { token: String },

    /// Join a document session.
    Join {
        #[serde(rename = "docId")]
        doc_id: String,
    },

    /// Leave a document session.
    Leave {
        #[serde(rename = "docId")]
        doc_id: String,
    },

    /// Send CRDT operations.
    Ops { ops: Vec<WireCrdtOp> },

    /// Acknowledge received operations.
    Ack {
        #[serde(rename = "opIds")]
        op_ids: Vec<WireOpId>,
    },

    /// Update presence information.
    Presence { state: WirePresenceState },

    /// Request sync from a vector clock position.
    SyncRequest { since: WireVectorClock },

    /// Ping for connection health.
    Ping,
}

/// Messages sent from server to client.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Authentication succeeded.
    AuthSuccess {
        #[serde(rename = "userId")]
        user_id: String,
        #[serde(rename = "displayName")]
        display_name: String,
    },

    /// Authentication failed.
    AuthError { message: String },

    /// Successfully joined a document.
    Joined {
        #[serde(rename = "docId")]
        doc_id: String,
        users: Vec<UserInfo>,
    },

    /// A user joined the document.
    UserJoined { user: UserInfo },

    /// A user left the document.
    UserLeft {
        #[serde(rename = "userId")]
        user_id: String,
    },

    /// CRDT operations from other clients.
    Ops { ops: Vec<WireCrdtOp> },

    /// Acknowledgment of received operations.
    Ack {
        #[serde(rename = "opIds")]
        op_ids: Vec<WireOpId>,
    },

    /// Presence update from another user.
    Presence {
        #[serde(rename = "userId")]
        user_id: String,
        state: WirePresenceState,
    },

    /// Response to sync request.
    SyncResponse {
        ops: Vec<WireCrdtOp>,
        clock: WireVectorClock,
    },

    /// Error message.
    Error { code: String, message: String },

    /// Pong response to ping.
    Pong,
}

impl ServerMessage {
    /// Create an error message.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_op_id_conversion() {
        let op_id = OpId::new(42u64, 5);
        let wire = WireOpId::from(op_id);

        assert_eq!(wire.client_id, "42");
        assert_eq!(wire.seq, 5);

        let back = wire.to_op_id().unwrap();
        assert_eq!(back.client_id.0, 42);
        assert_eq!(back.seq, 5);
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"auth","token":"secret123"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Auth { token } => assert_eq!(token, "secret123"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::AuthSuccess {
            user_id: "user-1".to_string(),
            display_name: "Alice".to_string(),
        };

        let json = msg.to_json().unwrap();
        assert!(json.contains("auth_success"));
        assert!(json.contains("userId"));
        assert!(json.contains("user-1"));
    }

    #[test]
    fn test_join_message_deserialization() {
        let json = r#"{"type":"join","docId":"doc-123"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Join { doc_id } => assert_eq!(doc_id, "doc-123"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_presence_serialization() {
        let state = WirePresenceState {
            cursor: Some(WirePosition {
                node_id: "node-1".to_string(),
                offset: 10,
            }),
            selection: None,
            is_typing: true,
            last_active: 1234567890,
        };

        let msg = ServerMessage::Presence {
            user_id: "user-1".to_string(),
            state,
        };

        let json = msg.to_json().unwrap();
        assert!(json.contains("presence"));
        assert!(json.contains("isTyping"));
        assert!(json.contains("node-1"));
    }
}
