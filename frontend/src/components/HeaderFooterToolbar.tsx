import { useState, useCallback } from 'react';
import './HeaderFooterToolbar.css';

export type HeaderFooterType = 'header' | 'footer';

export interface HeaderFooterOptions {
  differentFirstPage: boolean;
  differentOddEven: boolean;
  linkToPrevious: boolean;
}

interface HeaderFooterToolbarProps {
  type: HeaderFooterType;
  isVisible: boolean;
  options: HeaderFooterOptions;
  onClose: () => void;
  onInsertPageNumber: (format: 'simple' | 'page-x-of-y') => void;
  onInsertDate: () => void;
  onInsertTime: () => void;
  onInsertField: (fieldType: string) => void;
  onToggleDifferentFirstPage: (enabled: boolean) => void;
  onToggleDifferentOddEven: (enabled: boolean) => void;
  onToggleLinkToPrevious: (enabled: boolean) => void;
  onGoToHeader: () => void;
  onGoToFooter: () => void;
}

export function HeaderFooterToolbar({
  type,
  isVisible,
  options,
  onClose,
  onInsertPageNumber,
  onInsertDate,
  onInsertTime,
  onInsertField,
  onToggleDifferentFirstPage,
  onToggleDifferentOddEven,
  onToggleLinkToPrevious,
  onGoToHeader,
  onGoToFooter,
}: HeaderFooterToolbarProps) {
  const [showPageNumberMenu, setShowPageNumberMenu] = useState(false);
  const [showFieldMenu, setShowFieldMenu] = useState(false);

  const handleInsertPageNumber = useCallback((format: 'simple' | 'page-x-of-y') => {
    onInsertPageNumber(format);
    setShowPageNumberMenu(false);
  }, [onInsertPageNumber]);

  const handleInsertField = useCallback((fieldType: string) => {
    onInsertField(fieldType);
    setShowFieldMenu(false);
  }, [onInsertField]);

  if (!isVisible) return null;

  return (
    <div className="header-footer-toolbar">
      <div className="hf-toolbar-section">
        <span className="hf-toolbar-label">
          {type === 'header' ? 'Header' : 'Footer'} - Section 1
        </span>
      </div>

      <div className="hf-toolbar-divider" />

      {/* Insert Group */}
      <div className="hf-toolbar-section">
        <span className="hf-section-title">Insert</span>
        <div className="hf-toolbar-buttons">
          {/* Page Number */}
          <div className="hf-dropdown">
            <button
              className="hf-toolbar-button"
              onClick={() => setShowPageNumberMenu(!showPageNumberMenu)}
              title="Insert Page Number"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <rect x="2" y="2" width="12" height="12" fill="none" stroke="currentColor" strokeWidth="1.5" rx="1" />
                <text x="8" y="11" textAnchor="middle" fontSize="8" fontWeight="bold">#</text>
              </svg>
              <span className="dropdown-arrow">&#9662;</span>
            </button>
            {showPageNumberMenu && (
              <div className="hf-dropdown-menu">
                <button onClick={() => handleInsertPageNumber('simple')}>
                  Page Number
                </button>
                <button onClick={() => handleInsertPageNumber('page-x-of-y')}>
                  Page X of Y
                </button>
              </div>
            )}
          </div>

          {/* Date */}
          <button
            className="hf-toolbar-button"
            onClick={onInsertDate}
            title="Insert Date"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="1" y="3" width="14" height="12" fill="none" stroke="currentColor" strokeWidth="1.5" rx="1" />
              <line x1="1" y1="7" x2="15" y2="7" stroke="currentColor" strokeWidth="1" />
              <line x1="5" y1="1" x2="5" y2="4" stroke="currentColor" strokeWidth="1.5" />
              <line x1="11" y1="1" x2="11" y2="4" stroke="currentColor" strokeWidth="1.5" />
            </svg>
          </button>

          {/* Time */}
          <button
            className="hf-toolbar-button"
            onClick={onInsertTime}
            title="Insert Time"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <circle cx="8" cy="8" r="6.5" fill="none" stroke="currentColor" strokeWidth="1.5" />
              <line x1="8" y1="8" x2="8" y2="4" stroke="currentColor" strokeWidth="1.5" />
              <line x1="8" y1="8" x2="11" y2="8" stroke="currentColor" strokeWidth="1.5" />
            </svg>
          </button>

          {/* Other Fields */}
          <div className="hf-dropdown">
            <button
              className="hf-toolbar-button"
              onClick={() => setShowFieldMenu(!showFieldMenu)}
              title="Insert Field"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M2 4h12M2 8h8M2 12h10" stroke="currentColor" strokeWidth="1.5" fill="none" />
              </svg>
              <span className="dropdown-arrow">&#9662;</span>
            </button>
            {showFieldMenu && (
              <div className="hf-dropdown-menu">
                <button onClick={() => handleInsertField('fileName')}>File Name</button>
                <button onClick={() => handleInsertField('filePath')}>File Path</button>
                <button onClick={() => handleInsertField('author')}>Author</button>
                <button onClick={() => handleInsertField('title')}>Document Title</button>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="hf-toolbar-divider" />

      {/* Options Group */}
      <div className="hf-toolbar-section">
        <span className="hf-section-title">Options</span>
        <div className="hf-toolbar-options">
          <label className="hf-checkbox">
            <input
              type="checkbox"
              checked={options.differentFirstPage}
              onChange={(e) => onToggleDifferentFirstPage(e.target.checked)}
            />
            <span>Different First Page</span>
          </label>
          <label className="hf-checkbox">
            <input
              type="checkbox"
              checked={options.differentOddEven}
              onChange={(e) => onToggleDifferentOddEven(e.target.checked)}
            />
            <span>Different Odd & Even Pages</span>
          </label>
        </div>
      </div>

      <div className="hf-toolbar-divider" />

      {/* Navigation Group */}
      <div className="hf-toolbar-section">
        <span className="hf-section-title">Navigation</span>
        <div className="hf-toolbar-buttons">
          <button
            className="hf-toolbar-button"
            onClick={onGoToHeader}
            title="Go to Header"
            disabled={type === 'header'}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 2L4 6h8L8 2z" />
              <rect x="6" y="6" width="4" height="8" />
            </svg>
          </button>
          <button
            className="hf-toolbar-button"
            onClick={onGoToFooter}
            title="Go to Footer"
            disabled={type === 'footer'}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="6" y="2" width="4" height="8" />
              <path d="M8 14l4-4H4l4 4z" />
            </svg>
          </button>
        </div>
      </div>

      <div className="hf-toolbar-divider" />

      {/* Position Group */}
      <div className="hf-toolbar-section">
        <span className="hf-section-title">Position</span>
        <div className="hf-toolbar-inputs">
          <div className="hf-input-group">
            <label>{type === 'header' ? 'Header from Top:' : 'Footer from Bottom:'}</label>
            <input type="number" defaultValue={0.5} min={0} max={2} step={0.1} />
            <span>"</span>
          </div>
        </div>
      </div>

      <div className="hf-toolbar-spacer" />

      {/* Close Button */}
      <div className="hf-toolbar-section">
        <button
          className="hf-close-button"
          onClick={onClose}
        >
          Close Header and Footer
        </button>
      </div>
    </div>
  );
}

// Component to show header/footer editing boundaries
interface HeaderFooterBoundaryProps {
  type: HeaderFooterType;
  isEditing: boolean;
  label: string;
}

export function HeaderFooterBoundary({ type, isEditing, label }: HeaderFooterBoundaryProps) {
  if (!isEditing) return null;

  return (
    <div className={`hf-boundary ${type} ${isEditing ? 'editing' : ''}`}>
      <div className="hf-boundary-line">
        <span className="hf-boundary-label">{label}</span>
      </div>
    </div>
  );
}
