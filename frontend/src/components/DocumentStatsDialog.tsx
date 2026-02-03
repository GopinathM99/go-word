/**
 * DocumentStatsDialog - Detailed document statistics modal
 *
 * Features:
 * - Pages, Words, Characters (with spaces), Characters (no spaces)
 * - Paragraphs, Lines
 * - Selection statistics when text is selected
 * - Reading time estimate
 */

import { useCallback } from 'react';
import { DocumentStats, SelectionStats, formatReadingTime, formatNumber } from '../hooks/useDocumentStats';
import './DocumentStatsDialog.css';

// =============================================================================
// Types
// =============================================================================

interface DocumentStatsDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Close the dialog */
  onClose: () => void;
  /** Document statistics */
  documentStats: DocumentStats;
  /** Selection statistics (null if no selection) */
  selectionStats: SelectionStats | null;
  /** Whether stats are currently being calculated */
  isCalculating?: boolean;
}

// =============================================================================
// StatRow Component
// =============================================================================

interface StatRowProps {
  label: string;
  documentValue: number | string;
  selectionValue?: number | string | null;
  showSelection: boolean;
}

function StatRow({ label, documentValue, selectionValue, showSelection }: StatRowProps) {
  const formattedDocValue = typeof documentValue === 'number'
    ? formatNumber(documentValue)
    : documentValue;

  const formattedSelValue = selectionValue != null && typeof selectionValue === 'number'
    ? formatNumber(selectionValue)
    : selectionValue;

  return (
    <tr>
      <td className="stat-label">{label}</td>
      <td className="stat-value document-value">{formattedDocValue}</td>
      {showSelection && (
        <td className="stat-value selection-value">
          {formattedSelValue ?? '-'}
        </td>
      )}
    </tr>
  );
}

// =============================================================================
// DocumentStatsDialog Component
// =============================================================================

export function DocumentStatsDialog({
  isOpen,
  onClose,
  documentStats,
  selectionStats,
  isCalculating = false,
}: DocumentStatsDialogProps) {
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose]
  );

  const handleOverlayClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose();
      }
    },
    [onClose]
  );

  if (!isOpen) {
    return null;
  }

  const hasSelection = selectionStats !== null;
  const readingTime = formatReadingTime(documentStats.readingTimeMinutes);

  return (
    <div
      className="document-stats-overlay"
      onClick={handleOverlayClick}
      onKeyDown={handleKeyDown}
      role="presentation"
    >
      <div
        className="document-stats-dialog"
        role="dialog"
        aria-labelledby="document-stats-title"
        aria-modal="true"
      >
        <header className="document-stats-header">
          <h2 id="document-stats-title">Word Count</h2>
          <button
            className="close-button"
            onClick={onClose}
            aria-label="Close dialog"
            type="button"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <path d="M14 1.41L12.59 0L7 5.59L1.41 0L0 1.41L5.59 7L0 12.59L1.41 14L7 8.41L12.59 14L14 12.59L8.41 7L14 1.41Z" />
            </svg>
          </button>
        </header>

        <div className="document-stats-content">
          {isCalculating && (
            <div className="stats-calculating">
              Calculating statistics...
            </div>
          )}

          <table className="stats-table">
            <thead>
              <tr>
                <th className="stat-header-label">Statistic</th>
                <th className="stat-header-value">Document</th>
                {hasSelection && (
                  <th className="stat-header-value">Selection</th>
                )}
              </tr>
            </thead>
            <tbody>
              <StatRow
                label="Pages"
                documentValue={documentStats.pageCount}
                showSelection={false}
              />
              <StatRow
                label="Words"
                documentValue={documentStats.wordCount}
                selectionValue={selectionStats?.wordCount}
                showSelection={hasSelection}
              />
              <StatRow
                label="Characters (with spaces)"
                documentValue={documentStats.characterCount}
                selectionValue={selectionStats?.characterCount}
                showSelection={hasSelection}
              />
              <StatRow
                label="Characters (no spaces)"
                documentValue={documentStats.characterCountNoSpaces}
                selectionValue={selectionStats?.characterCountNoSpaces}
                showSelection={hasSelection}
              />
              <StatRow
                label="Paragraphs"
                documentValue={documentStats.paragraphCount}
                selectionValue={selectionStats?.paragraphCount}
                showSelection={hasSelection}
              />
              <StatRow
                label="Lines"
                documentValue={documentStats.lineCount}
                showSelection={false}
              />
            </tbody>
          </table>

          <div className="reading-time-section">
            <span className="reading-time-icon">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8-3.6-8-8-8zm0 14c-3.3 0-6-2.7-6-6s2.7-6 6-6 6 2.7 6 6-2.7 6-6 6z" />
                <path d="M8.5 4H7v5l4.3 2.5.7-1.2-3.5-2V4z" />
              </svg>
            </span>
            <span className="reading-time-text">{readingTime}</span>
          </div>

          {hasSelection && (
            <div className="selection-note">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
                <path d="M7 0C3.1 0 0 3.1 0 7s3.1 7 7 7 7-3.1 7-7-3.1-7-7-7zm0 12.6c-3.1 0-5.6-2.5-5.6-5.6S3.9 1.4 7 1.4s5.6 2.5 5.6 5.6-2.5 5.6-5.6 5.6z" />
                <circle cx="7" cy="4.5" r="1" />
                <path d="M7.7 6H6.3v4.5h1.4V6z" />
              </svg>
              <span>Statistics for selected text are shown in the "Selection" column.</span>
            </div>
          )}
        </div>

        <footer className="document-stats-footer">
          <button
            className="close-button-primary"
            onClick={onClose}
            type="button"
          >
            Close
          </button>
        </footer>
      </div>
    </div>
  );
}

export default DocumentStatsDialog;
