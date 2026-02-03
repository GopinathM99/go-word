/**
 * useOutline - Hook for managing document outline state
 *
 * Features:
 * - Extract outline from document via Tauri backend
 * - Track current position in outline
 * - Handle expand/collapse state
 * - Navigate to heading on click
 * - Real-time sync with document changes
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  OutlineHeading,
  OutlineState,
  RawOutlineHeading,
  buildHeadingTree,
  findHeadingById,
  getAllHeadingIds,
  getParentIds,
} from '../lib/outlineTypes';

// =============================================================================
// Types
// =============================================================================

export interface UseOutlineOptions {
  /** Document ID to get outline for */
  docId?: string;
  /** Polling interval for outline updates (in ms). Set to 0 to disable polling */
  pollInterval?: number;
  /** Callback when a heading is navigated to */
  onNavigate?: (headingId: string, position: { page: number; offset: number }) => void;
  /** Callback when outline is updated */
  onOutlineChange?: (headings: OutlineHeading[]) => void;
  /** Initial expanded state - 'all', 'none', or 'first-level' */
  initialExpandState?: 'all' | 'none' | 'first-level';
  /** Auto-expand parents when current heading changes */
  autoExpandToCurrentHeading?: boolean;
}

export interface UseOutlineReturn {
  /** Current outline state */
  state: OutlineState;
  /** Whether the outline is loading */
  isLoading: boolean;
  /** Error message if outline fetch failed */
  error: string | null;
  /** Navigate to a specific heading */
  navigateToHeading: (headingId: string) => Promise<void>;
  /** Toggle expand/collapse state for a heading */
  toggleExpanded: (headingId: string) => void;
  /** Expand a specific heading */
  expand: (headingId: string) => void;
  /** Collapse a specific heading */
  collapse: (headingId: string) => void;
  /** Expand all headings */
  expandAll: () => void;
  /** Collapse all headings */
  collapseAll: () => void;
  /** Set the current heading (e.g., based on scroll position) */
  setCurrentHeading: (headingId: string | null) => void;
  /** Refresh the outline from the backend */
  refresh: () => Promise<void>;
  /** Check if a heading is expanded */
  isExpanded: (headingId: string) => boolean;
  /** Check if a heading has children */
  hasChildren: (headingId: string) => boolean;
  /** Get flat list of visible headings (respecting expand state) */
  visibleHeadings: OutlineHeading[];
  /** Total count of headings */
  totalCount: number;
}

// =============================================================================
// Default Values
// =============================================================================

const DEFAULT_STATE: OutlineState = {
  headings: [],
  expandedIds: new Set(),
  currentHeadingId: null,
};

// =============================================================================
// Hook Implementation
// =============================================================================

export function useOutline(options: UseOutlineOptions = {}): UseOutlineReturn {
  const {
    docId = 'default',
    pollInterval = 0,
    onNavigate,
    onOutlineChange,
    initialExpandState = 'first-level',
    autoExpandToCurrentHeading = true,
  } = options;

  const [state, setState] = useState<OutlineState>(DEFAULT_STATE);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const lastDocIdRef = useRef(docId);
  const initializedRef = useRef(false);

  // ==========================================================================
  // Fetch Outline
  // ==========================================================================

  const fetchOutline = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await invoke<{ docId: string; headings: RawOutlineHeading[] }>(
        'get_document_outline',
        { docId }
      );

      const tree = buildHeadingTree(response.headings);

      setState((prev) => {
        // Preserve expanded state when refreshing
        let newExpandedIds = prev.expandedIds;

        // On first load or doc change, set initial expand state
        if (!initializedRef.current || lastDocIdRef.current !== docId) {
          initializedRef.current = true;
          lastDocIdRef.current = docId;

          switch (initialExpandState) {
            case 'all':
              newExpandedIds = new Set(getAllHeadingIds(tree));
              break;
            case 'none':
              newExpandedIds = new Set();
              break;
            case 'first-level':
            default:
              // Expand only first-level headings that have children
              newExpandedIds = new Set(
                tree.filter((h) => h.children.length > 0).map((h) => h.id)
              );
              break;
          }
        }

        return {
          ...prev,
          headings: tree,
          expandedIds: newExpandedIds,
        };
      });

      if (onOutlineChange) {
        onOutlineChange(tree);
      }
    } catch (e) {
      console.error('Failed to fetch document outline:', e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [docId, initialExpandState, onOutlineChange]);

  // ==========================================================================
  // Initial Fetch and Polling
  // ==========================================================================

  // Fetch on mount and when docId changes
  useEffect(() => {
    fetchOutline();
  }, [fetchOutline]);

  // Poll for updates if enabled
  useEffect(() => {
    if (pollInterval <= 0) return;

    const interval = setInterval(fetchOutline, pollInterval);
    return () => clearInterval(interval);
  }, [pollInterval, fetchOutline]);

  // ==========================================================================
  // Navigation
  // ==========================================================================

  const navigateToHeading = useCallback(
    async (headingId: string) => {
      const heading = findHeadingById(state.headings, headingId);
      if (!heading) {
        console.warn(`Heading not found: ${headingId}`);
        return;
      }

      try {
        await invoke('navigate_to_heading', {
          docId,
          headingId,
        });

        setState((prev) => ({
          ...prev,
          currentHeadingId: headingId,
        }));

        if (onNavigate) {
          onNavigate(headingId, heading.position);
        }
      } catch (e) {
        console.error('Failed to navigate to heading:', e);
        setError(String(e));
      }
    },
    [docId, state.headings, onNavigate]
  );

  // ==========================================================================
  // Expand/Collapse
  // ==========================================================================

  const toggleExpanded = useCallback((headingId: string) => {
    setState((prev) => {
      const newExpandedIds = new Set(prev.expandedIds);
      if (newExpandedIds.has(headingId)) {
        newExpandedIds.delete(headingId);
      } else {
        newExpandedIds.add(headingId);
      }
      return {
        ...prev,
        expandedIds: newExpandedIds,
      };
    });
  }, []);

  const expand = useCallback((headingId: string) => {
    setState((prev) => {
      const newExpandedIds = new Set(prev.expandedIds);
      newExpandedIds.add(headingId);
      return {
        ...prev,
        expandedIds: newExpandedIds,
      };
    });
  }, []);

  const collapse = useCallback((headingId: string) => {
    setState((prev) => {
      const newExpandedIds = new Set(prev.expandedIds);
      newExpandedIds.delete(headingId);
      return {
        ...prev,
        expandedIds: newExpandedIds,
      };
    });
  }, []);

  const expandAll = useCallback(() => {
    setState((prev) => ({
      ...prev,
      expandedIds: new Set(getAllHeadingIds(prev.headings)),
    }));
  }, []);

  const collapseAll = useCallback(() => {
    setState((prev) => ({
      ...prev,
      expandedIds: new Set(),
    }));
  }, []);

  // ==========================================================================
  // Current Heading
  // ==========================================================================

  const setCurrentHeading = useCallback(
    (headingId: string | null) => {
      setState((prev) => {
        let newExpandedIds = prev.expandedIds;

        // Auto-expand parents if enabled
        if (autoExpandToCurrentHeading && headingId) {
          const parentIds = getParentIds(prev.headings, headingId);
          if (parentIds && parentIds.length > 0) {
            newExpandedIds = new Set([...prev.expandedIds, ...parentIds]);
          }
        }

        return {
          ...prev,
          currentHeadingId: headingId,
          expandedIds: newExpandedIds,
        };
      });
    },
    [autoExpandToCurrentHeading]
  );

  // ==========================================================================
  // Helper Methods
  // ==========================================================================

  const isExpanded = useCallback(
    (headingId: string): boolean => {
      return state.expandedIds.has(headingId);
    },
    [state.expandedIds]
  );

  const hasChildren = useCallback(
    (headingId: string): boolean => {
      const heading = findHeadingById(state.headings, headingId);
      return heading ? heading.children.length > 0 : false;
    },
    [state.headings]
  );

  // ==========================================================================
  // Computed Values
  // ==========================================================================

  // Get visible headings (respecting expand/collapse state)
  const visibleHeadings = useMemo((): OutlineHeading[] => {
    const result: OutlineHeading[] = [];

    function traverse(headings: OutlineHeading[], depth: number = 0) {
      for (const heading of headings) {
        result.push(heading);
        if (heading.children.length > 0 && state.expandedIds.has(heading.id)) {
          traverse(heading.children, depth + 1);
        }
      }
    }

    traverse(state.headings);
    return result;
  }, [state.headings, state.expandedIds]);

  // Total count of all headings
  const totalCount = useMemo((): number => {
    function count(headings: OutlineHeading[]): number {
      let total = headings.length;
      for (const heading of headings) {
        total += count(heading.children);
      }
      return total;
    }
    return count(state.headings);
  }, [state.headings]);

  // ==========================================================================
  // Return
  // ==========================================================================

  return {
    state,
    isLoading,
    error,
    navigateToHeading,
    toggleExpanded,
    expand,
    collapse,
    expandAll,
    collapseAll,
    setCurrentHeading,
    refresh: fetchOutline,
    isExpanded,
    hasChildren,
    visibleHeadings,
    totalCount,
  };
}

// =============================================================================
// Utility Hooks
// =============================================================================

/**
 * Hook to sync current heading with scroll position
 */
export function useOutlineScrollSync(
  headings: OutlineHeading[],
  onCurrentHeadingChange: (headingId: string | null) => void
): {
  updateCurrentHeading: (scrollPosition: number, pageNumber: number) => void;
} {
  const updateCurrentHeading = useCallback(
    (scrollPosition: number, pageNumber: number) => {
      // Find the heading that best matches the current scroll position
      let currentHeading: OutlineHeading | null = null;
      let bestMatch: OutlineHeading | null = null;

      function findBestMatch(items: OutlineHeading[]) {
        for (const heading of items) {
          if (
            heading.position.page < pageNumber ||
            (heading.position.page === pageNumber &&
              heading.position.offset <= scrollPosition)
          ) {
            bestMatch = heading;
          }
          findBestMatch(heading.children);
        }
      }

      findBestMatch(headings);
      currentHeading = bestMatch;

      onCurrentHeadingChange(currentHeading?.id ?? null);
    },
    [headings, onCurrentHeadingChange]
  );

  return {
    updateCurrentHeading,
  };
}

export default useOutline;
