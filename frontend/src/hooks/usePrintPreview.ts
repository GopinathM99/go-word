/**
 * usePrintPreview - State management hook for print preview functionality
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

// =============================================================================
// Types
// =============================================================================

export type PreviewZoomMode = 'fit-width' | 'fit-page' | 'percentage';

export interface PreviewZoomState {
  mode: PreviewZoomMode;
  percentage: number;
}

export interface PrintPreviewState {
  isOpen: boolean;
  currentPage: number;
  totalPages: number;
  zoom: PreviewZoomState;
  isLoading: boolean;
  error: string | null;
}

export interface UsePrintPreviewOptions {
  documentId: string;
  onClose?: () => void;
}

export interface UsePrintPreviewResult {
  state: PrintPreviewState;
  previewPages: Map<number, string>;
  thumbnails: string[];
  openPreview: () => Promise<void>;
  closePreview: () => void;
  nextPage: () => void;
  prevPage: () => void;
  goToPage: (page: number) => void;
  setZoom: (zoom: PreviewZoomState) => void;
  zoomIn: () => void;
  zoomOut: () => void;
  refreshPage: (page: number) => Promise<void>;
}

// =============================================================================
// Constants
// =============================================================================

const ZOOM_PRESETS = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0];
const MIN_ZOOM = 0.25;
const MAX_ZOOM = 4.0;
const ZOOM_STEP = 0.25;

// =============================================================================
// Hook Implementation
// =============================================================================

export function usePrintPreview(options: UsePrintPreviewOptions): UsePrintPreviewResult {
  const { documentId, onClose } = options;

  const [state, setState] = useState<PrintPreviewState>({
    isOpen: false,
    currentPage: 1,
    totalPages: 0,
    zoom: { mode: 'fit-width', percentage: 100 },
    isLoading: false,
    error: null,
  });

  const [previewPages, setPreviewPages] = useState<Map<number, string>>(new Map());
  const [thumbnails, setThumbnails] = useState<string[]>([]);
  const loadingRef = useRef<Set<number>>(new Set());

  // Load page preview
  const loadPage = useCallback(async (pageNumber: number, scale: number = 1.5) => {
    if (loadingRef.current.has(pageNumber)) return;

    loadingRef.current.add(pageNumber);

    try {
      const preview = await invoke<string>('render_preview_page', {
        docId: documentId,
        pageNumber: pageNumber - 1, // 0-indexed
        scale,
      });

      setPreviewPages(prev => new Map(prev).set(pageNumber, preview));
    } catch (err) {
      console.error(`Failed to load page ${pageNumber}:`, err);
    } finally {
      loadingRef.current.delete(pageNumber);
    }
  }, [documentId]);

  // Load thumbnails
  const loadThumbnails = useCallback(async (totalPages: number) => {
    try {
      const thumbs = await invoke<string[]>('render_preview_thumbnails', {
        docId: documentId,
        startPage: 0,
        count: totalPages,
      });
      setThumbnails(thumbs);
    } catch (err) {
      console.error('Failed to load thumbnails:', err);
    }
  }, [documentId]);

  // Open preview
  const openPreview = useCallback(async () => {
    setState(prev => ({ ...prev, isOpen: true, isLoading: true, error: null }));

    try {
      // Get document info for total pages
      const docInfo = await invoke<{ pageCount: number }>('get_document_info', {
        docId: documentId,
      });

      const totalPages = docInfo.pageCount || 1;

      setState(prev => ({
        ...prev,
        totalPages,
        currentPage: 1,
        isLoading: false,
      }));

      // Load first page and thumbnails
      await Promise.all([
        loadPage(1),
        loadThumbnails(totalPages),
      ]);
    } catch (err) {
      setState(prev => ({
        ...prev,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to open preview',
      }));
    }
  }, [documentId, loadPage, loadThumbnails]);

  // Close preview
  const closePreview = useCallback(() => {
    setState(prev => ({ ...prev, isOpen: false }));
    setPreviewPages(new Map());
    setThumbnails([]);
    loadingRef.current.clear();
    onClose?.();
  }, [onClose]);

  // Navigation
  const nextPage = useCallback(() => {
    setState(prev => {
      if (prev.currentPage >= prev.totalPages) return prev;
      const newPage = prev.currentPage + 1;
      loadPage(newPage);
      return { ...prev, currentPage: newPage };
    });
  }, [loadPage]);

  const prevPage = useCallback(() => {
    setState(prev => {
      if (prev.currentPage <= 1) return prev;
      const newPage = prev.currentPage - 1;
      loadPage(newPage);
      return { ...prev, currentPage: newPage };
    });
  }, [loadPage]);

  const goToPage = useCallback((page: number) => {
    setState(prev => {
      const newPage = Math.max(1, Math.min(page, prev.totalPages));
      if (newPage === prev.currentPage) return prev;
      loadPage(newPage);
      return { ...prev, currentPage: newPage };
    });
  }, [loadPage]);

  // Zoom controls
  const setZoom = useCallback((zoom: PreviewZoomState) => {
    setState(prev => ({ ...prev, zoom }));
  }, []);

  const zoomIn = useCallback(() => {
    setState(prev => {
      const currentPct = prev.zoom.percentage;
      const newPct = Math.min(MAX_ZOOM * 100, currentPct + ZOOM_STEP * 100);
      return {
        ...prev,
        zoom: { mode: 'percentage', percentage: newPct },
      };
    });
  }, []);

  const zoomOut = useCallback(() => {
    setState(prev => {
      const currentPct = prev.zoom.percentage;
      const newPct = Math.max(MIN_ZOOM * 100, currentPct - ZOOM_STEP * 100);
      return {
        ...prev,
        zoom: { mode: 'percentage', percentage: newPct },
      };
    });
  }, []);

  // Refresh page
  const refreshPage = useCallback(async (page: number) => {
    setPreviewPages(prev => {
      const next = new Map(prev);
      next.delete(page);
      return next;
    });
    await loadPage(page);
  }, [loadPage]);

  // Preload adjacent pages when current page changes
  useEffect(() => {
    if (!state.isOpen || state.totalPages === 0) return;

    const pagesToLoad = [
      state.currentPage - 1,
      state.currentPage,
      state.currentPage + 1,
    ].filter(p => p >= 1 && p <= state.totalPages && !previewPages.has(p));

    pagesToLoad.forEach(p => loadPage(p));
  }, [state.currentPage, state.isOpen, state.totalPages, previewPages, loadPage]);

  // Keyboard navigation
  useEffect(() => {
    if (!state.isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          closePreview();
          break;
        case 'ArrowLeft':
        case 'PageUp':
          prevPage();
          break;
        case 'ArrowRight':
        case 'PageDown':
          nextPage();
          break;
        case 'Home':
          goToPage(1);
          break;
        case 'End':
          goToPage(state.totalPages);
          break;
        case '+':
        case '=':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            zoomIn();
          }
          break;
        case '-':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            zoomOut();
          }
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.isOpen, state.totalPages, closePreview, prevPage, nextPage, goToPage, zoomIn, zoomOut]);

  return {
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
    refreshPage,
  };
}

export default usePrintPreview;
