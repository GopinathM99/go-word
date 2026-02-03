/**
 * ViewModeSelector - Dropdown/tabs for switching between view modes
 *
 * Features:
 * - Dropdown menu with all view modes
 * - Keyboard shortcuts display
 * - Current mode indicator
 * - Icons for each mode
 * - Accessible navigation
 */

import { useState, useRef, useCallback, useEffect, KeyboardEvent } from 'react';
import {
  ViewMode,
  ViewModeInfo,
  VIEW_MODE_INFO,
  getShortcutForPlatform,
} from '../lib/viewModeTypes';
import './ViewModeSelector.css';

// =============================================================================
// Types
// =============================================================================

export interface ViewModeSelectorProps {
  /** Current view mode */
  viewMode: ViewMode;
  /** Callback when view mode changes */
  onViewModeChange: (mode: ViewMode) => void;
  /** Available modes (defaults to all) */
  availableModes?: ViewMode[];
  /** Display style */
  variant?: 'dropdown' | 'tabs' | 'compact';
  /** Show keyboard shortcuts */
  showShortcuts?: boolean;
  /** Disabled state */
  disabled?: boolean;
  /** Custom class name */
  className?: string;
}

// =============================================================================
// Icons
// =============================================================================

function PrintLayoutIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
      <rect x="5" y="3" width="14" height="18" rx="1" />
      <line x1="8" y1="7" x2="16" y2="7" />
      <line x1="8" y1="10" x2="16" y2="10" />
      <line x1="8" y1="13" x2="14" y2="13" />
    </svg>
  );
}

function DraftIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
      <line x1="4" y1="6" x2="20" y2="6" />
      <line x1="4" y1="10" x2="20" y2="10" />
      <line x1="4" y1="14" x2="20" y2="14" />
      <line x1="4" y1="18" x2="16" y2="18" />
    </svg>
  );
}

function OutlineIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
      <line x1="4" y1="6" x2="20" y2="6" />
      <line x1="8" y1="10" x2="20" y2="10" />
      <line x1="8" y1="14" x2="20" y2="14" />
      <line x1="4" y1="18" x2="20" y2="18" />
      <circle cx="4" cy="10" r="1" fill="currentColor" />
      <circle cx="4" cy="14" r="1" fill="currentColor" />
    </svg>
  );
}

function WebLayoutIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
      <circle cx="12" cy="12" r="10" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
    </svg>
  );
}

function getViewModeIcon(mode: ViewMode) {
  switch (mode) {
    case 'print_layout':
      return <PrintLayoutIcon />;
    case 'draft':
      return <DraftIcon />;
    case 'outline':
      return <OutlineIcon />;
    case 'web_layout':
      return <WebLayoutIcon />;
  }
}

// =============================================================================
// Dropdown Variant
// =============================================================================

function ViewModeDropdown({
  viewMode,
  onViewModeChange,
  availableModes,
  showShortcuts,
  disabled,
}: ViewModeSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLUListElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);

  const modes = availableModes || (Object.keys(VIEW_MODE_INFO) as ViewMode[]);
  const currentModeInfo = VIEW_MODE_INFO[viewMode];

  // Close menu when clicking outside
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        buttonRef.current &&
        !buttonRef.current.contains(e.target as Node) &&
        menuRef.current &&
        !menuRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  // Focus first item when menu opens
  useEffect(() => {
    if (isOpen && menuRef.current) {
      const currentIndex = modes.indexOf(viewMode);
      setFocusedIndex(currentIndex >= 0 ? currentIndex : 0);
    }
  }, [isOpen, modes, viewMode]);

  const handleToggle = useCallback(() => {
    if (!disabled) {
      setIsOpen((prev) => !prev);
    }
  }, [disabled]);

  const handleSelect = useCallback(
    (mode: ViewMode) => {
      onViewModeChange(mode);
      setIsOpen(false);
      buttonRef.current?.focus();
    },
    [onViewModeChange]
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (disabled) return;

      switch (e.key) {
        case 'Enter':
        case ' ':
          e.preventDefault();
          if (isOpen && focusedIndex >= 0) {
            handleSelect(modes[focusedIndex]);
          } else {
            setIsOpen(true);
          }
          break;
        case 'Escape':
          e.preventDefault();
          setIsOpen(false);
          buttonRef.current?.focus();
          break;
        case 'ArrowDown':
          e.preventDefault();
          if (!isOpen) {
            setIsOpen(true);
          } else {
            setFocusedIndex((prev) => (prev + 1) % modes.length);
          }
          break;
        case 'ArrowUp':
          e.preventDefault();
          if (isOpen) {
            setFocusedIndex((prev) => (prev - 1 + modes.length) % modes.length);
          }
          break;
        case 'Home':
          e.preventDefault();
          setFocusedIndex(0);
          break;
        case 'End':
          e.preventDefault();
          setFocusedIndex(modes.length - 1);
          break;
      }
    },
    [disabled, isOpen, focusedIndex, modes, handleSelect]
  );

  return (
    <div
      className="view-mode-selector dropdown"
      onKeyDown={handleKeyDown}
    >
      <button
        ref={buttonRef}
        className="view-mode-button"
        onClick={handleToggle}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-label={`View mode: ${currentModeInfo.displayName}`}
        disabled={disabled}
        type="button"
      >
        <span className="view-mode-icon">{getViewModeIcon(viewMode)}</span>
        <span className="view-mode-label">{currentModeInfo.displayName}</span>
        <span className="dropdown-arrow" aria-hidden="true">
          {isOpen ? '\u25B2' : '\u25BC'}
        </span>
      </button>

      {isOpen && (
        <ul
          ref={menuRef}
          className="view-mode-menu"
          role="listbox"
          aria-label="Select view mode"
          tabIndex={-1}
        >
          {modes.map((mode, index) => {
            const modeInfo = VIEW_MODE_INFO[mode];
            const isSelected = mode === viewMode;
            const isFocused = index === focusedIndex;

            return (
              <li
                key={mode}
                role="option"
                aria-selected={isSelected}
                className={`view-mode-option ${isSelected ? 'selected' : ''} ${isFocused ? 'focused' : ''}`}
                onClick={() => handleSelect(mode)}
                onMouseEnter={() => setFocusedIndex(index)}
              >
                <span className="option-icon">{getViewModeIcon(mode)}</span>
                <span className="option-content">
                  <span className="option-name">{modeInfo.displayName}</span>
                  <span className="option-description">{modeInfo.description}</span>
                </span>
                {showShortcuts && (
                  <span className="option-shortcut">
                    {getShortcutForPlatform(mode)}
                  </span>
                )}
                {isSelected && (
                  <span className="option-check" aria-hidden="true">
                    &#10003;
                  </span>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

// =============================================================================
// Tabs Variant
// =============================================================================

function ViewModeTabs({
  viewMode,
  onViewModeChange,
  availableModes,
  showShortcuts,
  disabled,
}: ViewModeSelectorProps) {
  const modes = availableModes || (Object.keys(VIEW_MODE_INFO) as ViewMode[]);
  const tabsRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (disabled) return;

      const currentIndex = modes.indexOf(viewMode);
      let newIndex = currentIndex;

      switch (e.key) {
        case 'ArrowRight':
          e.preventDefault();
          newIndex = (currentIndex + 1) % modes.length;
          break;
        case 'ArrowLeft':
          e.preventDefault();
          newIndex = (currentIndex - 1 + modes.length) % modes.length;
          break;
        case 'Home':
          e.preventDefault();
          newIndex = 0;
          break;
        case 'End':
          e.preventDefault();
          newIndex = modes.length - 1;
          break;
        default:
          return;
      }

      setFocusedIndex(newIndex);
      onViewModeChange(modes[newIndex]);
    },
    [disabled, modes, viewMode, onViewModeChange]
  );

  return (
    <div
      ref={tabsRef}
      className="view-mode-selector tabs"
      role="tablist"
      aria-label="View modes"
      onKeyDown={handleKeyDown}
    >
      {modes.map((mode, index) => {
        const modeInfo = VIEW_MODE_INFO[mode];
        const isSelected = mode === viewMode;

        return (
          <button
            key={mode}
            role="tab"
            aria-selected={isSelected}
            aria-controls={`view-panel-${mode}`}
            className={`view-mode-tab ${isSelected ? 'selected' : ''}`}
            onClick={() => onViewModeChange(mode)}
            disabled={disabled}
            tabIndex={isSelected ? 0 : -1}
            title={showShortcuts ? `${modeInfo.displayName} (${getShortcutForPlatform(mode)})` : modeInfo.displayName}
            type="button"
          >
            <span className="tab-icon">{getViewModeIcon(mode)}</span>
            <span className="tab-label">{modeInfo.displayName}</span>
          </button>
        );
      })}
    </div>
  );
}

// =============================================================================
// Compact Variant
// =============================================================================

function ViewModeCompact({
  viewMode,
  onViewModeChange,
  availableModes,
  showShortcuts,
  disabled,
}: ViewModeSelectorProps) {
  const modes = availableModes || (Object.keys(VIEW_MODE_INFO) as ViewMode[]);

  return (
    <div className="view-mode-selector compact" role="group" aria-label="View modes">
      {modes.map((mode) => {
        const modeInfo = VIEW_MODE_INFO[mode];
        const isSelected = mode === viewMode;

        return (
          <button
            key={mode}
            className={`view-mode-compact-btn ${isSelected ? 'selected' : ''}`}
            onClick={() => onViewModeChange(mode)}
            disabled={disabled}
            aria-pressed={isSelected}
            title={showShortcuts ? `${modeInfo.displayName} (${getShortcutForPlatform(mode)})` : modeInfo.displayName}
            type="button"
          >
            <span className="compact-icon">{getViewModeIcon(mode)}</span>
          </button>
        );
      })}
    </div>
  );
}

// =============================================================================
// Main Component
// =============================================================================

export function ViewModeSelector({
  variant = 'dropdown',
  className = '',
  ...props
}: ViewModeSelectorProps) {
  const Component = {
    dropdown: ViewModeDropdown,
    tabs: ViewModeTabs,
    compact: ViewModeCompact,
  }[variant];

  return (
    <div className={`view-mode-selector-wrapper ${className}`}>
      <Component {...props} />
    </div>
  );
}

export default ViewModeSelector;
