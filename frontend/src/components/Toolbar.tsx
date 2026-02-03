import { useState, useCallback, useRef, useEffect } from 'react';
import { SettingsDialog } from './SettingsDialog';
import { ParagraphDialog } from './ParagraphDialog';
import { InsertTableDialog } from './InsertTableDialog';
import { PageSetupDialog, PageSetupSettings } from './PageSetupDialog';
import { ListButton } from './ListGallery';
import { useToolbarNavigation } from '../lib/KeyboardNavigation';

interface ToolbarProps {
  onCommand: (command: string, params?: Record<string, unknown>) => void;
  /** Current formatting state for toggle buttons */
  formattingState?: {
    bold?: boolean;
    italic?: boolean;
    underline?: boolean;
    alignment?: 'left' | 'center' | 'right' | 'justify';
  };
}

export function Toolbar({ onCommand, formattingState }: ToolbarProps) {
  const [showSettings, setShowSettings] = useState(false);
  const [showParagraphDialog, setShowParagraphDialog] = useState(false);
  const [showTableDialog, setShowTableDialog] = useState(false);
  const [showPageSetupDialog, setShowPageSetupDialog] = useState(false);
  const [lineSpacingMenuOpen, setLineSpacingMenuOpen] = useState(false);

  // Ref for toolbar keyboard navigation
  const toolbarRef = useRef<HTMLElement>(null);
  const lineSpacingMenuRef = useRef<HTMLDivElement>(null);

  // Set up arrow key navigation within toolbar
  useToolbarNavigation(toolbarRef);

  // Close line spacing menu when clicking outside
  useEffect(() => {
    if (!lineSpacingMenuOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        lineSpacingMenuRef.current &&
        !lineSpacingMenuRef.current.contains(e.target as Node)
      ) {
        setLineSpacingMenuOpen(false);
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setLineSpacingMenuOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [lineSpacingMenuOpen]);

  // Handle alignment commands with keyboard shortcuts
  const handleAlignLeft = useCallback(() => {
    onCommand('setParagraphAlignment', { alignment: 'left' });
  }, [onCommand]);

  const handleAlignCenter = useCallback(() => {
    onCommand('setParagraphAlignment', { alignment: 'center' });
  }, [onCommand]);

  const handleAlignRight = useCallback(() => {
    onCommand('setParagraphAlignment', { alignment: 'right' });
  }, [onCommand]);

  const handleAlignJustify = useCallback(() => {
    onCommand('setParagraphAlignment', { alignment: 'justify' });
  }, [onCommand]);

  const handleIncreaseIndent = useCallback(() => {
    onCommand('setParagraphIndent', { type: 'increase' });
  }, [onCommand]);

  const handleDecreaseIndent = useCallback(() => {
    onCommand('setParagraphIndent', { type: 'decrease' });
  }, [onCommand]);

  const handleLineSpacing = useCallback((spacing: number) => {
    onCommand('setParagraphSpacing', { lineSpacing: spacing });
    setLineSpacingMenuOpen(false);
  }, [onCommand]);

  const handleInsertTable = useCallback((rows: number, cols: number, width?: number) => {
    onCommand('insertTable', { rows, cols, width });
  }, [onCommand]);

  const handleToggleBulletList = useCallback(() => {
    onCommand('toggleBulletList');
  }, [onCommand]);

  const handleToggleNumberedList = useCallback(() => {
    onCommand('toggleNumberedList');
  }, [onCommand]);

  const handleSelectListStyle = useCallback((numId: number) => {
    onCommand('changeListType', { numId });
  }, [onCommand]);

  const handleRemoveList = useCallback(() => {
    onCommand('removeFromList');
  }, [onCommand]);

  const handlePageSetupApply = useCallback((settings: PageSetupSettings) => {
    onCommand('setPageSetup', {
      paperSize: settings.paperSize,
      customWidth: settings.customWidth,
      customHeight: settings.customHeight,
      orientation: settings.orientation,
      marginTop: settings.marginTop,
      marginBottom: settings.marginBottom,
      marginLeft: settings.marginLeft,
      marginRight: settings.marginRight,
      marginHeader: settings.marginHeader,
      marginFooter: settings.marginFooter,
      gutter: settings.gutter,
      gutterPosition: settings.gutterPosition,
      differentFirstPage: settings.differentFirstPage,
      differentOddEven: settings.differentOddEven,
      lineNumbering: settings.lineNumbering,
    });
  }, [onCommand]);

  // Handle keyboard navigation in line spacing menu
  const handleLineSpacingKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!lineSpacingMenuOpen) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        e.preventDefault();
        setLineSpacingMenuOpen(true);
      }
      return;
    }

    const menu = lineSpacingMenuRef.current;
    if (!menu) return;

    const items = Array.from(menu.querySelectorAll('button')) as HTMLButtonElement[];
    const currentIndex = items.findIndex(item => item === document.activeElement);

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        if (currentIndex < items.length - 1) {
          items[currentIndex + 1]?.focus();
        } else {
          items[0]?.focus();
        }
        break;
      case 'ArrowUp':
        e.preventDefault();
        if (currentIndex > 0) {
          items[currentIndex - 1]?.focus();
        } else {
          items[items.length - 1]?.focus();
        }
        break;
      case 'Escape':
        e.preventDefault();
        setLineSpacingMenuOpen(false);
        break;
      case 'Tab':
        setLineSpacingMenuOpen(false);
        break;
    }
  }, [lineSpacingMenuOpen]);

  return (
    <>
      <header
        ref={toolbarRef}
        className="toolbar"
        role="toolbar"
        aria-label="Document formatting toolbar"
        id="main-toolbar"
      >
        {/* File Operations Group */}
        <div className="toolbar-group" role="group" aria-label="File operations">
          <button
            onClick={() => onCommand('new')}
            aria-label="New document"
            aria-keyshortcuts="Control+N"
          >
            New
          </button>
          <button
            onClick={() => onCommand('open')}
            aria-label="Open document"
            aria-keyshortcuts="Control+O"
          >
            Open
          </button>
          <button
            onClick={() => onCommand('save')}
            aria-label="Save document"
            aria-keyshortcuts="Control+S"
          >
            Save
          </button>
        </div>

        {/* History Group */}
        <div className="toolbar-group" role="group" aria-label="History">
          <button
            onClick={() => onCommand('undo')}
            aria-label="Undo"
            aria-keyshortcuts="Control+Z"
          >
            Undo
          </button>
          <button
            onClick={() => onCommand('redo')}
            aria-label="Redo"
            aria-keyshortcuts="Control+Y"
          >
            Redo
          </button>
        </div>

        {/* Text Formatting Group */}
        <div className="toolbar-group" role="group" aria-label="Text formatting">
          <button
            onClick={() => onCommand('bold')}
            aria-label="Bold"
            aria-keyshortcuts="Control+B"
            aria-pressed={formattingState?.bold ?? false}
          >
            B
          </button>
          <button
            onClick={() => onCommand('italic')}
            aria-label="Italic"
            aria-keyshortcuts="Control+I"
            aria-pressed={formattingState?.italic ?? false}
          >
            I
          </button>
          <button
            onClick={() => onCommand('underline')}
            aria-label="Underline"
            aria-keyshortcuts="Control+U"
            aria-pressed={formattingState?.underline ?? false}
          >
            U
          </button>
        </div>

        {/* Paragraph Alignment Group */}
        <div className="toolbar-group" role="group" aria-label="Paragraph alignment">
          <button
            onClick={handleAlignLeft}
            title="Align Left (Ctrl+L)"
            aria-label="Align left"
            aria-keyshortcuts="Control+L"
            aria-pressed={formattingState?.alignment === 'left'}
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="2" y="2" width="12" height="2" />
              <rect x="2" y="6" width="8" height="2" />
              <rect x="2" y="10" width="12" height="2" />
              <rect x="2" y="14" width="6" height="2" />
            </svg>
          </button>
          <button
            onClick={handleAlignCenter}
            title="Center (Ctrl+E)"
            aria-label="Align center"
            aria-keyshortcuts="Control+E"
            aria-pressed={formattingState?.alignment === 'center'}
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="2" y="2" width="12" height="2" />
              <rect x="4" y="6" width="8" height="2" />
              <rect x="2" y="10" width="12" height="2" />
              <rect x="5" y="14" width="6" height="2" />
            </svg>
          </button>
          <button
            onClick={handleAlignRight}
            title="Align Right (Ctrl+R)"
            aria-label="Align right"
            aria-keyshortcuts="Control+R"
            aria-pressed={formattingState?.alignment === 'right'}
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="2" y="2" width="12" height="2" />
              <rect x="6" y="6" width="8" height="2" />
              <rect x="2" y="10" width="12" height="2" />
              <rect x="8" y="14" width="6" height="2" />
            </svg>
          </button>
          <button
            onClick={handleAlignJustify}
            title="Justify (Ctrl+J)"
            aria-label="Justify"
            aria-keyshortcuts="Control+J"
            aria-pressed={formattingState?.alignment === 'justify'}
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="2" y="2" width="12" height="2" />
              <rect x="2" y="6" width="12" height="2" />
              <rect x="2" y="10" width="12" height="2" />
              <rect x="2" y="14" width="12" height="2" />
            </svg>
          </button>
        </div>

        {/* Indent Group */}
        <div className="toolbar-group" role="group" aria-label="Indentation">
          <button
            onClick={handleDecreaseIndent}
            title="Decrease Indent"
            aria-label="Decrease indent"
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="6" y="2" width="8" height="2" />
              <rect x="6" y="6" width="8" height="2" />
              <rect x="6" y="10" width="8" height="2" />
              <rect x="6" y="14" width="8" height="2" />
              <path d="M4 8L1 5v6z" />
            </svg>
          </button>
          <button
            onClick={handleIncreaseIndent}
            title="Increase Indent"
            aria-label="Increase indent"
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="6" y="2" width="8" height="2" />
              <rect x="6" y="6" width="8" height="2" />
              <rect x="6" y="10" width="8" height="2" />
              <rect x="6" y="14" width="8" height="2" />
              <path d="M1 8l3-3v6z" />
            </svg>
          </button>
        </div>

        {/* List Buttons */}
        <div className="toolbar-group" role="group" aria-label="Lists">
          <ListButton
            isBullet={true}
            onToggle={handleToggleBulletList}
            onSelectStyle={handleSelectListStyle}
            onRemoveList={handleRemoveList}
          />
          <ListButton
            isBullet={false}
            onToggle={handleToggleNumberedList}
            onSelectStyle={handleSelectListStyle}
            onRemoveList={handleRemoveList}
          />
        </div>

        {/* Line Spacing Dropdown */}
        <div className="toolbar-group" role="group" aria-label="Line spacing">
          <div className="toolbar-dropdown" ref={lineSpacingMenuRef}>
            <button
              onClick={() => setLineSpacingMenuOpen(!lineSpacingMenuOpen)}
              onKeyDown={handleLineSpacingKeyDown}
              title="Line Spacing"
              aria-label="Line spacing"
              aria-haspopup="menu"
              aria-expanded={lineSpacingMenuOpen}
              className="toolbar-dropdown-button"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <rect x="4" y="2" width="10" height="1.5" />
                <rect x="4" y="7" width="10" height="1.5" />
                <rect x="4" y="12" width="10" height="1.5" />
                <path d="M2 4l-1 2h2zM2 12l-1-2h2z" />
              </svg>
              <span className="dropdown-arrow" aria-hidden="true">&#9662;</span>
            </button>
            {lineSpacingMenuOpen && (
              <div
                className="toolbar-dropdown-menu"
                role="menu"
                aria-label="Line spacing options"
              >
                <button
                  role="menuitem"
                  onClick={() => handleLineSpacing(1.0)}
                  onKeyDown={handleLineSpacingKeyDown}
                >
                  1.0
                </button>
                <button
                  role="menuitem"
                  onClick={() => handleLineSpacing(1.15)}
                  onKeyDown={handleLineSpacingKeyDown}
                >
                  1.15
                </button>
                <button
                  role="menuitem"
                  onClick={() => handleLineSpacing(1.5)}
                  onKeyDown={handleLineSpacingKeyDown}
                >
                  1.5
                </button>
                <button
                  role="menuitem"
                  onClick={() => handleLineSpacing(2.0)}
                  onKeyDown={handleLineSpacingKeyDown}
                >
                  2.0
                </button>
                <div className="dropdown-divider" role="separator" />
                <button
                  role="menuitem"
                  onClick={() => setShowParagraphDialog(true)}
                  onKeyDown={handleLineSpacingKeyDown}
                >
                  More Options...
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Insert Group */}
        <div className="toolbar-group" role="group" aria-label="Insert elements">
          <button
            onClick={() => onCommand('insertHyperlink')}
            title="Insert Hyperlink (Ctrl+K)"
            aria-label="Insert hyperlink"
            aria-keyshortcuts="Control+K"
          >
            Link
          </button>
          <button
            onClick={() => onCommand('insertBookmark')}
            title="Bookmarks (Ctrl+Shift+F5)"
            aria-label="Insert bookmark"
            aria-keyshortcuts="Control+Shift+F5"
          >
            Bookmark
          </button>
          <button
            onClick={() => setShowTableDialog(true)}
            title="Insert Table"
            aria-label="Insert table"
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="1" y="1" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1" />
              <line x1="1" y1="5" x2="15" y2="5" stroke="currentColor" strokeWidth="1" />
              <line x1="1" y1="9" x2="15" y2="9" stroke="currentColor" strokeWidth="1" />
              <line x1="5" y1="1" x2="5" y2="15" stroke="currentColor" strokeWidth="1" />
              <line x1="10" y1="1" x2="10" y2="15" stroke="currentColor" strokeWidth="1" />
            </svg>
          </button>
          <button
            onClick={() => onCommand('insertImage')}
            title="Insert Image"
            aria-label="Insert image"
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="1" y="2" width="14" height="12" fill="none" stroke="currentColor" strokeWidth="1.5" rx="1" />
              <circle cx="5" cy="6" r="1.5" fill="currentColor" />
              <path d="M2 12l3-4 2 2 4-5 3 4v3H2z" fill="currentColor" opacity="0.7" />
            </svg>
          </button>
          <button
            onClick={() => onCommand('insertSymbol')}
            title="Insert Symbol (Ctrl+Shift+S)"
            aria-label="Insert symbol"
            aria-keyshortcuts="Control+Shift+S"
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <text x="8" y="13" textAnchor="middle" fontSize="14" fontFamily="serif" fontWeight="bold">&#937;</text>
            </svg>
          </button>
        </div>

        {/* Page Layout Group */}
        <div className="toolbar-group" role="group" aria-label="Page layout">
          <button
            onClick={() => setShowPageSetupDialog(true)}
            title="Page Setup"
            aria-label="Page setup"
            className="toolbar-icon-button"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <rect x="2" y="1" width="12" height="14" fill="none" stroke="currentColor" strokeWidth="1.5" rx="1" />
              <line x1="4" y1="4" x2="12" y2="4" stroke="currentColor" strokeWidth="1" strokeDasharray="2,1" />
              <line x1="4" y1="12" x2="12" y2="12" stroke="currentColor" strokeWidth="1" strokeDasharray="2,1" />
              <line x1="4" y1="4" x2="4" y2="12" stroke="currentColor" strokeWidth="1" strokeDasharray="2,1" />
              <line x1="12" y1="4" x2="12" y2="12" stroke="currentColor" strokeWidth="1" strokeDasharray="2,1" />
            </svg>
          </button>
          <button
            onClick={() => onCommand('editHeader')}
            title="Edit Header"
            aria-label="Edit header"
          >
            Header
          </button>
          <button
            onClick={() => onCommand('editFooter')}
            title="Edit Footer"
            aria-label="Edit footer"
          >
            Footer
          </button>
        </div>

        {/* Settings Group */}
        <div className="toolbar-group toolbar-right" role="group" aria-label="Settings">
          <button
            onClick={() => setShowSettings(true)}
            title="Settings"
            aria-label="Open settings"
          >
            Settings
          </button>
        </div>
      </header>

      <SettingsDialog
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />

      <ParagraphDialog
        isOpen={showParagraphDialog}
        onClose={() => setShowParagraphDialog(false)}
        onApply={(settings) => {
          onCommand('setParagraphFormatting', settings);
          setShowParagraphDialog(false);
        }}
      />

      <InsertTableDialog
        isOpen={showTableDialog}
        onClose={() => setShowTableDialog(false)}
        onInsert={handleInsertTable}
      />

      <PageSetupDialog
        isOpen={showPageSetupDialog}
        onClose={() => setShowPageSetupDialog(false)}
        onApply={handlePageSetupApply}
      />
    </>
  );
}
