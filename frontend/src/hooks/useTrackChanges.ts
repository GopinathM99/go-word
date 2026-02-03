/**
 * useTrackChanges - Hook to manage track changes state
 *
 * Features:
 * - Toggle track changes on/off
 * - Set markup viewing mode
 * - Fetch and navigate revisions
 * - Accept/reject individual or all revisions
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

// =============================================================================
// Types
// =============================================================================

/**
 * Type of revision
 */
export type RevisionType = 'insert' | 'delete' | 'format' | 'move';

/**
 * Markup viewing mode
 */
export type MarkupMode = 'original' | 'final' | 'all-markup' | 'simple-markup';

/**
 * Revision author information
 */
export interface RevisionAuthor {
  id: string;
  name: string;
  initials: string;
  color: string;
}

/**
 * A single revision in the document
 */
export interface Revision {
  id: string;
  type: RevisionType;
  author: RevisionAuthor;
  timestamp: number;
  contentPreview: string;
  paragraphId: string;
  startOffset: number;
  endOffset: number;
  /** For format changes, the formatting property that changed */
  formatProperty?: string;
  /** For moves, the original location */
  originalLocation?: {
    paragraphId: string;
    startOffset: number;
    endOffset: number;
  };
}

/**
 * Track changes state from the backend
 */
export interface TrackingState {
  enabled: boolean;
  markupMode: MarkupMode;
  revisionCount: number;
  currentUserId: string;
  currentUserName: string;
}

/**
 * Options for the useTrackChanges hook
 */
export interface UseTrackChangesOptions {
  /** Document ID */
  documentId?: string;
  /** Polling interval for status updates (in ms) */
  pollInterval?: number;
  /** Callback when tracking state changes */
  onTrackingStateChange?: (state: TrackingState) => void;
  /** Callback when revisions change */
  onRevisionsChange?: (revisions: Revision[]) => void;
  /** Callback when a revision is navigated to */
  onNavigateToRevision?: (revision: Revision) => void;
}

/**
 * Return type for the useTrackChanges hook
 */
export interface UseTrackChangesReturn {
  /** Whether track changes is enabled */
  isTrackingEnabled: boolean;
  /** Current markup viewing mode */
  markupMode: MarkupMode;
  /** All revisions in the document */
  revisions: Revision[];
  /** Currently selected revision (if any) */
  currentRevision: Revision | null;
  /** Index of the current revision */
  currentRevisionIndex: number;
  /** Loading state */
  isLoading: boolean;
  /** Error state */
  error: string | null;
  /** Toggle track changes on/off */
  toggleTracking: () => Promise<void>;
  /** Set markup viewing mode */
  setMarkupMode: (mode: MarkupMode) => Promise<void>;
  /** Navigate to a specific revision */
  navigateToRevision: (revisionId: string) => Promise<void>;
  /** Navigate to the next revision */
  nextRevision: () => Promise<void>;
  /** Navigate to the previous revision */
  previousRevision: () => Promise<void>;
  /** Accept a specific revision */
  acceptRevision: (revisionId: string) => Promise<void>;
  /** Reject a specific revision */
  rejectRevision: (revisionId: string) => Promise<void>;
  /** Accept the current revision */
  acceptCurrentRevision: () => Promise<void>;
  /** Reject the current revision */
  rejectCurrentRevision: () => Promise<void>;
  /** Accept all revisions */
  acceptAllRevisions: () => Promise<void>;
  /** Reject all revisions */
  rejectAllRevisions: () => Promise<void>;
  /** Refresh revisions from backend */
  refreshRevisions: () => Promise<void>;
  /** Get revisions filtered by type */
  getRevisionsByType: (type: RevisionType) => Revision[];
  /** Get revisions filtered by author */
  getRevisionsByAuthor: (authorId: string) => Revision[];
  /** Get unique authors from revisions */
  getAuthors: () => RevisionAuthor[];
}

// =============================================================================
// Default Values
// =============================================================================

const DEFAULT_TRACKING_STATE: TrackingState = {
  enabled: false,
  markupMode: 'all-markup',
  revisionCount: 0,
  currentUserId: 'user-1',
  currentUserName: 'Current User',
};

// =============================================================================
// Author Color Generation
// =============================================================================

/**
 * Generate a distinct color for an author based on their ID
 */
export function generateAuthorColor(authorId: string): string {
  // Predefined colors that are visually distinct
  const colors = [
    '#2E7D32', // Green
    '#C62828', // Red
    '#1565C0', // Blue
    '#6A1B9A', // Purple
    '#E65100', // Orange
    '#00838F', // Teal
    '#AD1457', // Pink
    '#4527A0', // Deep Purple
    '#00695C', // Dark Teal
    '#BF360C', // Deep Orange
  ];

  // Generate a hash from the author ID
  let hash = 0;
  for (let i = 0; i < authorId.length; i++) {
    const char = authorId.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }

  // Use the hash to select a color
  const index = Math.abs(hash) % colors.length;
  return colors[index];
}

// =============================================================================
// Hook Implementation
// =============================================================================

export function useTrackChanges(options: UseTrackChangesOptions = {}): UseTrackChangesReturn {
  const {
    documentId = 'default',
    pollInterval = 5000,
    onTrackingStateChange,
    onRevisionsChange,
    onNavigateToRevision,
  } = options;

  const [isTrackingEnabled, setIsTrackingEnabled] = useState(false);
  const [markupMode, setMarkupModeState] = useState<MarkupMode>('all-markup');
  const [revisions, setRevisions] = useState<Revision[]>([]);
  const [currentRevisionIndex, setCurrentRevisionIndex] = useState(-1);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const lastStateRef = useRef<TrackingState>(DEFAULT_TRACKING_STATE);

  // Current revision based on index
  const currentRevision = currentRevisionIndex >= 0 && currentRevisionIndex < revisions.length
    ? revisions[currentRevisionIndex]
    : null;

  // Fetch initial state
  useEffect(() => {
    const fetchInitialState = async () => {
      setIsLoading(true);
      try {
        const [state, revisionsData] = await Promise.all([
          invoke<TrackingState>('get_tracking_state', { docId: documentId }),
          invoke<Revision[]>('get_revisions', { docId: documentId }),
        ]);

        setIsTrackingEnabled(state.enabled);
        setMarkupModeState(state.markupMode);
        setRevisions(revisionsData);
        lastStateRef.current = state;

        if (revisionsData.length > 0) {
          setCurrentRevisionIndex(0);
        }
      } catch (e) {
        console.error('Failed to fetch track changes state:', e);
        setError(String(e));
      } finally {
        setIsLoading(false);
      }
    };

    fetchInitialState();
  }, [documentId]);

  // Poll for updates
  useEffect(() => {
    if (pollInterval <= 0) return;

    const interval = setInterval(async () => {
      try {
        const state = await invoke<TrackingState>('get_tracking_state', { docId: documentId });

        // Check if state changed
        if (
          state.enabled !== lastStateRef.current.enabled ||
          state.markupMode !== lastStateRef.current.markupMode ||
          state.revisionCount !== lastStateRef.current.revisionCount
        ) {
          setIsTrackingEnabled(state.enabled);
          setMarkupModeState(state.markupMode);
          lastStateRef.current = state;

          if (onTrackingStateChange) {
            onTrackingStateChange(state);
          }

          // Refresh revisions if count changed
          if (state.revisionCount !== lastStateRef.current.revisionCount) {
            const revisionsData = await invoke<Revision[]>('get_revisions', { docId: documentId });
            setRevisions(revisionsData);
            if (onRevisionsChange) {
              onRevisionsChange(revisionsData);
            }
          }
        }
      } catch (e) {
        console.error('Failed to poll track changes state:', e);
      }
    }, pollInterval);

    return () => clearInterval(interval);
  }, [documentId, pollInterval, onTrackingStateChange, onRevisionsChange]);

  /**
   * Toggle track changes on/off
   */
  const toggleTracking = useCallback(async () => {
    try {
      const newState = await invoke<TrackingState>('toggle_track_changes', { docId: documentId });
      setIsTrackingEnabled(newState.enabled);
      lastStateRef.current = newState;
      if (onTrackingStateChange) {
        onTrackingStateChange(newState);
      }
    } catch (e) {
      console.error('Failed to toggle track changes:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, onTrackingStateChange]);

  /**
   * Set markup viewing mode
   */
  const setMarkupMode = useCallback(async (mode: MarkupMode) => {
    try {
      await invoke('set_markup_mode', { docId: documentId, mode });
      setMarkupModeState(mode);
      lastStateRef.current = { ...lastStateRef.current, markupMode: mode };
      if (onTrackingStateChange) {
        onTrackingStateChange(lastStateRef.current);
      }
    } catch (e) {
      console.error('Failed to set markup mode:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, onTrackingStateChange]);

  /**
   * Navigate to a specific revision
   */
  const navigateToRevision = useCallback(async (revisionId: string) => {
    try {
      await invoke('navigate_to_revision', { docId: documentId, revisionId });
      const index = revisions.findIndex(r => r.id === revisionId);
      if (index >= 0) {
        setCurrentRevisionIndex(index);
        if (onNavigateToRevision) {
          onNavigateToRevision(revisions[index]);
        }
      }
    } catch (e) {
      console.error('Failed to navigate to revision:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, revisions, onNavigateToRevision]);

  /**
   * Navigate to the next revision
   */
  const nextRevision = useCallback(async () => {
    if (revisions.length === 0) return;

    const nextIndex = (currentRevisionIndex + 1) % revisions.length;
    await navigateToRevision(revisions[nextIndex].id);
  }, [revisions, currentRevisionIndex, navigateToRevision]);

  /**
   * Navigate to the previous revision
   */
  const previousRevision = useCallback(async () => {
    if (revisions.length === 0) return;

    const prevIndex = currentRevisionIndex <= 0 ? revisions.length - 1 : currentRevisionIndex - 1;
    await navigateToRevision(revisions[prevIndex].id);
  }, [revisions, currentRevisionIndex, navigateToRevision]);

  /**
   * Accept a specific revision
   */
  const acceptRevision = useCallback(async (revisionId: string) => {
    try {
      await invoke('accept_revision', { docId: documentId, revisionId });

      // Remove the revision from the list
      const newRevisions = revisions.filter(r => r.id !== revisionId);
      setRevisions(newRevisions);

      // Adjust current index if needed
      if (currentRevisionIndex >= newRevisions.length) {
        setCurrentRevisionIndex(Math.max(0, newRevisions.length - 1));
      }

      if (onRevisionsChange) {
        onRevisionsChange(newRevisions);
      }
    } catch (e) {
      console.error('Failed to accept revision:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, revisions, currentRevisionIndex, onRevisionsChange]);

  /**
   * Reject a specific revision
   */
  const rejectRevision = useCallback(async (revisionId: string) => {
    try {
      await invoke('reject_revision', { docId: documentId, revisionId });

      // Remove the revision from the list
      const newRevisions = revisions.filter(r => r.id !== revisionId);
      setRevisions(newRevisions);

      // Adjust current index if needed
      if (currentRevisionIndex >= newRevisions.length) {
        setCurrentRevisionIndex(Math.max(0, newRevisions.length - 1));
      }

      if (onRevisionsChange) {
        onRevisionsChange(newRevisions);
      }
    } catch (e) {
      console.error('Failed to reject revision:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, revisions, currentRevisionIndex, onRevisionsChange]);

  /**
   * Accept the current revision
   */
  const acceptCurrentRevision = useCallback(async () => {
    if (currentRevision) {
      await acceptRevision(currentRevision.id);
    }
  }, [currentRevision, acceptRevision]);

  /**
   * Reject the current revision
   */
  const rejectCurrentRevision = useCallback(async () => {
    if (currentRevision) {
      await rejectRevision(currentRevision.id);
    }
  }, [currentRevision, rejectRevision]);

  /**
   * Accept all revisions
   */
  const acceptAllRevisions = useCallback(async () => {
    try {
      await invoke('accept_all_revisions', { docId: documentId });
      setRevisions([]);
      setCurrentRevisionIndex(-1);
      if (onRevisionsChange) {
        onRevisionsChange([]);
      }
    } catch (e) {
      console.error('Failed to accept all revisions:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, onRevisionsChange]);

  /**
   * Reject all revisions
   */
  const rejectAllRevisions = useCallback(async () => {
    try {
      await invoke('reject_all_revisions', { docId: documentId });
      setRevisions([]);
      setCurrentRevisionIndex(-1);
      if (onRevisionsChange) {
        onRevisionsChange([]);
      }
    } catch (e) {
      console.error('Failed to reject all revisions:', e);
      setError(String(e));
      throw e;
    }
  }, [documentId, onRevisionsChange]);

  /**
   * Refresh revisions from backend
   */
  const refreshRevisions = useCallback(async () => {
    try {
      setIsLoading(true);
      const revisionsData = await invoke<Revision[]>('get_revisions', { docId: documentId });
      setRevisions(revisionsData);
      if (onRevisionsChange) {
        onRevisionsChange(revisionsData);
      }
    } catch (e) {
      console.error('Failed to refresh revisions:', e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [documentId, onRevisionsChange]);

  /**
   * Get revisions filtered by type
   */
  const getRevisionsByType = useCallback((type: RevisionType): Revision[] => {
    return revisions.filter(r => r.type === type);
  }, [revisions]);

  /**
   * Get revisions filtered by author
   */
  const getRevisionsByAuthor = useCallback((authorId: string): Revision[] => {
    return revisions.filter(r => r.author.id === authorId);
  }, [revisions]);

  /**
   * Get unique authors from revisions
   */
  const getAuthors = useCallback((): RevisionAuthor[] => {
    const authorMap = new Map<string, RevisionAuthor>();
    for (const revision of revisions) {
      if (!authorMap.has(revision.author.id)) {
        authorMap.set(revision.author.id, revision.author);
      }
    }
    return Array.from(authorMap.values());
  }, [revisions]);

  return {
    isTrackingEnabled,
    markupMode,
    revisions,
    currentRevision,
    currentRevisionIndex,
    isLoading,
    error,
    toggleTracking,
    setMarkupMode,
    navigateToRevision,
    nextRevision,
    previousRevision,
    acceptRevision,
    rejectRevision,
    acceptCurrentRevision,
    rejectCurrentRevision,
    acceptAllRevisions,
    rejectAllRevisions,
    refreshRevisions,
    getRevisionsByType,
    getRevisionsByAuthor,
    getAuthors,
  };
}
