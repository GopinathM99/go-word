import { useCallback, useState, useRef, useEffect } from 'react';
import { EditorCanvas, ViewMode } from './components/EditorCanvas';
import { Toolbar } from './components/Toolbar';
import { StatusBar } from './components/StatusBar';
import { HyperlinkDialog } from './components/HyperlinkDialog';
import { BookmarkDialog } from './components/BookmarkDialog';
import { DocumentStatsDialog } from './components/DocumentStatsDialog';
import { GoToDialog, useGoToShortcut } from './components/GoToDialog';
import { SymbolDialog } from './components/SymbolDialog';
import { SkipLinks, LiveRegionProvider, useLiveRegion } from './components/LiveRegion';
import { useDocument } from './lib/useDocument';
import { useZoom, useZoomShortcuts } from './hooks/useZoom';
import { useDocumentStats } from './hooks/useDocumentStats';
import { useViewModeShortcuts } from './hooks/useViewMode';
import { useFocusManager, FocusRegion, useKeyboardNavigation } from './lib/KeyboardNavigation';
import { useAccessibilityBridge } from './lib/AccessibilityBridge';
import { Command } from './lib/InputController';
import { HyperlinkData, HyperlinkRenderInfo } from './lib/types';
import { open } from '@tauri-apps/plugin-shell';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

// Storage key for view mode preference
const VIEW_MODE_STORAGE_KEY = 'go-word-view-mode';

// Storage key for high contrast preference
const HIGH_CONTRAST_STORAGE_KEY = 'go-word-high-contrast';

function AppContent() {
  const { document, selection, renderModel, executeCommand, newDocument, loadDocument, updateDocumentPath } = useDocument();

  // Live region for announcements
  const liveRegion = useLiveRegion();

  // Container dimensions for fit calculations
  const [containerSize, setContainerSize] = useState({ width: 800, height: 600 });

  // View mode state with persistence (now includes 'read-mode')
  const [viewMode, setViewMode] = useState<ViewMode>(() => {
    try {
      const stored = localStorage.getItem(VIEW_MODE_STORAGE_KEY);
      if (stored === 'print-layout' || stored === 'web-layout' || stored === 'read-mode') {
        return stored;
      }
    } catch {
      // localStorage may not be available
    }
    return 'print-layout';
  });

  // High contrast mode
  const [highContrast, setHighContrast] = useState(() => {
    try {
      const stored = localStorage.getItem(HIGH_CONTRAST_STORAGE_KEY);
      if (stored === 'true') return true;
      // Also check system preference
      return window.matchMedia?.('(prefers-contrast: more)').matches ?? false;
    } catch {
      return false;
    }
  });

  // Apply high contrast class to root element
  useEffect(() => {
    const root = document?.documentElement ?? window.document.documentElement;
    if (highContrast) {
      root.classList.add('theme-high-contrast');
    } else {
      root.classList.remove('theme-high-contrast');
    }
    try {
      localStorage.setItem(HIGH_CONTRAST_STORAGE_KEY, String(highContrast));
    } catch {
      // localStorage may not be available
    }
  }, [highContrast]);

  // Rulers visibility
  const [showRulers, setShowRulers] = useState(true);

  // Hyperlink dialog state
  const [hyperlinkDialogOpen, setHyperlinkDialogOpen] = useState(false);
  const [editingHyperlink, setEditingHyperlink] = useState<HyperlinkRenderInfo | null>(null);
  const [selectedTextForLink, setSelectedTextForLink] = useState('');

  // Bookmark dialog state
  const [bookmarkDialogOpen, setBookmarkDialogOpen] = useState(false);

  // Document stats dialog state
  const [statsDialogOpen, setStatsDialogOpen] = useState(false);

  // Go To dialog state
  const [goToDialogOpen, setGoToDialogOpen] = useState(false);

  // Symbol dialog state
  const [symbolDialogOpen, setSymbolDialogOpen] = useState(false);

  // Formatting state for toolbar toggle buttons
  const [formattingState, setFormattingState] = useState<{
    bold?: boolean;
    italic?: boolean;
    underline?: boolean;
    alignment?: 'left' | 'center' | 'right' | 'justify';
  }>({
    bold: false,
    italic: false,
    underline: false,
    alignment: 'left',
  });

  // Refs for focus management
  const toolbarRef = useRef<HTMLElement>(null);
  const editorRef = useRef<HTMLDivElement>(null);
  const statusBarRef = useRef<HTMLElement>(null);

  // Focus management
  const focusManager = useFocusManager({
    onRegionChange: (region, previousRegion) => {
      // Announce region change to screen readers
      const regionNames: Record<FocusRegion, string> = {
        [FocusRegion.Toolbar]: 'Toolbar',
        [FocusRegion.Canvas]: 'Document Editor',
        [FocusRegion.SidePanel]: 'Side Panel',
        [FocusRegion.Dialog]: 'Dialog',
        [FocusRegion.StatusBar]: 'Status Bar',
        [FocusRegion.Menu]: 'Menu',
      };
      liveRegion.announceNavigation(regionNames[region]);
    },
  });

  // Register focus regions
  useEffect(() => {
    focusManager.registerRegion(FocusRegion.Toolbar, toolbarRef.current);
    focusManager.registerRegion(FocusRegion.Canvas, editorRef.current);
    focusManager.registerRegion(FocusRegion.StatusBar, statusBarRef.current);

    return () => {
      focusManager.unregisterRegion(FocusRegion.Toolbar);
      focusManager.unregisterRegion(FocusRegion.Canvas);
      focusManager.unregisterRegion(FocusRegion.StatusBar);
    };
  }, [focusManager]);

  // Keyboard navigation
  useKeyboardNavigation(focusManager.focusManager!, []);

  // Accessibility bridge for canvas content
  const accessibilityBridge = useAccessibilityBridge(editorRef);

  // Update accessibility tree when render model changes
  useEffect(() => {
    accessibilityBridge.updateFromRenderModel(renderModel);
  }, [renderModel, accessibilityBridge]);

  // Document statistics
  const { documentStats, selectionStats, isCalculating } = useDocumentStats({
    renderModel,
    selection,
    debounceDelay: 300,
  });

  // Get page dimensions from render model
  const pageWidth = renderModel?.pages[0]?.width ?? 816;
  const pageHeight = renderModel?.pages[0]?.height ?? 1056;

  // Initialize zoom state
  const {
    zoom,
    fitMode,
    zoomPercentage,
    isAtMin,
    isAtMax,
    setZoom,
    zoomIn,
    zoomOut,
    resetZoom,
    fitToWidth,
    fitToPage,
    handleWheelZoom,
  } = useZoom({
    documentId: document?.id,
    pageWidth,
    pageHeight,
    containerWidth: containerSize.width,
    containerHeight: containerSize.height,
  });

  // Enable keyboard shortcuts for zoom
  useZoomShortcuts({
    zoomIn,
    zoomOut,
    resetZoom,
    handleWheelZoom,
    enabled: true,
  });

  // Handle view mode changes
  const handleViewModeChange = useCallback((mode: ViewMode) => {
    setViewMode(mode);
    const modeNames: Record<ViewMode, string> = {
      'print-layout': 'Print Layout',
      'web-layout': 'Web Layout',
      'read-mode': 'Read Mode',
    };
    liveRegion.announceDocumentStatus(`View mode changed to ${modeNames[mode]}`);
    try {
      localStorage.setItem(VIEW_MODE_STORAGE_KEY, mode);
    } catch {
      // localStorage may not be available
    }
  }, [liveRegion]);

  // Enable view mode keyboard shortcuts
  useViewModeShortcuts({
    setViewMode: handleViewModeChange,
    enabled: true,
  });

  // Enable Go To dialog keyboard shortcut (Ctrl+G)
  useGoToShortcut({
    onOpen: () => {
      focusManager.saveFocus();
      setGoToDialogOpen(true);
    },
    enabled: true,
  });

  // Handle container resize
  const handleContainerResize = useCallback((width: number, height: number) => {
    setContainerSize({ width, height });
  }, []);

  // Handle commands from the EditorCanvas input controller
  const handleEditorCommand = useCallback(
    (command: Command) => {
      switch (command.type) {
        case 'InsertText':
          executeCommand('InsertText', { text: command.text });
          break;

        case 'DeleteRange':
          executeCommand('DeleteRange', {
            direction: command.direction,
            unit: command.unit,
          });
          break;

        case 'SplitParagraph':
          executeCommand('SplitParagraph');
          break;

        case 'Navigate':
          executeCommand('Navigate', {
            direction: command.direction,
            unit: command.unit,
            extend: command.extend,
          });
          break;

        case 'Copy':
          executeCommand('Copy');
          liveRegion.announceDocumentStatus('Copied to clipboard');
          break;

        case 'Cut':
          executeCommand('Cut');
          liveRegion.announceDocumentStatus('Cut to clipboard');
          break;

        case 'Paste':
          navigator.clipboard.readText().then((text) => {
            if (text) {
              executeCommand('InsertText', { text });
              liveRegion.announceDocumentStatus('Pasted from clipboard');
            }
          }).catch(() => {
            liveRegion.announceError('Clipboard access denied');
          });
          break;

        case 'Undo':
          executeCommand('undo');
          liveRegion.announceDocumentStatus('Undo');
          break;

        case 'Redo':
          executeCommand('redo');
          liveRegion.announceDocumentStatus('Redo');
          break;

        case 'SetCursorPosition':
          executeCommand('SetCursorPosition', {
            paragraph: command.paragraph,
            offset: command.offset,
          });
          break;

        case 'SelectAll':
          executeCommand('SelectAll');
          liveRegion.announceDocumentStatus('All content selected');
          break;

        case 'OpenHyperlinkDialog':
          focusManager.saveFocus();
          setEditingHyperlink(null);
          setSelectedTextForLink('');
          setHyperlinkDialogOpen(true);
          break;

        case 'OpenBookmarkDialog':
          focusManager.saveFocus();
          setBookmarkDialogOpen(true);
          break;

        case 'OpenSymbolDialog':
          focusManager.saveFocus();
          setSymbolDialogOpen(true);
          break;

        default:
          console.warn('Unhandled command:', command);
      }
    },
    [executeCommand, liveRegion, focusManager]
  );

  // Handle symbol insertion
  const handleInsertSymbol = useCallback(
    (symbol: string) => {
      executeCommand('InsertText', { text: symbol });
      liveRegion.announceDocumentStatus(`Symbol ${symbol} inserted`);
    },
    [executeCommand, liveRegion]
  );

  // Close symbol dialog
  const handleCloseSymbolDialog = useCallback(() => {
    setSymbolDialogOpen(false);
    focusManager.restoreFocus();
  }, [focusManager]);

  // Handle hyperlink insertion
  const handleInsertHyperlink = useCallback(
    (data: HyperlinkData) => {
      const params: Record<string, unknown> = {
        targetType: data.targetType,
        tooltip: data.tooltip,
        displayText: data.displayText,
      };

      if (data.targetType === 'external' && data.url) {
        params.url = data.url;
      } else if (data.targetType === 'internal' && data.bookmark) {
        params.bookmark = data.bookmark;
      } else if (data.targetType === 'email' && data.email) {
        params.email = data.email;
        params.subject = data.subject;
      }

      executeCommand('InsertHyperlink', params);
      liveRegion.announceDocumentStatus('Hyperlink inserted');
    },
    [executeCommand, liveRegion]
  );

  // Handle hyperlink update
  const handleUpdateHyperlink = useCallback(
    (data: HyperlinkData) => {
      if (!editingHyperlink) return;

      const params: Record<string, unknown> = {
        hyperlinkId: editingHyperlink.node_id,
        targetType: data.targetType,
        tooltip: data.tooltip,
      };

      if (data.targetType === 'external' && data.url) {
        params.url = data.url;
      } else if (data.targetType === 'internal' && data.bookmark) {
        params.bookmark = data.bookmark;
      } else if (data.targetType === 'email' && data.email) {
        params.email = data.email;
        params.subject = data.subject;
      }

      executeCommand('EditHyperlink', params);
      liveRegion.announceDocumentStatus('Hyperlink updated');
    },
    [executeCommand, editingHyperlink, liveRegion]
  );

  // Handle hyperlink removal
  const handleRemoveHyperlink = useCallback(() => {
    if (editingHyperlink) {
      executeCommand('RemoveHyperlink', { hyperlinkId: editingHyperlink.node_id });
      liveRegion.announceDocumentStatus('Hyperlink removed');
    }
  }, [executeCommand, editingHyperlink, liveRegion]);

  // Handle hyperlink click in the editor
  const handleHyperlinkClick = useCallback(
    async (hyperlink: HyperlinkRenderInfo, ctrlKey: boolean) => {
      if (hyperlink.link_type === 'External') {
        if (ctrlKey) {
          try {
            await open(hyperlink.target);
            liveRegion.announceNavigation('external link');
          } catch (error) {
            liveRegion.announceError('Failed to open link');
          }
        } else {
          const confirmed = window.confirm(
            `Open external link?\n\n${hyperlink.target}\n\nClick OK to open in your default browser.`
          );
          if (confirmed) {
            try {
              await open(hyperlink.target);
              liveRegion.announceNavigation('external link');
            } catch (error) {
              liveRegion.announceError('Failed to open link');
            }
          }
        }
      } else if (hyperlink.link_type === 'Internal') {
        const bookmarkName = hyperlink.target.replace('#', '');
        executeCommand('GoToBookmark', { name: bookmarkName });
        liveRegion.announceNavigation(`bookmark ${bookmarkName}`);
      } else if (hyperlink.link_type === 'Email') {
        try {
          await open(hyperlink.target);
          liveRegion.announceNavigation('email link');
        } catch (error) {
          liveRegion.announceError('Failed to open email link');
        }
      }
    },
    [executeCommand, liveRegion]
  );

  // Extended toolbar command handler
  const handleToolbarCommand = useCallback(
    async (command: string, params?: Record<string, unknown>) => {
      switch (command) {
        case 'new': {
          console.log('[Toolbar] New document requested');
          await newDocument();
          setFormattingState({ bold: false, italic: false, underline: false, alignment: 'left' });
          liveRegion.announceDocumentStatus('New document created');
          window.document.title = 'Untitled - Go Word';
          console.log('[Toolbar] New document created successfully');
          break;
        }
        case 'open': {
          const filePath = await openDialog({
            multiple: false,
            filters: [
              { name: 'Text Files', extensions: ['txt'] },
              { name: 'Word Documents', extensions: ['docx', 'doc'] },
              { name: 'All Files', extensions: ['*'] },
            ],
          });
          if (filePath) {
            await loadDocument(filePath as string);
            setFormattingState({ bold: false, italic: false, underline: false, alignment: 'left' });
            liveRegion.announceDocumentStatus('Document opened');
          }
          break;
        }
        case 'save': {
          if (document?.path) {
            await invoke('save_document', { docId: document.id, path: document.path });
            updateDocumentPath(document.path);
            liveRegion.announceDocumentStatus('Document saved');
          } else {
            // No path yet — show Save As dialog
            const savePath = await saveDialog({
              filters: [
                { name: 'Text Files', extensions: ['txt'] },
                { name: 'All Files', extensions: ['*'] },
              ],
            });
            if (savePath) {
              await invoke('save_document', { docId: document?.id, path: savePath });
              updateDocumentPath(savePath as string);
              liveRegion.announceDocumentStatus('Document saved');
            }
          }
          break;
        }
        case 'undo':
          executeCommand('undo');
          liveRegion.announceDocumentStatus('Undo');
          break;
        case 'redo':
          executeCommand('redo');
          liveRegion.announceDocumentStatus('Redo');
          break;
        case 'zoomIn':
          zoomIn();
          liveRegion.announceDocumentStatus(`Zoom ${Math.round(zoom * 100 * 1.25)}%`);
          break;
        case 'zoomOut':
          zoomOut();
          liveRegion.announceDocumentStatus(`Zoom ${Math.round(zoom * 100 * 0.8)}%`);
          break;
        case 'zoomReset':
          resetZoom();
          liveRegion.announceDocumentStatus('Zoom reset to 100%');
          break;
        case 'fitToWidth':
          fitToWidth();
          liveRegion.announceDocumentStatus('Fit to width');
          break;
        case 'fitToPage':
          fitToPage();
          liveRegion.announceDocumentStatus('Fit to page');
          break;
        case 'toggleRulers':
          setShowRulers((prev) => {
            liveRegion.announceDocumentStatus(prev ? 'Rulers hidden' : 'Rulers shown');
            return !prev;
          });
          break;
        case 'toggleHighContrast':
          setHighContrast((prev) => {
            liveRegion.announceDocumentStatus(prev ? 'High contrast mode disabled' : 'High contrast mode enabled');
            return !prev;
          });
          break;
        case 'viewPrintLayout':
          handleViewModeChange('print-layout');
          break;
        case 'viewWebLayout':
          handleViewModeChange('web-layout');
          break;
        case 'viewReadMode':
          handleViewModeChange('read-mode');
          break;
        case 'insertHyperlink':
          focusManager.saveFocus();
          setEditingHyperlink(null);
          setSelectedTextForLink('');
          setHyperlinkDialogOpen(true);
          break;
        case 'insertBookmark':
        case 'openBookmarkDialog':
          focusManager.saveFocus();
          setBookmarkDialogOpen(true);
          break;
        case 'insertSymbol':
          focusManager.saveFocus();
          setSymbolDialogOpen(true);
          break;
        case 'openGoToDialog':
          focusManager.saveFocus();
          setGoToDialogOpen(true);
          break;
        case 'bold':
          executeCommand('bold', params);
          setFormattingState(prev => ({ ...prev, bold: !prev.bold }));
          liveRegion.announceFormatting('Bold', !formattingState.bold);
          break;
        case 'italic':
          executeCommand('italic', params);
          setFormattingState(prev => ({ ...prev, italic: !prev.italic }));
          liveRegion.announceFormatting('Italic', !formattingState.italic);
          break;
        case 'underline':
          executeCommand('underline', params);
          setFormattingState(prev => ({ ...prev, underline: !prev.underline }));
          liveRegion.announceFormatting('Underline', !formattingState.underline);
          break;
        case 'setParagraphAlignment': {
          executeCommand('SetParagraphAlignment', params);
          const alignment = params?.alignment as 'left' | 'center' | 'right' | 'justify' | undefined;
          if (alignment) {
            setFormattingState(prev => ({ ...prev, alignment }));
            liveRegion.announceFormatting(`Align ${alignment}`, true);
          }
          break;
        }
        case 'setParagraphIndent':
          if (params?.type === 'increase') {
            executeCommand('SetParagraphIndent', { left: 36 });
            liveRegion.announceFormatting('Indent increased', true);
          } else if (params?.type === 'decrease') {
            executeCommand('SetParagraphIndent', { left: -36 });
            liveRegion.announceFormatting('Indent decreased', true);
          } else {
            executeCommand('SetParagraphIndent', params);
          }
          break;
        case 'setParagraphSpacing':
          executeCommand('SetParagraphSpacing', params);
          if (params?.lineSpacing) {
            liveRegion.announceFormatting(`Line spacing ${params.lineSpacing}`, true);
          }
          break;
        case 'setParagraphFormatting':
          if (params) {
            if (params.indentLeft !== undefined || params.indentRight !== undefined || params.indentFirstLine !== undefined) {
              executeCommand('SetParagraphIndent', {
                left: params.indentLeft,
                right: params.indentRight,
                firstLine: params.indentFirstLine,
              });
            }
            if (params.spaceBefore !== undefined || params.spaceAfter !== undefined || params.lineSpacing !== undefined) {
              executeCommand('SetParagraphSpacing', {
                before: params.spaceBefore,
                after: params.spaceAfter,
                lineSpacing: params.lineSpacing,
                lineSpacingType: params.lineSpacingType,
              });
            }
            if (params.keepWithNext !== undefined || params.keepTogether !== undefined ||
                params.pageBreakBefore !== undefined || params.widowControl !== undefined) {
              executeCommand('SetParagraphPagination', {
                keepWithNext: params.keepWithNext,
                keepTogether: params.keepTogether,
                pageBreakBefore: params.pageBreakBefore,
                widowControl: params.widowControl,
              });
            }
            liveRegion.announceDocumentStatus('Paragraph formatting applied');
          }
          break;
        case 'setPageSetup':
          executeCommand('SetPageSetup', params);
          liveRegion.announceDocumentStatus('Page setup applied');
          break;
        case 'editHeader':
          // TODO: Enter header editing mode
          liveRegion.announceDocumentStatus('Header editing mode');
          break;
        case 'editFooter':
          // TODO: Enter footer editing mode
          liveRegion.announceDocumentStatus('Footer editing mode');
          break;
        case 'insertPageNumber':
          executeCommand('InsertPageNumber', params);
          liveRegion.announceDocumentStatus('Page number inserted');
          break;
        case 'setDifferentFirstPage':
          executeCommand('SetDifferentFirstPage', params);
          liveRegion.announceDocumentStatus('Different first page setting changed');
          break;
        case 'setDifferentOddEven':
          executeCommand('SetDifferentOddEven', params);
          liveRegion.announceDocumentStatus('Different odd/even pages setting changed');
          break;
        default:
          executeCommand(command, params);
      }
    },
    [executeCommand, newDocument, loadDocument, updateDocumentPath, document, formattingState, zoomIn, zoomOut, resetZoom, fitToWidth, fitToPage, handleViewModeChange, liveRegion, focusManager, zoom]
  );

  // Close hyperlink dialog
  const handleCloseHyperlinkDialog = useCallback(() => {
    setHyperlinkDialogOpen(false);
    setEditingHyperlink(null);
    setSelectedTextForLink('');
    focusManager.restoreFocus();
  }, [focusManager]);

  // Bookmark dialog handlers
  const handleCloseBookmarkDialog = useCallback(() => {
    setBookmarkDialogOpen(false);
    focusManager.restoreFocus();
  }, [focusManager]);

  const handleInsertBookmark = useCallback(
    (name: string) => {
      executeCommand('InsertBookmark', { name });
      liveRegion.announceDocumentStatus(`Bookmark "${name}" inserted`);
    },
    [executeCommand, liveRegion]
  );

  const handleGoToBookmark = useCallback(
    (name: string) => {
      executeCommand('GoToBookmark', { name });
      liveRegion.announceNavigation(`bookmark ${name}`);
    },
    [executeCommand, liveRegion]
  );

  const handleDeleteBookmark = useCallback(
    (name: string) => {
      executeCommand('DeleteBookmark', { name });
      liveRegion.announceDocumentStatus(`Bookmark "${name}" deleted`);
    },
    [executeCommand, liveRegion]
  );

  // Status bar handlers
  const handlePageInfoClick = useCallback(() => {
    focusManager.saveFocus();
    setGoToDialogOpen(true);
  }, [focusManager]);

  const handleWordCountClick = useCallback(() => {
    focusManager.saveFocus();
    setStatsDialogOpen(true);
  }, [focusManager]);

  // Go To dialog handlers
  const handleCloseGoToDialog = useCallback(() => {
    setGoToDialogOpen(false);
    focusManager.restoreFocus();
  }, [focusManager]);

  const handleGoToPage = useCallback(
    (pageNumber: number) => {
      executeCommand('GoToPage', { page: pageNumber });
      liveRegion.announceNavigation(`page ${pageNumber}`);
    },
    [executeCommand, liveRegion]
  );

  const handleGoToSection = useCallback(
    (sectionNumber: number) => {
      executeCommand('GoToSection', { section: sectionNumber });
      liveRegion.announceNavigation(`section ${sectionNumber}`);
    },
    [executeCommand, liveRegion]
  );

  // Stats dialog handlers
  const handleCloseStatsDialog = useCallback(() => {
    setStatsDialogOpen(false);
    focusManager.restoreFocus();
  }, [focusManager]);

  return (
    <div className="app" role="application" aria-label="Document Editor">
      {/* Skip links for keyboard navigation */}
      <SkipLinks
        links={[
          { label: 'Skip to toolbar', targetId: 'main-toolbar' },
          { label: 'Skip to document', targetId: 'main-content' },
          { label: 'Skip to status bar', targetId: 'status-bar' },
        ]}
      />

      <Toolbar onCommand={handleToolbarCommand} formattingState={formattingState} />

      <main id="main-content" ref={editorRef as React.RefObject<HTMLDivElement>} style={{ flex: 1, display: 'flex', overflow: 'hidden', minHeight: 0 }}>
        <EditorCanvas
          renderModel={renderModel}
          selection={selection}
          onCommand={handleEditorCommand}
          zoom={zoom}
          viewMode={viewMode === 'read-mode' ? 'web-layout' : viewMode}
          showRulers={showRulers && viewMode === 'print-layout'}
          onContainerResize={handleContainerResize}
          onWheelZoom={handleWheelZoom}
          onHyperlinkClick={handleHyperlinkClick}
        />
      </main>

      <HyperlinkDialog
        isOpen={hyperlinkDialogOpen}
        onClose={handleCloseHyperlinkDialog}
        onInsert={handleInsertHyperlink}
        onUpdate={handleUpdateHyperlink}
        onRemove={handleRemoveHyperlink}
        selectedText={selectedTextForLink}
        existingHyperlink={editingHyperlink}
      />

      <BookmarkDialog
        isOpen={bookmarkDialogOpen}
        onClose={handleCloseBookmarkDialog}
        onInsert={handleInsertBookmark}
        onGoTo={handleGoToBookmark}
        onDelete={handleDeleteBookmark}
        docId={document?.id}
      />

      <DocumentStatsDialog
        isOpen={statsDialogOpen}
        onClose={handleCloseStatsDialog}
        documentStats={documentStats}
        selectionStats={selectionStats}
        isCalculating={isCalculating}
      />

      <GoToDialog
        isOpen={goToDialogOpen}
        onClose={handleCloseGoToDialog}
        onGoToPage={handleGoToPage}
        onGoToSection={handleGoToSection}
        onGoToBookmark={handleGoToBookmark}
        currentPage={document?.currentPage ?? 1}
        totalPages={documentStats.pageCount || document?.totalPages || 1}
        totalSections={1}
        docId={document?.id}
      />

      <SymbolDialog
        isOpen={symbolDialogOpen}
        onClose={handleCloseSymbolDialog}
        onInsertSymbol={handleInsertSymbol}
      />

      <StatusBar
        document={document}
        zoom={zoom}
        fitMode={fitMode}
        zoomPercentage={zoomPercentage}
        isAtMin={isAtMin}
        isAtMax={isAtMax}
        onZoomChange={setZoom}
        onZoomIn={zoomIn}
        onZoomOut={zoomOut}
        onResetZoom={resetZoom}
        onFitToWidth={fitToWidth}
        onFitToPage={fitToPage}
        viewMode={viewMode}
        onViewModeChange={handleViewModeChange}
        documentStats={documentStats}
        selectionStats={selectionStats}
        onPageInfoClick={handlePageInfoClick}
        onWordCountClick={handleWordCountClick}
      />
    </div>
  );
}

function App() {
  return (
    <LiveRegionProvider debounceDelay={300} maxHistory={50}>
      <AppContent />
    </LiveRegionProvider>
  );
}

export default App;
