/**
 * useComments - Hook to manage document comments
 *
 * Features:
 * - Fetch and manage comments from the backend
 * - Add, edit, delete, and reply to comments
 * - Resolve and reopen comments
 * - Navigate to comment locations
 * - Track current selection for new comments
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

// =============================================================================
// Types
// =============================================================================

/**
 * Text range for a comment
 */
export interface CommentRange {
  /** Paragraph/node ID where the comment starts */
  startNodeId: string;
  /** Offset within the start node */
  startOffset: number;
  /** Paragraph/node ID where the comment ends */
  endNodeId: string;
  /** Offset within the end node */
  endOffset: number;
}

/**
 * Comment author information
 */
export interface CommentAuthor {
  /** Unique author ID */
  id: string;
  /** Display name */
  name: string;
  /** Optional email */
  email?: string;
  /** Optional avatar URL */
  avatarUrl?: string;
  /** Author color (for highlights) */
  color: string;
}

/**
 * A single comment or reply
 */
export interface Comment {
  /** Unique comment ID */
  id: string;
  /** Author of the comment */
  author: CommentAuthor;
  /** Comment content/text */
  content: string;
  /** When the comment was created (Unix timestamp ms) */
  createdAt: number;
  /** When the comment was last modified (Unix timestamp ms) */
  modifiedAt: number;
  /** Text range this comment refers to */
  range: CommentRange;
  /** Excerpt of the commented text */
  quotedText: string;
  /** Whether this comment is resolved */
  resolved: boolean;
  /** Who resolved this comment (if resolved) */
  resolvedBy?: CommentAuthor;
  /** When it was resolved (if resolved) */
  resolvedAt?: number;
  /** Replies to this comment */
  replies: CommentReply[];
  /** Parent comment ID (if this is a reply) */
  parentId?: string;
}

/**
 * A reply to a comment
 */
export interface CommentReply {
  /** Unique reply ID */
  id: string;
  /** Author of the reply */
  author: CommentAuthor;
  /** Reply content */
  content: string;
  /** When the reply was created */
  createdAt: number;
  /** When the reply was last modified */
  modifiedAt: number;
  /** Parent comment ID */
  parentCommentId: string;
}

/**
 * Filter options for comments
 */
export type CommentFilter = 'all' | 'resolved' | 'unresolved';

/**
 * Sort options for comments
 */
export type CommentSort = 'date' | 'author' | 'position';

/**
 * Sort direction
 */
export type SortDirection = 'asc' | 'desc';

/**
 * Current text selection for creating new comments
 */
export interface TextSelection {
  range: CommentRange;
  text: string;
}

/**
 * Options for the useComments hook
 */
export interface UseCommentsOptions {
  /** Document ID */
  docId?: string;
  /** Polling interval for updates (in ms) */
  pollInterval?: number;
  /** Whether to enable polling */
  enablePolling?: boolean;
  /** Callback when comments change */
  onCommentsChange?: (comments: Comment[]) => void;
  /** Callback when a comment is selected */
  onCommentSelect?: (comment: Comment | null) => void;
}

/**
 * Return type for the useComments hook
 */
export interface UseCommentsReturn {
  /** All comments in the document */
  comments: Comment[];
  /** Filtered and sorted comments */
  filteredComments: Comment[];
  /** Currently selected comment */
  selectedComment: Comment | null;
  /** Current filter */
  filter: CommentFilter;
  /** Current sort option */
  sort: CommentSort;
  /** Sort direction */
  sortDirection: SortDirection;
  /** Loading state */
  isLoading: boolean;
  /** Error message if any */
  error: string | null;
  /** Current text selection (for new comment) */
  currentSelection: TextSelection | null;
  /** Current user (for determining delete permissions) */
  currentUser: CommentAuthor | null;
  /** Set the filter */
  setFilter: (filter: CommentFilter) => void;
  /** Set the sort option */
  setSort: (sort: CommentSort) => void;
  /** Set the sort direction */
  setSortDirection: (direction: SortDirection) => void;
  /** Select a comment */
  selectComment: (comment: Comment | null) => void;
  /** Set the current text selection */
  setCurrentSelection: (selection: TextSelection | null) => void;
  /** Refresh comments from backend */
  refreshComments: () => Promise<void>;
  /** Add a new comment */
  addComment: (content: string) => Promise<Comment | null>;
  /** Edit a comment */
  editComment: (commentId: string, content: string) => Promise<boolean>;
  /** Delete a comment */
  deleteComment: (commentId: string) => Promise<boolean>;
  /** Reply to a comment */
  replyToComment: (commentId: string, content: string) => Promise<CommentReply | null>;
  /** Resolve a comment */
  resolveComment: (commentId: string) => Promise<boolean>;
  /** Reopen a resolved comment */
  reopenComment: (commentId: string) => Promise<boolean>;
  /** Navigate to a comment's location in the document */
  navigateToComment: (commentId: string) => Promise<boolean>;
  /** Get comment count */
  commentCount: number;
  /** Get unresolved comment count */
  unresolvedCount: number;
}

// =============================================================================
// Default Values
// =============================================================================

const DEFAULT_AUTHOR: CommentAuthor = {
  id: 'current-user',
  name: 'Current User',
  color: '#2563eb',
};

// =============================================================================
// Hook Implementation
// =============================================================================

export function useComments(options: UseCommentsOptions = {}): UseCommentsReturn {
  const {
    docId = 'default',
    pollInterval = 30000,
    enablePolling = false,
    onCommentsChange,
    onCommentSelect,
  } = options;

  const [comments, setComments] = useState<Comment[]>([]);
  const [selectedComment, setSelectedComment] = useState<Comment | null>(null);
  const [filter, setFilter] = useState<CommentFilter>('all');
  const [sort, setSort] = useState<CommentSort>('date');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentSelection, setCurrentSelection] = useState<TextSelection | null>(null);
  const [currentUser, setCurrentUser] = useState<CommentAuthor | null>(DEFAULT_AUTHOR);

  const previousCommentsRef = useRef<Comment[]>([]);

  // Fetch comments from backend
  const refreshComments = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const fetchedComments = await invoke<Comment[]>('get_comments', { docId });
      setComments(fetchedComments);
      previousCommentsRef.current = fetchedComments;
    } catch (e) {
      console.error('Failed to fetch comments:', e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [docId]);

  // Fetch current user
  const fetchCurrentUser = useCallback(async () => {
    try {
      const user = await invoke<CommentAuthor>('get_current_user');
      setCurrentUser(user);
    } catch (e) {
      // Use default user if not available
      console.warn('Could not fetch current user, using default:', e);
      setCurrentUser(DEFAULT_AUTHOR);
    }
  }, []);

  // Initial fetch
  useEffect(() => {
    refreshComments();
    fetchCurrentUser();
  }, [refreshComments, fetchCurrentUser]);

  // Polling for updates
  useEffect(() => {
    if (!enablePolling || pollInterval <= 0) return;

    const interval = setInterval(() => {
      refreshComments();
    }, pollInterval);

    return () => clearInterval(interval);
  }, [enablePolling, pollInterval, refreshComments]);

  // Notify on comments change
  useEffect(() => {
    if (onCommentsChange) {
      onCommentsChange(comments);
    }
  }, [comments, onCommentsChange]);

  // Notify on selection change
  useEffect(() => {
    if (onCommentSelect) {
      onCommentSelect(selectedComment);
    }
  }, [selectedComment, onCommentSelect]);

  // Filter and sort comments
  const filteredComments = useCallback(() => {
    let result = [...comments];

    // Apply filter
    switch (filter) {
      case 'resolved':
        result = result.filter((c) => c.resolved);
        break;
      case 'unresolved':
        result = result.filter((c) => !c.resolved);
        break;
      // 'all' - no filter
    }

    // Apply sort
    result.sort((a, b) => {
      let comparison = 0;
      switch (sort) {
        case 'date':
          comparison = a.createdAt - b.createdAt;
          break;
        case 'author':
          comparison = a.author.name.localeCompare(b.author.name);
          break;
        case 'position':
          // Sort by node ID first, then offset
          comparison = a.range.startNodeId.localeCompare(b.range.startNodeId);
          if (comparison === 0) {
            comparison = a.range.startOffset - b.range.startOffset;
          }
          break;
      }
      return sortDirection === 'desc' ? -comparison : comparison;
    });

    return result;
  }, [comments, filter, sort, sortDirection]);

  // Select a comment
  const selectComment = useCallback((comment: Comment | null) => {
    setSelectedComment(comment);
  }, []);

  // Add a new comment
  const addComment = useCallback(
    async (content: string): Promise<Comment | null> => {
      if (!currentSelection) {
        setError('No text selected');
        return null;
      }

      try {
        const newComment = await invoke<Comment>('add_comment', {
          docId,
          range: currentSelection.range,
          content,
        });
        setComments((prev) => [...prev, newComment]);
        setCurrentSelection(null);
        return newComment;
      } catch (e) {
        console.error('Failed to add comment:', e);
        setError(String(e));
        return null;
      }
    },
    [docId, currentSelection]
  );

  // Edit a comment
  const editComment = useCallback(
    async (commentId: string, content: string): Promise<boolean> => {
      try {
        await invoke('edit_comment', { commentId, content });
        setComments((prev) =>
          prev.map((c) =>
            c.id === commentId
              ? { ...c, content, modifiedAt: Date.now() }
              : c
          )
        );
        return true;
      } catch (e) {
        console.error('Failed to edit comment:', e);
        setError(String(e));
        return false;
      }
    },
    []
  );

  // Delete a comment
  const deleteComment = useCallback(
    async (commentId: string): Promise<boolean> => {
      try {
        await invoke('delete_comment', { commentId });
        setComments((prev) => prev.filter((c) => c.id !== commentId));
        if (selectedComment?.id === commentId) {
          setSelectedComment(null);
        }
        return true;
      } catch (e) {
        console.error('Failed to delete comment:', e);
        setError(String(e));
        return false;
      }
    },
    [selectedComment]
  );

  // Reply to a comment
  const replyToComment = useCallback(
    async (commentId: string, content: string): Promise<CommentReply | null> => {
      try {
        const reply = await invoke<CommentReply>('reply_to_comment', {
          commentId,
          content,
        });
        setComments((prev) =>
          prev.map((c) =>
            c.id === commentId
              ? { ...c, replies: [...c.replies, reply] }
              : c
          )
        );
        return reply;
      } catch (e) {
        console.error('Failed to reply to comment:', e);
        setError(String(e));
        return null;
      }
    },
    []
  );

  // Resolve a comment
  const resolveComment = useCallback(
    async (commentId: string): Promise<boolean> => {
      try {
        await invoke('resolve_comment', { commentId });
        setComments((prev) =>
          prev.map((c) =>
            c.id === commentId
              ? {
                  ...c,
                  resolved: true,
                  resolvedBy: currentUser || DEFAULT_AUTHOR,
                  resolvedAt: Date.now(),
                }
              : c
          )
        );
        return true;
      } catch (e) {
        console.error('Failed to resolve comment:', e);
        setError(String(e));
        return false;
      }
    },
    [currentUser]
  );

  // Reopen a comment
  const reopenComment = useCallback(
    async (commentId: string): Promise<boolean> => {
      try {
        await invoke('reopen_comment', { commentId });
        setComments((prev) =>
          prev.map((c) =>
            c.id === commentId
              ? {
                  ...c,
                  resolved: false,
                  resolvedBy: undefined,
                  resolvedAt: undefined,
                }
              : c
          )
        );
        return true;
      } catch (e) {
        console.error('Failed to reopen comment:', e);
        setError(String(e));
        return false;
      }
    },
    []
  );

  // Navigate to a comment's location in the document
  const navigateToComment = useCallback(
    async (commentId: string): Promise<boolean> => {
      try {
        await invoke('navigate_to_comment', { commentId });
        return true;
      } catch (e) {
        console.error('Failed to navigate to comment:', e);
        setError(String(e));
        return false;
      }
    },
    []
  );

  // Computed values
  const commentCount = comments.length;
  const unresolvedCount = comments.filter((c) => !c.resolved).length;

  return {
    comments,
    filteredComments: filteredComments(),
    selectedComment,
    filter,
    sort,
    sortDirection,
    isLoading,
    error,
    currentSelection,
    currentUser,
    setFilter,
    setSort,
    setSortDirection,
    selectComment,
    setCurrentSelection,
    refreshComments,
    addComment,
    editComment,
    deleteComment,
    replyToComment,
    resolveComment,
    reopenComment,
    navigateToComment,
    commentCount,
    unresolvedCount,
  };
}

// =============================================================================
// Utility Functions
// =============================================================================

/**
 * Format a timestamp for display
 */
export function formatCommentDate(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) {
    return 'Just now';
  } else if (diffMins < 60) {
    return `${diffMins}m ago`;
  } else if (diffHours < 24) {
    return `${diffHours}h ago`;
  } else if (diffDays < 7) {
    return `${diffDays}d ago`;
  } else {
    return date.toLocaleDateString();
  }
}

/**
 * Get initials from a name
 */
export function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 1) {
    return parts[0].substring(0, 2).toUpperCase();
  }
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

/**
 * Generate a color from a string (for consistent author colors)
 */
export function stringToColor(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = hash % 360;
  return `hsl(${hue}, 70%, 45%)`;
}

/**
 * Truncate text to a maximum length
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text;
  }
  return text.substring(0, maxLength - 3) + '...';
}

export default useComments;
