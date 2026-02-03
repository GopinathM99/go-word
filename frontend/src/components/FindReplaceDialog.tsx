import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './FindReplaceDialog.css';

/**
 * Find result from the backend
 */
interface FindResult {
  nodeId: string;
  startOffset: number;
  endOffset: number;
  matchedText: string;
  context: string | null;
  matchIndex: number;
}

/**
 * Find options
 */
interface FindOptions {
  caseSensitive: boolean;
  wholeWord: boolean;
  useRegex: boolean;
  wrapAround: boolean;
}

interface FindReplaceDialogProps {
  isOpen: boolean;
  onClose: () => void;
  /** Initial mode (find or replace) */
  initialMode?: 'find' | 'replace';
  /** Current document ID */
  docId?: string;
  /** Callback when selection should change */
  onSelectMatch?: (nodeId: string, startOffset: number, endOffset: number) => void;
}

export function FindReplaceDialog({
  isOpen,
  onClose,
  initialMode = 'find',
  docId = 'default',
  onSelectMatch,
}: FindReplaceDialogProps) {
  const [mode, setMode] = useState<'find' | 'replace'>(initialMode);
  const [findText, setFindText] = useState('');
  const [replaceText, setReplaceText] = useState('');
  const [options, setOptions] = useState<FindOptions>({
    caseSensitive: false,
    wholeWord: false,
    useRegex: false,
    wrapAround: true,
  });
  const [results, setResults] = useState<FindResult[]>([]);
  const [currentIndex, setCurrentIndex] = useState<number>(-1);
  const [isSearching, setIsSearching] = useState(false);
  const [replaceCount, setReplaceCount] = useState<number | null>(null);

  const findInputRef = useRef<HTMLInputElement>(null);

  // Focus the find input when dialog opens
  useEffect(() => {
    if (isOpen && findInputRef.current) {
      findInputRef.current.focus();
      findInputRef.current.select();
    }
  }, [isOpen]);

  // Reset when dialog closes
  useEffect(() => {
    if (!isOpen) {
      setResults([]);
      setCurrentIndex(-1);
      setReplaceCount(null);
    }
  }, [isOpen]);

  // Search when find text or options change
  useEffect(() => {
    if (findText.trim()) {
      performSearch();
    } else {
      setResults([]);
      setCurrentIndex(-1);
    }
  }, [findText, options]);

  const performSearch = useCallback(async () => {
    if (!findText.trim()) {
      setResults([]);
      setCurrentIndex(-1);
      return;
    }

    setIsSearching(true);
    try {
      const searchResults = await invoke<FindResult[]>('find_all', {
        docId,
        pattern: findText,
        options: {
          case_sensitive: options.caseSensitive,
          whole_word: options.wholeWord,
          use_regex: options.useRegex,
          wrap_around: options.wrapAround,
        },
      });

      setResults(searchResults);
      if (searchResults.length > 0) {
        setCurrentIndex(0);
        selectMatch(searchResults[0]);
      } else {
        setCurrentIndex(-1);
      }
    } catch (e) {
      console.error('Search failed:', e);
      setResults([]);
      setCurrentIndex(-1);
    } finally {
      setIsSearching(false);
    }
  }, [findText, options, docId]);

  const selectMatch = useCallback(
    (match: FindResult) => {
      if (onSelectMatch) {
        onSelectMatch(match.nodeId, match.startOffset, match.endOffset);
      }
    },
    [onSelectMatch]
  );

  const findNext = useCallback(() => {
    if (results.length === 0) return;

    const nextIndex = (currentIndex + 1) % results.length;
    setCurrentIndex(nextIndex);
    selectMatch(results[nextIndex]);
  }, [results, currentIndex, selectMatch]);

  const findPrevious = useCallback(() => {
    if (results.length === 0) return;

    const prevIndex = currentIndex <= 0 ? results.length - 1 : currentIndex - 1;
    setCurrentIndex(prevIndex);
    selectMatch(results[prevIndex]);
  }, [results, currentIndex, selectMatch]);

  const replaceCurrent = useCallback(async () => {
    if (currentIndex < 0 || currentIndex >= results.length) return;

    try {
      await invoke('replace_selection', {
        docId,
        replacement: replaceText,
      });

      // Re-search after replacement
      await performSearch();
      setReplaceCount(1);

      // Clear the count message after a delay
      setTimeout(() => setReplaceCount(null), 2000);
    } catch (e) {
      console.error('Replace failed:', e);
    }
  }, [currentIndex, results, replaceText, docId, performSearch]);

  const replaceAll = useCallback(async () => {
    if (!findText.trim()) return;

    try {
      const count = await invoke<number>('replace_all', {
        docId,
        pattern: findText,
        replacement: replaceText,
        options: {
          case_sensitive: options.caseSensitive,
          whole_word: options.wholeWord,
          use_regex: options.useRegex,
          wrap_around: options.wrapAround,
        },
      });

      setReplaceCount(count);
      setResults([]);
      setCurrentIndex(-1);

      // Clear the count message after a delay
      setTimeout(() => setReplaceCount(null), 3000);
    } catch (e) {
      console.error('Replace all failed:', e);
    }
  }, [findText, replaceText, options, docId]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'Enter') {
        if (e.shiftKey) {
          findPrevious();
        } else {
          findNext();
        }
        e.preventDefault();
      } else if (e.key === 'F3') {
        if (e.shiftKey) {
          findPrevious();
        } else {
          findNext();
        }
        e.preventDefault();
      }
    },
    [onClose, findNext, findPrevious]
  );

  const toggleOption = useCallback((option: keyof FindOptions) => {
    setOptions((prev) => ({
      ...prev,
      [option]: !prev[option],
    }));
  }, []);

  if (!isOpen) {
    return null;
  }

  const resultText =
    results.length === 0
      ? findText.trim()
        ? 'No results'
        : ''
      : `${currentIndex + 1} of ${results.length}`;

  return (
    <div
      className="find-replace-dialog"
      role="dialog"
      aria-labelledby="find-replace-title"
      onKeyDown={handleKeyDown}
    >
      <div className="find-replace-header">
        <div className="find-replace-tabs">
          <button
            className={`tab-button ${mode === 'find' ? 'active' : ''}`}
            onClick={() => setMode('find')}
            aria-pressed={mode === 'find'}
          >
            Find
          </button>
          <button
            className={`tab-button ${mode === 'replace' ? 'active' : ''}`}
            onClick={() => setMode('replace')}
            aria-pressed={mode === 'replace'}
          >
            Replace
          </button>
        </div>
        <button className="close-button" onClick={onClose} aria-label="Close">
          X
        </button>
      </div>

      <div className="find-replace-content">
        {/* Find input */}
        <div className="input-row">
          <label htmlFor="find-input" className="sr-only">
            Find
          </label>
          <input
            ref={findInputRef}
            id="find-input"
            type="text"
            className="find-input"
            placeholder="Find"
            value={findText}
            onChange={(e) => setFindText(e.target.value)}
            aria-label="Search text"
          />
          <span className="result-count" aria-live="polite">
            {isSearching ? 'Searching...' : resultText}
          </span>
        </div>

        {/* Replace input (only in replace mode) */}
        {mode === 'replace' && (
          <div className="input-row">
            <label htmlFor="replace-input" className="sr-only">
              Replace with
            </label>
            <input
              id="replace-input"
              type="text"
              className="replace-input"
              placeholder="Replace with"
              value={replaceText}
              onChange={(e) => setReplaceText(e.target.value)}
              aria-label="Replacement text"
            />
          </div>
        )}

        {/* Options */}
        <div className="options-row">
          <label className="option-checkbox">
            <input
              type="checkbox"
              checked={options.caseSensitive}
              onChange={() => toggleOption('caseSensitive')}
            />
            <span className="option-label" title="Match Case (Alt+C)">
              Aa
            </span>
          </label>
          <label className="option-checkbox">
            <input
              type="checkbox"
              checked={options.wholeWord}
              onChange={() => toggleOption('wholeWord')}
            />
            <span className="option-label" title="Match Whole Word (Alt+W)">
              [ab]
            </span>
          </label>
          <label className="option-checkbox">
            <input
              type="checkbox"
              checked={options.useRegex}
              onChange={() => toggleOption('useRegex')}
            />
            <span className="option-label" title="Use Regular Expression (Alt+R)">
              .*
            </span>
          </label>
        </div>

        {/* Action buttons */}
        <div className="actions-row">
          <div className="nav-buttons">
            <button
              className="nav-button"
              onClick={findPrevious}
              disabled={results.length === 0}
              title="Find Previous (Shift+Enter)"
              aria-label="Find previous"
            >
              Previous
            </button>
            <button
              className="nav-button"
              onClick={findNext}
              disabled={results.length === 0}
              title="Find Next (Enter)"
              aria-label="Find next"
            >
              Next
            </button>
          </div>

          {mode === 'replace' && (
            <div className="replace-buttons">
              <button
                className="replace-button"
                onClick={replaceCurrent}
                disabled={currentIndex < 0}
                title="Replace current match"
              >
                Replace
              </button>
              <button
                className="replace-all-button"
                onClick={replaceAll}
                disabled={!findText.trim()}
                title="Replace all matches"
              >
                Replace All
              </button>
            </div>
          )}
        </div>

        {/* Replace count notification */}
        {replaceCount !== null && (
          <div className="replace-notification" aria-live="polite">
            {replaceCount === 0
              ? 'No matches found'
              : replaceCount === 1
                ? '1 replacement made'
                : `${replaceCount} replacements made`}
          </div>
        )}
      </div>
    </div>
  );
}

export default FindReplaceDialog;
