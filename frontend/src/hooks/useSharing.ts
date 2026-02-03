import { useState, useCallback, useEffect, useRef } from 'react';

type PermissionLevel = 'viewer' | 'commenter' | 'editor' | 'owner';

interface Collaborator {
  userId: string;
  email: string;
  displayName: string;
  avatar?: string;
  permission: PermissionLevel;
  status: 'accepted' | 'pending';
}

interface ShareLink {
  id: string;
  url: string;
  permission: PermissionLevel;
  createdAt: Date;
  expiresAt?: Date;
  requiresPassword: boolean;
}

interface LinkOptions {
  expiresInDays?: number;
  password?: string;
}

interface UseSharingOptions {
  documentId: string;
  // Callbacks to backend
  onAddCollaborator?: (docId: string, email: string, permission: string) => Promise<Collaborator>;
  onRemoveCollaborator?: (docId: string, userId: string) => Promise<void>;
  onChangePermission?: (docId: string, userId: string, permission: string) => Promise<void>;
  onCreateLink?: (docId: string, permission: string, options: LinkOptions) => Promise<ShareLink>;
  onRevokeLink?: (docId: string, linkId: string) => Promise<void>;
  onFetchCollaborators?: (docId: string) => Promise<Collaborator[]>;
  onFetchLinks?: (docId: string) => Promise<ShareLink[]>;
  // Configuration
  autoRefresh?: boolean;
  refreshInterval?: number;
}

interface UseSharingReturn {
  collaborators: Collaborator[];
  shareLinks: ShareLink[];
  isLoading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
  addCollaborator: (email: string, permission: PermissionLevel) => Promise<Collaborator | undefined>;
  removeCollaborator: (userId: string) => Promise<void>;
  changePermission: (userId: string, permission: PermissionLevel) => Promise<void>;
  createLink: (permission: PermissionLevel, options?: LinkOptions) => Promise<ShareLink | undefined>;
  revokeLink: (linkId: string) => Promise<void>;
  clearError: () => void;
}

export function useSharing(options: UseSharingOptions): UseSharingReturn {
  const {
    documentId,
    autoRefresh = false,
    refreshInterval = 30000
  } = options;

  const [collaborators, setCollaborators] = useState<Collaborator[]>([]);
  const [shareLinks, setShareLinks] = useState<ShareLink[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Store options in ref to avoid stale closures
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const refresh = useCallback(async () => {
    const currentOptions = optionsRef.current;
    setIsLoading(true);
    setError(null);

    try {
      const [collabsResult, linksResult] = await Promise.all([
        currentOptions.onFetchCollaborators?.(documentId),
        currentOptions.onFetchLinks?.(documentId),
      ]);

      if (collabsResult) {
        setCollaborators(collabsResult);
      }
      if (linksResult) {
        setShareLinks(linksResult);
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to fetch sharing info');
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [documentId]);

  const addCollaborator = useCallback(async (
    email: string,
    permission: PermissionLevel
  ): Promise<Collaborator | undefined> => {
    const currentOptions = optionsRef.current;

    if (!email.trim()) {
      throw new Error('Email is required');
    }

    // Basic email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email.trim())) {
      throw new Error('Invalid email address');
    }

    // Check if already a collaborator
    const existingCollab = collaborators.find(
      (c) => c.email.toLowerCase() === email.toLowerCase()
    );
    if (existingCollab) {
      throw new Error('This person already has access to the document');
    }

    setIsLoading(true);
    setError(null);

    try {
      if (currentOptions.onAddCollaborator) {
        const newCollab = await currentOptions.onAddCollaborator(
          documentId,
          email.trim(),
          permission
        );
        setCollaborators((prev) => [...prev, newCollab]);
        return newCollab;
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to add collaborator');
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [documentId, collaborators]);

  const removeCollaborator = useCallback(async (userId: string) => {
    const currentOptions = optionsRef.current;

    const collab = collaborators.find((c) => c.userId === userId);
    if (!collab) {
      throw new Error('Collaborator not found');
    }

    if (collab.permission === 'owner') {
      throw new Error('Cannot remove the document owner');
    }

    setIsLoading(true);
    setError(null);

    try {
      if (currentOptions.onRemoveCollaborator) {
        await currentOptions.onRemoveCollaborator(documentId, userId);
        setCollaborators((prev) => prev.filter((c) => c.userId !== userId));
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to remove collaborator');
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [documentId, collaborators]);

  const changePermission = useCallback(async (
    userId: string,
    permission: PermissionLevel
  ) => {
    const currentOptions = optionsRef.current;

    const collab = collaborators.find((c) => c.userId === userId);
    if (!collab) {
      throw new Error('Collaborator not found');
    }

    if (collab.permission === 'owner') {
      throw new Error('Cannot change owner permissions');
    }

    if (collab.permission === permission) {
      return; // No change needed
    }

    setIsLoading(true);
    setError(null);

    try {
      if (currentOptions.onChangePermission) {
        await currentOptions.onChangePermission(documentId, userId, permission);
        setCollaborators((prev) =>
          prev.map((c) =>
            c.userId === userId ? { ...c, permission } : c
          )
        );
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to change permission');
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [documentId, collaborators]);

  const createLink = useCallback(async (
    permission: PermissionLevel,
    linkOptions: LinkOptions = {}
  ): Promise<ShareLink | undefined> => {
    const currentOptions = optionsRef.current;

    // Validate options
    if (linkOptions.expiresInDays !== undefined && linkOptions.expiresInDays <= 0) {
      throw new Error('Expiry days must be a positive number');
    }

    if (linkOptions.password !== undefined && linkOptions.password.length < 4) {
      throw new Error('Password must be at least 4 characters');
    }

    setIsLoading(true);
    setError(null);

    try {
      if (currentOptions.onCreateLink) {
        const link = await currentOptions.onCreateLink(documentId, permission, linkOptions);
        setShareLinks((prev) => [...prev, link]);
        return link;
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to create link');
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [documentId]);

  const revokeLink = useCallback(async (linkId: string) => {
    const currentOptions = optionsRef.current;

    const link = shareLinks.find((l) => l.id === linkId);
    if (!link) {
      throw new Error('Link not found');
    }

    setIsLoading(true);
    setError(null);

    try {
      if (currentOptions.onRevokeLink) {
        await currentOptions.onRevokeLink(documentId, linkId);
        setShareLinks((prev) => prev.filter((l) => l.id !== linkId));
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to revoke link');
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [documentId, shareLinks]);

  // Auto-refresh effect
  useEffect(() => {
    if (!autoRefresh) return;

    const intervalId = setInterval(() => {
      refresh().catch(() => {
        // Error is already handled in refresh
      });
    }, refreshInterval);

    return () => clearInterval(intervalId);
  }, [autoRefresh, refreshInterval, refresh]);

  // Initial fetch when documentId changes
  useEffect(() => {
    refresh().catch(() => {
      // Error is already handled in refresh
    });
  }, [documentId]); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    collaborators,
    shareLinks,
    isLoading,
    error,
    refresh,
    addCollaborator,
    removeCollaborator,
    changePermission,
    createLink,
    revokeLink,
    clearError,
  };
}

export type {
  Collaborator,
  ShareLink,
  LinkOptions,
  PermissionLevel,
  UseSharingOptions,
  UseSharingReturn
};
