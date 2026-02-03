import { useState, useCallback, useEffect, useRef } from 'react';

interface VersionInfo {
  id: string;
  timestamp: Date;
  author: string;
  name?: string;
  summary: string;
  isNamed: boolean;
  isCurrent: boolean;
}

interface DiffContent {
  type: 'unchanged' | 'added' | 'removed' | 'modified';
  content: string;
  location?: string;
}

interface UseVersionHistoryOptions {
  documentId: string;
  onFetchVersions?: (docId: string) => Promise<VersionInfo[]>;
  onPreviewVersion?: (docId: string, versionId: string) => Promise<void>;
  onRestoreVersion?: (docId: string, versionId: string) => Promise<void>;
  onGetDiff?: (docId: string, fromId: string, toId: string) => Promise<DiffContent[]>;
  onRenameVersion?: (docId: string, versionId: string, name: string) => Promise<void>;
  onCreateSnapshot?: (docId: string, name: string) => Promise<void>;
  autoRefresh?: boolean;
  autoRefreshInterval?: number;
}

interface UseVersionHistoryReturn {
  versions: VersionInfo[];
  isLoading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
  preview: (versionId: string) => Promise<void>;
  restore: (versionId: string) => Promise<void>;
  compare: (fromId: string, toId: string) => Promise<void>;
  rename: (versionId: string, name: string) => Promise<void>;
  createSnapshot: (name: string) => Promise<void>;
  diffContent: DiffContent[];
  comparingVersions: [VersionInfo, VersionInfo] | null;
  closeDiff: () => void;
  previewingVersionId: string | null;
  cancelPreview: () => void;
  isRestoring: boolean;
  isComparing: boolean;
  currentVersion: VersionInfo | undefined;
}

export function useVersionHistory(options: UseVersionHistoryOptions): UseVersionHistoryReturn {
  const {
    documentId,
    onFetchVersions,
    onPreviewVersion,
    onRestoreVersion,
    onGetDiff,
    onRenameVersion,
    onCreateSnapshot,
    autoRefresh = false,
    autoRefreshInterval = 30000,
  } = options;

  const [versions, setVersions] = useState<VersionInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [diffContent, setDiffContent] = useState<DiffContent[]>([]);
  const [comparingVersions, setComparingVersions] = useState<[VersionInfo, VersionInfo] | null>(null);
  const [previewingVersionId, setPreviewingVersionId] = useState<string | null>(null);
  const [isRestoring, setIsRestoring] = useState(false);
  const [isComparing, setIsComparing] = useState(false);

  const mountedRef = useRef(true);
  const refreshTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Cleanup on unmount
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current);
      }
    };
  }, []);

  const refresh = useCallback(async () => {
    if (!onFetchVersions) return;

    setIsLoading(true);
    setError(null);

    try {
      const data = await onFetchVersions(documentId);
      if (mountedRef.current) {
        // Ensure timestamps are Date objects
        const processedData = data.map((v) => ({
          ...v,
          timestamp: v.timestamp instanceof Date ? v.timestamp : new Date(v.timestamp),
        }));
        setVersions(processedData);
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error('Failed to fetch versions'));
      }
    } finally {
      if (mountedRef.current) {
        setIsLoading(false);
      }
    }
  }, [documentId, onFetchVersions]);

  // Initial fetch and auto-refresh
  useEffect(() => {
    refresh();

    if (autoRefresh && autoRefreshInterval > 0) {
      const scheduleRefresh = () => {
        refreshTimeoutRef.current = setTimeout(async () => {
          await refresh();
          if (mountedRef.current) {
            scheduleRefresh();
          }
        }, autoRefreshInterval);
      };

      scheduleRefresh();

      return () => {
        if (refreshTimeoutRef.current) {
          clearTimeout(refreshTimeoutRef.current);
        }
      };
    }
  }, [refresh, autoRefresh, autoRefreshInterval]);

  const preview = useCallback(async (versionId: string) => {
    if (!onPreviewVersion) return;

    setPreviewingVersionId(versionId);
    try {
      await onPreviewVersion(documentId, versionId);
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error('Failed to preview version'));
        setPreviewingVersionId(null);
      }
    }
  }, [documentId, onPreviewVersion]);

  const cancelPreview = useCallback(() => {
    setPreviewingVersionId(null);
  }, []);

  const restore = useCallback(async (versionId: string) => {
    if (!onRestoreVersion) return;

    setIsRestoring(true);
    setError(null);

    try {
      await onRestoreVersion(documentId, versionId);
      if (mountedRef.current) {
        setPreviewingVersionId(null);
        await refresh();
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error('Failed to restore version'));
      }
    } finally {
      if (mountedRef.current) {
        setIsRestoring(false);
      }
    }
  }, [documentId, onRestoreVersion, refresh]);

  const compare = useCallback(async (fromId: string, toId: string) => {
    if (!onGetDiff) return;

    setIsComparing(true);
    setError(null);

    try {
      const diff = await onGetDiff(documentId, fromId, toId);

      if (mountedRef.current) {
        setDiffContent(diff);
        const from = versions.find((v) => v.id === fromId);
        const to = versions.find((v) => v.id === toId);
        if (from && to) {
          setComparingVersions([from, to]);
        }
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error('Failed to compare versions'));
      }
    } finally {
      if (mountedRef.current) {
        setIsComparing(false);
      }
    }
  }, [documentId, onGetDiff, versions]);

  const rename = useCallback(async (versionId: string, name: string) => {
    if (!onRenameVersion) return;

    setError(null);

    try {
      await onRenameVersion(documentId, versionId, name);
      if (mountedRef.current) {
        // Optimistically update the local state
        setVersions((prev) =>
          prev.map((v) =>
            v.id === versionId
              ? { ...v, name, isNamed: true }
              : v
          )
        );
        await refresh();
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error('Failed to rename version'));
        // Revert optimistic update on error
        await refresh();
      }
    }
  }, [documentId, onRenameVersion, refresh]);

  const createSnapshot = useCallback(async (name: string) => {
    if (!onCreateSnapshot) return;

    setError(null);

    try {
      await onCreateSnapshot(documentId, name);
      if (mountedRef.current) {
        await refresh();
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err : new Error('Failed to create snapshot'));
      }
    }
  }, [documentId, onCreateSnapshot, refresh]);

  const closeDiff = useCallback(() => {
    setDiffContent([]);
    setComparingVersions(null);
  }, []);

  const currentVersion = versions.find((v) => v.isCurrent);

  return {
    versions,
    isLoading,
    error,
    refresh,
    preview,
    restore,
    compare,
    rename,
    createSnapshot,
    diffContent,
    comparingVersions,
    closeDiff,
    previewingVersionId,
    cancelPreview,
    isRestoring,
    isComparing,
    currentVersion,
  };
}

// Type exports for consumers
export type { VersionInfo, DiffContent, UseVersionHistoryOptions, UseVersionHistoryReturn };
