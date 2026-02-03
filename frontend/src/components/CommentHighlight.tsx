/**
 * CommentHighlight - Component to highlight commented text in the document
 *
 * Features:
 * - Highlights text that has comments
 * - Different colors per author
 * - Click highlight to select comment
 * - Visual distinction for resolved comments
 */

import React, { useCallback, useMemo } from 'react';
import { Comment, stringToColor } from '../hooks/useComments';
import './Comments.css';

// =============================================================================
// Types
// =============================================================================

interface CommentHighlightProps {
  /** The comment this highlight represents */
  comment: Comment;
  /** Children (the text content to highlight) */
  children: React.ReactNode;
  /** Whether this highlight is currently selected */
  isSelected?: boolean;
  /** Callback when the highlight is clicked */
  onClick?: (comment: Comment) => void;
  /** Custom class name */
  className?: string;
  /** Color index for consistent author colors (1-6) */
  colorIndex?: number;
}

interface CommentHighlightRangeProps {
  /** The comment this highlight represents */
  comment: Comment;
  /** Bounding rectangles for the highlighted text */
  rects: DOMRect[];
  /** Whether this highlight is currently selected */
  isSelected?: boolean;
  /** Callback when the highlight is clicked */
  onClick?: (comment: Comment) => void;
  /** Color index for consistent author colors (1-6) */
  colorIndex?: number;
}

// =============================================================================
// Inline Highlight Component (wraps text)
// =============================================================================

export function CommentHighlight({
  comment,
  children,
  isSelected = false,
  onClick,
  className = '',
  colorIndex,
}: CommentHighlightProps) {
  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onClick?.(comment);
    },
    [comment, onClick]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        e.stopPropagation();
        onClick?.(comment);
      }
    },
    [comment, onClick]
  );

  // Compute color index from author ID if not provided
  const computedColorIndex = useMemo(() => {
    if (colorIndex !== undefined) {
      return colorIndex;
    }
    // Hash author ID to get consistent color
    let hash = 0;
    for (let i = 0; i < comment.author.id.length; i++) {
      hash = comment.author.id.charCodeAt(i) + ((hash << 5) - hash);
    }
    return (Math.abs(hash) % 6) + 1;
  }, [colorIndex, comment.author.id]);

  return (
    <mark
      className={`comment-highlight ${isSelected ? 'selected' : ''} ${comment.resolved ? 'resolved' : ''} ${className}`}
      data-color={computedColorIndex}
      data-comment-id={comment.id}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="button"
      aria-label={`Comment by ${comment.author.name}: ${comment.content.substring(0, 50)}...`}
      aria-pressed={isSelected}
      style={{
        '--highlight-color': comment.author.color
          ? `${comment.author.color}33`
          : undefined,
        '--highlight-border': comment.author.color
          ? `${comment.author.color}80`
          : undefined,
      } as React.CSSProperties}
    >
      {children}
    </mark>
  );
}

// =============================================================================
// Overlay Highlight Component (positioned absolutely over text)
// =============================================================================

export function CommentHighlightRange({
  comment,
  rects,
  isSelected = false,
  onClick,
  colorIndex,
}: CommentHighlightRangeProps) {
  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onClick?.(comment);
    },
    [comment, onClick]
  );

  // Compute color based on author
  const highlightColor = useMemo(() => {
    if (comment.author.color) {
      return comment.resolved
        ? 'rgba(156, 163, 175, 0.2)'
        : `${comment.author.color}33`;
    }
    return stringToColor(comment.author.id) + '33';
  }, [comment.author.color, comment.author.id, comment.resolved]);

  const borderColor = useMemo(() => {
    if (comment.author.color) {
      return comment.resolved
        ? 'rgba(156, 163, 175, 0.5)'
        : `${comment.author.color}80`;
    }
    return stringToColor(comment.author.id) + '80';
  }, [comment.author.color, comment.author.id, comment.resolved]);

  if (rects.length === 0) {
    return null;
  }

  return (
    <>
      {rects.map((rect, index) => (
        <div
          key={`${comment.id}-${index}`}
          className={`comment-highlight-overlay ${isSelected ? 'selected' : ''} ${comment.resolved ? 'resolved' : ''}`}
          style={{
            position: 'absolute',
            top: rect.top,
            left: rect.left,
            width: rect.width,
            height: rect.height,
            backgroundColor: highlightColor,
            borderBottom: `2px solid ${borderColor}`,
            cursor: 'pointer',
            pointerEvents: 'auto',
          }}
          onClick={handleClick}
          role="button"
          tabIndex={0}
          aria-label={`Comment by ${comment.author.name}`}
          data-comment-id={comment.id}
          data-color={colorIndex}
        />
      ))}
    </>
  );
}

// =============================================================================
// Highlights Container (renders all comment highlights)
// =============================================================================

interface CommentHighlightsContainerProps {
  /** All comments to highlight */
  comments: Comment[];
  /** Currently selected comment ID */
  selectedCommentId?: string;
  /** Callback when a highlight is clicked */
  onHighlightClick: (comment: Comment) => void;
  /** Function to get bounding rects for a comment's range */
  getRangeRects: (comment: Comment) => DOMRect[];
  /** Container scroll offset */
  scrollOffset?: { top: number; left: number };
}

export function CommentHighlightsContainer({
  comments,
  selectedCommentId,
  onHighlightClick,
  getRangeRects,
  scrollOffset = { top: 0, left: 0 },
}: CommentHighlightsContainerProps) {
  // Assign color indices to authors for consistent coloring
  const authorColorMap = useMemo(() => {
    const map = new Map<string, number>();
    let colorIndex = 1;
    for (const comment of comments) {
      if (!map.has(comment.author.id)) {
        map.set(comment.author.id, colorIndex);
        colorIndex = (colorIndex % 6) + 1;
      }
    }
    return map;
  }, [comments]);

  return (
    <div
      className="comment-highlights-container"
      style={{
        position: 'absolute',
        top: -scrollOffset.top,
        left: -scrollOffset.left,
        width: '100%',
        height: '100%',
        pointerEvents: 'none',
        overflow: 'hidden',
      }}
      aria-hidden="true"
    >
      {comments.map((comment) => {
        const rects = getRangeRects(comment);
        const colorIndex = authorColorMap.get(comment.author.id) || 1;

        return (
          <CommentHighlightRange
            key={comment.id}
            comment={comment}
            rects={rects}
            isSelected={comment.id === selectedCommentId}
            onClick={onHighlightClick}
            colorIndex={colorIndex}
          />
        );
      })}
    </div>
  );
}

// =============================================================================
// Hook: useCommentHighlights
// =============================================================================

interface UseCommentHighlightsOptions {
  /** All comments in the document */
  comments: Comment[];
  /** Currently selected comment ID */
  selectedCommentId?: string;
  /** Callback when a comment is selected via highlight click */
  onSelectComment: (comment: Comment) => void;
}

interface UseCommentHighlightsReturn {
  /** Get highlight props for a text node */
  getHighlightProps: (
    nodeId: string,
    startOffset: number,
    endOffset: number
  ) => CommentHighlightProps | null;
  /** Get all comments affecting a range */
  getCommentsInRange: (
    nodeId: string,
    startOffset: number,
    endOffset: number
  ) => Comment[];
  /** Author color map */
  authorColorMap: Map<string, number>;
}

export function useCommentHighlights({
  comments,
  selectedCommentId,
  onSelectComment,
}: UseCommentHighlightsOptions): UseCommentHighlightsReturn {
  // Build index of comments by node ID for quick lookup
  const commentsByNode = useMemo(() => {
    const map = new Map<string, Comment[]>();
    for (const comment of comments) {
      const { startNodeId, endNodeId } = comment.range;

      // Add to start node
      const startComments = map.get(startNodeId) || [];
      startComments.push(comment);
      map.set(startNodeId, startComments);

      // Add to end node if different
      if (endNodeId !== startNodeId) {
        const endComments = map.get(endNodeId) || [];
        endComments.push(comment);
        map.set(endNodeId, endComments);
      }
    }
    return map;
  }, [comments]);

  // Assign consistent colors to authors
  const authorColorMap = useMemo(() => {
    const map = new Map<string, number>();
    let colorIndex = 1;
    for (const comment of comments) {
      if (!map.has(comment.author.id)) {
        map.set(comment.author.id, colorIndex);
        colorIndex = (colorIndex % 6) + 1;
      }
    }
    return map;
  }, [comments]);

  const getCommentsInRange = useCallback(
    (nodeId: string, startOffset: number, endOffset: number): Comment[] => {
      const nodeComments = commentsByNode.get(nodeId) || [];
      return nodeComments.filter((comment) => {
        const { range } = comment;

        // Check if the comment range overlaps with the given range
        if (range.startNodeId === nodeId && range.endNodeId === nodeId) {
          // Comment is entirely within this node
          return range.startOffset < endOffset && range.endOffset > startOffset;
        } else if (range.startNodeId === nodeId) {
          // Comment starts in this node
          return range.startOffset < endOffset;
        } else if (range.endNodeId === nodeId) {
          // Comment ends in this node
          return range.endOffset > startOffset;
        }

        return false;
      });
    },
    [commentsByNode]
  );

  const getHighlightProps = useCallback(
    (
      nodeId: string,
      startOffset: number,
      endOffset: number
    ): CommentHighlightProps | null => {
      const commentsInRange = getCommentsInRange(nodeId, startOffset, endOffset);

      if (commentsInRange.length === 0) {
        return null;
      }

      // Use the first (primary) comment for the highlight
      const primaryComment = commentsInRange[0];
      const colorIndex = authorColorMap.get(primaryComment.author.id) || 1;

      return {
        comment: primaryComment,
        children: null, // Will be provided by the caller
        isSelected: commentsInRange.some((c) => c.id === selectedCommentId),
        onClick: onSelectComment,
        colorIndex,
      };
    },
    [getCommentsInRange, authorColorMap, selectedCommentId, onSelectComment]
  );

  return {
    getHighlightProps,
    getCommentsInRange,
    authorColorMap,
  };
}

export default CommentHighlight;
