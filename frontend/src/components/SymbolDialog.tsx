/**
 * SymbolDialog.tsx
 *
 * A full-featured modal dialog for browsing and inserting Unicode characters.
 *
 * Features:
 * - Full Unicode character browser
 * - Browse by Unicode block
 * - Show character code (U+XXXX)
 * - Search by name functionality
 * - Preview of selected character
 * - Recent symbols
 * - Keyboard navigation
 */

import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import {
  useSymbolPicker,
  SymbolInfo,
  SymbolCategory,
  UnicodeBlock,
  UNICODE_BLOCKS,
  COMMON_SYMBOLS,
} from '../hooks/useSymbolPicker';
import './SymbolPicker.css';

// =============================================================================
// Types
// =============================================================================

export interface SymbolDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback to close the dialog */
  onClose: () => void;
  /** Callback when a symbol is inserted */
  onInsertSymbol: (symbol: string) => void;
  /** Initial tab to show */
  initialTab?: 'common' | 'unicode';
}

// =============================================================================
// Component
// =============================================================================

export function SymbolDialog({
  isOpen,
  onClose,
  onInsertSymbol,
  initialTab = 'common',
}: SymbolDialogProps) {
  const {
    filteredSymbols,
    recentSymbols,
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    categories,
    insertSymbol,
    clearRecentSymbols,
    getSymbolInfo,
    unicodeBlocks,
    getBlockCharacters,
  } = useSymbolPicker({
    onInsertSymbol,
  });

  const [activeTab, setActiveTab] = useState<'common' | 'unicode'>(initialTab);
  const [selectedBlock, setSelectedBlock] = useState<UnicodeBlock | null>(null);
  const [blockCharacters, setBlockCharacters] = useState<SymbolInfo[]>([]);
  const [selectedSymbol, setSelectedSymbol] = useState<SymbolInfo | null>(null);
  const [unicodeSearchQuery, setUnicodeSearchQuery] = useState('');
  const [codePointInput, setCodePointInput] = useState('');

  const dialogRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Load block characters when block changes
  useEffect(() => {
    if (selectedBlock) {
      const chars = getBlockCharacters(selectedBlock);
      setBlockCharacters(chars);
    } else {
      setBlockCharacters([]);
    }
  }, [selectedBlock, getBlockCharacters]);

  // Filter Unicode characters by search
  const filteredBlockCharacters = useMemo(() => {
    if (!unicodeSearchQuery.trim()) return blockCharacters;
    const query = unicodeSearchQuery.toLowerCase().trim();
    return blockCharacters.filter(
      (s) =>
        s.name.toLowerCase().includes(query) ||
        s.char.includes(query) ||
        s.code.toLowerCase().includes(query)
    );
  }, [blockCharacters, unicodeSearchQuery]);

  // Handle escape key to close dialog
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
      // Focus search input when dialog opens
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isOpen, onClose]);

  // Handle backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose();
      }
    },
    [onClose]
  );

  // Handle symbol selection
  const handleSymbolSelect = useCallback((symbol: SymbolInfo) => {
    setSelectedSymbol(symbol);
  }, []);

  // Handle symbol double-click (insert)
  const handleSymbolDoubleClick = useCallback(
    (symbol: SymbolInfo) => {
      insertSymbol(symbol.char);
      onClose();
    },
    [insertSymbol, onClose]
  );

  // Handle insert button click
  const handleInsertClick = useCallback(() => {
    if (selectedSymbol) {
      insertSymbol(selectedSymbol.char);
      onClose();
    }
  }, [selectedSymbol, insertSymbol, onClose]);

  // Handle code point input
  const handleCodePointInsert = useCallback(() => {
    if (!codePointInput.trim()) return;

    let codePoint: number;
    const input = codePointInput.trim().toUpperCase();

    // Support formats: "U+XXXX", "0xXXXX", "XXXX" (hex)
    if (input.startsWith('U+')) {
      codePoint = parseInt(input.slice(2), 16);
    } else if (input.startsWith('0X')) {
      codePoint = parseInt(input.slice(2), 16);
    } else {
      codePoint = parseInt(input, 16);
    }

    if (!isNaN(codePoint) && codePoint >= 0 && codePoint <= 0x10FFFF) {
      try {
        const char = String.fromCodePoint(codePoint);
        insertSymbol(char);
        setCodePointInput('');
      } catch {
        // Invalid code point
        console.warn('Invalid code point:', codePoint);
      }
    }
  }, [codePointInput, insertSymbol]);

  // Render symbol button
  const renderSymbolButton = (symbol: SymbolInfo, index: number) => (
    <button
      key={`${symbol.code}-${index}`}
      className={`symbol-button ${selectedSymbol?.char === symbol.char ? 'selected' : ''}`}
      onClick={() => handleSymbolSelect(symbol)}
      onDoubleClick={() => handleSymbolDoubleClick(symbol)}
      title={`${symbol.name} (${symbol.code})`}
      aria-label={symbol.name}
      type="button"
    >
      <span className="symbol-char">{symbol.char}</span>
    </button>
  );

  if (!isOpen) return null;

  return (
    <div className="symbol-dialog-overlay" onClick={handleBackdropClick}>
      <div
        ref={dialogRef}
        className="symbol-dialog"
        role="dialog"
        aria-labelledby="symbol-dialog-title"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Dialog Header */}
        <header className="symbol-dialog-header">
          <h2 id="symbol-dialog-title">Insert Symbol</h2>
          <button
            className="close-button"
            onClick={onClose}
            aria-label="Close dialog"
            type="button"
          >
            X
          </button>
        </header>

        {/* Tab Navigation */}
        <div className="symbol-dialog-tabs">
          <button
            className={`dialog-tab ${activeTab === 'common' ? 'active' : ''}`}
            onClick={() => setActiveTab('common')}
            type="button"
          >
            Common Symbols
          </button>
          <button
            className={`dialog-tab ${activeTab === 'unicode' ? 'active' : ''}`}
            onClick={() => setActiveTab('unicode')}
            type="button"
          >
            Unicode Browser
          </button>
        </div>

        {/* Dialog Content */}
        <div className="symbol-dialog-content">
          {activeTab === 'common' ? (
            // Common Symbols Tab
            <div className="common-symbols-tab">
              {/* Search */}
              <div className="symbol-search-container">
                <input
                  ref={searchInputRef}
                  type="text"
                  className="symbol-search-input"
                  placeholder="Search by name (e.g., 'arrow', 'pi', 'copyright')..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  aria-label="Search symbols"
                />
                {searchQuery && (
                  <button
                    className="symbol-search-clear"
                    onClick={() => setSearchQuery('')}
                    aria-label="Clear search"
                    type="button"
                  >
                    X
                  </button>
                )}
              </div>

              {/* Category Tabs */}
              <div className="symbol-category-tabs" role="tablist">
                <button
                  className={`category-tab ${selectedCategory === null ? 'active' : ''}`}
                  onClick={() => setSelectedCategory(null)}
                  role="tab"
                  aria-selected={selectedCategory === null}
                  type="button"
                >
                  All
                </button>
                {categories.map((cat) => (
                  <button
                    key={cat.id}
                    className={`category-tab ${selectedCategory === cat.id ? 'active' : ''}`}
                    onClick={() => setSelectedCategory(cat.id)}
                    role="tab"
                    aria-selected={selectedCategory === cat.id}
                    title={cat.description}
                    type="button"
                  >
                    {cat.name}
                  </button>
                ))}
              </div>

              {/* Recent Symbols */}
              {recentSymbols.length > 0 && !searchQuery && (
                <div className="symbol-recent-section">
                  <div className="symbol-section-header">
                    <span className="section-title">Recently Used</span>
                    <button
                      className="clear-recent-button"
                      onClick={clearRecentSymbols}
                      type="button"
                    >
                      Clear
                    </button>
                  </div>
                  <div className="symbol-recent-grid">
                    {recentSymbols.map((char, index) => {
                      const info = getSymbolInfo(char);
                      return (
                        <button
                          key={`recent-${index}`}
                          className={`symbol-button ${selectedSymbol?.char === char ? 'selected' : ''}`}
                          onClick={() =>
                            handleSymbolSelect(
                              info || { char, code: '', name: 'Recent', category: 'punctuation' }
                            )
                          }
                          onDoubleClick={() =>
                            handleSymbolDoubleClick(
                              info || { char, code: '', name: 'Recent', category: 'punctuation' }
                            )
                          }
                          title={info?.name || char}
                          type="button"
                        >
                          <span className="symbol-char">{char}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Symbol Grid */}
              <div className="symbol-grid-section">
                <div className="symbol-section-header">
                  <span className="section-title">
                    {selectedCategory
                      ? categories.find((c) => c.id === selectedCategory)?.name
                      : 'All Symbols'}
                    {searchQuery && ` - Results for "${searchQuery}"`}
                  </span>
                  <span className="symbol-count">{filteredSymbols.length} symbols</span>
                </div>
                <div className="symbol-grid-scroll">
                  {filteredSymbols.length > 0 ? (
                    <div className="symbol-grid">
                      {filteredSymbols.map((symbol, index) =>
                        renderSymbolButton(symbol, index)
                      )}
                    </div>
                  ) : (
                    <div className="symbol-empty-state">
                      <p>No symbols found matching your search.</p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ) : (
            // Unicode Browser Tab
            <div className="unicode-browser-tab">
              <div className="unicode-browser-layout">
                {/* Block List */}
                <div className="unicode-block-list">
                  <label className="block-list-label">Unicode Block:</label>
                  <select
                    className="block-select"
                    value={selectedBlock?.name || ''}
                    onChange={(e) => {
                      const block = unicodeBlocks.find((b) => b.name === e.target.value);
                      setSelectedBlock(block || null);
                    }}
                  >
                    <option value="">Select a Unicode block...</option>
                    {unicodeBlocks.map((block) => (
                      <option key={block.name} value={block.name}>
                        {block.name} (U+{block.start.toString(16).toUpperCase().padStart(4, '0')} -{' '}
                        U+{block.end.toString(16).toUpperCase().padStart(4, '0')})
                      </option>
                    ))}
                  </select>

                  {/* Code Point Input */}
                  <div className="code-point-input-section">
                    <label className="code-point-label">Insert by Code Point:</label>
                    <div className="code-point-input-row">
                      <input
                        type="text"
                        className="code-point-input"
                        placeholder="U+XXXX or hex value"
                        value={codePointInput}
                        onChange={(e) => setCodePointInput(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            handleCodePointInsert();
                          }
                        }}
                      />
                      <button
                        className="code-point-insert-button"
                        onClick={handleCodePointInsert}
                        disabled={!codePointInput.trim()}
                        type="button"
                      >
                        Insert
                      </button>
                    </div>
                  </div>
                </div>

                {/* Character Grid */}
                <div className="unicode-character-grid">
                  {selectedBlock ? (
                    <>
                      {/* Search within block */}
                      <div className="unicode-search-container">
                        <input
                          type="text"
                          className="symbol-search-input"
                          placeholder="Search within block..."
                          value={unicodeSearchQuery}
                          onChange={(e) => setUnicodeSearchQuery(e.target.value)}
                        />
                      </div>

                      <div className="symbol-grid-scroll">
                        <div className="symbol-grid unicode-grid">
                          {filteredBlockCharacters.map((symbol, index) =>
                            renderSymbolButton(symbol, index)
                          )}
                        </div>
                      </div>
                    </>
                  ) : (
                    <div className="unicode-empty-state">
                      <p>Select a Unicode block from the dropdown to browse characters.</p>
                      <p>Or enter a code point directly (e.g., U+00A9 for copyright symbol).</p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Preview and Insert Footer */}
        <div className="symbol-dialog-footer">
          <div className="symbol-preview-section">
            {selectedSymbol ? (
              <>
                <div className="preview-character">{selectedSymbol.char}</div>
                <div className="preview-details">
                  <div className="preview-name">{selectedSymbol.name}</div>
                  <div className="preview-code">{selectedSymbol.code}</div>
                </div>
              </>
            ) : (
              <div className="preview-placeholder">
                Click a symbol to select it, double-click to insert
              </div>
            )}
          </div>

          <div className="dialog-actions">
            <button className="cancel-button" onClick={onClose} type="button">
              Cancel
            </button>
            <button
              className="insert-button"
              onClick={handleInsertClick}
              disabled={!selectedSymbol}
              type="button"
            >
              Insert
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default SymbolDialog;
