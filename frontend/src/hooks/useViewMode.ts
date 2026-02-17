/**
 * useViewMode - View mode state management for the document editor
 *
 * Features:
 * - ViewMode enum: PrintLayout, Draft, Outline, WebLayout
 * - Store in document/app settings (localStorage)
 * - Emit view mode change events
 * - Tauri backend integration
 * - Draft and Outline view options
 */

import { useState, useCallback, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ViewMode,
  ViewModeInfo,
  VIEW_MODE_INFO,
  DraftViewOptions,
  OutlineViewOptions,
  DEFAULT_DRAFT_OPTIONS,
  DEFAULT_OUTLINE_OPTIONS,
  getShortcutForPlatform,
} from '../lib/viewModeTypes';

// =============================================================================
// Types
// =============================================================================

export interface UseViewModeOptions {
  /** Initial view mode */
  initialMode?: ViewMode;
  /** Document ID for storage key and backend sync */
  documentId?: string;
  /** Callback when view mode changes */
  onViewModeChange?: (mode: ViewMode) => void;
  /** Whether to persist to localStorage */
  persist?: boolean;
  /** Whether to sync with Tauri backend */
  syncWithBackend?: boolean;
}

export interface UseViewModeReturn {
  /** Current view mode */
  viewMode: ViewMode;
  /** Current view mode info */
  viewModeInfo: ViewModeInfo;
  /** Set view mode */
  setViewMode: (mode: ViewMode) => void;
  /** Draft view options */
  draftOptions: DraftViewOptions;
  /** Set draft view options */
  setDraftOptions: (options: Partial<DraftViewOptions>) => void;
  /** Outline view options */
  outlineOptions: OutlineViewOptions;
  /** Set outline view options */
  setOutlineOptions: (options: Partial<OutlineViewOptions>) => void;
  /** Toggle between print layout and draft */
  toggleDraft: () => void;
  /** Enter outline mode */
  enterOutlineMode: () => void;
  /** Exit to previous mode */
  exitToLastMode: () => void;
  /** Check if currently in a specific mode */
  isMode: (mode: ViewMode) => boolean;
  /** All available view modes */
  availableModes: ViewModeInfo[];
  /** Whether the view shows page breaks */
  showsPageBreaks: boolean;
  /** Whether the view is continuous scroll */
  isContinuous: boolean;
}

// =============================================================================
// Constants
// =============================================================================

const STORAGE_KEY_PREFIX = 'go-word-view-mode-';
const DRAFT_OPTIONS_STORAGE_KEY = 'go-word-draft-options-';
const OUTLINE_OPTIONS_STORAGE_KEY = 'go-word-outline-options-';
const DEFAULT_VIEW_MODE: ViewMode = 'print_layout';

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Get storage key for a document
 */
function getStorageKey(prefix: string, documentId: string | undefined): string {
  return documentId ? `${prefix}${documentId}` : `${prefix}global`;
}

/**
 * Load view mode from localStorage
 */
function loadViewModeFromStorage(documentId: string | undefined): ViewMode | null {
  try {
    const stored = localStorage.getItem(getStorageKey(STORAGE_KEY_PREFIX, documentId));
    if (stored && isValidViewMode(stored)) {
      return stored as ViewMode;
    }
  } catch {
    // localStorage may not be available
  }
  return null;
}

/**
 * Save view mode to localStorage
 */
function saveViewModeToStorage(documentId: string | undefined, mode: ViewMode): void {
  try {
    localStorage.setItem(getStorageKey(STORAGE_KEY_PREFIX, documentId), mode);
  } catch {
    // localStorage may not be available
  }
}

/**
 * Load draft options from localStorage
 */
function loadDraftOptionsFromStorage(documentId: string | undefined): DraftViewOptions | null {
  try {
    const stored = localStorage.getItem(getStorageKey(DRAFT_OPTIONS_STORAGE_KEY, documentId));
    if (stored) {
      return JSON.parse(stored) as DraftViewOptions;
    }
  } catch {
    // localStorage may not be available or JSON parse failed
  }
  return null;
}

/**
 * Save draft options to localStorage
 */
function saveDraftOptionsToStorage(documentId: string | undefined, options: DraftViewOptions): void {
  try {
    localStorage.setItem(getStorageKey(DRAFT_OPTIONS_STORAGE_KEY, documentId), JSON.stringify(options));
  } catch {
    // localStorage may not be available
  }
}

/**
 * Load outline options from localStorage
 */
function loadOutlineOptionsFromStorage(documentId: string | undefined): OutlineViewOptions | null {
  try {
    const stored = localStorage.getItem(getStorageKey(OUTLINE_OPTIONS_STORAGE_KEY, documentId));
    if (stored) {
      return JSON.parse(stored) as OutlineViewOptions;
    }
  } catch {
    // localStorage may not be available or JSON parse failed
  }
  return null;
}

/**
 * Save outline options to localStorage
 */
function saveOutlineOptionsToStorage(documentId: string | undefined, options: OutlineViewOptions): void {
  try {
    localStorage.setItem(getStorageKey(OUTLINE_OPTIONS_STORAGE_KEY, documentId), JSON.stringify(options));
  } catch {
    // localStorage may not be available
  }
}

/**
 * Check if a value is a valid view mode
 */
function isValidViewMode(value: string): value is ViewMode {
  return value === 'print_layout' || value === 'draft' || value === 'outline' || value === 'web_layout';
}

// =============================================================================
// Custom Event for View Mode Changes
// =============================================================================

export const VIEW_MODE_CHANGE_EVENT = 'viewModeChange';

export interface ViewModeChangeEventDetail {
  mode: ViewMode;
  previousMode: ViewMode;
  documentId?: string;
}

/**
 * Dispatch view mode change event
 */
function dispatchViewModeChangeEvent(detail: ViewModeChangeEventDetail): void {
  const event = new CustomEvent(VIEW_MODE_CHANGE_EVENT, { detail });
  window.dispatchEvent(event);
}

/**
 * Subscribe to view mode change events
 */
export function subscribeToViewModeChanges(
  callback: (event: CustomEvent<ViewModeChangeEventDetail>) => void
): () => void {
  const handler = callback as EventListener;
  window.addEventListener(VIEW_MODE_CHANGE_EVENT, handler);
  return () => window.removeEventListener(VIEW_MODE_CHANGE_EVENT, handler);
}

// =============================================================================
// Hook Implementation
// =============================================================================

export function useViewMode(options: UseViewModeOptions = {}): UseViewModeReturn {
  const {
    initialMode = DEFAULT_VIEW_MODE,
    documentId,
    onViewModeChange,
    persist = true,
    syncWithBackend = false,
  } = options;

  // Track previous mode for exit functionality
  const [previousMode, setPreviousMode] = useState<ViewMode>('print_layout');

  // Initialize from storage or use initial mode
  const [viewMode, setViewModeState] = useState<ViewMode>(() => {
    if (persist) {
      const stored = loadViewModeFromStorage(documentId);
      if (stored) return stored;
    }
    return initialMode;
  });

  // Draft view options
  const [draftOptions, setDraftOptionsState] = useState<DraftViewOptions>(() => {
    if (persist) {
      const stored = loadDraftOptionsFromStorage(documentId);
      if (stored) return stored;
    }
    return DEFAULT_DRAFT_OPTIONS;
  });

  // Outline view options
  const [outlineOptions, setOutlineOptionsState] = useState<OutlineViewOptions>(() => {
    if (persist) {
      const stored = loadOutlineOptionsFromStorage(documentId);
      if (stored) return stored;
    }
    return DEFAULT_OUTLINE_OPTIONS;
  });

  // Save to storage when view mode changes
  useEffect(() => {
    if (persist) {
      saveViewModeToStorage(documentId, viewMode);
    }
  }, [documentId, viewMode, persist]);

  // Save draft options to storage
  useEffect(() => {
    if (persist) {
      saveDraftOptionsToStorage(documentId, draftOptions);
    }
  }, [documentId, draftOptions, persist]);

  // Save outline options to storage
  useEffect(() => {
    if (persist) {
      saveOutlineOptionsToStorage(documentId, outlineOptions);
    }
  }, [documentId, outlineOptions, persist]);

  // Sync with backend
  useEffect(() => {
    if (syncWithBackend && documentId) {
      invoke('set_view_mode', { docId: documentId, mode: viewMode }).catch((err) => {
        console.warn('Failed to sync view mode with backend:', err);
      });
    }
  }, [viewMode, documentId, syncWithBackend]);

  // Sync draft options with backend
  useEffect(() => {
    if (syncWithBackend && documentId) {
      invoke('set_draft_options', { docId: documentId, options: draftOptions }).catch((err) => {
        console.warn('Failed to sync draft options with backend:', err);
      });
    }
  }, [draftOptions, documentId, syncWithBackend]);

  // Sync outline options with backend
  useEffect(() => {
    if (syncWithBackend && documentId) {
      invoke('set_outline_options', { docId: documentId, options: outlineOptions }).catch((err) => {
        console.warn('Failed to sync outline options with backend:', err);
      });
    }
  }, [outlineOptions, documentId, syncWithBackend]);

  // Notify parent when view mode changes
  useEffect(() => {
    if (onViewModeChange) {
      onViewModeChange(viewMode);
    }
  }, [viewMode, onViewModeChange]);

  /**
   * Set view mode with event dispatch
   */
  const setViewMode = useCallback(
    (mode: ViewMode) => {
      setViewModeState((current) => {
        if (current !== mode) {
          // Dispatch change event
          dispatchViewModeChangeEvent({
            mode,
            previousMode: current,
            documentId,
          });

          // Track previous mode (excluding outline which is a special mode)
          if (current !== 'outline') {
            setPreviousMode(current);
          }
        }
        return mode;
      });
    },
    [documentId]
  );

  /**
   * Set draft view options (merge with existing)
   */
  const setDraftOptions = useCallback((newOptions: Partial<DraftViewOptions>) => {
    setDraftOptionsState((current) => ({
      ...current,
      ...newOptions,
    }));
  }, []);

  /**
   * Set outline view options (merge with existing)
   */
  const setOutlineOptions = useCallback((newOptions: Partial<OutlineViewOptions>) => {
    setOutlineOptionsState((current) => ({
      ...current,
      ...newOptions,
    }));
  }, []);

  /**
   * Toggle between print layout and draft
   */
  const toggleDraft = useCallback(() => {
    setViewMode(viewMode === 'draft' ? 'print_layout' : 'draft');
  }, [viewMode, setViewMode]);

  /**
   * Enter outline mode
   */
  const enterOutlineMode = useCallback(() => {
    if (viewMode !== 'outline') {
      setPreviousMode(viewMode);
      setViewMode('outline');
    }
  }, [viewMode, setViewMode]);

  /**
   * Exit to previous mode
   */
  const exitToLastMode = useCallback(() => {
    setViewMode(previousMode);
  }, [previousMode, setViewMode]);

  /**
   * Check if currently in a specific mode
   */
  const isMode = useCallback(
    (mode: ViewMode): boolean => viewMode === mode,
    [viewMode]
  );

  // Computed values
  const viewModeInfo = useMemo(() => VIEW_MODE_INFO[viewMode], [viewMode]);
  const availableModes = useMemo(() => Object.values(VIEW_MODE_INFO), []);
  const showsPageBreaks = viewModeInfo.showsPageBreaks;
  const isContinuous = viewModeInfo.isContinuous;

  return {
    viewMode,
    viewModeInfo,
    setViewMode,
    draftOptions,
    setDraftOptions,
    outlineOptions,
    setOutlineOptions,
    toggleDraft,
    enterOutlineMode,
    exitToLastMode,
    isMode,
    availableModes,
    showsPageBreaks,
    isContinuous,
  };
}

// =============================================================================
// Keyboard Shortcut Handler Hook
// =============================================================================

export interface UseViewModeShortcutsOptions {
  setViewMode: (mode: ViewMode) => void;
  enabled?: boolean;
}

/**
 * Hook to handle view mode keyboard shortcuts
 * - Ctrl+Alt+P: Print Layout
 * - Ctrl+Alt+N: Draft (Normal)
 * - Ctrl+Alt+O: Outline
 * - Ctrl+Alt+W: Web Layout
 */
export function useViewModeShortcuts(options: UseViewModeShortcutsOptions): void {
  const { setViewMode, enabled = true } = options;

  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modifierKey = isMac ? e.metaKey : e.ctrlKey;

      if (!modifierKey || !e.altKey) return;

      switch (e.key.toLowerCase()) {
        case 'p':
          e.preventDefault();
          setViewMode('print_layout');
          break;
        case 'n':
          e.preventDefault();
          setViewMode('draft');
          break;
        case 'o':
          e.preventDefault();
          setViewMode('outline');
          break;
        case 'w':
          e.preventDefault();
          setViewMode('web_layout');
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [enabled, setViewMode]);
}

export default useViewMode;
