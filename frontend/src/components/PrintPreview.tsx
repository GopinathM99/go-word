/**
 * PrintPreview - Full-screen print preview component
 */

import { useCallback, useRef, useEffect } from 'react';
import { usePrintPreview, PreviewZoomState } from '../hooks/usePrintPreview';
import { PageThumbnails } from './PageThumbnails';
import { PrintPreviewToolbar } from './PrintPreviewToolbar';
import { PrintDialog } from './PrintDialog';
import { useState } from 'react';
import '../styles/PrintPreview.css';

// =============================================================================
// Types
// =============================================================================

export interface PrintPreviewProps {
  documentId: string;
  documentTitle: string;
  isOpen: boolean;
  onClose: () => void;
}

// =============================================================================
// Component
// =============================================================================

export function PrintPreview({
  documentId,
  documentTitle,
  isOpen,
  onClose,
}: PrintPreviewProps) {
  const {
    state,
    previewPages,
    thumbnails,
    openPreview,
    closePreview,
    nextPage,
    prevPage,
    goToPage,
    setZoom,
    zoomIn,
    zoomOut,
  } = usePrintPreview({ documentId, onClose });

  const [showPrintDialog, setShowPrintDialog] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const pageContainerRef = useRef<HTMLDivElement>(null);

  // Open preview when isOpen changes
  useEffect(() => {
    if (isOpen && !state.isOpen) {
      openPreview();
    } else if (!isOpen && state.isOpen) {
      closePreview();
    }
  }, [isOpen, state.isOpen, openPreview, closePreview]);

  // Calculate zoom scale
  const getZoomScale = useCallback((zoom: PreviewZoomState): number => {
    if (zoom.mode === 'percentage') {
      return zoom.percentage / 100;
    }
    // For fit modes, return 1 and let CSS handle it
    return 1;
  }, []);

  // Handle print
  const handlePrint = useCallback(async (options: any) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('print_document', {
        docId: documentId,
        options,
      });
      setShowPrintDialog(false);
    } catch (err) {
      console.error('Print failed:', err);
    }
  }, [documentId]);

  // Get current page image
  const currentPageImage = previewPages.get(state.currentPage);
  const zoomScale = getZoomScale(state.zoom);

  if (!state.isOpen) {
    return null;
  }

  return (
    <div className="print-preview" role="dialog" aria-label="Print Preview">
      {/* Toolbar */}
      <PrintPreviewToolbar
        currentPage={state.currentPage}
        totalPages={state.totalPages}
        zoom={state.zoom}
        onPrevPage={prevPage}
        onNextPage={nextPage}
        onGoToPage={goToPage}
        onZoomChange={setZoom}
        onZoomIn={zoomIn}
        onZoomOut={zoomOut}
        onPrint={() => setShowPrintDialog(true)}
        onClose={closePreview}
      />

      {/* Main content */}
      <div className="print-preview-content">
        {/* Sidebar with thumbnails */}
        <div className={`print-preview-sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
          <PageThumbnails
            totalPages={state.totalPages}
            currentPage={state.currentPage}
            onPageSelect={goToPage}
            thumbnailData={thumbnails}
            isLoading={state.isLoading}
          />
        </div>

        {/* Sidebar toggle */}
        <button
          className="print-preview-sidebar-toggle"
          onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
          aria-label={sidebarCollapsed ? 'Show thumbnails' : 'Hide thumbnails'}
          style={{ left: sidebarCollapsed ? 0 : 'var(--print-preview-sidebar-width)' }}
        >
          {sidebarCollapsed ? '›' : '‹'}
        </button>

        {/* Page display area */}
        <div className="print-preview-pages" ref={pageContainerRef}>
          {state.isLoading ? (
            <div className="print-preview-loading">
              <div className="print-preview-spinner" />
              <span>Loading preview...</span>
            </div>
          ) : state.error ? (
            <div className="print-preview-error">
              <span>Error: {state.error}</span>
              <button onClick={() => openPreview()}>Retry</button>
            </div>
          ) : (
            <div className="print-preview-page-container">
              <div
                className={`print-preview-page ${state.zoom.mode}`}
                style={{
                  transform: state.zoom.mode === 'percentage' ? `scale(${zoomScale})` : undefined,
                }}
              >
                {currentPageImage ? (
                  <img
                    src={`data:image/png;base64,${currentPageImage}`}
                    alt={`Page ${state.currentPage}`}
                    draggable={false}
                  />
                ) : (
                  <div className="print-preview-page-loading">
                    <div className="print-preview-spinner" />
                  </div>
                )}
              </div>
              <div className="print-preview-page-number">
                Page {state.currentPage} of {state.totalPages}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Print Dialog */}
      <PrintDialog
        isOpen={showPrintDialog}
        onClose={() => setShowPrintDialog(false)}
        onPrint={handlePrint}
        totalPages={state.totalPages}
        currentPage={state.currentPage}
        documentTitle={documentTitle}
      />
    </div>
  );
}

export default PrintPreview;
