/**
 * OutlinePanel - Document outline/navigation panel
 *
 * Features:
 * - Tree view of document headings (H1-H6)
 * - Hierarchical display (H2 nested under H1, etc.)
 * - Expand/collapse functionality for each level
 * - Click heading to navigate to that location
 * - Highlight current position in outline
 * - Show heading level indicators (icons or indentation)
 * - Real-time sync with document changes
 * - Full keyboard navigation support
 * - ARIA accessibility attributes
 */

import { useState, useCallback, useRef, useEffect, KeyboardEvent } from 'react';
import {
  OutlineHeading,
  getHeadingLevelIndicator,
  getHeadingAccessibleLabel,
} from '../lib/outlineTypes';
import { useOutline, UseOutlineOptions } from '../hooks/useOutline';
import './OutlinePanel.css';

// =============================================================================
// Types
// =============================================================================

export interface OutlinePanelProps {
  /** Whether the panel is visible */
  isOpen: boolean;
  /** Callback to close the panel */
  onClose?: () => void;
  /** Document ID */
  docId?: string;
  /** Polling interval for updates (ms) */
  pollInterval?: number;
  /** Callback when navigating to a heading */
  onNavigate?: (headingId: string, position: { page: number; offset: number }) => void;
  /** Initial expand state */
  initialExpandState?: UseOutlineOptions['initialExpandState'];
  /** Title for the panel */
  title?: string;
  /** Whether to show the close button */
  showCloseButton?: boolean;
  /** Whether to show heading level badges */
  showLevelBadges?: boolean;
  /** Whether to show page numbers */
  showPageNumbers?: boolean;
  /** Compact mode (less padding) */
  compact?: boolean;
}

interface OutlineItemProps {
  heading: OutlineHeading;
  depth: number;
  isExpanded: boolean;
  isCurrent: boolean;
  isFocused: boolean;
  hasChildren: boolean;
  showLevelBadge: boolean;
  showPageNumber: boolean;
  onToggleExpand: (headingId: string) => void;
  onNavigate: (headingId: string) => void;
  onKeyDown: (e: KeyboardEvent, headingId: string) => void;
  expandedIds: Set<string>;
  currentHeadingId: string | null;
  focusedId: string | null;
  setFocusedId: (id: string | null) => void;
}

// =============================================================================
// OutlineItem Component
// =============================================================================

function OutlineItem({
  heading,
  depth,
  isExpanded,
  isCurrent,
  isFocused,
  hasChildren,
  showLevelBadge,
  showPageNumber,
  onToggleExpand,
  onNavigate,
  onKeyDown,
  expandedIds,
  currentHeadingId,
  focusedId,
  setFocusedId,
}: OutlineItemProps) {
  const itemRef = useRef<HTMLDivElement>(null);

  // Focus management
  useEffect(() => {
    if (isFocused && itemRef.current) {
      itemRef.current.focus();
    }
  }, [isFocused]);

  const handleClick = useCallback(() => {
    onNavigate(heading.id);
    setFocusedId(heading.id);
  }, [heading.id, onNavigate, setFocusedId]);

  const handleToggleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onToggleExpand(heading.id);
    },
    [heading.id, onToggleExpand]
  );

  const handleKeyDownLocal = useCallback(
    (e: KeyboardEvent<HTMLDivElement>) => {
      onKeyDown(e, heading.id);
    },
    [heading.id, onKeyDown]
  );

  return (
    <li
      role="treeitem"
      aria-expanded={hasChildren ? isExpanded : undefined}
      aria-selected={isCurrent}
      aria-level={depth + 1}
      aria-setsize={1}
      aria-posinset={1}
    >
      <div
        ref={itemRef}
        className={`outline-item ${isCurrent ? 'current' : ''} ${isFocused ? 'focused' : ''}`}
        style={{ '--depth': depth } as React.CSSProperties}
        onClick={handleClick}
        onKeyDown={handleKeyDownLocal}
        tabIndex={isFocused ? 0 : -1}
        role="button"
        aria-label={getHeadingAccessibleLabel(heading)}
      >
        {/* Expand/Collapse Toggle */}
        {hasChildren ? (
          <button
            className="expand-toggle"
            onClick={handleToggleClick}
            aria-label={isExpanded ? 'Collapse' : 'Expand'}
            tabIndex={-1}
            type="button"
          >
            <span className={`expand-icon ${isExpanded ? 'expanded' : ''}`} aria-hidden="true">
              {isExpanded ? '\u25BC' : '\u25B6'}
            </span>
          </button>
        ) : (
          <span className="expand-placeholder" aria-hidden="true" />
        )}

        {/* Level Badge */}
        {showLevelBadge && (
          <span className={`level-badge level-${heading.level}`} aria-hidden="true">
            {getHeadingLevelIndicator(heading.level)}
          </span>
        )}

        {/* Heading Text */}
        <span className="heading-text" title={heading.text}>
          {heading.text || '(Untitled)'}
        </span>

        {/* Page Number */}
        {showPageNumber && (
          <span className="page-number" aria-label={`Page ${heading.position.page}`}>
            {heading.position.page}
          </span>
        )}
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <ul role="group" className="outline-children">
          {heading.children.map((child) => (
            <OutlineItem
              key={child.id}
              heading={child}
              depth={depth + 1}
              isExpanded={expandedIds.has(child.id)}
              isCurrent={currentHeadingId === child.id}
              isFocused={focusedId === child.id}
              hasChildren={child.children.length > 0}
              showLevelBadge={showLevelBadge}
              showPageNumber={showPageNumber}
              onToggleExpand={onToggleExpand}
              onNavigate={onNavigate}
              onKeyDown={onKeyDown}
              expandedIds={expandedIds}
              currentHeadingId={currentHeadingId}
              focusedId={focusedId}
              setFocusedId={setFocusedId}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

// =============================================================================
// OutlinePanel Component
// =============================================================================

export function OutlinePanel({
  isOpen,
  onClose,
  docId = 'default',
  pollInterval = 0,
  onNavigate: onNavigateExternal,
  initialExpandState = 'first-level',
  title = 'Document Outline',
  showCloseButton = true,
  showLevelBadges = true,
  showPageNumbers = true,
  compact = false,
}: OutlinePanelProps) {
  const [focusedId, setFocusedId] = useState<string | null>(null);

  const {
    state,
    isLoading,
    error,
    navigateToHeading,
    toggleExpanded,
    expand,
    collapse,
    expandAll,
    collapseAll,
    refresh,
    visibleHeadings,
    totalCount,
  } = useOutline({
    docId,
    pollInterval,
    onNavigate: onNavigateExternal,
    initialExpandState,
    autoExpandToCurrentHeading: true,
  });

  const treeRef = useRef<HTMLUListElement>(null);

  // Reset focus when panel closes
  useEffect(() => {
    if (!isOpen) {
      setFocusedId(null);
    }
  }, [isOpen]);

  // Handle navigation with focus update
  const handleNavigate = useCallback(
    (headingId: string) => {
      navigateToHeading(headingId);
      setFocusedId(headingId);
    },
    [navigateToHeading]
  );

  // Keyboard navigation handler
  const handleKeyDown = useCallback(
    (e: KeyboardEvent, headingId: string) => {
      const currentIndex = visibleHeadings.findIndex((h) => h.id === headingId);
      if (currentIndex === -1) return;

      const current = visibleHeadings[currentIndex];
      let newFocusId: string | null = null;
      let handled = false;

      switch (e.key) {
        case 'ArrowDown':
          // Move to next visible item
          if (currentIndex < visibleHeadings.length - 1) {
            newFocusId = visibleHeadings[currentIndex + 1].id;
          }
          handled = true;
          break;

        case 'ArrowUp':
          // Move to previous visible item
          if (currentIndex > 0) {
            newFocusId = visibleHeadings[currentIndex - 1].id;
          }
          handled = true;
          break;

        case 'ArrowRight':
          // Expand if collapsed and has children, otherwise move to first child
          if (current.children.length > 0) {
            if (!state.expandedIds.has(headingId)) {
              expand(headingId);
            } else {
              newFocusId = current.children[0].id;
            }
          }
          handled = true;
          break;

        case 'ArrowLeft':
          // Collapse if expanded, otherwise move to parent
          if (state.expandedIds.has(headingId) && current.children.length > 0) {
            collapse(headingId);
          } else {
            // Find parent
            const findParent = (
              items: OutlineHeading[],
              targetId: string,
              parent: OutlineHeading | null = null
            ): OutlineHeading | null => {
              for (const item of items) {
                if (item.id === targetId) {
                  return parent;
                }
                const found = findParent(item.children, targetId, item);
                if (found !== null) {
                  return found;
                }
              }
              return null;
            };
            const parent = findParent(state.headings, headingId);
            if (parent) {
              newFocusId = parent.id;
            }
          }
          handled = true;
          break;

        case 'Enter':
        case ' ':
          // Navigate to heading
          handleNavigate(headingId);
          handled = true;
          break;

        case 'Home':
          // Move to first item
          if (visibleHeadings.length > 0) {
            newFocusId = visibleHeadings[0].id;
          }
          handled = true;
          break;

        case 'End':
          // Move to last item
          if (visibleHeadings.length > 0) {
            newFocusId = visibleHeadings[visibleHeadings.length - 1].id;
          }
          handled = true;
          break;

        case '*':
          // Expand all
          expandAll();
          handled = true;
          break;

        case 'Escape':
          // Close panel
          if (onClose) {
            onClose();
          }
          handled = true;
          break;
      }

      if (handled) {
        e.preventDefault();
        e.stopPropagation();
      }

      if (newFocusId) {
        setFocusedId(newFocusId);
      }
    },
    [
      visibleHeadings,
      state.expandedIds,
      state.headings,
      expand,
      collapse,
      expandAll,
      handleNavigate,
      onClose,
    ]
  );

  // Handle panel-level keyboard events
  const handlePanelKeyDown = useCallback(
    (e: KeyboardEvent<HTMLDivElement>) => {
      if (e.key === 'Escape' && onClose) {
        e.preventDefault();
        onClose();
      }
    },
    [onClose]
  );

  // Initialize focus when panel opens
  useEffect(() => {
    if (isOpen && visibleHeadings.length > 0 && !focusedId) {
      // Focus current heading or first heading
      setFocusedId(state.currentHeadingId || visibleHeadings[0].id);
    }
  }, [isOpen, visibleHeadings, state.currentHeadingId, focusedId]);

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className={`outline-panel ${compact ? 'compact' : ''}`}
      role="complementary"
      aria-label={title}
      onKeyDown={handlePanelKeyDown}
    >
      {/* Header */}
      <div className="outline-header">
        <h2 id="outline-title">{title}</h2>
        <div className="outline-header-actions">
          <span className="heading-count" aria-live="polite">
            {totalCount} {totalCount === 1 ? 'heading' : 'headings'}
          </span>
          {showCloseButton && onClose && (
            <button
              className="close-button"
              onClick={onClose}
              aria-label="Close outline panel"
              type="button"
            >
              <span aria-hidden="true">X</span>
            </button>
          )}
        </div>
      </div>

      {/* Toolbar */}
      <div className="outline-toolbar" role="toolbar" aria-label="Outline actions">
        <button
          className="toolbar-button"
          onClick={expandAll}
          aria-label="Expand all"
          title="Expand all"
          type="button"
        >
          <span aria-hidden="true">+</span>
        </button>
        <button
          className="toolbar-button"
          onClick={collapseAll}
          aria-label="Collapse all"
          title="Collapse all"
          type="button"
        >
          <span aria-hidden="true">-</span>
        </button>
        <button
          className="toolbar-button"
          onClick={refresh}
          aria-label="Refresh outline"
          title="Refresh"
          disabled={isLoading}
          type="button"
        >
          <span aria-hidden="true">{isLoading ? '...' : '\u21BB'}</span>
        </button>
      </div>

      {/* Content */}
      <div className="outline-content">
        {isLoading && state.headings.length === 0 ? (
          <div className="outline-loading" role="status" aria-live="polite">
            <div className="loading-spinner" aria-hidden="true" />
            <span>Loading outline...</span>
          </div>
        ) : error ? (
          <div className="outline-error" role="alert">
            <span className="error-icon" aria-hidden="true">!</span>
            <span>Error loading outline</span>
            <button className="retry-button" onClick={refresh} type="button">
              Retry
            </button>
          </div>
        ) : state.headings.length === 0 ? (
          <div className="outline-empty" role="status">
            <span className="empty-icon" aria-hidden="true">#</span>
            <span>No headings found</span>
            <span className="empty-hint">
              Add headings (H1-H6) to your document to see them here.
            </span>
          </div>
        ) : (
          <ul
            ref={treeRef}
            className="outline-tree"
            role="tree"
            aria-labelledby="outline-title"
          >
            {state.headings.map((heading) => (
              <OutlineItem
                key={heading.id}
                heading={heading}
                depth={0}
                isExpanded={state.expandedIds.has(heading.id)}
                isCurrent={state.currentHeadingId === heading.id}
                isFocused={focusedId === heading.id}
                hasChildren={heading.children.length > 0}
                showLevelBadge={showLevelBadges}
                showPageNumber={showPageNumbers}
                onToggleExpand={toggleExpanded}
                onNavigate={handleNavigate}
                onKeyDown={handleKeyDown}
                expandedIds={state.expandedIds}
                currentHeadingId={state.currentHeadingId}
                focusedId={focusedId}
                setFocusedId={setFocusedId}
              />
            ))}
          </ul>
        )}
      </div>

      {/* Footer with keyboard hints */}
      <div className="outline-footer">
        <span className="keyboard-hint">
          <kbd>Enter</kbd> to navigate
        </span>
        <span className="keyboard-hint">
          <kbd>Arrow</kbd> keys to move
        </span>
      </div>
    </div>
  );
}

export default OutlinePanel;
