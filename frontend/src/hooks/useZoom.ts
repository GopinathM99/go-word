/**
 * useZoom - Zoom state management for the document editor
 *
 * Features:
 * - Zoom range: 25% to 500% (0.25 to 5.0)
 * - Preset levels: 25%, 50%, 75%, 100%, 125%, 150%, 200%, 400%
 * - "Fit page width" and "Fit whole page" options
 * - Persistent storage keyed by document ID
 * - Keyboard shortcuts support
 */

import { useState, useCallback, useEffect, useMemo } from 'react';

// =============================================================================
// Types
// =============================================================================

export type ZoomFitMode = 'none' | 'fit-width' | 'fit-page';

export interface ZoomState {
  /** Current zoom level (0.25 to 5.0) */
  zoom: number;
  /** Current fit mode */
  fitMode: ZoomFitMode;
  /** Whether zoom is at minimum */
  isAtMin: boolean;
  /** Whether zoom is at maximum */
  isAtMax: boolean;
}

export interface ZoomConfig {
  /** Minimum zoom level (default: 0.25) */
  minZoom: number;
  /** Maximum zoom level (default: 5.0) */
  maxZoom: number;
  /** Step for increment/decrement (default: 0.1) */
  step: number;
  /** Preset zoom levels */
  presets: number[];
}

export interface UseZoomOptions {
  /** Document ID for persistent storage */
  documentId?: string;
  /** Initial zoom level */
  initialZoom?: number;
  /** Page dimensions for fit calculations */
  pageWidth?: number;
  pageHeight?: number;
  /** Container dimensions for fit calculations */
  containerWidth?: number;
  containerHeight?: number;
  /** Callback when zoom changes */
  onZoomChange?: (zoom: number, fitMode: ZoomFitMode) => void;
}

export interface UseZoomReturn extends ZoomState {
  /** Set zoom to a specific level */
  setZoom: (zoom: number) => void;
  /** Zoom in by step */
  zoomIn: () => void;
  /** Zoom out by step */
  zoomOut: () => void;
  /** Reset to 100% */
  resetZoom: () => void;
  /** Set to a preset level */
  setPreset: (preset: number) => void;
  /** Fit to page width */
  fitToWidth: () => void;
  /** Fit whole page */
  fitToPage: () => void;
  /** Handle mouse wheel zoom */
  handleWheelZoom: (deltaY: number, ctrlKey: boolean) => boolean;
  /** Zoom configuration */
  config: ZoomConfig;
  /** Get zoom percentage string (e.g., "100%") */
  zoomPercentage: string;
}

// =============================================================================
// Constants
// =============================================================================

export const ZOOM_PRESETS = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 4.0];
export const MIN_ZOOM = 0.25;
export const MAX_ZOOM = 5.0;
export const ZOOM_STEP = 0.1;

const STORAGE_KEY_PREFIX = 'ms-word-zoom-';

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Clamp a zoom value to valid range
 */
function clampZoom(zoom: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, zoom));
}

/**
 * Round zoom to 2 decimal places
 */
function roundZoom(zoom: number): number {
  return Math.round(zoom * 100) / 100;
}

/**
 * Get storage key for a document
 */
function getStorageKey(documentId: string | undefined): string | null {
  if (!documentId) return null;
  return `${STORAGE_KEY_PREFIX}${documentId}`;
}

/**
 * Load zoom from localStorage
 */
function loadZoomFromStorage(documentId: string | undefined): number | null {
  const key = getStorageKey(documentId);
  if (!key) return null;

  try {
    const stored = localStorage.getItem(key);
    if (stored) {
      const parsed = parseFloat(stored);
      if (!isNaN(parsed) && parsed >= MIN_ZOOM && parsed <= MAX_ZOOM) {
        return parsed;
      }
    }
  } catch {
    // localStorage may not be available
  }

  return null;
}

/**
 * Save zoom to localStorage
 */
function saveZoomToStorage(documentId: string | undefined, zoom: number): void {
  const key = getStorageKey(documentId);
  if (!key) return;

  try {
    localStorage.setItem(key, zoom.toString());
  } catch {
    // localStorage may not be available
  }
}

// =============================================================================
// Hook Implementation
// =============================================================================

export function useZoom(options: UseZoomOptions = {}): UseZoomReturn {
  const {
    documentId,
    initialZoom = 1.0,
    pageWidth = 816, // Default US Letter width in pixels at 96 DPI
    pageHeight = 1056, // Default US Letter height in pixels at 96 DPI
    containerWidth = 800,
    containerHeight = 600,
    onZoomChange,
  } = options;

  // Load initial zoom from storage or use default
  const storedZoom = useMemo(() => loadZoomFromStorage(documentId), [documentId]);

  const [zoom, setZoomState] = useState<number>(() => {
    return storedZoom ?? initialZoom;
  });

  const [fitMode, setFitMode] = useState<ZoomFitMode>('none');

  // Zoom configuration
  const config: ZoomConfig = useMemo(() => ({
    minZoom: MIN_ZOOM,
    maxZoom: MAX_ZOOM,
    step: ZOOM_STEP,
    presets: ZOOM_PRESETS,
  }), []);

  // Derived state
  const isAtMin = zoom <= config.minZoom;
  const isAtMax = zoom >= config.maxZoom;
  const zoomPercentage = `${Math.round(zoom * 100)}%`;

  // Save zoom to storage when it changes
  useEffect(() => {
    saveZoomToStorage(documentId, zoom);
  }, [documentId, zoom]);

  // Notify parent of zoom changes
  useEffect(() => {
    if (onZoomChange) {
      onZoomChange(zoom, fitMode);
    }
  }, [zoom, fitMode, onZoomChange]);

  /**
   * Set zoom to a specific level
   */
  const setZoom = useCallback((newZoom: number) => {
    const clamped = clampZoom(roundZoom(newZoom), config.minZoom, config.maxZoom);
    setZoomState(clamped);
    setFitMode('none');
  }, [config.minZoom, config.maxZoom]);

  /**
   * Zoom in by step
   */
  const zoomIn = useCallback(() => {
    setZoomState(current => {
      const newZoom = clampZoom(roundZoom(current + config.step), config.minZoom, config.maxZoom);
      return newZoom;
    });
    setFitMode('none');
  }, [config.step, config.minZoom, config.maxZoom]);

  /**
   * Zoom out by step
   */
  const zoomOut = useCallback(() => {
    setZoomState(current => {
      const newZoom = clampZoom(roundZoom(current - config.step), config.minZoom, config.maxZoom);
      return newZoom;
    });
    setFitMode('none');
  }, [config.step, config.minZoom, config.maxZoom]);

  /**
   * Reset to 100%
   */
  const resetZoom = useCallback(() => {
    setZoomState(1.0);
    setFitMode('none');
  }, []);

  /**
   * Set to a preset level
   */
  const setPreset = useCallback((preset: number) => {
    if (config.presets.includes(preset)) {
      setZoomState(preset);
      setFitMode('none');
    }
  }, [config.presets]);

  /**
   * Fit to page width
   */
  const fitToWidth = useCallback(() => {
    if (pageWidth > 0 && containerWidth > 0) {
      // Account for some padding (40px on each side)
      const availableWidth = containerWidth - 80;
      const newZoom = clampZoom(
        roundZoom(availableWidth / pageWidth),
        config.minZoom,
        config.maxZoom
      );
      setZoomState(newZoom);
      setFitMode('fit-width');
    }
  }, [pageWidth, containerWidth, config.minZoom, config.maxZoom]);

  /**
   * Fit whole page
   */
  const fitToPage = useCallback(() => {
    if (pageWidth > 0 && pageHeight > 0 && containerWidth > 0 && containerHeight > 0) {
      // Account for padding
      const availableWidth = containerWidth - 80;
      const availableHeight = containerHeight - 80;

      const widthRatio = availableWidth / pageWidth;
      const heightRatio = availableHeight / pageHeight;

      const newZoom = clampZoom(
        roundZoom(Math.min(widthRatio, heightRatio)),
        config.minZoom,
        config.maxZoom
      );
      setZoomState(newZoom);
      setFitMode('fit-page');
    }
  }, [pageWidth, pageHeight, containerWidth, containerHeight, config.minZoom, config.maxZoom]);

  /**
   * Handle mouse wheel zoom (Ctrl/Cmd + scroll)
   * Returns true if the event was handled
   */
  const handleWheelZoom = useCallback((deltaY: number, ctrlKey: boolean): boolean => {
    if (!ctrlKey) {
      return false;
    }

    // Determine zoom direction based on scroll direction
    // Negative deltaY = scroll up = zoom in
    // Positive deltaY = scroll down = zoom out
    const zoomDelta = deltaY < 0 ? config.step : -config.step;

    setZoomState(current => {
      const newZoom = clampZoom(roundZoom(current + zoomDelta), config.minZoom, config.maxZoom);
      return newZoom;
    });
    setFitMode('none');

    return true;
  }, [config.step, config.minZoom, config.maxZoom]);

  return {
    zoom,
    fitMode,
    isAtMin,
    isAtMax,
    setZoom,
    zoomIn,
    zoomOut,
    resetZoom,
    setPreset,
    fitToWidth,
    fitToPage,
    handleWheelZoom,
    config,
    zoomPercentage,
  };
}

// =============================================================================
// Keyboard Shortcut Handler Hook
// =============================================================================

export interface UseZoomShortcutsOptions {
  zoomIn: () => void;
  zoomOut: () => void;
  resetZoom: () => void;
  handleWheelZoom: (deltaY: number, ctrlKey: boolean) => boolean;
  enabled?: boolean;
}

/**
 * Hook to handle zoom keyboard shortcuts
 * - Ctrl/Cmd + Plus: Zoom in 10%
 * - Ctrl/Cmd + Minus: Zoom out 10%
 * - Ctrl/Cmd + 0: Reset to 100%
 */
export function useZoomShortcuts(options: UseZoomShortcutsOptions): void {
  const { zoomIn, zoomOut, resetZoom, handleWheelZoom, enabled = true } = options;

  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modifierKey = isMac ? e.metaKey : e.ctrlKey;

      if (!modifierKey) return;

      // Ctrl/Cmd + Plus or Ctrl/Cmd + =
      if (e.key === '+' || e.key === '=' || e.key === 'Add') {
        e.preventDefault();
        zoomIn();
        return;
      }

      // Ctrl/Cmd + Minus
      if (e.key === '-' || e.key === 'Subtract') {
        e.preventDefault();
        zoomOut();
        return;
      }

      // Ctrl/Cmd + 0
      if (e.key === '0') {
        e.preventDefault();
        resetZoom();
        return;
      }
    };

    const handleWheel = (e: WheelEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modifierKey = isMac ? e.metaKey : e.ctrlKey;

      if (modifierKey && handleWheelZoom(e.deltaY, true)) {
        e.preventDefault();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('wheel', handleWheel, { passive: false });

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('wheel', handleWheel);
    };
  }, [enabled, zoomIn, zoomOut, resetZoom, handleWheelZoom]);
}
