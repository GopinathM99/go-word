/**
 * CommentBubble - Inline marker component for comments in the document margin
 *
 * Features:
 * - Small marker icon positioned in document margin
 * - Shows comment count if multiple comments
 * - Click to open comment in panel
 * - Hover for quick preview
 */

import React, { useState, useCallback, useRef } from 'react';
import { Comment, formatCommentDate, truncateText } from '../hooks/useComments';
import './Comments.css';

// =============================================================================
// Types
// =============================================================================

interface CommentBubbleProps {
  /** Comments at this location */
  comments: Comment[];
  /** Position in pixels from top of document */
  top: number;
  /** Position in pixels from left (optional, defaults to margin) */
  left?: number;
  /** Whether this bubble is currently selected */
  isSelected?: boolean;
  /** Callback when bubble is clicked */
  onClick: (comment: Comment) => void;
  /** Callback when bubble receives focus */
  onFocus?: () => void;
  /** Custom color (defaults to first comment's author color) */
  color?: string;
}

// =============================================================================
// Component
// =============================================================================

export function CommentBubble({
  comments,
  top,
  left,
  isSelected = false,
  onClick,
  onFocus,
  color,
}: CommentBubbleProps) {
  const [showPreview, setShowPreview] = useState(false);
  const bubbleRef = useRef<HTMLButtonElement>(null);
  const previewTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const primaryComment = comments[0];
  const commentCount = comments.length;
  const bubbleColor = color || primaryComment?.author.color || '#2563eb';

  const handleClick = useCallback(() => {
    onClick(primaryComment);
  }, [onClick, primaryComment]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        onClick(primaryComment);
      }
    },
    [onClick, primaryComment]
  );

  const handleMouseEnter = useCallback(() => {
    // Delay showing preview to avoid flickering
    previewTimeoutRef.current = setTimeout(() => {
      setShowPreview(true);
    }, 300);
  }, []);

  const handleMouseLeave = useCallback(() => {
    if (previewTimeoutRef.current) {
      clearTimeout(previewTimeoutRef.current);
      previewTimeoutRef.current = null;
    }
    setShowPreview(false);
  }, []);

  const handleFocus = useCallback(() => {
    setShowPreview(true);
    onFocus?.();
  }, [onFocus]);

  const handleBlur = useCallback(() => {
    setShowPreview(false);
  }, []);

  if (!primaryComment) {
    return null;
  }

  // Determine if any comments are unresolved
  const hasUnresolved = comments.some((c) => !c.resolved);

  return (
    <button
      ref={bubbleRef}
      className={`comment-bubble ${isSelected ? 'selected' : ''}`}
      style={{
        top: `${top}px`,
        left: left !== undefined ? `${left}px` : undefined,
        backgroundColor: hasUnresolved ? bubbleColor : '#9ca3af',
      }}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onFocus={handleFocus}
      onBlur={handleBlur}
      aria-label={`${commentCount} comment${commentCount !== 1 ? 's' : ''} by ${primaryComment.author.name}`}
      aria-expanded={showPreview}
      aria-haspopup="true"
    >
      <CommentBubbleIcon className="comment-bubble-icon" />

      {/* Comment count badge */}
      {commentCount > 1 && (
        <span className="comment-bubble-count" aria-hidden="true">
          {commentCount}
        </span>
      )}

      {/* Preview tooltip */}
      {showPreview && (
        <div
          className="comment-bubble-preview"
          role="tooltip"
          aria-hidden="true"
        >
          {commentCount === 1 ? (
            <>
              <div className="preview-author">
                {primaryComment.author.name} - {formatCommentDate(primaryComment.createdAt)}
              </div>
              <div className="preview-content">
                {truncateText(primaryComment.content, 120)}
              </div>
            </>
          ) : (
            <>
              <div className="preview-author">
                {commentCount} comments
              </div>
              <div className="preview-content">
                Click to view all comments at this location
              </div>
            </>
          )}
        </div>
      )}
    </button>
  );
}

// =============================================================================
// Multiple Bubbles Container
// =============================================================================

interface CommentBubblesContainerProps {
  /** All comments grouped by position */
  commentGroups: Map<string, Comment[]>;
  /** Function to get top position for a comment group */
  getTopPosition: (nodeId: string, offset: number) => number;
  /** Currently selected comment ID */
  selectedCommentId?: string;
  /** Callback when a bubble is clicked */
  onBubbleClick: (comment: Comment) => void;
  /** Container offset from left edge */
  marginLeft?: number;
}

export function CommentBubblesContainer({
  commentGroups,
  getTopPosition,
  selectedCommentId,
  onBubbleClick,
  marginLeft = 10,
}: CommentBubblesContainerProps) {
  const bubbles: React.ReactNode[] = [];

  commentGroups.forEach((comments, key) => {
    const firstComment = comments[0];
    const top = getTopPosition(firstComment.range.startNodeId, firstComment.range.startOffset);
    const isSelected = comments.some((c) => c.id === selectedCommentId);

    bubbles.push(
      <CommentBubble
        key={key}
        comments={comments}
        top={top}
        left={marginLeft}
        isSelected={isSelected}
        onClick={onBubbleClick}
      />
    );
  });

  return (
    <div
      className="comment-bubbles-container"
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '30px',
        height: '100%',
        pointerEvents: 'none',
      }}
      aria-label="Comment markers"
    >
      <div style={{ pointerEvents: 'auto' }}>{bubbles}</div>
    </div>
  );
}

// =============================================================================
// Icons
// =============================================================================

function CommentBubbleIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z" />
    </svg>
  );
}

// =============================================================================
// Utility: Group comments by position
// =============================================================================

export function groupCommentsByPosition(comments: Comment[]): Map<string, Comment[]> {
  const groups = new Map<string, Comment[]>();

  for (const comment of comments) {
    const key = `${comment.range.startNodeId}-${comment.range.startOffset}`;
    const existing = groups.get(key) || [];
    existing.push(comment);
    groups.set(key, existing);
  }

  return groups;
}

export default CommentBubble;
