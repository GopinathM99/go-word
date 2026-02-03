import {
  ClientMessage,
  ServerMessage,
  CrdtOp,
  PresenceState,
  OpId,
  VectorClock,
  ConnectionState,
  UserInfo,
} from './types';

export interface CollaborationClientOptions {
  url: string;
  authToken: string;
  docId: string;
  onOps?: (ops: CrdtOp[]) => void;
  onPresence?: (userId: string, state: PresenceState) => void;
  onUserJoined?: (user: UserInfo) => void;
  onUserLeft?: (userId: string) => void;
  onConnectionChange?: (state: ConnectionState) => void;
  onError?: (error: Error) => void;
  reconnectDelay?: number;
  maxReconnectDelay?: number;
  batchWindow?: number;
}

/**
 * Converts an OpId to a unique string key for tracking in sets/maps
 */
function opIdToKey(opId: OpId): string {
  return `${opId.clientId}:${opId.seq}`;
}

/**
 * WebSocket client for real-time collaboration
 * Handles connection management, operation batching, presence updates, and reconnection
 */
export class CollaborationClient {
  private ws: WebSocket | null = null;
  private options: Required<CollaborationClientOptions>;
  private connectionState: ConnectionState = 'disconnected';
  private reconnectAttempts = 0;
  private reconnectTimer?: ReturnType<typeof setTimeout>;
  private pingTimer?: ReturnType<typeof setInterval>;
  private pendingOps: CrdtOp[] = [];
  private acknowledged: Set<string> = new Set();
  private batchTimer?: ReturnType<typeof setTimeout>;
  private batchQueue: CrdtOp[] = [];
  private vectorClock: VectorClock = { clocks: {} };
  private userId?: string;
  private users: Map<string, UserInfo> = new Map();
  private currentPresence: PresenceState = { isTyping: false, lastActive: Date.now() };
  private lastPresenceUpdate = 0;
  private presenceThrottleMs = 1000 / 30; // Max 30 updates/second
  private pendingPresenceUpdate: PresenceState | null = null;
  private presenceThrottleTimer?: ReturnType<typeof setTimeout>;
  private isManualDisconnect = false;
  private connectPromise: Promise<void> | null = null;
  private connectResolve: (() => void) | null = null;
  private connectReject: ((error: Error) => void) | null = null;

  constructor(options: CollaborationClientOptions) {
    this.options = {
      reconnectDelay: 1000,
      maxReconnectDelay: 30000,
      batchWindow: 50,
      onOps: () => {},
      onPresence: () => {},
      onUserJoined: () => {},
      onUserLeft: () => {},
      onConnectionChange: () => {},
      onError: () => {},
      ...options,
    };
  }

  /**
   * Connect to the WebSocket server
   * Returns a promise that resolves when authenticated and joined the document
   */
  connect(): Promise<void> {
    // If already connecting or connected, return existing promise or resolve immediately
    if (this.connectionState === 'connected') {
      return Promise.resolve();
    }
    if (this.connectPromise) {
      return this.connectPromise;
    }

    this.isManualDisconnect = false;
    this.setConnectionState('connecting');

    this.connectPromise = new Promise((resolve, reject) => {
      this.connectResolve = resolve;
      this.connectReject = reject;

      try {
        this.ws = new WebSocket(this.options.url);

        this.ws.onopen = () => {
          // Send authentication message
          this.sendMessage({ type: 'auth', token: this.options.authToken });
        };

        this.ws.onclose = (event) => {
          this.handleClose(event);
        };

        this.ws.onerror = (_event) => {
          const error = new Error('WebSocket error');
          this.options.onError(error);

          // Reject connect promise if still pending
          if (this.connectReject) {
            this.connectReject(error);
            this.clearConnectPromise();
          }
        };

        this.ws.onmessage = (event) => {
          try {
            const msg = JSON.parse(event.data) as ServerMessage;
            this.handleMessage(msg);
          } catch (e) {
            this.options.onError(new Error('Failed to parse server message'));
          }
        };
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Failed to create WebSocket');
        this.setConnectionState('disconnected');
        reject(err);
        this.clearConnectPromise();
      }
    });

    return this.connectPromise;
  }

  /**
   * Disconnect from the server
   * Cleans up all timers and resets state
   */
  disconnect(): void {
    this.isManualDisconnect = true;
    this.cleanup();
    this.setConnectionState('disconnected');
  }

  /**
   * Send operations to the server immediately
   * Operations are tracked as pending until acknowledged
   */
  send(ops: CrdtOp[]): void {
    if (ops.length === 0) return;

    // Add to pending (unacknowledged) operations
    for (const op of ops) {
      const key = opIdToKey(op.id);
      if (!this.acknowledged.has(key)) {
        this.pendingOps.push(op);
      }
    }

    // Update local vector clock
    for (const op of ops) {
      const currentSeq = this.vectorClock.clocks[op.id.clientId] || 0;
      if (op.id.seq > currentSeq) {
        this.vectorClock.clocks[op.id.clientId] = op.id.seq;
      }
    }

    if (this.connectionState === 'connected' && this.ws?.readyState === WebSocket.OPEN) {
      this.sendMessage({ type: 'ops', ops });
    }
    // If not connected, ops will be retried after reconnect
  }

  /**
   * Queue a single operation for batched sending
   * Operations are collected and sent after batchWindow milliseconds
   */
  queueOp(op: CrdtOp): void {
    this.batchQueue.push(op);

    // Start batch timer if not already running
    if (!this.batchTimer) {
      this.batchTimer = setTimeout(() => {
        this.flushBatch();
      }, this.options.batchWindow);
    }
  }

  /**
   * Update local presence and send to server
   * Updates are throttled to max 30 per second
   */
  updatePresence(state: Partial<PresenceState>): void {
    // Merge with current presence
    this.currentPresence = {
      ...this.currentPresence,
      ...state,
      lastActive: Date.now(),
    };

    const now = Date.now();
    const timeSinceLastUpdate = now - this.lastPresenceUpdate;

    if (timeSinceLastUpdate >= this.presenceThrottleMs) {
      // Enough time has passed, send immediately
      this.sendPresenceUpdate();
    } else {
      // Throttle: schedule update for later
      this.pendingPresenceUpdate = this.currentPresence;

      if (!this.presenceThrottleTimer) {
        this.presenceThrottleTimer = setTimeout(() => {
          this.presenceThrottleTimer = undefined;
          if (this.pendingPresenceUpdate) {
            this.sendPresenceUpdate();
            this.pendingPresenceUpdate = null;
          }
        }, this.presenceThrottleMs - timeSinceLastUpdate);
      }
    }
  }

  /**
   * Request sync from a specific vector clock position
   * Used to catch up after reconnection
   */
  requestSync(since?: VectorClock): void {
    if (this.connectionState !== 'connected' || !this.ws) return;

    this.sendMessage({
      type: 'sync_request',
      since: since || { clocks: {} },
    });
  }

  /**
   * Get current connection state
   */
  getState(): ConnectionState {
    return this.connectionState;
  }

  /**
   * Get list of connected users
   */
  getUsers(): UserInfo[] {
    return Array.from(this.users.values());
  }

  /**
   * Get current user ID (available after authentication)
   */
  getUserId(): string | undefined {
    return this.userId;
  }

  /**
   * Handle acknowledgment of operations from server
   */
  private handleAck(opIds: OpId[]): void {
    for (const opId of opIds) {
      const key = opIdToKey(opId);
      this.acknowledged.add(key);

      // Remove from pending
      const index = this.pendingOps.findIndex(
        (op) => op.id.clientId === opId.clientId && op.id.seq === opId.seq
      );
      if (index !== -1) {
        this.pendingOps.splice(index, 1);
      }
    }
  }

  /**
   * Schedule reconnection with exponential backoff
   */
  private scheduleReconnect(): void {
    if (this.isManualDisconnect) return;

    this.setConnectionState('reconnecting');

    // Calculate delay with exponential backoff
    const delay = Math.min(
      this.options.reconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.options.maxReconnectDelay
    );

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.connectPromise = null; // Clear so connect() creates new promise
      this.connect().catch(() => {
        // Error handling is done in connect()
      });
    }, delay);
  }

  /**
   * Start ping timer for connection health monitoring
   * Sends ping every 30 seconds
   */
  private startPingTimer(): void {
    this.stopPingTimer();

    this.pingTimer = setInterval(() => {
      if (this.connectionState === 'connected' && this.ws?.readyState === WebSocket.OPEN) {
        this.sendMessage({ type: 'ping' });
      }
    }, 30000);
  }

  /**
   * Stop the ping timer
   */
  private stopPingTimer(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = undefined;
    }
  }

  /**
   * Handle incoming server messages
   */
  private handleMessage(msg: ServerMessage): void {
    switch (msg.type) {
      case 'auth_success':
        this.userId = msg.userId;
        // Join the document after successful auth
        this.sendMessage({ type: 'join', docId: this.options.docId });
        break;

      case 'auth_error':
        const authError = new Error(`Authentication failed: ${msg.message}`);
        this.options.onError(authError);
        if (this.connectReject) {
          this.connectReject(authError);
          this.clearConnectPromise();
        }
        // Don't reconnect on auth errors
        this.isManualDisconnect = true;
        this.cleanup();
        this.setConnectionState('disconnected');
        break;

      case 'joined':
        // Successfully joined document
        this.users.clear();
        for (const user of msg.users) {
          this.users.set(user.userId, user);
        }
        this.reconnectAttempts = 0;
        this.setConnectionState('connected');
        this.startPingTimer();

        // Resolve connect promise
        if (this.connectResolve) {
          this.connectResolve();
          this.clearConnectPromise();
        }

        // Resend any pending operations
        if (this.pendingOps.length > 0) {
          this.sendMessage({ type: 'ops', ops: this.pendingOps });
        }

        // Request sync to catch up on missed operations
        this.requestSync(this.vectorClock);
        break;

      case 'user_joined':
        this.users.set(msg.user.userId, msg.user);
        this.options.onUserJoined(msg.user);
        break;

      case 'user_left':
        this.users.delete(msg.userId);
        this.options.onUserLeft(msg.userId);
        break;

      case 'ops':
        // Update vector clock from received operations
        for (const op of msg.ops) {
          const currentSeq = this.vectorClock.clocks[op.id.clientId] || 0;
          if (op.id.seq > currentSeq) {
            this.vectorClock.clocks[op.id.clientId] = op.id.seq;
          }
        }
        this.options.onOps(msg.ops);

        // Send acknowledgment
        this.sendMessage({ type: 'ack', opIds: msg.ops.map((op) => op.id) });
        break;

      case 'ack':
        this.handleAck(msg.opIds);
        break;

      case 'presence':
        // Update user presence
        const user = this.users.get(msg.userId);
        if (user) {
          user.presence = msg.state;
          this.users.set(msg.userId, user);
        }
        this.options.onPresence(msg.userId, msg.state);
        break;

      case 'sync_response':
        // Update vector clock
        this.vectorClock = msg.clock;

        // Apply received operations
        if (msg.ops.length > 0) {
          this.options.onOps(msg.ops);
        }
        break;

      case 'error':
        this.options.onError(new Error(`Server error [${msg.code}]: ${msg.message}`));
        break;

      case 'pong':
        // Connection is healthy, nothing to do
        break;
    }
  }

  /**
   * Handle WebSocket close event
   */
  private handleClose(_event: CloseEvent): void {
    this.stopPingTimer();
    this.ws = null;

    if (!this.isManualDisconnect) {
      // Unexpected disconnect, attempt reconnection
      this.scheduleReconnect();
    } else {
      this.setConnectionState('disconnected');
    }
  }

  /**
   * Send presence update to server
   */
  private sendPresenceUpdate(): void {
    if (this.connectionState !== 'connected' || !this.ws) return;

    this.sendMessage({ type: 'presence', state: this.currentPresence });
    this.lastPresenceUpdate = Date.now();
  }

  /**
   * Flush batched operations
   */
  private flushBatch(): void {
    this.batchTimer = undefined;

    if (this.batchQueue.length > 0) {
      this.send(this.batchQueue);
      this.batchQueue = [];
    }
  }

  /**
   * Send a message to the server
   */
  private sendMessage(msg: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  /**
   * Update connection state and notify callback
   */
  private setConnectionState(state: ConnectionState): void {
    if (this.connectionState !== state) {
      this.connectionState = state;
      this.options.onConnectionChange(state);
    }
  }

  /**
   * Clear connect promise references
   */
  private clearConnectPromise(): void {
    this.connectPromise = null;
    this.connectResolve = null;
    this.connectReject = null;
  }

  /**
   * Clean up all timers and WebSocket
   */
  private cleanup(): void {
    this.stopPingTimer();

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = undefined;
    }

    if (this.batchTimer) {
      clearTimeout(this.batchTimer);
      this.batchTimer = undefined;
    }

    if (this.presenceThrottleTimer) {
      clearTimeout(this.presenceThrottleTimer);
      this.presenceThrottleTimer = undefined;
    }

    if (this.ws) {
      this.ws.onclose = null;
      this.ws.onerror = null;
      this.ws.onmessage = null;
      this.ws.onopen = null;
      this.ws.close();
      this.ws = null;
    }

    this.clearConnectPromise();
  }
}
