import { useState, useEffect, useCallback, useRef } from 'react';
import { CollaborationClient } from './CollaborationClient';
import { CrdtOp, UserInfo, ConnectionState, Position, Range } from './types';

export interface UseCollaborationOptions {
  serverUrl: string;
  authToken: string;
  docId: string;
  onRemoteOps?: (ops: CrdtOp[]) => void;
}

export interface UseCollaborationResult {
  // Connection state
  connectionState: ConnectionState;
  isConnected: boolean;

  // Users
  users: UserInfo[];
  currentUserId: string | undefined;

  // Operations
  sendOps: (ops: CrdtOp[]) => void;
  queueOp: (op: CrdtOp) => void;

  // Presence
  updateCursor: (position: Position | null) => void;
  updateSelection: (range: Range | null) => void;
  setTyping: (isTyping: boolean) => void;

  // Error
  error: Error | null;

  // Reconnect
  reconnect: () => void;
}

// Debounce time for typing indicator (ms)
const TYPING_DEBOUNCE_MS = 2000;

/**
 * React hook for real-time collaboration
 * Manages WebSocket connection, presence updates, and operation syncing
 */
export function useCollaboration(options: UseCollaborationOptions): UseCollaborationResult {
  const { serverUrl, authToken, docId, onRemoteOps } = options;

  // State
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
  const [users, setUsers] = useState<UserInfo[]>([]);
  const [currentUserId, setCurrentUserId] = useState<string | undefined>();
  const [error, setError] = useState<Error | null>(null);

  // Refs
  const clientRef = useRef<CollaborationClient | null>(null);
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);
  const onRemoteOpsRef = useRef(onRemoteOps);

  // Keep onRemoteOps ref updated
  useEffect(() => {
    onRemoteOpsRef.current = onRemoteOps;
  }, [onRemoteOps]);

  // Initialize and manage client
  useEffect(() => {
    // Create client
    const client = new CollaborationClient({
      url: serverUrl,
      authToken,
      docId,
      onOps: (ops) => {
        onRemoteOpsRef.current?.(ops);
      },
      onPresence: (userId, state) => {
        setUsers((prevUsers) => {
          return prevUsers.map((user) =>
            user.userId === userId ? { ...user, presence: state } : user
          );
        });
      },
      onUserJoined: (user) => {
        setUsers((prevUsers) => {
          // Avoid duplicates
          const exists = prevUsers.some((u) => u.userId === user.userId);
          if (exists) {
            return prevUsers.map((u) => (u.userId === user.userId ? user : u));
          }
          return [...prevUsers, user];
        });
      },
      onUserLeft: (userId) => {
        setUsers((prevUsers) => prevUsers.filter((u) => u.userId !== userId));
      },
      onConnectionChange: (state) => {
        setConnectionState(state);
        if (state === 'connected') {
          // Clear error on successful connection
          setError(null);
          // Update users list
          setUsers(client.getUsers());
          setCurrentUserId(client.getUserId());
        }
      },
      onError: (err) => {
        setError(err);
      },
    });

    clientRef.current = client;

    // Connect
    client.connect().catch(() => {
      // Error is already set via onError callback
    });

    // Cleanup on unmount
    return () => {
      // Clear typing timeout
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }

      // Disconnect client
      client.disconnect();
      clientRef.current = null;
    };
  }, [serverUrl, authToken, docId]);

  // Send operations immediately
  const sendOps = useCallback((ops: CrdtOp[]) => {
    clientRef.current?.send(ops);
  }, []);

  // Queue operation for batched sending
  const queueOp = useCallback((op: CrdtOp) => {
    clientRef.current?.queueOp(op);
  }, []);

  // Update cursor position
  const updateCursor = useCallback((position: Position | null) => {
    clientRef.current?.updatePresence({
      cursor: position || undefined,
    });
  }, []);

  // Update selection range
  const updateSelection = useCallback((range: Range | null) => {
    clientRef.current?.updatePresence({
      selection: range || undefined,
    });
  }, []);

  // Set typing indicator with debounce
  const setTyping = useCallback((isTyping: boolean) => {
    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }

    if (isTyping) {
      // Set typing to true
      if (!isTypingRef.current) {
        isTypingRef.current = true;
        clientRef.current?.updatePresence({ isTyping: true });
      }

      // Set timeout to clear typing indicator
      typingTimeoutRef.current = setTimeout(() => {
        isTypingRef.current = false;
        clientRef.current?.updatePresence({ isTyping: false });
        typingTimeoutRef.current = null;
      }, TYPING_DEBOUNCE_MS);
    } else {
      // Immediately set typing to false
      if (isTypingRef.current) {
        isTypingRef.current = false;
        clientRef.current?.updatePresence({ isTyping: false });
      }
    }
  }, []);

  // Manual reconnect
  const reconnect = useCallback(() => {
    if (clientRef.current) {
      // Disconnect and reconnect
      clientRef.current.disconnect();
      clientRef.current.connect().catch(() => {
        // Error handled via callback
      });
    }
  }, []);

  return {
    connectionState,
    isConnected: connectionState === 'connected',
    users,
    currentUserId,
    sendOps,
    queueOp,
    updateCursor,
    updateSelection,
    setTyping,
    error,
    reconnect,
  };
}
