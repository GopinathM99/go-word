/**
 * RemoteCursors - Component for rendering remote user cursors and selections
 *
 * This component displays cursors and text selections from other users
 * in a collaborative editing session. Each user has a unique color,
 * and their cursor shows their name with a typing indicator when active.
 */

import React, { useMemo } from 'react';
import './RemoteCursors.css';

// =============================================================================
// Types
// =============================================================================

/**
 * Position in the document
 */
export interface Position {
  /** Node ID in the document tree */
  nodeId: string;
  /** Character offset within the node */
  offset: number;
}

/**
 * Remote user cursor data
 */
export interface RemoteCursor {
  /** User ID */
  userId: string;
  /** Display name */
  displayName: string;
  /** Cursor color (hex) */
  color: string;
  /** Cursor position */
  position: Position;
  /** Whether user is currently typing */
  isTyping: boolean;
}

/**
 * Selection range in the document
 */
export interface SelectionRangeData {
  /** Start position */
  start: Position;
  /** End position */
  end: Position;
}

/**
 * Remote user selection data
 */
export interface RemoteSelection {
  /** User ID */
  userId: string;
  /** Selection color (hex) */
  color: string;
  /** Selection range */
  selection: SelectionRangeData;
}

/**
 * Coordinates for rendering a cursor
 */
export interface CursorCoordinates {
  /** X position in pixels */
  x: number;
  /** Y position in pixels */
  y: number;
  /** Line height in pixels */
  height: number;
}

/**
 * Props for the RemoteCursors component
 */
export interface RemoteCursorsProps {
  /** List of remote cursors to render */
  cursors: RemoteCursor[];
  /** List of remote selections to render */
  selections: RemoteSelection[];
  /**
   * Function to convert a document position to screen coordinates
   * Returns null if the position is not currently visible
   */
  getPositionCoords: (position: Position) => CursorCoordinates | null;
  /**
   * Function to get DOM rectangles for a selection range
   * Returns an array of DOMRect objects for each line of the selection
   */
  getSelectionRects: (selection: SelectionRangeData) => DOMRect[];
  /** Whether to always show cursor labels (default: show on hover) */
  alwaysShowLabels?: boolean;
  /** Opacity for selection highlights (0-1, default: 0.3) */
  selectionOpacity?: number;
  /** Whether cursor carets should blink (default: true) */
  enableBlinking?: boolean;
}

// =============================================================================
// Sub-components
// =============================================================================

/**
 * Single remote cursor component
 */
interface CursorCaretProps {
  cursor: RemoteCursor;
  coords: CursorCoordinates;
  alwaysShowLabel: boolean;
  enableBlinking: boolean;
}

const CursorCaret: React.FC<CursorCaretProps> = ({
  cursor,
  coords,
  alwaysShowLabel,
  enableBlinking,
}) => {
  return (
    <div
      className="remote-cursor"
      style={{
        left: coords.x,
        top: coords.y,
      }}
      data-user-id={cursor.userId}
      data-typing={cursor.isTyping}
    >
      <div
        className={`remote-cursor-caret ${enableBlinking ? 'blink' : ''}`}
        style={{
          backgroundColor: cursor.color,
          height: coords.height,
        }}
      />
      <div
        className={`remote-cursor-flag ${alwaysShowLabel ? 'visible' : ''}`}
        style={{
          backgroundColor: cursor.color,
        }}
      />
      <div
        className={`remote-cursor-label ${alwaysShowLabel ? 'visible' : ''}`}
        style={{
          backgroundColor: cursor.color,
        }}
      >
        <span className="remote-cursor-name">{cursor.displayName}</span>
        {cursor.isTyping && (
          <span className="typing-indicator" aria-label="typing">
            <span className="typing-dot" />
            <span className="typing-dot" />
            <span className="typing-dot" />
          </span>
        )}
      </div>
    </div>
  );
};

/**
 * Single selection rectangle
 */
interface SelectionRectProps {
  rect: DOMRect;
  color: string;
  opacity: number;
  index: number;
}

const SelectionRect: React.FC<SelectionRectProps> = ({
  rect,
  color,
  opacity,
  index,
}) => {
  return (
    <div
      className="remote-selection-rect"
      style={{
        left: rect.left,
        top: rect.top,
        width: rect.width,
        height: rect.height,
        backgroundColor: color,
        opacity,
      }}
      data-index={index}
    />
  );
};

/**
 * Remote selection component (may span multiple lines)
 */
interface SelectionHighlightProps {
  selection: RemoteSelection;
  rects: DOMRect[];
  opacity: number;
}

const SelectionHighlight: React.FC<SelectionHighlightProps> = ({
  selection,
  rects,
  opacity,
}) => {
  if (rects.length === 0) {
    return null;
  }

  return (
    <div
      className="remote-selection"
      data-user-id={selection.userId}
    >
      {rects.map((rect, index) => (
        <SelectionRect
          key={`${selection.userId}-rect-${index}`}
          rect={rect}
          color={selection.color}
          opacity={opacity}
          index={index}
        />
      ))}
    </div>
  );
};

// =============================================================================
// Main Component
// =============================================================================

export const RemoteCursors: React.FC<RemoteCursorsProps> = ({
  cursors,
  selections,
  getPositionCoords,
  getSelectionRects,
  alwaysShowLabels = false,
  selectionOpacity = 0.3,
  enableBlinking = true,
}) => {
  // Memoize cursor data with coordinates
  const cursorData = useMemo(() => {
    return cursors
      .map((cursor) => {
        const coords = getPositionCoords(cursor.position);
        return coords ? { cursor, coords } : null;
      })
      .filter((data): data is { cursor: RemoteCursor; coords: CursorCoordinates } => data !== null);
  }, [cursors, getPositionCoords]);

  // Memoize selection data with rects
  const selectionData = useMemo(() => {
    return selections
      .map((selection) => {
        const rects = getSelectionRects(selection.selection);
        return { selection, rects };
      })
      .filter(({ rects }) => rects.length > 0);
  }, [selections, getSelectionRects]);

  // Render nothing if there are no cursors or selections
  if (cursorData.length === 0 && selectionData.length === 0) {
    return null;
  }

  return (
    <div
      className="remote-cursors-container"
      aria-hidden="true"
      role="presentation"
    >
      {/* Render selections first (under cursors) */}
      <div className="remote-selections-layer">
        {selectionData.map(({ selection, rects }) => (
          <SelectionHighlight
            key={`sel-${selection.userId}`}
            selection={selection}
            rects={rects}
            opacity={selectionOpacity}
          />
        ))}
      </div>

      {/* Render cursors on top */}
      <div className="remote-cursors-layer">
        {cursorData.map(({ cursor, coords }) => (
          <CursorCaret
            key={`cursor-${cursor.userId}`}
            cursor={cursor}
            coords={coords}
            alwaysShowLabel={alwaysShowLabels}
            enableBlinking={enableBlinking}
          />
        ))}
      </div>
    </div>
  );
};

// =============================================================================
// Utility Functions
// =============================================================================

/**
 * Convert a hex color to RGBA
 */
export function hexToRgba(hex: string, alpha: number): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) {
    return `rgba(0, 0, 0, ${alpha})`;
  }
  const r = parseInt(result[1], 16);
  const g = parseInt(result[2], 16);
  const b = parseInt(result[3], 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/**
 * Get a contrasting text color (black or white) for a background color
 */
export function getContrastColor(hexColor: string): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hexColor);
  if (!result) {
    return '#ffffff';
  }
  const r = parseInt(result[1], 16);
  const g = parseInt(result[2], 16);
  const b = parseInt(result[3], 16);
  // Calculate relative luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.5 ? '#000000' : '#ffffff';
}

/**
 * Generate a deterministic color from a string (user ID)
 */
export function stringToColor(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }

  // Use HSL to ensure vivid, distinguishable colors
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 70%, 50%)`;
}

export default RemoteCursors;
