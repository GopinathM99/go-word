/**
 * ReviewingPane - Sidebar panel for reviewing document revisions
 *
 * Features:
 * - List of all revisions with type icon, author, date, and content preview
 * - Click to navigate to revision
 * - Filter by type and author
 * - Accept/Reject buttons per item
 */

import { useState, useCallback, useMemo } from 'react';
import { Revision, RevisionType, RevisionAuthor } from '../hooks/useTrackChanges';
import './TrackChanges.css';

// =============================================================================
// Types
// =============================================================================

export interface ReviewingPaneProps {
  /** Whether the pane is open */
  isOpen: boolean;
  /** All revisions in the document */
  revisions: Revision[];
  /** Currently selected revision */
  currentRevision: Revision | null;
  /** Callback to close the pane */
  onClose: () => void;
  /** Callback when a revision is clicked */
  onRevisionClick: (revisionId: string) => void;
  /** Callback to accept a revision */
  onAcceptRevision: (revisionId: string) => void;
  /** Callback to reject a revision */
  onRejectRevision: (revisionId: string) => void;
  /** Available authors for filtering */
  authors: RevisionAuthor[];
}

// =============================================================================
// Filter Types
// =============================================================================

type TypeFilter = 'all' | RevisionType;
type AuthorFilter = 'all' | string;

// =============================================================================
// Revision Type Icons
// =============================================================================

function RevisionTypeIcon({ type }: { type: RevisionType }) {
  switch (type) {
    case 'insert':
      return (
        <svg
          className="revision-type-icon revision-type-insert"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      );
    case 'delete':
      return (
        <svg
          className="revision-type-icon revision-type-delete"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      );
    case 'format':
      return (
        <svg
          className="revision-type-icon revision-type-format"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <path d="M4 7V4h16v3" />
          <path d="M9 20h6" />
          <path d="M12 4v16" />
        </svg>
      );
    case 'move':
      return (
        <svg
          className="revision-type-icon revision-type-move"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <path d="M5 9l4-4 4 4" />
          <path d="M9 5v14" />
          <path d="M19 15l-4 4-4-4" />
          <path d="M15 19V5" />
        </svg>
      );
    default:
      return null;
  }
}

// =============================================================================
// Revision Type Labels
// =============================================================================

const REVISION_TYPE_LABELS: Record<RevisionType, string> = {
  insert: 'Insertion',
  delete: 'Deletion',
  format: 'Formatting',
  move: 'Move',
};

// =============================================================================
// Helpers
// =============================================================================

function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return date.toLocaleTimeString(undefined, {
      hour: 'numeric',
      minute: '2-digit',
    });
  } else if (diffDays === 1) {
    return 'Yesterday';
  } else if (diffDays < 7) {
    return date.toLocaleDateString(undefined, { weekday: 'long' });
  } else {
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    });
  }
}

function truncatePreview(text: string, maxLength: number = 60): string {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength - 3) + '...';
}

// =============================================================================
// Component
// =============================================================================

export function ReviewingPane({
  isOpen,
  revisions,
  currentRevision,
  onClose,
  onRevisionClick,
  onAcceptRevision,
  onRejectRevision,
  authors,
}: ReviewingPaneProps) {
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all');
  const [authorFilter, setAuthorFilter] = useState<AuthorFilter>('all');
  const [searchQuery, setSearchQuery] = useState('');

  // Filter revisions
  const filteredRevisions = useMemo(() => {
    return revisions.filter((revision) => {
      // Type filter
      if (typeFilter !== 'all' && revision.type !== typeFilter) {
        return false;
      }

      // Author filter
      if (authorFilter !== 'all' && revision.author.id !== authorFilter) {
        return false;
      }

      // Search query
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        const matchesContent = revision.contentPreview.toLowerCase().includes(query);
        const matchesAuthor = revision.author.name.toLowerCase().includes(query);
        if (!matchesContent && !matchesAuthor) {
          return false;
        }
      }

      return true;
    });
  }, [revisions, typeFilter, authorFilter, searchQuery]);

  // Group revisions by date
  const groupedRevisions = useMemo(() => {
    const groups: Map<string, Revision[]> = new Map();

    filteredRevisions.forEach((revision) => {
      const date = new Date(revision.timestamp);
      const dateKey = date.toLocaleDateString();

      if (!groups.has(dateKey)) {
        groups.set(dateKey, []);
      }
      groups.get(dateKey)!.push(revision);
    });

    return groups;
  }, [filteredRevisions]);

  const handleRevisionClick = useCallback((revision: Revision) => {
    onRevisionClick(revision.id);
  }, [onRevisionClick]);

  const handleAccept = useCallback((e: React.MouseEvent, revisionId: string) => {
    e.stopPropagation();
    onAcceptRevision(revisionId);
  }, [onAcceptRevision]);

  const handleReject = useCallback((e: React.MouseEvent, revisionId: string) => {
    e.stopPropagation();
    onRejectRevision(revisionId);
  }, [onRejectRevision]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  }, [onClose]);

  if (!isOpen) {
    return null;
  }

  return (
    <aside
      className="reviewing-pane"
      role="complementary"
      aria-label="Reviewing pane"
      onKeyDown={handleKeyDown}
    >
      {/* Header */}
      <div className="reviewing-pane-header">
        <h2 className="reviewing-pane-title">Revisions</h2>
        <div className="reviewing-pane-count">
          {filteredRevisions.length}
          {filteredRevisions.length !== revisions.length && ` of ${revisions.length}`}
          {' '}change{revisions.length !== 1 ? 's' : ''}
        </div>
        <button
          className="reviewing-pane-close"
          onClick={onClose}
          aria-label="Close reviewing pane"
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
        </button>
      </div>

      {/* Filters */}
      <div className="reviewing-pane-filters">
        {/* Search */}
        <div className="reviewing-pane-search">
          <svg
            className="search-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden="true"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="text"
            placeholder="Search changes..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            aria-label="Search changes"
          />
          {searchQuery && (
            <button
              className="search-clear"
              onClick={() => setSearchQuery('')}
              aria-label="Clear search"
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
            </button>
          )}
        </div>

        {/* Filter dropdowns */}
        <div className="reviewing-pane-filter-row">
          <select
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value as TypeFilter)}
            aria-label="Filter by type"
          >
            <option value="all">All Types</option>
            <option value="insert">Insertions</option>
            <option value="delete">Deletions</option>
            <option value="format">Formatting</option>
            <option value="move">Moves</option>
          </select>

          <select
            value={authorFilter}
            onChange={(e) => setAuthorFilter(e.target.value as AuthorFilter)}
            aria-label="Filter by author"
          >
            <option value="all">All Authors</option>
            {authors.map((author) => (
              <option key={author.id} value={author.id}>
                {author.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Revision List */}
      <div className="reviewing-pane-content" role="list" aria-label="List of changes">
        {filteredRevisions.length === 0 ? (
          <div className="reviewing-pane-empty">
            {revisions.length === 0 ? (
              <>
                <svg
                  className="empty-icon"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  aria-hidden="true"
                >
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                  <polyline points="22 4 12 14.01 9 11.01" />
                </svg>
                <p>No tracked changes in this document.</p>
              </>
            ) : (
              <>
                <svg
                  className="empty-icon"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  aria-hidden="true"
                >
                  <circle cx="11" cy="11" r="8" />
                  <line x1="21" y1="21" x2="16.65" y2="16.65" />
                </svg>
                <p>No changes match your filters.</p>
              </>
            )}
          </div>
        ) : (
          Array.from(groupedRevisions.entries()).map(([dateKey, dateRevisions]) => (
            <div key={dateKey} className="revision-group">
              <div className="revision-group-header">
                {dateKey === new Date().toLocaleDateString() ? 'Today' : dateKey}
              </div>
              {dateRevisions.map((revision) => (
                <div
                  key={revision.id}
                  className={`revision-item ${currentRevision?.id === revision.id ? 'selected' : ''}`}
                  onClick={() => handleRevisionClick(revision)}
                  role="listitem"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleRevisionClick(revision);
                    }
                  }}
                  aria-selected={currentRevision?.id === revision.id}
                >
                  {/* Type indicator */}
                  <div className="revision-item-type">
                    <RevisionTypeIcon type={revision.type} />
                  </div>

                  {/* Main content */}
                  <div className="revision-item-content">
                    {/* Author and time */}
                    <div className="revision-item-meta">
                      <span
                        className="revision-author"
                        style={{ color: revision.author.color }}
                      >
                        {revision.author.name}
                      </span>
                      <span className="revision-time">
                        {formatTimestamp(revision.timestamp)}
                      </span>
                    </div>

                    {/* Type label */}
                    <div className="revision-item-type-label">
                      {REVISION_TYPE_LABELS[revision.type]}
                      {revision.formatProperty && `: ${revision.formatProperty}`}
                    </div>

                    {/* Content preview */}
                    <div className="revision-item-preview">
                      {revision.type === 'delete' ? (
                        <span className="preview-deleted">
                          {truncatePreview(revision.contentPreview)}
                        </span>
                      ) : revision.type === 'insert' ? (
                        <span className="preview-inserted">
                          {truncatePreview(revision.contentPreview)}
                        </span>
                      ) : (
                        <span>{truncatePreview(revision.contentPreview)}</span>
                      )}
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="revision-item-actions">
                    <button
                      className="revision-action-button accept"
                      onClick={(e) => handleAccept(e, revision.id)}
                      title="Accept this change"
                      aria-label={`Accept ${REVISION_TYPE_LABELS[revision.type].toLowerCase()} by ${revision.author.name}`}
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
                    </button>
                    <button
                      className="revision-action-button reject"
                      onClick={(e) => handleReject(e, revision.id)}
                      title="Reject this change"
                      aria-label={`Reject ${REVISION_TYPE_LABELS[revision.type].toLowerCase()} by ${revision.author.name}`}
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
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ))
        )}
      </div>

      {/* Summary footer */}
      {revisions.length > 0 && (
        <div className="reviewing-pane-footer">
          <div className="revision-summary">
            <span className="summary-item summary-insertions">
              <RevisionTypeIcon type="insert" />
              {revisions.filter(r => r.type === 'insert').length}
            </span>
            <span className="summary-item summary-deletions">
              <RevisionTypeIcon type="delete" />
              {revisions.filter(r => r.type === 'delete').length}
            </span>
            <span className="summary-item summary-formatting">
              <RevisionTypeIcon type="format" />
              {revisions.filter(r => r.type === 'format').length}
            </span>
            {revisions.some(r => r.type === 'move') && (
              <span className="summary-item summary-moves">
                <RevisionTypeIcon type="move" />
                {revisions.filter(r => r.type === 'move').length}
              </span>
            )}
          </div>
        </div>
      )}
    </aside>
  );
}

export default ReviewingPane;
