/**
 * RevisionMarker - Inline component to render revision markup in document
 *
 * Features:
 * - Insertions: colored text + underline
 * - Deletions: strikethrough
 * - Moved text: double underline
 * - Author-specific colors
 * - Hover to show details popup
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { Revision, RevisionType, RevisionAuthor } from '../hooks/useTrackChanges';
import './TrackChanges.css';

// =============================================================================
// Types
// =============================================================================

export interface RevisionMarkerProps {
  /** The revision data */
  revision: Revision;
  /** The text content to display */
  children: React.ReactNode;
  /** Whether this revision is currently selected */
  isSelected?: boolean;
  /** Callback when the revision is clicked */
  onClick?: (revisionId: string) => void;
  /** Callback to accept this revision */
  onAccept?: (revisionId: string) => void;
  /** Callback to reject this revision */
  onReject?: (revisionId: string) => void;
}

export interface RevisionTooltipProps {
  /** The revision to show details for */
  revision: Revision;
  /** Position for the tooltip */
  position: { x: number; y: number };
  /** Whether the tooltip is visible */
  isVisible: boolean;
  /** Callback when accept is clicked */
  onAccept?: () => void;
  /** Callback when reject is clicked */
  onReject?: () => void;
}

// =============================================================================
// Helpers
// =============================================================================

function formatFullTimestamp(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

function getRevisionTypeLabel(type: RevisionType): string {
  switch (type) {
    case 'insert':
      return 'Inserted';
    case 'delete':
      return 'Deleted';
    case 'format':
      return 'Formatted';
    case 'move':
      return 'Moved';
    default:
      return 'Changed';
  }
}

// =============================================================================
// Revision Tooltip Component
// =============================================================================

export function RevisionTooltip({
  revision,
  position,
  isVisible,
  onAccept,
  onReject,
}: RevisionTooltipProps) {
  if (!isVisible) {
    return null;
  }

  return (
    <div
      className="revision-tooltip"
      style={{
        left: position.x,
        top: position.y,
      }}
      role="tooltip"
      aria-label={`${getRevisionTypeLabel(revision.type)} by ${revision.author.name}`}
    >
      <div className="revision-tooltip-header">
        <span
          className="revision-tooltip-author"
          style={{ color: revision.author.color }}
        >
          {revision.author.name}
        </span>
        <span className="revision-tooltip-type">
          {getRevisionTypeLabel(revision.type)}
        </span>
      </div>

      <div className="revision-tooltip-time">
        {formatFullTimestamp(revision.timestamp)}
      </div>

      {revision.formatProperty && (
        <div className="revision-tooltip-format">
          Changed: {revision.formatProperty}
        </div>
      )}

      {revision.type === 'move' && revision.originalLocation && (
        <div className="revision-tooltip-move-info">
          Moved from original location
        </div>
      )}

      {(onAccept || onReject) && (
        <div className="revision-tooltip-actions">
          {onAccept && (
            <button
              className="revision-tooltip-action accept"
              onClick={(e) => {
                e.stopPropagation();
                onAccept();
              }}
              aria-label="Accept this change"
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                aria-hidden="true"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
              Accept
            </button>
          )}
          {onReject && (
            <button
              className="revision-tooltip-action reject"
              onClick={(e) => {
                e.stopPropagation();
                onReject();
              }}
              aria-label="Reject this change"
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                aria-hidden="true"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
              Reject
            </button>
          )}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Revision Marker Component
// =============================================================================

export function RevisionMarker({
  revision,
  children,
  isSelected = false,
  onClick,
  onAccept,
  onReject,
}: RevisionMarkerProps) {
  const [showTooltip, setShowTooltip] = useState(false);
  const [tooltipPosition, setTooltipPosition] = useState({ x: 0, y: 0 });
  const markerRef = useRef<HTMLSpanElement>(null);
  const hoverTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Build CSS class name based on revision type
  const getClassName = useCallback(() => {
    const classes = ['revision-marker'];

    switch (revision.type) {
      case 'insert':
        classes.push('revision-insert');
        break;
      case 'delete':
        classes.push('revision-delete');
        break;
      case 'format':
        classes.push('revision-format');
        break;
      case 'move':
        classes.push('revision-move');
        break;
    }

    if (isSelected) {
      classes.push('revision-selected');
    }

    return classes.join(' ');
  }, [revision.type, isSelected]);

  // Handle mouse enter with delay
  const handleMouseEnter = useCallback((e: React.MouseEvent) => {
    if (hoverTimeoutRef.current) {
      clearTimeout(hoverTimeoutRef.current);
    }

    hoverTimeoutRef.current = setTimeout(() => {
      const rect = (e.target as HTMLElement).getBoundingClientRect();
      setTooltipPosition({
        x: rect.left,
        y: rect.bottom + 4,
      });
      setShowTooltip(true);
    }, 500); // 500ms delay before showing tooltip
  }, []);

  // Handle mouse leave
  const handleMouseLeave = useCallback(() => {
    if (hoverTimeoutRef.current) {
      clearTimeout(hoverTimeoutRef.current);
      hoverTimeoutRef.current = null;
    }
    setShowTooltip(false);
  }, []);

  // Handle click
  const handleClick = useCallback(() => {
    if (onClick) {
      onClick(revision.id);
    }
  }, [onClick, revision.id]);

  // Handle accept
  const handleAccept = useCallback(() => {
    setShowTooltip(false);
    if (onAccept) {
      onAccept(revision.id);
    }
  }, [onAccept, revision.id]);

  // Handle reject
  const handleReject = useCallback(() => {
    setShowTooltip(false);
    if (onReject) {
      onReject(revision.id);
    }
  }, [onReject, revision.id]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (hoverTimeoutRef.current) {
        clearTimeout(hoverTimeoutRef.current);
      }
    };
  }, []);

  // Get inline styles for author color
  const getInlineStyle = useCallback((): React.CSSProperties => {
    const style: React.CSSProperties = {};

    switch (revision.type) {
      case 'insert':
        style.color = revision.author.color;
        style.borderBottomColor = revision.author.color;
        break;
      case 'delete':
        style.color = revision.author.color;
        style.textDecorationColor = revision.author.color;
        break;
      case 'format':
        style.backgroundColor = `${revision.author.color}20`; // 20% opacity
        style.borderLeftColor = revision.author.color;
        break;
      case 'move':
        style.color = revision.author.color;
        style.borderBottomColor = revision.author.color;
        break;
    }

    return style;
  }, [revision.type, revision.author.color]);

  return (
    <>
      <span
        ref={markerRef}
        className={getClassName()}
        style={getInlineStyle()}
        onClick={handleClick}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        role="mark"
        aria-label={`${getRevisionTypeLabel(revision.type)} by ${revision.author.name}`}
        data-revision-id={revision.id}
        data-revision-type={revision.type}
        data-author-id={revision.author.id}
      >
        {children}
      </span>

      <RevisionTooltip
        revision={revision}
        position={tooltipPosition}
        isVisible={showTooltip}
        onAccept={onAccept ? handleAccept : undefined}
        onReject={onReject ? handleReject : undefined}
      />
    </>
  );
}

// =============================================================================
// Deletion Marker Component (for showing deleted content)
// =============================================================================

export interface DeletionMarkerProps {
  /** The revision data for the deletion */
  revision: Revision;
  /** The deleted text content */
  deletedText: string;
  /** Whether this revision is currently selected */
  isSelected?: boolean;
  /** Callback when the revision is clicked */
  onClick?: (revisionId: string) => void;
  /** Callback to accept this revision (removes the deleted text permanently) */
  onAccept?: (revisionId: string) => void;
  /** Callback to reject this revision (restores the deleted text) */
  onReject?: (revisionId: string) => void;
}

export function DeletionMarker({
  revision,
  deletedText,
  isSelected = false,
  onClick,
  onAccept,
  onReject,
}: DeletionMarkerProps) {
  return (
    <RevisionMarker
      revision={{ ...revision, type: 'delete' }}
      isSelected={isSelected}
      onClick={onClick}
      onAccept={onAccept}
      onReject={onReject}
    >
      {deletedText}
    </RevisionMarker>
  );
}

// =============================================================================
// Insertion Marker Component (for showing inserted content)
// =============================================================================

export interface InsertionMarkerProps {
  /** The revision data for the insertion */
  revision: Revision;
  /** The inserted text content */
  insertedText: string;
  /** Whether this revision is currently selected */
  isSelected?: boolean;
  /** Callback when the revision is clicked */
  onClick?: (revisionId: string) => void;
  /** Callback to accept this revision (keeps the inserted text) */
  onAccept?: (revisionId: string) => void;
  /** Callback to reject this revision (removes the inserted text) */
  onReject?: (revisionId: string) => void;
}

export function InsertionMarker({
  revision,
  insertedText,
  isSelected = false,
  onClick,
  onAccept,
  onReject,
}: InsertionMarkerProps) {
  return (
    <RevisionMarker
      revision={{ ...revision, type: 'insert' }}
      isSelected={isSelected}
      onClick={onClick}
      onAccept={onAccept}
      onReject={onReject}
    >
      {insertedText}
    </RevisionMarker>
  );
}

// =============================================================================
// Move Marker Component (for showing moved content)
// =============================================================================

export interface MoveMarkerProps {
  /** The revision data for the move */
  revision: Revision;
  /** The moved text content */
  movedText: string;
  /** Whether this is the source (original) or destination (new) location */
  location: 'source' | 'destination';
  /** Whether this revision is currently selected */
  isSelected?: boolean;
  /** Callback when the revision is clicked */
  onClick?: (revisionId: string) => void;
  /** Callback to accept this revision */
  onAccept?: (revisionId: string) => void;
  /** Callback to reject this revision */
  onReject?: (revisionId: string) => void;
}

export function MoveMarker({
  revision,
  movedText,
  location,
  isSelected = false,
  onClick,
  onAccept,
  onReject,
}: MoveMarkerProps) {
  const className = location === 'source' ? 'revision-move-source' : 'revision-move-destination';

  return (
    <span className={className}>
      <RevisionMarker
        revision={{ ...revision, type: 'move' }}
        isSelected={isSelected}
        onClick={onClick}
        onAccept={onAccept}
        onReject={onReject}
      >
        {movedText}
      </RevisionMarker>
    </span>
  );
}

// =============================================================================
// Format Change Indicator Component
// =============================================================================

export interface FormatChangeIndicatorProps {
  /** The revision data */
  revision: Revision;
  /** Whether to show a visual indicator bar on the left */
  showIndicatorBar?: boolean;
  /** Whether this revision is currently selected */
  isSelected?: boolean;
  /** Callback when the revision is clicked */
  onClick?: (revisionId: string) => void;
}

export function FormatChangeIndicator({
  revision,
  showIndicatorBar = true,
  isSelected = false,
  onClick,
}: FormatChangeIndicatorProps) {
  const handleClick = useCallback(() => {
    if (onClick) {
      onClick(revision.id);
    }
  }, [onClick, revision.id]);

  if (!showIndicatorBar) {
    return null;
  }

  return (
    <span
      className={`format-change-indicator ${isSelected ? 'selected' : ''}`}
      style={{ backgroundColor: revision.author.color }}
      onClick={handleClick}
      role="mark"
      aria-label={`Format change by ${revision.author.name}: ${revision.formatProperty || 'formatting'}`}
      data-revision-id={revision.id}
      title={`${revision.author.name}: ${revision.formatProperty || 'Formatting change'}`}
    />
  );
}

export default RevisionMarker;
