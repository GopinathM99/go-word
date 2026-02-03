/**
 * PrintPreviewToolbar - Toolbar for print preview navigation and controls
 */

import { useCallback, useState } from 'react';
import { PreviewZoomState, PreviewZoomMode } from '../hooks/usePrintPreview';

// =============================================================================
// Types
// =============================================================================

export interface PrintPreviewToolbarProps {
  currentPage: number;
  totalPages: number;
  zoom: PreviewZoomState;
  onPrevPage: () => void;
  onNextPage: () => void;
  onGoToPage: (page: number) => void;
  onZoomChange: (zoom: PreviewZoomState) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onPrint: () => void;
  onClose: () => void;
}

// =============================================================================
// Constants
// =============================================================================

const ZOOM_OPTIONS: Array<{ label: string; value: PreviewZoomState }> = [
  { label: 'Fit Width', value: { mode: 'fit-width', percentage: 100 } },
  { label: 'Fit Page', value: { mode: 'fit-page', percentage: 100 } },
  { label: '50%', value: { mode: 'percentage', percentage: 50 } },
  { label: '75%', value: { mode: 'percentage', percentage: 75 } },
  { label: '100%', value: { mode: 'percentage', percentage: 100 } },
  { label: '125%', value: { mode: 'percentage', percentage: 125 } },
  { label: '150%', value: { mode: 'percentage', percentage: 150 } },
  { label: '200%', value: { mode: 'percentage', percentage: 200 } },
];

// =============================================================================
// Component
// =============================================================================

export function PrintPreviewToolbar({
  currentPage,
  totalPages,
  zoom,
  onPrevPage,
  onNextPage,
  onGoToPage,
  onZoomChange,
  onZoomIn,
  onZoomOut,
  onPrint,
  onClose,
}: PrintPreviewToolbarProps) {
  const [pageInput, setPageInput] = useState(String(currentPage));

  // Update input when page changes externally
  if (String(currentPage) !== pageInput && document.activeElement?.tagName !== 'INPUT') {
    setPageInput(String(currentPage));
  }

  const handlePageInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setPageInput(e.target.value);
  }, []);

  const handlePageInputBlur = useCallback(() => {
    const page = parseInt(pageInput, 10);
    if (!isNaN(page) && page >= 1 && page <= totalPages) {
      onGoToPage(page);
    } else {
      setPageInput(String(currentPage));
    }
  }, [pageInput, currentPage, totalPages, onGoToPage]);

  const handlePageInputKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handlePageInputBlur();
    }
  }, [handlePageInputBlur]);

  const getZoomLabel = useCallback((z: PreviewZoomState): string => {
    if (z.mode === 'fit-width') return 'Fit Width';
    if (z.mode === 'fit-page') return 'Fit Page';
    return `${z.percentage}%`;
  }, []);

  const handleZoomSelect = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const idx = parseInt(e.target.value, 10);
    if (idx >= 0 && idx < ZOOM_OPTIONS.length) {
      onZoomChange(ZOOM_OPTIONS[idx].value);
    }
  }, [onZoomChange]);

  const getCurrentZoomIndex = useCallback((): number => {
    return ZOOM_OPTIONS.findIndex(opt =>
      opt.value.mode === zoom.mode &&
      (zoom.mode !== 'percentage' || opt.value.percentage === zoom.percentage)
    );
  }, [zoom]);

  return (
    <div className="print-preview-toolbar">
      {/* Left section - Close */}
      <div className="print-preview-toolbar-left">
        <button
          className="print-preview-btn print-preview-btn-close"
          onClick={onClose}
          aria-label="Close preview"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
          </svg>
          <span>Close</span>
        </button>
      </div>

      {/* Center section - Navigation and Zoom */}
      <div className="print-preview-toolbar-center">
        {/* Page navigation */}
        <div className="print-preview-nav">
          <button
            className="print-preview-nav-btn"
            onClick={onPrevPage}
            disabled={currentPage <= 1}
            aria-label="Previous page"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path fillRule="evenodd" d="M11.354 1.646a.5.5 0 0 1 0 .708L5.707 8l5.647 5.646a.5.5 0 0 1-.708.708l-6-6a.5.5 0 0 1 0-.708l6-6a.5.5 0 0 1 .708 0z"/>
            </svg>
          </button>

          <div className="print-preview-page-input">
            <input
              type="text"
              value={pageInput}
              onChange={handlePageInputChange}
              onBlur={handlePageInputBlur}
              onKeyDown={handlePageInputKeyDown}
              aria-label="Page number"
            />
            <span>of {totalPages}</span>
          </div>

          <button
            className="print-preview-nav-btn"
            onClick={onNextPage}
            disabled={currentPage >= totalPages}
            aria-label="Next page"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path fillRule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/>
            </svg>
          </button>
        </div>

        <div className="print-preview-divider" />

        {/* Zoom controls */}
        <div className="print-preview-zoom">
          <button
            className="print-preview-nav-btn"
            onClick={onZoomOut}
            aria-label="Zoom out"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4 8a.5.5 0 0 1 .5-.5h7a.5.5 0 0 1 0 1h-7A.5.5 0 0 1 4 8z"/>
            </svg>
          </button>

          <select
            className="print-preview-zoom-select"
            value={getCurrentZoomIndex()}
            onChange={handleZoomSelect}
            aria-label="Zoom level"
          >
            {ZOOM_OPTIONS.map((opt, idx) => (
              <option key={idx} value={idx}>{opt.label}</option>
            ))}
          </select>

          <button
            className="print-preview-nav-btn"
            onClick={onZoomIn}
            aria-label="Zoom in"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
            </svg>
          </button>
        </div>
      </div>

      {/* Right section - Print */}
      <div className="print-preview-toolbar-right">
        <button
          className="print-preview-btn print-preview-btn-print"
          onClick={onPrint}
          aria-label="Print document"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2.5 8a.5.5 0 1 0 0-1 .5.5 0 0 0 0 1z"/>
            <path d="M5 1a2 2 0 0 0-2 2v2H2a2 2 0 0 0-2 2v3a2 2 0 0 0 2 2h1v1a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2v-1h1a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-1V3a2 2 0 0 0-2-2H5zM4 3a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2H4V3zm1 5a2 2 0 0 0-2 2v1H2a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v-1a2 2 0 0 0-2-2H5zm7 2v3a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1z"/>
          </svg>
          <span>Print</span>
        </button>
      </div>
    </div>
  );
}

export default PrintPreviewToolbar;
