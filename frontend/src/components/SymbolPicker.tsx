/**
 * SymbolPicker.tsx
 *
 * A grid-based symbol picker component for inserting special characters.
 *
 * Features:
 * - Grid of common symbols organized by category
 * - Recent symbols section (stored in localStorage)
 * - Category tabs for navigation
 * - Search by character name
 * - Click to insert symbol
 * - Keyboard navigation support
 */

import { useState, useCallback, useRef, useEffect, KeyboardEvent } from 'react';
import {
  useSymbolPicker,
  SymbolInfo,
  SymbolCategory,
  SYMBOL_CATEGORIES,
} from '../hooks/useSymbolPicker';
import './SymbolPicker.css';

// =============================================================================
// Types
// =============================================================================

export interface SymbolPickerProps {
  /** Callback when a symbol is selected for insertion */
  onInsertSymbol: (symbol: string) => void;
  /** Optional initial category to display */
  initialCategory?: SymbolCategory | null;
  /** Whether to show the recent symbols section */
  showRecent?: boolean;
  /** Whether to show the search bar */
  showSearch?: boolean;
  /** Custom class name */
  className?: string;
  /** Whether the picker is in compact mode */
  compact?: boolean;
}

// =============================================================================
// Component
// =============================================================================

export function SymbolPicker({
  onInsertSymbol,
  initialCategory = null,
  showRecent = true,
  showSearch = true,
  className = '',
  compact = false,
}: SymbolPickerProps) {
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
  } = useSymbolPicker({
    onInsertSymbol,
  });

  const [focusedIndex, setFocusedIndex] = useState(-1);
  const [hoveredSymbol, setHoveredSymbol] = useState<SymbolInfo | null>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Initialize category
  useEffect(() => {
    if (initialCategory) {
      setSelectedCategory(initialCategory);
    }
  }, [initialCategory, setSelectedCategory]);

  // Handle symbol click
  const handleSymbolClick = useCallback(
    (symbol: string) => {
      insertSymbol(symbol);
    },
    [insertSymbol]
  );

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLDivElement>) => {
      const symbols = filteredSymbols;
      const cols = compact ? 8 : 12;

      switch (e.key) {
        case 'ArrowRight':
          e.preventDefault();
          setFocusedIndex((prev) =>
            prev < symbols.length - 1 ? prev + 1 : prev
          );
          break;
        case 'ArrowLeft':
          e.preventDefault();
          setFocusedIndex((prev) => (prev > 0 ? prev - 1 : prev));
          break;
        case 'ArrowDown':
          e.preventDefault();
          setFocusedIndex((prev) =>
            prev + cols < symbols.length ? prev + cols : prev
          );
          break;
        case 'ArrowUp':
          e.preventDefault();
          setFocusedIndex((prev) => (prev - cols >= 0 ? prev - cols : prev));
          break;
        case 'Enter':
        case ' ':
          e.preventDefault();
          if (focusedIndex >= 0 && focusedIndex < symbols.length) {
            handleSymbolClick(symbols[focusedIndex].char);
          }
          break;
        case 'Home':
          e.preventDefault();
          setFocusedIndex(0);
          break;
        case 'End':
          e.preventDefault();
          setFocusedIndex(symbols.length - 1);
          break;
        case 'Tab':
          // Allow normal tab behavior
          break;
        default:
          // If a single printable character is typed, search for it
          if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
            searchInputRef.current?.focus();
          }
          break;
      }
    },
    [filteredSymbols, focusedIndex, handleSymbolClick, compact]
  );

  // Focus the selected symbol button
  useEffect(() => {
    if (focusedIndex >= 0 && gridRef.current) {
      const buttons = gridRef.current.querySelectorAll('.symbol-button');
      const button = buttons[focusedIndex] as HTMLButtonElement;
      if (button) {
        button.focus();
      }
    }
  }, [focusedIndex]);

  // Handle category tab click
  const handleCategoryClick = useCallback(
    (category: SymbolCategory | null) => {
      setSelectedCategory(category);
      setFocusedIndex(-1);
    },
    [setSelectedCategory]
  );

  // Handle search change
  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchQuery(e.target.value);
      setFocusedIndex(-1);
    },
    [setSearchQuery]
  );

  // Clear search
  const handleClearSearch = useCallback(() => {
    setSearchQuery('');
    searchInputRef.current?.focus();
  }, [setSearchQuery]);

  // Render a symbol button
  const renderSymbolButton = (symbol: SymbolInfo, index: number) => (
    <button
      key={`${symbol.char}-${index}`}
      className={`symbol-button ${focusedIndex === index ? 'focused' : ''}`}
      onClick={() => handleSymbolClick(symbol.char)}
      onMouseEnter={() => setHoveredSymbol(symbol)}
      onMouseLeave={() => setHoveredSymbol(null)}
      onFocus={() => setFocusedIndex(index)}
      title={`${symbol.name} (${symbol.code})`}
      aria-label={symbol.name}
      type="button"
    >
      <span className="symbol-char">{symbol.char}</span>
    </button>
  );

  // Render recent symbol button
  const renderRecentSymbolButton = (char: string, index: number) => {
    const info = getSymbolInfo(char);
    return (
      <button
        key={`recent-${char}-${index}`}
        className="symbol-button recent"
        onClick={() => handleSymbolClick(char)}
        onMouseEnter={() =>
          setHoveredSymbol(
            info || { char, code: '', name: 'Recent Symbol', category: 'punctuation' }
          )
        }
        onMouseLeave={() => setHoveredSymbol(null)}
        title={info ? `${info.name} (${info.code})` : char}
        aria-label={info?.name || `Symbol ${char}`}
        type="button"
      >
        <span className="symbol-char">{char}</span>
      </button>
    );
  };

  return (
    <div
      className={`symbol-picker ${compact ? 'compact' : ''} ${className}`}
      role="application"
      aria-label="Symbol Picker"
    >
      {/* Search Bar */}
      {showSearch && (
        <div className="symbol-search-container">
          <input
            ref={searchInputRef}
            type="text"
            className="symbol-search-input"
            placeholder="Search symbols by name..."
            value={searchQuery}
            onChange={handleSearchChange}
            aria-label="Search symbols"
          />
          {searchQuery && (
            <button
              className="symbol-search-clear"
              onClick={handleClearSearch}
              aria-label="Clear search"
              type="button"
            >
              X
            </button>
          )}
        </div>
      )}

      {/* Category Tabs */}
      <div className="symbol-category-tabs" role="tablist" aria-label="Symbol categories">
        <button
          className={`category-tab ${selectedCategory === null ? 'active' : ''}`}
          onClick={() => handleCategoryClick(null)}
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
            onClick={() => handleCategoryClick(cat.id)}
            role="tab"
            aria-selected={selectedCategory === cat.id}
            title={cat.description}
            type="button"
          >
            {cat.name}
          </button>
        ))}
      </div>

      {/* Recent Symbols Section */}
      {showRecent && recentSymbols.length > 0 && !searchQuery && (
        <div className="symbol-recent-section">
          <div className="symbol-section-header">
            <span className="section-title">Recent</span>
            <button
              className="clear-recent-button"
              onClick={clearRecentSymbols}
              aria-label="Clear recent symbols"
              type="button"
            >
              Clear
            </button>
          </div>
          <div className="symbol-recent-grid">
            {recentSymbols.map((char, index) =>
              renderRecentSymbolButton(char, index)
            )}
          </div>
        </div>
      )}

      {/* Symbol Grid */}
      <div
        ref={gridRef}
        className="symbol-grid-container"
        onKeyDown={handleKeyDown}
        role="grid"
        aria-label="Symbols"
      >
        {filteredSymbols.length > 0 ? (
          <div className="symbol-grid" role="row">
            {filteredSymbols.map((symbol, index) =>
              renderSymbolButton(symbol, index)
            )}
          </div>
        ) : (
          <div className="symbol-empty-state">
            {searchQuery ? (
              <p>No symbols found matching "{searchQuery}"</p>
            ) : (
              <p>No symbols available in this category</p>
            )}
          </div>
        )}
      </div>

      {/* Symbol Info Footer */}
      <div className="symbol-info-footer" aria-live="polite">
        {hoveredSymbol ? (
          <>
            <span className="symbol-preview">{hoveredSymbol.char}</span>
            <span className="symbol-name">{hoveredSymbol.name}</span>
            <span className="symbol-code">{hoveredSymbol.code}</span>
          </>
        ) : (
          <span className="symbol-hint">
            Hover over a symbol to see details. Click or press Enter to insert.
          </span>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// Compact Symbol Picker (for toolbar dropdown)
// =============================================================================

export interface CompactSymbolPickerProps {
  /** Callback when a symbol is selected */
  onInsertSymbol: (symbol: string) => void;
  /** Callback when picker should close */
  onClose?: () => void;
  /** Category to show */
  category?: SymbolCategory | null;
}

export function CompactSymbolPicker({
  onInsertSymbol,
  onClose,
  category = null,
}: CompactSymbolPickerProps) {
  const handleInsert = useCallback(
    (symbol: string) => {
      onInsertSymbol(symbol);
      if (onClose) {
        onClose();
      }
    },
    [onInsertSymbol, onClose]
  );

  return (
    <SymbolPicker
      onInsertSymbol={handleInsert}
      initialCategory={category}
      showRecent={true}
      showSearch={false}
      compact={true}
    />
  );
}

// =============================================================================
// Quick Symbol Buttons (for inline toolbar)
// =============================================================================

export interface QuickSymbolButtonsProps {
  /** Callback when a symbol is selected */
  onInsertSymbol: (symbol: string) => void;
  /** Symbols to display */
  symbols?: string[];
}

const DEFAULT_QUICK_SYMBOLS = [
  '\u00A9', // Copyright
  '\u00AE', // Registered
  '\u2122', // Trademark
  '\u2014', // Em dash
  '\u2026', // Ellipsis
  '\u00B0', // Degree
  '\u00B1', // Plus-minus
  '\u00D7', // Multiplication
  '\u00F7', // Division
  '\u2192', // Right arrow
];

export function QuickSymbolButtons({
  onInsertSymbol,
  symbols = DEFAULT_QUICK_SYMBOLS,
}: QuickSymbolButtonsProps) {
  const { getSymbolInfo } = useSymbolPicker({});

  return (
    <div className="quick-symbol-buttons" role="toolbar" aria-label="Quick symbols">
      {symbols.map((char, index) => {
        const info = getSymbolInfo(char);
        return (
          <button
            key={`quick-${char}-${index}`}
            className="quick-symbol-button"
            onClick={() => onInsertSymbol(char)}
            title={info?.name || char}
            aria-label={info?.name || `Insert ${char}`}
            type="button"
          >
            {char}
          </button>
        );
      })}
    </div>
  );
}

export default SymbolPicker;
