/**
 * CommentsPanel - Sidebar panel for viewing and managing document comments
 *
 * Features:
 * - List all comments with author info and timestamps
 * - View commented text excerpts
 * - Reply threads with nesting
 * - Resolve/Reopen functionality
 * - Delete comments (own only)
 * - Filter and sort controls
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Comment,
  CommentReply,
  CommentFilter,
  CommentSort,
  SortDirection,
  CommentAuthor,
  formatCommentDate,
  getInitials,
  truncateText,
} from '../hooks/useComments';
import './Comments.css';

// =============================================================================
// Types
// =============================================================================

interface CommentsPanelProps {
  /** Whether the panel is open */
  isOpen: boolean;
  /** Callback to close the panel */
  onClose: () => void;
  /** All comments to display */
  comments: Comment[];
  /** Currently selected comment */
  selectedComment: Comment | null;
  /** Current filter */
  filter: CommentFilter;
  /** Current sort option */
  sort: CommentSort;
  /** Sort direction */
  sortDirection: SortDirection;
  /** Current user (for delete permission) */
  currentUser: CommentAuthor | null;
  /** Loading state */
  isLoading?: boolean;
  /** Error message */
  error?: string | null;
  /** Callback when filter changes */
  onFilterChange: (filter: CommentFilter) => void;
  /** Callback when sort changes */
  onSortChange: (sort: CommentSort) => void;
  /** Callback when sort direction changes */
  onSortDirectionChange: (direction: SortDirection) => void;
  /** Callback when a comment is selected */
  onSelectComment: (comment: Comment | null) => void;
  /** Callback to reply to a comment */
  onReply: (commentId: string, content: string) => Promise<CommentReply | null>;
  /** Callback to resolve a comment */
  onResolve: (commentId: string) => Promise<boolean>;
  /** Callback to reopen a comment */
  onReopen: (commentId: string) => Promise<boolean>;
  /** Callback to delete a comment */
  onDelete: (commentId: string) => Promise<boolean>;
  /** Callback to navigate to a comment */
  onNavigate: (commentId: string) => Promise<boolean>;
  /** Total comment count */
  commentCount: number;
  /** Unresolved comment count */
  unresolvedCount: number;
}

// =============================================================================
// Component
// =============================================================================

export function CommentsPanel({
  isOpen,
  onClose,
  comments,
  selectedComment,
  filter,
  sort,
  sortDirection,
  currentUser,
  isLoading = false,
  error = null,
  onFilterChange,
  onSortChange,
  onSortDirectionChange,
  onSelectComment,
  onReply,
  onResolve,
  onReopen,
  onDelete,
  onNavigate,
  commentCount,
  unresolvedCount,
}: CommentsPanelProps) {
  const panelRef = useRef<HTMLDivElement>(null);

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose]
  );

  // Focus the panel when opened
  useEffect(() => {
    if (isOpen && panelRef.current) {
      panelRef.current.focus();
    }
  }, [isOpen]);

  if (!isOpen) {
    return null;
  }

  return (
    <aside
      ref={panelRef}
      className="comments-panel"
      role="complementary"
      aria-label="Comments"
      tabIndex={-1}
      onKeyDown={handleKeyDown}
    >
      {/* Header */}
      <header className="comments-panel-header">
        <div className="comments-panel-title">
          <h2>Comments</h2>
          {commentCount > 0 && (
            <span
              className={`comments-count-badge ${unresolvedCount > 0 ? 'unresolved' : ''}`}
              aria-label={`${unresolvedCount} unresolved of ${commentCount} total comments`}
            >
              {unresolvedCount > 0 ? unresolvedCount : commentCount}
            </span>
          )}
        </div>
        <button
          className="comments-panel-close"
          onClick={onClose}
          aria-label="Close comments panel"
        >
          <span aria-hidden="true">&times;</span>
        </button>
      </header>

      {/* Controls */}
      <div className="comments-controls">
        <div className="comments-filter">
          <label htmlFor="comment-filter" className="visually-hidden">
            Filter comments
          </label>
          <select
            id="comment-filter"
            value={filter}
            onChange={(e) => onFilterChange(e.target.value as CommentFilter)}
            aria-label="Filter comments"
          >
            <option value="all">All Comments</option>
            <option value="unresolved">Unresolved</option>
            <option value="resolved">Resolved</option>
          </select>
        </div>
        <div className="comments-sort">
          <label htmlFor="comment-sort" className="visually-hidden">
            Sort comments
          </label>
          <select
            id="comment-sort"
            value={sort}
            onChange={(e) => onSortChange(e.target.value as CommentSort)}
            aria-label="Sort comments by"
          >
            <option value="date">Date</option>
            <option value="author">Author</option>
            <option value="position">Position</option>
          </select>
        </div>
        <button
          className="sort-direction-btn"
          onClick={() => onSortDirectionChange(sortDirection === 'asc' ? 'desc' : 'asc')}
          aria-label={`Sort ${sortDirection === 'asc' ? 'descending' : 'ascending'}`}
          title={`Sort ${sortDirection === 'asc' ? 'descending' : 'ascending'}`}
        >
          {sortDirection === 'asc' ? '\u2191' : '\u2193'}
        </button>
      </div>

      {/* Comments List */}
      <div className="comments-list" role="list" aria-label="Comments list">
        {isLoading ? (
          <div className="comments-loading" role="status" aria-label="Loading comments">
            <div className="comments-spinner" aria-hidden="true" />
            <span className="visually-hidden">Loading comments...</span>
          </div>
        ) : error ? (
          <div className="comments-empty" role="alert">
            <p>Error loading comments</p>
            <p className="comments-empty-hint">{error}</p>
          </div>
        ) : comments.length === 0 ? (
          <div className="comments-empty">
            <CommentIcon className="comments-empty-icon" />
            <p>No comments yet</p>
            <p className="comments-empty-hint">
              Select text and click "Add Comment" to start a discussion
            </p>
          </div>
        ) : (
          comments.map((comment) => (
            <CommentCard
              key={comment.id}
              comment={comment}
              isSelected={selectedComment?.id === comment.id}
              currentUser={currentUser}
              onSelect={() => onSelectComment(comment)}
              onReply={onReply}
              onResolve={onResolve}
              onReopen={onReopen}
              onDelete={onDelete}
              onNavigate={onNavigate}
            />
          ))
        )}
      </div>
    </aside>
  );
}

// =============================================================================
// Comment Card Component
// =============================================================================

interface CommentCardProps {
  comment: Comment;
  isSelected: boolean;
  currentUser: CommentAuthor | null;
  onSelect: () => void;
  onReply: (commentId: string, content: string) => Promise<CommentReply | null>;
  onResolve: (commentId: string) => Promise<boolean>;
  onReopen: (commentId: string) => Promise<boolean>;
  onDelete: (commentId: string) => Promise<boolean>;
  onNavigate: (commentId: string) => Promise<boolean>;
}

function CommentCard({
  comment,
  isSelected,
  currentUser,
  onSelect,
  onReply,
  onResolve,
  onReopen,
  onDelete,
  onNavigate,
}: CommentCardProps) {
  const [replyText, setReplyText] = useState('');
  const [isReplying, setIsReplying] = useState(false);
  const [showReplyInput, setShowReplyInput] = useState(false);
  const replyInputRef = useRef<HTMLTextAreaElement>(null);

  const canDelete = currentUser?.id === comment.author.id;

  const handleReplySubmit = useCallback(async () => {
    if (!replyText.trim()) return;

    setIsReplying(true);
    try {
      const reply = await onReply(comment.id, replyText.trim());
      if (reply) {
        setReplyText('');
        setShowReplyInput(false);
      }
    } finally {
      setIsReplying(false);
    }
  }, [comment.id, replyText, onReply]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        handleReplySubmit();
      } else if (e.key === 'Escape') {
        setShowReplyInput(false);
        setReplyText('');
      }
    },
    [handleReplySubmit]
  );

  const handleCardClick = useCallback(() => {
    onSelect();
    onNavigate(comment.id);
  }, [onSelect, onNavigate, comment.id]);

  // Focus reply input when shown
  useEffect(() => {
    if (showReplyInput && replyInputRef.current) {
      replyInputRef.current.focus();
    }
  }, [showReplyInput]);

  return (
    <article
      className={`comment-card ${isSelected ? 'selected' : ''} ${comment.resolved ? 'resolved' : ''}`}
      role="listitem"
      onClick={handleCardClick}
      tabIndex={0}
      aria-selected={isSelected}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          handleCardClick();
        }
      }}
    >
      {/* Comment Header */}
      <header className="comment-header">
        <div
          className="comment-avatar"
          style={{ backgroundColor: comment.author.color }}
          aria-hidden="true"
        >
          {comment.author.avatarUrl ? (
            <img src={comment.author.avatarUrl} alt="" />
          ) : (
            getInitials(comment.author.name)
          )}
        </div>
        <div className="comment-meta">
          <div className="comment-author">{comment.author.name}</div>
          <time className="comment-date" dateTime={new Date(comment.createdAt).toISOString()}>
            {formatCommentDate(comment.createdAt)}
          </time>
        </div>
        <div className="comment-actions" onClick={(e) => e.stopPropagation()}>
          {comment.resolved ? (
            <button
              className="comment-action-btn"
              onClick={() => onReopen(comment.id)}
              title="Reopen comment"
              aria-label="Reopen comment"
            >
              <span aria-hidden="true">&#x21bb;</span>
            </button>
          ) : (
            <button
              className="comment-action-btn"
              onClick={() => onResolve(comment.id)}
              title="Resolve comment"
              aria-label="Resolve comment"
            >
              <span aria-hidden="true">&#x2713;</span>
            </button>
          )}
          <button
            className="comment-action-btn"
            onClick={() => setShowReplyInput(!showReplyInput)}
            title="Reply to comment"
            aria-label="Reply to comment"
            aria-expanded={showReplyInput}
          >
            <span aria-hidden="true">&#x21a9;</span>
          </button>
          {canDelete && (
            <button
              className="comment-action-btn delete"
              onClick={() => onDelete(comment.id)}
              title="Delete comment"
              aria-label="Delete comment"
            >
              <span aria-hidden="true">&#x2715;</span>
            </button>
          )}
        </div>
      </header>

      {/* Resolved Badge */}
      {comment.resolved && comment.resolvedBy && (
        <div className="comment-resolved-badge" role="status">
          <span aria-hidden="true">&#x2713;</span>
          Resolved by {comment.resolvedBy.name}
        </div>
      )}

      {/* Quoted Text */}
      {comment.quotedText && (
        <blockquote className="comment-quoted" style={{ borderColor: comment.author.color }}>
          {truncateText(comment.quotedText, 100)}
        </blockquote>
      )}

      {/* Comment Content */}
      <div className="comment-content">{comment.content}</div>

      {/* Replies */}
      {(comment.replies.length > 0 || showReplyInput) && (
        <div className="comment-replies">
          {comment.replies.map((reply) => (
            <div key={reply.id} className="comment-reply">
              <div className="reply-header">
                <div
                  className="reply-avatar"
                  style={{ backgroundColor: reply.author.color }}
                  aria-hidden="true"
                >
                  {getInitials(reply.author.name)}
                </div>
                <div className="reply-meta">
                  <span className="reply-author">{reply.author.name}</span>
                  <time className="reply-date" dateTime={new Date(reply.createdAt).toISOString()}>
                    {formatCommentDate(reply.createdAt)}
                  </time>
                </div>
              </div>
              <div className="reply-content">{reply.content}</div>
            </div>
          ))}

          {/* Reply Input */}
          {showReplyInput && (
            <div className="reply-input-container" onClick={(e) => e.stopPropagation()}>
              <textarea
                ref={replyInputRef}
                className="reply-input"
                value={replyText}
                onChange={(e) => setReplyText(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Write a reply... (Ctrl+Enter to submit)"
                aria-label="Reply to comment"
                disabled={isReplying}
              />
              <button
                className="reply-submit-btn"
                onClick={handleReplySubmit}
                disabled={!replyText.trim() || isReplying}
                aria-label="Submit reply"
              >
                {isReplying ? '...' : 'Reply'}
              </button>
            </div>
          )}
        </div>
      )}
    </article>
  );
}

// =============================================================================
// Icons
// =============================================================================

function CommentIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  );
}

export default CommentsPanel;
