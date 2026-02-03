/**
 * useAutosave - Hook to manage autosave state
 *
 * Features:
 * - Track autosave status from the backend
 * - Manage autosave configuration
 * - Provide methods to mark document as dirty
 * - Check for recovery files on startup
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  AutosaveStatus,
  AutosaveConfig,
  RecoveryFile,
  SaveState,
  getSaveState,
} from '../lib/types';

// =============================================================================
// Types
// =============================================================================

export interface UseAutosaveOptions {
  /** Document ID */
  documentId?: string;
  /** Whether autosave is enabled */
  enabled?: boolean;
  /** Polling interval for status updates (in ms) */
  pollInterval?: number;
  /** Callback when status changes */
  onStatusChange?: (status: AutosaveStatus) => void;
  /** Callback when save state changes */
  onSaveStateChange?: (state: SaveState) => void;
}

export interface UseAutosaveReturn {
  /** Current autosave status */
  status: AutosaveStatus;
  /** Current save state */
  saveState: SaveState;
  /** Current autosave configuration */
  config: AutosaveConfig;
  /** Mark the document as dirty (has unsaved changes) */
  markDirty: () => void;
  /** Mark the document as clean (saved) */
  markClean: () => void;
  /** Update autosave configuration */
  setConfig: (config: AutosaveConfig) => Promise<void>;
  /** Check for recovery files */
  checkForRecovery: () => Promise<RecoveryFile[]>;
  /** Whether there are recovery files */
  hasRecoveryFiles: boolean;
  /** Recovery files available */
  recoveryFiles: RecoveryFile[];
  /** Refresh status from backend */
  refreshStatus: () => Promise<void>;
}

// =============================================================================
// Default Values
// =============================================================================

const DEFAULT_STATUS: AutosaveStatus = {
  enabled: true,
  hasUnsavedChanges: false,
  isSaving: false,
  lastSaveTime: null,
  lastError: null,
  nextSaveInSecs: null,
};

const DEFAULT_CONFIG: AutosaveConfig = {
  enabled: true,
  intervalSecs: 300, // 5 minutes
  maxVersions: 5,
};

// =============================================================================
// Hook Implementation
// =============================================================================

export function useAutosave(options: UseAutosaveOptions = {}): UseAutosaveReturn {
  const {
    documentId,
    enabled = true,
    pollInterval = 5000,
    onStatusChange,
    onSaveStateChange,
  } = options;

  const [status, setStatus] = useState<AutosaveStatus>(DEFAULT_STATUS);
  const [config, setConfigState] = useState<AutosaveConfig>(DEFAULT_CONFIG);
  const [recoveryFiles, setRecoveryFiles] = useState<RecoveryFile[]>([]);
  const [hasRecoveryFiles, setHasRecoveryFiles] = useState(false);

  const lastSaveStateRef = useRef<SaveState>('saved');

  // Fetch initial status and config
  useEffect(() => {
    if (!enabled) return;

    const fetchInitial = async () => {
      try {
        const [statusResult, configResult] = await Promise.all([
          invoke<AutosaveStatus>('get_autosave_status'),
          invoke<AutosaveConfig>('get_autosave_config'),
        ]);
        setStatus(statusResult);
        setConfigState(configResult);
      } catch (e) {
        console.error('Failed to fetch autosave status:', e);
      }
    };

    fetchInitial();
  }, [enabled]);

  // Poll for status updates
  useEffect(() => {
    if (!enabled || pollInterval <= 0) return;

    const interval = setInterval(async () => {
      try {
        const statusResult = await invoke<AutosaveStatus>('get_autosave_status');
        setStatus(statusResult);
      } catch (e) {
        console.error('Failed to poll autosave status:', e);
      }
    }, pollInterval);

    return () => clearInterval(interval);
  }, [enabled, pollInterval]);

  // Notify on status change
  useEffect(() => {
    if (onStatusChange) {
      onStatusChange(status);
    }

    const newSaveState = getSaveState(status);
    if (newSaveState !== lastSaveStateRef.current) {
      lastSaveStateRef.current = newSaveState;
      if (onSaveStateChange) {
        onSaveStateChange(newSaveState);
      }
    }
  }, [status, onStatusChange, onSaveStateChange]);

  // Check for recovery files on mount
  useEffect(() => {
    if (!enabled) return;

    const checkRecovery = async () => {
      try {
        const hasFiles = await invoke<boolean>('has_recovery_files');
        setHasRecoveryFiles(hasFiles);

        if (hasFiles) {
          const files = await invoke<RecoveryFile[]>('get_recovery_files');
          setRecoveryFiles(files);
        }
      } catch (e) {
        console.error('Failed to check for recovery files:', e);
      }
    };

    checkRecovery();
  }, [enabled]);

  /**
   * Mark the document as dirty (has unsaved changes)
   */
  const markDirty = useCallback(() => {
    setStatus((prev) => ({
      ...prev,
      hasUnsavedChanges: true,
    }));
  }, []);

  /**
   * Mark the document as clean (saved)
   */
  const markClean = useCallback(() => {
    setStatus((prev) => ({
      ...prev,
      hasUnsavedChanges: false,
      lastSaveTime: Date.now(),
      lastError: null,
    }));
  }, []);

  /**
   * Update autosave configuration
   */
  const setConfig = useCallback(async (newConfig: AutosaveConfig) => {
    try {
      await invoke('set_autosave_config', { config: newConfig });
      setConfigState(newConfig);
      setStatus((prev) => ({
        ...prev,
        enabled: newConfig.enabled,
      }));
    } catch (e) {
      console.error('Failed to update autosave config:', e);
      throw e;
    }
  }, []);

  /**
   * Check for recovery files
   */
  const checkForRecovery = useCallback(async (): Promise<RecoveryFile[]> => {
    try {
      const files = await invoke<RecoveryFile[]>('get_recovery_files');
      setRecoveryFiles(files);
      setHasRecoveryFiles(files.length > 0);
      return files;
    } catch (e) {
      console.error('Failed to check for recovery files:', e);
      return [];
    }
  }, []);

  /**
   * Refresh status from backend
   */
  const refreshStatus = useCallback(async () => {
    try {
      const statusResult = await invoke<AutosaveStatus>('get_autosave_status');
      setStatus(statusResult);
    } catch (e) {
      console.error('Failed to refresh autosave status:', e);
    }
  }, []);

  const saveState = getSaveState(status);

  return {
    status,
    saveState,
    config,
    markDirty,
    markClean,
    setConfig,
    checkForRecovery,
    hasRecoveryFiles,
    recoveryFiles,
    refreshStatus,
  };
}

// =============================================================================
// Utility Hooks
// =============================================================================

/**
 * Hook to check for recovery files on startup and show dialog
 */
export function useRecoveryCheck(): {
  hasRecoveryFiles: boolean;
  recoveryFiles: RecoveryFile[];
  checkComplete: boolean;
  refresh: () => Promise<void>;
} {
  const [hasRecoveryFiles, setHasRecoveryFiles] = useState(false);
  const [recoveryFiles, setRecoveryFiles] = useState<RecoveryFile[]>([]);
  const [checkComplete, setCheckComplete] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const hasFiles = await invoke<boolean>('has_recovery_files');
      setHasRecoveryFiles(hasFiles);

      if (hasFiles) {
        const files = await invoke<RecoveryFile[]>('get_recovery_files');
        setRecoveryFiles(files);
      } else {
        setRecoveryFiles([]);
      }
    } catch (e) {
      console.error('Failed to check for recovery files:', e);
    } finally {
      setCheckComplete(true);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    hasRecoveryFiles,
    recoveryFiles,
    checkComplete,
    refresh,
  };
}

/**
 * Hook to track dirty state with debouncing
 */
export function useDirtyState(
  onChange?: (isDirty: boolean) => void
): {
  isDirty: boolean;
  markDirty: () => void;
  markClean: () => void;
} {
  const [isDirty, setIsDirty] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  const markDirty = useCallback(() => {
    // Debounce rapid changes
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    timeoutRef.current = setTimeout(() => {
      setIsDirty(true);
      if (onChange) {
        onChange(true);
      }
    }, 100);
  }, [onChange]);

  const markClean = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setIsDirty(false);
    if (onChange) {
      onChange(false);
    }
  }, [onChange]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return {
    isDirty,
    markDirty,
    markClean,
  };
}
