// Message types for WebSocket protocol

export interface OpId {
  clientId: string;
  seq: number;
}

export interface Timestamp {
  physical: number;
  logical: number;
  clientId: string;
}

export interface VectorClock {
  clocks: Record<string, number>;
}

// All message types
export type ClientMessage =
  | { type: 'auth'; token: string }
  | { type: 'join'; docId: string }
  | { type: 'leave'; docId: string }
  | { type: 'ops'; ops: CrdtOp[] }
  | { type: 'ack'; opIds: OpId[] }
  | { type: 'presence'; state: PresenceState }
  | { type: 'sync_request'; since: VectorClock }
  | { type: 'ping' };

export type ServerMessage =
  | { type: 'auth_success'; userId: string; displayName: string }
  | { type: 'auth_error'; message: string }
  | { type: 'joined'; docId: string; users: UserInfo[] }
  | { type: 'user_joined'; user: UserInfo }
  | { type: 'user_left'; userId: string }
  | { type: 'ops'; ops: CrdtOp[] }
  | { type: 'ack'; opIds: OpId[] }
  | { type: 'presence'; userId: string; state: PresenceState }
  | { type: 'sync_response'; ops: CrdtOp[]; clock: VectorClock }
  | { type: 'error'; code: string; message: string }
  | { type: 'pong' };

export interface CrdtOp {
  id: OpId;
  type: 'text_insert' | 'text_delete' | 'format_set' | 'block_insert' | 'block_delete' | 'block_move' | 'block_update';
  payload: unknown;
}

export interface PresenceState {
  cursor?: Position;
  selection?: Range;
  isTyping: boolean;
  lastActive: number;
}

export interface Position {
  nodeId: string;
  offset: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface UserInfo {
  userId: string;
  displayName: string;
  color: string;
  presence?: PresenceState;
}

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting';
