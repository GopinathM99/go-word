/**
 * StatusBar - MS Word-style status bar component
 *
 * Features:
 * - Fixed position at bottom of editor window
 * - Left section: Page count, word count, character count
 * - Middle section: Language indicator, spell check status
 * - Right section: Zoom controls (integrate existing ZoomControls)
 * - Responsive design (collapse items on narrow screens)
 */

import { DocumentInfo } from '../lib/types';
import { ZoomControls } from './ZoomControls';
import { ZoomFitMode } from '../hooks/useZoom';
import { DocumentStats, SelectionStats, formatNumber } from '../hooks/useDocumentStats';
import '../styles/StatusBar.css';

// =============================================================================
// Types
// =============================================================================

/** Text direction for BiDi support */
export type TextDirection = 'ltr' | 'rtl' | 'auto';

/** View mode for the editor (compatible with EditorCanvas) */
export type ViewMode = 'print-layout' | 'web-layout' | 'read-mode';

/** Spell check status */
export type SpellCheckStatus = 'idle' | 'checking' | 'has-errors' | 'no-errors' | 'disabled';

/** Proofing language */
export interface ProofingLanguage {
  code: string;
  name: string;
  shortName: string;
}

// =============================================================================
// Props
// =============================================================================

interface StatusBarProps {
  document: DocumentInfo | null;
  /** Current zoom level */
  zoom: number;
  /** Current fit mode */
  fitMode: ZoomFitMode;
  /** Zoom percentage string */
  zoomPercentage: string;
  /** Whether at min zoom */
  isAtMin: boolean;
  /** Whether at max zoom */
  isAtMax: boolean;
  /** Set zoom level */
  onZoomChange: (zoom: number) => void;
  /** Zoom in */
  onZoomIn: () => void;
  /** Zoom out */
  onZoomOut: () => void;
  /** Reset zoom */
  onResetZoom: () => void;
  /** Fit to width */
  onFitToWidth: () => void;
  /** Fit to page */
  onFitToPage: () => void;
  /** Current view mode */
  viewMode: ViewMode;
  /** Toggle view mode */
  onViewModeChange: (mode: ViewMode) => void;
  /** Current paragraph text direction (optional) */
  textDirection?: TextDirection;
  /** Document statistics (optional - for detailed stats) */
  documentStats?: DocumentStats;
  /** Selection statistics (optional) */
  selectionStats?: SelectionStats | null;
  /** Callback when page info is clicked */
  onPageInfoClick?: () => void;
  /** Callback when word count is clicked */
  onWordCountClick?: () => void;
  /** Callback when language indicator is clicked */
  onLanguageClick?: () => void;
  /** Spell check status */
  spellCheckStatus?: SpellCheckStatus;
  /** Callback when spell check status is clicked */
  onSpellCheckClick?: () => void;
  /** Current proofing language */
  proofingLanguage?: ProofingLanguage;
}

// =============================================================================
// Icon Components
// =============================================================================

/** Print Layout Icon */
function PrintLayoutIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <rect x="2" y="1" width="12" height="14" rx="1" stroke="currentColor" strokeWidth="1" fill="none" />
      <line x1="4" y1="4" x2="12" y2="4" stroke="currentColor" strokeWidth="1" />
      <line x1="4" y1="7" x2="12" y2="7" stroke="currentColor" strokeWidth="1" />
      <line x1="4" y1="10" x2="10" y2="10" stroke="currentColor" strokeWidth="1" />
    </svg>
  );
}

/** Web Layout Icon */
function WebLayoutIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <rect x="1" y="2" width="14" height="12" rx="1" stroke="currentColor" strokeWidth="1" fill="none" />
      <line x1="3" y1="5" x2="13" y2="5" stroke="currentColor" strokeWidth="1" />
      <line x1="3" y1="8" x2="13" y2="8" stroke="currentColor" strokeWidth="1" />
      <line x1="3" y1="11" x2="11" y2="11" stroke="currentColor" strokeWidth="1" />
    </svg>
  );
}

/** Read Mode Icon */
function ReadModeIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path
        d="M8 3C5 3 2.5 5 1 8c1.5 3 4 5 7 5s5.5-2 7-5c-1.5-3-4-5-7-5zm0 8c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3z"
        stroke="currentColor"
        strokeWidth="1"
        fill="none"
      />
      <circle cx="8" cy="8" r="1.5" fill="currentColor" />
    </svg>
  );
}

/** Spell Check Icon */
function SpellCheckIcon({ status }: { status: SpellCheckStatus }) {
  const getIconClass = () => {
    switch (status) {
      case 'checking':
        return 'spell-check-icon checking';
      case 'has-errors':
        return 'spell-check-icon has-errors';
      case 'no-errors':
        return 'spell-check-icon no-errors';
      case 'disabled':
        return 'spell-check-icon disabled';
      default:
        return 'spell-check-icon';
    }
  };

  return (
    <span className={getIconClass()} title={getSpellCheckTitle(status)}>
      {status === 'has-errors' ? (
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M1 7h12M7 1v12" stroke="currentColor" strokeWidth="2" transform="rotate(45 7 7)" />
        </svg>
      ) : status === 'no-errors' ? (
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M2 7l3 3 7-7" stroke="currentColor" strokeWidth="2" fill="none" />
        </svg>
      ) : (
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <text x="7" y="11" textAnchor="middle" fontSize="10" fontWeight="bold">
            ABC
          </text>
          <line x1="1" y1="12" x2="13" y2="12" stroke="currentColor" strokeWidth="1.5" strokeDasharray="2,1" />
        </svg>
      )}
    </span>
  );
}

function getSpellCheckTitle(status: SpellCheckStatus): string {
  switch (status) {
    case 'checking':
      return 'Checking spelling...';
    case 'has-errors':
      return 'Spelling errors found';
    case 'no-errors':
      return 'No spelling errors';
    case 'disabled':
      return 'Spell check disabled';
    default:
      return 'Spell check';
  }
}

// =============================================================================
// StatusBar Component
// =============================================================================

export function StatusBar({
  document,
  zoom,
  fitMode,
  zoomPercentage,
  isAtMin,
  isAtMax,
  onZoomChange,
  onZoomIn,
  onZoomOut,
  onResetZoom,
  onFitToWidth,
  onFitToPage,
  viewMode,
  onViewModeChange,
  textDirection,
  documentStats,
  selectionStats,
  onPageInfoClick,
  onWordCountClick,
  onLanguageClick,
  spellCheckStatus = 'idle',
  onSpellCheckClick,
  proofingLanguage,
}: StatusBarProps) {
  // Get direction indicator text
  const getDirectionIndicator = () => {
    switch (textDirection) {
      case 'rtl':
        return 'RTL';
      case 'ltr':
        return 'LTR';
      case 'auto':
        return 'Auto';
      default:
        return null;
    }
  };

  const directionIndicator = getDirectionIndicator();

  // Calculate display values
  const currentPage = document?.currentPage ?? 1;
  const totalPages = documentStats?.pageCount ?? document?.totalPages ?? 1;
  const wordCount = documentStats?.wordCount ?? document?.wordCount ?? 0;
  const language = proofingLanguage?.name ?? document?.language ?? 'English';
  const languageShort = proofingLanguage?.shortName ?? language.substring(0, 2).toUpperCase();

  // Word count display text
  const wordCountText = selectionStats
    ? `${formatNumber(selectionStats.wordCount)} of ${formatNumber(wordCount)} words`
    : `${formatNumber(wordCount)} words`;

  return (
    <footer className="status-bar" id="status-bar" role="contentinfo" aria-label="Document status">
      {/* Left section: Document info */}
      <div className="status-bar-left">
        {/* Page Info */}
        <button
          className={`status-item page-info ${onPageInfoClick ? 'clickable' : ''}`}
          onClick={onPageInfoClick}
          disabled={!onPageInfoClick}
          type="button"
          aria-label={`Page ${currentPage} of ${totalPages}. ${onPageInfoClick ? 'Click to go to page.' : ''}`}
        >
          Page {currentPage} of {totalPages}
        </button>

        {/* Word Count */}
        <button
          className={`status-item word-count ${onWordCountClick ? 'clickable' : ''}`}
          onClick={onWordCountClick}
          disabled={!onWordCountClick}
          type="button"
          aria-label={`${wordCountText}. ${onWordCountClick ? 'Click for detailed statistics.' : ''}`}
        >
          <span className="word-count-value">{wordCountText}</span>
        </button>

        {/* Text Direction (if available) */}
        {directionIndicator && (
          <span className="status-item status-direction" title="Text Direction">
            {directionIndicator}
          </span>
        )}
      </div>

      {/* Middle section: Language and spell check */}
      <div className="status-bar-center">
        {/* Language Indicator */}
        <button
          className={`status-item language ${onLanguageClick ? 'clickable' : ''}`}
          onClick={onLanguageClick}
          disabled={!onLanguageClick}
          type="button"
          aria-label={`Proofing language: ${language}. ${onLanguageClick ? 'Click to change.' : ''}`}
        >
          <span className="language-full">{language}</span>
          <span className="language-short" style={{ display: 'none' }}>{languageShort}</span>
        </button>

        {/* Spell Check Status */}
        <button
          className={`status-item spell-check ${onSpellCheckClick ? 'clickable' : ''}`}
          onClick={onSpellCheckClick}
          disabled={!onSpellCheckClick}
          type="button"
          aria-label={getSpellCheckTitle(spellCheckStatus)}
        >
          <SpellCheckIcon status={spellCheckStatus} />
        </button>
      </div>

      {/* Right section: View mode and Zoom controls */}
      <div className="status-bar-right">
        {/* View mode toggle */}
        <div className="view-mode-toggle" role="group" aria-label="View mode">
          <button
            className={`view-mode-button ${viewMode === 'print-layout' ? 'active' : ''}`}
            onClick={() => onViewModeChange('print-layout')}
            title="Print Layout (Ctrl+Alt+P)"
            aria-label="Print Layout view"
            aria-pressed={viewMode === 'print-layout'}
            type="button"
          >
            <PrintLayoutIcon />
          </button>
          <button
            className={`view-mode-button ${viewMode === 'web-layout' ? 'active' : ''}`}
            onClick={() => onViewModeChange('web-layout')}
            title="Web Layout (Ctrl+Alt+W)"
            aria-label="Web Layout view"
            aria-pressed={viewMode === 'web-layout'}
            type="button"
          >
            <WebLayoutIcon />
          </button>
          <button
            className={`view-mode-button ${viewMode === 'read-mode' ? 'active' : ''}`}
            onClick={() => onViewModeChange('read-mode')}
            title="Read Mode (Ctrl+Alt+R)"
            aria-label="Read Mode view"
            aria-pressed={viewMode === 'read-mode'}
            type="button"
          >
            <ReadModeIcon />
          </button>
        </div>

        {/* Zoom controls */}
        <ZoomControls
          zoom={zoom}
          fitMode={fitMode}
          zoomPercentage={zoomPercentage}
          isAtMin={isAtMin}
          isAtMax={isAtMax}
          onZoomChange={onZoomChange}
          onZoomIn={onZoomIn}
          onZoomOut={onZoomOut}
          onResetZoom={onResetZoom}
          onFitToWidth={onFitToWidth}
          onFitToPage={onFitToPage}
        />
      </div>
    </footer>
  );
}

export default StatusBar;
