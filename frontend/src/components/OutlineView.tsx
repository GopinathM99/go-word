/**
 * OutlineView - Hierarchical heading view for document navigation and reorganization
 *
 * Features:
 * - Show document as collapsible outline
 * - Headings shown at their level with indentation
 * - Body text collapsed under headings (expandable)
 * - Promote/demote headings with buttons
 * - Drag and drop reordering
 * - Show only certain heading levels (filter)
 * - Full keyboard navigation
 * - ARIA accessibility
 */

import { useState, useCallback, useRef, useEffect, KeyboardEvent, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  OutlineViewOptions,
  shouldShowLevel,
} from '../lib/viewModeTypes';
import {
  OutlineHeading,
  getHeadingLevelIndicator,
  getHeadingAccessibleLabel,
} from '../lib/outlineTypes';
import { useOutline } from '../hooks/useOutline';
import './OutlineView.css';

// Local helper to flatten outline headings
function flattenOutlineHeadingsLocal(
  headings: OutlineHeading[],
  expandedIds: Set<string>,
  options: OutlineViewOptions
): OutlineHeading[] {
  const result: OutlineHeading[] = [];

  function traverse(items: OutlineHeading[]) {
    for (const heading of items) {
      if (shouldShowLevel(heading.level, options)) {
        result.push(heading);
        if (heading.children.length > 0 && expandedIds.has(heading.id)) {
          traverse(heading.children);
        }
      }
    }
  }

  traverse(headings);
  return result;
}

// =============================================================================
// Types
// =============================================================================

export interface OutlineViewProps {
  /** Document ID */
  docId?: string;
  /** Outline view options */
  options: OutlineViewOptions;
  /** Callback when options change */
  onOptionsChange?: (options: Partial<OutlineViewOptions>) => void;
  /** Callback when navigating to a heading */
  onNavigate?: (headingId: string, position: { page: number; offset: number }) => void;
  /** Callback when heading is promoted */
  onPromote?: (headingId: string) => void;
  /** Callback when heading is demoted */
  onDemote?: (headingId: string) => void;
  /** Callback when section is moved */
  onMoveSection?: (headingId: string, beforeId: string | null) => void;
  /** Callback to exit outline view */
  onExit?: () => void;
}

// =============================================================================
// Level Filter Component
// =============================================================================

interface LevelFilterProps {
  options: OutlineViewOptions;
  onOptionsChange: (options: Partial<OutlineViewOptions>) => void;
}

function LevelFilter({ options, onOptionsChange }: LevelFilterProps) {
  const levels = [1, 2, 3, 4, 5, 6];

  const handleChange = useCallback(
    (level: number) => {
      const currentEnd = options.showLevelsEnd;
      // Toggle: if clicking at or above current end, show all up to that level
      // If clicking below, expand to include that level
      const newEnd = level + 1;
      onOptionsChange({ showLevelsEnd: newEnd });
    },
    [options.showLevelsEnd, onOptionsChange]
  );

  return (
    <div className="outline-level-filter" role="group" aria-label="Show heading levels">
      <span className="filter-label">Show levels:</span>
      <div className="filter-buttons">
        {levels.map((level) => {
          const isVisible = level >= options.showLevelsStart && level < options.showLevelsEnd;
          return (
            <button
              key={level}
              className={`filter-button ${isVisible ? 'active' : ''}`}
              onClick={() => handleChange(level)}
              aria-pressed={isVisible}
              type="button"
            >
              H{level}
            </button>
          );
        })}
        <button
          className="filter-button show-all"
          onClick={() => onOptionsChange({ showLevelsStart: 1, showLevelsEnd: 7 })}
          type="button"
        >
          All
        </button>
      </div>
    </div>
  );
}

// =============================================================================
// Outline Item Component
// =============================================================================

interface OutlineItemProps {
  heading: OutlineHeading;
  depth: number;
  options: OutlineViewOptions;
  isExpanded: boolean;
  isCurrent: boolean;
  isFocused: boolean;
  isDragTarget: boolean;
  onToggleExpand: (headingId: string) => void;
  onNavigate: (headingId: string) => void;
  onPromote: (headingId: string) => void;
  onDemote: (headingId: string) => void;
  onKeyDown: (e: KeyboardEvent, headingId: string) => void;
  onDragStart: (headingId: string) => void;
  onDragOver: (headingId: string) => void;
  onDragEnd: () => void;
  onDrop: (targetId: string) => void;
  expandedIds: Set<string>;
  currentHeadingId: string | null;
  focusedId: string | null;
  setFocusedId: (id: string | null) => void;
  draggedId: string | null;
}

function OutlineItem({
  heading,
  depth,
  options,
  isExpanded,
  isCurrent,
  isFocused,
  isDragTarget,
  onToggleExpand,
  onNavigate,
  onPromote,
  onDemote,
  onKeyDown,
  onDragStart,
  onDragOver,
  onDragEnd,
  onDrop,
  expandedIds,
  currentHeadingId,
  focusedId,
  setFocusedId,
  draggedId,
}: OutlineItemProps) {
  const itemRef = useRef<HTMLDivElement>(null);
  const hasChildren = heading.children.length > 0;
  const canPromote = heading.level > 1;
  const canDemote = heading.level < 6;
  const isDragging = draggedId === heading.id;

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

  const handlePromoteClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (canPromote) onPromote(heading.id);
    },
    [heading.id, canPromote, onPromote]
  );

  const handleDemoteClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (canDemote) onDemote(heading.id);
    },
    [heading.id, canDemote, onDemote]
  );

  const handleKeyDownLocal = useCallback(
    (e: KeyboardEvent<HTMLDivElement>) => {
      onKeyDown(e, heading.id);
    },
    [heading.id, onKeyDown]
  );

  const handleDragStartLocal = useCallback(() => {
    if (options.enableDragDrop) {
      onDragStart(heading.id);
    }
  }, [heading.id, options.enableDragDrop, onDragStart]);

  const handleDragOverLocal = useCallback(
    (e: React.DragEvent) => {
      if (options.enableDragDrop && draggedId && draggedId !== heading.id) {
        e.preventDefault();
        onDragOver(heading.id);
      }
    },
    [heading.id, options.enableDragDrop, draggedId, onDragOver]
  );

  const handleDropLocal = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      if (options.enableDragDrop && draggedId && draggedId !== heading.id) {
        onDrop(heading.id);
      }
    },
    [heading.id, options.enableDragDrop, draggedId, onDrop]
  );

  return (
    <li
      role="treeitem"
      aria-expanded={hasChildren ? isExpanded : undefined}
      aria-selected={isCurrent}
      aria-level={depth + 1}
    >
      <div
        ref={itemRef}
        className={`outline-view-item ${isCurrent ? 'current' : ''} ${isFocused ? 'focused' : ''} ${isDragging ? 'dragging' : ''} ${isDragTarget ? 'drag-target' : ''}`}
        style={{
          '--depth': depth,
          '--indent': options.indentPerLevel,
        } as React.CSSProperties}
        onClick={handleClick}
        onKeyDown={handleKeyDownLocal}
        onDragStart={handleDragStartLocal}
        onDragOver={handleDragOverLocal}
        onDragEnd={onDragEnd}
        onDrop={handleDropLocal}
        draggable={options.enableDragDrop}
        tabIndex={isFocused ? 0 : -1}
        role="button"
        aria-label={getHeadingAccessibleLabel(heading)}
      >
        {/* Expand/Collapse Toggle */}
        {hasChildren ? (
          <button
            className="outline-expand-toggle"
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
          <span className="outline-expand-placeholder" aria-hidden="true" />
        )}

        {/* Level Indicator */}
        {options.showLevelIndicators && (
          <span className={`outline-level-badge level-${heading.level}`} aria-hidden="true">
            {getHeadingLevelIndicator(heading.level)}
          </span>
        )}

        {/* Heading Text */}
        <span className="outline-heading-text" title={heading.text}>
          {heading.text || '(Untitled)'}
        </span>

        {/* Actions */}
        <div className="outline-actions">
          <button
            className="outline-action-btn promote"
            onClick={handlePromoteClick}
            disabled={!canPromote}
            aria-label="Promote heading"
            title="Promote (decrease level)"
            tabIndex={-1}
            type="button"
          >
            <span aria-hidden="true">&larr;</span>
          </button>
          <button
            className="outline-action-btn demote"
            onClick={handleDemoteClick}
            disabled={!canDemote}
            aria-label="Demote heading"
            title="Demote (increase level)"
            tabIndex={-1}
            type="button"
          >
            <span aria-hidden="true">&rarr;</span>
          </button>
        </div>
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <ul role="group" className="outline-children">
          {heading.children
            .filter((child) => shouldShowLevel(child.level, options))
            .map((child) => (
              <OutlineItem
                key={child.id}
                heading={child}
                depth={depth + 1}
                options={options}
                isExpanded={expandedIds.has(child.id)}
                isCurrent={currentHeadingId === child.id}
                isFocused={focusedId === child.id}
                isDragTarget={false}
                onToggleExpand={onToggleExpand}
                onNavigate={onNavigate}
                onPromote={onPromote}
                onDemote={onDemote}
                onKeyDown={onKeyDown}
                onDragStart={onDragStart}
                onDragOver={onDragOver}
                onDragEnd={onDragEnd}
                onDrop={onDrop}
                expandedIds={expandedIds}
                currentHeadingId={currentHeadingId}
                focusedId={focusedId}
                setFocusedId={setFocusedId}
                draggedId={draggedId}
              />
            ))}
        </ul>
      )}
    </li>
  );
}

// =============================================================================
// Main OutlineView Component
// =============================================================================

export function OutlineView({
  docId = 'default',
  options,
  onOptionsChange,
  onNavigate: onNavigateExternal,
  onPromote: onPromoteExternal,
  onDemote: onDemoteExternal,
  onMoveSection,
  onExit,
}: OutlineViewProps) {
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragTargetId, setDragTargetId] = useState<string | null>(null);

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
    pollInterval: 0,
    onNavigate: onNavigateExternal,
    initialExpandState: 'all',
    autoExpandToCurrentHeading: true,
  });

  const treeRef = useRef<HTMLUListElement>(null);

  // Filter headings by level
  const filteredHeadings = useMemo(() => {
    return state.headings.filter((h) => shouldShowLevel(h.level, options));
  }, [state.headings, options]);

  // Get visible headings for keyboard navigation
  const visibleFilteredHeadings = useMemo(() => {
    return flattenOutlineHeadingsLocal(
      state.headings,
      state.expandedIds,
      options
    );
  }, [state.headings, state.expandedIds, options]);

  // Handle navigation with focus update
  const handleNavigate = useCallback(
    (headingId: string) => {
      navigateToHeading(headingId);
      setFocusedId(headingId);
    },
    [navigateToHeading]
  );

  // Handle promote
  const handlePromote = useCallback(
    async (headingId: string) => {
      try {
        await invoke('promote_heading', { docId, headingId });
        onPromoteExternal?.(headingId);
        refresh();
      } catch (e) {
        console.error('Failed to promote heading:', e);
      }
    },
    [docId, onPromoteExternal, refresh]
  );

  // Handle demote
  const handleDemote = useCallback(
    async (headingId: string) => {
      try {
        await invoke('demote_heading', { docId, headingId });
        onDemoteExternal?.(headingId);
        refresh();
      } catch (e) {
        console.error('Failed to demote heading:', e);
      }
    },
    [docId, onDemoteExternal, refresh]
  );

  // Drag and drop handlers
  const handleDragStart = useCallback((headingId: string) => {
    setDraggedId(headingId);
  }, []);

  const handleDragOver = useCallback((headingId: string) => {
    setDragTargetId(headingId);
  }, []);

  const handleDragEnd = useCallback(() => {
    setDraggedId(null);
    setDragTargetId(null);
  }, []);

  const handleDrop = useCallback(
    async (targetId: string) => {
      if (draggedId && draggedId !== targetId) {
        try {
          await invoke('move_section', {
            docId,
            headingId: draggedId,
            beforeId: targetId,
          });
          onMoveSection?.(draggedId, targetId);
          refresh();
        } catch (e) {
          console.error('Failed to move section:', e);
        }
      }
      handleDragEnd();
    },
    [docId, draggedId, onMoveSection, refresh, handleDragEnd]
  );

  // Keyboard navigation handler
  const handleKeyDown = useCallback(
    (e: KeyboardEvent, headingId: string) => {
      const currentIndex = visibleFilteredHeadings.findIndex((h) => h.id === headingId);
      if (currentIndex === -1) return;

      const current = visibleFilteredHeadings[currentIndex];
      let newFocusId: string | null = null;
      let handled = false;

      switch (e.key) {
        case 'ArrowDown':
          if (currentIndex < visibleFilteredHeadings.length - 1) {
            newFocusId = visibleFilteredHeadings[currentIndex + 1].id;
          }
          handled = true;
          break;

        case 'ArrowUp':
          if (currentIndex > 0) {
            newFocusId = visibleFilteredHeadings[currentIndex - 1].id;
          }
          handled = true;
          break;

        case 'ArrowRight':
          if (current.children.length > 0) {
            if (!state.expandedIds.has(headingId)) {
              expand(headingId);
            } else if (current.children.length > 0) {
              const firstVisibleChild = current.children.find((c) =>
                shouldShowLevel(c.level, options)
              );
              if (firstVisibleChild) {
                newFocusId = firstVisibleChild.id;
              }
            }
          }
          handled = true;
          break;

        case 'ArrowLeft':
          if (state.expandedIds.has(headingId) && current.children.length > 0) {
            collapse(headingId);
          }
          handled = true;
          break;

        case 'Enter':
        case ' ':
          handleNavigate(headingId);
          handled = true;
          break;

        case '+':
        case '=':
          if (e.shiftKey) {
            handlePromote(headingId);
          } else {
            expand(headingId);
          }
          handled = true;
          break;

        case '-':
          if (e.shiftKey) {
            handleDemote(headingId);
          } else {
            collapse(headingId);
          }
          handled = true;
          break;

        case 'Home':
          if (visibleFilteredHeadings.length > 0) {
            newFocusId = visibleFilteredHeadings[0].id;
          }
          handled = true;
          break;

        case 'End':
          if (visibleFilteredHeadings.length > 0) {
            newFocusId = visibleFilteredHeadings[visibleFilteredHeadings.length - 1].id;
          }
          handled = true;
          break;

        case '*':
          expandAll();
          handled = true;
          break;

        case 'Escape':
          if (onExit) {
            onExit();
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
      visibleFilteredHeadings,
      state.expandedIds,
      options,
      expand,
      collapse,
      expandAll,
      handleNavigate,
      handlePromote,
      handleDemote,
      onExit,
    ]
  );

  // Initialize focus
  useEffect(() => {
    if (visibleFilteredHeadings.length > 0 && !focusedId) {
      setFocusedId(state.currentHeadingId || visibleFilteredHeadings[0].id);
    }
  }, [visibleFilteredHeadings, state.currentHeadingId, focusedId]);

  return (
    <div className="outline-view" role="region" aria-label="Document Outline">
      {/* Toolbar */}
      <div className="outline-view-toolbar">
        <div className="outline-view-header">
          <h2>Outline View</h2>
          <span className="heading-count">
            {totalCount} {totalCount === 1 ? 'heading' : 'headings'}
          </span>
        </div>

        {/* Level Filter */}
        {onOptionsChange && (
          <LevelFilter options={options} onOptionsChange={onOptionsChange} />
        )}

        {/* Actions */}
        <div className="outline-view-actions">
          <button
            className="outline-toolbar-btn"
            onClick={expandAll}
            aria-label="Expand all"
            title="Expand all"
            type="button"
          >
            <span aria-hidden="true">+</span>
          </button>
          <button
            className="outline-toolbar-btn"
            onClick={collapseAll}
            aria-label="Collapse all"
            title="Collapse all"
            type="button"
          >
            <span aria-hidden="true">-</span>
          </button>
          <button
            className="outline-toolbar-btn"
            onClick={refresh}
            aria-label="Refresh"
            title="Refresh"
            disabled={isLoading}
            type="button"
          >
            <span aria-hidden="true">{isLoading ? '...' : '\u21BB'}</span>
          </button>
          {onExit && (
            <button
              className="outline-toolbar-btn exit"
              onClick={onExit}
              aria-label="Exit outline view"
              title="Exit outline view"
              type="button"
            >
              <span aria-hidden="true">X</span>
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="outline-view-content">
        {isLoading && filteredHeadings.length === 0 ? (
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
        ) : filteredHeadings.length === 0 ? (
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
            className="outline-view-tree"
            role="tree"
            aria-label="Document headings"
          >
            {filteredHeadings.map((heading) => (
              <OutlineItem
                key={heading.id}
                heading={heading}
                depth={0}
                options={options}
                isExpanded={state.expandedIds.has(heading.id)}
                isCurrent={state.currentHeadingId === heading.id}
                isFocused={focusedId === heading.id}
                isDragTarget={dragTargetId === heading.id}
                onToggleExpand={toggleExpanded}
                onNavigate={handleNavigate}
                onPromote={handlePromote}
                onDemote={handleDemote}
                onKeyDown={handleKeyDown}
                onDragStart={handleDragStart}
                onDragOver={handleDragOver}
                onDragEnd={handleDragEnd}
                onDrop={handleDrop}
                expandedIds={state.expandedIds}
                currentHeadingId={state.currentHeadingId}
                focusedId={focusedId}
                setFocusedId={setFocusedId}
                draggedId={draggedId}
              />
            ))}
          </ul>
        )}
      </div>

      {/* Footer */}
      <div className="outline-view-footer">
        <span className="keyboard-hint">
          <kbd>+/-</kbd> promote/demote
        </span>
        <span className="keyboard-hint">
          <kbd>Enter</kbd> navigate
        </span>
        <span className="keyboard-hint">
          <kbd>Esc</kbd> exit
        </span>
      </div>
    </div>
  );
}

export default OutlineView;
