/**
 * GoToDialog - Navigation dialog for jumping to specific locations
 *
 * Features:
 * - Go to Page number
 * - Go to Section
 * - Go to Bookmark (list available bookmarks)
 * - Keyboard shortcut: Ctrl+G
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './GoToDialog.css';

// =============================================================================
// Types
// =============================================================================

type GoToTarget = 'page' | 'section' | 'bookmark' | 'line';

interface BookmarkData {
  id: string;
  name: string;
  isPoint: boolean;
  preview: string | null;
  paragraphId: string;
  offset: number;
}

interface GoToDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Close the dialog */
  onClose: () => void;
  /** Callback when navigating to a page */
  onGoToPage: (pageNumber: number) => void;
  /** Callback when navigating to a section */
  onGoToSection?: (sectionNumber: number) => void;
  /** Callback when navigating to a bookmark */
  onGoToBookmark: (bookmarkName: string) => void;
  /** Callback when navigating to a line */
  onGoToLine?: (lineNumber: number) => void;
  /** Current page number */
  currentPage?: number;
  /** Total page count */
  totalPages?: number;
  /** Total section count */
  totalSections?: number;
  /** Document ID for loading bookmarks */
  docId?: string;
}

// =============================================================================
// GoToDialog Component
// =============================================================================

export function GoToDialog({
  isOpen,
  onClose,
  onGoToPage,
  onGoToSection,
  onGoToBookmark,
  onGoToLine,
  currentPage = 1,
  totalPages = 1,
  totalSections = 1,
  docId = 'default',
}: GoToDialogProps) {
  const [target, setTarget] = useState<GoToTarget>('page');
  const [pageInput, setPageInput] = useState('');
  const [sectionInput, setSectionInput] = useState('');
  const [lineInput, setLineInput] = useState('');
  const [selectedBookmark, setSelectedBookmark] = useState<string | null>(null);
  const [bookmarks, setBookmarks] = useState<BookmarkData[]>([]);
  const [isLoadingBookmarks, setIsLoadingBookmarks] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const inputRef = useRef<HTMLInputElement>(null);

  // Load bookmarks when dialog opens or target changes to bookmark
  useEffect(() => {
    if (isOpen && target === 'bookmark') {
      loadBookmarks();
    }
  }, [isOpen, target, docId]);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setPageInput(currentPage.toString());
      setSectionInput('1');
      setLineInput('1');
      setSelectedBookmark(null);
      setError(null);
      // Focus input after render
      setTimeout(() => {
        inputRef.current?.focus();
        inputRef.current?.select();
      }, 50);
    }
  }, [isOpen, currentPage]);

  const loadBookmarks = async () => {
    setIsLoadingBookmarks(true);
    try {
      const result = await invoke<BookmarkData[]>('list_bookmarks', { docId });
      setBookmarks(result);
    } catch (e) {
      console.error('Failed to load bookmarks:', e);
      setBookmarks([]);
    } finally {
      setIsLoadingBookmarks(false);
    }
  };

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'Enter') {
        handleGoTo();
      }
    },
    [onClose, target, pageInput, sectionInput, lineInput, selectedBookmark]
  );

  const handleGoTo = useCallback(() => {
    setError(null);

    switch (target) {
      case 'page': {
        const pageNum = parseInt(pageInput, 10);
        if (isNaN(pageNum) || pageNum < 1 || pageNum > totalPages) {
          setError(`Please enter a page number between 1 and ${totalPages}`);
          return;
        }
        onGoToPage(pageNum);
        onClose();
        break;
      }
      case 'section': {
        const sectionNum = parseInt(sectionInput, 10);
        if (isNaN(sectionNum) || sectionNum < 1 || sectionNum > totalSections) {
          setError(`Please enter a section number between 1 and ${totalSections}`);
          return;
        }
        if (onGoToSection) {
          onGoToSection(sectionNum);
          onClose();
        }
        break;
      }
      case 'bookmark': {
        if (!selectedBookmark) {
          setError('Please select a bookmark');
          return;
        }
        onGoToBookmark(selectedBookmark);
        onClose();
        break;
      }
      case 'line': {
        const lineNum = parseInt(lineInput, 10);
        if (isNaN(lineNum) || lineNum < 1) {
          setError('Please enter a valid line number');
          return;
        }
        if (onGoToLine) {
          onGoToLine(lineNum);
          onClose();
        }
        break;
      }
    }
  }, [
    target,
    pageInput,
    sectionInput,
    lineInput,
    selectedBookmark,
    totalPages,
    totalSections,
    onGoToPage,
    onGoToSection,
    onGoToBookmark,
    onGoToLine,
    onClose,
  ]);

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

  return (
    <div
      className="goto-dialog-overlay"
      onClick={handleOverlayClick}
      onKeyDown={handleKeyDown}
      role="presentation"
    >
      <div
        className="goto-dialog"
        role="dialog"
        aria-labelledby="goto-dialog-title"
        aria-modal="true"
      >
        <header className="goto-dialog-header">
          <h2 id="goto-dialog-title">Go To</h2>
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

        <div className="goto-dialog-content">
          {/* Target Selection */}
          <div className="goto-targets">
            <label className="goto-target-label">Go to what:</label>
            <ul className="goto-target-list" role="listbox">
              <li
                className={`goto-target-item ${target === 'page' ? 'selected' : ''}`}
                onClick={() => setTarget('page')}
                role="option"
                aria-selected={target === 'page'}
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && setTarget('page')}
              >
                Page
              </li>
              <li
                className={`goto-target-item ${target === 'section' ? 'selected' : ''}`}
                onClick={() => setTarget('section')}
                role="option"
                aria-selected={target === 'section'}
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && setTarget('section')}
              >
                Section
              </li>
              <li
                className={`goto-target-item ${target === 'line' ? 'selected' : ''}`}
                onClick={() => setTarget('line')}
                role="option"
                aria-selected={target === 'line'}
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && setTarget('line')}
              >
                Line
              </li>
              <li
                className={`goto-target-item ${target === 'bookmark' ? 'selected' : ''}`}
                onClick={() => setTarget('bookmark')}
                role="option"
                aria-selected={target === 'bookmark'}
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && setTarget('bookmark')}
              >
                Bookmark
              </li>
            </ul>
          </div>

          {/* Input Section */}
          <div className="goto-input-section">
            {target === 'page' && (
              <div className="goto-input-group">
                <label htmlFor="goto-page-input">Enter page number:</label>
                <input
                  ref={inputRef}
                  id="goto-page-input"
                  type="number"
                  min={1}
                  max={totalPages}
                  value={pageInput}
                  onChange={(e) => setPageInput(e.target.value)}
                  placeholder={`1-${totalPages}`}
                />
                <span className="input-hint">of {totalPages}</span>
              </div>
            )}

            {target === 'section' && (
              <div className="goto-input-group">
                <label htmlFor="goto-section-input">Enter section number:</label>
                <input
                  ref={inputRef}
                  id="goto-section-input"
                  type="number"
                  min={1}
                  max={totalSections}
                  value={sectionInput}
                  onChange={(e) => setSectionInput(e.target.value)}
                  placeholder={`1-${totalSections}`}
                />
                <span className="input-hint">of {totalSections}</span>
              </div>
            )}

            {target === 'line' && (
              <div className="goto-input-group">
                <label htmlFor="goto-line-input">Enter line number:</label>
                <input
                  ref={inputRef}
                  id="goto-line-input"
                  type="number"
                  min={1}
                  value={lineInput}
                  onChange={(e) => setLineInput(e.target.value)}
                  placeholder="1"
                />
              </div>
            )}

            {target === 'bookmark' && (
              <div className="goto-bookmark-section">
                <label>Select bookmark:</label>
                {isLoadingBookmarks ? (
                  <div className="bookmark-loading">Loading bookmarks...</div>
                ) : bookmarks.length === 0 ? (
                  <div className="bookmark-empty">No bookmarks in this document</div>
                ) : (
                  <ul className="bookmark-list" role="listbox">
                    {bookmarks.map((bookmark) => (
                      <li
                        key={bookmark.id}
                        className={`bookmark-item ${selectedBookmark === bookmark.name ? 'selected' : ''}`}
                        onClick={() => setSelectedBookmark(bookmark.name)}
                        onDoubleClick={() => {
                          setSelectedBookmark(bookmark.name);
                          handleGoTo();
                        }}
                        role="option"
                        aria-selected={selectedBookmark === bookmark.name}
                        tabIndex={0}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            setSelectedBookmark(bookmark.name);
                            handleGoTo();
                          }
                        }}
                      >
                        <span className="bookmark-name">{bookmark.name}</span>
                        {bookmark.preview && (
                          <span className="bookmark-preview">{bookmark.preview}</span>
                        )}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}

            {error && <div className="goto-error">{error}</div>}
          </div>
        </div>

        <footer className="goto-dialog-footer">
          <div className="footer-hint">
            <span className="keyboard-hint">Press Ctrl+G to open this dialog</span>
          </div>
          <div className="footer-buttons">
            <button
              className="cancel-button"
              onClick={onClose}
              type="button"
            >
              Close
            </button>
            <button
              className="goto-button"
              onClick={handleGoTo}
              disabled={target === 'bookmark' && !selectedBookmark}
              type="button"
            >
              Go To
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}

// =============================================================================
// Keyboard Shortcut Hook
// =============================================================================

interface UseGoToShortcutOptions {
  onOpen: () => void;
  enabled?: boolean;
}

/**
 * Hook to handle Ctrl+G keyboard shortcut for Go To dialog
 */
export function useGoToShortcut({ onOpen, enabled = true }: UseGoToShortcutOptions): void {
  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modifierKey = isMac ? e.metaKey : e.ctrlKey;

      if (modifierKey && e.key.toLowerCase() === 'g') {
        e.preventDefault();
        onOpen();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [enabled, onOpen]);
}

export default GoToDialog;
