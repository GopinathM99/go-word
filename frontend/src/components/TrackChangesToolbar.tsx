/**
 * TrackChangesToolbar - Toolbar component for track changes controls
 *
 * Features:
 * - Toggle tracking on/off
 * - View mode selector (Original, Final, All Markup, Simple Markup)
 * - Accept/Reject buttons for selected revision
 * - Accept All / Reject All dropdown
 * - Previous/Next revision navigation
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { MarkupMode, Revision } from '../hooks/useTrackChanges';
import './TrackChanges.css';

// =============================================================================
// Types
// =============================================================================

export interface TrackChangesToolbarProps {
  /** Whether track changes is enabled */
  isTrackingEnabled: boolean;
  /** Current markup viewing mode */
  markupMode: MarkupMode;
  /** Currently selected revision */
  currentRevision: Revision | null;
  /** Total number of revisions */
  revisionCount: number;
  /** Current revision index (0-based) */
  currentRevisionIndex: number;
  /** Callback to toggle tracking */
  onToggleTracking: () => void;
  /** Callback to change markup mode */
  onMarkupModeChange: (mode: MarkupMode) => void;
  /** Callback to accept current revision */
  onAcceptRevision: () => void;
  /** Callback to reject current revision */
  onRejectRevision: () => void;
  /** Callback to accept all revisions */
  onAcceptAll: () => void;
  /** Callback to reject all revisions */
  onRejectAll: () => void;
  /** Callback to navigate to previous revision */
  onPreviousRevision: () => void;
  /** Callback to navigate to next revision */
  onNextRevision: () => void;
  /** Whether to show reviewing pane toggle */
  showReviewingPaneToggle?: boolean;
  /** Whether reviewing pane is open */
  isReviewingPaneOpen?: boolean;
  /** Callback to toggle reviewing pane */
  onToggleReviewingPane?: () => void;
}

// =============================================================================
// Markup Mode Labels
// =============================================================================

const MARKUP_MODE_LABELS: Record<MarkupMode, string> = {
  'original': 'Original',
  'final': 'No Markup',
  'all-markup': 'All Markup',
  'simple-markup': 'Simple Markup',
};

const MARKUP_MODE_DESCRIPTIONS: Record<MarkupMode, string> = {
  'original': 'Show original document without any changes',
  'final': 'Show final document with all changes accepted',
  'all-markup': 'Show all revision markup inline',
  'simple-markup': 'Show simplified revision indicators',
};

// =============================================================================
// Component
// =============================================================================

export function TrackChangesToolbar({
  isTrackingEnabled,
  markupMode,
  currentRevision,
  revisionCount,
  currentRevisionIndex,
  onToggleTracking,
  onMarkupModeChange,
  onAcceptRevision,
  onRejectRevision,
  onAcceptAll,
  onRejectAll,
  onPreviousRevision,
  onNextRevision,
  showReviewingPaneToggle = true,
  isReviewingPaneOpen = false,
  onToggleReviewingPane,
}: TrackChangesToolbarProps) {
  const [markupModeMenuOpen, setMarkupModeMenuOpen] = useState(false);
  const [acceptRejectMenuOpen, setAcceptRejectMenuOpen] = useState(false);

  const markupModeMenuRef = useRef<HTMLDivElement>(null);
  const acceptRejectMenuRef = useRef<HTMLDivElement>(null);

  // Close menus when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (markupModeMenuRef.current && !markupModeMenuRef.current.contains(e.target as Node)) {
        setMarkupModeMenuOpen(false);
      }
      if (acceptRejectMenuRef.current && !acceptRejectMenuRef.current.contains(e.target as Node)) {
        setAcceptRejectMenuOpen(false);
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setMarkupModeMenuOpen(false);
        setAcceptRejectMenuOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, []);

  const handleMarkupModeSelect = useCallback((mode: MarkupMode) => {
    onMarkupModeChange(mode);
    setMarkupModeMenuOpen(false);
  }, [onMarkupModeChange]);

  const handleAcceptAll = useCallback(() => {
    onAcceptAll();
    setAcceptRejectMenuOpen(false);
  }, [onAcceptAll]);

  const handleRejectAll = useCallback(() => {
    onRejectAll();
    setAcceptRejectMenuOpen(false);
  }, [onRejectAll]);

  // Navigation position text
  const positionText = revisionCount > 0
    ? `${currentRevisionIndex + 1} of ${revisionCount}`
    : 'No changes';

  return (
    <div
      className="track-changes-toolbar"
      role="toolbar"
      aria-label="Track changes toolbar"
    >
      {/* Track Changes Toggle */}
      <div className="tc-toolbar-group" role="group" aria-label="Tracking toggle">
        <button
          className={`tc-toolbar-button tc-toggle-button ${isTrackingEnabled ? 'active' : ''}`}
          onClick={onToggleTracking}
          aria-pressed={isTrackingEnabled}
          title={isTrackingEnabled ? 'Turn off track changes' : 'Turn on track changes'}
        >
          <svg
            className="tc-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden="true"
          >
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
          </svg>
          <span className="tc-button-label">Track Changes</span>
          <span className={`tc-status-indicator ${isTrackingEnabled ? 'on' : 'off'}`}>
            {isTrackingEnabled ? 'On' : 'Off'}
          </span>
        </button>
      </div>

      {/* Markup Mode Dropdown */}
      <div className="tc-toolbar-group" role="group" aria-label="View mode">
        <div className="tc-dropdown" ref={markupModeMenuRef}>
          <button
            className="tc-toolbar-button tc-dropdown-button"
            onClick={() => setMarkupModeMenuOpen(!markupModeMenuOpen)}
            aria-haspopup="menu"
            aria-expanded={markupModeMenuOpen}
            title="Change markup view mode"
          >
            <svg
              className="tc-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              aria-hidden="true"
            >
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
            <span className="tc-button-label">{MARKUP_MODE_LABELS[markupMode]}</span>
            <span className="tc-dropdown-arrow" aria-hidden="true">&#9662;</span>
          </button>

          {markupModeMenuOpen && (
            <div className="tc-dropdown-menu" role="menu" aria-label="Markup view options">
              {(Object.keys(MARKUP_MODE_LABELS) as MarkupMode[]).map((mode) => (
                <button
                  key={mode}
                  role="menuitem"
                  className={`tc-dropdown-item ${mode === markupMode ? 'selected' : ''}`}
                  onClick={() => handleMarkupModeSelect(mode)}
                  aria-current={mode === markupMode ? 'true' : undefined}
                >
                  <span className="tc-dropdown-item-label">{MARKUP_MODE_LABELS[mode]}</span>
                  <span className="tc-dropdown-item-description">
                    {MARKUP_MODE_DESCRIPTIONS[mode]}
                  </span>
                  {mode === markupMode && (
                    <span className="tc-dropdown-item-check" aria-hidden="true">&#10003;</span>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Accept/Reject Buttons */}
      <div className="tc-toolbar-group" role="group" aria-label="Accept and reject changes">
        <button
          className="tc-toolbar-button tc-accept-button"
          onClick={onAcceptRevision}
          disabled={!currentRevision}
          title="Accept current change"
          aria-label="Accept current change"
        >
          <svg
            className="tc-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden="true"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <span className="tc-button-label">Accept</span>
        </button>

        <button
          className="tc-toolbar-button tc-reject-button"
          onClick={onRejectRevision}
          disabled={!currentRevision}
          title="Reject current change"
          aria-label="Reject current change"
        >
          <svg
            className="tc-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden="true"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
          <span className="tc-button-label">Reject</span>
        </button>

        {/* Accept/Reject All Dropdown */}
        <div className="tc-dropdown" ref={acceptRejectMenuRef}>
          <button
            className="tc-toolbar-button tc-dropdown-arrow-button"
            onClick={() => setAcceptRejectMenuOpen(!acceptRejectMenuOpen)}
            aria-haspopup="menu"
            aria-expanded={acceptRejectMenuOpen}
            title="More accept/reject options"
            aria-label="More accept and reject options"
          >
            <span className="tc-dropdown-arrow" aria-hidden="true">&#9662;</span>
          </button>

          {acceptRejectMenuOpen && (
            <div className="tc-dropdown-menu tc-dropdown-menu-right" role="menu">
              <button
                role="menuitem"
                className="tc-dropdown-item"
                onClick={handleAcceptAll}
                disabled={revisionCount === 0}
              >
                <svg
                  className="tc-icon tc-icon-accept"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  aria-hidden="true"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <span>Accept All Changes</span>
              </button>
              <button
                role="menuitem"
                className="tc-dropdown-item"
                onClick={handleRejectAll}
                disabled={revisionCount === 0}
              >
                <svg
                  className="tc-icon tc-icon-reject"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  aria-hidden="true"
                >
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
                <span>Reject All Changes</span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Navigation */}
      <div className="tc-toolbar-group" role="group" aria-label="Change navigation">
        <button
          className="tc-toolbar-button tc-nav-button"
          onClick={onPreviousRevision}
          disabled={revisionCount <= 1}
          title="Previous change"
          aria-label="Go to previous change"
        >
          <svg
            className="tc-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden="true"
          >
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>

        <span className="tc-nav-position" aria-label={`Change ${positionText}`}>
          {positionText}
        </span>

        <button
          className="tc-toolbar-button tc-nav-button"
          onClick={onNextRevision}
          disabled={revisionCount <= 1}
          title="Next change"
          aria-label="Go to next change"
        >
          <svg
            className="tc-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            aria-hidden="true"
          >
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
      </div>

      {/* Reviewing Pane Toggle */}
      {showReviewingPaneToggle && onToggleReviewingPane && (
        <div className="tc-toolbar-group tc-toolbar-right" role="group" aria-label="Panels">
          <button
            className={`tc-toolbar-button tc-pane-toggle ${isReviewingPaneOpen ? 'active' : ''}`}
            onClick={onToggleReviewingPane}
            aria-pressed={isReviewingPaneOpen}
            title={isReviewingPaneOpen ? 'Hide reviewing pane' : 'Show reviewing pane'}
          >
            <svg
              className="tc-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              aria-hidden="true"
            >
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <line x1="15" y1="3" x2="15" y2="21" />
            </svg>
            <span className="tc-button-label">Reviewing Pane</span>
          </button>
        </div>
      )}
    </div>
  );
}

export default TrackChangesToolbar;
