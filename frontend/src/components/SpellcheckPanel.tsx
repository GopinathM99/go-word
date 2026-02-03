import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './SpellcheckPanel.css';

/**
 * Spelling error from the backend
 */
interface SpellingError {
  paraId: string;
  startOffset: number;
  endOffset: number;
  word: string;
  suggestions: string[];
}

/**
 * Spellcheck results
 */
interface SpellcheckResults {
  errors: SpellingError[];
  currentIndex: number | null;
  wordsChecked: number;
}

interface SpellcheckPanelProps {
  isOpen: boolean;
  onClose: () => void;
  docId?: string;
  /** Language for spell checking */
  language?: string;
  /** Callback when a word is selected */
  onSelectError?: (paraId: string, startOffset: number, endOffset: number) => void;
  /** Callback when the document is modified (correction made) */
  onDocumentChanged?: () => void;
}

export function SpellcheckPanel({
  isOpen,
  onClose,
  docId = 'default',
  language = 'en-US',
  onSelectError,
  onDocumentChanged,
}: SpellcheckPanelProps) {
  const [results, setResults] = useState<SpellcheckResults | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // Run spellcheck when panel opens
  useEffect(() => {
    if (isOpen) {
      runSpellcheck();
    } else {
      setResults(null);
      setError(null);
      setStatusMessage(null);
    }
  }, [isOpen, docId, language]);

  const runSpellcheck = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const spellcheckResults = await invoke<SpellcheckResults>('spellcheck_document', {
        docId,
        language,
      });
      setResults(spellcheckResults);

      if (spellcheckResults.errors.length > 0 && onSelectError) {
        const firstError = spellcheckResults.errors[0];
        onSelectError(firstError.paraId, firstError.startOffset, firstError.endOffset);
      }
    } catch (e) {
      console.error('Spellcheck failed:', e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [docId, language, onSelectError]);

  const getCurrentError = useCallback((): SpellingError | null => {
    if (!results || results.currentIndex === null || results.currentIndex < 0) {
      return null;
    }
    return results.errors[results.currentIndex] || null;
  }, [results]);

  const navigateToError = useCallback(
    (index: number) => {
      if (!results) return;

      const newResults = { ...results, currentIndex: index };
      setResults(newResults);

      const error = results.errors[index];
      if (error && onSelectError) {
        onSelectError(error.paraId, error.startOffset, error.endOffset);
      }
    },
    [results, onSelectError]
  );

  const nextError = useCallback(() => {
    if (!results || results.errors.length === 0) return;

    const current = results.currentIndex ?? -1;
    const next = (current + 1) % results.errors.length;
    navigateToError(next);
  }, [results, navigateToError]);

  const previousError = useCallback(() => {
    if (!results || results.errors.length === 0) return;

    const current = results.currentIndex ?? 0;
    const prev = current <= 0 ? results.errors.length - 1 : current - 1;
    navigateToError(prev);
  }, [results, navigateToError]);

  const ignoreOnce = useCallback(async () => {
    const currentError = getCurrentError();
    if (!currentError) return;

    try {
      await invoke('ignore_spelling_once', {
        docId,
        paraId: currentError.paraId,
        startOffset: currentError.startOffset,
        endOffset: currentError.endOffset,
      });

      // Remove the error from results
      if (results) {
        const newErrors = results.errors.filter((_, i) => i !== results.currentIndex);
        const newIndex =
          newErrors.length === 0
            ? null
            : Math.min(results.currentIndex || 0, newErrors.length - 1);
        setResults({
          ...results,
          errors: newErrors,
          currentIndex: newIndex,
        });

        if (newIndex !== null && newErrors[newIndex] && onSelectError) {
          onSelectError(
            newErrors[newIndex].paraId,
            newErrors[newIndex].startOffset,
            newErrors[newIndex].endOffset
          );
        }
      }

      setStatusMessage('Word ignored');
      setTimeout(() => setStatusMessage(null), 2000);
    } catch (e) {
      console.error('Ignore failed:', e);
      setError(String(e));
    }
  }, [getCurrentError, results, docId, onSelectError]);

  const ignoreAll = useCallback(async () => {
    const currentError = getCurrentError();
    if (!currentError) return;

    try {
      await invoke('ignore_spelling_all', {
        docId,
        word: currentError.word,
      });

      // Remove all errors with this word
      if (results) {
        const wordLower = currentError.word.toLowerCase();
        const newErrors = results.errors.filter(
          (e) => e.word.toLowerCase() !== wordLower
        );
        const newIndex =
          newErrors.length === 0
            ? null
            : Math.min(results.currentIndex || 0, newErrors.length - 1);
        setResults({
          ...results,
          errors: newErrors,
          currentIndex: newIndex,
        });

        if (newIndex !== null && newErrors[newIndex] && onSelectError) {
          onSelectError(
            newErrors[newIndex].paraId,
            newErrors[newIndex].startOffset,
            newErrors[newIndex].endOffset
          );
        }
      }

      setStatusMessage(`"${currentError.word}" ignored in document`);
      setTimeout(() => setStatusMessage(null), 2000);
    } catch (e) {
      console.error('Ignore all failed:', e);
      setError(String(e));
    }
  }, [getCurrentError, results, docId, onSelectError]);

  const addToDictionary = useCallback(async () => {
    const currentError = getCurrentError();
    if (!currentError) return;

    try {
      await invoke('add_to_dictionary', {
        word: currentError.word,
        language,
      });

      // Remove all errors with this word (like ignoreAll)
      if (results) {
        const wordLower = currentError.word.toLowerCase();
        const newErrors = results.errors.filter(
          (e) => e.word.toLowerCase() !== wordLower
        );
        const newIndex =
          newErrors.length === 0
            ? null
            : Math.min(results.currentIndex || 0, newErrors.length - 1);
        setResults({
          ...results,
          errors: newErrors,
          currentIndex: newIndex,
        });

        if (newIndex !== null && newErrors[newIndex] && onSelectError) {
          onSelectError(
            newErrors[newIndex].paraId,
            newErrors[newIndex].startOffset,
            newErrors[newIndex].endOffset
          );
        }
      }

      setStatusMessage(`"${currentError.word}" added to dictionary`);
      setTimeout(() => setStatusMessage(null), 2000);
    } catch (e) {
      console.error('Add to dictionary failed:', e);
      setError(String(e));
    }
  }, [getCurrentError, results, language, onSelectError]);

  const changeTo = useCallback(
    async (suggestion: string) => {
      const currentError = getCurrentError();
      if (!currentError) return;

      try {
        await invoke('correct_spelling', {
          docId,
          paraId: currentError.paraId,
          startOffset: currentError.startOffset,
          endOffset: currentError.endOffset,
          correction: suggestion,
        });

        // Notify that document changed
        if (onDocumentChanged) {
          onDocumentChanged();
        }

        // Re-run spellcheck to update positions
        await runSpellcheck();

        setStatusMessage('Correction applied');
        setTimeout(() => setStatusMessage(null), 2000);
      } catch (e) {
        console.error('Correction failed:', e);
        setError(String(e));
      }
    },
    [getCurrentError, docId, onDocumentChanged, runSpellcheck]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'F7') {
        // F7 closes spellcheck
        onClose();
      }
    },
    [onClose]
  );

  if (!isOpen) {
    return null;
  }

  const currentError = getCurrentError();
  const positionText =
    results && results.errors.length > 0
      ? `${(results.currentIndex || 0) + 1} of ${results.errors.length}`
      : '0 of 0';

  return (
    <div
      className="spellcheck-panel"
      role="dialog"
      aria-labelledby="spellcheck-title"
      onKeyDown={handleKeyDown}
    >
      <div className="spellcheck-header">
        <h2 id="spellcheck-title">Spelling</h2>
        <button className="close-button" onClick={onClose} aria-label="Close">
          X
        </button>
      </div>

      <div className="spellcheck-content">
        {isLoading ? (
          <div className="loading-state">
            <div className="spinner"></div>
            <p>Checking spelling...</p>
          </div>
        ) : error ? (
          <div className="error-state">
            <p>Error: {error}</p>
            <button onClick={runSpellcheck}>Try Again</button>
          </div>
        ) : !results || results.errors.length === 0 ? (
          <div className="complete-state">
            <div className="checkmark">checkmark</div>
            <p>No spelling errors found!</p>
            <p className="words-checked">
              {results ? `${results.wordsChecked} words checked` : ''}
            </p>
          </div>
        ) : (
          <>
            {/* Current error display */}
            <div className="current-error">
              <div className="error-header">
                <span className="error-label">Not in Dictionary:</span>
                <span className="error-position">{positionText}</span>
              </div>
              <div className="error-word">{currentError?.word || ''}</div>
            </div>

            {/* Suggestions list */}
            <div className="suggestions-section">
              <label className="suggestions-label">Suggestions:</label>
              <div className="suggestions-list">
                {currentError?.suggestions && currentError.suggestions.length > 0 ? (
                  currentError.suggestions.map((suggestion, index) => (
                    <button
                      key={index}
                      className="suggestion-item"
                      onClick={() => changeTo(suggestion)}
                      onDoubleClick={() => changeTo(suggestion)}
                    >
                      {suggestion}
                    </button>
                  ))
                ) : (
                  <div className="no-suggestions">No suggestions available</div>
                )}
              </div>
            </div>

            {/* Action buttons */}
            <div className="action-buttons">
              <div className="button-group">
                <button
                  className="action-button"
                  onClick={ignoreOnce}
                  disabled={!currentError}
                  title="Ignore this occurrence"
                >
                  Ignore Once
                </button>
                <button
                  className="action-button"
                  onClick={ignoreAll}
                  disabled={!currentError}
                  title="Ignore all occurrences in document"
                >
                  Ignore All
                </button>
              </div>
              <div className="button-group">
                <button
                  className="action-button primary"
                  onClick={addToDictionary}
                  disabled={!currentError}
                  title="Add to custom dictionary"
                >
                  Add to Dictionary
                </button>
              </div>
            </div>

            {/* Navigation buttons */}
            <div className="navigation-buttons">
              <button
                className="nav-button"
                onClick={previousError}
                disabled={results.errors.length <= 1}
                aria-label="Previous error"
              >
                Previous
              </button>
              <button
                className="nav-button"
                onClick={nextError}
                disabled={results.errors.length <= 1}
                aria-label="Next error"
              >
                Next
              </button>
            </div>
          </>
        )}

        {/* Status message */}
        {statusMessage && (
          <div className="status-message" aria-live="polite">
            {statusMessage}
          </div>
        )}
      </div>
    </div>
  );
}

export default SpellcheckPanel;
