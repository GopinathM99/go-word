/**
 * usePresence - Hook for managing user presence in collaborative editing
 *
 * This hook handles:
 * - Tracking local user's cursor position and selection
 * - Broadcasting presence updates to other collaborators
 * - Managing typing indicators with automatic timeout
 * - Throttling updates to reduce network traffic
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';

// =============================================================================
// Types
// =============================================================================

/**
 * Position in the document
 */
export interface Position {
  /** Node ID in the document tree */
  nodeId: string;
  /** Character offset within the node */
  offset: number;
}

/**
 * Selection range in the document
 */
export interface SelectionRange {
  /** Start position */
  start: Position;
  /** End position */
  end: Position;
}

/**
 * Local presence state
 */
export interface PresenceState {
  /** Current cursor position */
  cursor: Position | null;
  /** Current selection range */
  selection: SelectionRange | null;
  /** Whether user is currently typing */
  isTyping: boolean;
  /** Current scroll position (for follow feature) */
  scrollPosition: number | null;
}

/**
 * Remote user presence data
 */
export interface RemotePresence {
  /** User ID */
  userId: string;
  /** Display name */
  displayName: string;
  /** Assigned color (hex) */
  color: string;
  /** Cursor position */
  cursor: Position | null;
  /** Selection range */
  selection: SelectionRange | null;
  /** Whether user is typing */
  isTyping: boolean;
  /** Last activity timestamp */
  lastActive: number;
  /** Scroll position */
  scrollPosition: number | null;
}

/**
 * Options for the usePresence hook
 */
export interface UsePresenceOptions {
  /** Current user ID */
  userId: string;
  /** Display name for the current user */
  displayName?: string;
  /** Minimum interval between presence updates (ms) */
  updateInterval?: number;
  /** Time before typing indicator is automatically cleared (ms) */
  typingTimeout?: number;
  /** Time before a user is considered idle (ms) */
  idleTimeout?: number;
  /** Callback when local presence changes (for broadcasting) */
  onPresenceChange?: (state: PresenceState) => void;
  /** Callback when remote presences are updated */
  onRemotePresenceUpdate?: (presences: RemotePresence[]) => void;
  /** Whether presence tracking is enabled */
  enabled?: boolean;
}

/**
 * Return type for the usePresence hook
 */
export interface UsePresenceReturn {
  /** Current local presence state */
  localPresence: PresenceState;
  /** Remote users' presence states */
  remotePresences: RemotePresence[];
  /** Update cursor position */
  updateCursor: (position: Position | null) => void;
  /** Update selection range */
  updateSelection: (selection: SelectionRange | null) => void;
  /** Trigger typing indicator */
  setTyping: () => void;
  /** Clear typing indicator */
  clearTyping: () => void;
  /** Update scroll position */
  updateScrollPosition: (position: number | null) => void;
  /** Set remote presences (for receiving from server) */
  setRemotePresences: (presences: RemotePresence[]) => void;
  /** Add or update a single remote presence */
  updateRemotePresence: (presence: RemotePresence) => void;
  /** Remove a remote presence */
  removeRemotePresence: (userId: string) => void;
  /** Check if a user is idle */
  isUserIdle: (userId: string) => boolean;
  /** Get active (non-idle) remote presences */
  activeRemotePresences: RemotePresence[];
}

// =============================================================================
// Constants
// =============================================================================

const DEFAULT_UPDATE_INTERVAL = 50; // ms
const DEFAULT_TYPING_TIMEOUT = 1500; // ms
const DEFAULT_IDLE_TIMEOUT = 60000; // ms (1 minute)

// =============================================================================
// Hook Implementation
// =============================================================================

export function usePresence(options: UsePresenceOptions): UsePresenceReturn {
  const {
    userId,
    // displayName is passed through for future use in local user display
    displayName: _displayName = 'Anonymous',
    updateInterval = DEFAULT_UPDATE_INTERVAL,
    typingTimeout = DEFAULT_TYPING_TIMEOUT,
    idleTimeout = DEFAULT_IDLE_TIMEOUT,
    onPresenceChange,
    onRemotePresenceUpdate,
    enabled = true,
  } = options;

  // Preserve displayName for potential future use
  void _displayName;

  // Local presence state
  const [localPresence, setLocalPresence] = useState<PresenceState>({
    cursor: null,
    selection: null,
    isTyping: false,
    scrollPosition: null,
  });

  // Remote presences
  const [remotePresences, setRemotePresences] = useState<RemotePresence[]>([]);

  // Refs for throttling and timeouts
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastUpdateRef = useRef<number>(0);
  const pendingUpdateRef = useRef<PresenceState | null>(null);
  const updateTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onPresenceChangeRef = useRef(onPresenceChange);

  // Keep callback ref updated
  useEffect(() => {
    onPresenceChangeRef.current = onPresenceChange;
  }, [onPresenceChange]);

  // Throttled update function
  const scheduleUpdate = useCallback(
    (state: PresenceState) => {
      if (!enabled) return;

      const now = Date.now();
      const timeSinceLastUpdate = now - lastUpdateRef.current;

      if (timeSinceLastUpdate >= updateInterval) {
        // Enough time has passed, send immediately
        lastUpdateRef.current = now;
        pendingUpdateRef.current = null;

        if (updateTimerRef.current) {
          clearTimeout(updateTimerRef.current);
          updateTimerRef.current = null;
        }

        onPresenceChangeRef.current?.(state);
      } else {
        // Schedule update for later
        pendingUpdateRef.current = state;

        if (!updateTimerRef.current) {
          const delay = updateInterval - timeSinceLastUpdate;
          updateTimerRef.current = setTimeout(() => {
            if (pendingUpdateRef.current) {
              onPresenceChangeRef.current?.(pendingUpdateRef.current);
              pendingUpdateRef.current = null;
              lastUpdateRef.current = Date.now();
            }
            updateTimerRef.current = null;
          }, delay);
        }
      }
    },
    [enabled, updateInterval]
  );

  // Update cursor position
  const updateCursor = useCallback(
    (position: Position | null) => {
      setLocalPresence((prev) => {
        const next = { ...prev, cursor: position };
        scheduleUpdate(next);
        return next;
      });
    },
    [scheduleUpdate]
  );

  // Update selection range
  const updateSelection = useCallback(
    (selection: SelectionRange | null) => {
      setLocalPresence((prev) => {
        const next = { ...prev, selection };
        scheduleUpdate(next);
        return next;
      });
    },
    [scheduleUpdate]
  );

  // Update scroll position
  const updateScrollPosition = useCallback(
    (position: number | null) => {
      setLocalPresence((prev) => {
        const next = { ...prev, scrollPosition: position };
        scheduleUpdate(next);
        return next;
      });
    },
    [scheduleUpdate]
  );

  // Set typing indicator (with auto-clear)
  const setTyping = useCallback(() => {
    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    setLocalPresence((prev) => {
      if (!prev.isTyping) {
        const next = { ...prev, isTyping: true };
        scheduleUpdate(next);
        return next;
      }
      return prev;
    });

    // Set timeout to clear typing indicator
    typingTimeoutRef.current = setTimeout(() => {
      setLocalPresence((prev) => {
        if (prev.isTyping) {
          const next = { ...prev, isTyping: false };
          scheduleUpdate(next);
          return next;
        }
        return prev;
      });
      typingTimeoutRef.current = null;
    }, typingTimeout);
  }, [typingTimeout, scheduleUpdate]);

  // Clear typing indicator immediately
  const clearTyping = useCallback(() => {
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }

    setLocalPresence((prev) => {
      if (prev.isTyping) {
        const next = { ...prev, isTyping: false };
        scheduleUpdate(next);
        return next;
      }
      return prev;
    });
  }, [scheduleUpdate]);

  // Set all remote presences
  const setRemotePresencesCallback = useCallback(
    (presences: RemotePresence[]) => {
      // Filter out the current user
      const filtered = presences.filter((p) => p.userId !== userId);
      setRemotePresences(filtered);
      onRemotePresenceUpdate?.(filtered);
    },
    [userId, onRemotePresenceUpdate]
  );

  // Add or update a single remote presence
  const updateRemotePresence = useCallback(
    (presence: RemotePresence) => {
      if (presence.userId === userId) return;

      setRemotePresences((prev) => {
        const index = prev.findIndex((p) => p.userId === presence.userId);
        let next: RemotePresence[];

        if (index >= 0) {
          next = [...prev];
          next[index] = presence;
        } else {
          next = [...prev, presence];
        }

        onRemotePresenceUpdate?.(next);
        return next;
      });
    },
    [userId, onRemotePresenceUpdate]
  );

  // Remove a remote presence
  const removeRemotePresence = useCallback(
    (removeUserId: string) => {
      setRemotePresences((prev) => {
        const next = prev.filter((p) => p.userId !== removeUserId);
        if (next.length !== prev.length) {
          onRemotePresenceUpdate?.(next);
        }
        return next;
      });
    },
    [onRemotePresenceUpdate]
  );

  // Check if a user is idle
  const isUserIdle = useCallback(
    (checkUserId: string): boolean => {
      const presence = remotePresences.find((p) => p.userId === checkUserId);
      if (!presence) return true;

      const now = Date.now();
      return now - presence.lastActive > idleTimeout;
    },
    [remotePresences, idleTimeout]
  );

  // Get active (non-idle) remote presences
  const activeRemotePresences = useMemo(() => {
    const now = Date.now();
    return remotePresences.filter((p) => now - p.lastActive <= idleTimeout);
  }, [remotePresences, idleTimeout]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      if (updateTimerRef.current) {
        clearTimeout(updateTimerRef.current);
      }
    };
  }, []);

  // Clear presence when disabled
  useEffect(() => {
    if (!enabled) {
      setLocalPresence({
        cursor: null,
        selection: null,
        isTyping: false,
        scrollPosition: null,
      });
    }
  }, [enabled]);

  return {
    localPresence,
    remotePresences,
    updateCursor,
    updateSelection,
    setTyping,
    clearTyping,
    updateScrollPosition,
    setRemotePresences: setRemotePresencesCallback,
    updateRemotePresence,
    removeRemotePresence,
    isUserIdle,
    activeRemotePresences,
  };
}

// =============================================================================
// Utility Functions
// =============================================================================

/**
 * Check if two positions are equal
 */
export function positionsEqual(a: Position | null, b: Position | null): boolean {
  if (a === null && b === null) return true;
  if (a === null || b === null) return false;
  return a.nodeId === b.nodeId && a.offset === b.offset;
}

/**
 * Check if two selection ranges are equal
 */
export function selectionsEqual(
  a: SelectionRange | null,
  b: SelectionRange | null
): boolean {
  if (a === null && b === null) return true;
  if (a === null || b === null) return false;
  return positionsEqual(a.start, b.start) && positionsEqual(a.end, b.end);
}

/**
 * Check if a selection range is collapsed (cursor position)
 */
export function isSelectionCollapsed(selection: SelectionRange | null): boolean {
  if (!selection) return true;
  return positionsEqual(selection.start, selection.end);
}

/**
 * Create a presence state from DOM selection
 */
export function presenceFromDOMSelection(
  selection: Selection | null,
  getNodeId: (node: Node) => string | null
): { cursor: Position | null; selection: SelectionRange | null } {
  if (!selection || selection.rangeCount === 0) {
    return { cursor: null, selection: null };
  }

  const range = selection.getRangeAt(0);

  const startNodeId = getNodeId(range.startContainer);
  const endNodeId = getNodeId(range.endContainer);

  if (!startNodeId || !endNodeId) {
    return { cursor: null, selection: null };
  }

  const start: Position = {
    nodeId: startNodeId,
    offset: range.startOffset,
  };

  const end: Position = {
    nodeId: endNodeId,
    offset: range.endOffset,
  };

  const selectionRange: SelectionRange = { start, end };

  // If collapsed, return as cursor
  if (range.collapsed) {
    return { cursor: start, selection: null };
  }

  return { cursor: end, selection: selectionRange };
}

/**
 * Generate a random user color
 */
export function generateUserColor(): string {
  const colors = [
    '#E91E63', // Pink
    '#9C27B0', // Purple
    '#3F51B5', // Indigo
    '#2196F3', // Blue
    '#00BCD4', // Cyan
    '#4CAF50', // Green
    '#FF9800', // Orange
    '#795548', // Brown
  ];
  return colors[Math.floor(Math.random() * colors.length)];
}

/**
 * Generate a deterministic color from a user ID
 */
export function userIdToColor(userId: string): string {
  let hash = 0;
  for (let i = 0; i < userId.length; i++) {
    hash = userId.charCodeAt(i) + ((hash << 5) - hash);
  }

  const colors = [
    '#E91E63',
    '#9C27B0',
    '#3F51B5',
    '#2196F3',
    '#00BCD4',
    '#4CAF50',
    '#FF9800',
    '#795548',
  ];

  return colors[Math.abs(hash) % colors.length];
}

export default usePresence;
